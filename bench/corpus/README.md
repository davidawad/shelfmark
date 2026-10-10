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

ACM classes come from the `arXivOld` OAI format. The `arXivRaw` and `arXiv`
formats served by `oaipmh.arxiv.org` repeat the `msc-class` value inside
`acm-class`; for example, 1310.2306 has ACM classes `I.2.8; I.6` on its abstract
page and in `arXivOld`, but its MSC codes in `arXivRaw`. The builder drops an
`arXivRaw` `acm-class` that mirrors `msc-class` and uses the `arXivOld` ACM
classes whenever an `arXivOld` record exists for the ID.

`holdout.jsonl` contains 500 separate Open Library edition records. Each row
has a nonempty `lc_classifications` array copied from the edition record.
`sample-ids.txt` and `holdout-ids.txt` pin both datasets by ID. Open Library
asks that community contributions be shared under [CC0 1.0](https://openlibrary.org/help/faq/using)
and states its [catalog licensing terms](https://openlibrary.org/developers/licensing).

## Harvest and sampling

The arXiv harvest uses the keyless OAI-PMH endpoint
`https://oaipmh.arxiv.org/oai`, `metadataPrefix=arXivRaw` (plus `arXivOld`
for ACM classes), and the inclusive date window 2026-01-01 through 2026-10-09
for the top-level sets `cs`, `math`, `physics`, `q-bio`, `q-fin`, `stat`,
`econ`, and `eess`. Resumption tokens are followed on one persistent
connection with at least 3.15 seconds between requests. Open Library is queried through its Search and works-editions APIs
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

The OAI harvester resumes from its checkpoint if interrupted. To refresh only
the ACM classes of the pinned sample, harvest `arXivOld` records for the pinned
IDs (ListRecords over the same window, then GetRecord for IDs whose datestamp
moved out of it), then rewrite `acm_ccs`; every other field is kept:

```sh
python3 bench/corpus/builder/harvest_arxiv.py --pinned bench/corpus/sample-ids.txt
cargo run --manifest-path bench/corpus/builder/Cargo.toml -- --relabel-acm \
  bench/corpus/corpus.jsonl bench/corpus/cache/arXivOld-pinned.xml corpus.new.jsonl
```

The Open Library harvester writes the holdout after reaching 500 qualifying
editions. Unit tests use synthetic XML records to check parsing, MSC and ACM
labels, the ACM relabel, deterministic sampling, and archive floors:

```sh
cargo test --manifest-path bench/corpus/builder/Cargo.toml
```

## Pinned release

arXiv records: **10,000**. Open Library holdout records: **500**.

- `corpus.jsonl` SHA-256: `80c4adb1bff89f997bbf75245a77e41c1c9a3eb68a1f89ba500efb7aa0bd5664`
- ID files: `sample-ids.txt`, `holdout-ids.txt`
- ACM classes: 690 codes on 317 papers, re-harvested from `arXivOld` on
  2026-10-10 for the pinned IDs; no other field changed. The previous pin
  (`7685685d…`) carried 433 MSC codes on 132 papers in `acm_ccs`.

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
