# ILC2 scheme (`ilc2`)

Builds a shelfmark scheme file from the
[Integrative Levels Classification, edition 2](https://www.iskoi.org/ilc/)
(ILC2, ISKO Italy, Claudio Gnoli and contributors): a universal, freely
faceted scheme that orders phenomena by integrative levels. The scheme is
**not bundled**: each user fetches the source and builds the file locally.

## License

Unsettled, so the data is not redistributed here. Checked 2026-10-09:

- The published SKOS file itself declares
  `cc:license <http://creativecommons.org/licenses/by/3.0>` (CC BY 3.0) on
  the scheme, in [`ILC2.nt`](https://www.iskoi.org/ilc/2/skos/ILC2.nt)
  (and the `.ttl` and `.rdf` siblings).
- [BARTOC](https://bartoc.org/en/node/472) lists CC BY-SA 3.0.
- No iskoi.org page states a license:
  [the ILC page](https://www.iskoi.org/ilc/),
  [the SKOS page](https://www.iskoi.org/ilc/skos.php) and
  [the ILC2 schedule](https://www.iskoi.org/ilc/2/scheme) are silent.

The file's own statement is the primary source and would allow committing
the data, but it disagrees with BARTOC (share-alike or not), and the site
says nothing. So this directory commits only the tool, a lock (URL, size,
sha256) and a synthetic test fixture. The fetched file and the built JSON
stay in the user's cache and `out/`, which git ignores. The built file's
`source.license` records the disagreement and says to treat it as CC BY-SA
3.0 if redistributed. If the publisher settles it as CC BY, the built
file (about 1 MB) could be committed like `ipc`.

## Use

```sh
cd schemes/ilc2
cargo run --release -- build --cache ~/.cache/ilc2 --fetch
shelfmark scheme compile out/ilc2.json          # preview; prints the commit command
shelfmark scheme compile out/ilc2.json --commit --expected-revision <rev>
```

`build --fetch` downloads `https://www.iskoi.org/ilc/2/skos/ILC2.nt`
(11 MB) when the cache lacks it. Either way the build refuses a file whose
sha256 differs from `sources.lock.json`, so one lock always builds the same
file (`check` rebuilds and compares). Maintainers run `fetch --cache DIR`
to refetch and rewrite the lock. The scheme version is the file's
`dct:issued` date, `2019-10-17`.

## What the scheme holds

`scheme:ilc2@2019-10-17`, 10,117 classes of two kinds, both kept:

- **Concepts (9,174).** The schedule, a tree by `skos:broader`. Code is the
  `skos:notation` (letters and digits, plus the two top signs `*` and `~`),
  label is `skos:prefLabel`.
- **Facet indicators (943).** ILC2's relationship facets (`0` "as for
  perspective", `mq94` "affected by disease"). They are classes of their
  own, a tree by `rdfs:subPropertyOf`, never flattened into the concepts.
  The scheme schema has no facet field, so each indicator's domain and
  range are data on the first line of its `note`:
  `facet: domain=mq range=mqad` (`*` = any class). A class is a facet
  indicator exactly when its note starts with `facet:`.
- **Vocabulary.** Each `skos:altLabel` is a link with source `ilc-alt`,
  weight 0.9 (4,659 labels in the file).
- **Notes.** One line each, when present: `note:` (`skos:note`), `scope:`
  (`skos:scopeNote`), `ddc:` (the 389 `skos:closeMatch` Dewey numbers),
  `related:` (`skos:related` codes). There is no LCC or LCSH mapping.
- **Not in the file.** The publisher left faceted combinations such as
  `x5y` and other minor classes out of the SKOS conversion, so they are
  composed by the user from the indicators, not listed.

### Gaps in the source, kept visible

- 365 classes name a parent (`broader` or `subPropertyOf`) that the file
  never defines (e.g. facet `8389` under `838`, `mq94` under `mq9`). They
  become roots, with a note line `undefined-parent: <code>`.
- A few classes have a second `prefLabel` (e.g. `aia`: "truth value" and
  "attributes of #aia"); the first is the label and the rest are `ilc-alt`
  links.
- Facet indicators with an empty label take the form
  `[facet <code> of <domain>]`; the build counts them.

## Shape

Measured on the built file (root = level 1; branching = children per class
that has children):

| Part | Classes | Roots | Depth | Max branching | Mean branching |
|---|---|---|---|---|---|
| Concepts | 9,174 | 65 | 12 | 213 | 4.85 |
| Facet indicators | 943 | 406 | 5 | 10 | 2.34 |
| All | 10,117 | 471 | 12 | 213 | 4.58 |

(The publisher's count of 10,845 "classes and facets" includes combined
classes that the SKOS file omits.)

## Known limits

- Exact lookup, browse, ancestors and search follow the explicit
  hierarchy. The shelf-order key in `crates/scheme` is built for LCC, so
  browse order across ILC2 notations (which sort letter by letter) is not
  ILC's own filing order.
- ILC2's value is in combining classes; a combined classmark is
  grammatical here only as far as `[A-Za-z0-9*~]+` goes, and `scheme
  validate` does not check facet composition against `domain`/`range`.

## Tests

```sh
cargo test                                      # parser, builder, fixture
ILC2_CACHE=~/.cache/ilc2 cargo test             # also the real, locked file
```

`tests/fixture.rs` builds a small synthetic N-Triples file in the
iskoi.org format (invented labels). It checks the scheme against the
published JSON Schema, `SchemeDoc` and the compiler, then exercises
facets-as-data, notes, the source-gap cases, browse, search, code checks
and the failure cases. `tests/real.rs` builds the locked file. Without
`ILC2_CACHE` it is skipped and logged to `TEST_SKIP_LOG`; with
`ILC2_REQUIRE_CACHE=1` that skip becomes a failure.
