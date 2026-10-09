# LCC outline with LCSH vocabulary: `lcc-lcsh.json.gz`

`lcc-lcsh.json.gz` is the scheme compiled into the `shelfmark` binary. It is
`lcc.json` (scheme `lcc@2024`, built by `tools/lcc-outline`, see `SOURCES.md`)
with the same 7,392 classes in the same order, plus vocabulary: every active
LCSH heading that carries an LCC number becomes a link on the class that holds
that number. The holder is the class itself or the deepest span covering the
number, as shelfmark's compiled index decides (`check_code`). A heading held by
several classes splits its weight evenly. `lcc.json`'s own vocabulary is kept.

The file is gzipped canonical JSON: the bytes are `serde_json::to_vec` of the
`SchemeDoc`, so the bundled scheme has the same content hash as the same
document compiled into a store. `crates/cli/tests/cache.rs` checks this, checks
that it validates, and checks that it is `lcc.json` plus vocabulary.

| | |
|---|---|
| headings with an LCC number | 91,580 |
| headings placed in a class | 91,499 (the rest carry only letter spans such as `D-F`) |
| LCC numbers / placed | 63,843 / 63,754 |
| terms added | 97,848 (preferred labels only) |
| classes with vocabulary | 3,647 |
| uncompressed / gzipped | 8,317,595 / 1,340,441 bytes |
| content hash | `s256:d6234e5562a05e5c58eb4c8cb2321b62b1e32143c22c9e5a36a436fb606e7200` |

## Inputs

- LCSH bulk MADS/RDF, `subjects.madsrdf.nt.gz` from id.loc.gov, sha256
  `f57e6dcf40d510e8be37ef25ba9e78f6f6768d24118563dd5898bc394596eebd`: the same
  file `sources.lock.json` locks for `lcc.json`.
- `lcc.json` as committed next to this file.

## Rebuild

The ingester and the enrichment step are the vocabulary spike's
(`research/vocab`, outside this workspace; moving them into
`tools/lcc-outline` is open work):

```sh
cd research/vocab && cargo build --release
B=target/release/vocab-ingest
$B lcsh --input subjects.madsrdf.nt.gz --out lcsh.tsv.gz --stats lcsh-stats.json
$B enrich-scheme --scheme ../../data/schemes/lcc/lcc.json \
  --table lcsh.tsv.gz --out lcc-lcsh.json --stats enrich-stats.json
```

Then set the title to "Library of Congress Classification outline with LCSH
vocabulary" and the source description to the one in the bundled file, write
the document back as canonical JSON (`serde_json::to_vec` of the parsed
`SchemeDoc`, no trailing newline), and compress it with `gzip -9n`.

Measured with `--alt true` (alternate and uninverted labels too, 250,859 terms,
2.45 MB gzipped) the lexical profile scored subclass top-1 36.5% against 34.6%
for preferred labels only, and class top-1 45.2% against 47.1%: a tie on the
104-item starter gold set. The preferred-label file is bundled because it is
under half the size and its vocab matcher loads in half the time.

## License

LCSH and LCC are published by the Library of Congress. As works of the US
government they are not subject to copyright in the United States (17 U.S.C.
105). Credit: Library of Congress Subject Headings and Library of Congress
Classification, Library of Congress, Policy, Training, and Cooperative Programs
Division.
