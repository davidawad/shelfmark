#!/usr/bin/env python3
"""Derived placements and paired bootstraps for report 9.

Everything here is computed from committed inputs (the 2026-10-10-ddc and
2026-10-10-embed placements, the committed scheme files and graphs, and the
work-level numbers of `ol_work_numbers.py`) plus the books graphs this
script's `graphs` step builds. Outputs go to
`bench/results/2026-10-10-proposal/`:

- `placements/books/`: FDC from the librarians' Dewey numbers cut to FDC
  depth 1-3 (`fdc-d{1,2,3}`) and rolled up to classes holding at least m
  books (`fdc-m{5,10,25,50}`); Dewey cut to depth 1, 2 and 4
  (`ddc-d{1,2,4}`; depth 3 is `ddc-section`); and the availability
  pipelines `fdc-pipe-NN` / `lcc-pipe-NN`: the librarian number for a
  seeded NN% of books (the same books for both schemes), the lexical@2
  classifier for the rest.
- `placements/holdout/`: `fdc-work`, the work-level consensus Dewey number
  mapped to FDC (the edition's own number first), `fdc-pipeline`
  (`fdc-work`, else the classifier) and `lcc-pipeline` (gold LCC, else the
  classifier).
- `placements/corpus/` and `schemes/fdc-arxiv.json`: papers shelved in FDC
  through the arXiv crosswalk below, at the FDC class (`fdc-arxiv`) or with
  the arXiv category nested under it (`fdc-arxiv-nested`).
- `scores/`: the scoring catalogue (report 6) of the main books runs, and of
  FDC and LCC on the holdout books both place, with 1,000 split-stream
  replicates of the cost, triplet, locality and TSD bootstraps.
- `bootstrap/<universe>-<model>-k<k>.json` and `.draws.json.gz`: one
  `shelfmark-bench bootstrap` call per universe and graph (1,000 paired
  item draws, seed 20261010), with every draw's cost_norm, nav_log_a1,
  coverage and mean_depth.

    python3 -I research/reports/scripts/kb_proposal_runs.py all --harness <shelfmark-bench>
"""
import argparse
import gzip
import hashlib
import importlib.util
import json
import shutil
import subprocess
import tempfile
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "bench/results/2026-10-10-proposal"
DDC_R = ROOT / "bench/results/2026-10-10-ddc"
EMB_R = ROOT / "bench/results/2026-10-10-embed"
CACHE = ROOT / "bench/corpus/cache/proposal"
SEED = 20261010
MODELS = ["minilm", "bge-small", "nomic"]
KS = [10, 20, 50]
PIPE_SHARES = [20, 40, 60, 80, 90]
ROLLUPS = [5, 10, 25, 50]
NESTED = [1, 2, 3]

_spec = importlib.util.spec_from_file_location("ddc_numbers", ROOT / "crates/bench/scripts/ddc_numbers.py")
ddc = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(ddc)

SCHEMES = {
    "fdc": ROOT / "schemes/fdc/fdc.json",
    "lcc": ROOT / "data/schemes/lcc/lcc.json",
    "ddc": DDC_R / "schemes/ddc.json",
    "unesco-thesaurus": ROOT / "schemes/unesco-thesaurus/unesco-thesaurus.json",
    "learned": ROOT / "schemes/learned/learned.json",
    "learned-books": DDC_R / "schemes/learned-books.json",
    "learned-holdout": EMB_R / "placements/learned-holdout.json",
    "arxiv": ROOT / "schemes/arxiv/arxiv.json",
    "openalex-topics": ROOT / "schemes/openalex-topics/openalex-topics.json",
    "fdc-arxiv": OUT / "schemes/fdc-arxiv.json",
    "lcc-arxiv": OUT / "schemes/lcc-arxiv.json",
    **{f"fdc-learned{n}": OUT / f"schemes/fdc-learned{n}.json" for n in (1, 2, 3)},
}

# arXiv archive (or category) -> FDC class. Written for this report from the
# FDC captions (CC0); a category entry overrides its archive.
ARXIV_FDC = {
    "cs": "004", "cmp-lg": "004", "econ": "330", "eess": "621.3", "q-fin": "332", "stat": "519",
    "q-bio": "570", "adap-org": "570", "math": "510", "alg-geom": "516", "dg-ga": "516",
    "funct-an": "515", "q-alg": "512", "bayes-an": "519", "astro-ph": "520", "physics": "530",
    "cond-mat": "530", "gr-qc": "530", "quant-ph": "530", "math-ph": "530", "nlin": "530",
    "chao-dyn": "530", "patt-sol": "530", "solv-int": "530", "comp-gas": "530", "mtrl-th": "530",
    "plasm-ph": "530", "hep-ex": "539", "hep-lat": "539", "hep-ph": "539", "hep-th": "539",
    "nucl-ex": "539", "nucl-th": "539", "atom-ph": "539", "acc-phys": "539", "supr-con": "537",
    "chem-ph": "540", "ao-sci": "550",
    "math.PR": "519", "math.ST": "519", "math.AG": "516", "math.GT": "514", "math.AT": "514",
    "math.CA": "515", "math.FA": "515", "math.AP": "515", "math.CV": "515", "math.NT": "512",
    "math.RA": "512", "math.AC": "512", "math.GR": "512", "math.RT": "512", "math.KT": "512",
    "math.MG": "513", "cond-mat.supr-con": "537", "physics.optics": "535",
    "physics.chem-ph": "540", "physics.ao-ph": "550", "physics.geo-ph": "550",
    "physics.bio-ph": "570", "physics.med-ph": "610",
}

# The same crosswalk into LCC, class for class, so papers can be compared
# in both candidate backbones.
FDC_LCC = {
    "004": "QA75.5-76.95", "330": "HB", "621.3": "TK", "332": "HG", "519": "QA273-280",
    "570": "QH301-705.5", "510": "QA", "516": "QA440-699", "515": "QA299.6-433", "512": "QA150-272.5",
    "514": "QA440-699", "513": "QA440-699", "520": "QB", "530": "QC", "539": "QC770-798",
    "537": "QC501-766", "535": "QC350-467", "540": "QD", "550": "QE", "610": "R",
}


def jsonl(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def write_jsonl(path, rows):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("".join(json.dumps(r) + "\n" for r in rows), encoding="utf-8")


def first(row):
    c = row["classes"]
    if not c:
        return None
    return c[0] if isinstance(c[0], str) else c[0]["code"]


def write_gz(path, data):
    """Reproducible gzip: no file name or time stamp in the header."""
    path.write_bytes(gzip.compress(data, compresslevel=9, mtime=0))


def share_draw(item_id):
    """Uniform in [0, 1) from the id and the seed: which books keep their number."""
    h = hashlib.sha256(f"kb-proposal-{SEED}|{item_id}".encode()).digest()
    return int.from_bytes(h[:8], "big") / 2 ** 64


def parents_of(scheme):
    return {c["code"]: c.get("parent") for c in json.loads(Path(scheme).read_text())["classes"]}


def ancestors(code, parents):
    """`code` and its ancestors, root-most first."""
    chain = []
    while code:
        chain.append(code)
        code = parents[code]
    return chain[::-1]


def cut(rows, parents, depth):
    return [{"id": r["id"], "classes": [ancestors(c, parents)[:depth][-1]] if (c := first(r)) else []}
            for r in rows]


def rollup(rows, parents, m):
    """Each item at its deepest class (or ancestor) whose subtree holds >= m items; main classes always."""
    count = Counter()
    for r in rows:
        if c := first(r):
            count.update(ancestors(c, parents))
    out = []
    for r in rows:
        c = first(r)
        if c:
            chain = ancestors(c, parents)
            c = next((a for a in reversed(chain) if count[a] >= m), chain[0])
        out.append({"id": r["id"], "classes": [c] if c else []})
    return out


def consensus_ddc(values):
    """Majority Dewey section of a work's distinct numbers, then their longest common prefix."""
    numbers = sorted({n for v in values if (n := ddc.ddc_number(v))})
    if not numbers:
        return None
    sections = Counter(n[:3] for n in numbers)
    best = min(sections, key=lambda s: (-sections[s], s))
    group = [n for n in numbers if n[:3] == best]
    prefix = group[0]
    for n in group[1:]:
        while not n.startswith(prefix):
            prefix = prefix[:-1]
    return prefix.rstrip(".")


def nested_learned(levels):
    """FDC sections (depth 3) with each book's learned-scheme path, cut to `levels`, nested inside."""
    src = DDC_R / "placements/books"
    fp = parents_of(SCHEMES["fdc"])
    fdc = json.loads(SCHEMES["fdc"].read_text())
    section = {r["id"]: r["classes"] for r in cut(jsonl(src / "fdc-ddc.jsonl"), fp, 3)}
    added, rows = {}, []
    for r in jsonl(src / "learned-own.jsonl"):
        parts = first(r).split(".")[:levels]
        parent = section[r["id"]][0]
        for i in range(1, len(parts) + 1):
            code = f"{parent}|{'.'.join(parts[:i])}"
            added.setdefault(code, {"code": code, "label": code, "parent": parent if i == 1 else
                                    f"{parent}|{'.'.join(parts[:i - 1])}"})
        rows.append({"id": r["id"], "classes": [code]})
    name = f"fdc-learned{levels}"
    scheme = {**fdc, "name": name, "version": f"{fdc['version']}+learned{levels}", "code_grammar": ".*",
              "title": f"FDC sections with learned-scheme paths ({levels} levels) nested (report 9)",
              "classes": fdc["classes"] + list(added.values())}
    path = OUT / f"schemes/{name}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(scheme, indent=1) + "\n", encoding="utf-8")
    SCHEMES[name] = path
    write_jsonl(OUT / f"placements/books/{name}.jsonl", rows)


def books_placements():
    src, out = DDC_R / "placements/books", OUT / "placements/books"
    fdc_rows, ddc_rows = jsonl(src / "fdc-ddc.jsonl"), jsonl(src / "ddc-gold.jsonl")
    fp, dp = parents_of(SCHEMES["fdc"]), parents_of(SCHEMES["ddc"])
    for d in (1, 2, 3):
        write_jsonl(out / f"fdc-d{d}.jsonl", cut(fdc_rows, fp, d))
    for d in (1, 2, 4):
        write_jsonl(out / f"ddc-d{d}.jsonl", cut(ddc_rows, dp, d))
    for m in ROLLUPS:
        write_jsonl(out / f"fdc-m{m}.jsonl", rollup(fdc_rows, fp, m))
    for levels in NESTED:
        nested_learned(levels)
    for scheme, gold in (("fdc", "fdc-ddc"), ("lcc", "lcc-gold")):
        g = {r["id"]: r for r in jsonl(src / f"{gold}.jsonl")}
        p = {r["id"]: r for r in jsonl(src / f"{scheme}-placed.jsonl")}
        for share in PIPE_SHARES:
            rows = [g[i] if share_draw(i) < share / 100 else p[i] for i in g]
            write_jsonl(out / f"{scheme}-pipe-{share}.jsonl", rows)


def holdout_placements():
    src, out = DDC_R / "placements/holdout", OUT / "placements/holdout"
    classes, depths = ddc.fdc_index(SCHEMES["fdc"])
    edition = {r["id"]: ddc.first_number(r) for r in jsonl(ROOT / "bench/corpus/holdout-ddc.jsonl")}
    works = {r["id"]: r for r in jsonl(OUT / "availability/holdout-works.jsonl")}
    rows = []
    for item in jsonl(ROOT / "bench/corpus/holdout.jsonl"):
        number = edition.get(item["id"]) or consensus_ddc(works[item["id"]]["ddc"])
        code = ddc.fdc_class(number, classes, depths) if number else None
        rows.append({"id": item["id"], "classes": [code] if code else []})
    write_jsonl(out / "fdc-work.jsonl", rows)
    for scheme, gold in (("fdc", out / "fdc-work.jsonl"), ("lcc", src / "lcc-gold.jsonl")):
        g = {r["id"]: r for r in jsonl(gold)}
        p = {r["id"]: r for r in jsonl(src / f"{scheme}-placed.jsonl")}
        write_jsonl(out / f"{scheme}-pipeline.jsonl", [g[i] if g[i]["classes"] else p[i] for i in g])


def arxiv_fdc(code, parents):
    if code not in parents:
        return ARXIV_FDC.get(code) or ARXIV_FDC.get(code.split(".")[0])
    for c in reversed(ancestors(code, parents)):
        if c in ARXIV_FDC:
            return ARXIV_FDC[c]
    return None


def corpus_placements():
    """Papers through the arXiv crosswalk: at the backbone class, or with the arXiv category nested."""
    ap = parents_of(SCHEMES["arxiv"])
    arxiv = json.loads(SCHEMES["arxiv"].read_text())
    gold = jsonl(EMB_R / "placements/corpus-arxiv-gold.jsonl")
    out = OUT / "placements/corpus"
    for backbone, to_class in (("fdc", lambda f: f), ("lcc", FDC_LCC.get)):
        base = json.loads(SCHEMES[backbone].read_text())

        def target(code):
            f = arxiv_fdc(code, ap)
            return to_class(f) if f else None

        nested = [{**c, "code": f"arxiv:{c['code']}", "parent": target(c["code"]), "range": None}
                  for c in arxiv["classes"] if not c["code"].startswith("grp_") and target(c["code"])]
        name = f"{backbone}-arxiv"
        scheme = {**base, "name": name, "version": f"{base['version']}+arxiv", "code_grammar": ".*",
                  "title": f"{backbone.upper()} with arXiv categories nested under their crosswalk class "
                           "(report 9)", "classes": base["classes"] + nested}
        SCHEMES[name].parent.mkdir(parents=True, exist_ok=True)
        SCHEMES[name].write_text(json.dumps(scheme, indent=1) + "\n", encoding="utf-8")
        write_jsonl(out / f"{name}.jsonl", [
            {"id": r["id"], "classes": [t] if (c := first(r)) and (t := target(c)) else []} for r in gold])
        write_jsonl(out / f"{name}-nested.jsonl", [
            {"id": r["id"], "classes": [f"arxiv:{c}"] if (c := first(r)) and target(c) else []}
            for r in gold])


def run_specs(universe):
    P, D, E = OUT / "placements" / universe, DDC_R / "placements" / universe, EMB_R / "placements"
    if universe == "books":
        runs = [("ddc-gold", "ddc", D), ("ddc-section", "ddc", D), ("fdc-ddc", "fdc", D),
                ("lcc-gold", "lcc", D), ("lcc-placed", "lcc", D), ("fdc-placed", "fdc", D),
                ("unesco-thesaurus-placed", "unesco-thesaurus", D), ("learned-placed", "learned", D),
                ("learned-own", "learned-books", D)]
        runs += [(f"ddc-d{d}", "ddc", P) for d in (1, 2, 4)]
        runs += [(f"fdc-d{d}", "fdc", P) for d in (1, 2, 3)]
        runs += [(f"fdc-m{m}", "fdc", P) for m in ROLLUPS]
        runs += [(f"fdc-learned{n}", f"fdc-learned{n}", P) for n in NESTED]
        runs += [(f"{s}-pipe-{n}", s, P) for s in ("fdc", "lcc") for n in PIPE_SHARES]
        return [{"key": k, "scheme": str(SCHEMES[s]), "placements": str(d / f"{k}.jsonl")} for k, s, d in runs]
    if universe == "holdout":
        runs = [("lcc-gold", "lcc", D / "lcc-gold.jsonl"), ("fdc-ddc", "fdc", D / "fdc-ddc.jsonl"),
                ("fdc-work", "fdc", P / "fdc-work.jsonl"), ("fdc-pipeline", "fdc", P / "fdc-pipeline.jsonl"),
                ("lcc-pipeline", "lcc", P / "lcc-pipeline.jsonl"), ("fdc-placed", "fdc", D / "fdc-placed.jsonl"),
                ("lcc-placed", "lcc", D / "lcc-placed.jsonl"),
                ("learned-own", "learned-holdout", E / "holdout-learned-own.jsonl")]
    else:
        runs = [("arxiv-gold", "arxiv", E / "corpus-arxiv-gold.jsonl"),
                ("fdc-arxiv", "fdc", P / "fdc-arxiv.jsonl"),
                ("fdc-arxiv-nested", "fdc-arxiv", P / "fdc-arxiv-nested.jsonl"),
                ("lcc-arxiv", "lcc", P / "lcc-arxiv.jsonl"),
                ("lcc-arxiv-nested", "lcc-arxiv", P / "lcc-arxiv-nested.jsonl"),
                ("fdc-placed", "fdc", E / "corpus-fdc-placed.jsonl"),
                ("openalex-topics-placed", "openalex-topics", E / "corpus-openalex-topics-placed.jsonl"),
                ("learned-placed", "learned", E / "corpus-learned-placed.jsonl")]
    return [{"key": k, "scheme": str(SCHEMES[s]), "placements": str(p)} for k, s, p in runs]


def graph_path(universe, model, k, tmp):
    """The committed graph when there is one, else this run's (books) graph."""
    if universe == "books":
        if (model, k) == ("minilm", 20):
            src = DDC_R / "graphs/books-k20.json.gz"
        else:
            src = OUT / f"graphs/{model}-books-k{k}.json.gz"
    elif (model, k) == ("minilm", 20):
        src = ROOT / f"bench/results/2026-10-10/graphs/{universe}-k20.json.gz"
    else:
        src = EMB_R / f"graphs/{model}-{universe}-k{k}.json.gz"
    dst = Path(tmp) / src.name.removesuffix(".gz")
    dst.write_bytes(gzip.decompress(src.read_bytes()))
    return dst


CORPORA = {"books": "bench/corpus/holdout-books.jsonl", "holdout": "bench/corpus/holdout.jsonl",
           "corpus": "bench/corpus/corpus.jsonl"}
CONDITIONS = {"books": [(m, k) for m in MODELS for k in KS],
              "holdout": [(m, 20) for m in MODELS], "corpus": [(m, 20) for m in MODELS]}


def bootstrap(a):
    (OUT / "bootstrap").mkdir(parents=True, exist_ok=True)
    for universe, conditions in CONDITIONS.items():
        if a.universe and universe != a.universe:
            continue
        for model, k in conditions:
            name = OUT / f"bootstrap/{universe}-{model}-k{k}"
            if name.with_suffix(".json").exists() and not a.force:
                continue
            with tempfile.TemporaryDirectory() as tmp:
                spec = Path(tmp) / "runs.json"
                spec.write_text(json.dumps(run_specs(universe)))
                draws = Path(tmp) / "draws.json"
                subprocess.run([a.harness, "bootstrap", "--corpus", CORPORA[universe],
                                "--graph", graph_path(universe, model, k, tmp), "--runs", spec,
                                "--resamples", "1000", "--seed", str(SEED), "--threads", str(a.threads),
                                "--output", name.with_suffix(".json"), "--draws-out", draws,
                                "--draw-metrics", "cost_norm,nav_log_a1,coverage,mean_depth"],
                               cwd=ROOT, check=True)
                result = json.loads(name.with_suffix(".json").read_text())
                result["runs_spec"] = [{**r, "scheme": str(Path(r["scheme"]).relative_to(ROOT)),
                                        "placements": str(Path(r["placements"]).relative_to(ROOT))}
                                       for r in run_specs(universe)]
                result["condition"] = {"universe": universe, "model": model, "k": k}
                name.with_suffix(".json").write_text(json.dumps(result, indent=1) + "\n")
                write_gz(Path(f"{name}.draws.json.gz"), draws.read_bytes())
            print(f"{universe} {model} k{k}: done", flush=True)


BOOT_KEYS = ["dasgupta.cost_norm", "vertex.dasgupta.cost_norm", "vertex.triplet.accuracy",
             "vertex.order.locality", "vertex.tsd.nats"]
SCORED = ["ddc-gold", "ddc-section", "fdc-ddc", "fdc-d2", "fdc-d3", "lcc-gold", "learned-own",
          "fdc-placed", "unesco-thesaurus-placed"]


def catalogue(a, spec, corpus, graph, out, subset=None):
    """`shelfmark-bench scores` for one run, keeping only the replicates of BOOT_KEYS."""
    if out.exists() and not a.force:
        return
    args = [a.harness, "scores", "--corpus", corpus, "--scheme", spec["scheme"], "--graph", graph,
            "--placements", spec["placements"], "--seed", str(SEED), "--reps", "10", "--boot", "1000",
            "--output", out]
    if subset:
        args += ["--subset", subset]
    subprocess.run([str(x) for x in args], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
    result = json.loads(out.read_text())
    boot = result["catalogue"].pop("boot", {})
    result["catalogue"]["boot"] = {k: boot[k] for k in BOOT_KEYS if k in boot}
    result["spec"] = {"key": spec["key"], "scheme": str(Path(spec["scheme"]).relative_to(ROOT)),
                      "placements": str(Path(spec["placements"]).relative_to(ROOT))}
    out.write_text(json.dumps(result, separators=(",", ":")) + "\n")


def scores(a):
    """Catalogue scores of the main books runs (MiniLM k = 20), and FDC against LCC on the
    holdout books both place (all three k = 20 graphs)."""
    (OUT / "scores").mkdir(parents=True, exist_ok=True)
    specs = {r["key"]: r for r in run_specs("books")}
    with tempfile.TemporaryDirectory() as tmp:
        graph = graph_path("books", "minilm", 20, tmp)
        for key in SCORED:
            catalogue(a, specs[key], CORPORA["books"], graph, OUT / f"scores/books-{key}.json")
        specs = {r["key"]: r for r in run_specs("holdout")}
        both = [r["id"] for r, s in zip(jsonl(specs["fdc-work"]["placements"]),
                                        jsonl(specs["lcc-gold"]["placements"]))
                if r["classes"] and s["classes"]]
        ids = OUT / "scores/holdout-both-ids.txt"
        ids.write_text("".join(i + "\n" for i in both))
        for model in MODELS:
            graph = graph_path("holdout", model, 20, tmp)
            for key in ("fdc-work", "lcc-gold"):
                catalogue(a, specs[key], CORPORA["holdout"], graph,
                          OUT / f"scores/holdout-both-{model}-{key}.json", subset=ids)
        print(f"scores: {len(SCORED)} books runs, {len(both)} holdout books placed by both", flush=True)


def graphs(a):
    """Books graphs for bge-small and nomic at k = 10, 20, 50 and MiniLM at k = 10, 50."""
    (OUT / "graphs").mkdir(parents=True, exist_ok=True)
    for model in MODELS:
        for k in KS:
            if (model, k) == ("minilm", 20):
                continue
            src = CACHE / f"graphs/{model}-books-k{k}.json"
            dst = OUT / f"graphs/{model}-books-k{k}.json.gz"
            write_gz(dst, src.read_bytes())
            shutil.copy(src.with_name(f"{model}-books-k{k}.manifest.json"),
                        OUT / f"graphs/{model}-books-k{k}.manifest.json")


def placements(_a):
    books_placements()
    holdout_placements()
    corpus_placements()


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("step", choices=["graphs", "placements", "bootstrap", "scores", "all"])
    p.add_argument("--harness", default="target/release/shelfmark-bench")
    p.add_argument("--threads", type=int, default=16)
    p.add_argument("--universe", choices=list(CONDITIONS))
    p.add_argument("--force", action="store_true")
    a = p.parse_args()
    for step in (["graphs", "placements", "bootstrap", "scores"] if a.step == "all" else [a.step]):
        globals()[step](a)


if __name__ == "__main__":
    main()
