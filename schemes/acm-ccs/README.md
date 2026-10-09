# ACM Computing Classification System 2012

`acm-ccs` builds the [2012 ACM Computing Classification System](https://dl.acm.org/ccs)
as shelfmark scheme `scheme:acm-ccs@2012` from ACM's official SKOS file. The data is
**not bundled**: the builder downloads it, checks it against `sources.lock.json`, and
writes the scheme locally.

## License check

- **Owner and terms.** The CCS is © ACM, Inc. ACM's CCS page says that "the full CCS
  classification tree is freely available for educational and research purposes", and
  the DL pages carry "Copyright © ACM, Inc." The SKOS file has no license statement, and
  ACM publishes no open license or redistribution grant for the CCS.
  (Sources checked 2026-10-08: [acm.org/publications/class-2012](https://www.acm.org/publications/class-2012)
  and [dl.acm.org/ccs](https://dl.acm.org/ccs), both read through the Wayback Machine
  because the live pages sit behind a browser challenge.)
- **Attribution.** Credit "ACM Computing Classification System (2012), Association for
  Computing Machinery, https://dl.acm.org/ccs". The built scheme carries this in
  `source.license`.
- **Can it ship inside an open-source binary?** No. Use is granted only for education
  and research, so redistributing it under MIT/Apache-2.0 is not permitted. Nothing
  derived from the ACM file is committed here. The test fixture
  (`tests/fixtures/mini-ccs.xml`) is synthetic, with invented concepts and labels.
- **What ships instead.** The builder is a downloader that runs at scheme-compile time.
  Each user fetches the file from ACM under ACM's terms and builds the scheme
  on their own machine.

## Build

```sh
cd schemes/acm-ccs
cargo run --release -- build          # downloads into cache/ if needed; writes out/acm-ccs.json
shelfmark scheme compile out/acm-ccs.json --commit --expected-revision <revision from the preview>
```

- `build --offline` refuses to touch the network.
- `--cache DIR` and `--out FILE` move the cache and the output. `cache/` and `out/` are
  git-ignored.
- `fetch` only downloads.
- The build is deterministic and re-runnable. It refuses any file whose sha256 differs
  from the lock, so the same lock always writes the same bytes.
- If ACM publishes a changed file, `fetch --relock` accepts it and rewrites the lock.
  Review the scheme diff before committing the new lock.

## The scheme

| | |
|---|---|
| name, version | `acm-ccs`, `2012` (`class:acm-ccs@2012:<code>`) |
| classes | 2113: 13 top-level, depth up to 6 |
| code | the concept's dotted path of 8-digit ACM concept IDs, e.g. `10010147.10010257`; grammar `[0-9]{8}(\.[0-9]{8})*` |
| hierarchy | parent = the path minus its last segment (checked against `skos:broader`/`narrower`) |
| labels | the English `skos:prefLabel`, whitespace-normalized |
| poly-hierarchy | 159 concepts are filed under more than one parent. Each path is its own class, and its `note` names the other paths |
| vocabulary links | none. The source maps no concept to LCC or LCSH, so none are invented |
| provenance | `source.url` is the download URL. `source.description` names the file, size, sha256, Last-Modified and the CCS page |

The codes are the paths ACM's CCSXML index terms use (`<concept_id>`), so a paper's
CCS block maps directly to classes. 1998-CCS codes such as `I.2.6` are not 2012
codes and fail `scheme validate`.

The builder rejects any SKOS element it does not model. If a later ACM release adds
mappings (for example `skos:exactMatch`), the build fails instead of silently
dropping them.

## Tests

```sh
cargo test                                         # synthetic fixture; official tests skip
ACM_CCS_CACHE=cache ACM_CCS_REQUIRE=1 cargo test   # also the real file (run `fetch` first)
```

`tests/fixture.rs` checks the builder on the synthetic file: schema validation,
the compiler's checks, browse, search and code validation, determinism, the
poly-hierarchy notes, and rejection of tampered or malformed sources.
`tests/official.rs` checks the real file's shape (2113 classes, 13 top-level, depth 6)
and that browse, search and validate work on it. Without the file it skips and logs
to `TEST_SKIP_LOG`.
