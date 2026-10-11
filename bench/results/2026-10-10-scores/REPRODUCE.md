# Reproducing the 2026-10-10-scores results

This run computes the scoring catalogue
(`research/reports/6-scoring-catalogue.md`) for every Paper 1 run and the
learned scheme. It uses the published graphs, the schemes and baselines of
`../2026-10-10/`, the placements of `../2026-10-10-r2/` and
`../2026-10-10-r3/`, and the learned-scheme runs of
`../2026-10-10-learned/`.

All commands run from the repository root on Linux. Intermediate files go
under `bench/corpus/cache/`, which git ignores. Network access is needed only
for the fetch-only scheme builds and the embedding model download. The whole
run took about 25 minutes on 8 CPUs; placement is most of it, and the
catalogue scoring itself takes about 70 seconds.

## 1. Tools

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-cli -p shelfmark-bench
cargo build --release --manifest-path schemes/learned/Cargo.toml
BIN=${CARGO_TARGET_DIR:-target}/release
```

## 2. Graphs, schemes, placements

The placements are not committed, so regenerate them with the r2 commands.
First decompress the published graphs (sha256 checked against the manifests,
as in `../2026-10-10-r2/REPRODUCE.md` step 2):

```sh
mkdir -p bench/corpus/cache/graphs
for g in corpus holdout; do
  gzip -dc bench/results/2026-10-10/graphs/$g-k20.json.gz > bench/corpus/cache/graphs/$g-k20.json
  cp bench/results/2026-10-10/graphs/$g-k20.manifest.json bench/corpus/cache/graphs/
done
crates/bench/scripts/build_schemes.sh
python3 crates/bench/scripts/run_benchmark.py --shelfmark $BIN/shelfmark \
  --harness $BIN/shelfmark-bench --jobs 4 \
  --reference-from bench/results/2026-10-10-r2/results.json
```

All 13 scheme files hashed as in `../2026-10-10/results.json`. All 19 placed
and 5 gold harness results reproduced the published files exactly (tree
SHA-256 and cost_norm; ACM CCS gold is skipped, as before).

The learned scheme's own placements and its shelfmark placements come from
`learned_benchmark.py`'s `own` and `placed` steps. The 80/20 split step is
not needed. Run them without the split:

```sh
cd crates/bench/scripts && python3 -c "
import argparse, learned_benchmark as lb
a = argparse.Namespace(cache='../../../bench/corpus/cache', shelfmark='$BIN/shelfmark',
    harness='$BIN/shelfmark-bench', learned='$BIN/learned-scheme', vectors=None,
    graph_gz='../../../bench/results/2026-10-10/graphs/corpus-k20.json.gz')
x = lb.Learned(a); x.own(); x.placed()" && cd -
```

Use absolute paths for `$BIN` if it is relative. `learned-own` and
`learned-own-holdout` reproduced `../2026-10-10-learned/runs/` exactly.

## 3. Embedding vectors

```sh
uv venv /tmp/embvenv && VIRTUAL_ENV=/tmp/embvenv uv pip install fastembed==0.9.0
mkdir -p bench/corpus/cache/embed
for g in holdout corpus; do
  /tmp/embvenv/bin/python crates/bench/scripts/build_graph_fastembed.py bench/corpus/$g.jsonl \
    bench/corpus/cache/embed/$g-k20.json bench/corpus/cache/embed/$g-k20.manifest.json \
    --vectors bench/corpus/cache/embed/$g.f32
done
sha256sum bench/corpus/cache/embed/*.f32
```

This run used fastembed 0.9.0, numpy 2.5.3 and onnxruntime 1.31.0. The
rebuilt corpus graph had the published SHA-256 (`759d3b84…`), so the vectors
are those behind the published graph. The vector hashes were:

- corpus: `5659678124f989f0ab1a6771777d81527970b8f455618c91fb9c98ef22f9c2e5`
- holdout: `143b8f076319c48ffa84d95ce90299e5faf8d000f1c7dac364dffba35ebeccb1`

A different onnxruntime build can change them in the last float bits. Only
the embedding scores (cophenetic, silhouette, Davies–Bouldin, centroid
probe) read the vectors; every other score reads only the graph.

## 4. Scores

```sh
python3 crates/bench/scripts/score_catalogue.py run --harness $BIN/shelfmark-bench --jobs 4
```

For each of the 28 runs this calls

```sh
shelfmark-bench scores --corpus <corpus or gold corpus> --scheme <scheme.json> \
  --graph <graph> --vectors <vectors> --external <external labels> \
  --placements <placements.jsonl> [--compare-gold <gold> | --compare-placements <own>] \
  --seed 20261010 --reps 10 --boot 200 --output <cache>/scores/<run>.json
```

It writes the external gold labels (`scores/external-*.jsonl`): for papers,
the arXiv primary category and its group; for books, the LCC class letters
of the first call number. It checks that the catalogue's Dasgupta cost
equals the harness cost_norm. It then adds the harness structure statistics
(including the sorted-size Gini) and the per-item classification latency.
Every random draw comes from SplitMix64 with seed 20261010: the same inputs
give the same scores.

## 5. Publish

Write the notes that open `results.md` (provenance, caveats and conclusions)
to `bench/corpus/cache/scores-notes.md`, then run:

```sh
python3 crates/bench/scripts/score_catalogue.py publish --harness $BIN/shelfmark-bench \
  --notes bench/corpus/cache/scores-notes.md
```

This runs `shelfmark-bench agree` once per group: gold, placed-corpus and
placed-holdout. Each run computes Spearman and Kendall matrices between the
scores across runs, the average-linkage order on `1 − |ρ|`, and ranking
stability under bootstrap and noise. It then writes `results.json` (every
run without its bootstrap replicates, input hashes, harness hash and the
agreement) and `results.md`.
