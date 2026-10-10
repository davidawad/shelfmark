#!/usr/bin/env python3
"""Cross-embedding robustness: rescore every placement on graphs from other models and k.

The placements are embedding-free inputs; only the similarity graph changes.
For each condition (model, k) this scores, with item-bootstrap intervals:

- corpus: the gold labels (arxiv, msc, jel), every shelfmark-placed scheme,
  the learned scheme's own placements (fitted on all-MiniLM-L6-v2) and the
  learned schemes refitted on each other model;
- holdout: the LCC gold labels, the placed general schemes, and each learned
  scheme's insertion of the 500 books (with its own model's vectors);
- split: the 2,000 test works of the learned 80/20 split, on the graph the
  condition induces on them: the held-out insertion of each 80% fit, the
  full fit (in-sample) and every other placement restricted to them.

Needs the Paper 1 cache from run_benchmark.py, the learned cache from
learned_benchmark.py and the graphs and vectors of build_graph_fastembed.py
(see bench/results/2026-10-10-embed/REPRODUCE.md). Every step caches its
output under <cache>/embed-bench and is skipped when the output exists.
"""
import argparse
import json
import subprocess
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
MODELS = ["minilm", "bge-small", "nomic"]
KS = [10, 20, 50]
REFIT = ["bge-small", "nomic"]  # learned schemes refitted on these models (k = 20)
GOLD_CORPUS = ["arxiv", "msc", "jel"]  # acm-ccs has no valid label
GOLD_HOLDOUT = ["lcc", "lcc-full"]
GENERAL = ["lcc", "lcc-full", "fdc", "bk", "ilc2", "unesco-thesaurus"]
RESAMPLES = 1000
MODEL_NAMES = {
    "bge-small": "BAAI/bge-small-en-v1.5 (Qdrant/bge-small-en-v1.5-onnx-Q "
                 "aa8f8b060edb00e03bfdd08813a2949946c8ba55, fastembed 0.9.0)",
    "nomic": "nomic-ai/nomic-embed-text-v1.5 (nomic-ai/nomic-embed-text-v1.5 "
             "e9b6763023c676ca8431644204f50c2b100d9aab onnx/model.onnx, fastembed 0.9.0, "
             "truncated at 512 tokens)",
}
# The harness field each gold scheme name reads (shelfmark-bench load_placements).
GOLD_FIELDS = {"arxiv": ["categories"], "msc": ["msc_classes", "msc"], "jel": ["jel_codes", "jel"],
               "lcc": ["lc_classifications"]}


def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}", flush=True)


def run(cmd):
    proc = subprocess.run([str(c) for c in cmd], capture_output=True, text=True)
    if proc.returncode != 0:
        raise RuntimeError(f"{' '.join(map(str, cmd))} failed ({proc.returncode}): "
                           f"{(proc.stderr or proc.stdout).strip()[-800:]}")
    return proc.stdout


def read_jsonl(path):
    return [json.loads(line) for line in Path(path).read_text(encoding="utf-8").splitlines() if line.strip()]


def write_lines(path, lines):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text("".join(line + "\n" for line in lines), encoding="utf-8")


def restrict(src, dst, ids):
    keep = set(ids)
    write_lines(dst, [line for line in Path(src).read_text().splitlines()
                      if line.strip() and json.loads(line)["id"] in keep])


def condition(model, k):
    return f"{model}-k{k}"


class Embed:
    def __init__(self, a):
        self.a = a
        self.cache = Path(a.cache)
        self.work = self.cache / "embed-bench"
        self.learned_work = self.cache / "learned-bench"
        self.place_dir = self.work / "placements"
        self.corpora = {"corpus": ROOT / "bench/corpus/corpus.jsonl",
                        "holdout": ROOT / "bench/corpus/holdout.jsonl"}
        self.split_dir = self.learned_work / "split"
        self.test_ids = (self.split_dir / "test-ids.txt").read_text().split()
        self.train_ids = self.split_dir / "train-ids.txt"
        self.test_corpus = self.work / "split" / "test-corpus.jsonl"
        self.locks = {}
        self.lock = threading.Lock()

    # -- inputs -------------------------------------------------------------
    def graph(self, model, k, corpus):
        if model == "minilm" and k == 20:
            return self.cache / "graphs" / f"{corpus}-k20.json"  # the published graph
        return self.cache / "embed2" / f"{model}-{corpus}-k{k}.json"

    def vectors(self, model, corpus):
        return self.cache / "embed2" / f"{model}-{corpus}.f32"

    def split_graph(self, model, k):
        out = self.work / "split" / condition(model, k) / "test-graph.json"
        with self.lock:  # every condition rewrites the same test corpus
            if out.exists():
                return out
            out.parent.mkdir(parents=True, exist_ok=True)
            run([self.a.learned, "subset", "--corpus", self.corpora["corpus"], "--graph",
                 self.graph(model, k, "corpus"), "--ids", self.split_dir / "test-ids.txt",
                 "--out-corpus", self.test_corpus, "--out-graph", out])
        return out

    def gold_placements(self, name, corpus):
        """Gold labels as a placement file: the first label the harness would read."""
        out = self.place_dir / f"{corpus}-{name}-gold.jsonl"
        if not out.exists():
            fields = GOLD_FIELDS.get(name)
            lines = []
            for row in read_jsonl(self.cache / "gold" / f"{corpus}-{name}.jsonl"):
                labels = next((row[f] for f in fields if row.get(f)), []) if fields else \
                    row.get("gold", {}).get(name, [])
                if labels:
                    lines.append(json.dumps({"id": row["id"], "classes": [{"code": labels[0], "mass": 1.0}]}))
            write_lines(out, lines)
        return out

    # -- learned refits -----------------------------------------------------
    def refit(self, m):
        """Fit the learned scheme on model m's k = 20 graph; insert the holdout and the 20% split."""
        w = self.work / "refit" / m
        w.mkdir(parents=True, exist_ok=True)
        L, corpus, holdout = self.a.learned, self.corpora["corpus"], self.corpora["holdout"]
        graph, vec, name = self.graph(m, 20, "corpus"), self.vectors(m, "corpus"), MODEL_NAMES[m]
        if not (w / "build-report.json").exists():
            run([L, "build", "--corpus", corpus, "--graph", graph, "--vectors", vec, "--allow-auto-labels",
                 "--version", f"2026-10-10-{m}", "--model", name, "--out", w / f"learned-{m}.json",
                 "--centroids", w / "centroids.json.gz", "--placements", w / "corpus-placements.jsonl",
                 "--report", w / "build-report.json"])
        if not (w / "insert-report.json").exists():
            run([L, "quantize", "--corpus", holdout, "--vectors", self.vectors(m, "holdout"),
                 "--model", name, "--out", w / "holdout-vectors.json.gz"])
            run([L, "insert", "--scheme", w / f"learned-{m}.json", "--centroids", w / "centroids.json.gz",
                 "--corpus", holdout, "--vectors", w / "holdout-vectors.json.gz",
                 "--out", w / f"learned-{m}-holdout.json", "--placements", w / "holdout-placements.jsonl",
                 "--report", w / "insert-report.json"])
        if not (w / "split-build-report.json").exists():
            run([L, "build", "--corpus", corpus, "--graph", graph, "--subset", self.train_ids,
                 "--vectors", vec, "--allow-auto-labels", "--version", f"2026-10-10-{m}-split80",
                 "--model", name, "--out", w / f"learned-{m}-split80.json",
                 "--centroids", w / "split-centroids.json.gz", "--placements", w / "train-placements.jsonl",
                 "--report", w / "split-build-report.json"])
        if not (w / "split-insert-report.json").exists():
            run([L, "quantize", "--corpus", corpus, "--vectors", vec, "--ids", self.split_dir / "test-ids.txt",
                 "--model", name, "--out", w / "test-vectors.json.gz"])
            run([L, "insert", "--scheme", w / f"learned-{m}-split80.json", "--centroids",
                 w / "split-centroids.json.gz", "--corpus", corpus, "--ids", self.split_dir / "test-ids.txt",
                 "--vectors", w / "test-vectors.json.gz", "--base-placements", w / "train-placements.jsonl",
                 "--out", w / f"learned-{m}-split80-inserted.json", "--placements",
                 w / "split-all-placements.jsonl", "--report", w / "split-insert-report.json"])
        restrict(w / "split-all-placements.jsonl", w / "split-test-placements.jsonl", self.test_ids)
        return w

    # -- what to score ------------------------------------------------------
    def sources(self):
        """(universe, key, scheme file, placements, track, label) for every scored placement."""
        schemes, placed, lw = self.cache / "schemes", self.cache / "placed", self.learned_work
        built = sorted(p.stem for p in schemes.glob("*.json"))
        out = []
        for name in GOLD_CORPUS:
            out.append(("corpus", f"{name}-gold", schemes / f"{name}.json",
                        self.gold_placements(name, "corpus"), "A", "gold (source labels)"))
        for name in built:
            out.append(("corpus", f"{name}-placed", schemes / f"{name}.json",
                        placed / f"corpus-{name}" / "placements.jsonl", "P", "shelfmark lexical@2"))
        out.append(("corpus", "learned-own", lw / "learned.json", lw / "corpus-placements.jsonl", "P",
                    "own, fitted on MiniLM k20 (in-sample works)"))
        for name in GOLD_HOLDOUT:
            out.append(("holdout", f"{name}-gold", schemes / f"{name}.json",
                        self.gold_placements(name, "holdout"), "A", "gold (source labels)"))
        for name in GENERAL + ["learned"]:
            out.append(("holdout", f"{name}-placed", schemes / f"{name}.json",
                        placed / f"holdout-{name}" / "placements.jsonl", "P", "shelfmark lexical@2"))
        out.append(("holdout", "learned-own", lw / "learned-holdout.json", lw / "holdout-placements.jsonl",
                    "P", "own insertion, MiniLM fit and vectors"))
        sd = self.split_dir
        out.append(("split", "learned-heldout", sd / "split-inserted.json", sd / "test-placements.jsonl", "P",
                    "own insertion into the MiniLM 80% fit (held out)"))
        out.append(("split", "learned-own", lw / "learned.json", sd / "test-full-placements.jsonl", "P",
                    "own, MiniLM full fit (in-sample works)"))
        for universe, key, scheme, placements, track, label in list(out):
            if universe == "corpus" and not key.startswith("learned"):
                dst = self.place_dir / "split" / f"test-{key}.jsonl"
                if not dst.exists():
                    restrict(placements, dst, self.test_ids)
                out.append(("split", key, scheme, dst, track, label + ", restricted to the 20%"))
        for m in REFIT:
            w = self.work / "refit" / m
            out.append(("corpus", f"learned-{m}-own", w / f"learned-{m}.json", w / "corpus-placements.jsonl",
                        "P", f"own, refitted on {m} k20 (in-sample works)"))
            out.append(("holdout", f"learned-{m}-own", w / f"learned-{m}-holdout.json",
                        w / "holdout-placements.jsonl", "P", f"own insertion, {m} fit and vectors"))
            out.append(("split", f"learned-{m}-heldout", w / f"learned-{m}-split80-inserted.json",
                        w / "split-test-placements.jsonl", "P", f"own insertion into the {m} 80% fit (held out)"))
        return out

    # -- scoring ------------------------------------------------------------
    def universe(self, model, k, universe):
        """(corpus file, graph) of one universe under one condition."""
        if universe == "split":
            return self.test_corpus, self.split_graph(model, k)
        return self.corpora[universe], self.graph(model, k, universe)

    def lock_for(self, key):
        with self.lock:
            return self.locks.setdefault(key, threading.Lock())

    def reference(self, model, k, universe):
        """Average-linkage baseline of a graph and its bootstrap replicates, computed once."""
        d = self.work / "runs" / condition(model, k) / universe
        path, reps = d / "reference.json", d / "reference.reps.json"
        with self.lock_for(str(path)):
            if not path.exists():
                d.mkdir(parents=True, exist_ok=True)
                corpus, graph = self.universe(model, k, universe)
                started = time.monotonic()
                result = json.loads(run([self.a.harness, "run", "--corpus", corpus, "--scheme",
                                         self.cache / "schemes" / "fdc.json", "--graph", graph, "--k", k,
                                         "--bootstrap", RESAMPLES, "--replicates-out", reps]))
                ref = {"ref_cost_norm": result["ref_cost_norm"], "graph_sha256": result["hashes"]["graph"],
                       "ci": result["bootstrap"]["ci"]["ref_cost_norm"], "n": result["n"],
                       "seconds": round(time.monotonic() - started, 1)}
                path.write_text(json.dumps(ref, indent=2) + "\n")
                log(f"reference {condition(model, k)}/{universe}: {ref['ref_cost_norm']:.4f}")
        return json.loads(path.read_text()), reps

    def score(self, model, k, source):
        universe, key, scheme, placements, track, label = source
        d = self.work / "runs" / condition(model, k) / universe
        out = d / f"{key}.json"
        if out.exists():
            return
        ref, reps = self.reference(model, k, universe)
        corpus, graph = self.universe(model, k, universe)
        started = time.monotonic()
        result = json.loads(run([self.a.harness, "run", "--corpus", corpus, "--scheme", scheme,
                                 "--graph", graph, "--k", k, "--placements", placements, "--track", track,
                                 "--reference-cost", repr(ref["ref_cost_norm"]), "--reference-replicates", reps,
                                 "--bootstrap", RESAMPLES, "--replicates-out", d / f"{key}.reps.json"]))
        result["run"] = {"condition": condition(model, k), "model": model, "graph_k": k, "universe": universe,
                         "key": key, "placement_source": label,
                         "placements": str(Path(placements).relative_to(ROOT)),
                         "score_seconds": round(time.monotonic() - started, 1)}
        out.write_text(json.dumps(result, indent=2) + "\n")
        log(f"scored {condition(model, k)}/{universe}/{key}: cost_norm={result['cost_norm']:.4f}")


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--harness", required=True)
    p.add_argument("--learned", required=True, help="learned-scheme binary")
    p.add_argument("--jobs", type=int, default=6)
    p.add_argument("--models", default=",".join(MODELS))
    a = p.parse_args()
    e = Embed(a)
    started = time.monotonic()
    for m in REFIT:
        e.refit(m)
        log(f"refit {m} ready")
    sources = e.sources()
    missing = [s for s in sources if not Path(s[2]).exists() or not Path(s[3]).exists()]
    if missing:
        raise SystemExit(f"missing inputs: {[(s[0], s[1]) for s in missing]}")
    conds = [(m, k) for m in a.models.split(",") for k in KS]
    refs = [(m, k, u) for m, k in conds for u in ("corpus", "holdout", "split")]
    failures = {}

    def attempt(fn, label, *args):
        try:
            fn(*args)
        except Exception as err:  # recorded and reported; other jobs continue
            failures[label] = str(err)
            log(f"FAILED {label}: {err}")

    with ThreadPoolExecutor(a.jobs) as pool:
        list(pool.map(lambda r: attempt(e.reference, f"reference {r}", *r), refs))
    with ThreadPoolExecutor(a.jobs) as pool:
        list(pool.map(lambda job: attempt(e.score, f"{condition(*job[:2])}/{job[2][0]}/{job[2][1]}", *job),
                      [(m, k, s) for m, k in conds for s in sources]))
    (e.work / "failures.json").write_text(json.dumps(failures, indent=2) + "\n")
    log(f"done in {time.monotonic() - started:.0f}s; {len(conds) * len(sources)} runs, {len(failures)} failed")
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
