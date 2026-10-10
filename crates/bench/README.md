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

## Full benchmark run

`scripts/build_schemes.sh` builds every benchmark scheme into the ignored
`bench/corpus/cache/schemes/`, `scripts/run_benchmark.py` prepares gold and
shelfmark-placed placements and scores them, and `scripts/publish_results.py`
writes a dated results directory. `bench/results/2026-10-10/REPRODUCE.md`
lists the exact commands. `build_schemes.sh` takes scheme names to build only
those schemes. `run_benchmark.py --gold-only` and `publish_results.py
--gold-only` rescore only the source labels, as in
`bench/results/2026-10-10-r3/`.

## Learned scheme

`scripts/learned_benchmark.py` scores the fitted scheme from
`schemes/learned` against the Paper 1 runs (own placements, holdout
insertion, shelfmark placement and a held-out 80/20 split), and
`scripts/publish_learned.py` writes `bench/results/2026-10-10-learned/`.
That directory's `REPRODUCE.md` lists the commands.
