# Paper corpus, version 1

This release contains a deterministic sample of **10,000 arXiv papers**, a
separate holdout of **500 Open Library editions** with librarian-supplied LC
classifications, and a holdout of **3,000 Open Library editions** with both
LC and Dewey numbers. The arXiv rows are metadata only; no full text is included.

## Data and labels

`corpus.jsonl` contains one arXiv paper per line. Each row includes the arXiv
identifier, title, abstract, authors, primary and cross-listed categories,
MSC classes, ACM CCS classes, JEL codes when provided, source URL, and CC0
metadata license. arXiv applies [CC0 1.0 to metadata](https://info.arxiv.org/help/license/index.html).
The first category is the arXiv primary category;
remaining categories are cross-lists. MSC, ACM CCS, and JEL values are the
source-provided labels. MSC values are the tokens of arXiv's `msc-class` text
shaped like MSC 2020 codes (`05C10`, `05Cxx`, `05-XX`, `05-01`), each once and
in source order. Markers such as `(Primary)` and `Secondary:`, years, and words
are dropped. Letter case is normalized (`91g70` becomes `91G70`). JEL codes
are also extracted from comments when a JEL marker is present.

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

## Dewey numbers and the 3,000-book holdout

`holdout-ddc.jsonl` has one row per `holdout.jsonl` edition: its id, the
edition record's `dewey_decimal_class` values as Open Library gives them
(empty when there are none), and the record revision read. 134 of the 500
editions carry a value and 110 of those hold a Dewey number. `holdout.jsonl`
itself is unchanged, so the published graphs still apply.

`holdout-books.jsonl` is a second book holdout of 3,000 Open Library
editions. Each has an LC call number (`lc_classifications`), a Dewey number
(`dewey_decimal_class`) and subjects; 320 also have a description
(`ol_description`). `abstract_text`, which the graph and the classifier
read, is the description followed by `Subjects: ` and up to 30 subjects. No
edition shares a work with `holdout.jsonl`. `holdout-books-ids.txt` pins the
ids and `holdout-books.sampling.json` records the per-class counts. Every
row holds only Open Library record fields and the derived text.

The 21 LCC main classes get equal quotas: 143 each for the 18 classes A to
T, and 142 each for U, V and Z. Within a class, candidate works come from 20
evenly spaced pages of the Search API ordered by `lcc_sort`. They are ranked
by seeded SHA-256, and each work contributes at most one edition: the
lowest-ranked one whose first LC call number is in the class and that has a
Dewey number. `bench/results/2026-10-10-ddc/REPRODUCE.md` has the details.
Rebuild, from the response cache when present, with:

```sh
python3 bench/corpus/builder/harvest_openlibrary_ddc.py all
```

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

To apply the MSC filter to an existing corpus, use `--relabel-msc`. It
filters the stored `msc` values of each row and keeps every other field.
Commas, semicolons and whitespace never occur inside a code, so filtering the
stored tokens gives the same codes as filtering the source text:

```sh
cargo run --manifest-path bench/corpus/builder/Cargo.toml -- --relabel-msc \
  bench/corpus/corpus.jsonl corpus.new.jsonl
```

The Open Library harvester writes the holdout after reaching 500 qualifying
editions. Unit tests use synthetic XML records to check parsing, MSC and ACM
labels, the MSC code filter, the ACM and MSC relabels, deterministic sampling,
and archive floors:

```sh
cargo test --manifest-path bench/corpus/builder/Cargo.toml
```

## Pinned release

arXiv records: **10,000**. Open Library holdout records: **500**. Second
Open Library holdout (`holdout-books.jsonl`): **3,000**.

- `corpus.jsonl` SHA-256: `1027bf9d34928f470ed8b0d7b5ddc5c983fde49bc0cd33eaa162bbd1f830fea5`
- ID files: `sample-ids.txt`, `holdout-ids.txt`, `holdout-books-ids.txt`
- `holdout.jsonl` SHA-256: `3e057eb4a257488ca1668a1ed370306e74b5b21a79cf7b4fdbd2f5b21873c833`
- `holdout-ddc.jsonl` SHA-256: `093d21fd3d73ab55f06bc8ef7f97a124c0385a082c9746bfc69b3744f4e96808`
- `holdout-books.jsonl` SHA-256: `38e5463bf5907de4c5a5413bf812efba433f955e168afbb8853b91028e6c905d`
- `holdout-books-ids.txt` SHA-256: `de8018945a15531c89eb9027d39e435b44f7f4ab611ed92822b2f3fe86b0c439`
- MSC classes: 4,298 codes on 1,325 papers, filtered on 2026-10-10 with
  `--relabel-msc`. The previous pin (`80c4adb1…`) carried 4,885 `msc` values
  on 1,348 papers. 581 of those values held no code, such as `(Primary)` and
  `Secondary:`; others had a code glued to punctuation (`15A23.`) or written
  in lowercase. 316 rows changed. A row-by-row comparison found
  no changed byte outside the two `msc` arrays of each row (`msc` and
  `source.label_sources.msc`).
- ACM classes: 690 codes on 317 papers, re-harvested from `arXivOld` on
  2026-10-10 for the pinned IDs; no other field changed. The pin before that
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
1,325; ACM CCS 317; JEL 2.
