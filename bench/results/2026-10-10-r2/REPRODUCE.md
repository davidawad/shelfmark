# Reproducing the 2026-10-10-r2 results

This run repeats only the shelfmark-placed part of the 2026-10-10 benchmark
(`../2026-10-10/`) after the vocab tier stopped matching class labels and
terms made only of English stopwords (`vocab-2`). The built-in profiles moved
from `profile:lexical@1` to `profile:lexical@2`; every placement request in
this run names `profile:lexical@2`. The graphs, scheme files and
average-linkage baselines are the 2026-10-10 ones, and the gold results are
not rerun: they use the source labels, not the classifier.

All commands run from the repository root on Linux with network access.
Intermediate files go under `bench/corpus/cache/`, which git ignores.

## 1. Tools

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-cli -p shelfmark-bench
BIN=${CARGO_TARGET_DIR:-target}/release
```

No Python packages are needed: the graphs are not recomputed.

## 2. Similarity graphs

Decompress the published 2026-10-10 graphs and check them against their
manifests:

```sh
mkdir -p bench/corpus/cache/graphs
for g in corpus holdout; do
  gzip -dc bench/results/2026-10-10/graphs/$g-k20.json.gz > bench/corpus/cache/graphs/$g-k20.json
  cp bench/results/2026-10-10/graphs/$g-k20.manifest.json bench/corpus/cache/graphs/
done
sha256sum bench/corpus/cache/graphs/*-k20.json
```

The hashes must be the manifests' `graph_sha256`:
`759d3b8461a9384d4359244c9fb42c2bc66496431e0f4681d33b7a59798ed35a`
(corpus) and
`03d67d06b82633e1ca3461e3c9f11f3ee3bd55f43079a28b2126a1a90c1f6c75`
(holdout). `run_benchmark.py` also checks them and stops on a mismatch.

## 3. Schemes

```sh
crates/bench/scripts/build_schemes.sh
```

Same builders as 2026-10-10. Every `status.tsv` hash must equal that
scheme's `file_sha256` in `../2026-10-10/results.json`; in this run all 13
did.

## 4. Placements and scoring

```sh
python3 crates/bench/scripts/run_benchmark.py --shelfmark $BIN/shelfmark \
  --harness $BIN/shelfmark-bench --jobs 4 --placed-only \
  --reference-from bench/results/2026-10-10/results.json
```

`--placed-only` skips the gold runs. `--reference-from` takes each graph's
average-linkage `ref_cost_norm` from the published results (corpus
0.15183392202502477, holdout 0.301409582139011) after checking that its
`graph_sha256` matches the local graph, and passes it to every scoring run
as `--reference-cost`. Everything else is as in 2026-10-10 step 4.3 and 4.4:
one isolated store per scheme with an empty `rules.json`, requests through
`shelfmark batch` with `profile:lexical@2`, low-confidence placements kept,
fallback placements (no candidate) counted as unplaced, every built scheme
placed over the arXiv corpus and the general schemes (`lcc`, `lcc-full`,
`fdc`, `bk`, `ilc2`, `unesco-thesaurus`) over the holdout as well.

## 5. Publish

```sh
python3 crates/bench/scripts/publish_results.py --out bench/results/2026-10-10-r2 \
  --harness $BIN/shelfmark-bench --shelfmark $BIN/shelfmark \
  --runtime bench/corpus/cache/runtime.json --baseline bench/results/2026-10-10
```

With `--baseline`, the graphs are referenced from the baseline directory
instead of being copied, the gold section of `results.md` points to the
baseline, and `results.json` gains `baseline` and `comparison` (coverage,
cost, skill and largest single-class share of each placed run, before and
after). `results.json` also records the classifier commit and binary hash.

The placed scores depend on the graph, the scheme file, the placements and
so on the shelfmark build; with the same build they reproduce exactly.
