#import "../template.typ": *
#import "../generated.typ": v

= Reproducibility <sec:repro>

All inputs and outputs are in the project repository or pinned by hash.

- *Corpus.* `bench/corpus/corpus.jsonl` and `holdout.jsonl`, with ID lists
  and SHA-256 hashes. Metadata only, CC0 1.0. The original runs used a
  `corpus.jsonl` with SHA-256 prefix `7685685d`; the corrected label fields
  used for the MSC gold rescoring give prefix `1027bf9d`. Neither change
  touches row order, identifier, title or abstract, the only inputs to the
  graph.
- *Similarity graphs.* Published gzipped, with manifests, under
  `bench/results/2026-10-10/graphs/`. The manifests give the model revision
  `d13954661f83248295ba75c1ed411eef3b7b936e`, the `model.onnx` SHA-256, $tau$,
  $k$ and the graph SHA-256 (corpus graph prefix #raw(v.c_graph_corpus_sha)).
- *Harness.* `crates/bench`, at the commit recorded in each `results.json`
  (#raw(v.c_harness_commit)). The classifier for the placed run is at
  #raw(v.c_classifier_commit) (`profile:lexical@2`).
- *Schemes.* Bundled schemes are in the repository. Fetch-only schemes are
  rebuilt from the publishers' downloads, checked against committed
  `sources.lock.json` files, and must match the `file_sha256` recorded in
  `results.json`.
- *Commands.* Each result directory has a `REPRODUCE.md` listing every
  command. The first run took about 45 minutes and the rerun about 25
  minutes, both on 8 CPUs.
- *Later results.* `docs/validity/` (sample, rubric, labels and tables) and
  `bench/results/2026-10-10-covered`, `-embed`, `-semantic`, `-learned`,
  `2026-10-11-falsejoin`, `-stability` and `-book-stability`, each with a
  `REPRODUCE.md`. Rerunning the judge needs the same model and rubric.
- *Determinism.* With the same graph and scheme files, gold scores reproduce
  exactly. Placed scores reproduce exactly with the same classifier build.
  Rebuilding the graph from text on a different ONNX Runtime version can
  change near-tied neighbours, so the published graph file, not the
  embedding pipeline, is the reproducible artefact.
- *This paper.* `paper/method-survey/scripts/gen_tables.py` regenerates every
  table and every headline value (the dictionary `v` in `generated.typ`) from
  the result files, with `paper/common/extra_results.py` for the validity,
  stability, book and semantic-tier values; `--check` fails if
  `generated.typ` is stale. @tab:survey and the prose of @sec:survey are not
  generated: they are taken from the scheme READMEs and the survey and
  licensing reports.
