# MSC2020 scheme

The Mathematics Subject Classification 2020 as shelfmark scheme
`scheme:msc@2020` (class IDs like `class:msc@2020:05C10`). Its data is
**not** in this repository and is not compiled into the shelfmark binary.
`tool/` (`msc-scheme`) downloads the official CSV, checks it against
`sources.lock.json`, and writes a scheme file you compile yourself.

## License check

| | |
|---|---|
| Source | <https://msc2020.org/> (`MSC_2020.csv`, also PDF and TeX) |
| Publisher | Mathematical Reviews (American Mathematical Society) and zbMATH Open (FIZ Karlsruhe), jointly |
| License | [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/), as stated on msc2020.org (checked 2026-10-08) |
| Attribution | Credit "Mathematics Subject Classification MSC2020, Mathematical Reviews and zbMATH Open, https://msc2020.org/", and keep the license. The generated file carries both in `source`. |
| Can it ship in the binary? | **No.** shelfmark is MIT OR Apache-2.0, and both allow commercial use. NonCommercial forbids that use, and ShareAlike would require the data to stay under CC BY-NC-SA. Bundling would impose those terms on everyone who redistributes the binary. |

So the data is fetched when you build the scheme, never bundled. Nothing
derived from the real CSV is committed here: the test fixture
(`tool/tests/fixtures/sample.tsv`) uses made-up labels in the official
format. A scheme file you build is still CC BY-NC-SA, so keep it to
non-commercial use and share it only under those terms.

## Build and use

```sh
cd schemes/msc/tool
cargo run --release -- build --out ~/msc2020.json      # fetches into ~/.cache/shelfmark/msc
cargo run --release -- check --out ~/msc2020.json      # fails if the file is stale
shelfmark scheme compile ~/msc2020.json                # preview, then --commit
shelfmark scheme browse --scheme scheme:msc@2020 05Cxx
shelfmark scheme search --scheme scheme:msc@2020 "computational real algebraic geometry"
```

`--cache DIR` picks the download cache and `--offline` refuses to fetch.
`build` refuses a CSV whose sha256 differs from the lock, so one lock
always builds the same file (6603 classes, sha256 `7a50db75…aac6` for the
lock dated 2026-10-08). If upstream publishes a correction, review it and
run `msc-scheme relock` to pin the new file.

## Mapping

| CSV code | Class | Parent | Range |
|---|---|---|---|
| `05-XX` | section (63) | none | `05A00`-`05-99` |
| `05-01` | section-wide form (`-00` to `-11`) | `05-XX` | none |
| `05Cxx` | subsection | `05-XX` | `05C00`-`05C99` |
| `05C10` | leaf | `05Cxx` | none |

- The code grammar is `[0-9]{2}-(?:XX|[0-9]{2})|[0-9]{2}[A-Z](?:xx|[0-9]{2})`.
  The ranges let `check_code` place an unassigned code (`05C98`) within
  its subsection.
- The label is the CSV's `text` column, verbatim, TeX math included
  (`\(p\)-adic`). Cross-references the `description` column adds
  (`[See also 68R10]`, `\{For ..., see ...\}`) become the class `note`.
- In the 534 `..99` rows, `text` repeats the subsection caption. Their
  label is the printed caption instead: "None of the above, but in this
  section".
- The CSV is Latin-1. The builder decodes it to UTF-8. One upstream
  character was already lost before publication (`Kre?n` in 46B22) and is
  kept as published.
- Browse order is shelfmark's shelf order, where letters sort before
  digits. So under `05-XX` the subsections come before `05-00`..`05-11`,
  the reverse of the printed MSC.
- **Vocabulary links: none.** The CSV has no LCC or LCSH links. MSC is
  not linked from LCSH either, so no class carries `vocab`, and search and
  the lexical tier match on labels only.

## Tests

```sh
cd schemes/msc/tool
cargo test                                                    # fixture; real-source tests skip
MSC_SOURCE_CACHE=~/.cache/shelfmark/msc MSC_REQUIRE_SOURCE=1 cargo test   # also the real CSV
```

Without `MSC_SOURCE_CACHE`, each real-source test logs a skip line to
`$TEST_SKIP_LOG`. The tests never download anything.
