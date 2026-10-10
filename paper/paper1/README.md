# Paper 1: Scoring universal classification schemes with hierarchical-clustering objectives

**Status: DRAFT, not submitted.** Nothing here has been submitted, posted or
shared. The author line is a placeholder (`AUTHOR TBD`).

Candidate venues: JASIST, Knowledge Organization, JCDL, TPDL.

## Files

- `paper.md`: the draft (Markdown with pandoc-style `[@key]` citations).
- `references.bib`: references. Each entry notes where its link was checked
  (research/reports/1-4, research/benchmark/DESIGN.md, or the official
  page). Author fields hold surnames only, as recorded in those reports.
  Before submission, complete them and any titles marked "check before
  submission" from the DOI or URL.
- `scripts/make_tables.py`: generates every table and every marked number
  in `paper.md` (standard-library Python 3).

Claims with no verified source are marked `TODO(cite)` in `paper.md`.

## Sources

- Gold results: `bench/results/2026-10-10/` (source labels), with the
  `msc` gold run and the `acm-ccs` skip taken from
  `bench/results/2026-10-10-r3/` (rescored on the fixed corpus).
- Placed results: `bench/results/2026-10-10-r2/` (shelfmark
  `profile:lexical@2`, after the stopword fix). Its `results.json` also
  holds the before/after comparison with `profile:lexical@1`, which the
  paper reports as a methods note.
- Corpus: `bench/corpus/corpus.jsonl`, `bench/corpus/holdout.jsonl`.
- Background: `research/reports/1-4`, `research/benchmark/DESIGN.md`,
  `bench/corpus/README.md`, `crates/bench/README.md`.

## Rebuilding the tables

From the repository root:

```sh
python3 paper/paper1/scripts/make_tables.py          # rewrite generated parts of paper.md
python3 paper/paper1/scripts/make_tables.py --check  # exit 1 if paper.md is stale
```

Generated regions are delimited in `paper.md` by
`<!-- table:NAME -->` ... `<!-- /table:NAME -->` and inline
`<!-- v:NAME -->value<!-- /v -->`. Do not type numbers into them by hand.
To add a number to the prose, add a value to `build()` in the script and a
marker in the text. The script stops with an error on an unknown marker.
It also asserts that the r2 and r3 runs reused the 2026-10-10 graphs and
scheme files and that the corpus files match their recorded SHA-256.

To render a PDF (not needed for the draft):

```sh
pandoc paper/paper1/paper.md --citeproc \
  --bibliography paper/paper1/references.bib -o paper1.pdf
```

To regenerate the underlying results, follow
`bench/results/2026-10-10/REPRODUCE.md`, then
`bench/results/2026-10-10-r2/REPRODUCE.md` and
`bench/results/2026-10-10-r3/REPRODUCE.md`.
