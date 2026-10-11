# Reproducing the book-side rank stability sweep

Run from the repository root on Linux. This directory extends
`../2026-10-11-stability/` (paper corpus, 13 schemes, no false-join term) to
report 9's book claims. It scores every report 9 run on the 3,000 books
(`bench/corpus/holdout-books.jsonl`) and on the 500-book title-only holdout
(`bench/corpus/holdout.jsonl`), and the classifier-placed runs on the
10,000 papers (`bench/corpus/corpus.jsonl`). Each universe gets 3 embedding
models × k ∈ {10, 20, 50} × 3 graph thresholds (27 settings).

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-bench -p shelfmark-cli
crates/bench/scripts/build_schemes.sh lcc-full ilc2      # fetch-and-build schemes; hashes match the published runs
python3 -I bench/results/2026-10-11-book-stability/generate_book_stability.py all \
  --harness <target>/release/shelfmark-bench
```

## What each setting is

- **Graph.** Books: `../2026-10-11-metrics/books/graphs/<model>-books-k20.json`
  at k = 20, `../2026-10-10-proposal/graphs/<model>-books-k<k>.json.gz`
  otherwise. Holdout and papers: `../2026-10-11-metrics/embed/graphs/` at
  k = 20, `../2026-10-10-embed/graphs/` otherwise. These are the graphs of the
  published runs.
- **Threshold.** This is `generate_stability.py`'s sweep. q00 is the
  published graph. q25 and q50 subtract the 25th or 50th percentile of the
  positive edge weights from every edge and drop the edges that reach zero.
  The cut removes the weakest quarter or half of the edges, so the graph
  is sparser.
- **Negatives.** These are the committed false-join samples of
  `../2026-10-11-falsejoin/negatives/`, for books at each k. The holdout and
  papers have samples at k = 20 only. Those are reused at k = 10 and 50: a
  negative pair has a cosine below τ, the corpus's mean pair cosine. τ is
  the same at every k, and the generator checks it against the graph, so
  such a pair is never a kNN edge at any k.
- **Runs.** These are the runs of the published false-join bootstrap of the
  universe (its `runs_spec`). The holdout also gets the Dewey cuts
  `ddc-section` and `ddc-gold`, and the papers keep only their `-placed`
  runs. Every universe also gets `lcc-lcsh-placed`: the scheme the
  `shelfmark` binary bundles (the LCC outline plus LCSH vocabulary,
  `data/schemes/lcc/lcc-lcsh.json.gz`) placed by the same classifier as the
  other `-placed` runs. The published `lcc-placed` runs compiled `lcc.json`,
  the outline without LCSH, into their store.
- **Scoring.** There is one `shelfmark-bench bootstrap --negatives` call per
  setting: 1,000 paired item draws, seed 20261010, at most two calls at a
  time. Every q00 setting must reproduce the published false-join points
  (cost_norm, fjs_norm, nav_log_a1) to 1e-9. All 1,005 run × setting checks
  passed.

`lcc-lcsh-placed` was made with the CLI on an empty store whose rules file
is empty, `profile:lexical@2`. Low-confidence placements are kept and
fallbacks are left unplaced, as for the published `-placed` runs:

```sh
SM=<target>/release/shelfmark; PREP="python3 crates/bench/scripts/prepare_placements.py"
mkdir -p store && echo '{"rules":[]}' > store/rules.json
$PREP requests --corpus bench/corpus/holdout-books.jsonl --scheme scheme:lcc@2024 \
  --profile profile:lexical@2 --output requests.jsonl
$SM --json --store store batch --requests requests.jsonl --out batch.jsonl
$PREP extract --corpus bench/corpus/holdout-books.jsonl --batch batch.jsonl --scheme scheme:lcc@2024 \
  --profile profile:lexical@2 --output lcc-lcsh-placed.jsonl
```

The same commands make the holdout and paper placements, with a fresh store
for each corpus. The run is scored against `data/schemes/lcc/lcc.json`,
which has the same 7,392 classes in the same order. The books' text includes
their subject headings, which are mostly LCSH, so LCSH vocabulary has a home
advantage on the books. That advantage is absent on the title-only holdout
and the papers.

## Objectives and checks

All four objectives are lower-is-better. C is cost_norm. J = C + 0.002·N,
where N is nav_log_a1. C + FJs adds the specificity false-join term at
β = 1. J′ = C + FJs + 0.002·N is report 9's corrected objective.

- **Pair claims.** Each claim's paired difference comes from the same
  bootstrap draws in every setting. The verdict is the share of settings
  with the claimed sign and the number whose 95% CI excludes zero each way.
- **Ranking claims.** The check is the minimum Spearman ρ over all 351
  setting pairs, against a threshold of 0.8.
- **Bead rule.** Among the placed runs with coverage ≥ 0.80, the highest
  skill wins. Runs whose paired difference to it is not significant tie
  with it, and a tie goes to the lowest nav_log_a1. The rule is reported
  for all placed runs and for those the binary may bundle, and at floors
  from 0.70 to 0.95.
- **Regret.** This is report 9's `fj-cuts` minimax regret of J′ over
  λ ∈ [0.0005, 0.005] and β ∈ [0.5, 2], on the 9 published graphs and on
  all 27 settings.

## Steps and outputs

- `score` runs the bootstrap calls and writes one compact summary per
  setting to `settings/`. Each summary holds the points, CIs and paired
  deltas of the claim pairs and of every pair of placed runs. The raw
  bootstrap and draw files go to `--raw` (default
  `/work/artifacts/book-stability/raw`); they are not committed.
- `summarise` rebuilds `settings/` from the raw files without rescoring.
- `publish` writes `results.json` and `results.md` from `settings/`.

`score` skips any setting whose summary exists, unless given `--force`.
