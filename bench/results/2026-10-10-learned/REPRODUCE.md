# Reproducing the 2026-10-10-learned results

This run scores the learned scheme (`schemes/learned`) on the Paper 1
benchmark: the same corpus, holdout, graphs, average-linkage baselines and
harness as `../2026-10-10/` and `../2026-10-10-r2/`. Paper 1 placements are
regenerated (they are not committed) and must score exactly as published;
`results.json` records the check (`baselines.paper1_rerun_matches_published`).

All commands run from the repository root on Linux. Intermediate files go
under `bench/corpus/cache/`, which git ignores. Network access is needed for
the Paper 1 scheme builds and the embedding model download only.

## 1. Tools

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-cli -p shelfmark-bench
cargo build --release --manifest-path schemes/learned/Cargo.toml
BIN=${CARGO_TARGET_DIR:-target}/release
```

`schemes/learned` is a standalone crate like the other scheme builders. With
a shared `CARGO_TARGET_DIR` all three binaries land in `$BIN`; otherwise
`learned-scheme` is in `schemes/learned/target/release`.

## 2. Graphs and Paper 1 placements

As in `../2026-10-10-r2/REPRODUCE.md` steps 2-4: decompress the published
graphs into `bench/corpus/cache/graphs/` (sha256 checked against the
manifests), build the schemes, then place and score them:

```sh
crates/bench/scripts/build_schemes.sh
python3 crates/bench/scripts/run_benchmark.py --shelfmark $BIN/shelfmark \
  --harness $BIN/shelfmark-bench --jobs 3 \
  --reference-from bench/results/2026-10-10-r2/results.json
```

This run did not pass `--placed-only`, so the gold runs are rescored too. All
13 scheme files hashed as in 2026-10-10, and all 19 placed and 5 gold scores
equal the published ones.

## 3. Embedding vectors

The learned scheme's insertion step and leaf centroids use the vectors behind
the graphs. `build_graph_fastembed.py` gained a `--vectors` option that writes
them (little-endian f32, n x 384). In a scratch virtual environment:

```sh
uv venv /tmp/embvenv && VIRTUAL_ENV=/tmp/embvenv uv pip install fastembed==0.9.0
for g in holdout corpus; do
  /tmp/embvenv/bin/python crates/bench/scripts/build_graph_fastembed.py \
    bench/corpus/$g.jsonl bench/corpus/cache/embed/$g-k20.json \
    bench/corpus/cache/embed/$g-k20.manifest.json --vectors bench/corpus/cache/embed/$g.f32
done
```

Same pinned weights (`model.onnx` sha256 `bbd7b466...`), here with
onnxruntime 1.31.0. The rebuilt graphs differ from the published ones by
at most 1e-6 in edge weight and in one or two near-tied neighbour choices
(the graph hashes therefore differ). The fit uses only the published graph.
The vectors enter only through i8-quantized centroids and holdout vectors,
whose quantization step (about 1e-3) is much larger than that noise. The
committed `schemes/learned/data/*.json.gz` were made from these vectors:

```sh
$BIN/learned-scheme build --corpus bench/corpus/corpus.jsonl \
  --graph bench/results/2026-10-10/graphs/corpus-k20.json.gz \
  --vectors bench/corpus/cache/embed/corpus.f32 --labels schemes/learned/labels.tsv \
  --out schemes/learned/learned.json --centroids schemes/learned/data/centroids.json.gz
$BIN/learned-scheme quantize --corpus bench/corpus/holdout.jsonl \
  --vectors bench/corpus/cache/embed/holdout.f32 --out schemes/learned/data/holdout-vectors.json.gz
```

## 4. Learned-scheme runs

```sh
python3 crates/bench/scripts/learned_benchmark.py --shelfmark $BIN/shelfmark \
  --harness $BIN/shelfmark-bench --learned $BIN/learned-scheme
```

It works in `bench/corpus/cache/learned-bench/` and, in order:

1. rebuilds `learned.json` from the committed corpus, graph and labels and
   stops unless it equals `schemes/learned/learned.json` byte for byte;
   scores its own placements of the corpus (in-sample);
2. inserts the 500 holdout records with the committed centroids and holdout
   vectors and scores the result on the holdout graph;
3. places the corpus and the holdout into the learned scheme with shelfmark
   (`profile:lexical@2`, an isolated store with empty rules, through
   `run_benchmark.py`'s code) and scores them;
4. splits the corpus 80/20 (`learned-scheme split-ids`, SHA-256 ranking with
   seed `learned-split-2026-10-10`; the 2,000 test ids are in
   `split/test-ids.txt`), fits on the 80% with the same parameters and
   automatic labels, inserts the 20% as new works, and scores the 20% on its
   induced graph (its own average-linkage reference, cost_norm 0.1024) next
   to every Paper 1 placed scheme and gold label set restricted to the same
   2,000 works, and next to the full fit (in-sample). It also scores the 80%
   fit on the 80% graph, and the 80% fit plus inserted 20% on the whole
   corpus graph.

Every harness call is `shelfmark-bench run --graph ... --placements ...
--reference-cost <baseline>`, as in Paper 1; for the 20% graph the first
call computes the baseline and the rest reuse it.

The sensitivity sweep (36 fits, unlabelled):

```sh
mkdir -p bench/corpus/cache/learned-bench/sweep
for a in 1 2; do for d in 3 4 6; do for l in 0.001 0.002 0.005 0.01 0.02 0.05; do
  $BIN/learned-scheme build --corpus bench/corpus/corpus.jsonl \
    --graph bench/results/2026-10-10/graphs/corpus-k20.json.gz --allow-auto-labels \
    --out /tmp/sweep.json --lambda $l --max-depth $d --branch-cost $a \
    --report bench/corpus/cache/learned-bench/sweep/a$a-d$d-l$l.json
done; done; done
```

## 5. Publish

```sh
python3 crates/bench/scripts/publish_learned.py --harness $BIN/shelfmark-bench \
  --shelfmark $BIN/shelfmark --learned $BIN/learned-scheme
```

It writes `results.json` (objective, inputs, build stages, insertion
summaries, every table row, the sweep, tool hashes, and every harness result
under `runs`), `runs/*.json`, the holdout-inserted scheme
`learned-holdout.json`, the reports, and the 80% scheme
`split/learned-split80.json`. It also writes the tables of `results.md` to
`bench/corpus/cache/learned-bench/tables.md`; the prose of `results.md` is
written by hand.

## Determinism

The fit is deterministic for a fixed seed and needs only committed files.
`cargo test --release --manifest-path schemes/learned/Cargo.toml` rebuilds
the scheme byte for byte and checks the objective against the harness.
Insertion is deterministic given the committed vectors. The 80/20 runs
depend on the locally regenerated vectors (step 3) through the 80% fit's
centroids and the test vectors.
