# Reproducing the 2026-10-10 results

All commands run from the repository root on Linux with network access.
Intermediate files go under `bench/corpus/cache/`, which git ignores. The
run took about 45 minutes on 8 CPUs with 15 GB RAM; `results.json` records
the per-stage timings.

## 1. Tools

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-cli -p shelfmark-bench
BIN=${CARGO_TARGET_DIR:-target}/release
uv venv bench/corpus/cache/venv
VIRTUAL_ENV=bench/corpus/cache/venv uv pip install 'fastembed==0.9.0'
```

The published run used fastembed 0.9.0, numpy 2.5.3 and onnxruntime 1.31.0.

## 2. Similarity graphs

```sh
PY=bench/corpus/cache/venv/bin/python
$PY crates/bench/scripts/build_graph_fastembed.py bench/corpus/corpus.jsonl \
  bench/corpus/cache/graphs/corpus-k20.json bench/corpus/cache/graphs/corpus-k20.manifest.json --k 20
$PY crates/bench/scripts/build_graph_fastembed.py bench/corpus/holdout.jsonl \
  bench/corpus/cache/graphs/holdout-k20.json bench/corpus/cache/graphs/holdout-k20.manifest.json --k 20
```

The script downloads `Qdrant/all-MiniLM-L6-v2-onnx` at the pinned revision
`d13954661f83248295ba75c1ed411eef3b7b936e`. Each manifest records the model
id, revision, `model.onnx` sha256, corpus sha256, tau and graph sha256. The
published graphs (`graphs/*.json.gz`) are the same files, gzipped; to skip
this step, decompress them into `bench/corpus/cache/graphs/` and check the
manifests' `graph_sha256`:

```sh
mkdir -p bench/corpus/cache/graphs
for g in corpus holdout; do
  gzip -dc bench/results/2026-10-10/graphs/$g-k20.json.gz > bench/corpus/cache/graphs/$g-k20.json
  cp bench/results/2026-10-10/graphs/$g-k20.manifest.json bench/corpus/cache/graphs/
done
sha256sum bench/corpus/cache/graphs/*-k20.json
```

## 3. Schemes

```sh
crates/bench/scripts/build_schemes.sh
```

This copies the bundled scheme files (`lcc`, `lcc-full` decompressed,
`openalex-topics`, `fdc`, `unesco-thesaurus`, `bk`, `arxiv`) and builds
the others from upstream with their own builders: `ipc` (fetched into a
scratch data directory so the committed lock is not rewritten, then built
from the committed lock), and the fetch-only `acm-ccs`, `eurovoc`, `ilc2`,
`msc` and `jel`. Every builder checks the download against its committed
`sources.lock.json`. Outputs go to `bench/corpus/cache/schemes/<name>.json`,
with one status line per scheme in `status.tsv`. Do not commit them: the
fetch-only schemes are not redistributable (see each scheme's README).

## 4. Placements and scoring

```sh
python3 crates/bench/scripts/run_benchmark.py --shelfmark $BIN/shelfmark \
  --harness $BIN/shelfmark-bench --jobs 4
```

Each step is skipped when its output exists, so an interrupted run resumes.
The script:

1. Computes the average-linkage baseline once per graph and passes its
   `cost_norm` to every later scoring run as `--reference-cost`.
2. Gold placements (track `A`), from normalized copies of the corpus
   (`prepare_placements.py`):
   - `arxiv`: `categories` kept when they are scheme codes; the harness
     uses the first, the primary category.
   - `msc`: MSC2020 codes extracted from the `msc` field, keeping valid codes.
   - `jel`: `jel` codes kept when they are scheme codes (2 papers carry any).
   - `acm-ccs`: `acm_ccs` codes kept when they are scheme codes. None is, so
     this gold run is skipped (reason in `results.json`).
   - `lcc` and `lcc-full` on the holdout: each LC call number (edition year
     dropped) is checked against the compiled scheme with `scheme-browse`
     requests through `shelfmark batch`, trying the full call number, then
     the class number without cutters, then the class letters. The first
     to resolve gives the deepest covering class. `MLCS` microform numbers
     and empty values are skipped.
3. shelfmark placements (track `P`) for every built scheme over the arXiv
   corpus, and for the general schemes (`lcc`, `lcc-full`, `fdc`, `bk`,
   `ilc2`, `unesco-thesaurus`) over the holdout as well. Each scheme gets
   its own store, `bench/corpus/cache/stores/<name>/`, holding only that
   compiled scheme and an empty `rules.json` (`{"rules":[]}`), which
   replaces the bundled starter rules (those name LCC classes such as
   `QC174.45` that other schemes lack). Requests use
   `profile:lexical@1`; `prepare_placements.py extract` keeps each
   placement's class, including flagged ones below the 0.8 target, and
   treats fallback placements (no candidate at all) as unplaced.
4. Scores each placement with `shelfmark-bench run --graph ... --k 20
   --reference-cost ...` and writes `bench/corpus/cache/runs/<key>.json`.

## 5. Publish

```sh
python3 crates/bench/scripts/publish_results.py --out bench/results/2026-10-10 \
  --harness $BIN/shelfmark-bench --runtime bench/corpus/cache/runtime.json
```

`runtime.json` is a hand-written object of stage timings, taken from the
logs. This writes one JSON per (scheme, placement source):
`<scheme>-gold.json`, `<scheme>-placed.json` and
`<scheme>-placed-holdout.json`. It also writes the gzipped graphs with their
manifests, `results.json` (corpus hashes, model, graphs and baselines,
harness commit and binary hash, scheme versions and file hashes, skipped
runs with reasons, runtime) and `results.md`.

The scores depend only on the graph, the scheme file and the placements.
With the same graph and scheme files, the gold results reproduce exactly.
The placed results also depend on the shelfmark build used for
classification.
