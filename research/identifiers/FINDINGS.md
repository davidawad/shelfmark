# Identifier tables: findings

Spike 3. Plan R7 (identifier tier), AE1.
Measured on a remote box (12 cores, 22 GiB RAM) on 2026-10-07. The dump
and full outputs stayed on the box. Committed artifacts are listed under
"Reproduce".

## Bottom line

- **Coverage is good for LCCN and OCLC numbers and fair for ISBNs.**
  - LCCNs: 90.5% resolve to a class.
  - OCLC numbers: 57.4% resolve.
  - ISBNs: 11.5 M of the 34.6 M distinct ISBNs in Open Library (33.3%)
    resolve to an LCC or DDC number. 31.0% resolve to LCC.
- **Most answers are unambiguous.**
  - ISBNs with LCC: 95.6% agree on the subclass letters and 92.0% agree on
    the class number.
  - ISBNs with DDC: 99.4% agree on the main class.
  - Most disagreement comes from one Open Library record that merges several
    libraries' call numbers, not from editions disagreeing.
- **The license is permissive enough for LCC.**
  - Open Library states its catalog is public domain or CC0, and the
    Internet Archive asserts no rights.
  - DDC numbers carry a modest OCLC risk.
  - Details are under "License".
- **Do not embed the table in the binary.** Ship it as a separately
  downloaded, versioned data pack. Size is the main reason, licensing the
  second; see "Shipping recommendation". The arXiv map and the domain rules
  (about 20 KB together) can be embedded.

## Input

| | |
|---|---|
| Dump | `ol_dump_editions_2026-09-30.txt.gz` (https://openlibrary.org/developers/dumps → archive.org item `ol_dump_2026-09-30`) |
| Size / MD5 | 12,617,475,043 bytes; `615bfae55bfc4581bded0e43909b3f42` (matches archive.org metadata) |
| Records | 56,827,250 edition records, 0 JSON parse errors |

## Build cost and determinism

- **Time:** 262 s wall clock for the full build: 156 s to decompress and
  parse, the rest to sort and write. The build uses 1 reader thread and 10
  parser threads, and the single gzip stream is the bottleneck.
- **Memory:** peak RSS is 14.1 GiB (14,760,096 kB VmHWM). Most of it is the
  set of every valid identifier, which is kept to compute coverage
  denominators, plus the key-to-value map. A builder that does not need
  coverage could drop the sets. Building needs a large machine, not a laptop.
- **Determinism:**
  - Two full-dump runs at `--jobs 10` and `--jobs 4` produced byte-identical
    `lookup.tsv`, `conflicts.tsv` and `stats.json`. That run used the build
    from before the subclass allowlist; the allowlist change touches only
    the pure normalizer, not the pipeline.
  - The test suite checks the committed sample against golden output at
    jobs 1, 3 and 8.

## Coverage

Editions (n = 56,827,250):

| | editions | share |
|---|---:|---:|
| any valid ISBN/LCCN/OCLC | 48,336,553 | 85.1% |
| a usable LCC number | 17,658,189 | 31.1% |
| a usable DDC number | 6,632,686 | 11.7% |
| an identifier **and** a class (feeds the table) | 17,410,500 | 30.6% |

Distinct identifiers (denominator = every valid identifier in the dump):

| key | distinct | → LCC | → DDC | → either | → both |
|---|---:|---:|---:|---:|---:|
| ISBN (10 and 13 merged as ISBN-13) | 34,588,597 | 10,726,746 (31.0%) | 4,633,781 (13.4%) | 11,522,245 (33.3%) | 3,838,282 |
| LCCN | 11,869,981 | 10,699,675 (90.1%) | 4,522,389 (38.1%) | 10,737,619 (90.5%) | 4,484,445 |
| OCLC | 20,284,978 | 11,234,147 (55.4%) | 4,158,781 (20.5%) | 11,652,271 (57.4%) | 3,740,657 |

Table totals:
- 33,912,135 keys and 57,303,516 lookup rows.
- 16,057,693 distinct full LC call numbers, which reduce to 371,929 distinct
  LCC class prefixes.
- 750,360 distinct DDC numbers.

Where the classes come from. An edition with several `source_records`
counts once per source type:
- `marc` (library MARC records): 15,587,392 editions, about 88% of the
  editions with any class. The denominator is at least the 17,658,189 with
  LCC.
- `bwb`: 4,897,833.
- `ia`: 3,600,278.
- `promise`: 2,300,694.
- `amazon`: 1,471,802.
- Everything else: under 150 k.

In practice the LC numbers come from library cataloguing. Trade-only
records (Amazon, Better World Books) seldom bring a class of their own.

LCC editions by main class:

| A | B | C | D | E | F | G | H | J | K | L |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 114 k | 1.60 M | 198 k | 1.80 M | 247 k | 422 k | 521 k | 2.39 M | 476 k | 918 k | 423 k |

| M | N | P | Q | R | S | T | U | V | Z |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 298 k | 813 k | 4.35 M | 1.17 M | 657 k | 246 k | 973 k | 114 k | 39 k | 274 k |

**What the ISBN rate means for a real corpus is unmeasured.** I planned to
measure the hit rate on the ISBNs, arXiv IDs and URLs in a real library corpus.
That failed: this box's snapshot is a partial clone without those blobs, and
fetching them needs the remote. The 33% is the share of Open Library's own
ISBNs. Library-held books, which are most of what gets cited, should resolve
far more often than self-published or trade-only ISBNs. **That is an
inference, not a measurement.** The R16 eval fixture should report the
identifier tier's real hit rate.

## Conflicts

A key is in `conflicts.tsv` when its class numbers disagree at the `prefix`
level (LCC class number / DDC section) or the `top` level (LCC subclass
letters / DDC main class).

ISBN keys:

| | keys | share of ISBNs with that scheme |
|---|---:|---:|
| LCC class number disagrees | 856,266 | 8.0% |
| LCC subclass disagrees | 472,026 | 4.4% |
| ... and only one edition carries the ISBN | 390,186 | 82.7% of the subclass conflicts |
| DDC section disagrees | 104,921 | 2.3% |
| DDC main class disagrees | 26,974 | 0.6% |

Findings, over the 1,714,574 LCC subclass-conflict rows for all key kinds:

1. **85.2% are inside one edition record.** Open Library merges MARC records
   from LC, Harvard, Columbia, Claremont and others into one edition's
   `lc_classifications`, and keeps no provenance per value. For example,
   `isbn:9780002721196` has `DA632.R33 1986` and `GV811` on the same record.
   Edition counts therefore cannot break most ties: only 28.0% of these
   conflicts have a strict majority.
2. **12.8% are LC's fiction alternates.** PS|PZ (98,989) and PR|PZ (84,346)
   are the top two pairs. LC used PZ3/PZ4 for fiction alongside the
   literature number.
   - Proposed tie-break for the identifier tier: prefer the non-PZ class
     when PZ1–PZ4 co-occur with a P literature class. Keep PZ5–PZ90
     (juvenile) as is.
3. **The rest are mostly legitimate neighbours:** N|ND, PN|PQ, PR|PS, PN|PR,
   HD|HF, HC|HD, QA|TK, DS|PL, and Z (subject bibliography) against the
   subject class.
   - Recommendation: when a key has a top-level conflict, the identifier
     tier should return the candidates and let vocabulary matching (LCSH)
     decide. That is the plan's flag-don't-block rule (R10).
   - At full confidence, the tier should decide only when the candidates
     agree at the class-number level, or differ only in a PZ alternate.
4. **Multi-volume sets inflate row counts, not conflicts.** 7,474 keys have
   more than 50 rows. For example, one record (`/books/OL43785976M`) carries
   310 OCLC numbers and 310 per-volume call numbers under `AC149`. The class
   prefix agrees, so these resolve cleanly. Only the `full` column is
   volume-specific.

## Validation and rejected values

Rejected values are counted and never guessed. `stats.json` keeps the 20
lexically smallest rejected values per field.

| field | valid | invalid | note |
|---|---:|---:|---|
| ISBN-10 | 19.43 M | 128,721 | bad check digit, ASINs, prices, free text |
| ISBN-13 | 30.24 M | 46,058 | includes recoverable `ISBN-13: 978…` free text (not recovered) |
| LCCN | 12.93 M | 42 | |
| OCLC | 21.17 M | 441 | |
| LCC | 21.59 M | 1,313,566 (5.7%) | see below |
| DDC | 6.85 M | 493,321 (6.7%) | `[E]`, `Fic`, `B`, `(…)` optional numbers, MARC subfield junk |

**The LCC subclass allowlist was a fix this measurement forced.** The first
full build accepted any 1–3 capital letters followed by a number, and that
let through 1,316 distinct "subclasses":
- LC's `LAW` placeholder: 160,164 rows.
- `MLC` shelf numbers.
- `CPB`, `ESL`, `LOT`, `GOV` and similar local marks.

The normalizer now checks the LC Classification Outline's subclass list. It
accepts class K by pattern (K plus a second letter in use, plus any third
letter), because the K jurisdiction tables run to hundreds of subclasses.
After the fix there are 587 distinct letter prefixes, and 122,372 more raw
values are rejected.

The K pattern still admits a few unassigned three-letter forms, such as
`KDO` and `KEL`, which appear once each. When the L1 LCC scheme lands
(sibling bead), validation should switch to that scheme's subclass list.

## Size

Measured on the full table (`scripts/measure-sizes.sh`):

| file | rows | raw | gzip -9 | xz -9 |
|---|---:|---:|---:|---:|
| `lookup.tsv` (everything) | 57,303,516 | 2.43 GB | 555 MB | 371 MB |
| `conflicts.tsv` | 5,017,604 | 215 MB | 45.3 MB | 28.7 MB |
| `decision.tsv` (best full number per key and scheme) | 45,975,519 | 1.55 GB | 422 MB | 302 MB |
| `decision-prefix.tsv` (class prefix only) | 45,975,519 | 1.19 GB | 257 MB | 171 MB |
| `decision-isbn-lcc.tsv` (ISBN → LCC only) | 10,726,746 | 418 MB | 100 MB | 73.3 MB |

The decision files are text. A compiled runtime form would be much smaller.
For example, ISBN → LCC prefix could be stored as:
- sorted ISBN-13 deltas (varint);
- a 3-byte index into the 371,929 prefixes;
- a conflict bit.

That is roughly 10.7 M × ~5 bytes ≈ 55 MB before compression. **This is an
estimate, not a measurement;** building the compiled form belongs to the
L1/L2 runtime work.

## License

Verified by fetching each page on 2026-10-07:

- https://openlibrary.org/developers/dumps says nothing about licensing. The
  only license text on the page is the site JavaScript's AGPL notice.
- https://openlibrary.org/developers/licensing: "The Internet Archive does
  not assert any new copyright or other proprietary rights over any of the
  material in the Open Library database. There may be existing rights
  issues on some contributions and in some jurisdictions."
- https://openlibrary.org/help/faq/using, under "Who owns the Open Library
  catalog?": "You do! Our catalog was created by the generous donations of
  libraries who dedicated their data into the public domain." Contributions
  are requested under CC0 1.0.
- archive.org item metadata (`/metadata/ol_dump_2026-09-30`) has no
  `licenseurl` or `rights` field.
- The Open Library code (internetarchive/openlibrary) is AGPL-3.0. That
  covers the software, not the catalog data.

Upstream caveats. These were researched by a sub-agent and are only partly
verified; they are not legal advice.

- **LC MARC records** are US federal government works (17 USC 105), so the
  LC-derived call numbers are public domain. No current loc.gov page
  states this explicitly.
- **OCLC / WorldCat:**
  - OCLC recommends ODC-BY attribution for WorldCat-derived data and asks
    for "reasonable efforts to attribute the OCLC cooperative".
  - Some Open Library MARC sources may derive from WorldCat, and the OCLC
    numbers are themselves OCLC identifiers.
  - This is a norm for OCLC members, not a contract binding downstream
    users.
- **DDC:**
  - OCLC's WebDewey license says records "may display DDC numbers, but
    shall not display DDC captions".
  - Bare numbers on records are therefore an anticipated use. DDC
    captions and schedules are OCLC copyright, and must never be bundled.
  - "Dewey" and "DDC" are OCLC trademarks.

**Can the table ship in an open-source binary?**
- **LCC (from ISBN, LCCN or OCLC): yes**, with a NOTICE. Confidence:
  medium-high.
- **DDC: numbers only, never captions.** Confidence: medium.
- Keeping DDC as its own data file lets it be withdrawn without a code
  release.

Suggested NOTICE text:

> Classification data derived from the Open Library data dump
> (ol_dump_2026-09-30, https://openlibrary.org/developers/dumps), Internet
> Archive. Open Library states its catalog is public domain and asserts no
> rights; upstream rights issues may exist on some contributions. Some
> records may derive from OCLC WorldCat; with thanks to the OCLC
> cooperative. Dewey Decimal Classification numbers only; DDC and Dewey are
> trademarks of OCLC, Inc.; no DDC captions are included.

License the data pack CC0 or ODC-BY, separately from the code license.
ODC-BY matches OCLC's request.

## Shipping recommendation

Ship the identifier table as a **versioned, separately downloaded data
pack**, not inside the binary.

- **Size:** the ISBN → LCC projection alone is 418 MB as text and 73 MB as
  xz. The prefix-only projection over all keys is 171 MB as xz, and the
  full table is 371 MB as xz. That would
  dominate a CPU-first CLI's download for every user, including users who
  never classify books.
- **Updates:** Open Library publishes a dump monthly. A data pack can
  refresh without a code release, and each placement already records the
  hashes of its inputs (R14). The pack's version, for example
  `ids:openlibrary@2026-09-30`, becomes one of those inputs, so a refresh
  names exactly which placements are stale.
- **Licensing:** a separate pack carries its own NOTICE and license, and
  the DDC column can be split out or withdrawn on its own.
- **What to embed in the binary:** the arXiv map and the URL-domain rules.
  They are tiny, hand-curated and reviewable as code.

## arXiv and URL-domain rules

- **`data/arxiv-lcc.tsv`:**
  - All 155 categories in the 2026-10-07 taxonomy have a class, and a test
    enforces this.
  - It also has the 6 arXiv aliases (`cs.NA`→`math.NA`, `cs.SY`→`eess.SY`,
    `math.IT`→`cs.IT`, `math.MP`→`math-ph`, `q-fin.EC`→`econ.GN`,
    `stat.TH`→`math.ST`), 11 bare-archive fallbacks, and 18 discontinued
    legacy archives (`solv-int`, `alg-geom`, `cmp-lg`, ...).
  - Of the 149 categories mapped directly (the other 6 are aliases), 84
    are marked `exact` and 65 `broad`.
  - The map was hand-assigned from the LCC schedules and **needs librarian
    review.** The physics, quantitative-biology and quantitative-finance
    rows are the least certain.
- **New-style arXiv IDs (`2101.00001`) carry no category.**
  - The identifier tier needs the arXiv API or OAI-PMH metadata to get the
    primary category. That is a network call, or a cached arXiv metadata
    snapshot, which is CC0 via Kaggle and OAI.
  - Old-style IDs carry it (`hep-th/9901001`).
- **`data/url-domain-rules.tsv`:**
  - 54 starter rules: 10 `decide` (Rust and Python documentation and
    registries, ACL Anthology, JMLR, the Supreme Court, Oyez, arXiv
    delegation) and 44 `hint`.
  - Matching uses the host or any subdomain plus an optional path prefix,
    and the most specific rule wins.
  - Hints should only adjust priors. R18 expects new domain rules to be
    mined from corrections, and this file is the seed for that.

## Reproduce

```sh
cd research/identifiers
cargo test --release                      # 18 tests
cargo run --release -- check              # rule files
cargo run --release -- build /path/ol_dump_editions_2026-09-30.txt.gz --out out/
scripts/measure-sizes.sh out/
```

Committed artifacts, all small:
- `samples/editions-sample.txt` plus golden `samples/expected/`.
- `samples/full-dump-2026-09-30/stats.json` and `sizes.tsv`: the full-dump
  statistics and size measurements behind this report.
- Every 57,303rd row of the full `lookup.tsv` and every 5,017th row of the
  full `conflicts.tsv` (about 1,000 rows each).

## Follow-ups (not done here)

1. Identifier-tier decision policy: the PZ alternate tie-break, and deciding
   only on prefix agreement (see "Conflicts").
2. A compiled runtime form and a data-pack format, with `ids:` version
   hashing for R14.
3. Real-corpus hit rate in the R16 eval (a real library corpus ISBNs, arXiv
   IDs, URLs).
4. Switch LCC subclass validation to the L1 LCC scheme once it exists.
5. Fill DDC gaps from the Open Library works dump (`dewey_number` on works)
   and the LC MARC records directly. Not measured here.
6. Librarian review of `arxiv-lcc.tsv` and the domain rules.
