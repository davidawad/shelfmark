# OpenAlex Topics scheme

`openalex-topics.json` is scheme `openalex-topics@2026-10-09`: OpenAlex's
topic hierarchy, domain > field > subfield > topic, as a shelfmark scheme
file. It validates against `schemas/scheme.schema.json`. It is generated
by `builder/` (`openalex-topics-scheme build`), so do not edit it by hand.
Official page: <https://docs.openalex.org/api-entities/topics>.

```sh
shelfmark scheme compile schemes/openalex-topics/openalex-topics.json   # preview, then --commit
```

## License

OpenAlex releases its data as a CC0 public-domain dedication. The pricing
page says so: "the data is a public good, released under a CC0
public-domain license"
(<https://help.openalex.org/hc/en-us/articles/24397762024087-Pricing>).
The terms are at <https://creativecommons.org/publicdomain/zero/1.0/>.

- **May it ship inside an open-source binary?** Yes. CC0 waives
  attribution and use restrictions, so the built file is committed. It is
  large (about 13 MB, 2.4 MB gzipped), so it is not compiled into the
  binary; load it with `shelfmark scheme compile`.
- The builder refuses to fetch or build if that page stops saying CC0
  and public-domain, so a change of terms stops a refresh until someone
  reviews it. The page itself is not hash-pinned, only checked.

## What is in it

- **Hierarchy:** domain (`D3`) > field (`F31`) > subfield (`S3106`) >
  topic (`T10346`). A code is the OpenAlex id's last path segment with a
  one-letter level prefix, so codes survive refreshes. Code grammar:
  `[DFST][0-9]+`.
- **Notes:** OpenAlex's description of each class (every class in this
  snapshot has one).
- **Vocabulary:** each topic's keywords (up to 25), weighted by OpenAlex's
  keyword score (source `openalex-keyword`, `source_id` is the keyword
  slug). Alternative names of domains, fields and subfields are
  vocabulary too (source `openalex-alt`). Terms are de-duplicated without
  regard to case, keeping the strongest weight; a term equal to its
  class label is dropped, as are non-positive scores.
- **No LCC or LCSH links:** the source provides none.
- **Consistency checks:** every parent exists, ids are unique, and each
  subfield's and topic's own field and domain match the ones reached
  through the hierarchy.

## Size of the scheme

| Measure | Value |
|---|---|
| Classes | 4,798 |
| Depth (levels) | 4 |
| Per level | 4 domains, 26 fields, 252 subfields, 4,516 topics |
| Branching, max | 224 children |
| Branching, mean over classes with children | 17.0 (domain 6.5, field 9.7, subfield 17.9) |
| Vocabulary links | 102,083 |

## Provenance and rebuild

OpenAlex serves only its current data, and the raw topic pages are about
16 MB, so the pages are not committed. `sources.lock.json` pins each
fetched page: URL, fetch date, size and sha256. The built file is the
committed record, and its `source.description` repeats every page hash.
The scheme version is the date the snapshot was fetched. Requests carry
the generic contact `mailto=shelfmark@example.org`, wait 300 ms between
pages and retry with backoff.

```sh
cd schemes/openalex-topics/builder
cargo run --release -- fetch --cache /tmp/openalex-cache   # network; refreshes the cache and the lock
cargo run --release -- build --cache /tmp/openalex-cache   # offline; writes openalex-topics.json
cargo run --release -- check --cache /tmp/openalex-cache   # fails if openalex-topics.json is stale
cargo test                                                 # needs no cache; checks the committed file
```

`build` and `check` read only cached pages whose sha256 matches the lock.
Run `fetch` and then `build` to move to a newer OpenAlex snapshot; expect
the topic list and keyword scores to drift between snapshots.
