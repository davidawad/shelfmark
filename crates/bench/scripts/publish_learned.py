#!/usr/bin/env python3
"""Publish the learned-scheme benchmark (learned_benchmark.py) as a dated
results directory: results.json, results.md and the per-run harness JSON.
REPRODUCE.md is written by hand and left alone."""
import argparse
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
LAMBDA, BRANCH = 0.005, 1.0
PLACED = ["acm-ccs", "arxiv", "bk", "eurovoc", "fdc", "ilc2", "ipc", "jel", "lcc", "lcc-full", "msc",
          "openalex-topics", "unesco-thesaurus"]
GENERAL = ["lcc", "lcc-full", "fdc", "bk", "ilc2", "unesco-thesaurus"]
GOLD = {"corpus": ["arxiv", "msc", "jel"], "holdout": ["lcc", "lcc-full"]}


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path):
    return json.loads(Path(path).read_text())


def objective(r):
    return r["cost_norm"] + LAMBDA * r["structure"]["nav_log_a1"]


def row(label, source, r, published=None):
    s = r["structure"]
    out = {"scheme": label, "placements": source, "n": r["n"], "coverage": r["coverage"],
           "cost_norm": r["cost_norm"], "revenue_norm": r["revenue_norm"], "skill": r["skill"],
           "ref_cost_norm": r["ref_cost_norm"], "objective_J": objective(r), "nav_log_a1": s["nav_log_a1"],
           "nav_scan": s["nav_scan"], "effective_classes": s["effective_classes"], "mean_depth": s["mean_depth"],
           "max_depth": s["max_depth"], "max_branching": s["max_branching"], "root_fanout": s["root_fanout"]}
    if published is not None:
        out["published_cost_norm"] = published
        out["matches_published"] = abs(published - r["cost_norm"]) < 1e-12
    return out


def md_table(rows, ref=None):
    head = ("| scheme | placements | coverage | cost_norm | skill | J | nav log a=1 | classes used | "
            "mean depth | max branching |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n")
    body = "".join(
        f"| {r['scheme']} | {r['placements']} | {r['coverage']:.4f} | {r['cost_norm']:.4f} | {r['skill']:.4f} | "
        f"{r['objective_J']:.4f} | {r['nav_log_a1']:.2f} | {r['effective_classes']:.1f} | {r['mean_depth']:.2f} | "
        f"{r['max_branching']} |\n" for r in sorted(rows, key=lambda r: r["objective_J"]))
    if ref is not None:
        body += f"| average linkage (binary reference, not a classification) | - | 1.0000 | {ref:.4f} | 1.0000 | - | - | - | - | 2 |\n"
    return head + body


def published(path, key):
    try:
        return load(path)["results"][key]["cost_norm"]
    except (FileNotFoundError, KeyError):
        return None


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--out", default=str(ROOT / "bench/results/2026-10-10-learned"))
    p.add_argument("--harness", required=True)
    p.add_argument("--shelfmark", required=True)
    p.add_argument("--learned", required=True)
    a = p.parse_args()
    cache, out = Path(a.cache), Path(a.out)
    lb = cache / "learned-bench"
    runs = {f.stem: load(f) for f in sorted((lb / "runs").glob("*.json"))}
    p1 = {f.stem: load(f) for f in sorted((cache / "runs").glob("*.json"))
          if not f.stem.startswith("reference") and not f.stem.endswith(".skipped") and f.stem != "failures"}
    r2 = ROOT / "bench/results/2026-10-10-r2/results.json"
    gold_pub = ROOT / "bench/results/2026-10-10/results.json"
    refs = {g: load(cache / "runs" / f"reference-{g}.json")["ref_cost_norm"] for g in ("corpus", "holdout")}

    tables = {"corpus": [], "holdout": [], "test20": [], "split": []}
    tables["corpus"].append(row("learned", "own, in-sample (fitted on this corpus)", runs["learned-own"]))
    tables["corpus"].append(row("learned", "shelfmark lexical@2", runs["learned-placed"]))
    tables["holdout"].append(row("learned", "own insertion (embedding nearest centroid)", runs["learned-own-holdout"]))
    tables["holdout"].append(row("learned", "shelfmark lexical@2", runs["learned-placed-holdout"]))
    for name in PLACED:
        if f"{name}-placed" in p1:
            tables["corpus"].append(row(name, "shelfmark lexical@2", p1[f"{name}-placed"],
                                        published(r2, f"{name}-placed")))
        if name in GENERAL and f"{name}-placed-holdout" in p1:
            tables["holdout"].append(row(name, "shelfmark lexical@2", p1[f"{name}-placed-holdout"],
                                         published(r2, f"{name}-placed-holdout")))
        if f"test-{name}-placed" in runs:
            tables["test20"].append(row(name, "shelfmark lexical@2", runs[f"test-{name}-placed"]))
    for corpus, names in GOLD.items():
        for name in names:
            if f"{name}-gold" in p1:
                tables[corpus].append(row(name, "gold (source labels)", p1[f"{name}-gold"],
                                          published(gold_pub, f"{name}-gold")))
    for name in GOLD["corpus"]:
        if f"test-{name}-gold" in runs:
            tables["test20"].append(row(name, "gold (source labels)", runs[f"test-{name}-gold"]))
    tables["test20"].append(row("learned (fit on the other 80%)", "own insertion, held out", runs["learned-split-heldout"]))
    tables["test20"].append(row("learned (full fit)", "own, in-sample", runs["test-learned-own"]))
    tables["test20"].append(row("learned (full fit)", "shelfmark lexical@2", runs["test-learned-placed"]))
    tables["split"].append(row("learned (fit on 80%)", "own, in-sample, 80% graph", runs["learned-split-train"]))
    tables["split"].append(row("learned (fit on 80%)", "80% fitted + 20% inserted, whole corpus graph",
                               runs["learned-split-full-corpus"]))
    tables["split"].append(row("learned (full fit)", "own, in-sample, whole corpus graph", runs["learned-own"]))

    out.mkdir(parents=True, exist_ok=True)
    (out / "runs").mkdir(exist_ok=True)
    for key, r in runs.items():
        (out / "runs" / f"{key}.json").write_text(json.dumps(r, indent=2) + "\n")
    (out / "split").mkdir(exist_ok=True)
    for src, dst in [(lb / "learned-holdout.json", out / "learned-holdout.json"),
                     (lb / "insert-report.json", out / "insert-report.json"),
                     (lb / "build-report.json", out / "build-report.json"),
                     (lb / "split/test-ids.txt", out / "split/test-ids.txt"),
                     (lb / "split/split.json", out / "split/learned-split80.json"),
                     (lb / "split/build-report.json", out / "split/build-report.json"),
                     (lb / "split/insert-report.json", out / "split/insert-report.json")]:
        shutil.copy(src, dst)
    sweep = []
    for f in sorted((lb / "sweep").glob("*.json")):
        r = load(f)
        c = r["config"]["params"]
        sweep.append({"branch_cost": c["branch_cost"], "max_depth": c["max_depth"], "lambda": c["lambda"],
                      "J": r["objective"]["total"], "dasgupta": r["objective"]["dasgupta"], "nav": r["objective"]["nav"],
                      "classes": r["shape"]["classes"], "leaf_classes": r["shape"]["leaf_classes"],
                      "root_children": r["shape"]["children"]["root"],
                      "mean_item_class_depth": r["shape"]["mean_item_class_depth"],
                      "median_leaf": r["shape"]["leaf_size"]["median"]})
    sweep.sort(key=lambda s: (s["branch_cost"], s["max_depth"], s["lambda"]))
    build = load(lb / "build-report.json")
    ins = load(lb / "insert-report.json")
    split_ins = load(lb / "split/insert-report.json")
    commit = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=ROOT).stdout.strip()
    result = {
        "date": "2026-10-10-learned",
        "objective": {
            "formula": "J = cost_norm + lambda * nav_log_a; cost_norm = sum_e w_e |items(lca_e)| / (n W) (Dasgupta, "
                       "items under their class); nav_log_a = mean over items of sum over the classes on the path of "
                       "(a + log2 fan-out), fan-out = child classes, or works in a leaf class",
            "lambda": LAMBDA, "branch_cost_a": BRANCH,
            "constraints": "2-12 child classes, at most 4 class levels, leaf classes of at least 5 works, every class "
                           "a contiguous run of the fixed shelf order",
        },
        "corpus": {"corpus.jsonl": sha256(ROOT / "bench/corpus/corpus.jsonl"),
                   "holdout.jsonl": sha256(ROOT / "bench/corpus/holdout.jsonl")},
        "graphs": {g: {"graph_sha256": runs[k]["hashes"]["graph"], "ref_cost_norm": refs[g],
                       "published": f"bench/results/2026-10-10/graphs/{g}-k20.json.gz"}
                   for g, k in (("corpus", "learned-own"), ("holdout", "learned-own-holdout"))},
        "learned": {"scheme": "schemes/learned/learned.json",
                    "scheme_sha256": sha256(ROOT / "schemes/learned/learned.json"),
                    "build": build},
        "insertion": {
            "holdout": {k: ins[k] for k in ("scheme", "base", "inserted", "params", "classes_before",
                                             "classes_after", "stable")} | {"splits": len(ins["splits"])},
            "split": {k: split_ins[k] for k in ("scheme", "base", "inserted", "params", "classes_before",
                                                 "classes_after", "stable")} | {"splits": len(split_ins["splits"])},
        },
        "split": {"seed": "learned-split-2026-10-10", "fraction": 0.2, "test_ids": "split/test-ids.txt",
                  "test_ref_cost_norm": runs["learned-split-heldout"]["ref_cost_norm"]},
        "tables": tables,
        "sweep": sweep,
        "baselines": {"placed": "bench/results/2026-10-10-r2/results.json",
                      "gold": "bench/results/2026-10-10/results.json",
                      "paper1_rerun_matches_published": all(r.get("matches_published", True)
                                                            for t in tables.values() for r in t)},
        "tools": {"commit": commit, "shelfmark_sha256": sha256(a.shelfmark),
                  "shelfmark_bench_sha256": sha256(a.harness), "learned_scheme_sha256": sha256(a.learned)},
        "runs": runs,
    }
    (out / "results.json").write_text(json.dumps(result, indent=2) + "\n")
    (lb / "tables.md").write_text("\n\n".join(f"## {k}\n\n" + md_table(v, refs.get(k)) for k, v in tables.items()))
    print(json.dumps({k: len(v) for k, v in tables.items()}))


if __name__ == "__main__":
    main()
