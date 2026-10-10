# Reproducing the 2026-10-10-semantic results

This run repeats the shelfmark-placed part of the benchmark (as in
`../2026-10-10-r2/`) once per classifier profile:

- `lexical` = `profile:lexical@2` (identifier, vocab and rules tiers), the r2
  profile, rerun here so all three tiers carry bootstrap intervals and
  published placements;
- `shortlist` = `profile:shortlist@2`, the lexical tiers and then the BGE-small
  embedding shortlist (`crates/tier-shortlist`);
- `rerank` = `profile:rerank@2`, the shortlist profile and then a
  cross-encoder rerank. No rerank runtime ships in this build: the `rerank`
  tier reports `tier_unavailable`, the cascade records it as `unavailable`
  and carries on, so its decisions are the shortlist profile's.

The graphs, scheme files and average-linkage baselines are the 2026-10-10
ones. Gold results are not rerun: they use the source labels, not the
classifier.

All commands run from the repository root on Linux with network access (the
BGE-small model and ONNX Runtime are downloaded on first use). Intermediate
files go under `bench/corpus/cache/`, which git ignores.

## 1. Tools

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-cli --features semantic-shortlist -p shelfmark-bench
mkdir -p bench/corpus/cache/bin
cp ${CARGO_TARGET_DIR:-target}/release/{shelfmark,shelfmark-bench} bench/corpus/cache/bin/
BIN=bench/corpus/cache/bin
```

Copying the binaries keeps a later workspace build, which does not enable
`semantic-shortlist`, from replacing the classifier mid-run. The model files
go to `$XDG_CACHE_HOME/shelfmark/models` (or `~/.cache/shelfmark/models`).

## 2. Similarity graphs

As in `../2026-10-10-r2/REPRODUCE.md` step 2: decompress
`../2026-10-10/graphs/{corpus,holdout}-k20.json.gz` into
`bench/corpus/cache/graphs/` with their manifests. The hashes must be
`759d3b8461a9384d4359244c9fb42c2bc66496431e0f4681d33b7a59798ed35a`
(corpus) and
`03d67d06b82633e1ca3461e3c9f11f3ee3bd55f43079a28b2126a1a90c1f6c75`
(holdout).

## 3. Schemes

```sh
crates/bench/scripts/build_schemes.sh
```

Every `status.tsv` hash must equal that scheme's `file_sha256` in
`../2026-10-10/results.json`; in this run all 13 did.

## 4. Placements and scoring

One tagged run per profile. `--tag` keeps each profile's stores, placements
and scores apart (`bench/corpus/cache/{stores,placed,runs}/<tag>/`);
`--bootstrap 200` adds item-bootstrap 95% intervals to every score (seed 1,
the default, for every run, so all runs over one corpus are paired).

```sh
COMMON="--shelfmark $BIN/shelfmark --harness $BIN/shelfmark-bench --placed-only \
  --reference-from bench/results/2026-10-10/results.json --bootstrap 200 --jobs 3 --score-threads 2"
python3 crates/bench/scripts/run_benchmark.py $COMMON --cpus-per-job 5 \
  --profile profile:shortlist@2 --tag shortlist
python3 crates/bench/scripts/run_benchmark.py $COMMON --cpus-per-job 5 \
  --profile profile:rerank@2 --tag rerank
python3 crates/bench/scripts/run_benchmark.py $COMMON --profile profile:lexical@2 --tag lexical
```

The tiers ran one after another with at most three placement jobs at once
(16 CPUs, 31 GiB RAM). `--cpus-per-job 5` pins each semantic batch to its
own five CPUs: ONNX Runtime sizes its thread pool from the CPUs the process
may use. In this run shortlist `lcc-full` was placed first on its own
(`--only lcc-full`, scored on 5 threads) to measure memory, then the
other schemes; neither `--only` nor the thread count changes any output
(rescoring that run on 2 threads gave identical intervals).

Memory: a semantic corpus batch held about 1.2 GiB and a bootstrap scoring
run at most 0.44 GiB (2 threads). The largest process was a holdout batch
for `lcc-full` (6.9 GiB): it shares the scheme's store with the corpus
batch and loads the 1.46 GB `memo.jsonl` that batch leaves. The summed
resident memory of all shelfmark and harness processes peaked at 8.0 GiB
(sampled every 5 s). An earlier attempt that ran eight semantic jobs at
once ran out of memory; with `--jobs 3` plan for about 8 GiB.

Everything else is as in r2 step 4: one isolated
store per scheme (here per scheme and tier) with an empty `rules.json`,
requests through `shelfmark batch`, low-confidence placements kept, fallback
placements (no candidate) counted as unplaced, every scheme placed over the
arXiv corpus and the general schemes (`lcc`, `lcc-full`, `fdc`, `bk`, `ilc2`,
`unesco-thesaurus`) over the holdout as well.

### Bootstrap

`shelfmark-bench run --bootstrap N [--seed S] [--threads T]`
(`crates/bench/src/bootstrap.rs`). Each resample draws n items with
replacement; a drawn item keeps its placements, every graph edge between two
originals becomes an edge between every pair of their copies, the scheme tree
is induced again from the drawn items, and the resample is scored like the
full run. Intervals are percentile intervals. The draws depend only on n,
the seed and the round, so replicate r of every scheme and tier saw the same
items; `publish_semantic.py` uses the kept replicates (coverage, cost_norm,
revenue_norm, skill) for paired differences and correlation intervals. The
average-linkage reference is not refitted per resample: skill uses the
full-sample `ref_cost_norm`, so its interval carries the scheme's cost
variability only. The intervals do not depend on `--threads`.

## 5. Publish

```sh
python3 crates/bench/scripts/publish_semantic.py --out bench/results/2026-10-10-semantic \
  --tier lexical=profile:lexical@2 --tier shortlist=profile:shortlist@2 \
  --tier rerank=profile:rerank@2 --harness $BIN/shelfmark-bench --shelfmark $BIN/shelfmark \
  --runtime bench/corpus/cache/runtime-semantic.json --graphs-from bench/results/2026-10-10 \
  --check-against bench/results/2026-10-10-r2
```

It writes `<run>.<tier>.json` per scored run, the placements under
`placements/<tier>/` (gzipped if over 20 MB; none was), the graph manifests
(pointing at the 2026-10-10 graphs), `results.json` and `results.md`.
`--check-against` compares the lexical tier's coverage, cost_norm and skill
with r2's (all 19 equal here). Next to each tier's coverage/skill
correlation (undefined for the semantic tiers, which place every item), it
reports the correlation of the coverage gain with the skill gain over the
lexical tier, run by run, with paired bootstrap intervals.

The placed scores depend on the graph, the scheme file, the placements and
so on the shelfmark build and the model; with the same build and model they
reproduce exactly. Rescoring needs only the published placements, the
2026-10-10 graphs and the scheme files:

```sh
$BIN/shelfmark-bench run --corpus bench/corpus/corpus.jsonl \
  --scheme bench/corpus/cache/schemes/msc.json --graph bench/corpus/cache/graphs/corpus-k20.json \
  --k 20 --reference-cost 0.15183392202502477 \
  --placements bench/results/2026-10-10-semantic/placements/shortlist/corpus-msc.jsonl \
  --bootstrap 200
```
