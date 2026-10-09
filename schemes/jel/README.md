# JEL classification scheme (economics)

`jel-scheme` builds the [JEL Classification System](https://www.aeaweb.org/econlit/jelCodes.php)
(the *Journal of Economic Literature* codes, maintained by the American
Economic Association) as shelfmark scheme `scheme:jel@2026`: 20 categories,
their subcategories and codes, 1015 classes in all (A, A1, A10, ...).

## License: not bundled

Checked 2026-10-08:

- The JEL page and its XML say "Copyright 2026 American Economic Association.
  All rights reserved." No open license is published for the codes.
- The [aeaweb.org terms of use](https://www.aeaweb.org/terms-of-service/site)
  grant only "a limited, revocable, non-exclusive and non-transferable
  license" to download and reproduce site content "for personal purposes".
  They grant no right to redistribute it.
- Attribution: credit the JEL Classification System, American Economic
  Association, with a link to the JEL page.

So the data **may not ship inside shelfmark's MIT/Apache binary or this
repository**. Only the builder, the source lock (URL, size, sha256) and a
synthetic test fixture are committed. Each user fetches the AEA's XML and
generates `jel.json` on their own machine, and `.gitignore` keeps the
generated file out of git. The same terms are recorded in `sources.lock.json`
(`"redistributable": false`), and the build copies them into the scheme's
`source.license`. If the AEA grants redistribution, set `redistributable` to
true and bundle the file the way `data/schemes/lcc/` is bundled.

## Build

```sh
cd schemes/jel
cargo run --release -- build --cache /tmp/jel-sources   # fetches if missing; writes jel.json
cargo run --release -- check --cache /tmp/jel-sources   # fails if jel.json is stale
shelfmark scheme compile schemes/jel/jel.json            # from the repo root; preview, then --commit
```

`build` downloads `classificationTree.xml` into the cache when it is absent.
It refuses any file whose sha256 or size differs from `sources.lock.json`, so
the same lock always produces byte-identical output, and a cached source
needs no network. If the AEA changes the file, `build` fails and names both
hashes. Review the change, then run `cargo run -- fetch --cache DIR --relock`,
and bump `version` in the lock if any codes changed (the version is part of
every class ID).

## What the scheme holds

- **Classes and hierarchy:** every `<classification>` in the XML, in document
  order. The parent comes from the nesting. The builder checks that each code
  extends its parent's code and has one character per level.
- **Code grammar:** `[A-Z]([0-9]{1,2})?`.
- **Labels:** the descriptions, with HTML entities decoded and the AEA's `•`
  topic separators written as `; `, as in the AEA's flat `classifications.xml`.
- **Provenance:** `source` records the XML URL, fetch date, size and sha256,
  and the license above.
- **Vocabulary:** none. The AEA source carries no LCC or LCSH links. The JEL
  Guide's per-code keywords are a possible later source, under the same
  license.

Classifying by labels alone is weak for JEL: many leaves are labelled
"General" or "Other", and their parents give them meaning. Browse, search and
code checks work well.

## Registration

`crates/cli/src/wiring/mod.rs` lists `scheme:jel@2026` as a known but
uncompiled scheme. Until the scheme is compiled, a request for it fails with
`scheme_not_compiled`, and the next command is
`shelfmark scheme compile schemes/jel/jel.json`.

## Tests

```sh
cargo test                                              # unit tests + synthetic fixture; no network
JEL_CACHE=/tmp/jel-sources JEL_REQUIRE_SOURCE=1 cargo test --test source
```

The `source` tests run against the real locked XML. Without `JEL_CACHE`, each
one appends a `SKIP:` line to `$TEST_SKIP_LOG`. With `JEL_REQUIRE_SOURCE=1`, a
missing source is a failure instead of a skip. Set `JEL_FETCH=1` to download
the source into an empty cache.
