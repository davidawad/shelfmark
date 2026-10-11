# Inputs and paired bootstraps for report 9, 2026-10-10-proposal

Derived runs for `research/reports/9-kb-structure-proposal.md`. Everything
here is computed from committed results (`../2026-10-10-ddc/`,
`../2026-10-10-embed/`, the committed scheme files and graphs) plus
Open Library work-level lookups. The report's tables are regenerated, and
checked for drift, by `research/reports/scripts/kb_proposal_tables.py`.

| path | what | written by |
| --- | --- | --- |
| `availability/{holdout,books50}-works.jsonl` | Dewey (`ddc`) and LC (`lcc`) numbers Open Library aggregates over all editions of each work (CC0) | `ol_work_numbers.py` |
| `placements/books/` | Dewey cut to 1, 2, 4 digits (`ddc-d*`); FDC from Dewey cut to tree depth 1-3 (`fdc-d*`) or rolled up to classes with at least m books (`fdc-m*`); FDC sections with learned-scheme paths nested (`fdc-learned*`); librarian numbers for a seeded NN% of books, else the lexical classifier (`{fdc,lcc}-pipe-NN`) | `kb_proposal_runs.py placements` |
| `placements/holdout/` | `fdc-work` (edition Dewey number, else the work's consensus number, mapped to FDC); `fdc-pipeline`, `lcc-pipeline` (gold, else classifier) | same |
| `placements/corpus/`, `schemes/{fdc,lcc}-arxiv.json` | arXiv papers shelved in FDC or LCC through the report's arXiv crosswalk, at the class or with the arXiv category nested | same |
| `schemes/fdc-learned*.json` | FDC with learned paths nested under its sections | same |
| `graphs/` | 3,000-book kNN graphs for bge-small and nomic at k = 10, 20, 50 and MiniLM at k = 10, 50 (MiniLM k = 20 is `../2026-10-10-ddc/graphs/books-k20.json.gz`) | `build_graph_fastembed.py`, copied by `kb_proposal_runs.py graphs` |
| `bootstrap/<universe>-<model>-k<k>.json`, `.draws.json.gz` | one `shelfmark-bench bootstrap` call per universe and graph: 1,000 paired item draws, seed 20261010, every draw's cost_norm, nav_log_a1, coverage and mean_depth | `kb_proposal_runs.py bootstrap` |
| `scores/` | scoring catalogue (report 6) of the main books runs, and of FDC and LCC on the 424 holdout books both place, with 1,000 vertex replicates | `kb_proposal_runs.py scores` |
| `crosswalk/lcc-fdc.json` | LCC class to modal FDC class at depth 1-3 over the 3,000 books, with support and share, for the placement rule of report 9 §6.3 | `kb_proposal_tables.py --write` |

Reproduce:

```sh
cargo build --release -p shelfmark-bench
python3 -I research/reports/scripts/ol_work_numbers.py
python3 -I research/reports/scripts/kb_proposal_runs.py all --harness target/release/shelfmark-bench
python3 -I research/reports/scripts/kb_proposal_tables.py
```

Universes: `books` (3,000 books, `bench/corpus/holdout-books.jsonl`),
`holdout` (500 books, title-only graph), `corpus` (10,000 arXiv papers).
