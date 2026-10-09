# CPC scheme for shelfmark

`cpc-scheme` builds `scheme:cpc@2026.08`, the Cooperative Patent
Classification, from the official EPO/USPTO title list. The output is a
shelfmark scheme file: 254,314 classes from section down to the deepest
subgroup, with hierarchy, code grammar, labels, version and provenance.

Shelfmark does **not** bundle CPC, and the generated file is never
committed (`.gitignore`). This directory holds the builder and a lock
that pins the source and the expected output, so anyone can rebuild the
identical bytes locally.

## License check

| question | answer |
|---|---|
| owner | The EPO and the USPTO, jointly ([CPC FAQ](https://www.cooperativepatentclassification.org/faq): "co-owned by the EPO and USPTO") |
| license | No license of its own. The CPC site is run by the EPO and links no terms, so the [EPO website terms of use](https://www.epo.org/en/terms-and-conditions-use-website-european-patent-office-0) (last updated 26.4.2022) apply |
| reuse | §6.2 allows downloading and processing the information "for private and commercial purposes" |
| attribution | §6.4: cite the website as the source. For a derived work, also mark the changes, name the author (EPO/USPTO), and "expressly grant third parties a right to reproduce and distribute your adaptation" on the same conditions |
| may it ship inside an open-source binary? | **No, not safely.** §6.4 also says the right of use "does not include permission to copy all - or a substantial part of - the data ... from the Office or EPO databases". Shipping all 254k titles inside an MIT/Apache binary would be exactly that, and it would also pass the data on under terms other than §6. The USPTO half is a US government work, but the EPO's co-ownership still applies outside the US |

So the bead's fallback applies: shelfmark ships a downloader and builder
rather than the data. Each user fetches the official archive and builds
the scheme on their own machine, the use §6.2 permits. Every generated
file carries the credit, the change note and the terms in its `source`
block. If you redistribute a built `cpc.json`, keep that block and pass
the same §6 terms on. This is a reading of published terms, not legal
advice. Before bundling CPC, get written permission from the EPO through the
contact its terms give.

## Build

```sh
cd schemes/cpc
cargo run --release -- fetch --cache /tmp/cpc-sources --release 202608  # network; rewrites sources.lock.json
cargo run --release -- build --cache /tmp/cpc-sources                   # offline; writes cpc.json
cargo run --release -- build --cache /tmp/cpc-sources --depth main-group  # cpc-main-group.json
cargo run --release -- check --cache /tmp/cpc-sources                   # fails unless both rebuild byte for byte
```

`fetch` without `--release` picks the newest `CPCTitleListYYYYMM.zip` linked
from the [bulk download page](https://www.cooperativepatentclassification.org/cpcSchemeAndDefinitions/bulk).
A new release becomes a new scheme version, which means a new registration
entry in `crates/cli/src/wiring/mod.rs`.
`build` and `check` refuse any cached archive whose sha256 differs from
`sources.lock.json`. The lock also pins the sha256 of `cpc.json` and
`cpc-main-group.json`, so a builder change that alters output fails
`check`.

Then load it into a store (R5: a scheme loads from a user file):

```sh
shelfmark scheme compile schemes/cpc/cpc.json                 # preview
shelfmark scheme compile schemes/cpc/cpc.json --commit --expected-revision <rev>
```

`scheme:cpc@2026.08` is registered as a known scheme. Until it is
compiled, `classify --scheme scheme:cpc@2026.08` answers
`scheme_not_compiled` with that compile command as its next step.

The full scheme is 29 MB. In a store that holds it, every CLI call
currently costs about 6 s (measured 2026-10-08 on a 4-CPU box, release
build), because the store reloads it per call. For interactive use,
compile `cpc-main-group.json` instead (10,694 classes, 1.7 MB, about
0.5 s per call). Both builds use the same scheme ID. The main-group build
is a strict subset with identical labels, so pick one per store.

## What the build does

- **Classes:** every line of the title list, in its order. There are
  sections (`H`), classes (`H04`), subclasses (`H04L`), main groups
  (`H04L9/00`) and subgroups (`H04L9/0861`). `--depth` stops at
  `section`, `class`, `subclass`, `main-group` or `all`.
- **Hierarchy:** above main-group level, the symbol's prefix. A main
  group's parent is its subclass. A subgroup's parent is the nearest
  earlier group with one dot fewer. Only the dot level says this. A
  2000-series code can also sit inside another main group's run:
  `A01C2001/048` is under `A01C1/04`.
- **Code grammar:** `[A-HY]([0-9]{2}([A-Z]([0-9]{1,4}/[0-9]{2,6})?)?)?`.
  The builder rejects a symbol that does not match it, a duplicate, an
  empty title, or a subgroup with no open parent.
- **Labels:** the official titles, unchanged except that runs of
  whitespace collapse to one space. Braces `{...}` are kept: in CPC they
  mark text CPC adds to the IPC.
- **Version and provenance:** the version is the release (`2026.08`).
  `source.description` names the archive, its URL and its sha256. The
  lock adds the fetch date and size.
- **Vocabulary links:** none. The CPC sources (scheme, definitions,
  concordances) map to the IPC and to other CPC symbols, never to LCC or
  LCSH. The build adds no links of its own.

## Limits

- **No ranges, so unknown codes place coarsely.** Shelfmark's shelf-order
  key (`crates/scheme/src/code.rs`) reads every digit run after the first
  as a decimal fraction and drops its trailing zeros. That is right for
  LCC, but in CPC it makes main group `A01B10` key like `A01B1`.
  Synthesized subgroup ranges would then cover the wrong codes, so none
  are emitted. An exact symbol always resolves. A symbol the release
  lacks normally lands on its subclass (`H04L9/0643` lands on `H04L`).
- **The same key collision still applies without ranges.** An absent main
  group whose number is another's times ten keys like that one: absent
  `H04L90/00` lands on `H04L9/00`. A CPC-aware key in the scheme crate
  would fix both. `tests/scheme.rs` asserts the current behaviour, so the
  fix will flag this section.

## Tests

```sh
cargo test                                          # builder units + excerpt scheme tests; one logged skip
CPC_SOURCES=/tmp/cpc-sources cargo test --release   # also builds the locked release and checks its hashes
```

`tests/fixtures/titles-excerpt.txt` holds 42 synthetic lines in the shape of the 2026.08 title
list (real CPC codes and the same markup, invented labels; no CPC title text is
committed). The tests check that the excerpt's scheme validates against
`schemas/scheme.schema.json`, round-trips through `SchemeDoc`, passes the
scheme compiler, and browses, searches and checks codes as expected.
Without `CPC_SOURCES`, the full-release test logs a `SKIP` line to
`$TEST_SKIP_LOG`. With `CPC_REQUIRE_SOURCES=1`, that skip fails instead.
