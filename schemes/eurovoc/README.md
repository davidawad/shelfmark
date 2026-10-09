# EuroVoc scheme

`eurovoc-scheme` builds [EuroVoc](https://op.europa.eu/en/web/eu-vocabularies/th-dataset/-/resource/dataset/eurovoc)
(the multilingual thesaurus of the Publications Office of the European Union),
version 4.24 (2026-07-08), as shelfmark scheme `scheme:eurovoc@4.24`, from the
Publications Office's SKOS-core distribution
(`eurovoc_in_skos_core_concepts.zip`, one RDF/XML file).

## Size and shape

Built from the locked source (checked 2026-10-09):

| | |
|---|---|
| Classes | 7,663 = 21 domains + 127 microthesauri + 7,515 descriptors |
| Depth | 8 levels (domain, microthesaurus, then descriptors down to 6 levels) |
| Level counts | 1: 21, 2: 127, 3: 551, 4: 4,054, 5: 2,364, 6: 487, 7: 53, 8: 6 |
| Leaves / internal | 5,983 / 1,680 |
| Branching (internal classes) | max 51 (`C5769`), mean 4.55 |
| Vocabulary links | 9,916 English alternative labels |
| Output | about 1.3 MB of JSON |

## License: not bundled

Checked 2026-10-09. The sources disagree, so the data is not committed:

- The dataset metadata on data.europa.eu, which lists the SKOS-core
  distribution, names the **European Commission reuse notice**
  (Decision 2011/833/EU, <https://data.europa.eu/eli/dec/2011/833/oj>; the
  server answers scripted requests with a challenge page, so the text was not
  fetched here). The Commission's legal notice
  (<https://commission.europa.eu/legal-notice_en>) says reuse is allowed with
  appropriate credit and an indication of changes.
- The Publications Office copyright notice
  (<https://op.europa.eu/en/web/about-us/legal-notices/publications-office-of-the-european-union-copyright>)
  licenses its editorial content under CC BY 4.0 but refers questions on
  reusing "the EU Vocabularies" to `op-copyright@publications.europa.eu`.
- The EuroVoc page on op.europa.eu states no license at all, and
  [BARTOC](https://bartoc.org/en/node/15) lists CC0 1.0 (a secondary record).

Because the primary sources do not settle it, this port uses the
fetch-and-build pattern. Only the builder, the source lock (URL, size,
sha256), a synthetic test fixture and this README are committed; each user
fetches the zip and generates `eurovoc.json` locally (`.gitignore` keeps it
out of git). `sources.lock.json` records `"redistributable": false`, and the
build copies the terms into `source.license`. If the Publications Office
confirms CC BY 4.0 or CC0, set it to true and bundle the file.

Attribution: EuroVoc, Publications Office of the European Union. Reformatted
by shelfmark; not endorsed by the Publications Office.

## Build

```sh
cd schemes/eurovoc
cargo run --release -- build --cache /tmp/eurovoc-sources   # fetches if missing; writes eurovoc.json
cargo run --release -- check --cache /tmp/eurovoc-sources   # fails if eurovoc.json is stale
shelfmark scheme compile schemes/eurovoc/eurovoc.json       # preview, then --commit
```

`build` downloads the zip (8.6 MB) into the cache when absent and refuses a
file whose sha256 or size differs from `sources.lock.json`, so the same lock
always produces byte-identical output. If the Publications Office publishes a
new release at the same URL, `build` fails and names both hashes; review the
change, run `cargo run -- fetch --cache DIR --relock`, and set `version` in the
lock to the new EuroVoc version (it is part of every class ID).

## What the scheme holds

- **Levels:** domain (21, e.g. `04` POLITICS), microthesaurus (127, e.g.
  `0406` political framework; its first two digits are its domain), then
  descriptors along `skos:broader`. Domain and microthesaurus labels are the
  source labels without the leading code.
- **Codes** (`[0-9]{2}|[0-9]{4}|C[0-9]+|c_[0-9a-f]+`): domains and
  microthesauri use their notation; a descriptor is `C` plus its numeric
  EuroVoc id (`C5263`). 473 descriptors have hash-style ids in the source
  (`c_58d943b0`); they keep that id as the code.
- **Labels:** the English `skos:prefLabel`. No other language is read.
- **Vocabulary:** every English `skos:altLabel` that differs from the label,
  as a link with source `eurovoc` and the descriptor's EuroVoc id as
  `source_id`.
- **Note:** the English scope note and the links the single parent leaves
  out (below). `skos:related`, definitions and history notes are not kept.

### Polyhierarchy rule

EuroVoc is polyhierarchical: 244 descriptors have several broader terms, and
a few hundred belong to several microthesauri. A shelfmark class has one
parent, so every descriptor is placed once:

1. A descriptor with no broader term is a top term; its parent is its
   microthesaurus (the lowest code, if it is in several).
2. Otherwise its parent is the broader descriptor that shares a
   microthesaurus with it, the lowest EuroVoc id among those (numeric ids
   first, then hash ids); if none shares one, the lowest id of all.
3. The other broader descriptors (`Also narrower than: label (code)`) and the
   other microthesauri of a top term (`Also a top term of: code label`) go in
   the note, so no link is lost.
4. Siblings are ordered by label, case-insensitively, then id.

The builder fails on a cycle among the chosen parents (descriptors left
unreachable), a `skos:broader` to a resource the file does not describe, a
missing or repeated English preferred label, a microthesaurus outside a known
domain, and a zip that holds other than one `.rdf` file.

## Tests

```sh
cargo test                                              # unit tests + synthetic fixture; no network
EUROVOC_CACHE=/tmp/eurovoc-sources EUROVOC_REQUIRE_SOURCE=1 cargo test
```

The `source` tests run against the real locked zip. Without `EUROVOC_CACHE`,
each one appends a `SKIP:` line to `$TEST_SKIP_LOG`. With
`EUROVOC_REQUIRE_SOURCE=1`, a missing source is a failure instead of a skip.
Set `EUROVOC_FETCH=1` to download the zip into an empty cache.

## Not done

`crates/cli/src/wiring/mod.rs` does not list `scheme:eurovoc@4.24` as a known
but uncompiled scheme (as it does for `jel`), so a request for it before
compiling gives the generic unknown-scheme error rather than the
`scheme_not_compiled` hint.
