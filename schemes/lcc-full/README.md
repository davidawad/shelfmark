# LCC, complete schedules (`lcc-full`)

`lcc-full` builds the complete [Library of Congress Classification](https://www.loc.gov/aba/publications/FreeLCC/freelcc.html)
schedules as shelfmark scheme `scheme:lcc-full@2024`. The bundled `lcc`
scheme (`data/schemes/lcc/`) holds the outline: 7,392 main classes,
subclasses and numbered spans. This one adds every numbered entry of the
44 FreeLCC 2024 **Text** PDFs, so a call number such as `QA76.73.R3`
(R, the programming language) is a class with its own caption and place in the
hierarchy.

| | |
|---|---|
| classes | 416,186 (the outline's 7,392 plus 408,794 from the schedules) |
| depth | 13 levels (main class = 1) |
| branching | max 7,994 children, mean 17.06 over classes with children |
| source | 44 Text PDFs, 18,131 pages, 419,538 numbered lines ([`COVERAGE.md`](COVERAGE.md)) |
| data file | [`lcc-full.json.gz`](lcc-full.json.gz), 7.5 MB, 70 MB uncompressed |

## License: US government work, data committed

Checked 2026-10-09. The schedules are published by the Library of Congress
(Policy, Training, and Cooperative Programs Division) and are works of the
US government. "Copyright protection under this title is not available for any
work of the United States Government" ([17 U.S.C. 105](https://www.law.cornell.edu/uscode/text/17/105)).
The FreeLCC page ([freelcc.html](https://www.loc.gov/aba/publications/FreeLCC/freelcc.html))
offers the PDFs for download and states no further terms; the PDFs carry no
copyright notice. This is the same basis as the bundled outline scheme, so
the built file is committed. Credit: Library of Congress Classification,
Library of Congress, Policy, Training, and Cooperative Programs Division. The
terms are recorded in the scheme's `source.license`.

## Machine-readable sources checked first

No complete open extraction exists, so the builder parses the PDFs.

- id.loc.gov has classification records fetched one at a time (the outline
  builder `tools/lcc-outline` uses a few); its
  [bulk download page](https://id.loc.gov/download/) lists no LCC schedule
  dump (only experimental embeddings per class letter, checked 2026-10-09).
- The Library of Congress MDSConnect MARC Classification file (a 2016
  snapshot, about 649,000 records) is a free machine-readable alternative.
  It was not used as the base: it predates the 2024 schedules by eight
  years, and the PDF parser was already working. It is a candidate
  cross-check or fallback source (MARC 153 number and caption fields) and is
  not verified or used by this builder.
- FreeLCC offers PDFs only. The only other bulk form is the paid
  Classification Web.

## Build

```sh
cd schemes/lcc-full
cargo run --release -- fetch --cache /tmp/lcc-text            # downloads the 44 PDFs; checks them against the lock
cargo run --release -- build --cache /tmp/lcc-text            # offline; writes lcc-full.json(.gz), COVERAGE.md, coverage-failures.txt
cargo run --release -- check --cache /tmp/lcc-text            # fails if any of them is stale
shelfmark scheme compile schemes/lcc-full/lcc-full.json       # from the repo root; preview, then --commit
```

`build` refuses a PDF whose sha256 differs from `sources.lock.json` and an
outline file that differs from the lock's `outline_sha256`, so the same lock
always builds the same bytes. A parse takes about 7 seconds (files parse in
parallel). `cargo test` checks the parser on synthetic pages and the
committed file for validity, compilation, and that it keeps every outline
code (so ids agree with `lcc@2024`).

## How the parser reads a Text PDF

The PDFs are untagged but regular: every run of text is its own block with an
absolute position (`pdf.rs`, `lopdf`).

- **Header** (top of page): subclass letters (`QA`), title, letters again.
  The letters prefix every number on the page. A header that is not plain
  letters (`Q5`, `KF10A`, `P-PZ1`) marks a table page; `INDEX` marks the
  index.
- **Body lines:** number at the left margin, caption at the right with its
  indent as leading no-break spaces (three per level), scope notes in 9 pt.
- **Continued pages** restate the ancestors of their first entry, ending in
  `-- Continued`; those lines are dropped.
- **Numbers** go through the `lcc-outline` number reader (`code.rs`, copied):
  `1.A3-Z`, `75.5-76.95`, `80.A-Z` become `Q1.A3-1.Z`, `QA75.5-76.95`,
  `Q80.A-80.Z`, the same code form the outline uses. Before that, a number
  wrapped over two lines is joined, `<75>` (an optional number) loses its
  brackets, an end such as `8.A-Z` becomes `.Z`, and an end whose letters
  truncate the subclass (`187.A-S187.Z` under SB) gets the subclass.
- **Hierarchy** is the indentation: the parent is the nearest numbered line
  above with a smaller indent whose range holds the entry. Unnumbered
  lines are headings; a class needs a code, so they become the class's
  `note` (`Under: Societies > America`), followed by its scope notes.
- **Merge** (`build.rs`): an entry whose code the outline has keeps the
  outline class, label, parent and vocabulary (`lcc@2024` carries LCSH links).
  Other entries become new classes. A top-level entry goes under the deepest
  outline span that holds it, else its subclass. Subclasses the outline lacks
  (`KIA`, `KVJ`, ...) are created from the page header. Finally shelfmark's
  own validator runs; a parent that fails its range check is moved up to the
  nearest ancestor that holds the class (listed in `COVERAGE.md`).
- **Order** is shelf order: depth first, siblings by range start, wider span
  first.

## What is not in it (exactly)

Per-schedule counts are in [`COVERAGE.md`](COVERAGE.md); every unreadable
line is in [`coverage-failures.txt`](coverage-failures.txt) with its page.

- **Tables are skipped:** 263,486 lines on 929 table pages, including both
  table-only PDFs (`K_TABLES`, `P_TABLES`) and every in-schedule table (`P-PZ1`
  to `P-PZ40`, the `KF1`..., `KJ-KKZ1`... law tables, cutter and geographic
  tables). Their numbers are relative to the class that applies them
  (`.A2` under a table, not under a call number), and the scheme grammar has
  no place for a number-less template. Entries that point to a table keep the
  reference in their label (`(Table KF7)`); the table is not expanded under
  them. This is the biggest gap, and it is largest for the K law schedules,
  where tables carry most of the structure (`KDZ_KG-KH` has 671 classes
  from 332 pages; the rest is tables).
- **Indexes are skipped** (K, KIM-KIR, KVJ, PJ-PK, ...): they point to
  numbers, they do not define classes.
- **Numbers no longer used are skipped:** 8,511 lines printed in parentheses or
  square brackets (J has 2,458: its parenthesized Congress/Parliament series).
- **178 numbered lines fail to parse** and are not classes:
  - suffix cutters the grammar cannot spell: ordinals and words glued to a
    number (`546.55th`, `3862.A3date`, `20.A73b`, `4558 1st`);
  - an A-Z sub-arrangement of a cutter (`128.K83732A-` / `.K83732Z`,
    `591.A45W47-`);
  - a decimal-extended cutter span (`195.3.A3-195.Z`);
  - the schedules' own typos (reversed ranges such as `6221-6220`);
  - captionless lines (a number with nothing beside it).
  KF has the most (48), then PQ (24) and D-DR (14); 12 of the 44 files have none.
- **Duplicate codes keep the first:** 100 repeats inside a schedule (G has 34)
  and 1,069 in `KVJ`, whose entries `KL-KWX` already has.
- **Wrapped captions** are not specially detected: a caption's second line,
  if the PDF has one, reads as a heading, so it can show up in the `note`
  context of the entries below it, never as a class.
- **Hierarchy is the printed indentation.** Where the indentation parent does
  not hold the range, the class is moved up (56 classes, listed in
  `COVERAGE.md`; mostly outline subclasses narrower than the schedule such as
  `KPH`, plus typos like `BQ5975.A-BZ5975.Z`).
- 6,414 outline classes have no matching Text entry (the outline summarizes
  runs of numbers the Text prints one by one); they are kept as is.
- Labels are the printed captions, not expanded with their ancestors; `note`
  carries the ancestor headings.

## Updating

LC republishes the PDFs yearly. Run `fetch --relock` (rewrites
`sources.lock.json`), then `build`, review the `COVERAGE.md` diff, and bump
the scheme `version` (it is part of every class ID) if the outline's
`version` changed. The scheme version follows the outline scheme it layers on.
