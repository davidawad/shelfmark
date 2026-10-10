#!/usr/bin/env python3
"""Rescore a sample of published runs with the current shelfmark-bench and
check that every number matches.

Covers the three bootstrap paths of the harness:

* `run --bootstrap` (2026-10-10-embed): point scores, hashes and every
  95% interval, including skill (the reference replicates are recomputed
  in-run from the average-linkage tree on the same resamples);
* `bootstrap` (2026-10-10-ddc): points, intervals and paired differences of
  the runs whose scheme files are committed;
* `scores --boot` (2026-10-10-scores with `--boot-streams single`,
  2026-10-10-covered with the default split streams): the catalogue scores
  and intervals that do not read the embedding vectors (not committed).

Only committed inputs are read; fetch-built schemes (lcc-full, ilc2, ...)
are skipped. Exit status 1 on any difference.

    python3 crates/bench/scripts/check_published.py --harness target/release/shelfmark-bench
"""
import argparse
import gzip
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
R = ROOT / "bench/results"
SCHEMES = {
    "lcc": "data/schemes/lcc/lcc.json",
    "fdc": "schemes/fdc/fdc.json",
    "bk": "schemes/bk/bk.json",
    "unesco-thesaurus": "schemes/unesco-thesaurus/unesco-thesaurus.json",
    "openalex-topics": "schemes/openalex-topics/openalex-topics.json",
    "arxiv": "schemes/arxiv/arxiv.json",
    "learned": "schemes/learned/learned.json",
    "ddc": "bench/results/2026-10-10-ddc/schemes/ddc.json",
}
CORPORA = {"corpus": "bench/corpus/corpus.jsonl", "holdout": "bench/corpus/holdout.jsonl",
           "books": "bench/corpus/holdout-books.jsonl"}
# (condition, universe, row, scheme file, placements file, track)
EMBED = [
    ("bge-small-k20", "holdout", "lcc-placed", SCHEMES["lcc"], "holdout-lcc-placed.jsonl", None),
    ("bge-small-k20", "holdout", "fdc-placed", SCHEMES["fdc"], "holdout-fdc-placed.jsonl", None),
    ("nomic-k50", "holdout", "bk-placed", SCHEMES["bk"], "holdout-bk-placed.jsonl", None),
    ("nomic-k10", "holdout", "lcc-gold", SCHEMES["lcc"], "holdout-lcc-gold.jsonl", "A"),
    ("minilm-k50", "holdout", "learned-own", "bench/results/2026-10-10-embed/placements/learned-holdout.json",
     "holdout-learned-own.jsonl", None),
    ("minilm-k10", "holdout", "unesco-thesaurus-placed", SCHEMES["unesco-thesaurus"],
     "holdout-unesco-thesaurus-placed.jsonl", None),
    ("bge-small-k50", "corpus", "fdc-placed", SCHEMES["fdc"], "corpus-fdc-placed.jsonl", None),
]
# In the published run order, so paired differences keep their sign.
DDC_RUNS = {
    "ddc-gold": "ddc", "ddc-section": "ddc", "lcc-gold": "lcc", "fdc-ddc": "fdc",
    "lcc-placed": "lcc", "fdc-placed": "fdc", "bk-placed": "bk",
    "unesco-thesaurus-placed": "unesco-thesaurus", "learned-placed": "learned",
}
DDC_GRAPHS = {"holdout": "bench/results/2026-10-10/graphs/holdout-k20.json.gz",
              "books": "bench/results/2026-10-10-ddc/graphs/books-k20.json.gz"}
LCC_LETTERS = re.compile(r"^([A-Z]{1,3})(?=[0-9]|$| )")  # as score_catalogue.py

failures = []


def same(label, want, got):
    if want != got:
        failures.append(f"{label}: published {want!r}, rescored {got!r}")
        return False
    return True


def gunzip(src, tmp):
    out = Path(tmp) / Path(src).name.removesuffix(".gz")
    if not out.exists():
        out.write_bytes(gzip.decompress((ROOT / src).read_bytes()))
    return out


def harness(a, *args):
    subprocess.run([a.harness, *map(str, args)], check=True, cwd=ROOT,
                   stdout=subprocess.DEVNULL)


def check_embed(a, tmp):
    for cond, universe, row, scheme, placements, track in EMBED:
        published = json.loads((R / f"2026-10-10-embed/runs/{cond}/{universe}/{row}.json").read_text())
        model, k = cond.rsplit("-k", 1)
        graph = (gunzip(f"bench/results/2026-10-10/graphs/{universe}-k20.json.gz", tmp)
                 if (model, k) == ("minilm", "20")
                 else gunzip(f"bench/results/2026-10-10-embed/graphs/{model}-{universe}-k{k}.json.gz", tmp))
        out = Path(tmp) / f"embed-{cond}-{universe}-{row}.json"
        args = ["run", "--corpus", CORPORA[universe], "--scheme", scheme, "--graph", graph, "--k", k,
                "--placements", R / "2026-10-10-embed/placements" / placements,
                "--bootstrap", 1000, "--output", out]
        if track:
            args += ["--track", track]
        harness(a, *args)
        got = json.loads(out.read_text())
        label = f"embed {cond}/{universe}/{row}"
        ok = all(same(f"{label} {key}", published[key], got[key])
                 for key in ["cost", "cost_norm", "revenue_norm", "ref_cost_norm", "skill", "coverage",
                             "structure", "hashes", "track"])
        ok &= same(f"{label} bootstrap", published["bootstrap"], got["bootstrap"])
        print(f"{'ok  ' if ok else 'FAIL'} {label}: cost_norm {got['cost_norm']:.6f}, "
              f"skill {got['skill']:.6f}, {len(got['bootstrap']['ci'])} intervals")


def check_ddc(a, tmp):
    for corpus, graph in DDC_GRAPHS.items():
        published = json.loads((R / f"2026-10-10-ddc/bootstrap/{corpus}.json").read_text())
        runs = [{"key": key, "scheme": str(ROOT / SCHEMES[scheme]),
                 "placements": str(R / f"2026-10-10-ddc/placements/{corpus}/{key}.jsonl")}
                for key, scheme in DDC_RUNS.items()]
        spec = Path(tmp) / f"ddc-{corpus}-runs.json"
        spec.write_text(json.dumps(runs))
        out = Path(tmp) / f"ddc-{corpus}.json"
        harness(a, "bootstrap", "--corpus", CORPORA[corpus], "--graph", gunzip(graph, tmp),
                "--runs", spec, "--resamples", 1000, "--output", out)
        got = json.loads(out.read_text())
        label = f"ddc {corpus}"
        ok = same(f"{label} reference", published["reference"], got["reference"])
        for key in DDC_RUNS:
            for part in ["point", "ci", "scheme_sha256", "placements_sha256"]:
                ok &= same(f"{label} {key} {part}", published["runs"][key][part], got["runs"][key][part])
        pairs = 0
        for metric, rows in got["paired"].items():
            want = {(r["a"], r["b"]): r for r in published["paired"][metric]}
            for r in rows:
                ok &= same(f"{label} paired {metric} {r['a']}-{r['b']}", want[(r["a"], r["b"])], r)
                pairs += 1
        print(f"{'ok  ' if ok else 'FAIL'} {label}: {len(DDC_RUNS)} runs, {pairs} paired differences")


def check_scores(a, tmp):
    published = json.loads((R / "2026-10-10-scores/results.json").read_text())
    external = Path(tmp) / "external-holdout.jsonl"
    with open(ROOT / CORPORA["holdout"], encoding="utf-8") as f, open(external, "w") as out:
        for line in f:
            r = json.loads(line)
            calls = [c for c in r.get("lc_classifications", []) if c and not c.startswith("MLCS")]
            m = LCC_LETTERS.match(calls[0].strip()) if calls else None
            out.write(json.dumps({"id": r["id"], "fine": m.group(1) if m else None,
                                  "coarse": m.group(1)[0] if m else None}) + "\n")
    covered = json.loads((R / "2026-10-10-covered/results.json").read_text())["runs"]
    graph = gunzip("bench/results/2026-10-10/graphs/holdout-k20.json.gz", tmp)
    for key, scheme, placements in [
        ("fdc-placed-holdout", "fdc", "holdout-fdc-placed.jsonl"),
        ("bk-placed-holdout", "bk", "holdout-bk-placed.jsonl"),
    ]:
        # 2026-10-10-scores drew every resample from one stream; the
        # covered run (and the default) draws each kind from its own.
        for streams, want, source in [("single", published["runs"][key], "scores"),
                                      ("split", covered[key]["all"], "covered")]:
            out = Path(tmp) / f"scores-{key}-{streams}.json"
            harness(a, "scores", "--corpus", CORPORA["holdout"], "--scheme", SCHEMES[scheme],
                    "--graph", graph, "--external", external,
                    "--placements", R / "2026-10-10-embed/placements" / placements,
                    "--seed", 20261010, "--reps", 10, "--boot", 200, "--boot-streams", streams,
                    "--output", out)
            got = json.loads(out.read_text())
            label = f"{source} {key}"
            ok = True
            if "hashes" in want:
                ok = all(same(f"{label} {h} sha256", want["hashes"][h], got["hashes"][h])
                         for h in ["corpus", "external", "graph", "placements", "scheme"])
            compared = 0
            for part in ["scores", "ci"]:
                for name, value in got["catalogue"][part].items():
                    if name.startswith(("embed.", "probe.")) or name not in want["catalogue"][part]:
                        continue
                    ok &= same(f"{label} {part} {name}", want["catalogue"][part][name], value)
                    compared += 1
            print(f"{'ok  ' if ok else 'FAIL'} {label} (--boot-streams {streams}): "
                  f"{compared} scores and intervals")


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--harness", required=True)
    a = p.parse_args()
    with tempfile.TemporaryDirectory() as tmp:
        check_embed(a, tmp)
        check_ddc(a, tmp)
        check_scores(a, tmp)
    for f in failures:
        print("DIFF", f, file=sys.stderr)
    print(f"{len(failures)} differences")
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
