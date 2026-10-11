#!/usr/bin/env python3
"""Score every benchmark run with the scoring catalogue (`shelfmark-bench
scores`) and the metric-agreement analysis (`shelfmark-bench agree`).

Inputs come from the ignored cache of run_benchmark.py and
learned_benchmark.py (scheme files, gold corpora, shelfmark placements,
learned-scheme placements, harness results) plus the published graphs and
the embedding vectors written by build_graph_fastembed.py --vectors. Each
run writes <cache>/scores/<key>.json and is skipped when that file exists.
`publish` writes results.json and results.md into the results directory.
"""
import argparse
import hashlib
import json
import math
import re
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PLACED = ["acm-ccs", "arxiv", "bk", "eurovoc", "fdc", "ilc2", "ipc", "jel", "lcc", "lcc-full", "msc",
          "openalex-topics", "unesco-thesaurus"]
GENERAL = ["lcc", "lcc-full", "fdc", "bk", "ilc2", "unesco-thesaurus"]
GOLD = {"corpus": ["arxiv", "msc", "jel"], "holdout": ["lcc", "lcc-full"]}
CORPORA = {"corpus": ROOT / "bench/corpus/corpus.jsonl", "holdout": ROOT / "bench/corpus/holdout.jsonl"}
LCC_LETTERS = re.compile(r"^([A-Z]{1,3})(?=[0-9]|$| )")


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}", flush=True)


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def jsonl(path):
    return [json.loads(line) for line in Path(path).read_text(encoding="utf-8").splitlines() if line.strip()]


class Runs:
    def __init__(self, a):
        self.a = a
        self.cache = Path(a.cache)
        self.out = self.cache / "scores"
        self.out.mkdir(parents=True, exist_ok=True)
        self.graphs = {g: self.cache / "graphs" / f"{g}-k20.json" for g in CORPORA}
        self.vectors = {g: self.cache / "embed" / f"{g}.f32" for g in CORPORA}
        for g, path in self.graphs.items():
            manifest = load(path.with_name(f"{g}-k20.manifest.json"))
            if sha256(path) != manifest["graph_sha256"]:
                raise SystemExit(f"{path}: sha256 does not match its manifest")

    def external(self, corpus):
        """External gold labels: arXiv primary category and its group for the
        papers, LCC class letters and main class for the books."""
        path = self.cache / "scores" / f"external-{corpus}.jsonl"
        if path.exists():
            return path
        rows = []
        if corpus == "corpus":
            parent = {c["code"]: c.get("parent") for c in load(self.cache / "schemes/arxiv.json")["classes"]}

            def top(code):
                while parent.get(code):
                    code = parent[code]
                return code
            for r in jsonl(CORPORA[corpus]):
                cats = [c for c in r.get("categories", []) if c in parent]
                fine = cats[0] if cats else None
                rows.append({"id": r["id"], "fine": fine, "coarse": top(fine) if fine else None})
        else:
            for r in jsonl(CORPORA[corpus]):
                calls = [c for c in r.get("lc_classifications", []) if c and not c.startswith("MLCS")]
                m = LCC_LETTERS.match(calls[0].strip()) if calls else None
                rows.append({"id": r["id"], "fine": m.group(1) if m else None,
                             "coarse": m.group(1)[0] if m else None})
        # Written once before the parallel scoring, atomically, so no run
        # reads a partial file.
        tmp = path.with_suffix(".tmp")
        tmp.write_text("".join(json.dumps(r) + "\n" for r in rows), encoding="utf-8")
        tmp.replace(path)
        return path

    def specs(self):
        """(key, group, corpus, scheme file, corpus file, placements, compare option, harness result, stats)."""
        c, learned = self.cache, self.cache / "learned-bench"
        out = []
        for corpus, names in GOLD.items():
            for name in names:
                out.append((f"{name}-gold", "gold", corpus, c / f"schemes/{name}.json",
                            c / f"gold/{corpus}-{name}.jsonl", None, None, c / f"runs/{name}-gold.json", None))
        for name in PLACED + ["learned"]:
            for corpus in ["corpus", "holdout"]:
                if corpus == "holdout" and name not in GENERAL + ["learned"]:
                    continue
                suffix = "" if corpus == "corpus" else "-holdout"
                compare = None
                if name in GOLD[corpus]:
                    compare = ("--compare-gold", c / f"gold/{corpus}-{name}.jsonl")
                elif name == "learned":
                    own = "corpus-placements.jsonl" if corpus == "corpus" else "holdout-placements.jsonl"
                    compare = ("--compare-placements", learned / own)
                scheme = c / f"schemes/{name}.json"
                harness = (learned / f"runs/learned-placed{suffix}.json" if name == "learned"
                           else c / f"runs/{name}-placed{suffix}.json")
                out.append((f"{name}-placed{suffix}", f"placed-{corpus}", corpus, scheme, CORPORA[corpus],
                            c / f"placed/{corpus}-{name}/placements.jsonl", compare, harness,
                            c / f"placed/{corpus}-{name}/stats.json"))
        out.append(("learned-own", "gold", "corpus", learned / "learned.json", CORPORA["corpus"],
                    learned / "corpus-placements.jsonl", None, learned / "runs/learned-own.json", None))
        out.append(("learned-own-holdout", "gold", "holdout", learned / "learned-holdout.json", CORPORA["holdout"],
                    learned / "holdout-placements.jsonl", None, learned / "runs/learned-own-holdout.json", None))
        return out

    def score(self, spec):
        key, group, corpus, scheme, corpus_file, placements, compare, harness, stats = spec
        path = self.out / f"{key}.json"
        if path.exists():
            return
        cmd = [self.a.harness, "scores", "--corpus", corpus_file, "--scheme", scheme,
               "--graph", self.graphs[corpus], "--vectors", self.vectors[corpus],
               "--external", self.external(corpus), "--seed", str(self.a.seed),
               "--reps", str(self.a.reps), "--boot", str(self.a.boot), "--output", path]
        if placements:
            cmd += ["--placements", placements]
        if compare:
            cmd += list(compare)
        started = time.monotonic()
        proc = subprocess.run([str(x) for x in cmd], capture_output=True, text=True, check=False)
        if proc.returncode != 0:
            raise RuntimeError(f"{key}: {proc.stderr.strip()[-600:]}")
        result = load(path)
        h = load(harness)
        scores = result["catalogue"]["scores"]
        # The catalogue recomputes Dasgupta cost on single placements; it must
        # equal the harness result for the same tree.
        if abs(scores["dasgupta.cost_norm"] - h["cost_norm"]) > 1e-9:
            raise RuntimeError(f"{key}: cost_norm {scores['dasgupta.cost_norm']} != harness {h['cost_norm']}")
        scores["dasgupta.skill"] = h["skill"]
        for k, v in h["structure"].items():
            scores[f"structure.{k}"] = float(v)
        if stats:
            st = load(stats)
            scores["latency.ms_per_item"] = 1000.0 * st["classify_seconds"] / st["rows"]
        result["run"] = {"key": key, "group": group, "corpus": corpus, "harness_result": str(harness),
                         "harness_tree_sha256": h["hashes"]["tree"],
                         "seconds": round(time.monotonic() - started, 1)}
        path.write_text(json.dumps(result, indent=1) + "\n", encoding="utf-8")
        log(f"scored {key} in {result['run']['seconds']}s")


def run(a):
    runs = Runs(a)
    for corpus in CORPORA:
        runs.external(corpus)
    failures = {}

    def attempt(spec):
        try:
            runs.score(spec)
        except Exception as e:  # noqa: BLE001 - recorded; other runs continue
            failures[spec[0]] = str(e)
            log(f"FAILED {spec[0]}: {e}")
    with ThreadPoolExecutor(a.jobs) as pool:
        list(pool.map(attempt, runs.specs()))
    (runs.out / "failures.json").write_text(json.dumps(failures, indent=2) + "\n")
    log(f"done; {len(failures)} failed")


# Scores entered into the agreement analysis, with the better direction.
AGREE = [
    ("dasgupta.cost_norm", "down"), ("dasgupta.binarized_cost_norm", "down"),
    ("dasgupta.level1_cost_norm", "down"), ("tsd.nats", "up"), ("triplet.accuracy", "up"),
    ("embed.cophenetic", "up"), ("embed.silhouette_d1", "up"), ("embed.davies_bouldin_d1", "down"),
    ("probe.knn_d1", "up"), ("probe.centroid_d1", "up"), ("gold.ami_fine", "up"),
    ("gold.v_measure_fine", "up"), ("gold.dendrogram_purity_fine", "up"),
    ("gold.hierarchical_f_fine", "up"), ("gold.dendrogram_purity_coarse", "up"),
    ("order.locality", "up"), ("order.kendall_tau", "up"), ("collocation.author_same_class", "up"),
    ("search.scheme_bits", "down"), ("search.gap_bits", "down"), ("mdl.total_bits_per_item", "down"),
    ("coverage", "up"), ("warrant.useful_depth_share", "up"), ("granularity.empty_class_share", "down"),
    ("hospitality.array", "up"), ("notation.length", "down"), ("notation.regularity", "up"),
    ("insertion.splits_per_1000", "down"), ("structure.gini", "down"),
    ("structure.largest_class_share", "down"),
]
GROUPS = {"gold": "gold labels (source classifiers; learned: own placements)",
          "placed-corpus": "shelfmark-placed, arXiv corpus", "placed-holdout": "shelfmark-placed, book holdout"}


def fmt(v, digits=3):
    if v is None:
        return "-"
    if isinstance(v, float) and math.isnan(v):
        return "nan"
    return f"{v:.{digits}f}"


def agreement(a, runs, group):
    keys = sorted(k for k, r in runs.items() if r["run"]["group"] == group and k != "jel-gold")
    metrics = [m for m, _ in AGREE
               if sum(m in runs[k]["catalogue"]["scores"] for k in keys) >= 4]
    point = [[runs[k]["catalogue"]["scores"].get(m) for k in keys] for m in metrics]
    boot = [[runs[k]["catalogue"].get("boot", {}).get(m, []) for k in keys] for m in metrics]
    boot = [b if all(b) else [] for b in boot]
    noise = [[runs[k]["catalogue"]["noise"]["0.2"].get(m) for k in keys] for m in metrics]
    work = Path(a.cache) / "scores" / f"agree-{group}"
    work.with_suffix(".input.json").write_text(json.dumps(
        {"metrics": metrics, "runs": keys, "point": point, "boot": boot, "noise": noise}))
    subprocess.run([str(a.harness), "agree", "--input", str(work.with_suffix(".input.json")),
                    "--output", str(work.with_suffix(".json"))], check=True)
    return load(work.with_suffix(".json"))


def heatmap(ag):
    order = ag["order"]
    names = [ag["metrics"][i] for i in order]
    head = "| | " + " | ".join(f"M{j + 1}" for j in range(len(order))) + " |\n"
    head += "| --- |" + " ---: |" * len(order) + "\n"
    body = ""
    for j, i in enumerate(order):
        cells = []
        for k in order:
            v = ag["spearman"][i][k]
            text = "1" if i == k else f"{v:+.2f}".replace("0.", ".")
            cells.append(f"**{text}**" if i != k and abs(v) >= 0.7 else text)
        body += f"| M{j + 1} `{names[j]}` | " + " | ".join(cells) + " |\n"
    return head + body


def table(rows, cols):
    out = "| run | " + " | ".join(c for c, _ in cols) + " |\n| --- |" + " ---: |" * len(cols) + "\n"
    for key, r in rows:
        out += f"| {key} | " + " | ".join(f(r) for _, f in cols) + " |\n"
    return out


def publish(a):
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    cache = Path(a.cache)
    runs = {}
    for path in sorted((cache / "scores").glob("*.json")):
        if path.name == "failures.json" or path.name.startswith("agree-"):
            continue
        runs[path.stem] = load(path)
    agree = {g: agreement(a, runs, g) for g in GROUPS}
    status = {line.split("\t")[0]: line.rstrip("\n").split("\t")
              for line in (cache / "schemes/status.tsv").open()}
    published = {}
    for key, r in runs.items():
        cat = dict(r["catalogue"])
        cat.pop("boot", None)
        published[key] = dict(r, catalogue=cat)
    harness = Path(a.harness)
    git = ["git", "-C", str(ROOT)]
    commit = subprocess.run([*git, "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    dirty = bool(subprocess.run([*git, "status", "--porcelain", "--", "crates/bench/src"],
                                capture_output=True, text=True, check=True).stdout.strip())
    doc = {
        "date": "2026-10-10",
        "inputs": {
            "corpus_sha256": {g: sha256(p) for g, p in CORPORA.items()},
            "graphs_sha256": {g: sha256(cache / f"graphs/{g}-k20.json") for g in CORPORA},
            "vectors_sha256": {g: sha256(cache / f"embed/{g}.f32") for g in CORPORA},
            "schemes_sha256": {k: v[2] for k, v in status.items() if v[1] == "ok"},
            "learned_sha256": sha256(ROOT / "schemes/learned/learned.json"),
        },
        "harness": {"base_commit": commit, "source_modified_since_base": dirty,
                    "binary_sha256": sha256(harness)},
        "agree_metrics": AGREE,
        "runs": published,
        "agreement": {g: {k: v for k, v in ag.items()} for g, ag in agree.items()},
    }
    (out / "results.json").write_text(json.dumps(doc, indent=1) + "\n", encoding="utf-8")
    (out / "results.md").write_text(markdown(a, runs, agree), encoding="utf-8")
    log(f"published {len(runs)} runs to {out}")


def ci(r, key, digits=3):
    s = r["catalogue"]["scores"].get(key)
    c = r["catalogue"]["ci"].get(key)
    if s is None:
        return "-"
    return f"{s:.{digits}f} [{c[0]:.{digits}f}, {c[1]:.{digits}f}]" if c else f"{s:.{digits}f}"


def z(model, key):
    return lambda run: fmt(run["catalogue"]["null"][model].get(key, {}).get("z"), 1)


def noisy(key, p="0.2"):
    return lambda run: fmt(run["catalogue"]["noise"][p].get(key))


def consistency_cell(key, digits):
    return lambda r: fmt(r["consistency"][key], digits)


def sc(key, digits=3):
    return lambda r: fmt(r["catalogue"]["scores"].get(key), digits)


def markdown(a, runs, agree):
    notes = Path(a.notes).read_text(encoding="utf-8") if a.notes else ""
    md = "# Scoring catalogue, 2026-10-10\n\n" + notes + "\n"
    for group, title in GROUPS.items():
        rows = sorted(((k, r) for k, r in runs.items() if r["run"]["group"] == group),
                      key=lambda kr: kr[1]["catalogue"]["scores"]["dasgupta.cost_norm"])
        md += f"## {title}\n\n### Data-science scores\n\n"
        md += table(rows, [
            ("coverage", sc("coverage")), ("cost_norm [95% CI]", lambda r: ci(r, "dasgupta.cost_norm")),
            ("binarized", sc("dasgupta.binarized_cost_norm")), ("level 1", sc("dasgupta.level1_cost_norm")),
            ("TSD nats [CI]", lambda r: ci(r, "tsd.nats")), ("triplet [CI]", lambda r: ci(r, "triplet.accuracy")),
            ("cophenetic", sc("embed.cophenetic")), ("silhouette d1", sc("embed.silhouette_d1")),
            ("DB d1", sc("embed.davies_bouldin_d1", 2)), ("kNN d1/2/3", lambda r: "/".join(
                fmt(r["catalogue"]["scores"].get(f"probe.knn_d{d}"), 2) for d in (1, 2, 3))),
            ("centroid d1/2/3", lambda r: "/".join(
                fmt(r["catalogue"]["scores"].get(f"probe.centroid_d{d}"), 2) for d in (1, 2, 3))),
            ("majority d1", sc("probe.majority_d1", 2)),
        ])
        md += "\n### Against external gold (arXiv category / LCC letters)\n\n"
        md += table(rows, [
            ("AMI fine", sc("gold.ami_fine")), ("ARI fine", sc("gold.ari_fine")),
            ("V fine", sc("gold.v_measure_fine")), ("purity fine", sc("gold.purity_fine")),
            ("DP fine [CI]", lambda r: ci(r, "gold.dendrogram_purity_fine")),
            ("hier F fine", sc("gold.hierarchical_f_fine")), ("AMI coarse", sc("gold.ami_coarse")),
            ("DP coarse", sc("gold.dendrogram_purity_coarse")), ("hier F coarse", sc("gold.hierarchical_f_coarse")),
        ])
        md += "\n### Search, order and description length\n\n"
        md += table(rows, [
            ("scheme bits", sc("search.scheme_bits", 2)), ("Hu-Tucker", sc("search.hu_tucker_bits", 2)),
            ("Huffman", sc("search.huffman_bits", 2)), ("entropy", sc("search.entropy_bits", 2)),
            ("gap", sc("search.gap_bits", 2)), ("worst", sc("search.worst_bits", 0)),
            ("MDL bits/item", sc("mdl.total_bits_per_item", 2)), ("model bits", sc("mdl.model_bits", 0)),
            ("locality [CI]", lambda r: ci(r, "order.locality")), ("Kendall tau", sc("order.kendall_tau")),
            ("splits/1000", sc("insertion.splits_per_1000", 1)), ("churn/insert", sc("insertion.churn_per_insert", 2)),
            ("ms/item", sc("latency.ms_per_item", 2)),
        ])
        md += "\n### Librarian scores\n\n"
        md += table(rows, [
            ("warrant d>=2", sc("warrant.useful_depth_share")), ("hospitality", sc("hospitality.array")),
            ("ordered sibs", sc("hospitality.ordered_siblings")), ("notation len", sc("notation.length", 1)),
            ("expressive", sc("notation.expressiveness")), ("regular", sc("notation.regularity")),
            ("classes/1000", sc("granularity.classes_per_1000", 1)),
            ("works/leaf mean/med", lambda r: f"{fmt(r['catalogue']['scores'].get('granularity.works_per_leaf_mean'), 1)}"
             f"/{fmt(r['catalogue']['scores'].get('granularity.works_per_leaf_median'), 0)}"),
            ("empty classes", sc("granularity.empty_class_share")), ("exclusive", sc("canon.exclusiveness")),
            ("author same-class (chance)", lambda r: "-" if "collocation.author_same_class" not in r["catalogue"]["scores"]
             else f"{fmt(r['catalogue']['scores']['collocation.author_same_class'])} "
                  f"({fmt(r['catalogue']['scores']['collocation.author_same_class_chance'])})"),
            ("Gini", sc("structure.gini")),
        ])
        md += "\n### Null models (z = (score - null mean) / null sd, 10 draws) and noise\n\n"
        md += table(rows, [
            ("cost z shuffled", z("shuffled_labels", "dasgupta.cost_norm")),
            ("cost z shape", z("random_shape", "dasgupta.cost_norm")),
            ("triplet z shuffled", z("shuffled_labels", "triplet.accuracy")),
            ("triplet z shape", z("random_shape", "triplet.accuracy")),
            ("locality z shape", z("random_shape", "order.locality")),
            ("DP fine z shuffled", z("shuffled_labels", "gold.dendrogram_purity_fine")),
            ("cost @ p=0.05/0.2", lambda r: f"{noisy('dasgupta.cost_norm', '0.05')(r)}/{noisy('dasgupta.cost_norm')(r)}"),
            ("triplet @ 0.2", noisy("triplet.accuracy")), ("DP fine @ 0.2", noisy("gold.dendrogram_purity_fine")),
        ])
        cons = [(k, r) for k, r in rows if r.get("consistency")]
        if cons:
            md += "\n### Inter-classifier consistency (reference labels vs shelfmark, items with both)\n\n"
            keys = ["items", "exact", "kappa_exact", "depth1", "kappa_depth1", "depth2", "ari"]
            md += table(cons, [(c, consistency_cell(f"consistency.{c}", 0 if c == "items" else 3)) for c in keys])
        ag = agree[group]
        md += (f"\n### Metric agreement: Spearman rho across {len(ag['runs'])} runs, clustered\n\n"
               f"Runs: {', '.join(ag['runs'])}. Rows and columns are in average-linkage order on 1 - |rho|; "
               "bold marks |rho| >= 0.7. Signs are raw: a cost (lower is better) that agrees with a "
               "quality score shows a negative rho.\n\n")
        md += heatmap(ag)
        md += "\nRank stability: Kendall tau between the point ranking of runs and the ranking in each " \
              "bootstrap replicate (mean, 5th percentile), and the ranking under placement noise p = 0.2. " \
              "Only scores with bootstrap or noise replicates are listed; \"-\" = not replicated.\n\n"
        md += "| score | better | boot tau mean | boot tau p05 | noise tau |\n| --- | --- | ---: | ---: | ---: |\n"
        direction = dict(AGREE)
        for m in ag["metrics"]:
            st = ag["rank_stability"][m]
            if all(v is None for v in st.values()):
                continue
            md += f"| {m} | {direction[m]} | {fmt(st['boot_tau_mean'])} | {fmt(st['boot_tau_p05'])} | {fmt(st['noise_tau'])} |\n"
        md += "\n"
    return md


def main():
    p = argparse.ArgumentParser()
    p.add_argument("command", choices=["run", "publish"])
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--harness", required=True)
    p.add_argument("--jobs", type=int, default=4)
    p.add_argument("--seed", type=int, default=20261010)
    p.add_argument("--reps", type=int, default=10)
    p.add_argument("--boot", type=int, default=200)
    p.add_argument("--out", default=str(ROOT / "bench/results/2026-10-10-scores"))
    p.add_argument("--notes", help="markdown inserted after the results.md title")
    a = p.parse_args()
    run(a) if a.command == "run" else publish(a)


if __name__ == "__main__":
    main()
