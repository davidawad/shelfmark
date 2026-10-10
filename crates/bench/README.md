# shelfmark-bench

Deterministic tree scoring for classification schemes. The crate exposes an
`EmbeddingModel` trait for run-specific models and `DeterministicEmbedder` for
tests and smoke runs. It builds a kNN cosine graph after subtracting the mean
pair cosine, then shares that graph across the scheme and average-linkage
reference trees.

Run one scheme from the workspace root:

```sh
cargo run -p shelfmark-bench -- run \
  --corpus corpus.jsonl --scheme scheme.json \
  --placements placements.jsonl --k 20 --output result.json
```

For a production run, pass a precomputed `SimilarityGraph` JSON with
`--graph graph.json`; the graph is reused unchanged for all schemes. The
first scoring invocation computes the average-linkage baseline. Later
invocations may pass its `cost_norm` as `--reference-cost` to reuse that
baseline. `crates/bench/scripts/build_graph_fastembed.py` builds the graph
with pinned CPU sentence embeddings; model revision and weight digest are
written to its manifest. Without `--graph`, the deterministic test embedder
is used.

Corpus files are JSONL with `id`, `title`, and `abstract` (or `description`);
the arXiv metadata fields `msc_classes`, `acm_ccs`, `jel_codes`, and the Open
Library `lc_classifications` field are also accepted. Without `--placements`,
the reader selects the gold field matching the scheme name (or its `gold`
mapping) and reports track `A`. A
placement file has one JSON object per item, such as
`{"id":"2601.00001","classes":[{"code":"68T01","mass":1.0}]}`;
plain code strings are accepted too. Supplying it reports track `P`.

The JSON result includes Dasgupta cost and Moseley–Wang revenue, normalized
scores, average-linkage reference cost and skill, placement coverage, item
depth and branching summaries, class-size balance, expected pair distance,
and uniform-use navigation costs for scan and log menu models. This command
uses the deterministic test embedder; production runs should call the library
with a pinned model implementation of `EmbeddingModel` and record that model's
revision alongside the result. Graph, induced tree, and scheme file SHA-256
digests are included in the JSON result.

## Bootstrap intervals

All resampling lives in `src/bootstrap/`, with one SplitMix64 generator and
one percentile-interval rule. Each command keeps its own draw order, so
every published interval reproduces bit for bit
(`scripts/check_published.py` rescores a sample of each kind):

| command | module | resampling | published in |
| --- | --- | --- | --- |
| `run --bootstrap N` | `bootstrap::weighted` | items as weights on the fixed graph and tree | `2026-10-10-embed` |
| `bootstrap` | `bootstrap::resample` | trees and graphs rebuilt from the drawn copies, several runs paired | `2026-10-10-ddc` |
| `scores --boot N` | `bootstrap::catalogue` | edge, vertex and item resamples of the catalogue scores | `2026-10-10-scores`, `2026-10-10-covered` |

`run --bootstrap N [--seed S] [--reference-replicates FILE]
[--replicates-out FILE]` adds a `bootstrap` object of 95% intervals to the
result. A work drawn c times counts c times in every subtree, edge
(`w_ij c_i c_j`), coverage, class size and navigation cost; the draws depend
only on n, the seed and the replicate, so every run on one graph is paired.
Skill uses the average-linkage cost on the same resample, computed in-run or
read from the `ref_cost_norm` series of `--reference-replicates`. `--track
A|P` overrides the track inferred from `--placements`.

`scores --boot-streams split` (the default) draws edge, item and kNN
resamples from separate streams and adds the vertex bootstrap;
`--boot-streams single` uses one stream for all three and no vertex
bootstrap, which is how `2026-10-10-scores` was drawn.

`bootstrap` scores several placement runs on one corpus and graph and adds
paired percentile intervals over items:

```sh
cargo run --release -p shelfmark-bench -- bootstrap \
  --corpus corpus.jsonl --graph graph.json --runs runs.json \
  --resamples 1000 --seed 20261010 --confidence 0.95 --output bootstrap.json
```

`runs.json` is a list of `{"key", "scheme", "placements"}` objects
(`placements` may be omitted to use the corpus gold field, as in `run`).
Each draw takes n items with replacement. An item drawn c times appears as
c copies under its original class, each graph edge (i, j) is kept once per
pair of copies, and classes left without items are pruned; that is the tree
the scheme would induce from the drawn placements. Every run, and the
average-linkage reference tree built once on the full graph, is rescored on
the same draws, so skill uses the reference cost of the same draw and
differences between runs are paired. The output has, per run, the point
value and interval of every scalar the `run` result reports (`cost_norm`,
`revenue_norm`, `skill`, `coverage`, `expected_tree_distance` and each
structural statistic), and paired differences of `cost_norm`, `nav_log_a1`
and `largest_class_share` for every pair of runs. The same inputs and seed
give the same intervals, whatever `--threads` is.

Structural statistics under this resampling are approximate: a drawn copy
adds to its class's size but not to the number of distinct works, so
intervals for class-size measures (`effective_classes`, `gini`, the
navigation costs) are wider than and can sit off-centre from the point
value.

## Full benchmark run

`scripts/build_schemes.sh` builds every benchmark scheme into the ignored
`bench/corpus/cache/schemes/`, `scripts/run_benchmark.py` prepares gold and
shelfmark-placed placements and scores them, and `scripts/publish_results.py`
writes a dated results directory. `bench/results/2026-10-10/REPRODUCE.md`
lists the exact commands. `build_schemes.sh` takes scheme names to build only
those schemes. `run_benchmark.py --gold-only` and `publish_results.py
--gold-only` rescore only the source labels, as in
`bench/results/2026-10-10-r3/`.

## DDC comparison

`bench/corpus/builder/harvest_openlibrary_ddc.py` fetches the Dewey
numbers of the book holdout and a 3,000-edition holdout with both LCC and
DDC. `scripts/ddc_numbers.py` builds a Dewey tree from the numbers
alone and maps numbers into FDC, `scripts/ddc_benchmark.py` places and
scores every general scheme on both holdouts with bootstrap intervals, and
`scripts/publish_ddc.py` writes `bench/results/2026-10-10-ddc/`, whose
`REPRODUCE.md` lists the commands.

## Learned scheme

`scripts/learned_benchmark.py` scores the fitted scheme from
`schemes/learned` against the Paper 1 runs (own placements, holdout
insertion, shelfmark placement and a held-out 80/20 split), and
`scripts/publish_learned.py` writes `bench/results/2026-10-10-learned/`.
That directory's `REPRODUCE.md` lists the commands.

## Scoring catalogue

`shelfmark-bench scores` computes every score of
`research/reports/6-scoring-catalogue.md` for one run: Dasgupta variants,
tree-sampling divergence, triplets, embedding and probe scores, agreement
with external gold labels, search cost against Huffman and Hu–Tucker bounds,
MDL, shelf order, insertion, notation, granularity and collocation. It adds
bootstrap intervals, null models and placement noise, all from a seeded
generator. `shelfmark-bench agree` computes rank agreement between scores
across runs. `scripts/score_catalogue.py run` and `publish` drive both over
every benchmark run; `bench/results/2026-10-10-scores/REPRODUCE.md` lists
the commands.
