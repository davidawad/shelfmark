# Reproducing the 2026-10-10-embed results

This run asks whether the benchmark's conclusions depend on the embedding
model and on k. Every published graph used all-MiniLM-L6-v2 at k = 20. Here
the corpus and the holdout are embedded with two independent open models of
other families and training data, BAAI/bge-small-en-v1.5 and
nomic-ai/nomic-embed-text-v1.5, and graphs are built at k = 10, 20 and 50
for all three models (nine conditions). The placements do not depend on the
embedding, so they are made once and every one is rescored on every graph.
Each score carries a 95% item-bootstrap interval.

All commands run from the repository root on Linux. Intermediate files go
under `bench/corpus/cache/`, which git ignores. Network access is needed for
the scheme builds and the model downloads only.

## 1. Tools

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-cli -p shelfmark-bench
cargo build --release --manifest-path schemes/learned/Cargo.toml
BIN=${CARGO_TARGET_DIR:-target}/release   # learned-scheme may be in schemes/learned/target/release
uv venv /tmp/embvenv && VIRTUAL_ENV=/tmp/embvenv uv pip install fastembed==0.9.0
```

The run used fastembed 0.9.0, onnxruntime 1.31.0 and numpy 2.5.3 on 16 CPUs (30 GiB RAM, ~12 GiB budget).

## 2. Embeddings and graphs

`build_graph_fastembed.py` now takes `--model minilm|bge-small|nomic` and
`--from-vectors` (build another k from vectors already written by
`--vectors`, without embedding again). The pins:

| key | model | ONNX weights | revision | weights file | text prefix |
| --- | --- | --- | --- | --- | --- |
| `minilm` | sentence-transformers/all-MiniLM-L6-v2 | Qdrant/all-MiniLM-L6-v2-onnx | `d13954661f83248295ba75c1ed411eef3b7b936e` | `model.onnx` | none |
| `bge-small` | BAAI/bge-small-en-v1.5 | Qdrant/bge-small-en-v1.5-onnx-Q (int8) | `aa8f8b060edb00e03bfdd08813a2949946c8ba55` | `model_optimized.onnx` | none |
| `nomic` | nomic-ai/nomic-embed-text-v1.5 | nomic-ai/nomic-embed-text-v1.5 | `e9b6763023c676ca8431644204f50c2b100d9aab` | `onnx/model.onnx` | `clustering: ` |

Each manifest in `graphs/` records the model, repository, revision, weights
sha256, corpus sha256, tau and graph sha256. The weights are never committed.

```sh
PY=/tmp/embvenv/bin/python; E=bench/corpus/cache/embed2
for m in minilm bge-small nomic; do
  # nomic-embed-text-v1.5: cap at 512 tokens, small batches (see below).
  [ $m = nomic ] && X="--max-length 512 --batch-size 16" || X=""
  for g in holdout corpus; do
  $PY -I crates/bench/scripts/build_graph_fastembed.py bench/corpus/$g.jsonl $E/$m-$g-k20.json \
    $E/$m-$g-k20.manifest.json --k 20 --model $m --threads 6 $X --vectors $E/$m-$g.f32
  for k in 10 50; do
    $PY -I crates/bench/scripts/build_graph_fastembed.py bench/corpus/$g.jsonl $E/$m-$g-k$k.json \
      $E/$m-$g-k$k.manifest.json --k $k --model $m $X --from-vectors $E/$m-$g.f32
  done
done; done
```

The script embeds batch by batch, normalizes each batch and streams it into
a memmap at `--vectors`, and prints the peak RSS. At nomic's own 8192-token
limit with batches of 64, an earlier attempt was killed at 16 GiB on the 10k
corpus. With `--max-length 512 --batch-size 16`, the corpus peaked at 4.23 GiB
in 2,687 s on 6 threads. 114 of the 10,000 corpus texts exceed 512 tokens; no
holdout text does. The nomic manifests record `max_length_tokens`.
`--vectors-only` stops after writing the vectors. Building a graph from
stored vectors stays under 0.4 GiB.

In this run, the re-embedded MiniLM and bge-small vectors rebuilt the
committed MiniLM k = 10 and bge-small k = 20 graphs byte for byte. Their
graphs were therefore taken from `graphs/` (decompressed) instead of being
rewritten, and the vectors were used only for the refits and the learned
split.

The all-MiniLM-L6-v2 k = 20 condition uses the published graphs
(`../2026-10-10/graphs/`), decompressed into `bench/corpus/cache/graphs/` as
in `../2026-10-10-r2/REPRODUCE.md` step 2; the rebuilt MiniLM k = 20 graph
is used only for its vectors (k = 10 and 50 come from them). The other
graphs are published gzipped in `graphs/`; decompress them into
`bench/corpus/cache/embed2/` to skip embedding, and check `graph_sha256`.
Embeddings from ONNX Runtime differ across builds by about 1e-6, so a
re-embedding may change near-tied neighbours and the graph hash; rescoring
the published graphs reproduces the scores exactly.

## 3. Placements

Build the schemes and regenerate the Paper 1 placements and gold labels as
in `../2026-10-10-learned/REPRODUCE.md` steps 2-4. In this run all 13 scheme
files hashed as in 2026-10-10.

```sh
crates/bench/scripts/build_schemes.sh
python3 crates/bench/scripts/run_benchmark.py --shelfmark $BIN/shelfmark \
  --harness $BIN/shelfmark-bench --jobs 2 \
  --reference-from bench/results/2026-10-10-r2/results.json
python3 crates/bench/scripts/learned_benchmark.py --shelfmark $BIN/shelfmark \
  --harness $BIN/shelfmark-bench --learned $BIN/learned-scheme \
  --vectors bench/corpus/cache/embed2/minilm-corpus.f32
```

All placements are committed in `placements/` (JSONL, one
`{"id", "classes"}` object per work): `corpus-<scheme>-placed.jsonl` and
`holdout-<scheme>-placed.jsonl` (shelfmark `profile:lexical@2`, flagged
placements kept, fallbacks unplaced), `<corpus>-<scheme>-gold.jsonl` (the
first normalized source label, which is what the harness scores for track
A), and the learned scheme's own placements with the schemes they refer to
(`learned-holdout.json`, `learned-split80-inserted.json`; the full fit is
`schemes/learned/learned.json`). The 20% split rows are these placements
restricted to `../2026-10-10-learned/split/test-ids.txt`.

## 4. Learned refits and scoring

```sh
python3 crates/bench/scripts/embed_benchmark.py --harness $BIN/shelfmark-bench \
  --learned $BIN/learned-scheme --jobs 2
```

It works in `bench/corpus/cache/embed-bench/` and, in order:

1. refits the learned scheme on the bge-small and the nomic k = 20 corpus
   graphs (same parameters, automatic labels), inserts the 500 holdout books
   with that model's vectors, and repeats the 80/20 split fit and insertion
   with that model's graph and vectors. The schemes and reports are in
   `refit/<model>/`;
2. for each of the nine conditions, computes the average-linkage reference
   of the corpus graph, the holdout graph and the graph induced on the 2,000
   test works (`learned-scheme subset`), with 1,000 bootstrap replicates;
3. scores every placement on every condition with `shelfmark-bench run
   --placements ... --reference-cost <ref> --reference-replicates <ref reps>
   --bootstrap 1000 --replicates-out ...`. Gold placements pass `--track A`.

`--bootstrap N` (new) resamples the works with replacement N times (seed
20261010) on the fixed graph and tree: a work drawn c times counts c times
in every subtree, edge (`w_ij c_i c_j`), coverage, class size and
navigation cost. The draws depend only on n, the seed and the replicate, so
every row of a universe, under every condition, sees the same resamples:
differences and rank correlations are paired. Skill uses the reference
tree's cost on the same resample. The tree shape (depths, fan-outs) is
fixed, so `max_branching` and similar shape counts have no interval.

`--reference-replicates` reads the `ref_cost_norm` series of the reference
run's replicates, the average-linkage tree's cost on each resample. Before the
fix in this run, it read `cost_norm`, the cost of the scheme tree the
reference run scored, and every skill interval was wrong. Rescore with a
`shelfmark-bench` that has the fix. Point values do not depend on it.

This run took 446 s for 468 runs and 27 references, with 2 concurrent jobs,
at a sampled peak RSS of 0.40 GiB.

## 5. Publish

```sh
python3 crates/bench/scripts/publish_embed.py --harness $BIN/shelfmark-bench \
  --learned $BIN/learned-scheme --runtime bench/corpus/cache/embed-runtime.json \
  --notes bench/corpus/cache/embed-notes.md
```

It writes `results.json` (graphs, references, every run with its intervals,
rank correlations, the learned scheme's margins, the cross-fit matrix and
the check of the MiniLM k = 20 rescoring against the published runs),
`runs/<condition>/<universe>/<row>.json` (the full harness output),
`placements/`, `refit/` and `graphs/`. The tables of `results.md` are
generated; its opening prose (`embed-notes.md`) is written by hand.
