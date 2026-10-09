# Free Decimal Correspondence (FDC)

`fdc.json` is scheme `fdc@0.06`: John Mark Ockerbloom's Free Decimal
Correspondence, a Dewey-shaped set of decimal numbers 000 to 999.9999 for
subjects, as a shelfmark scheme file. It is not compiled into the binary;
load it like any user scheme:

```sh
shelfmark scheme validate --path schemes/fdc/fdc.json
shelfmark scheme compile schemes/fdc/fdc.json   # preview; prints the commit command
shelfmark scheme browse --scheme scheme:fdc@0.06 970
shelfmark scheme search --scheme scheme:fdc@0.06 "incunabula"
```

## License

CC0 1.0. Checked 2026-10-09 against the primary sources:

- the header of `fdc.txt` ("The free decimal correspondence is dedicated to
  the public domain. The CC0 dedication applies to this work."):
  <https://raw.githubusercontent.com/JohnMarkOckerbloom/fdc/4d247249c4e2be7cbe13baf84f16fa939b084e0a/fdc.txt>
- the repository's `LICENSE` (CC0 1.0 Universal; GitHub reports `CC0-1.0`):
  <https://github.com/JohnMarkOckerbloom/fdc>
- the FDC page ("dedicated to the public domain"):
  <https://everybodyslibraries.com/free-decimal-correspondence/>

So the data file is committed and may ship in the binary or a package. No
OCLC Dewey data is used; FDC was written from the 1922 11th edition of the
Dewey classification (out of copyright) and checked only for compatibility.
The scheme has been frozen since v0.06 (2010-08-19); the author says it has
no long-term maintenance, and `fdc.txt` last changed in 2015.

## Shape

- 1,097 classes, 6 levels deep. Classes per level: 10, 166, 648, 149, 69,
  55.
- Branching: max 18 children (`200`), mean 7.25 over the 150 classes that
  have children.
- 10 hundreds-level roots (000 to 900), then tens classes, units classes
  and decimals (`971.23`). 23 classes are ranges, which the file uses to
  group numbers (`017-019 Library catalogs`); the numbers inside a range are
  its children, and the range sits under the tens class.
- Plain hundreds and tens classes carry their decimal span as a range
  (`010` covers 010-019), so a number the file leaves unassigned (013)
  resolves to the class that contains it.
- Code grammar: `[0-9]{3}(\.[0-9]+)?(-[0-9]{3}(\.[0-9]+)?)?`.
- Hierarchy comes from the numbers and ranges, not from the file's
  indentation, which is loose.
- Labels: the entry's first alternative (`|` separates two ways of saying
  one subject); several subjects on one number (`+`) are joined with `; `.
- Vocabulary: 940 terms, the other alternatives and the individual
  subjects. Terms the file quotes as authorized LCSH headings (760) have
  source `fdc-lcsh`, the rest `fdc`. LCSH heading ids are not in the file,
  so there are no `source_id`s.
- Changes to upstream, all listed in [SOURCES.md](SOURCES.md): three
  duplicated numbers that are plain typos (`975` Gulf States is 976; the
  Spanish `855` and `857` are 865 and 867), one same-number pair merged
  (`615.88`), and the range `940-990` widened to end at 999 because 991 to
  999 are listed inside it.

## Rebuild

`builder/` is a standalone crate (`fdc-scheme`) that downloads `fdc.txt`
from a pinned commit, locks its sha256 in `sources.lock.json`, and
regenerates `fdc.json` and `SOURCES.md` deterministically from the locked
file:

```sh
cd schemes/fdc/builder
cargo run --release -- fetch --cache /tmp/fdc-sources   # network; rewrites the lock
cargo run --release -- build --cache /tmp/fdc-sources   # offline
cargo run --release -- check --cache /tmp/fdc-sources   # fails if outputs are stale
cargo test && cargo clippy --all-targets -- -D warnings
```

`cargo test` validates the committed scheme against the published schema
and the scheme compiler, and checks browse, code lookup and search.
