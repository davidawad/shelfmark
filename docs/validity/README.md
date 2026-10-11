# Librarian-side validity check of the Dasgupta-cost metric

Question: does the benchmark's Dasgupta-cost metric reward what a librarian
would call good classification, or only agreement with the embedding
geometry? This directory holds a stratified pair sample, a fixed judging
rubric, one LLM judge's blind labels for every pair, and the agreement
between those labels and the metric.

Files:

- `rubric.md`: the judging rubric. It was fixed before any pair was judged
  and was used unchanged for both passes.
- `sample.jsonl`: one row per sampled pair. A row holds the pair id
  (`run:id_a|id_b`), the stratum, both assigned classes, the metric fields
  and the judge labels (`labels.pass1`, plus `labels.pass2` for the repeat
  subset).
- `sample-meta.json`: seed, input hashes, and per-run tree checks and pool
  sizes.
- `results.md` / `results.json`: the agreement tables, generated from
  `sample.jsonl`.
- `../../crates/bench/scripts/validity_check.py`: regenerates the sample, the
  blinded batches and the tables. It uses only the standard library.

## Method

**Inputs.** The published corpus graph (`bench/results/2026-10-10/graphs`,
k = 20, sha256 `759d3b84…`) and the placements regenerated exactly as
`bench/results/2026-10-10-scores/REPRODUCE.md` steps 1–3 say. Those steps
cover the r2 placements, the learned scheme's `placed` step and the
fastembed vectors. The rebuilt vectors hash to `5659678…`, the hash in
REPRODUCE.md. For every run, the script rebuilds the induced tree the way
the harness does: the virtual root is at depth 0, top classes are at
depth 1, gold runs use each item's first label, and unplaced items hang off
the root. It recomputes the sparse Dasgupta cost and stops if the cost
differs from the harness `cost_norm` by more than 1e-9. All eight runs
matched.

**Runs.** `lcc-full-placed`, `fdc-placed`, `ilc2-placed`,
`unesco-thesaurus-placed`, `openalex-topics-placed`, `arxiv-gold`,
`msc-gold` and `learned-placed`.

**Strata** (seed 20261010, 40 pairs per stratum per run):

- **A**: high-similarity pairs the scheme *splits*. These are graph edges
  with weight at or above the median edge weight, between placed items,
  whose LCA is the root or a depth-1 class.
- **B**: low-similarity pairs the scheme *joins deep*. The cosine is at or
  below the graph's τ, so the pair has zero weight in both the sparse graph
  and the dense `max(0, cos − τ)` graph. The LCA depth is ≥ 2 and ≥ the
  run's deep threshold, which is the class depth of the median placed item;
  meeting at either item's own class also counts. Pairs are drawn uniformly
  from that population. `msc-gold` has only 33 such pairs among its 13%
  placed items, so all of them are used.
- **C**: uniform random pairs of placed items (the control).

That gives 953 pairs.

**Metric side.** A pair's cost contribution is `w · |leaves(LCA)| / n`, its
term in the normalized Dasgupta cost. The metric *flags* a pair when that
contribution exceeds the run's mean contribution per graph edge, i.e. when
the pair raises the cost above average. The ranking comparison uses the raw
contribution (AUC). The dense-graph variant (`w = max(0, cos − τ)`) is
reported alongside; it gives the same κ to within 0.01.

**Blinding.** `validity_check.py blind` writes batches of 40 pairs. Each
pair carries only an opaque uid, the scheme name, each work's title and
abstract (cut at about 900 characters), and each work's class path with
labels. Stratum, cosine, edge weight, LCA mass and contribution are left
out. Pairs from all runs and strata are shuffled together, and the order of
the two works is randomized. The uid → pair key is written outside the repo,
under the ignored cache. Pass 2 is a seeded 25% subset (238 pairs). It gets
fresh uids, a fresh shuffle and a fresh work order, and is judged without
access to the pass-1 labels.

**Judge.** Claude Opus 5.5 is the only annotator; no external model or paid
API was used. To fit the volume, the judging ran as 15 independent
instances of the same model, 12 for pass 1 and 3 for pass 2. Each handled
two batches. Each instance could read only `rubric.md` and its own batch
files. Each was told to label every pair from its own reading, with no
label-producing scripts. For each pair the judge gives:

- the subject relation (same-topic / same-field / different-field);
- whether a careful librarian would put the two works *together* or *apart*;
- whether this scheme's decision for the pair is *correct* or *incorrect*;
- whether each assigned class is defensible;
- a short reason.

**Agreement.** Per run, the report compares the judge's `scheme_decision =
incorrect` with the metric's flag: percent agreement, Cohen's κ, and AUC of
the raw contribution against the judge verdict. It does the same per
stratum. It also compares the judge's together/apart call with "the pair is
a graph edge", and it gives judge self-consistency as pass-1 vs pass-2 κ.
Intervals are 95% percentile bootstraps with 2000 replicates, resampling
pairs within strata (seeded).

## Results

Full tables are in `results.md`. Per-run κ between the judge's verdict and
the metric's flag, with 95% CI:

| run | judge: incorrect | metric: flagged | agreement | κ [95% CI] | AUC |
|---|---|---|---|---|---|
| lcc-full-placed | 0.72 | 0.28 | 0.55 | 0.25 [0.18, 0.32] | 0.71 |
| fdc-placed | 0.69 | 0.30 | 0.59 | 0.29 [0.21, 0.37] | 0.72 |
| ilc2-placed | 0.72 | 0.30 | 0.56 | 0.25 [0.18, 0.33] | 0.69 |
| unesco-thesaurus-placed | 0.68 | 0.25 | 0.55 | 0.24 [0.16, 0.32] | 0.69 |
| openalex-topics-placed | 0.64 | 0.32 | 0.59 | 0.26 [0.16, 0.35] | 0.68 |
| arxiv-gold | 0.07 | 0.23 | 0.79 | 0.24 [0.04, 0.43] | 0.75 |
| msc-gold | 0.19 | 0.25 | 0.73 | 0.20 [0.00, 0.40] | 0.60 |
| learned-placed | 0.31 | 0.33 | 0.71 | 0.33 [0.15, 0.50] | 0.68 |
| all runs | 0.51 | 0.28 | 0.63 | 0.27 [0.22, 0.32] | 0.65 |

Shares and κ are over the stratified sample (one third per stratum), not
over the population of pairs.

Judge self-consistency on the 238 repeated pairs: `scheme_decision` κ = 0.92
[0.87, 0.97] (96% agreement), `together` κ = 0.87 and `relation` κ = 0.87.
Within itself, the judge is stable.

Per stratum (all runs pooled; judge-incorrect / metric-flagged / agreement):

- **A (similar, split):** 0.70 / 0.84 / 0.72, κ = 0.23. The metric and the
  judge agree that most of these splits are bad *for the placed general
  schemes*: the judge calls 88–97% of them false splits. For the gold
  schemes, the metric flags 70% of the A pairs, but the judge calls only 15%
  (arXiv) and 23% (MSC) incorrect. Only 38% and 35% of those pairs are ones
  a librarian would keep together. Embedding similarity there comes from
  shared method or vocabulary across specialties, and the scheme is right to
  split them. For the learned scheme the metric flags 100% and the judge
  53%.
- **B (dissimilar, joined deep):** 0.70 / 0.00 / 0.30. The metric flags *no*
  pair in this stratum, and it cannot: a zero-weight pair adds nothing to
  the Dasgupta cost however deep it is joined. Its only effect is to enlarge
  the LCA mass of the edges that pass through it. The judge calls 93–97% of
  these pairs false joins in the five placed general schemes, 38% in the
  learned scheme, 36% in MSC and 7% in arXiv.
- **C (random):** 0.13 / 0.00 / 0.87. Random pairs are almost never graph
  edges (2 of 320), so the metric flags none. The judge finds 10–28% of
  random pairs wrongly handled in the placed general schemes, and 0–3% in
  the gold and learned schemes. Of the 41 such pairs, 33 are false splits:
  same-field pairs that misplacement sent to different top classes. Their
  edge weight is zero, so the metric does not charge for them either.

Other observations:

- **Graph against librarian.** Only 57% of the sampled graph edges are pairs
  the judge would put together; this figure is dominated by stratum A,
  i.e. strong edges that schemes split. The cost charges each edge for being
  split whether or not a librarian would join the pair.
- **Placement quality.** The judge rejects 81–92% of the assigned classes in
  `lcc-full-`, `fdc-`, `ilc2-` and `unesco-thesaurus-placed`, 57% in
  `openalex-topics-placed`, 23% in `learned-placed`, and 1% and 8% in the
  arXiv and MSC gold. The high cost of the placed general schemes reflects
  bad placement more than the schemes themselves. Both the metric and the
  judge see this, which accounts for most of their agreement.
- **Across schemes.** Spearman ρ between `cost_norm` and the judge's error
  rate on random pairs is 0.23 over the 8 runs (0.42 on all strata). Two
  cases break the ranking:
  - `msc-gold` has the worst cost (0.98) but few judged errors on its placed
    items. Its 87% unplaced items hang off the root, so the cost measures
    coverage, not classification.
  - `learned-placed` scores close to arXiv gold on cost (0.38 vs 0.33), but
    the judge finds about 4× as many wrong decisions (0.31 vs 0.07 over the
    sample). The learned scheme is optimized against the same similarity
    geometry the metric reads.

## Verdict

The metric rewards librarian-good classification only partly, and mostly
through the embedding geometry.

- **Above chance:** its per-pair cost ranking agrees with the judge in every
  run (pooled κ = 0.27 [0.22, 0.32], AUC 0.65). It separates badly placed
  schemes from the gold schemes.
- **Weak:** that agreement is far below the judge's own repeatability
  (κ = 0.92).
- **Structurally blind:** it never penalizes a false join of dissimilar
  works.
- **Penalizes good decisions:** it charges a scheme for splitting
  embedding-similar works that a librarian would keep apart, and it rewards
  a scheme built to fit the same geometry (`learned-placed`) more than the
  judge does.

Read Dasgupta cost here as a measure of agreement with the embedding
similarity graph, not as a measure of classification correctness. It needs a
complementary check: a false-join measure, a coverage-separated score, or a
judged sample like this one.

## What this does and does not show

It shows how one LLM judge, following a fixed rubric, labels a seeded,
reproducible, stratified sample. It shows how those labels relate to each
pair's term in the Dasgupta cost, and that the judge is self-consistent on
a blind repeat.

It does **not** show agreement with human librarians.

- There is a single annotator model; the 15 instances share one model and
  one rubric. No human labelled anything, and no second model was used, so
  inter-annotator reliability is unknown. High self-consistency is not
  validity: the judge's own biases are repeated, not averaged out. For
  example, it may favour the arXiv taxonomy, whose categories are familiar
  from its training data.
- The judge sees the class paths, so its pair verdict and its class-validity
  verdict are not independent.
- Abstracts are truncated.
- The sample is stratified by design: pooled shares and κ depend on the
  stratum mix (one third each) and are not population estimates. Stratum A
  and B pairs are extreme cases chosen to probe the metric's failure modes.
- The per-pair flag (contribution above the mean edge contribution) is one
  reasonable reading of "the metric thinks this decision is bad"; the
  Dasgupta cost itself is a sum, and a pair's effect on the tree is not
  only its own term.
- The results cover one corpus (10,000 arXiv papers),
  one embedding model and one k = 20 graph.

## Human-librarian check (pending a librarian)

`human/` holds the packet for the missing human labels. It is a seeded subset of
`sample.jsonl`: 5 pairs per run and stratum, 120 pairs in all, blinded and
shuffled the same way as for the judge. `packet.md` shows each pair's two
works and class paths. `labels.csv` is blank, with one row per pair, and
`annotator.md` records who labelled. The labeller follows `rubric.md` and
does not open the sample or the results first.

Running `python3 crates/bench/scripts/validity_check.py human` regenerates the
packet. It needs the eight scheme files and checks each against
`sample-meta.json`: six are committed, and `ilc2` and `msc` come from
`crates/bench/scripts/build_schemes.sh ilc2 msc`. The uid is a hash of the
pair id, so there is no key file. Once `labels.csv` is filled in,
`python3 crates/bench/scripts/validity_check.py human-report` writes
`human/results.md` and `human/results.json`. These give the human labels'
agreement and Cohen κ with the metric flag and with the LLM judge's pass-1
labels on the same pairs, which shows how far the judge-based numbers above
stand in for a librarian.

## False-join re-validation (2026-10-11)

This check found that the metric never charges a false join. The bench
now has a false-join term for that, with an outside-mass form and a
specificity form (`crates/bench/src/falsejoin.rs`), and a combined
objective C + β · FJ. `crates/bench/scripts/falsejoin_validity.py judge`
re-runs the agreement on the same 953 pairs and the same stored pass-1
labels, with no relabelling, for every variant and β. Results are in
`falsejoin-results.md` and `falsejoin-results.json`. The Dasgupta variant
reproduces the table above (κ = 0.27, AUC 0.65).

| variant | κ [95% CI] | Δκ against Dasgupta | AUC | ΔAUC |
|---|---|---|---|---|
| Dasgupta alone | 0.27 [0.22, 0.32] | – | 0.65 | – |
| C + 1 · FJ (outside mass) | 0.47 [0.41, 0.52] | +0.20 [+0.14, +0.26] | 0.78 | +0.13 [+0.09, +0.17] |
| C + 1 · FJs (specificity) | 0.48 [0.43, 0.53] | +0.21 [+0.15, +0.27] | 0.78 | +0.14 [+0.09, +0.18] |

Where the gain comes from:

- **The placed general schemes.** κ is 0.54–0.68 per run, against
  0.24–0.29.
- **`msc-gold`.** AUC rises from 0.60 to 0.85.
- **`arxiv-gold`.** κ falls from 0.24 to 0.11, because the judge accepts
  most of its deep joins of embedding-dissimilar papers.
- **`learned-placed`.** No change.

Within stratum B, the term flags pairs that C could not flag. It still ranks
them only weakly (AUC 0.55), because almost every B pair is a deep join.

Two non-LLM checks follow in `bench/results/2026-10-11-falsejoin/external/`:
bibliographic coupling and co-citation from OpenAlex for the papers, and
shared LCSH headings for the books. They do not confirm the gain at the
pair level (AUC −0.04, +0.06 and −0.02), and they leave the scheme-level
rank agreement unchanged (ρ ≈ 0.9 at β ≤ 1). Report 9, §4.5 and §4.6,
discusses both checks.

## Reproduce

After `bench/results/2026-10-10-scores/REPRODUCE.md` steps 1–3 (graphs,
schemes, placements, learned `own`/`placed`, vectors):

```sh
S=crates/bench/scripts/validity_check.py
python3 $S sample --check             # pair ids and metric fields reproduce; trees match the harness
python3 $S blind --out /tmp/judge     # blinded batches + private key under bench/corpus/cache/validity/
# label each /tmp/judge/pass*-batch*.jsonl per rubric.md into a labels directory (same file names)
python3 $S merge --labels <labels-dir>
python3 $S report                     # writes results.md and results.json
```

`report` is deterministic given `sample.jsonl`: the bootstrap is seeded.
`sample` without `--check` rewrites `sample.jsonl` and keeps the labels of
any pair id it reproduces.
