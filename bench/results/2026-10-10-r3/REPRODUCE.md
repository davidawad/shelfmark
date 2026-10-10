# Reproducing the 2026-10-10-r3 results

This run rescores only the gold placements of `acm-ccs` and `msc` on the
fixed corpus. `corpus.jsonl` has sha256
`1027bf9d34928f470ed8b0d7b5ddc5c983fde49bc0cd33eaa162bbd1f830fea5`: its
`acm_ccs` field now comes from `arXivOld` and its `msc` field holds only MSC
2020 codes (see `results.md` and `bench/corpus/README.md`). The graphs,
average-linkage baselines and scheme files are the 2026-10-10 ones
(`../2026-10-10/`). The shelfmark-placed results are not rerun: the
classifier reads only titles and abstracts, which did not change.

All commands run from the repository root on Linux with network access.
Intermediate files go under `bench/corpus/cache/`, which git ignores.

## 1. Tools

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-cli -p shelfmark-bench
BIN=${CARGO_TARGET_DIR:-target}/release
```

No Python packages are needed. `run_benchmark.py` requires a `shelfmark`
binary, but gold runs over the arXiv corpus do not call it.

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
(holdout). In this run they did. `run_benchmark.py` also checks them and
stops on a mismatch. The manifests name the corpus the graphs were built from
(`7685685d…`). The graph script embeds only `title` and `abstract_text`, in
row order. Neither corpus fix changed the row order, `id`, `title` or
`abstract_text`, so the graph is still the graph of this corpus.

## 3. Schemes

```sh
crates/bench/scripts/build_schemes.sh acm-ccs msc
```

This builds only the two fetch-only schemes from upstream. Each builder checks
its download against the committed `sources.lock.json`. Do not commit the
outputs. Both `status.tsv` hashes must equal the `file_sha256` values in
`../2026-10-10/results.json`. In this run both did: acm-ccs
`24b0d8242b2ae613d1c7f0cafd4889ea71ed34625c4ebbe5e0de3974b5883ecf`, msc
`7a50db75a5fb7f7656feb48d1a7a6d3ffd23a71bce244be363e064c1f9b1aac6`.

## 4. Gold placements and scoring

```sh
python3 crates/bench/scripts/run_benchmark.py --shelfmark $BIN/shelfmark \
  --harness $BIN/shelfmark-bench --gold-only \
  --reference-from bench/results/2026-10-10/results.json
```

`--gold-only` skips the shelfmark-placed runs. Only built schemes run, so the
two gold runs are `acm-ccs` and `msc`. `--reference-from` takes the
average-linkage `ref_cost_norm` of each graph from the published 2026-10-10
results (corpus 0.15183392202502477, holdout 0.301409582139011). It first
checks that the recorded `graph_sha256` matches the local graph. It then passes
each value to the scoring runs as `--reference-cost`.

- `msc`: `prepare_placements.py normalize-msc` keeps each paper's `msc` codes
  that are codes of the scheme, in source order. It accepts every shape in the
  scheme grammar: `05C10`, `05Cxx`, `05-XX` and `05-01`. 1,322 papers get a
  label. The harness scores the first one.
- `acm-ccs`: `normalize-codes --field acm_ccs` keeps the labels that are codes
  of the scheme. None is: the corpus has CCS 1998 codes such as `I.2.6`, but the
  scheme has CCS 2012 concept-ID paths. The run is skipped, and the reason is
  recorded.

The 2026-10-10 normalizer accepted only five-character codes such as `05C10`.
With it, the gold rows from the fixed corpus equal those from the old one, and
`shelfmark-bench run` reproduces `../2026-10-10/msc-gold.json` exactly,
including tree hash
`6310d7844ec8583d66136ae0f40b252613c6cd052560df4a75b97bcb1f384db0`.

## 5. Publish

Write the notes that open `results.md` to `bench/corpus/cache/r3-notes.md`
(the "Corpus fixes since 2026-10-10" section of `results.md`). Write the stage
timings to `bench/corpus/cache/runtime.json`. Then run:

```sh
python3 crates/bench/scripts/publish_results.py --out bench/results/2026-10-10-r3 \
  --harness $BIN/shelfmark-bench --runtime bench/corpus/cache/runtime.json \
  --baseline bench/results/2026-10-10 --gold-only --notes bench/corpus/cache/r3-notes.md
```

This writes:

- `msc-gold.json`: the scored run.
- `acm-ccs-gold.json`: the skip record, with its reason and normalizer
  statistics.
- `results.json`: corpus hashes, the baseline's graphs and references, scheme
  hashes, skips, and a before-and-after `comparison` against the 2026-10-10
  gold results.
- `results.md`.

With the same graph, scheme files and corpus, the gold results reproduce
exactly.
