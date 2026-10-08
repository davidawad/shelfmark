# IPC: International Patent Classification

> `ipc.json` (12 MB) is not committed because it exceeds the repo size limit. Build it with the builder below (`fetch`, then `build`); the output is byte-for-byte reproducible from `sources.lock.json`.

`ipc.json` is the IPC as a shelfmark scheme: `scheme:ipc@2026.01`, 80,145
classes (8 sections, 132 classes, 655 subclasses, 7,668 main groups and
71,682 subgroups, including indexing-scheme entries). It validates against
`src/shelfmark/schemas/scheme.schema.json`. The data comes from WIPO's
2026.01 master files. `SOURCES.md` records the URLs, sha256 hashes, license
and build counts.

```sh
shelfmark scheme validate --path src/shelfmark/schemes/ipc/ipc.json
shelfmark scheme compile src/shelfmark/schemes/ipc/ipc.json   # preview; add --commit --expected-revision <rev>
shelfmark scheme browse H01B1/00 --scheme scheme:ipc@2026.01
shelfmark scheme search "superconductive cable" --scheme scheme:ipc@2026.01
```

## Codes

A code is the printed IPC symbol with its space removed, because shelfmark
codes cannot contain whitespace: `A01B 1/02` becomes `A01B1/02`, and the
class ID is `class:ipc@2026.01:A01B1/02`. Sections (`H`), classes (`H01`)
and subclasses (`H01B`) stay as printed. Each class's parent comes from the
master file's nesting, so subgroups sit under their dot-level parent
(`H01B1/08` → `H01B1/06` → `H01B1/00` → `H01B` → `H01` → `H`). Labels are
WIPO's titles verbatim. A title made of several parts is joined with `; `,
and title images are written `[image]`. The cross-references in a title
(`entryReference`) go into the class's `note`, so they don't pull other
topics into label search.

## Vocabulary

- `ipc-catchword`: 23,567 links from WIPO's catchword index (weight 1,
  `source_id` is the catchword entry ID). A catchword that points at a range
  of groups links to each top-level group in the range.
- `ipc-heading`: 3,365 links from guidance headings ("Ploughs") to every
  main group under them (weight 0.5).

The IPC sources have no links to LCC or LCSH, so the scheme carries none.

## License

Under WIPO's Open Access Policy the master files are licensed CC BY 4.0
(see `SOURCES.md`), so they can ship inside an open-source repository or
binary as long as the attribution is kept:

> International Patent Classification (IPC), World Intellectual Property
> Organization (WIPO), edition 20260101. Licensed CC BY 4.0. Reformatted by
> shelfmark; not endorsed by WIPO.

The license allows bundling, so no compile-time downloader is needed. The
scheme is still loaded from its file at run time, not compiled into the
binary (plan R5). Its entry in `../registry.json` records this.

## Rebuild

`builder/` is a standalone Rust crate. It is not in the shelfmark workspace
and has its own `Cargo.lock`.

```sh
cd src/shelfmark/schemes/ipc/builder
cargo run --release -- fetch --cache /tmp/ipc-sources   # network; rewrites sources.lock.json
cargo run --release -- build --cache /tmp/ipc-sources   # offline; same lock, same bytes
cargo run --release -- check --cache /tmp/ipc-sources   # fails if ipc.json or SOURCES.md is stale
cargo test --release                                    # this scheme's tests
```

`fetch --edition 20270101` pins a later edition. WIPO publishes each
edition in advance; 2027.01 takes effect on 1 January 2027.

## Known limitation: main-group order

`shelfmark-scheme` sorts codes by LCC shelf order, which treats every digit
run after the first as a decimal fraction. For IPC this means:

- `browse` lists a subclass's main groups in string order: `A01B1/00`,
  `A01B11/00`, …, `A01B3/00`, `A01B31/00`. Subgroups within a main group
  come out in the right order.
- A code that is **not** in the scheme can be placed under the wrong class
  when its main group is ten times an existing one. For example,
  `check_code("A01B10/02")` returns `Within A01B1/02`. Codes that are in the
  scheme always resolve exactly.

The fix belongs in `crates/scheme`, which needs a scheme-aware order where
the main group counts as a whole number. This port does not change it.
