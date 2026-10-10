#!/usr/bin/env python3
"""Run the Paper 1 benchmark: gold and shelfmark-placed placements, then scoring.

Inputs come from the ignored cache (`build_schemes.sh` and
`build_graph_fastembed.py` outputs). Every step writes its output under
`<cache>/runs` and is skipped when that output already exists, so an
interrupted run resumes. `publish` copies the per-run JSON into the results
directory and writes the combined `results.json`.

`--profile` picks the classification profile of the placed runs and `--tag`
keeps a profile's stores, placements and scores under `<cache>/stores/<tag>`,
`<cache>/placed/<tag>` and `<cache>/runs/<tag>`, so several profiles share one
cache. `--bootstrap N` adds item-bootstrap intervals to every score.
"""
import argparse
import hashlib
import json
import os
import queue
import subprocess
import sys
import threading
import time
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PREP = [sys.executable, str(HERE / "prepare_placements.py")]
PROFILE = "profile:lexical@2"
# Gold label sources: scheme -> (corpus, normalizer args). The field names
# are the corpus fields the harness reads for that scheme name.
GOLD = {
    "arxiv": ("corpus", ["normalize-codes", "--field", "categories"]),
    "msc": ("corpus", ["normalize-msc"]),
    "acm-ccs": ("corpus", ["normalize-codes", "--field", "acm_ccs"]),
    "jel": ("corpus", ["normalize-codes", "--field", "jel"]),
    "lcc": ("holdout", ["lcc-deep"]),
    "lcc-full": ("holdout", ["lcc-deep"]),
}
# LCC/DDC-style general schemes, also placed and scored on the book holdout.
GENERAL = ["lcc", "lcc-full", "fdc", "bk", "ilc2", "unesco-thesaurus"]


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}", flush=True)


def run(cmd, **kw):
    proc = subprocess.run([str(c) for c in cmd], capture_output=True, text=True, check=False, **kw)
    if proc.returncode != 0:
        raise RuntimeError(f"{' '.join(map(str, cmd))} failed ({proc.returncode}): "
                           f"{(proc.stderr or proc.stdout).strip()[-800:]}")
    return proc.stdout


def last_json(text):
    return json.loads(text.strip().splitlines()[-1])


class Bench:
    def __init__(self, a):
        self.a = a
        self.cache = Path(a.cache)
        self.profile = a.profile
        tag = [a.tag] if a.tag else []
        self.runs = self.cache.joinpath("runs", *tag)
        self.stores = self.cache.joinpath("stores", *tag)
        self.placements = self.cache.joinpath("placed", *tag)
        self.runs.mkdir(parents=True, exist_ok=True)
        self.status = {line.split("\t")[0]: line.rstrip("\n").split("\t")
                       for line in (self.cache / "schemes" / "status.tsv").open()}
        self.corpora = {"corpus": ROOT / "bench/corpus/corpus.jsonl",
                        "holdout": ROOT / "bench/corpus/holdout.jsonl"}
        self.graphs = {name: self.cache / "graphs" / f"{name}-k20.json" for name in self.corpora}
        for name, graph in self.graphs.items():
            manifest = json.loads(graph.with_name(f"{name}-k20.manifest.json").read_text())
            if sha256(graph) != manifest["graph_sha256"]:
                raise SystemExit(f"{graph}: sha256 does not match its manifest")
        # One job per scheme at a time: its store is not shared between writers.
        self.locks = defaultdict(threading.Lock)
        # With --cpus-per-job, every batch runs pinned to its own CPUs.
        self.slots = None
        if a.cpus_per_job:
            cpus = sorted(os.sched_getaffinity(0))
            n = a.cpus_per_job
            self.slots = queue.Queue()
            for i in range(0, len(cpus) - n + 1, n):
                self.slots.put(",".join(map(str, cpus[i:i + n])))

    def scheme_file(self, name):
        return self.cache / "schemes" / f"{name}.json"

    def scheme_id(self, name):
        doc = json.loads(self.scheme_file(name).read_text(encoding="utf-8"))
        return f"scheme:{doc['name']}@{doc['version']}"

    def built(self):
        return [n for n, row in self.status.items() if row[1] == "ok"]

    def store(self, name):
        """An isolated store holding only this scheme and an empty rules file."""
        store = self.stores / name
        marker = store / "compiled.json"
        if marker.exists():
            return store
        store.mkdir(parents=True, exist_ok=True)
        (store / "rules.json").write_text('{"rules":[]}\n', encoding="utf-8")
        base = [self.a.shelfmark, "--json", "--store", store]
        preview = json.loads(run(base + ["scheme", "compile", self.scheme_file(name)]))
        out = json.loads(run(base + ["--commit", "--expected-revision", preview["data"]["revision"],
                                     "scheme", "compile", self.scheme_file(name)]))
        marker.write_text(json.dumps(out["data"], indent=2) + "\n", encoding="utf-8")
        return store

    def gold(self, name):
        """Normalized gold corpus for `name`, or None when no row has a valid label."""
        corpus, how = GOLD[name]
        out = self.cache / "gold" / f"{corpus}-{name}.jsonl"
        stats_path = out.with_suffix(".stats.json")
        if not stats_path.exists():
            out.parent.mkdir(parents=True, exist_ok=True)
            if how[0] == "lcc-deep":
                cmd = PREP + ["normalize-lcc-deep", "--shelfmark", self.a.shelfmark,
                              "--store", self.store(name), "--scheme-id", self.scheme_id(name)]
                if name != "lcc":
                    cmd += ["--gold-key", name]
            else:
                cmd = PREP + how + ["--scheme", self.scheme_file(name)]
            stats = last_json(run(cmd + ["--corpus", self.corpora[corpus], "--output", out]))
            stats_path.write_text(json.dumps(stats, indent=2) + "\n", encoding="utf-8")
        stats = json.loads(stats_path.read_text(encoding="utf-8"))
        placed = next(v for k, v in stats.items() if k.startswith("rows_with"))
        return (out, stats) if placed else (None, stats)

    def placed(self, name, corpus):
        """shelfmark placements for `name` over `corpus`."""
        work = self.placements / f"{corpus}-{name}"
        out = work / "placements.jsonl"
        stats_path = work / "stats.json"
        if stats_path.exists():
            return out, json.loads(stats_path.read_text(encoding="utf-8"))
        work.mkdir(parents=True, exist_ok=True)
        store = self.store(name)
        scheme_id = self.scheme_id(name)
        requests, batch = work / "requests.jsonl", work / "batch.jsonl"
        run(PREP + ["requests", "--corpus", self.corpora[corpus], "--scheme", scheme_id,
                    "--profile", self.profile, "--output", requests])
        timing = work / "timing.json"
        started = time.monotonic()
        resume = ["--resume"] if batch.exists() else []
        cmd = [self.a.shelfmark, "--json", "--store", store, "batch", "--requests", requests,
               "--out", batch] + resume
        if self.slots is None:
            run(cmd)
        else:
            slot = self.slots.get()
            try:
                run(["taskset", "-c", slot] + cmd)
            finally:
                self.slots.put(slot)
        # Keep the first complete batch's time; a resumed rerun skips the work.
        if not timing.exists():
            timing.write_text(json.dumps({"classify_seconds": round(time.monotonic() - started, 1)}))
        seconds = json.loads(timing.read_text())["classify_seconds"]
        stats = last_json(run(PREP + ["extract", "--corpus", self.corpora[corpus], "--batch", batch,
                                      "--scheme", scheme_id, "--profile", self.profile,
                                      "--output", out]))
        stats.update(classify_seconds=seconds, profile=self.profile, store_rules="empty")
        stats_path.write_text(json.dumps(stats, indent=2) + "\n", encoding="utf-8")
        return out, stats

    def reference(self, corpus):
        """Average-linkage baseline cost_norm for a graph, computed once."""
        path = self.runs / f"reference-{corpus}.json"
        if not path.exists() and self.a.reference_from:
            # Reuse a published baseline; it is only valid for the same graph.
            ref = json.loads(Path(self.a.reference_from).read_text())["graphs"][corpus]["reference"]
            if ref["graph_sha256"] != sha256(self.graphs[corpus]):
                raise SystemExit(f"{self.a.reference_from}: {corpus} reference is for another graph")
            ref = dict(ref, source=self.a.reference_from)
            path.write_text(json.dumps(ref, indent=2) + "\n", encoding="utf-8")
        if not path.exists():
            # Any scheme without a gold field works: only the reference score is kept.
            started = time.monotonic()
            result = json.loads(run([self.a.harness, "run", "--corpus", self.corpora[corpus],
                                     "--scheme", self.scheme_file("fdc"), "--graph",
                                     self.graphs[corpus], "--k", "20"]))
            ref = {"corpus": corpus, "ref_cost_norm": result["ref_cost_norm"],
                   "graph_sha256": result["hashes"]["graph"],
                   "seconds": round(time.monotonic() - started, 1)}
            path.write_text(json.dumps(ref, indent=2) + "\n", encoding="utf-8")
        return json.loads(path.read_text(encoding="utf-8"))["ref_cost_norm"]

    def score(self, key, name, corpus, source, corpus_path, placements=None, stats=None):
        path = self.runs / f"{key}.json"
        if path.exists():
            return
        cmd = [self.a.harness, "run", "--corpus", corpus_path, "--scheme", self.scheme_file(name),
               "--graph", self.graphs[corpus], "--k", "20",
               "--reference-cost", repr(self.reference(corpus))]
        if placements:
            cmd += ["--placements", placements]
        if self.a.bootstrap:
            cmd += ["--bootstrap", self.a.bootstrap, "--seed", self.a.seed]
            if self.a.score_threads:
                cmd += ["--threads", self.a.score_threads]
        started = time.monotonic()
        result = json.loads(run(cmd))
        result["run"] = {"scheme_id": self.scheme_id(name), "corpus": corpus,
                         "placement_source": source, "placement_stats": stats,
                         "score_seconds": round(time.monotonic() - started, 1)}
        path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        log(f"scored {key}: cost_norm={result['cost_norm']:.4f} coverage={result['coverage']:.4f}")

    def jobs(self):
        """(scheme, key, thunk) triples for every gold and placed result."""
        built = self.built()
        if self.a.only:
            built = [n for n in built if n in self.a.only]
        jobs = []
        for name in [] if self.a.placed_only else GOLD:
            if name in built:
                jobs.append((name, f"{name}-gold", lambda n=name: self.gold_job(n)))
        placed = [] if self.a.gold_only else built
        for name in [] if self.a.holdout_only else placed:
            jobs.append((name, f"{name}-placed", lambda n=name: self.placed_job(n, "corpus")))
        # Holdout jobs last: queued next to the same scheme's corpus job, a
        # worker would only wait on that scheme's store lock.
        for name in placed:
            if name in GENERAL:
                jobs.append((name, f"{name}-placed-holdout",
                             lambda n=name: self.placed_job(n, "holdout")))
        return jobs

    def gold_job(self, name):
        corpus = GOLD[name][0]
        path, stats = self.gold(name)
        if path is None:
            skip = self.runs / f"{name}-gold.skipped.json"
            skip.write_text(json.dumps({"scheme": name, "corpus": corpus, "stats": stats}) + "\n")
            log(f"skipped {name}-gold: no row has a valid label")
            return
        self.score(f"{name}-gold", name, corpus, "gold", path, stats=stats)

    def placed_job(self, name, corpus):
        suffix = "" if corpus == "corpus" else "-holdout"
        if (self.runs / f"{name}-placed{suffix}.json").exists():
            return
        placements, stats = self.placed(name, corpus)
        self.score(f"{name}-placed{suffix}", name, corpus, f"shelfmark {self.profile}",
                   self.corpora[corpus], placements, stats)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--shelfmark", required=True)
    p.add_argument("--harness", required=True)
    p.add_argument("--jobs", type=int, default=4)
    p.add_argument("--placed-only", action="store_true",
                   help="skip the gold runs (they do not depend on the classifier)")
    p.add_argument("--gold-only", action="store_true",
                   help="skip the shelfmark-placed runs; only the source labels are scored")
    p.add_argument("--reference-from", metavar="RESULTS_JSON",
                   help="take the average-linkage baselines from a published results.json "
                        "instead of recomputing them; the graph hashes must match")
    p.add_argument("--profile", default=PROFILE,
                   help=f"classification profile of the placed runs (default {PROFILE})")
    p.add_argument("--tag", help="keep stores, placements and scores under this subdirectory")
    p.add_argument("--bootstrap", type=int, metavar="N",
                   help="add N-resample item-bootstrap intervals to every score")
    p.add_argument("--seed", type=int, default=1, help="bootstrap seed (default 1)")
    p.add_argument("--score-threads", type=int, metavar="N",
                   help="bootstrap threads per scoring run (default: every available CPU)")
    p.add_argument("--cpus-per-job", type=int, metavar="N",
                   help="pin each shelfmark batch to its own N CPUs (a model tier sizes its "
                        "threads from them); at most --jobs batches run at once either way")
    p.add_argument("--only", nargs="+", metavar="SCHEME",
                   help="limit the runs to these schemes (default: every built scheme)")
    p.add_argument("--holdout-only", action="store_true",
                   help="only the holdout placed runs of the general schemes")
    a = p.parse_args()
    if a.gold_only and a.placed_only:
        p.error("--gold-only and --placed-only exclude each other")
    bench = Bench(a)
    started = time.monotonic()
    for corpus in bench.corpora:
        log(f"reference {corpus}: {bench.reference(corpus)}")
    failures = {}

    def attempt(item):
        name, key, thunk = item
        try:
            with bench.locks[name]:
                thunk()
        except Exception as e:  # noqa: BLE001 -- recorded and reported; other jobs continue
            failures[key] = str(e)
            log(f"FAILED {key}: {e}")

    with ThreadPoolExecutor(a.jobs) as pool:
        list(pool.map(attempt, bench.jobs()))
    (bench.runs / "failures.json").write_text(json.dumps(failures, indent=2) + "\n")
    log(f"done in {time.monotonic() - started:.0f}s; {len(failures)} failed")


if __name__ == "__main__":
    main()
