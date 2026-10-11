# LCC and LCSH in shelfmark: what is here, where it came from, how to rebuild it

This is the single record of the Library of Congress Classification (LCC) and
Library of Congress Subject Headings (LCSH) data in this repository. It lists
every source file by URL, date, size and sha256, the license, the commands
that rebuild each scheme from those sources, and what is missing. The
per-builder notes it summarizes are `data/schemes/lcc/SOURCES.md` (generated),
`data/schemes/lcc/LCSH.md`, `schemes/lcc-full/README.md`,
`schemes/lcc-full/COVERAGE.md` and `research/vocab/README.md`.

Checked 2026-10-11. Every source below was fetched again that day on a box
and matched its lock, except the id.loc.gov records described under A4.

## The three files

| file | scheme id | what it is | classes | LCSH | built by |
|---|---|---|---:|---|---|
| `data/schemes/lcc/lcc.json` (975 KB) | `lcc@2024` | LCC outline: 21 classes, every subclass, numbered spans | 7,392 | 1,343 links (headings whose number is exactly a class) | `tools/lcc-outline` |
| `data/schemes/lcc/lcc-lcsh.json.gz` (1.3 MB) | `lcc@2024` | the outline plus LCSH vocabulary; **compiled into the binary** | 7,392 | 91,499 headings placed, 97,848 terms | `research/vocab` (`scripts/build-lcc-lcsh.sh`) |
| `schemes/lcc-full/lcc-full.json.gz` (7.5 MB, 70 MB raw) | `lcc-full@2024` | the complete 2024 schedules, every numbered entry | 416,186 | the outline's 1,343 links only | `schemes/lcc-full` |

## Is it complete?

### LCC

**Classes and subclasses: yes, all of them.** The official list is the LC
Classification Outline, <https://www.loc.gov/aba/cataloging/classification/lcco/>
(checked 2026-10-11; its 20 PDFs are the A1 sources below, byte-identical to
the lock). It names 21 main classes (A B C D E F G H J K L M N P Q R S T U V Z)
and 215 lettered subclasses, plus three K ranges given only as spans
(`KD-KDK`, `KJ-KKZ`, `KL-KWX`). `lcc-full` has all 21 classes, all 215
subclasses, and every subclass inside the three K ranges (5, 41 and 181 of
them). It also has 92 letter prefixes the outline does not list but the 2024 schedules do: the
US state law subclasses `KFA`-`KFZ`, Canadian provinces `KEA`-`KEZ`, the
`KG`/`KH` countries, `KI`-`KIR` (law of indigenous peoples), `KBS`, `KBT`,
`KZA`, `KZD`. One is spurious: `BG` ("Sonambulism", `BG1068-1073`)
comes from a misprint in the B-BJ schedule (somnambulism is `BF1068-1073`).

Classes per main class in `lcc-full`: A 1,644; B 36,190; C 4,958; D 36,513;
E 4,360; F 11,746; G 17,702; H 17,030; J 6,332; K 62,136; L 9,188; M 6,742;
N 14,255; P 105,880; Q 18,929; R 11,090; S 9,063; T 16,309; U 3,730; V 4,403;
Z 17,986.

**Numbered entries: every numbered caption the 44 schedule PDFs print, minus
the listed gaps.** The 2024 FreeLCC Text PDFs (B1 below, 18,131 pages,
419,538 numbered lines) are the complete public edition of the schedules
(<https://www.loc.gov/aba/publications/FreeLCC/freelcc.html>). `lcc-full`
keeps 410,749 parsed lines as 408,702 new classes on top of the outline.
What it does not hold (per schedule in `schemes/lcc-full/COVERAGE.md`, every
unread line in `schemes/lcc-full/coverage-failures.txt`):

| gap | lines | why |
|---|---:|---|
| tables (form divisions, cutter and geographic tables, `K_TABLES`, `P_TABLES`, `P-PZ1`...`P-PZ40`, the K law tables) | 263,486 on 929 pages | a table number is relative to the class that applies it; the scheme format has no number-less template. Largest gap, worst in K (`KDZ_KG-KH`: 671 classes from 332 pages) |
| numbers no longer used (printed in parentheses or brackets) | 8,511 | not valid for new classification (J has 2,458) |
| lines the number grammar cannot read | 178 | ordinals and dates glued to numbers (`546.55th`), cutter sub-arrangements, schedule typos (`6221-6220`) |
| duplicate codes (first kept) | 100 + 1,069 | 1,069 are `KVJ` entries that `KL-KWX` already has |
| schedule indexes | all index pages | they point to numbers; they define no classes |
| classes moved to a wider parent by the range check | 56 | listed in `COVERAGE.md` |

So `lcc-full` is complete at class, subclass and printed-caption level for the
2024 edition, and is not the complete LC Classification as LC maintains it:
the expanded tables, later changes (LC edits the schedules continuously in
Classification Web, a paid subscription) and the index are not in it.

**The bundled scheme is not the full schedules.** The binary carries the
7,392-class outline. See "Can the bundled scheme be the full LCC and full
LCSH?" below.

### LCSH

**The bundled vocabulary is a subset: 91,580 headings, 17.8% of the 513,470
active headings.**

- Full LCSH on id.loc.gov: the bulk dump issued 2026-10-06 (A3) holds 521,392
  authority records, 513,470 active and 7,922 deprecated
  (`research/vocab/stats/lcsh-stats.json`). LC's printed-edition count is
  lower because it counts verified heading and subdivision records only:
  "There were 390,024 authority records in the file as of April 2026"
  (LCSH 47 introduction, <https://www.loc.gov/aba/publications/FreeLCSH/LCSH-Main-intro.pdf>).
- Headings that carry an LCC number: 91,580, all active. Of those, 91,499
  land on an outline class (the other 81 carry only letter spans such as
  `D-F`), contributing 97,848 preferred labels to 3,647 classes. That is
  `lcc-lcsh.json.gz`.
- The other 421,890 active headings carry no LCC number and cannot be placed
  by number. Letting a heading borrow its nearest broader heading's number
  (`vocab-ingest enrich-scheme --inherit 3`) reaches 74,150 more; the rest
  (mostly `ComplexSubject` and geographic headings with no numbered broader
  heading) have no route to a class at all.
- The full table of all 521,392 headings (`lcsh.tsv.gz`, 11 MB) is built from
  the dump by `vocab-ingest lcsh` and kept out of git.

## License

US government work. "Copyright protection under this title is not available
for any work of the United States Government" (17 U.S.C. 105,
<https://www.law.cornell.edu/uscode/text/17/105>). id.loc.gov states that "The
Library of Congress has prepared this linked data system and is making it
available as a public domain data set" (<https://id.loc.gov/about/>). The
FreeLCC and LCCO pages offer the PDFs for download with no further terms and
the PDFs carry no copyright notice. Section 105 is a US rule: the House
Report note printed with it says "The prohibition on copyright protection for
United States Government works is not intended to have any effect on
protection of these works abroad" (same page), so outside the US the basis is
LC's public-domain statement. Credit, as LC asks: Library of
Congress Classification and Library of Congress Subject Headings, Library of
Congress, Policy, Training, and Cooperative Programs Division. Every scheme
file carries this in `source.license`, and `schemes/registry.json` marks
`lcc-full` `may_bundle: true`.

## Where the raw sources live

| source | in git? | why |
|---|---|---|
| LCCO and FreeLCC PDFs (A1, A2, B1; 103 files, 52.5 MB) | no, a cache directory | re-fetchable by URL; the locks pin sha256 and every one still matched on 2026-10-11 |
| LCSH dump (A3, 230 MB) | no, a cache directory | too large. LC replaces it in place (same URL) at each issue, so a later fetch fails the lock; relock then (see Rebuild) |
| id.loc.gov classification records (A4, 92 files, 1.2 MB) | **yes**, `data/schemes/lcc/sources/id-classification/` | id.loc.gov serves each record's JSON-LD with different bytes on each request (two fetches a second apart differ in sha256), so a lock of fetched records can never be met again. Small, so committed |
| `lcsh.tsv.gz` heading table (11 MB) | no | derived; `vocab-ingest lcsh` rebuilds it byte-identically from the dump |

LC publishes a SHA-1 fixity for the dump in its VoID description,
<https://id.loc.gov/download/authorities/subjects.madsrdf.nt.json>: for the
2026-10-06 issue (modified 2026-10-07) it is
`52c39fa376374c8fb654e3d1a44b6f7c4b3be806`.

## Rebuild

Heavy steps (the dump, the 44 Text PDFs, the release builds) run on a box,
not a laptop. One command re-fetches every source, checks all three files
against the locks, and writes a report:

```sh
REPRODUCE_ONLY=1 research/vocab/scripts/reproduce-lcc-lcsh.sh /tmp/lcc-report /tmp/lcc-cache
```

Per scheme:

```sh
# lcc.json and SOURCES.md (outline)
cd tools/lcc-outline
cargo run --release -- fetch --cache /tmp/lcc-sources   # network; rewrites the lock and the committed records
cargo run --release -- build --cache /tmp/lcc-sources   # offline, deterministic from the lock
cargo run --release -- check --cache /tmp/lcc-sources   # fails if lcc.json or SOURCES.md is stale

# lcc-lcsh.json.gz (bundled); reads the dump from the same cache
research/vocab/scripts/build-lcc-lcsh.sh /tmp/lcc-sources build   # or: check

# lcc-full.json.gz
cd schemes/lcc-full
git cat-file blob 70d8c8628088a7b30cd35655d7fa4ba7421c3003 > /tmp/lcc-outline-b21e4c.json
cargo run --release -- fetch --cache /tmp/lcc-text
cargo run --release -- check --cache /tmp/lcc-text --outline /tmp/lcc-outline-b21e4c.json
```

`lcc-full` layers on the outline file whose sha256 its lock records
(`b21e4c66...`). That is `lcc.json` as committed before the path in its
`source.description` changed; the classes are identical. Rebuilding against
today's `lcc.json` (`fetch --relock`, then `build`) gives the same 416,186
classes and differs only in the outline hash quoted in the description. It
is not done because published benchmark results pin the current file's
content hash.

Verified on a box on 2026-10-11 (`research/vocab/stats/bundle-candidates.json`
and the box report): `lcc-outline check` passes with the committed lock and
records; the rebuilt `lcc.json` is byte-identical; `build-lcc-lcsh.sh check`
reproduces `lcc-lcsh.json.gz` byte for byte (decompressed); `lcc-full` fetch
matches all 44 PDFs and its rebuild matches the committed classes.

## Can the bundled scheme be the full LCC and full LCSH?

Measured 2026-10-11 on a box (8 vCPU, 16 GiB), each candidate standing in for
the bundled `lcc@2024`; accuracy on the 104-item starter gold set
(`research/eval`), `profile:lexical@2`, top-1 with 95% bootstrap CI. Full
numbers: `research/vocab/stats/bundle-candidates.json`.

| candidate | binary | classify, no cache | warm | peak RSS (no cache) | class top-1 | subclass top-1 |
|---|---:|---:|---:|---:|---|---|
| bundled: outline + LCSH with numbers | 12.7 MB | 0.79 s | 0.20 s | 211 MiB | 46.2% [37.5, 55.8] | 34.6% [26.0, 44.2] |
| outline + LCSH, broader inheritance (3 levels) | 14.2 MB | 1.55 s | 0.34 s | 415 MiB | 51.0% [41.3, 59.6] | 38.5% [29.8, 48.1] |
| full schedules + LCSH with numbers | 20.1 MB | 8.24 s | 1.34 s | 1.18 GiB | 37.5% [27.9, 47.1] | 24.0% [16.3, 32.7] |
| full schedules + LCSH, inheritance | 21.8 MB | 9.66 s | 1.49 s | 1.50 GiB | 38.5% [28.8, 48.1] | 26.0% [17.3, 34.6] |

- **Full schedules as the default: no.** Ten times the start-up time and six
  times the memory, and it is less accurate at class and subclass level,
  because 416,186 narrow captions compete with the outline's broad ones. This
  agrees with the 3,000-book benchmark in `docs/decisions/default-scheme.md`,
  where `lcc-full-placed` scored skill 0.309 against 0.423 for the bundled
  scheme. `lcc-full` stays a scheme to compile into a store
  (`shelfmark scheme compile schemes/lcc-full/lcc-full.json`).
- **Full LCSH: not possible as such.** Only headings that carry an LCC number,
  or inherit one, have a class to sit on: 165,649 of 513,470 at most.
- **Outline + inherited LCSH** is better on this gold set at every
  level up to caption (+4.8 points class, +3.9 subclass, inside overlapping
  intervals on 104 items) for twice the memory. Changing the bundled file
  changes its content hash and marks every existing placement stale, and the
  published default-scheme results were run on the current file, so it is
  left as a measured candidate to take through the book benchmark, not
  swapped in here. Build it with
  `vocab-ingest enrich-scheme --scheme data/schemes/lcc/lcc.json --table lcsh.tsv.gz --inherit 3 --canonical true --out ...`.

## Other LC sources, not used

| source | what | why not used |
|---|---|---|
| Classification Web (<https://classweb.org/>) | LC's maintained schedules, tables and index | paid subscription |
| MDSConnect Classification MARC records (<https://www.loc.gov/item/2020445552/>) | free bulk MARC 21 classification records, 2014, 2016 and 2019 snapshots (2019 ZIP 85.5 MB) | older than the 2024 PDFs; the PDF parser already covers the schedules. The best candidate for filling the table gap and cross-checking captions (MARC field 153) |
| id.loc.gov "Experimental Embeddings Data by LCC" (<https://id.loc.gov/download/>) | vectors for catalog records, one file per class letter | records, not the schedules; no LCC schedule dump exists on id.loc.gov |
| LCSH SKOS dumps (`subjects.skosrdf.*`) | the same headings in SKOS | MADS carries the LCC numbers the build needs |

## Known gaps and defects

- Tables, indexes, obsolete numbers and 178 unreadable lines are not classes (above).
- `BG` is a spurious subclass from a schedule misprint.
- The bundled scheme's `source.description` still names `src/shelfmark/tools/lcc-outline`;
  it is part of the content hash, so it changes only with the next content change.
- The FAST vocabulary (OCLC, ODC-By) is not built: its bulk download is behind a
  Cloudflare challenge (`research/vocab/README.md`).
- LCSH is re-issued in place: `fetch` after a new issue rewrites the lock and the
  bundled file changes with it.

## Sources, file by file

Every URL, fetch date, size and sha256. `research/vocab/tests/sources_doc.rs`
fails if a locked file is missing here.

### A1. LC Classification Outline PDFs (class and subclass captions)

20 files, locked in `data/schemes/lcc/sources.lock.json`.

| file | url | fetched | bytes | sha256 | stored |
|---|---|---|---:|---|---|
| `lcco_a.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_a.pdf> | 2026-10-11 | 70527 | `f186ec440e1c5711cac8062557c0654b7c1a1c4d452038adef3e058f03caf6fd` | cache |
| `lcco_b.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_b.pdf> | 2026-10-11 | 421387 | `00b2f90f4bfaf1256b8f3048de917b00fec59b119b360ceda29a2d80d7e4f78a` | cache |
| `lcco_c.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_c.pdf> | 2026-10-11 | 73098 | `f2951cfaa1a0f690143133c5af9b4338c02e09b04bf533b4742b313b9dde3cd5` | cache |
| `lcco_d.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_d.pdf> | 2026-10-11 | 596276 | `b6d69e6350c291fcf9c91cc5b829b1980069e59e58a8c1682682313043336c7e` | cache |
| `lcco_ef.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_ef.pdf> | 2026-10-11 | 144402 | `bf5a7af6a78736f70855000b6bc2b30703eb99b0ab54472563d0a495bdc494f1` | cache |
| `lcco_g.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_g.pdf> | 2026-10-11 | 80937 | `70ea61c03c2a62cf77465ce77f093161a3b2507cd7c6491e126074d64d710c41` | cache |
| `lcco_h.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_h.pdf> | 2026-10-11 | 48722 | `89bff79a05d1b15085fee89c3062a2bbfc25b1c4cfd89a634829e3f1729a8e5f` | cache |
| `lcco_j.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_j.pdf> | 2026-10-11 | 122335 | `1fdd1697456f64ab512355044be64166c64e9e5854405b89b2baf3dbe81e4fdb` | cache |
| `lcco_k.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_k.pdf> | 2026-10-11 | 575630 | `ac4f8289c5b541e6fea6959b7df0be7c9ce6ab9b13b759b346fa3b3a197f7f9f` | cache |
| `lcco_l.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_l.pdf> | 2026-10-11 | 194012 | `632f8473b07bf7ca158e0751987966ef55befb76a8434c92af26dfa6d6774fdf` | cache |
| `lcco_m.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_m.pdf> | 2026-10-11 | 80806 | `6f80de523fd0a005596f6e75910a1b1fb546e3891b85f80aa41850907110344b` | cache |
| `lcco_n.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_n.pdf> | 2026-10-11 | 74223 | `daf5cb94c84251a1ee166641e4ad97e3e3f9edc372035c0933c84a01b340c099` | cache |
| `lcco_p.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_p.pdf> | 2026-10-11 | 432001 | `e66dae8ac8d71875a138aadc50c43dab774bfac6e573ece356bda2e0c9dfb90e` | cache |
| `lcco_q.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_q.pdf> | 2026-10-11 | 76361 | `34db61d912e62aa3220052e6bfb5ee64d253ee1629fdd6b24aedbf4a5c4d5fcd` | cache |
| `lcco_r.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_r.pdf> | 2026-10-11 | 84731 | `4b1f9a6fd362f494a717ccfc4589da9b20554043d38f84318a5034adc94215f8` | cache |
| `lcco_s.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_s.pdf> | 2026-10-11 | 77051 | `43afbac54d15c026cafd66ad035ea704a1596d232c496e1c1d253110ab3ebfc4` | cache |
| `lcco_t.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_t.pdf> | 2026-10-11 | 99643 | `7776fd8ba7fd213a44a4df28c5f207873c041d1ba89b26c82c29f6e2d9f69496` | cache |
| `lcco_u.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_u.pdf> | 2026-10-11 | 72468 | `bf0e7f09f1e65ffb546a1c1f126cab64fdaa19a8743b1ff99f0d3b16f42ec8d7` | cache |
| `lcco_v.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_v.pdf> | 2026-10-11 | 79705 | `da4b0aaac0544bc9881409a7a041ee10a00eb87cfd9991a2eedcab42e45b1844` | cache |
| `lcco_z.pdf` | <https://www.loc.gov/aba/cataloging/classification/lcco/lcco_z.pdf> | 2026-10-11 | 50822 | `236ffd5a66fd212bef09c6ba1a3269df0e709b3eb695556c6133f2619d4bdcc0` | cache |

### A2. FreeLCC 2024 outline PDFs (numbered spans)

39 files, locked in `data/schemes/lcc/sources.lock.json`.

| file | url | fetched | bytes | sha256 | stored |
|---|---|---|---:|---|---|
| `LCC_A2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_A2024OUT.pdf> | 2026-10-11 | 109555 | `babb002ce4d05bf023320cc45cf487e3c19d1e0371d637be85f334af5fd614d6` | cache |
| `LCC_B-BJ2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_B-BJ2024OUT.pdf> | 2026-10-11 | 136942 | `8e0879c6eb918e95df2e316bfa2ef52acdc448d7f3cd2c4cafbe25145d774c32` | cache |
| `LCC_BL-BQ2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_BL-BQ2024OUT.pdf> | 2026-10-11 | 241811 | `982b70e3da65d8b4719b687929488098b4a4d67e65d2146c9d52511ce32ce44b` | cache |
| `LCC_BR-BX2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_BR-BX2024OUT.pdf> | 2026-10-11 | 240822 | `7d21b5db8f12e032d0fb15659b809068cef756609a99afb0bc010e5f2105caed` | cache |
| `LCC_C2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_C2024OUT.pdf> | 2026-10-11 | 387212 | `c0ba5f9f9b3a05ac1d1317ac13d380d1e95c5dd069a25729b18cad6156fbde2f` | cache |
| `LCC_D-DR2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_D-DR2024OUT.pdf> | 2026-10-11 | 359211 | `b3be736ea70339abf8ab105fd3c9acdff3172ad4d9ca478e2bf795b5df59dfed` | cache |
| `LCC_DS-DX2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_DS-DX2024OUT.pdf> | 2026-10-11 | 220462 | `cdaea23855dee6852d90170dac20c3a1a5e1cfeb8927acec3d76481b59719e99` | cache |
| `LCC_E-F2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_E-F2024OUT.pdf> | 2026-10-11 | 108339 | `326858295e89a81b5d3c102beac4755153b907e5da343df831edc9759bd68ee7` | cache |
| `LCC_G2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_G2024OUT.pdf> | 2026-10-11 | 111411 | `67c04ce3f9429739b96b6e239e3d987133dfd43fec00b883d19092789aada448` | cache |
| `LCC_H2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_H2024OUT.pdf> | 2026-10-11 | 129275 | `23434a6456a69156ca15aa481d974df00ce2e776ab2d2b1691617a21ff4da32b` | cache |
| `LCC_J2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_J2024OUT.pdf> | 2026-10-11 | 107143 | `acf542a7ee9f6144224df888bfa16e5fd6c132802019a6967068fe24b75dbc8a` | cache |
| `LCC_K2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_K2024OUT.pdf> | 2026-10-11 | 102719 | `7e628ebc087e6b8bdcaa57c6c98e686f517ed384b47f86153a008b1109ec71a3` | cache |
| `LCC_KB2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KB2024OUT.pdf> | 2026-10-11 | 202711 | `705c53b6f9c94b0b2ff565a6a5b796c81a7020be149b5a3f11f050266b0ceab9` | cache |
| `LCC_KD2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KD2024OUT.pdf> | 2026-10-11 | 128155 | `4c696889526cf259ee74e8bf3834197c0f0b975f1761b51fb7bf80cb13fe9e98` | cache |
| `LCC_KDZ_KG-KH2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KDZ_KG-KH2024OUT.pdf> | 2026-10-11 | 150574 | `f6c786149bd97f53cd418d5a92025b5d71c22ac0f1d7c139929852f68de5550b` | cache |
| `LCC_KE2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KE2024OUT.pdf> | 2026-10-11 | 126903 | `c812c8678c31379726a487a9d6d59fe7b92dd89f9d8ec973f96e367ea4143776` | cache |
| `LCC_KF2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KF2024OUT.pdf> | 2026-10-11 | 111863 | `af920709d42936dd2851e04dd044943104167859493dc64ee2177cad068457bb` | cache |
| `LCC_KJ-KKZ2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KJ-KKZ2024OUT.pdf> | 2026-10-11 | 178012 | `95366dd27844e2c314bac9e59848daed8f78773dc7429caefca3e8c96079a881` | cache |
| `LCC_KJV-KJW2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KJV-KJW2024OUT.pdf> | 2026-10-11 | 117398 | `66931f47ee4f23ab5c21abb72bf7d5f0b5e02a44fb02db978b8b52d87d883dbb` | cache |
| `LCC_KK-KKC2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KK-KKC2024OUT.pdf> | 2026-10-11 | 115472 | `cfb7b577ac7ddbb4a14589bcd2f40569b0b300d668b35a110a85c6f8108cc3e3` | cache |
| `LCC_KL-KWX2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KL-KWX2024OUT.pdf> | 2026-10-11 | 211396 | `7b992aedf8d39fb37f8698581ded95e63f8751e49fa06efdf8a70b5c2ea3b56f` | cache |
| `LCC_KZ2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KZ2024OUT.pdf> | 2026-10-11 | 113916 | `bf3fba2864ec50b9125da5c9a2021b4d6f559c607fcb58605c16d7f68563624a` | cache |
| `LCC_L2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_L2024OUT.pdf> | 2026-10-11 | 177430 | `aa85bb4f314a9e90ebe5d30ccdbdf56417877d4f2cd0fd673d782fc79063f04a` | cache |
| `LCC_M2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_M2024OUT.pdf> | 2026-10-11 | 139862 | `0aee199b6156d39eff36c101f58852a15f493685959fa9b84cb69d8ef4b61abb` | cache |
| `LCC_N2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_N2024OUT.pdf> | 2026-10-11 | 128239 | `01a4621cd9dbf67e09827895e5e4c5685de6c3e54a01e23531710c4dc4f3aace` | cache |
| `LCC_P-PA2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_P-PA2024OUT.pdf> | 2026-10-11 | 102892 | `8e39e07a45bcdb40510079d1610db709502fb269b90aaab2aadf831e4ba9e5cd` | cache |
| `LCC_PB-PH2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PB-PH2024OUT.pdf> | 2026-10-11 | 125742 | `e24036455f13b622c3eecd8e952c32dbdbe80c84eb82992988cc9d010840b4ac` | cache |
| `LCC_PJ-PK2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PJ-PK2024OUT.pdf> | 2026-10-11 | 127963 | `9703f0e7a8582759f5d9c0f8f23a6c0cf168ecd52ae2f5b3f28520c17ca6542f` | cache |
| `LCC_PL-PM2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PL-PM2024OUT.pdf> | 2026-10-11 | 108321 | `eba59907f9134e8317f7d8a2a6f7b1a0ee76345f2f51b22f755e1437dd90f061` | cache |
| `LCC_PN2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PN2024OUT.pdf> | 2026-10-11 | 84520 | `9640b559b2e55380a264de030419d40a64a65419771e3d3900d774a39443c77a` | cache |
| `LCC_PQ2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PQ2024OUT.pdf> | 2026-10-11 | 97619 | `b37596dcaa553ccf26f6645b91762dfa969d5b9e3c001257c5ae0599e6f92ed8` | cache |
| `LCC_PR-PS-PZ2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PR-PS-PZ2024OUT.pdf> | 2026-10-11 | 98196 | `7860a98d0d2d8a20e86390ffabccd1460d21bf3883fa39be6e0f6d5e1a1ecbfd` | cache |
| `LCC_PT2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PT2024OUT.pdf> | 2026-10-11 | 103988 | `ecb86c5e6527e770a69c78733a5fbcee43a9ea6b9bb1f7b90eb18211d53da2ef` | cache |
| `LCC_Q2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_Q2024OUT.pdf> | 2026-10-11 | 102522 | `2d7098a1bda0d2f3c469f3111c537a7a1ee1730ace47fe2a3ba955460327ea15` | cache |
| `LCC_R2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_R2024OUT.pdf> | 2026-10-11 | 122772 | `e3b70cc76a8597f21a3e52a93629660e900eac944d8ccd66eae3c645231119c1` | cache |
| `LCC_S2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_S2024OUT.pdf> | 2026-10-11 | 118843 | `15f6abd398f7660781500d18591b2c64cb3fbebf96e4c6732c1ff0164bfec365` | cache |
| `LCC_T2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_T2024OUT.pdf> | 2026-10-11 | 116137 | `7bdb7d56e73c3f6b025d7e9a9c15009d64104c8f38af3517037d886803cbf1de` | cache |
| `LCC_U-V2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_U-V2024OUT.pdf> | 2026-10-11 | 117367 | `bcdc1029f0daf9b2e51b4d20475bc33b9708d00eee58020f277530781a010482` | cache |
| `LCC_Z2024OUT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_Z2024OUT.pdf> | 2026-10-11 | 110373 | `fc16ac4caa23cf9e38753cc5f6be7258cc8c2d5a624860142594dfddbe37a8a5` | cache |

### A3. LCSH bulk dump (MADS/RDF N-Triples)

1 files, locked in `data/schemes/lcc/sources.lock.json`.

| file | url | fetched | bytes | sha256 | stored |
|---|---|---|---:|---|---|
| `subjects.madsrdf.nt.gz` | <https://id.loc.gov/download/authorities/subjects.madsrdf.nt.gz> | 2026-10-11 | 230228148 | `f57e6dcf40d510e8be37ef25ba9e78f6f6768d24118563dd5898bc394596eebd` | cache |

### A4. id.loc.gov classification records (captions the outline PDFs omit)

92 files, locked in `data/schemes/lcc/sources.lock.json`.

| file | url | fetched | bytes | sha256 | stored |
|---|---|---|---:|---|---|
| `id-classification/CD1-CD6471.json` | <https://id.loc.gov/authorities/classification/CD1-CD6471.json> | 2026-10-11 | 8752 | `e9c9ad5b07231e81fb0da59e7cff34ff2c762ece060e54767b85e601a1f316be` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD1-CD511.json` | <https://id.loc.gov/authorities/classification/CD1-CD511.json> | 2026-10-11 | 13292 | `ba7bcca4302282116f25df36c52e4a81c33753011897f4473dd93eb6fc198af0` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD70-CD79.json` | <https://id.loc.gov/authorities/classification/CD70-CD79.json> | 2026-10-11 | 10650 | `f3e76429e8d02a89d08b7a0f1096ff4850e89cff2eae4862c1a0f78769bbe212` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD80-CD81.json` | <https://id.loc.gov/authorities/classification/CD80-CD81.json> | 2026-10-11 | 8366 | `2ec20d3a889cedd01ce68da9bad597e46f49ed7d6ad84f4b9d02ec7652deef93` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD87.json` | <https://id.loc.gov/authorities/classification/CD87.json> | 2026-10-11 | 7282 | `4a1f0ce595f00c42e885e118b4bee39de6f54158f69bf2645d3b8f2d15911ee1` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD91-CD392.json` | <https://id.loc.gov/authorities/classification/CD91-CD392.json> | 2026-10-11 | 9005 | `442bff3927e5c9d709c3371250e04fddccc70c7de383da48d35a5266822f25fe` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD501-CD511.json` | <https://id.loc.gov/authorities/classification/CD501-CD511.json> | 2026-10-11 | 8202 | `27887db3d1039411b568a1e5730b02dde0eb0fd411171a609a9a3593674244e3` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD921-CD4280.json` | <https://id.loc.gov/authorities/classification/CD921-CD4280.json> | 2026-10-11 | 17020 | `93a7e18443389ba319270f34bdf7591a5eec593de1ca3270a7fd2cd99a20516e` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD995-CD4280.json` | <https://id.loc.gov/authorities/classification/CD995-CD4280.json> | 2026-10-11 | 9966 | `66e5b5aba53db900afe1e7f731a61e4a52c976388a13e2310d7ef945e1747f15` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD997.A1-CD997.Z.json` | <https://id.loc.gov/authorities/classification/CD997.A1-CD997.Z.json> | 2026-10-11 | 9173 | `ea6233a7d1176c7e2e6e32041ebe51353d2e4c0de29fa71e5aaf16ab958347d9` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD1000-CD4280.json` | <https://id.loc.gov/authorities/classification/CD1000-CD4280.json> | 2026-10-11 | 13234 | `acf765b86c23571e52508a893f856f7b7cdbd5ac6939f3cd1650454631ed482e` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD5001-CD6471.json` | <https://id.loc.gov/authorities/classification/CD5001-CD6471.json> | 2026-10-11 | 20912 | `cf9098117b068b9207f74bbdd6cf242ede6f8b7d07714046e10e9eda34dc564b` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD5191.json` | <https://id.loc.gov/authorities/classification/CD5191.json> | 2026-10-11 | 7186 | `ef6ea7a2a9798da27028cf15c4d58456b13f62c6e391500458c9b0f8eababf97` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD5201-CD5391.json` | <https://id.loc.gov/authorities/classification/CD5201-CD5391.json> | 2026-10-11 | 11330 | `cfa28982ff2f7091083a94f9e18d2f29e1f351a3e4665105836bf5bc503a6c32` | git: `data/schemes/lcc/sources/` |
| `id-classification/CD5561.json` | <https://id.loc.gov/authorities/classification/CD5561.json> | 2026-10-11 | 7411 | `ce644b87279650d86a53ba734bb21aab99a2e679ad3afd1bae0bb7b6461f18b3` | git: `data/schemes/lcc/sources/` |
| `id-classification/CT21-CT9999.json` | <https://id.loc.gov/authorities/classification/CT21-CT9999.json> | 2026-10-11 | 10077 | `6daf9c4453b3ffbd16dfdfd24239cfbfc1ad9a13f3dac458ba8e5aae9a04291e` | git: `data/schemes/lcc/sources/` |
| `id-classification/CT21-CT22.json` | <https://id.loc.gov/authorities/classification/CT21-CT22.json> | 2026-10-11 | 7369 | `641d45eaa04e73510b00d1845b182618da3b2c11762c8b53a017d6b0f06f12c4` | git: `data/schemes/lcc/sources/` |
| `id-classification/CT31-CT83.json` | <https://id.loc.gov/authorities/classification/CT31-CT83.json> | 2026-10-11 | 9907 | `ace75c1fa959e2b0a413b0f48ff460147a44e4c763b402e024c99847677dbe48` | git: `data/schemes/lcc/sources/` |
| `id-classification/CT88-CT206.json` | <https://id.loc.gov/authorities/classification/CT88-CT206.json> | 2026-10-11 | 8461 | `f14f0a65d6fae58e5047a9c73283b2ecf39186fefd4a498def132f3e86e002c1` | git: `data/schemes/lcc/sources/` |
| `id-classification/CT206.json` | <https://id.loc.gov/authorities/classification/CT206.json> | 2026-10-11 | 8029 | `830ffb0435c1f98c23beecf09401d275c17a87ce1f701aaba1cb886a1e0c5c37` | git: `data/schemes/lcc/sources/` |
| `id-classification/CT208-CT3150.json` | <https://id.loc.gov/authorities/classification/CT208-CT3150.json> | 2026-10-11 | 8452 | `4aa87a2360f68b648485441e19a79027f5edebde4af19707590c17ac7f665ba4` | git: `data/schemes/lcc/sources/` |
| `id-classification/CT3200-CT9999.json` | <https://id.loc.gov/authorities/classification/CT3200-CT9999.json> | 2026-10-11 | 9593 | `3c90fffd52e8b59eead87c0fe2c0f1902eb7fb1dbf4aaa41aeae22e30d688c85` | git: `data/schemes/lcc/sources/` |
| `id-classification/CT3200-CT3830.json` | <https://id.loc.gov/authorities/classification/CT3200-CT3830.json> | 2026-10-11 | 14376 | `c2d0ffe34fe7ef655e4a9429dddb79eef5e2f507bb11a1ff9790eef966cdc96b` | git: `data/schemes/lcc/sources/` |
| `id-classification/CT3990.A2-CT3990.Z.json` | <https://id.loc.gov/authorities/classification/CT3990.A2-CT3990.Z.json> | 2026-10-11 | 10780 | `f9bb8f0553c11b420b93e0510a45a4ebaa023748149e6fff00c6da76b78b4116` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK507.92-DK507.939.json` | <https://id.loc.gov/authorities/classification/DK507.92-DK507.939.json> | 2026-10-11 | 14157 | `f526f47fa93e536038bed33d07abc67bf3ef0afa21075066bde22703ec4ad7ca` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK509.json` | <https://id.loc.gov/authorities/classification/DK509.json> | 2026-10-11 | 10361 | `e31d2f816871c74f945f851edd811165b99b9eda1deeab3081285ad725cf6516` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK509.1-DK509.95.json` | <https://id.loc.gov/authorities/classification/DK509.1-DK509.95.json> | 2026-10-11 | 19863 | `be92b5b00f52a9b7a69255abbf8a5e2d2bb4a587c1021e3be6160f26ee278e6b` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK509.92-DK509.939.json` | <https://id.loc.gov/authorities/classification/DK509.92-DK509.939.json> | 2026-10-11 | 14205 | `9fe56fc100ed45298b6b287706a3f6fcefac7cadfa17f5f1f764401b765b502a` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK510-DK651.json` | <https://id.loc.gov/authorities/classification/DK510-DK651.json> | 2026-10-11 | 20200 | `f7033e0e159dda401b778ad7a0c673b449f5ac983c03ca11f5747a95b3fe1d0b` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK541-DK579.json` | <https://id.loc.gov/authorities/classification/DK541-DK579.json> | 2026-10-11 | 16221 | `8e9d6a23231d7aac7e7bf2e6bb7b63d4daa9213fd888c8238ae457fddf114338` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK588-DK609.json` | <https://id.loc.gov/authorities/classification/DK588-DK609.json> | 2026-10-11 | 16457 | `2640b9ff412643d554e1026c33befd93f84ccc23f149dab7f0a4fb1ad7c0fa36` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK670-DK679.5.json` | <https://id.loc.gov/authorities/classification/DK670-DK679.5.json> | 2026-10-11 | 17956 | `0744071f505b2cf7c32f7990e3ce7d7a347e64da690807717f51a4698f0fcbcf` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK679.2-DK679.39.json` | <https://id.loc.gov/authorities/classification/DK679.2-DK679.39.json> | 2026-10-11 | 14190 | `2b5b8a913e11ce5c2b7bd73f7e9fd533add8bc889ce91aff4e65f08b69771006` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK680-DK689.5.json` | <https://id.loc.gov/authorities/classification/DK680-DK689.5.json> | 2026-10-11 | 17994 | `a52e2fcfff42de31d46f43e3e0ea77aab34f2d4d628f19e4c177b1fb3db9bfab` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK689.2-DK689.39.json` | <https://id.loc.gov/authorities/classification/DK689.2-DK689.39.json> | 2026-10-11 | 14070 | `9fb3c4c3b9c622b0f8e4ad66d16d08a4cbb10364389f05d4f27e45f2c341ab98` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK690-DK699.5.json` | <https://id.loc.gov/authorities/classification/DK690-DK699.5.json> | 2026-10-11 | 17510 | `2dad8f57326777c822508a07390641aa51b311242bbc5acf8699913c3e89a7f3` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK699.2-DK699.39.json` | <https://id.loc.gov/authorities/classification/DK699.2-DK699.39.json> | 2026-10-11 | 13968 | `ca9dd7272f0d56b42dafb96aab9b1f21af1afa4ba254ab1c38d70cecfc78b69a` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK751-DK781.json` | <https://id.loc.gov/authorities/classification/DK751-DK781.json> | 2026-10-11 | 17085 | `c3459f654d816b6a1541b9e6720ea94749b6ad16f2b2cddaeb36a02110bbc6a0` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK845-DK860.json` | <https://id.loc.gov/authorities/classification/DK845-DK860.json> | 2026-10-11 | 18033 | `11974a7faa9fdcc427c9fcb7fe873c6db360560c4df4f82e9f8d35647e586d3d` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK901-DK909.5.json` | <https://id.loc.gov/authorities/classification/DK901-DK909.5.json> | 2026-10-11 | 17091 | `8d5e198cf78c702df40c43161ebb2b06a82c9ffe3e502fd1a7d9c57dd893ed6d` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK909.2-DK909.39.json` | <https://id.loc.gov/authorities/classification/DK909.2-DK909.39.json> | 2026-10-11 | 13968 | `cb1484a36bcdd26865f914d18a1e65f08063e8f4d6d3a2d53536d9a4311684d1` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK911-DK919.5.json` | <https://id.loc.gov/authorities/classification/DK911-DK919.5.json> | 2026-10-11 | 17788 | `fea0e0f1d4b6f1e91d2ad17823842ca6820c0e762091bd6f6432e72d0657944e` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK919.2-DK919.39.json` | <https://id.loc.gov/authorities/classification/DK919.2-DK919.39.json> | 2026-10-11 | 14076 | `cbadf5ff7fa5aef192e033137f509c5ec1e77b39b00c991b5ff3174328aa8e0b` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK921-DK929.5.json` | <https://id.loc.gov/authorities/classification/DK921-DK929.5.json> | 2026-10-11 | 17852 | `15c758e88128dc39060aa497b2aeb70c6c2c098cc89c65afd00f9fddd9636d5e` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK929.2-DK929.39.json` | <https://id.loc.gov/authorities/classification/DK929.2-DK929.39.json> | 2026-10-11 | 14040 | `47ab4565c99ad9eab40a7fc75b4e0a11f422a2bbc82ab7c7dfd1ef14efd33915` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK931-DK939.5.json` | <https://id.loc.gov/authorities/classification/DK931-DK939.5.json> | 2026-10-11 | 17806 | `a1dcd936513f7eaa8c96e8740f0bdc5cf735669608341fa40f3df679b81653ec` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK939.2-DK939.39.json` | <https://id.loc.gov/authorities/classification/DK939.2-DK939.39.json> | 2026-10-11 | 14052 | `2f4ac8178b0d8ad072a9fe68e1bcf9d1c6ccd0a8562e7040fce703a56ebc8d7e` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK941-DK949.5.json` | <https://id.loc.gov/authorities/classification/DK941-DK949.5.json> | 2026-10-11 | 17082 | `1a2ea8c0bed83f521ac0501e8272f3d383b0ec2eded27558cb755fbc4185f7d1` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK949.2-DK949.39.json` | <https://id.loc.gov/authorities/classification/DK949.2-DK949.39.json> | 2026-10-11 | 13962 | `611ee81c3c62ba86fe1e0fe705a3269b9ee4b6db8e40d3fe3d659467c06304a5` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4010-DK4800.json` | <https://id.loc.gov/authorities/classification/DK4010-DK4800.json> | 2026-10-11 | 14462 | `61be0fb40b1b3ca0a162466817ad4608289fc72304f1ca6beeb06a599420c7e9` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4120-DK4122.json` | <https://id.loc.gov/authorities/classification/DK4120-DK4122.json> | 2026-10-11 | 8593 | `e345c650f2e31e82402e11a9c720e224b48e64d535b0286f45604855739f631d` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4186-DK4348.json` | <https://id.loc.gov/authorities/classification/DK4186-DK4348.json> | 2026-10-11 | 10039 | `145dabcc97ca62ef5118b1232fa7c28624fca823d29950b6dcbc0988ad9ab07f` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4348.5-DK4395.json` | <https://id.loc.gov/authorities/classification/DK4348.5-DK4395.json> | 2026-10-11 | 10635 | `5e95b4963b95c6bbdf8741add9cd75c1b2781c0a6b7bcb8c96487a0c80438e77` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4397-DK4420.json` | <https://id.loc.gov/authorities/classification/DK4397-DK4420.json> | 2026-10-11 | 12395 | `5faa7151d21a1bd0e0130b6e95b06ea16d66880d3bb5200a18938b6ee1daec77` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4429-DK4442.json` | <https://id.loc.gov/authorities/classification/DK4429-DK4442.json> | 2026-10-11 | 11651 | `44de686d5ee7be39154d33c889e6372b9ad0e0971fb2b325489ed05666840031` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4445-DK4452.json` | <https://id.loc.gov/authorities/classification/DK4445-DK4452.json> | 2026-10-11 | 9951 | `d4560617f48f6d5596f55bced463c89f6750b1fce7a591b495dab57d2ba86c19` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4600-DK4800.json` | <https://id.loc.gov/authorities/classification/DK4600-DK4800.json> | 2026-10-11 | 11248 | `ad77538c83a85d3905ad5dcb242d6c43f9abcbb7552354e31dc1ea9faf7c0308` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4610-DK4645.json` | <https://id.loc.gov/authorities/classification/DK4610-DK4645.json> | 2026-10-11 | 15595 | `06d05b768fa866ea5c1322eb3a36755488c0a07585f2843c83aead44dc000d71` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4650-DK4685.json` | <https://id.loc.gov/authorities/classification/DK4650-DK4685.json> | 2026-10-11 | 15504 | `7dee0e5493dff275bd0189ecae0f7832fab5249b7d10060726d732bb305e057f` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK4700-DK4735.json` | <https://id.loc.gov/authorities/classification/DK4700-DK4735.json> | 2026-10-11 | 15488 | `e3f32da9b4815548871b60c9304380a4378f5e996adf649fae1eb9e03d1e975d` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5001-DK5995.json` | <https://id.loc.gov/authorities/classification/DK5001-DK5995.json> | 2026-10-11 | 14233 | `cbe169593b6882599b70fddf939e2121e5f409d3d12752f00f19d42f1ba82933` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5091-DK5099.json` | <https://id.loc.gov/authorities/classification/DK5091-DK5099.json> | 2026-10-11 | 8570 | `113f94b923bb3291891caa3d78109466f9c0c68cdad5e93b4dad1146d38f8c02` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5109-DK5529.json` | <https://id.loc.gov/authorities/classification/DK5109-DK5529.json> | 2026-10-11 | 6789 | `4c20231e0c9fb51d592d9b1a3258e6899350139e803afcd33b180796f0b4bf6a` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5205-DK5219.json` | <https://id.loc.gov/authorities/classification/DK5205-DK5219.json> | 2026-10-11 | 13570 | `956e0137e3c494d16d62eb1a8e3ea5623a0febf45c0c774fe49f62d041ab9d7a` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5223-DK5237.json` | <https://id.loc.gov/authorities/classification/DK5223-DK5237.json> | 2026-10-11 | 10761 | `3b11d37999cd75081a62ffc2d2e70ad232d2f92c1af71f6726d74fb4ec6c1453` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5241-DK5255.json` | <https://id.loc.gov/authorities/classification/DK5241-DK5255.json> | 2026-10-11 | 10605 | `a8ddf2d0252081bb322771b120f53e42055cddb7dcbd8c094d968fb831d4ec66` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5259-DK5287.json` | <https://id.loc.gov/authorities/classification/DK5259-DK5287.json> | 2026-10-11 | 11838 | `81890425cf67af6f88dd2ec786f4c00997199e7e978ce166159b8605a695db5e` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5289-DK5303.json` | <https://id.loc.gov/authorities/classification/DK5289-DK5303.json> | 2026-10-11 | 10629 | `05926f3c1766520934b5dcd8253a9f1876e55115d59141bcaae781b092d06347` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5309-DK5357.json` | <https://id.loc.gov/authorities/classification/DK5309-DK5357.json> | 2026-10-11 | 11541 | `a00f1317cbe251f96fcedbe1f908b07b1a99a2eacbf532753b3f0e99cf9b1538` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5363-DK5381.json` | <https://id.loc.gov/authorities/classification/DK5363-DK5381.json> | 2026-10-11 | 11004 | `f459a6e6c4652040f2b430aaaece106195702546f90390acbd4d1321967f6e13` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5385-DK5403.json` | <https://id.loc.gov/authorities/classification/DK5385-DK5403.json> | 2026-10-11 | 10434 | `c081db153f884cc04aa69f0b627e202a0086c539e27574ad2a9e41d7e72efb51` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5409-DK5529.json` | <https://id.loc.gov/authorities/classification/DK5409-DK5529.json> | 2026-10-11 | 7473 | `e0f1b0731c61d7dc5304e3b6d088647b7ff3a0a86b6ebc5f7159e2f0cfe61fa9` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5850-DK5995.json` | <https://id.loc.gov/authorities/classification/DK5850-DK5995.json> | 2026-10-11 | 9443 | `f7b15cb70676aeea7c5d7f436240ddadb64fdd33ab8d107eed88b53659840ff5` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5860-DK5878.json` | <https://id.loc.gov/authorities/classification/DK5860-DK5878.json> | 2026-10-11 | 16232 | `091d6a55e3fcd296ab6bf373fc010f41725700dd63f4a9555e67c1192ae80f19` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5880-DK5898.json` | <https://id.loc.gov/authorities/classification/DK5880-DK5898.json> | 2026-10-11 | 16200 | `8322a1d45273789fa4506bda85b5ca3f2473ce090a97ad885adc18f16d6ef5ee` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5900-DK5918.json` | <https://id.loc.gov/authorities/classification/DK5900-DK5918.json> | 2026-10-11 | 16240 | `c020a40af767b91687bcf38bbe8a31921206fd50eb52b123bcbc28a5c03dbbbe` | git: `data/schemes/lcc/sources/` |
| `id-classification/DK5920-DK5938.json` | <https://id.loc.gov/authorities/classification/DK5920-DK5938.json> | 2026-10-11 | 16184 | `e7228f4228a0218d5393da776a02f3d42c476b6d2a07df439fbb070abd075dda` | git: `data/schemes/lcc/sources/` |
| `id-classification/DT2701-DT2825.json` | <https://id.loc.gov/authorities/classification/DT2701-DT2825.json> | 2026-10-11 | 14187 | `13eb9f5c02d857178e4f7930c4e5f0aff8efde41fe16bfd850e89cab08a7a227` | git: `data/schemes/lcc/sources/` |
| `id-classification/KBM523.72.json` | <https://id.loc.gov/authorities/classification/KBM523.72.json> | 2026-10-11 | 7974 | `b727d335a8f5a9db89ae3e56275f086813d70576dfcc59a00f97a8d0de3e0e42` | git: `data/schemes/lcc/sources/` |
| `id-classification/KD.json` | <https://id.loc.gov/authorities/classification/KD.json> | 2026-10-11 | 1471 | `f4d3ee55190762f4cfbc822a6a21ed78dc23b52b824b46fa61a930fa409d49a0` | git: `data/schemes/lcc/sources/` |
| `id-classification/KJ.json` | <https://id.loc.gov/authorities/classification/KJ.json> | 2026-10-11 | 6926 | `7b369577458c4a0892c4b327f121e9d770e828e3d496497cf5bfb6b9bdc92ecb` | git: `data/schemes/lcc/sources/` |
| `id-classification/KL.json` | <https://id.loc.gov/authorities/classification/KL.json> | 2026-10-11 | 3314 | `810ccb69a9d578f24688f2281c92ef53400912d19ea66e1668a0a94b0d2183ca` | git: `data/schemes/lcc/sources/` |
| `id-classification/KM.json` | <https://id.loc.gov/authorities/classification/KM.json> | 2026-10-11 | 3905 | `ecfc3867e99ccd5ad97ec471d789f7457480052b961470065ce4203951cbb8bb` | git: `data/schemes/lcc/sources/` |
| `id-classification/KN.json` | <https://id.loc.gov/authorities/classification/KN.json> | 2026-10-11 | 3627 | `4ccd04759228e2d42a542d367e29788ec087eaa37a04b40557bdf5e648f6e8be` | git: `data/schemes/lcc/sources/` |
| `id-classification/KP.json` | <https://id.loc.gov/authorities/classification/KP.json> | 2026-10-11 | 2766 | `2ea7b33b7674e78d780bda57046b3c95f579456bdb1bb5715c5b73cde07c4d5d` | git: `data/schemes/lcc/sources/` |
| `id-classification/KQ.json` | <https://id.loc.gov/authorities/classification/KQ.json> | 2026-10-11 | 2902 | `f26760b668a35ffa205dd0807fe4546684f8f923f286415dec4d8e450d855fd7` | git: `data/schemes/lcc/sources/` |
| `id-classification/KR.json` | <https://id.loc.gov/authorities/classification/KR.json> | 2026-10-11 | 3009 | `d336af697ffe2f2dbdef8b27dfe268cc3702b76c2984ca096195c65744ae37e4` | git: `data/schemes/lcc/sources/` |
| `id-classification/KS.json` | <https://id.loc.gov/authorities/classification/KS.json> | 2026-10-11 | 3754 | `54dfcc79e41339b03a49005441794f1e15686108106a3938e099abbcc91ca5dd` | git: `data/schemes/lcc/sources/` |
| `id-classification/KT.json` | <https://id.loc.gov/authorities/classification/KT.json> | 2026-10-11 | 3881 | `762bbdb1ec65e67edee7b7779aa0bcafc59c8bbebbee6b9f7d933d309b82c676` | git: `data/schemes/lcc/sources/` |
| `id-classification/KU.json` | <https://id.loc.gov/authorities/classification/KU.json> | 2026-10-11 | 2475 | `acfe266b558126fe9e2dd9871c7c681d459b8b1d745d701f81c2b57c4065c172` | git: `data/schemes/lcc/sources/` |
| `id-classification/KV.json` | <https://id.loc.gov/authorities/classification/KV.json> | 2026-10-11 | 3045 | `c6590ea1aabfedd30afc6f4987a869cecf0b5cdd5b695df293969c75b360eb73` | git: `data/schemes/lcc/sources/` |
| `id-classification/KW.json` | <https://id.loc.gov/authorities/classification/KW.json> | 2026-10-11 | 2751 | `6cf80c397a71c9b6708eb017eab1fcf7ad1e06658f98e805d8c2e278224e1631` | git: `data/schemes/lcc/sources/` |

### B1. FreeLCC 2024 Text PDFs (the complete schedules)

44 files, locked in `schemes/lcc-full/sources.lock.json` (outline_sha256 `b21e4c6680c72f82df633f397e54291eeb1ada3ffe3a2df1dcf0af6211eb79f7`).

| file | url | fetched | bytes | sha256 | stored |
|---|---|---|---:|---|---|
| `LCC_A2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_A2024TEXT.pdf> | 2026-10-09 | 197228 | `61d02bb98f21bfbf943b04ad97605b0a646a92e82d6412ad1916f1b52ca783a2` | cache |
| `LCC_B-BJ2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_B-BJ2024TEXT.pdf> | 2026-10-09 | 980654 | `6d940aa5bec7ade87031b084da040076e1df57c50a4df2f9010e7286d916a430` | cache |
| `LCC_BL-BQ2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_BL-BQ2024TEXT.pdf> | 2026-10-09 | 1181197 | `a597f5ee4808941a86b7f44033f8532f47c7afb144e6770ba30938aa89104039` | cache |
| `LCC_BR-BX2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_BR-BX2024TEXT.pdf> | 2026-10-09 | 1421673 | `2d311af1d2316672d73962da0cbac7460e3b1b38077b03c95b393d43e0b55e06` | cache |
| `LCC_C2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_C2024TEXT.pdf> | 2026-10-09 | 426191 | `a4203b748375517fc71ba0684b9ea6f3357b2ec094bbc35fdbc6f55f79cee5da` | cache |
| `LCC_D-DR2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_D-DR2024TEXT.pdf> | 2026-10-09 | 1710193 | `6240a23125292d7c1db50b751efd1d462822d4aff3c00e770b34a3b4db5f289b` | cache |
| `LCC_DS-DX2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_DS-DX2024TEXT.pdf> | 2026-10-09 | 976534 | `e6caba39142d894864b77b42c6ae47fc8dd76041860863b16d6c803f6aeb63f0` | cache |
| `LCC_E-F2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_E-F2024TEXT.pdf> | 2026-10-09 | 1758665 | `92b67beeb8b8af2b173180022e90365a1fb8c5b277b849a61c81a1bf95ccecff` | cache |
| `LCC_G2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_G2024TEXT.pdf> | 2026-10-09 | 1468459 | `731de5b09bcf0aa2673f16a05c0ca9c2b3da3d43d3531aee68cb686b74f8ff45` | cache |
| `LCC_H2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_H2024TEXT.pdf> | 2026-10-09 | 1932027 | `53e68cded3ecf0acf337ebd904e3484a2ccd3924bf8ab57c3325806d6953ae7d` | cache |
| `LCC_J2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_J2024TEXT.pdf> | 2026-10-09 | 923999 | `0034b367dc4bd45d904b93b523d37e20f0f99668d4098bd80eb69fcc3709e99c` | cache |
| `LCC_K2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_K2024TEXT.pdf> | 2026-10-09 | 373728 | `09db5d3b4692ebcf595f134c7f031fa108c52eb4f2bb51fd40ee7dd2d7252433` | cache |
| `LCC_KB2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KB2024TEXT.pdf> | 2026-10-09 | 1149630 | `77035b1dfbec0fd89ced31066db2e05d7cf19b867afbdad6c001564a81f38819` | cache |
| `LCC_KD2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KD2024TEXT.pdf> | 2026-10-09 | 508339 | `7cff079176736758a179e171554428322c5d0ade44be74c57333bc9f46acfb5d` | cache |
| `LCC_KDZ_KG-KH2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KDZ_KG-KH2024TEXT.pdf> | 2026-10-09 | 714992 | `2b23a9a7680558bc12224a332bacd274105ce4147acedce7b2815a2ae30b4ecc` | cache |
| `LCC_KE2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KE2024TEXT.pdf> | 2026-10-09 | 526504 | `14224dbe5980b8ad6c1cd280f8ed39fee174c6dcc05c07c8ce9edf4b772ba23e` | cache |
| `LCC_KF2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KF2024TEXT.pdf> | 2026-10-09 | 1199369 | `39574ec61370763fcc272338b71b18f3340ab8cd527b0bec14b179be36ba610a` | cache |
| `LCC_KI-KIL2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KI-KIL2024TEXT.pdf> | 2026-10-09 | 478219 | `1d9759712d7603a3f562bf7659ce04237eba382a1ce2ce4c1d5ffbf35637ed76` | cache |
| `LCC_KIM-KIR2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KIM-KIR2024TEXT.pdf> | 2026-10-09 | 314938 | `0e7473e3ad9bab5f7207951be59e3455c2ec224f04f97dd8a6e488de2e1bd304` | cache |
| `LCC_KJ-KKZ2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KJ-KKZ2024TEXT.pdf> | 2026-10-09 | 1325905 | `5cd2094d208320cb294e1d1f21478a866c71605db0e5fc646586169508414043` | cache |
| `LCC_KJV-KJW2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KJV-KJW2024TEXT.pdf> | 2026-10-09 | 600340 | `b3c31d0447da42f4ee87171ab2cb62e4dee0a07e733092ca98fee326ffd49377` | cache |
| `LCC_KK-KKC2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KK-KKC2024TEXT.pdf> | 2026-10-09 | 1151961 | `63ed9abc6058100efa07bd359d3f3e73ef836fcdfb6433ad062034e3b2c6912b` | cache |
| `LCC_KL-KWX2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KL-KWX2024TEXT.pdf> | 2026-10-09 | 2009163 | `454310fb5dd5e6e2035c4a1835e7ea0dfa9d88b95ebc73ff061ae5fc63e12456` | cache |
| `LCC_KVJ2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KVJ2024TEXT.pdf> | 2026-10-09 | 187984 | `7b7e202130a9fb178b40af5466e6a6432af0b782330e70701a8f4f994c803a06` | cache |
| `LCC_KZ2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_KZ2024TEXT.pdf> | 2026-10-09 | 440853 | `80db6c7eeb771f4e3550f15da9d30aba29b01b2633e12b0532283a066615b100` | cache |
| `LCC_K_TABLES2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_K_TABLES2024TEXT.pdf> | 2026-10-09 | 130986 | `d1d6ad49660a906710e9cead27392a08fff8d6936e23d9eea977e9d48bdc9300` | cache |
| `LCC_L2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_L2024TEXT.pdf> | 2026-10-09 | 775341 | `1655c703e4c421490b1660cdfd7cf48fa4f6cdce81e3377859887b30590c6958` | cache |
| `LCC_M2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_M2024TEXT.pdf> | 2026-10-09 | 738387 | `0c3e3859cd4b142168dfaa7ccd9ea2036b207f0b7bb1532022c8d8735e2cd7fc` | cache |
| `LCC_N2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_N2024TEXT.pdf> | 2026-10-09 | 1419028 | `d7f775b27aba5db854c5a5872182ff1d6d76eb001e0777dc8fa26e79a6dae369` | cache |
| `LCC_P-PA2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_P-PA2024TEXT.pdf> | 2026-10-09 | 949779 | `c52ea10481f41d618a648f62f39bb7e7e819b4162b475b40761d9ca1ec7ddf9a` | cache |
| `LCC_PB-PH2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PB-PH2024TEXT.pdf> | 2026-10-09 | 1541911 | `67a7eec8fce4e9efc20d2611cf3c44785330fdc7beddcbd46dbfcda15f473581` | cache |
| `LCC_PJ-PK2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PJ-PK2024TEXT.pdf> | 2026-10-09 | 691951 | `4d31c80ccdca02556577a4c8634348e20202a2d8629a81dcc641ea6bdcabb5af` | cache |
| `LCC_PL-PM2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PL-PM2024TEXT.pdf> | 2026-10-09 | 1605542 | `57be508ae0fc8c25d1da647f8bc0355d907012d6a206000c90333e1122e2d43e` | cache |
| `LCC_PN2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PN2024TEXT.pdf> | 2026-10-09 | 737384 | `cbafb0ecceef8f32a8b0f3be9a5c89f376dfdd82c800188cdbf85add40f92752` | cache |
| `LCC_PQ2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PQ2024TEXT.pdf> | 2026-10-09 | 1117538 | `8d02376571aa520463acbf852b11444723aa7132ca78bd0723de63f634378056` | cache |
| `LCC_PR-PS_PZ2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PR-PS_PZ2024TEXT.pdf> | 2026-10-09 | 1504218 | `a04bf24f565ad683b27c9682e6c01c4c26b9f72049ba255fd3bda6ecfad39d9a` | cache |
| `LCC_PT2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_PT2024TEXT.pdf> | 2026-10-09 | 907760 | `f2aa577955b84f7fdf637f4ae4e53059a15bd23a5037b23b1db97cf2e5b2ac58` | cache |
| `LCC_P_TABLES2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_P_TABLES2024TEXT.pdf> | 2026-10-09 | 290006 | `54ffbc4a6ec119e9e24104a89559b59fc619e1082d1875784f8d81f0db04566c` | cache |
| `LCC_Q2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_Q2024TEXT.pdf> | 2026-10-09 | 1586875 | `a5952ce7dc306507d80de6bff70e28635ab456df2c2625bbc65c80369d6b9899` | cache |
| `LCC_R2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_R2024TEXT.pdf> | 2026-10-09 | 1046772 | `02280d5836cd09c3af14588db4501dd4e015d1e4907e2663f4a02f84416c24f4` | cache |
| `LCC_S2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_S2024TEXT.pdf> | 2026-10-09 | 890293 | `386d12e8990968f60115333dd84705e4611f61cc16a57fdc2f4355df714209e4` | cache |
| `LCC_T2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_T2024TEXT.pdf> | 2026-10-09 | 1537115 | `1f22d05e52828717692ddfa8e7bae7bdb750f0a8ffafe814577f6ca80a4da967` | cache |
| `LCC_U-V2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_U-V2024TEXT.pdf> | 2026-10-09 | 587141 | `20e36b5b1ca42089fd1cc2e4cab865cbf52def24f5b9c7d8190143eff834167e` | cache |
| `LCC_Z2024TEXT.pdf` | <https://www.loc.gov/aba/publications/FreeLCC/LCC_Z2024TEXT.pdf> | 2026-10-09 | 1347651 | `58dbf29b12c740f343655de0dade081835d7d831a61c41f160f2aaaabe0abd4f` | cache |

