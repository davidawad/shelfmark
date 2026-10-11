#!/usr/bin/env python3
"""Covered-subset scoring, coverage analysis and the dense-vs-sparse check.

Builds on score_catalogue.py (same cache, same 28 runs, same scores call):

* `run` scores every run on all items (writing the ids of the items it
  places), then again restricted to its own covered items, and scores every
  run of a corpus restricted to each gold run's labelled items (one
  "family" per gold scheme: arxiv, msc, jel on the papers; lcc, lcc-full on
  the books). A restricted run keeps only the subset's items and the graph
  edges between them. Families or covered subsets under MIN_ITEMS items
  are reported by size only.
* `dense` runs `shelfmark-bench dense` on the papers (random subsets of
  1000, 2000 and 5000) and on the 500-book holdout.
* `publish` runs `shelfmark-bench agree` per group (with the coverage
  control, the covered-subset comparison and, where replicates are paired,
  paired-bootstrap differences) and writes results.json and results.md.
"""
import argparse
import json
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import score_catalogue as sc

ROOT = sc.ROOT
MIN_ITEMS = 30
FAMILIES = {"arxiv-gold": "corpus", "msc-gold": "corpus", "jel-gold": "corpus",
            "lcc-gold": "holdout", "lcc-full-gold": "holdout"}
# Gold run and its shelfmark-placed counterpart, also scored on the items
# both place (strictly equal footing: neither leaves an item at the root).
PAIRS = {"arxiv-gold": "arxiv-placed", "msc-gold": "msc-placed",
         "lcc-gold": "lcc-placed-holdout", "lcc-full-gold": "lcc-full-placed-holdout"}
# Scores with bootstrap replicates (edge, vertex and item bootstrap); the
# paired-difference analysis uses these.
BOOTED = ["dasgupta.cost_norm", "vertex.dasgupta.cost_norm", "dasgupta.binarized_cost_norm",
          "tsd.nats", "vertex.tsd.nats", "triplet.accuracy", "vertex.triplet.accuracy",
          "order.locality", "vertex.order.locality", "probe.knn_d1", "gold.ari_fine",
          "gold.v_measure_fine", "gold.dendrogram_purity_fine", "gold.hierarchical_f_fine",
          "gold.dendrogram_purity_coarse"]
DIRECTION = dict(sc.AGREE)
DIRECTION.update({m: DIRECTION.get(m.removeprefix("vertex."), "up") for m in BOOTED})


class Covered(sc.Runs):
    def __init__(self, a):
        super().__init__(a)
        self.root = Path(a.cache) / "covered"
        self.ids = self.root / "ids"
        for d in (self.root, self.ids):
            d.mkdir(parents=True, exist_ok=True)

    def scores(self, spec, out, subset=None, write_ids=None):
        """One `shelfmark-bench scores` call; skipped when `out` exists."""
        key, group, corpus, scheme, corpus_file, placements, compare, harness, stats = spec
        if out.exists():
            return
        out.parent.mkdir(parents=True, exist_ok=True)
        cmd = [self.a.harness, "scores", "--corpus", corpus_file, "--scheme", scheme,
               "--graph", self.graphs[corpus], "--vectors", self.vectors[corpus],
               "--external", self.external(corpus), "--seed", str(self.a.seed),
               "--reps", str(self.a.reps), "--boot", str(self.a.boot), "--output", out]
        if placements:
            cmd += ["--placements", placements]
        if compare:
            cmd += list(compare)
        if subset:
            cmd += ["--subset", subset]
        if write_ids:
            cmd += ["--write-covered", write_ids]
        started = time.monotonic()
        proc = subprocess.run([str(x) for x in cmd], capture_output=True, text=True, check=False)
        if proc.returncode != 0:
            raise RuntimeError(f"{key}: {proc.stderr.strip()[-600:]}")
        result = sc.load(out)
        scores = result["catalogue"]["scores"]
        if subset is None:
            # All items: the catalogue cost must equal the harness result, as
            # in score_catalogue.py; the harness structure statistics apply.
            h = sc.load(harness)
            if abs(scores["dasgupta.cost_norm"] - h["cost_norm"]) > 1e-9:
                raise RuntimeError(f"{key}: cost_norm {scores['dasgupta.cost_norm']} != harness {h['cost_norm']}")
            scores["dasgupta.skill"] = h["skill"]
            for k, v in h["structure"].items():
                scores[f"structure.{k}"] = float(v)
            result["run_harness_tree_sha256"] = h["hashes"]["tree"]
        if stats:
            st = sc.load(stats)
            scores["latency.ms_per_item"] = 1000.0 * st["classify_seconds"] / st["rows"]
        result["run"] = {"key": key, "group": group, "corpus": corpus,
                         "subset": str(subset) if subset else None,
                         "seconds": round(time.monotonic() - started, 1)}
        out.write_text(json.dumps(result, indent=1) + "\n", encoding="utf-8")
        sc.log(f"scored {out.parent.name}/{key} in {result['run']['seconds']}s")

    def count(self, key):
        path = self.ids / f"{key}.txt"
        return sum(1 for line in path.read_text().splitlines() if line.strip())

    def jobs(self):
        """(spec, output, subset ids) for the restricted runs, after `all`."""
        specs = self.specs()
        out = []
        for spec in specs:
            key = spec[0]
            n = self.count(key)
            full = len(sc.jsonl(sc.CORPORA[spec[2]]))
            if MIN_ITEMS <= n < full:
                out.append((spec, self.root / "own" / f"{key}.json", self.ids / f"{key}.txt"))
        for family, corpus in FAMILIES.items():
            if self.count(family) < MIN_ITEMS:
                continue
            for spec in specs:
                if spec[2] == corpus:
                    out.append((spec, self.root / family / f"{spec[0]}.json", self.ids / f"{family}.txt"))
        by_key = {s[0]: s for s in specs}
        for gold, placed in PAIRS.items():
            both = self.ids / f"both-{gold}.txt"
            ids = [set((self.ids / f"{k}.txt").read_text().split()) for k in (gold, placed)]
            both.write_text("".join(f"{i}\n" for i in sorted(ids[0] & ids[1])))
            if self.count(both.stem) >= MIN_ITEMS:
                for k in (gold, placed):
                    out.append((by_key[k], self.root / both.stem / f"{k}.json", both))
        return out


def parallel(jobs, fn, workers):
    failures = {}

    def attempt(job):
        try:
            fn(job)
        except Exception as e:  # noqa: BLE001 - recorded; other runs continue
            failures[str(job[1])] = str(e)
            sc.log(f"FAILED {job[1]}: {e}")
    with ThreadPoolExecutor(workers) as pool:
        list(pool.map(attempt, jobs))
    return failures


def run(a):
    c = Covered(a)
    for corpus in sc.CORPORA:
        c.external(corpus)
    first = [(s, c.root / "all" / f"{s[0]}.json") for s in c.specs()]
    failures = parallel(first, lambda j: c.scores(j[0], j[1], write_ids=c.ids / f"{j[0][0]}.txt"), a.jobs)
    failures.update(parallel(c.jobs(), lambda j: c.scores(j[0], j[1], subset=j[2]), a.jobs))
    (c.root / "failures.json").write_text(json.dumps(failures, indent=2) + "\n")
    sc.log(f"done; {len(failures)} failed")


def dense(a):
    c = Covered(a)
    for corpus, sizes in (("corpus", a.dense_sizes), ("holdout", "500")):
        out = c.root / f"dense-{corpus}.json"
        if out.exists():
            continue
        runs = [{"key": s[0], "scheme": str(s[3]), "corpus": str(s[4]),
                 "placements": str(s[5]) if s[5] else None, "placed": s[1].startswith("placed")}
                for s in c.specs() if s[2] == corpus]
        spec = c.root / f"dense-{corpus}.runs.json"
        spec.write_text(json.dumps(runs, indent=1) + "\n")
        started = time.monotonic()
        subprocess.run([str(x) for x in [
            a.harness, "dense", "--graph", c.graphs[corpus], "--vectors", c.vectors[corpus],
            "--runs", spec, "--sizes", sizes, "--ks", "10,20,50", "--subsets", a.subsets,
            "--boot", a.boot, "--seed", a.seed, "--threads", a.threads, "--output", out]], check=True)
        sc.log(f"dense {corpus} in {time.monotonic() - started:.0f}s")


# ---------------------------------------------------------------- publish

def load_dir(path):
    return {p.stem: sc.load(p) for p in sorted(path.glob("*.json"))} if path.exists() else {}


def agree_call(a, name, runs, metrics, compare=None, control=None, paired=False):
    """`shelfmark-bench agree` over `runs` ({key: result}) for `metrics`."""
    keys = sorted(runs)

    def values(rs, m):
        return [rs[k]["catalogue"]["scores"].get(m.removeprefix("vertex.")) if k in rs else None for k in keys]

    def reps(rs, m):
        return [rs[k]["catalogue"].get("boot", {}).get(m, []) if k in rs else [] for k in keys]
    doc = {"metrics": metrics, "runs": keys, "point": [values(runs, m) for m in metrics],
           "boot": [reps(runs, m) for m in metrics],
           "noise": [[runs[k]["catalogue"]["noise"]["0.2"].get(m) for k in keys] for m in metrics],
           "control": control, "paired": paired}
    if compare is not None:
        doc["compare_point"] = [values(compare, m) for m in metrics]
        doc["compare_boot"] = [reps(compare, m) for m in metrics]
    work = Path(a.cache) / "covered" / "agree"
    work.mkdir(parents=True, exist_ok=True)
    (work / f"{name}.input.json").write_text(json.dumps(doc))
    subprocess.run([str(a.harness), "agree", "--input", str(work / f"{name}.input.json"),
                    "--output", str(work / f"{name}.json")], check=True)
    return sc.load(work / f"{name}.json")


def strip(result):
    cat = dict(result["catalogue"])
    cat.pop("boot", None)
    return dict(result, catalogue=cat)


def analyses(a, allr, own, families, both):
    out = {"groups": {}, "families": {}}
    for group in sc.GROUPS:
        runs = {k: r for k, r in allr.items() if r["run"]["group"] == group and k != "jel-gold"}
        metrics = [m for m, _ in sc.AGREE if sum(m in r["catalogue"]["scores"] for r in runs.values()) >= 4]
        # Covered-subset values: the run's own subset, or all items when it
        # covers everything.
        cov = {k: own.get(k, allr[k]) for k in runs if k in own or allr[k]["catalogue"]["scores"]["coverage"] == 1.0}
        out["groups"][group] = {
            "agree": agree_call(a, f"group-{group}", runs, metrics, compare=cov, control="coverage"),
            # Replicates pair only within one graph: not across the mixed gold group.
            "paired": None if group == "gold" else agree_call(
                a, f"paired-{group}", runs, BOOTED, paired=True)["paired"],
        }
    for family, runs in families.items():
        if not runs:
            continue
        metrics = [m for m, _ in sc.AGREE if m != "coverage"
                   and sum(m in r["catalogue"]["scores"] for r in runs.values()) >= 4]
        placed = {k: r for k, r in runs.items() if r["run"]["group"].startswith("placed")}
        out["families"][family] = {
            "agree_placed": agree_call(a, f"family-{family}", placed, metrics),
            "paired": agree_call(a, f"paired-{family}", runs, BOOTED, paired=True)["paired"],
        }
    out["both"] = {}
    for gold in PAIRS:
        runs = both.get(gold) or {}
        if len(runs) == 2:
            out["both"][gold] = agree_call(a, f"paired-both-{gold}", runs, BOOTED, paired=True)["paired"]
    return out


def publish(a):
    root = Path(a.cache) / "covered"
    allr, own = load_dir(root / "all"), load_dir(root / "own")
    families = {f: load_dir(root / f) for f in FAMILIES}
    counts = {p.stem: sum(1 for x in p.read_text().splitlines() if x.strip()) for p in sorted((root / "ids").glob("*.txt"))}
    dense_res = {c: sc.load(root / f"dense-{c}.json") for c in ("corpus", "holdout") if (root / f"dense-{c}.json").exists()}
    both = {g: load_dir(root / f"both-{g}") for g in PAIRS}
    an = analyses(a, allr, own, families, both)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    cache = Path(a.cache)
    status = {line.split("\t")[0]: line.rstrip("\n").split("\t") for line in (cache / "schemes/status.tsv").open()}
    git = ["git", "-C", str(ROOT)]
    commit = subprocess.run([*git, "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    dirty = bool(subprocess.run([*git, "status", "--porcelain", "--", "crates/bench/src"],
                                capture_output=True, text=True, check=True).stdout.strip())
    doc = {
        "date": "2026-10-10",
        "params": {"seed": a.seed, "boot": a.boot, "reps": a.reps, "min_items": MIN_ITEMS,
                   "dense_sizes": a.dense_sizes, "dense_subsets": a.subsets, "dense_ks": [10, 20, 50]},
        "inputs": {
            "corpus_sha256": {g: sc.sha256(p) for g, p in sc.CORPORA.items()},
            "graphs_sha256": {g: sc.sha256(cache / f"graphs/{g}-k20.json") for g in sc.CORPORA},
            "vectors_sha256": {g: sc.sha256(cache / f"embed/{g}.f32") for g in sc.CORPORA},
            "schemes_sha256": {k: v[2] for k, v in status.items() if v[1] == "ok"},
            "learned_sha256": sc.sha256(ROOT / "schemes/learned/learned.json"),
            "subset_sha256": {k: sc.sha256(root / "ids" / f"{k}.txt") for k in counts},
        },
        "harness": {"base_commit": commit, "source_modified_since_base": dirty,
                    "binary_sha256": sc.sha256(a.harness)},
        "covered_items": counts,
        "runs": {k: {"all": strip(r), "covered": strip(own[k]) if k in own else None} for k, r in allr.items()},
        "families": {f: {"items": counts.get(f), "runs": {k: strip(r) for k, r in rs.items()}}
                     for f, rs in families.items()},
        "both": {g: {"items": counts.get(f"both-{g}"), "runs": {k: strip(r) for k, r in rs.items()}}
                 for g, rs in both.items()},
        "analysis": an,
        "dense": dense_res,
    }
    (out / "results.json").write_text(json.dumps(doc, indent=1) + "\n", encoding="utf-8")
    import covered_report
    (out / "results.md").write_text(covered_report.markdown(a, doc, allr, own, families, both), encoding="utf-8")
    sc.log(f"published {len(allr)} runs, {sum(map(len, families.values()))} family runs to {out}")


def main():
    p = argparse.ArgumentParser()
    p.add_argument("command", choices=["run", "dense", "publish"])
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--harness", required=True)
    p.add_argument("--jobs", type=int, default=6)
    p.add_argument("--threads", type=int, default=8)
    p.add_argument("--seed", type=int, default=20261010)
    p.add_argument("--reps", type=int, default=10)
    p.add_argument("--boot", type=int, default=200)
    p.add_argument("--subsets", type=int, default=5)
    p.add_argument("--dense-sizes", default="1000,2000,5000")
    p.add_argument("--out", default=str(ROOT / "bench/results/2026-10-10-covered"))
    p.add_argument("--notes", help="markdown inserted after the results.md title")
    a = p.parse_args()
    {"run": run, "dense": dense, "publish": publish}[a.command](a)


if __name__ == "__main__":
    main()
