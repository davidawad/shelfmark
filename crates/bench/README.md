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
