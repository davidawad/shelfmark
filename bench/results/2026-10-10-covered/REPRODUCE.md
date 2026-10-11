# Reproducing the 2026-10-10-covered results

This run rescores every run of `../2026-10-10-scores/` on all items and on
covered subsets, analyses how much of each score's ranking is coverage,
and checks the kNN-sparse Dasgupta cost against the dense one. It uses the
same inputs as the scores run: the published graphs, the schemes and
baselines of `../2026-10-10/`, the placements of `../2026-10-10-r2/` and
`../2026-10-10-r3/`, the learned-scheme runs of `../2026-10-10-learned/`
and the pinned embedding vectors.

All commands run from the repository root on Linux (steps 2–4 from
`crates/bench/scripts`, as shown). Intermediate files go
under `bench/corpus/cache/`, which git ignores. The whole run took about
35 minutes on 8 CPUs. Regenerating the placements and embeddings is most
of it; the scoring takes about 4 minutes and the dense check under 1.

## 1. Inputs

Follow `../2026-10-10-scores/REPRODUCE.md` steps 1–3 (tools, graphs,
schemes, placements, learned-scheme placements, embedding vectors). In
this run all 13 scheme files hashed as published, every harness result
reproduced the published files (tree SHA-256 and cost_norm), and both
vector files had the published SHA-256:

- corpus: `5659678124f989f0ab1a6771777d81527970b8f455618c91fb9c98ef22f9c2e5`
- holdout: `143b8f076319c48ffa84d95ce90299e5faf8d000f1c7dac364dffba35ebeccb1`

`results.json` pins every input hash, the subset id files and the harness
binary.

## 2. Scores on all items and on covered subsets

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-bench
BIN=${CARGO_TARGET_DIR:-target}/release
cd crates/bench/scripts
python3 covered_catalogue.py run --harness $BIN/shelfmark-bench --jobs 6
```

Use an absolute `$BIN`. This first scores the 28 runs of the scores run on
all items, with the same `shelfmark-bench scores` call as
`score_catalogue.py` plus `--write-covered <cache>/covered/ids/<run>.txt`
(the ids of the items the run places). It checks that each catalogue
cost_norm equals the harness value. It then scores, with
`--subset <ids file>`:

- every run whose covered subset has at least 30 items and is not the
  whole corpus, on its own covered subset (`covered/own/`);
- every run of a corpus on each gold run's covered subset
  (`covered/<gold run>/`): arxiv-gold and msc-gold on the papers, lcc-gold
  and lcc-full-gold on the books. jel-gold labels 2 papers and is reported
  by size only;
- each gold run and its shelfmark-placed counterpart (arxiv, msc, lcc,
  lcc-full) on the items both place (`covered/both-<gold run>/`).

`--subset` keeps the listed items in corpus order, the graph edges with
both endpoints among them (renumbered, original order) and their vectors.

The bootstrap draws edge, item and kNN-probe resamples from separate
SplitMix64 streams (seed 20261010), so two runs on the same graph and item
set get the same resamples in every replicate and their differences can be
bootstrapped pairwise. This changes the bootstrap intervals relative to
`../2026-10-10-scores/` (one stream there); point scores are unchanged.
The vertex bootstrap (`vertex.*` keys) is new.

## 3. Dense against sparse cost

```sh
python3 covered_catalogue.py dense --harness $BIN/shelfmark-bench --threads 8
```

For the papers this calls

```sh
shelfmark-bench dense --graph <cache>/graphs/corpus-k20.json --vectors <cache>/embed/corpus.f32 \
  --runs <cache>/covered/dense-corpus.runs.json --sizes 1000,2000,5000 --ks 10,20,50 \
  --subsets 5 --boot 200 --seed 20261010 --threads 8 --output <cache>/covered/dense-corpus.json
```

and for the books the same with `--sizes 500` (the whole holdout, one
subset). The runs file lists every run of the corpus with its scheme,
label source and placements. Check of the construction: with
`--sizes 10000 --ks 20` on the papers the kNN graph has 148,111 edges, as
the published graph does, and its cost_norm is within 6·10⁻⁶ of the
harness value for arXiv, MSC and JEL gold (a few near-tied neighbours
differ: cosines are summed in f64 here, in f32 by numpy); the published
graph restricted to all items gives the harness value to 10⁻⁹.

## 4. Publish

Write the opening notes to `bench/corpus/cache/covered-notes.md`, then:

```sh
python3 covered_catalogue.py publish --harness $BIN/shelfmark-bench \
  --notes ../../../bench/corpus/cache/covered-notes.md
```

This runs `shelfmark-bench agree` per group (gold, placed-corpus,
placed-holdout) with `"control": "coverage"` (Spearman ρ with coverage,
partial Spearman matrix on coverage, intervals over the bootstrap
replicates) and the covered-subset values as the comparison set (Kendall τ
between the all-items and covered rankings), and with `"paired": true`
for the placed groups and every gold subset (paired-bootstrap differences
between every two runs). It writes `results.json` (every run on all items
and on its covered subset, every gold-subset run, without bootstrap
replicates; the analyses; the dense check) and `results.md`.
