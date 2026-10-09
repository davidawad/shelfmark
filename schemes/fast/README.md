# FAST scheme (`scheme:fast@2024`)

[FAST](https://www.oclc.org/research/areas/data-science/fast.html) (Faceted
Application of Subject Terminology) is OCLC's simplified form of LCSH. This
directory builds its topical facet as a shelfmark scheme: 440,612 classes,
each a live FAST topical heading coded `fst` plus its eight-digit FAST
number (`fst01004795` is "Machine learning").

| file | what | committed |
|---|---|---|
| `src/`, `Cargo.toml` | `fast-scheme`, the builder (a standalone crate, not a workspace member) | yes |
| `sources.lock.json` | the source file's URL, pinned mirror, size and sha256 | yes |
| `SOURCES.md` | generated: sources, what the build kept, output hashes | yes |
| `fast-sample.json` | generated: `scheme:fast@2024-sample`, 1,486 classes around seven seed headings | yes |
| `out/fast.json` | generated: the full scheme, 79 MB | no |

## Build and use

```sh
cd schemes/fast
cargo run --release -- fetch --cache /tmp/fast-sources   # network, about 55 MB
cargo run --release -- build --cache /tmp/fast-sources   # offline, about 10 s
cd ../.. && shelfmark scheme compile schemes/fast/out/fast.json --commit --expected-revision <rev>
```

`build` reads only cached files whose sha256 matches the lock, so it is
deterministic. `check` rebuilds and fails if `out/fast.json`, the sample or
SOURCES.md differ. `fetch` tries OCLC's URL first. OCLC serves the files
behind a bot challenge that refuses scripted downloads (HTTP 403), so it then
tries the Internet Archive copy pinned in the lock. Either source must give
the locked bytes.

The full scheme is not committed and not compiled into the binary. It is
four times the size of the bundled LCC scheme gzipped (10 MB against 1.3 MB),
and the repository's commit gate refuses files over 2 MB. shelfmark lists
`scheme:fast@2024` as known but not compiled, with this file as the compile
command. Whether to bundle it, and how (a release asset, a smaller cut), is
an open decision. The license allows bundling either way.

## Rules

FAST is a list of headings, not a tree, so the parent is chosen by rule
(`src/build.rs`):

1. The lowest-numbered live `skos:broader` heading, unless it would make
   the class its own ancestor (147,272 classes; one link refused).
2. Otherwise, for a subdivided heading, the nearest live heading named by
   dropping trailing subdivisions. `Computers--History--20th century` goes
   under `Computers--History`, or `Computers` if that heading is missing
   (234,154 classes).
3. Otherwise none: 59,186 top-level classes.

Each class's vocabulary has three kinds of entry:

- its alternate labels (`source: fast`);
- the LCSH heading it is `schema:sameAs`, with its LCSH ID (`source: lcsh`,
  e.g. `sh85079324`). These are the source's links to LCSH. The FAST
  dump carries no LCC numbers;
- the labels of deprecated headings that `dcterms:isReplacedBy` it
  (`source: fast`, with the old code). The weight is split across the
  replacements.

The build drops deprecated headings as classes, along with links to GND,
BnF, Wikidata and Wikipedia and the other facets (personal, corporate and
geographic names, events, titles, form/genre, chronological). Exact
counts are in SOURCES.md.

## License

Step 1 of the port was the license check. The FAST data files are published
by OCLC under the [Open Data Commons Attribution License
(ODC-By) v1.0](https://www.oclc.org/research/activities/fast/odcby.html),
which also ships inside the source zip (`License02_15_2012.txt`). ODC-By lets
anyone copy, distribute and adapt the database, including in commercial or
open-source products, on one condition: attribution. So the data may ship
inside an open-source binary. Any public use of the scheme, and any binary
that ships it, must carry this notice, which the scheme's `source.description`
already does:

> This scheme contains information from FAST (Faceted Application of Subject
> Terminology) Data (https://www.oclc.org/research/areas/data-science/fast/download.html)
> which is made available by OCLC Online Computer Library Center, Inc. under
> the ODC Attribution License (https://www.oclc.org/research/activities/fast/odcby.html).

The builder code is MIT OR Apache-2.0 like the rest of shelfmark.

## Tests

```sh
cargo test --release                                     # rules on tests/data/mini.nt, the sample
FAST_SOURCES=/tmp/fast-sources cargo test --release      # plus the full rebuild and compile
```

Without `FAST_SOURCES` the full-build test skips: it says so on stderr and
appends a line to `$TEST_SKIP_LOG`. `FAST_REQUIRE_SOURCES=1` turns the skip
into a failure.
