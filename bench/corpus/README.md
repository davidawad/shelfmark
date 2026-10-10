# Paper corpus, version 1

This release contains a deterministic sample of **10,000 arXiv papers** and a
separate holdout of **500 Open Library editions** with librarian-supplied LC
classifications. The arXiv rows are metadata only; no full text is included.

## Data and labels

`corpus.jsonl` contains one arXiv paper per line. Each row includes the arXiv
identifier, title, abstract, authors, primary and cross-listed categories,
MSC classes, ACM CCS classes, JEL codes when provided, source URL, and CC0
metadata license. arXiv applies [CC0 1.0 to metadata](https://info.arxiv.org/help/license/index.html).
The first category is the arXiv primary category;
remaining categories are cross-lists. MSC, ACM CCS, and JEL values are the
source-provided labels. JEL codes are also extracted from comments when a JEL
marker is present.

`holdout.jsonl` contains 500 separate Open Library edition records. Each row
has a nonempty `lc_classifications` array copied from the edition record.
`sample-ids.txt` and `holdout-ids.txt` pin both datasets by ID. Open Library
asks that community contributions be shared under [CC0 1.0](https://openlibrary.org/help/faq/using)
and states its [catalog licensing terms](https://openlibrary.org/developers/licensing).

## Harvest and sampling

The arXiv harvest uses the keyless OAI-PMH endpoint
`https://oaipmh.arxiv.org/oai`, `metadataPrefix=arXivRaw`, and the inclusive
date window 2026-01-01 through 2026-10-09 for the top-level sets `cs`, `math`,
`physics`, `q-bio`, `q-fin`, `stat`, `econ`, and `eess`. Resumption tokens are
followed on one persistent connection with at least 3.15 seconds between
requests. Open Library is queried through its Search and works-editions APIs
with at least 1.05 seconds between requests. The harvest scripts checkpoint
raw pages and progress under the ignored `cache/` directory; raw harvest
responses are not part of this release.

The Rust builder deduplicates arXiv IDs and chooses records using SHA-256
ranking with seed `arxiv-corpus-v1-2026`. Sampling is proportional to harvested
top-level archive populations, with a minimum of 100 papers per archive.
Selected records and ID lists are sorted by ID for stable output.

## Rebuild

From the repository root:

```sh
export PATH=/usr/local/cargo/bin:$PATH
python3 bench/corpus/builder/harvest_arxiv.py
cargo run --manifest-path bench/corpus/builder/Cargo.toml -- \
  bench/corpus/cache/arxiv-all.xml 10000 bench/corpus/corpus.jsonl
python3 bench/corpus/builder/harvest_openlibrary.py
```

The OAI harvester resumes from its checkpoint if interrupted. The Open Library
harvester writes the holdout after reaching 500 qualifying editions. Unit tests
use a synthetic XML fixture to check parsing, labels, deterministic sampling,
and archive floors:

```sh
cargo test --manifest-path bench/corpus/builder/Cargo.toml
```

## Pinned release

arXiv records: **10,000**. Open Library holdout records: **500**.

- `corpus.jsonl` SHA-256: `7685685d0de25d6e8a614859783dcb5376d81e4fe336a65f22b550f4c7527b8d`
- ID files: `sample-ids.txt`, `holdout-ids.txt`

Sample counts by top-level archive:

| Archive | Papers |
| --- | ---: |
| cs | 3,968 |
| econ | 185 |
| eess | 519 |
| math | 2,042 |
| physics | 2,630 |
| q-bio | 172 |
| q-fin | 139 |
| stat | 345 |

Gold-label coverage (records with at least one source-provided label): MSC
1,348; ACM CCS 132; JEL 2.
