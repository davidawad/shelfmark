#!/usr/bin/env python3
"""DDC comparison on the two book holdouts: gold Dewey numbers against gold
LCC, FDC and the shelfmark-placed general schemes, with bootstrap intervals.

Corpora: `holdout` (the 500 Open Library editions of Paper 1, scored on the
published 2026-10-10 graph; Dewey numbers from `holdout-ddc.jsonl`) and
`books` (the 3,000 editions of `holdout-books.jsonl`, scored on a graph
built here with the same model and k). Work goes to
`bench/corpus/cache/ddc-bench/`; every step is skipped when its output
exists. Runs per corpus:

- gold: `ddc` (the numbers tree of `ddc_numbers.py scheme`), `ddc-section`
  (the same numbers cut to the three-digit section), `lcc` and
  `lcc-full` (deepest covering class, as in Paper 1), and `fdc-ddc`
  (each Dewey number mapped to FDC by `ddc_numbers.py fdc`);
- shelfmark `profile:lexical@2` placements into `lcc`, `lcc-full`, `fdc`,
  `bk`, `ilc2`, `unesco-thesaurus` and `learned` (isolated store, empty
  rules, low-confidence placements kept, fallbacks unplaced);
- `learned-own`: the books inserted into the learned scheme as new works
  (nearest leaf centroid), as in 2026-10-10-learned.

A third corpus, `holdout-ddc`, is the holdout restricted to the editions
with a Dewey number, scored on the induced subgraph (`learned-scheme
subset`) with the holdout's placements, so that every scheme is compared on
books that both Dewey and LCC librarians classified.

Every run is scored with `shelfmark-bench run` against the graph's
average-linkage baseline, and all runs of a corpus together with
`shelfmark-bench bootstrap` (paired draws).
"""
import argparse
import gzip
import json
import shutil
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from run_benchmark import PREP, PROFILE, last_json, log, run, sha256

ROOT = HERE.parents[2]
CORPUS = ROOT / "bench/corpus"
LEARNED = ROOT / "schemes/learned"
PUBLISHED = ROOT / "bench/results/2026-10-10"
DDC = [sys.executable, str(HERE / "ddc_numbers.py")]
PLACED = ["lcc", "lcc-full", "fdc", "bk", "ilc2", "unesco-thesaurus", "learned"]
CORPORA = {
    "holdout": {"rows": CORPUS / "holdout.jsonl", "labels": CORPUS / "holdout-ddc.jsonl"},
    "books": {"rows": CORPUS / "holdout-books.jsonl", "labels": None},
    # The holdout editions that carry a Dewey number, on the induced subgraph;
    # placements are those made on the whole holdout.
    "holdout-ddc": {"subset_of": "holdout"},
}


class DdcBench:
    def __init__(self, a):
        self.a = a
        self.work = Path(a.cache) / "ddc-bench"
        self.schemes = self.work / "schemes"
        self.schemes.mkdir(parents=True, exist_ok=True)
        self.runs = self.work / "runs"
        self.runs.mkdir(parents=True, exist_ok=True)
        for name, c in CORPORA.items():
            if "subset_of" in c:
                c.setdefault("rows", self.work / "subsets" / f"{name}.jsonl")
                c.setdefault("labels", CORPORA[c["subset_of"]]["labels"])

    # ---- inputs -------------------------------------------------------
    def graph(self, corpus):
        graphs = self.work / "graphs"
        graphs.mkdir(parents=True, exist_ok=True)
        path, manifest = graphs / f"{corpus}-k20.json", graphs / f"{corpus}-k20.manifest.json"
        parent = CORPORA[corpus].get("subset_of")
        if parent and not manifest.exists():
            ids = self.work / "subsets" / f"{corpus}-ids.txt"
            ids.parent.mkdir(parents=True, exist_ok=True)
            gold, _ = self.gold_ddc(parent)
            ids.write_text("".join(r["id"] + "\n" for r in map(json.loads, gold.open()) if r["classes"]))
            run([self.a.learned, "subset", "--corpus", CORPORA[parent]["rows"], "--graph", self.graph(parent),
                 "--ids", ids, "--out-corpus", CORPORA[corpus]["rows"], "--out-graph", path])
            manifest.write_text(json.dumps({
                "subset_of": parent, "ids": len(ids.read_text().split()), "ids_sha256": sha256(ids),
                "rule": "graph induced on the ids (edges among them kept, same k and tau)",
                "corpus_sha256": sha256(CORPORA[corpus]["rows"]), "graph_sha256": sha256(path)}, indent=2) + "\n")
        if corpus == "holdout" and not path.exists():
            with gzip.open(PUBLISHED / "graphs/holdout-k20.json.gz") as src, path.open("wb") as dst:
                shutil.copyfileobj(src, dst)
            shutil.copy(PUBLISHED / "graphs/holdout-k20.manifest.json", manifest)
        if corpus == "books" and not path.exists():
            started = time.monotonic()
            run([self.a.python, HERE / "build_graph_fastembed.py", CORPORA[corpus]["rows"], path, manifest,
                 "--k", "20", "--vectors", graphs / f"{corpus}.f32"])
            m = json.loads(manifest.read_text())
            m["seconds"] = round(time.monotonic() - started, 1)
            manifest.write_text(json.dumps(m, indent=2) + "\n")
        m = json.loads(manifest.read_text())
        if sha256(path) != m["graph_sha256"] or sha256(CORPORA[corpus]["rows"]) != m["corpus_sha256"]:
            raise SystemExit(f"{path}: graph or corpus does not match its manifest")
        return path

    def scheme(self, name):
        path = self.schemes / f"{name}.json"
        if path.exists():
            return path
        if name == "ddc":
            cmd = DDC + ["scheme", "--out", path]
            full = [c for c in CORPORA.values() if "subset_of" not in c]
            for c in full:
                cmd += ["--corpus", c["rows"]]
            for c in full:
                cmd += ["--labels", c["labels"] or c["rows"]]
            log(f"ddc scheme: {run(cmd).strip()}")
        elif name == "learned":
            shutil.copy(LEARNED / "learned.json", path)
        else:
            shutil.copy(Path(self.a.cache) / "schemes" / f"{name}.json", path)
        return path

    def scheme_id(self, name):
        doc = json.loads(self.scheme(name).read_text(encoding="utf-8"))
        return f"scheme:{doc['name']}@{doc['version']}"

    def store(self, name):
        """An isolated store holding only this scheme and an empty rules file."""
        store = self.work / "stores" / name
        marker = store / "compiled.json"
        if marker.exists():
            return store
        store.mkdir(parents=True, exist_ok=True)
        (store / "rules.json").write_text('{"rules":[]}\n', encoding="utf-8")
        base = [self.a.shelfmark, "--json", "--store", store]
        preview = json.loads(run(base + ["scheme", "compile", self.scheme(name)]))
        out = json.loads(run(base + ["--commit", "--expected-revision", preview["data"]["revision"],
                                     "scheme", "compile", self.scheme(name)]))
        marker.write_text(json.dumps(out["data"], indent=2) + "\n", encoding="utf-8")
        return store

    # ---- placements ---------------------------------------------------
    def placements_dir(self, corpus):
        d = self.work / "placements" / corpus
        d.mkdir(parents=True, exist_ok=True)
        return d

    def gold_ddc(self, corpus, level=None):
        out = self.placements_dir(corpus) / ("ddc-section.jsonl" if level else "ddc-gold.jsonl")
        stats = out.with_suffix(".stats.json")
        if not stats.exists():
            c = CORPORA[corpus]
            cmd = DDC + ["gold", "--corpus", c["rows"], "--output", out] + (["--level", str(level)] if level else [])
            stats.write_text(run(cmd + (["--labels", c["labels"]] if c["labels"] else [])))
        return out, json.loads(stats.read_text())

    def gold_fdc(self, corpus):
        out = self.placements_dir(corpus) / "fdc-ddc.jsonl"
        stats = out.with_suffix(".stats.json")
        if not stats.exists():
            c = CORPORA[corpus]
            cmd = DDC + ["fdc", "--fdc", self.scheme("fdc"), "--corpus", c["rows"], "--output", out]
            stats.write_text(run(cmd + (["--labels", c["labels"]] if c["labels"] else [])))
        return out, json.loads(stats.read_text())

    def gold_lcc(self, corpus, name):
        """Deepest covering class of each LC call number, as in Paper 1, as placements."""
        out = self.placements_dir(corpus) / f"{name}-gold.jsonl"
        stats = out.with_suffix(".stats.json")
        if not stats.exists():
            normalized = self.work / "gold" / f"{corpus}-{name}.jsonl"
            cmd = PREP + ["normalize-lcc-deep", "--shelfmark", self.a.shelfmark, "--store", self.store(name),
                          "--scheme-id", self.scheme_id(name), "--corpus", CORPORA[corpus]["rows"],
                          "--output", normalized]
            if name != "lcc":
                cmd += ["--gold-key", name]
            result = last_json(run(cmd))
            rows = [json.loads(line) for line in normalized.open(encoding="utf-8")]
            with out.open("w", encoding="utf-8") as f:
                for row in rows:
                    codes = row["lc_classifications"] if name == "lcc" else row["gold"][name]
                    # The harness scores the first label, as in Paper 1.
                    f.write(json.dumps({"id": row["id"], "classes": codes[:1]}) + "\n")
            stats.write_text(json.dumps(result) + "\n")
        return out, json.loads(stats.read_text())

    def placed(self, corpus, name):
        work = self.work / "placed" / f"{corpus}-{name}"
        out = self.placements_dir(corpus) / f"{name}-placed.jsonl"
        stats_path = out.with_suffix(".stats.json")
        if stats_path.exists():
            return out, json.loads(stats_path.read_text())
        work.mkdir(parents=True, exist_ok=True)
        store, scheme_id = self.store(name), self.scheme_id(name)
        requests, batch = work / "requests.jsonl", work / "batch.jsonl"
        run(PREP + ["requests", "--corpus", CORPORA[corpus]["rows"], "--scheme", scheme_id,
                    "--profile", PROFILE, "--output", requests])
        started = time.monotonic()
        resume = ["--resume"] if batch.exists() else []
        run([self.a.shelfmark, "--json", "--store", store, "batch", "--requests", requests,
             "--out", batch] + resume)
        seconds = round(time.monotonic() - started, 1)
        stats = last_json(run(PREP + ["extract", "--corpus", CORPORA[corpus]["rows"], "--batch", batch,
                                      "--scheme", scheme_id, "--profile", PROFILE, "--output", out]))
        stats.update(classify_seconds=seconds, profile=PROFILE, store_rules="empty")
        stats_path.write_text(json.dumps(stats, indent=2) + "\n", encoding="utf-8")
        return out, stats

    def learned_own(self, corpus):
        """Insert the corpus into the learned scheme as new works."""
        out = self.placements_dir(corpus) / "learned-own.jsonl"
        report = out.with_suffix(".report.json")
        inserted = self.schemes / f"learned-{corpus}.json"
        if not report.exists():
            vectors = LEARNED / "data/holdout-vectors.json.gz"
            if corpus != "holdout":
                vectors = self.work / "graphs" / f"{corpus}-vectors.json.gz"
                self.graph(corpus)
                run([self.a.learned, "quantize", "--corpus", CORPORA[corpus]["rows"], "--vectors",
                     self.work / "graphs" / f"{corpus}.f32", "--out", vectors])
            run([self.a.learned, "insert", "--scheme", LEARNED / "learned.json", "--centroids",
                 LEARNED / "data/centroids.json.gz", "--corpus", CORPORA[corpus]["rows"], "--vectors", vectors,
                 "--out", inserted, "--placements", out, "--report", report])
        return out, json.loads(report.read_text()), inserted

    def run_specs(self, corpus):
        """(key, scheme file, placements, source, stats) for every run of a corpus."""
        if "subset_of" in CORPORA[corpus]:
            return self.run_specs(CORPORA[corpus]["subset_of"])
        specs = []
        path, stats = self.gold_ddc(corpus)
        specs.append(("ddc-gold", self.scheme("ddc"), path, "gold Dewey number (notation tree)", stats))
        path, stats = self.gold_ddc(corpus, level=3)
        specs.append(("ddc-section", self.scheme("ddc"), path, "gold Dewey number cut to its section", stats))
        for name in ("lcc", "lcc-full"):
            path, stats = self.gold_lcc(corpus, name)
            specs.append((f"{name}-gold", self.scheme(name), path, "gold LC call number", stats))
        path, stats = self.gold_fdc(corpus)
        specs.append(("fdc-ddc", self.scheme("fdc"), path, "gold Dewey number mapped to FDC", stats))
        for name in PLACED:
            path, stats = self.placed(corpus, name)
            specs.append((f"{name}-placed", self.scheme(name), path, f"shelfmark {PROFILE}", stats))
        path, stats, inserted = self.learned_own(corpus)
        specs.append(("learned-own", inserted, path, "own insertion (nearest leaf centroid, leaf splits)", stats))
        return specs

    # ---- scoring ------------------------------------------------------
    def reference(self, corpus):
        path = self.runs / f"{corpus}-reference.json"
        if not path.exists():
            graph = self.graph(corpus)
            started = time.monotonic()
            result = json.loads(run([self.a.harness, "run", "--corpus", CORPORA[corpus]["rows"], "--scheme",
                                     self.scheme("fdc"), "--graph", graph, "--k", "20"]))
            ref = {"corpus": corpus, "ref_cost_norm": result["ref_cost_norm"],
                   "graph_sha256": result["hashes"]["graph"], "seconds": round(time.monotonic() - started, 1)}
            path.write_text(json.dumps(ref, indent=2) + "\n")
        return json.loads(path.read_text())["ref_cost_norm"]

    def score(self, corpus):
        graph, ref = self.graph(corpus), self.reference(corpus)
        specs = self.run_specs(corpus)
        for key, scheme, placements, source, stats in specs:
            path = self.runs / f"{corpus}-{key}.json"
            if path.exists():
                continue
            started = time.monotonic()
            result = json.loads(run([self.a.harness, "run", "--corpus", CORPORA[corpus]["rows"], "--scheme",
                                     scheme, "--graph", graph, "--k", "20", "--reference-cost", repr(ref),
                                     "--placements", placements]))
            result["run"] = {"scheme_id": self.scheme_id_of(scheme), "corpus": corpus, "placement_source": source,
                             "placement_stats": stats, "score_seconds": round(time.monotonic() - started, 1)}
            path.write_text(json.dumps(result, indent=2) + "\n")
            log(f"{corpus} {key}: cost_norm={result['cost_norm']:.4f} skill={result['skill']:.4f} "
                f"coverage={result['coverage']:.3f}")
        boot = self.runs / f"{corpus}-bootstrap.json"
        if not boot.exists():
            spec_file = self.runs / f"{corpus}-bootstrap-runs.json"
            spec_file.write_text(json.dumps([{"key": k, "scheme": str(s), "placements": str(p)}
                                             for k, s, p, _, _ in specs], indent=2) + "\n")
            started = time.monotonic()
            run([self.a.harness, "bootstrap", "--corpus", CORPORA[corpus]["rows"], "--graph", graph,
                 "--runs", spec_file, "--resamples", str(self.a.resamples), "--seed", str(self.a.seed),
                 "--threads", str(self.a.threads), "--output", boot])
            data = json.loads(boot.read_text())
            data["seconds"] = round(time.monotonic() - started, 1)
            boot.write_text(json.dumps(data, indent=2) + "\n")
            log(f"{corpus} bootstrap: {self.a.resamples} draws in {data['seconds']}s")

    @staticmethod
    def scheme_id_of(path):
        doc = json.loads(Path(path).read_text(encoding="utf-8"))
        return f"scheme:{doc['name']}@{doc['version']}"


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--shelfmark", required=True)
    p.add_argument("--harness", required=True)
    p.add_argument("--learned", required=True, help="learned-scheme binary")
    p.add_argument("--python", default=str(ROOT / "bench/corpus/cache/venv/bin/python"),
                   help="interpreter with fastembed 0.9.0, for the books graph")
    p.add_argument("--resamples", type=int, default=1000)
    p.add_argument("--seed", type=int, default=20261010)
    p.add_argument("--threads", type=int, default=8)
    p.add_argument("--corpus", action="append", choices=list(CORPORA), help="default: both")
    a = p.parse_args()
    bench = DdcBench(a)
    started = time.monotonic()
    for corpus in a.corpus or list(CORPORA):
        bench.score(corpus)
    log(f"done in {time.monotonic() - started:.0f}s")


if __name__ == "__main__":
    main()
