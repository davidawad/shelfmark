#!/usr/bin/env python3
"""Rescore the published runs with the false-join term (crates/bench/src/falsejoin.rs).

Writes bench/results/2026-10-11-falsejoin/ and never touches older result
directories. Steps, each skipped when its output exists (unless --force):

  negatives  One seeded kNN-complement negative sample per graph (books at
             k = 10, 20, 50 and papers and the title-only holdout at k = 20,
             for MiniLM, bge-small and nomic) with `shelfmark-bench
             negatives`: 10 pairs started per item, floor = the graph's tau,
             seed 20261010. Needs the vectors of build_graph_fastembed.py
             --vectors in bench/corpus/cache/falsejoin/vectors/
             (<model>-<corpus>.f32); the samples are committed, so later steps
             do not need the vectors.
  bootstrap  One `shelfmark-bench bootstrap --negatives` call per graph: every
             published run of that universe (the 2026-10-11-metrics runs plus
             report 9's runs), 1,000 paired item draws, seed 20261010. Every
             run's point cost_norm must equal its published value to 1e-9.
  publish    Per-run records (point and 95% CI of cost_norm, fj_norm, fjs_norm
             and both combined objectives at every beta), results.json and
             results.md.

    python3 -I crates/bench/scripts/falsejoin_rescore.py all --harness <shelfmark-bench>
"""
import argparse
import gzip
import hashlib
import importlib.util
import json
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = ROOT / "bench/results/2026-10-11-falsejoin"
METRICS = ROOT / "bench/results/2026-10-11-metrics"
PROPOSAL = ROOT / "bench/results/2026-10-10-proposal"
DDC_R = ROOT / "bench/results/2026-10-10-ddc"
EMB_R = ROOT / "bench/results/2026-10-10-embed"
VECTORS = ROOT / "bench/corpus/cache/falsejoin/vectors"
SCHEME_CACHE = ROOT / "bench/corpus/cache/schemes"
SEED = 20261010
PER_ITEM = 10
MODELS = ["minilm", "bge-small", "nomic"]
CORPORA = {"books": "bench/corpus/holdout-books.jsonl", "holdout": "bench/corpus/holdout.jsonl",
           "corpus": "bench/corpus/corpus.jsonl"}
VECTOR_NAME = {"books": "holdout-books", "holdout": "holdout", "corpus": "corpus"}
CONDITIONS = ([("books", m, k) for m in MODELS for k in (10, 20, 50)]
              + [(u, m, 20) for u in ("corpus", "holdout") for m in MODELS])
DRAW_METRICS = "cost_norm,fj_norm,fjs_norm,nav_log_a1,coverage,mean_depth"
BETAS = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0]
PRIMARY_BETA = 1.0
TOLERANCE = 1e-9

_spec = importlib.util.spec_from_file_location("kb_proposal_runs",
                                               ROOT / "research/reports/scripts/kb_proposal_runs.py")
kb = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(kb)


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write_gz(path, data):
    """Reproducible gzip: no file name or time stamp in the header."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(gzip.compress(data, compresslevel=9, mtime=0))


def cond_name(universe, model, k):
    return f"{universe}-{model}-k{k}"


def rel(path):
    return str(Path(path).resolve().relative_to(ROOT))


def graph_file(universe, model, k, tmp):
    """The published graph of a condition, decompressed when needed."""
    if k == 20:
        name = f"{model}-books-k20.json" if universe == "books" else f"{model}-{universe}-k20.json"
        return METRICS / ("books" if universe == "books" else "embed") / "graphs" / name
    src = PROPOSAL / f"graphs/{model}-books-k{k}.json.gz"
    dst = Path(tmp) / src.name.removesuffix(".gz")
    dst.write_bytes(gzip.decompress(src.read_bytes()))
    return dst


def scheme_index():
    """Scheme file by sha256: committed schemes, result-directory schemes and the
    fetch-and-build schemes of build_schemes.sh in the ignored cache."""
    index = {}
    roots = [ROOT / "schemes", ROOT / "data/schemes", SCHEME_CACHE, DDC_R / "schemes", EMB_R / "refit",
             EMB_R / "placements", PROPOSAL / "schemes", ROOT / "bench/results/2026-10-10-learned"]
    for base in roots:
        for path in sorted(base.rglob("*.json")) if base.exists() else []:
            if path.stat().st_size > 300_000_000 or "runs" in path.parts:
                continue
            index.setdefault(sha256(path), path)
    return index


def metrics_runs(universe, model):
    """The 2026-10-11-metrics records of one universe and k = 20 graph, by run key."""
    if universe == "books":
        base = METRICS / f"books/runs/{model}-k20"
    else:
        base = METRICS / f"embed/runs/{model}-k20/{universe}"
    return {p.stem: json.loads(p.read_text()) for p in sorted(base.glob("*.json"))}


def placements_for(universe, key):
    if universe == "books":
        for base in (PROPOSAL / "placements/books", DDC_R / "placements/books"):
            if (base / f"{key}.jsonl").exists():
                return base / f"{key}.jsonl"
    else:
        for path in (EMB_R / f"placements/{universe}-{key}.jsonl", DDC_R / f"placements/{universe}/{key}.jsonl",
                     PROPOSAL / f"placements/{universe}/{key}.jsonl"):
            if path.exists():
                return path
    raise SystemExit(f"{universe} {key}: no committed placements")


def run_specs(universe, model, k, index):
    """Report 9's runs of the universe, plus every published 2026-10-11-metrics run
    of the k = 20 graph; the published cost_norm to check against, by key."""
    specs = {r["key"]: r for r in kb.run_specs(universe)}
    expected = {}
    if k == 20:
        for key, rec in metrics_runs(universe, model).items():
            expected[key] = rec["cost_norm"]
            if key in specs:
                continue
            scheme = index.get(rec["hashes"]["scheme"])
            if scheme is None:
                raise SystemExit(f"{universe} {model} {key}: scheme {rec['scheme']} {rec['version']} not found; "
                                 "run crates/bench/scripts/build_schemes.sh")
            specs[key] = {"key": key, "scheme": str(scheme), "placements": str(placements_for(universe, key))}
    prior = PROPOSAL / f"bootstrap/{cond_name(universe, model, k)}.json"
    if prior.exists():
        for key, run in json.loads(prior.read_text())["runs"].items():
            expected.setdefault(key, run["point"]["cost_norm"])
    return list(specs.values()), expected


def scheme_ref(path):
    """Repository path of a scheme file; fetch-and-build schemes are named by
    their build_schemes.sh output, which is never committed."""
    path = Path(path).resolve()
    if path.is_relative_to(SCHEME_CACHE.resolve()):
        return f"build_schemes.sh:{path.name}"
    return rel(path)


def negatives(a):
    with tempfile.TemporaryDirectory() as tmp:
        for universe, model, k in CONDITIONS:
            name = cond_name(universe, model, k)
            out = OUT / f"negatives/{name}.json.gz"
            if out.exists() and not a.force:
                continue
            graph = graph_file(universe, model, k, tmp)
            vectors = VECTORS / f"{model}-{VECTOR_NAME[universe]}.f32"
            if not vectors.exists():
                print(f"negatives {name}: skipped, no vectors at {rel(vectors)}", flush=True)
                continue
            raw = Path(tmp) / f"{name}.json"
            res = subprocess.run([a.harness, "negatives", "--graph", graph, "--vectors", vectors,
                                  "--per-item", str(PER_ITEM), "--seed", str(SEED), "--output", raw],
                                 cwd=ROOT, check=True, capture_output=True, text=True)
            manifest = json.loads(res.stdout)
            manifest.update({"graph": rel(graph) if k == 20 else rel(PROPOSAL / f"graphs/{model}-books-k{k}.json.gz"),
                             "corpus": CORPORA[universe], "vectors": rel(vectors),
                             "vectors_rule": "build_graph_fastembed.py --vectors on this corpus and model "
                                             "(nomic: --max-length 512)"})
            write_gz(out, raw.read_bytes())
            out.with_name(f"{name}.manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
            print(f"negatives {name}: {manifest['pairs']} pairs", flush=True)


def bootstrap_one(a, universe, model, k, index):
    name = cond_name(universe, model, k)
    out = OUT / f"bootstrap/{name}.json.gz"
    if out.exists() and not a.force:
        return name, "kept"
    if not (OUT / f"negatives/{name}.json.gz").exists():
        return name, "skipped, no negative sample yet"
    specs, expected = run_specs(universe, model, k, index)
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        graph = graph_file(universe, model, k, tmp)
        neg = tmp / "negatives.json"
        neg.write_bytes(gzip.decompress((OUT / f"negatives/{name}.json.gz").read_bytes()))
        (tmp / "runs.json").write_text(json.dumps(specs))
        subprocess.run([a.harness, "bootstrap", "--corpus", CORPORA[universe], "--graph", graph,
                        "--runs", tmp / "runs.json", "--negatives", neg, "--resamples", "1000",
                        "--seed", str(SEED), "--threads", str(a.threads), "--output", tmp / "boot.json",
                        "--draws-out", tmp / "draws.json", "--draw-metrics", DRAW_METRICS],
                       cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
        result = json.loads((tmp / "boot.json").read_text())
        checked = 0
        for key, value in expected.items():
            if key not in result["runs"]:
                continue
            got = result["runs"][key]["point"]["cost_norm"]
            if abs(got - value) > TOLERANCE:
                raise SystemExit(f"{name} {key}: cost_norm {got} differs from the published {value}")
            checked += 1
        result["cost_norm_checked"] = {"runs": checked, "tolerance": TOLERANCE,
                                       "against": "2026-10-11-metrics run records and 2026-10-10-proposal bootstrap"}
        result["runs_spec"] = [{**r, "scheme": scheme_ref(r["scheme"]), "placements": rel(r["placements"])}
                               for r in specs]
        result["condition"] = {"universe": universe, "model": model, "k": k}
        result["negatives_file"] = rel(OUT / f"negatives/{name}.json.gz")
        write_gz(out, json.dumps(result, separators=(",", ":"), sort_keys=True).encode())
        write_gz(OUT / f"bootstrap/{name}.draws.json.gz", (tmp / "draws.json").read_bytes())
    return name, f"{len(specs)} runs, {checked} cost_norm values checked"


def bootstrap(a):
    index = scheme_index()
    with ThreadPoolExecutor(max_workers=a.jobs) as pool:
        futures = [pool.submit(bootstrap_one, a, u, m, k, index) for u, m, k in CONDITIONS
                   if not a.universe or u == a.universe]
        for f in futures:
            name, status = f.result()
            print(f"bootstrap {name}: {status}", flush=True)


def load(name):
    return json.loads(gzip.decompress((OUT / f"bootstrap/{name}.json.gz").read_bytes()))


SCORES = ["cost_norm", "fj_norm", "fjs_norm"] + [f"cfj_b{b:g}" for b in BETAS] + [f"cfjs_b{b:g}" for b in BETAS]


def fmt(v, d=4):
    return f"{v:.{d}f}"


def publish(_a):
    records = {}
    for universe, model, k in CONDITIONS:
        name = cond_name(universe, model, k)
        res = load(name)
        for key, run in res["runs"].items():
            rec = {"condition": res["condition"], "key": key, "scheme": run["scheme"], "version": run["version"],
                   "scheme_sha256": run["scheme_sha256"], "placements_sha256": run["placements_sha256"],
                   "graph_sha256": res["graph_sha256"], "negatives": res["negatives"],
                   "point": {m: run["point"][m] for m in SCORES + ["coverage", "nav_log_a1", "skill"]},
                   "ci": {m: run["ci"][m] for m in SCORES + ["coverage", "nav_log_a1", "skill"]},
                   "bootstrap": {"resamples": res["resamples"], "seed": res["seed"], "method": res["method"]}}
            path = OUT / f"runs/{model}-k{k}/{universe}/{key}.json"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(rec, indent=1, sort_keys=True) + "\n")
            records[f"{name}/{key}"] = {"point": rec["point"], "ci": rec["ci"]}
    summary = {"date": "2026-10-11", "conditions": [cond_name(*c) for c in CONDITIONS], "runs": records,
               "negative_samples": {cond_name(*c): json.loads((OUT / f"negatives/{cond_name(*c)}.manifest.json")
                                                             .read_text()) for c in CONDITIONS},
               "primary_beta": PRIMARY_BETA, "betas": BETAS}
    (OUT / "results.json").write_text(json.dumps(summary, indent=1, sort_keys=True) + "\n")
    L = ["# False-join rescoring results", "",
         "Generated by `crates/bench/scripts/falsejoin_rescore.py publish`. Method: `README.md`. "
         "Brackets are 95% paired item-bootstrap intervals (1,000 draws, seed 20261010). "
         "C = cost_norm; FJ = fj_norm (outside mass); FJs = fjs_norm (specificity); "
         "J = C + β·FJ at β = 1. Lower is better for every score.", ""]
    for universe, model, k in CONDITIONS:
        name = cond_name(universe, model, k)
        res = load(name)
        L += [f"## {universe}, {model}, k = {k}", "",
              f"{len(res['runs'])} runs; {res['negatives']['pairs']} negative pairs; "
              f"cost_norm reproduced for {res['cost_norm_checked']['runs']} published runs.", "",
              "| run | coverage | C | FJ | FJs | C + FJ | C + FJs |", "| --- | ---: | ---: | ---: | ---: | ---: | ---: |"]
        for key, run in sorted(res["runs"].items(), key=lambda kv: kv[1]["point"]["cfjs_b1"]):
            p, c = run["point"], run["ci"]
            cells = [f"{fmt(p[m])} [{fmt(c[m][0])}, {fmt(c[m][1])}]" for m in ("cost_norm", "fj_norm", "fjs_norm")]
            L.append(f"| `{key}` | {p['coverage']:.3f} | " + " | ".join(cells)
                     + f" | {fmt(p['cfj_b1'])} | {fmt(p['cfjs_b1'])} |")
        L.append("")
    (OUT / "results.md").write_text("\n".join(L))
    print(f"published {len(records)} run records")


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("step", choices=["negatives", "bootstrap", "publish", "all"])
    p.add_argument("--harness", default="/cargo-target/release/shelfmark-bench")
    p.add_argument("--threads", type=int, default=6)
    p.add_argument("--jobs", type=int, default=2, help="concurrent scoring jobs (at most 2)")
    p.add_argument("--universe", choices=list(CORPORA))
    p.add_argument("--force", action="store_true")
    a = p.parse_args()
    a.jobs = min(a.jobs, 2)
    for step in (["negatives", "bootstrap", "publish"] if a.step == "all" else [a.step]):
        globals()[step](a)


if __name__ == "__main__":
    main()
