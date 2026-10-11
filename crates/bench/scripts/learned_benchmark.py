#!/usr/bin/env python3
"""Score the learned scheme (schemes/learned) against the Paper 1 schemes.

Runs, all on the published 2026-10-10 graphs and scored by shelfmark-bench:

1. own: the learned scheme's own placements of the corpus it was fitted on
   (in-sample), and the holdout inserted as new works (nearest centroid,
   leaf splits) on the holdout graph;
2. placed: the learned scheme placed by shelfmark (`profile:lexical@2`) like
   every Paper 1 placed run;
3. split: fit on 80% of the corpus, insert the other 20% as new works, and
   score the 20% on its induced graph next to every Paper 1 scheme's
   placements (and gold labels) restricted to the same 20%, the fully fitted
   learned scheme (in-sample), and the average-linkage tree of the 20%.

Needs the Paper 1 cache from run_benchmark.py (scheme files, placements,
gold corpora) and the embedding vectors from build_graph_fastembed.py
--vectors. Every step caches its output under <cache>/learned-bench.
"""
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE))
import run_benchmark  # noqa: E402, RUF100

LEARNED = ROOT / "schemes/learned"
SPLIT_SEED = "learned-split-2026-10-10"
LAMBDA = 0.005  # the learned objective's navigation weight, for J of every run
PAPER1_GOLD = ["arxiv", "msc", "jel"]  # corpus gold; acm-ccs has no valid label


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}", flush=True)


def run(cmd):
    proc = subprocess.run([str(c) for c in cmd], capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        raise SystemExit(f"{' '.join(map(str, cmd))} failed: {(proc.stderr or proc.stdout)[-800:]}")
    return proc.stdout


def lines_for(path, ids):
    keep = set(ids)
    return [line for line in Path(path).read_text().splitlines()
            if line.strip() and json.loads(line)["id"] in keep]


class Learned:
    def __init__(self, a):
        self.a = a
        self.cache = Path(a.cache)
        self.work = self.cache / "learned-bench"
        self.work.mkdir(parents=True, exist_ok=True)
        self.graphs = {g: self.cache / "graphs" / f"{g}-k20.json" for g in ("corpus", "holdout")}
        self.corpora = {"corpus": ROOT / "bench/corpus/corpus.jsonl", "holdout": ROOT / "bench/corpus/holdout.jsonl"}
        self.refs = {g: json.loads((self.cache / "runs" / f"reference-{g}.json").read_text())["ref_cost_norm"]
                     for g in ("corpus", "holdout")}
        self.results = {}

    def learned(self, *args):
        return run([self.a.learned, *args])

    def score(self, key, scheme, corpus, graph, placements=None, ref=None, meta=None):
        out = self.work / "runs" / f"{key}.json"
        if not out.exists():
            out.parent.mkdir(exist_ok=True)
            cmd = [self.a.harness, "run", "--corpus", corpus, "--scheme", scheme, "--graph", graph, "--k", "20"]
            if ref is not None:
                cmd += ["--reference-cost", repr(ref)]
            if placements:
                cmd += ["--placements", placements]
            started = time.monotonic()
            result = json.loads(run(cmd))
            result["run"] = dict(meta or {}, score_seconds=round(time.monotonic() - started, 1))
            out.write_text(json.dumps(result, indent=2) + "\n")
            log(f"scored {key}: cost_norm={result['cost_norm']:.4f}")
        self.results[key] = json.loads(out.read_text())
        return self.results[key]

    def own(self):
        """Full fit (must equal the committed file) and holdout insertion."""
        w = self.work
        scheme = w / "learned.json"
        if not scheme.exists():
            self.learned("build", "--corpus", self.corpora["corpus"], "--graph", self.a.graph_gz,
                         "--labels", LEARNED / "labels.tsv", "--out", scheme,
                         "--placements", w / "corpus-placements.jsonl", "--report", w / "build-report.json")
        if scheme.read_bytes() != (LEARNED / "learned.json").read_bytes():
            raise SystemExit("rebuilt learned.json differs from the committed schemes/learned/learned.json")
        if not (w / "insert-report.json").exists():
            self.learned("insert", "--scheme", scheme, "--centroids", LEARNED / "data/centroids.json.gz",
                         "--corpus", self.corpora["holdout"], "--vectors", LEARNED / "data/holdout-vectors.json.gz",
                         "--out", w / "learned-holdout.json", "--placements", w / "holdout-placements.jsonl",
                         "--report", w / "insert-report.json")
        meta = {"placement_source": "own (fitted on this corpus: in-sample)"}
        self.score("learned-own", scheme, self.corpora["corpus"], self.graphs["corpus"],
                   w / "corpus-placements.jsonl", self.refs["corpus"], meta)
        meta = {"placement_source": "own insertion (nearest leaf centroid, leaf splits)"}
        self.score("learned-own-holdout", w / "learned-holdout.json", self.corpora["holdout"],
                   self.graphs["holdout"], w / "holdout-placements.jsonl", self.refs["holdout"], meta)

    def placed(self):
        """shelfmark lexical placements, through the Paper 1 runner."""
        schemes = self.cache / "schemes"
        target = schemes / "learned.json"
        if not target.exists() or target.read_bytes() != (LEARNED / "learned.json").read_bytes():
            shutil.copy(LEARNED / "learned.json", target)
        status = schemes / "status.tsv"
        rows = [line for line in status.read_text().splitlines() if not line.startswith("learned\t")]
        rows.append(f"learned\tok\t{sha256(target)}")
        status.write_text("\n".join(rows) + "\n")
        bench = run_benchmark.Bench(argparse.Namespace(cache=str(self.cache), shelfmark=self.a.shelfmark,
                                                       harness=self.a.harness, reference_from=None))
        for corpus, suffix in (("corpus", ""), ("holdout", "-holdout")):
            placements, stats = bench.placed("learned", corpus)
            meta = {"placement_source": f"shelfmark {run_benchmark.PROFILE}", "placement_stats": stats}
            self.score(f"learned-placed{suffix}", target, self.corpora[corpus], self.graphs[corpus],
                       placements, self.refs[corpus], meta)

    def split(self):
        w = self.work / "split"
        w.mkdir(exist_ok=True)
        corpus = self.corpora["corpus"]
        if not (w / "test-ids.txt").exists():
            self.learned("split-ids", "--corpus", corpus, "--fraction", "0.2", "--seed", SPLIT_SEED,
                         "--train", w / "train-ids.txt", "--test", w / "test-ids.txt")
        test = (w / "test-ids.txt").read_text().split()
        train = (w / "train-ids.txt").read_text().split()
        if not (w / "split.json").exists():
            self.learned("build", "--corpus", corpus, "--graph", self.a.graph_gz, "--subset", w / "train-ids.txt",
                         "--vectors", self.a.vectors, "--allow-auto-labels", "--version", "2026-10-10-split80",
                         "--out", w / "split.json", "--centroids", w / "centroids.json.gz",
                         "--placements", w / "train-placements.jsonl", "--report", w / "build-report.json")
        if not (w / "insert-report.json").exists():
            self.learned("quantize", "--corpus", corpus, "--vectors", self.a.vectors, "--ids", w / "test-ids.txt",
                         "--out", w / "test-vectors.json.gz")
            self.learned("insert", "--scheme", w / "split.json", "--centroids", w / "centroids.json.gz",
                         "--corpus", corpus, "--ids", w / "test-ids.txt", "--vectors", w / "test-vectors.json.gz",
                         "--base-placements", w / "train-placements.jsonl", "--out", w / "split-inserted.json",
                         "--placements", w / "all-placements.jsonl", "--report", w / "insert-report.json")
        for part, ids in (("test", test), ("train", train)):
            if not (w / f"{part}-graph.json").exists():
                self.learned("subset", "--corpus", corpus, "--graph", self.graphs["corpus"], "--ids",
                             w / f"{part}-ids.txt", "--out-corpus", w / f"{part}-corpus.jsonl",
                             "--out-graph", w / f"{part}-graph.json")

        def sub(name, src, ids):
            (w / name).write_text("".join(line + "\n" for line in lines_for(src, ids)))

        sub("test-placements.jsonl", w / "all-placements.jsonl", test)
        sub("test-full-placements.jsonl", self.work / "corpus-placements.jsonl", test)
        tg, tc = w / "test-graph.json", w / "test-corpus.jsonl"
        # Average-linkage reference of the 20% graph, computed once.
        held = self.score("learned-split-heldout", w / "split-inserted.json", tc, tg, w / "test-placements.jsonl",
                          None, {"placement_source": "own insertion into the scheme fitted on the other 80% (held out)"})
        ref = held["ref_cost_norm"]
        self.score("learned-split-train", w / "split.json", w / "train-corpus.jsonl", w / "train-graph.json",
                   w / "train-placements.jsonl", None,
                   {"placement_source": "own (the 80% the split scheme was fitted on: in-sample)"})
        self.score("learned-split-full-corpus", w / "split-inserted.json", corpus, self.graphs["corpus"],
                   w / "all-placements.jsonl", self.refs["corpus"],
                   {"placement_source": "own: 80% fitted, 20% inserted, scored on the whole corpus graph"})
        self.score("test-learned-own", LEARNED / "learned.json", tc, tg, w / "test-full-placements.jsonl", ref,
                   {"placement_source": "own placements of the full fit (in-sample) restricted to the 20%"})
        placed_dir = self.cache / "placed"
        for scheme_file in sorted((self.cache / "schemes").glob("*.json")):
            name = scheme_file.stem
            src = placed_dir / f"corpus-{name}" / "placements.jsonl"
            if not src.exists():
                continue
            sub(f"test-{name}-placed.jsonl", src, test)
            self.score(f"test-{name}-placed", scheme_file, tc, tg, w / f"test-{name}-placed.jsonl", ref,
                       {"placement_source": f"shelfmark {run_benchmark.PROFILE}, restricted to the 20%"})
        for name in PAPER1_GOLD:
            gold = self.cache / "gold" / f"corpus-{name}.jsonl"
            if gold.exists():
                sub(f"test-{name}-gold-corpus.jsonl", gold, test)
                self.score(f"test-{name}-gold", self.cache / "schemes" / f"{name}.json",
                           w / f"test-{name}-gold-corpus.jsonl", tg, None, ref,
                           {"placement_source": "gold (source labels), restricted to the 20%"})


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--shelfmark", required=True)
    p.add_argument("--harness", required=True)
    p.add_argument("--learned", required=True, help="learned-scheme binary")
    p.add_argument("--vectors", default=str(ROOT / "bench/corpus/cache/embed/corpus.f32"))
    p.add_argument("--graph-gz", default=str(ROOT / "bench/results/2026-10-10/graphs/corpus-k20.json.gz"))
    a = p.parse_args()
    lb = Learned(a)
    started = time.monotonic()
    lb.own()
    lb.placed()
    lb.split()
    log(f"done in {time.monotonic() - started:.0f}s; {len(lb.results)} runs")


if __name__ == "__main__":
    main()
