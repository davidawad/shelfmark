# Paper 2: a learned, order-preserving open general classification

**Status: DRAFT. Not submitted.** Nothing in this directory has been
submitted, posted, preprinted or shared outside this repository.

It is a proposal paper. It defines an objective for a general
classification, describes the fitting algorithm and the stable-identifier
insertion procedure, and reports a first benchmark of the prototype
`schemes/learned` against the Paper 1 schemes. The limitations are stated
in the paper.

## Candidate venues

None chosen. Candidates under consideration:

- JASIST (Journal of the Association for Information Science and Technology)
- Knowledge Organization
- JCDL (ACM/IEEE Joint Conference on Digital Libraries)
- TPDL (International Conference on Theory and Practice of Digital Libraries)
- an arXiv preprint, later, and only with the owner's approval

## Files

| file | what |
| --- | --- |
| `paper.src.md` | the text, with `{{key}}` and `{{table:name}}` placeholders. Edit this one |
| `paper.md` | generated from `paper.src.md`. Do not edit by hand |
| `tables/` | generated tables (`*.md`) and every substituted value (`values.json`) |
| `scripts/make_tables.py` | renders `paper.md` and `tables/` from the committed JSON |
| `references.bib` | bibliography; every entry has a DOI, an arXiv id or a checked URL |

## Numbers come from JSON only

Every number in `paper.md` is substituted by `scripts/make_tables.py`. None
is typed by hand. The inputs are:

- `bench/results/2026-10-10-learned/results.json`, `insert-report.json`, `learned-holdout.json`
- `bench/results/2026-10-10-r2/results.json` (Paper 1 placed runs)
- `bench/results/2026-10-10/results.json` (Paper 1 gold runs)
- `schemes/learned/learned.json` and `schemes/learned/data/centroids.json.gz`
- `bench/corpus/corpus.jsonl` (primary categories only)

The script also cross-checks the inputs and stops on any mismatch. It
checks that J equals `cost_norm + lambda * nav_log_a1` for every row, that
the placed and gold rows equal the Paper 1 results, that the builder's
objective equals the harness's numbers for the committed scheme, and that
both insertion runs report `stable`.

From the repository root (Python 3, standard library only):

```sh
python3 paper/paper2/scripts/make_tables.py          # regenerate
python3 paper/paper2/scripts/make_tables.py --check  # fail if paper.md or tables/ are stale
```

Run `--check` before committing any change to the text or the results.

## References

`paper/paper1` is not on this branch, so its verified bibliography could
not be reused directly. `references.bib` was assembled from the sources
already checked in `research/reports/` and `research/benchmark/DESIGN.md`,
and each entry was checked again on 2026-10-10:

- arXiv entries against the arXiv API (title and authors)
- DOIs through doi.org metadata (title, venue, volume, pages)
- URLs by an HTTP 200 whose page title matches

Merge it with the Paper 1 bibliography when both papers share a branch.
Claims without a verifiable source are marked `TODO(cite)` in the text.

## Open items before any submission

- Author line (`AUTHOR TBD`) and repository URL (marked TODO in section 9).
- `TODO(cite)`: the history of LCC/DDC editorial development; menu
  depth/breadth studies.
- A second, independent similarity (another embedding model, citations
  or subject headings) to remove the embedding circularity in the held-out
  and holdout results.
- Bootstrap intervals and permutation nulls for the learned rows.
- A human evaluation of the hand-written and automatic labels.
- Venue choice and formatting. Citations are written as pandoc-style
  `[@key]` and are not yet rendered.
