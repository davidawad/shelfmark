# Benchmark results

One directory per run; each has `results.md`, `results.json` and a
`REPRODUCE.md` with the exact commands.

| directory | what it holds | status |
| --- | --- | --- |
| `2026-10-10/` | Paper 1 benchmark: 13 schemes, gold and shelfmark-placed, 10,000 arXiv papers and 500 Open Library books, all-MiniLM-L6-v2 k = 20 | published |
| `2026-10-10-r2/`, `2026-10-10-r3/` | reruns with corrected placements and gold-only rescoring | published |
| `2026-10-10-learned/` | the learned scheme against every Paper 1 scheme | published |
| `2026-10-10-scores/` | scoring catalogue (report 6) for 28 runs, single-stream bootstrap | published |
| `2026-10-10-covered/` | covered-subset scores, coverage analysis, dense-vs-sparse cost, paired split-stream bootstrap | published |
| `2026-10-10-embed/` | cross-embedding robustness: MiniLM, bge-small, nomic at k = 10, 20, 50, item bootstrap | published |
| `2026-10-10-ddc/` | gold Dewey numbers against LCC, FDC and the placed schemes; 3,000-book DDC holdout (report 7) | published |
| `2026-10-10-proposal/` | depth cuts, availability pipelines, work-level lookups, papers under a book backbone and paired bootstraps on all three models for report 9 (knowledge-base structure proposal) | published |
| `2026-10-10-semantic/` | the placed runs once per classifier profile (lexical, shortlist, rerank) | **pending**: the run was not completed and published no results, so it is not merged |

`crates/bench/scripts/check_published.py` rescores a sample of the
`embed`, `ddc`, `scores` and `covered` runs from committed inputs and fails
on any difference.
