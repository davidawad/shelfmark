# NAICS 2022

`naics.json` is scheme `naics@2022`: the United States edition of the
North American Industry Classification System, 2022, as a shelfmark scheme
file. It is not compiled into the binary; load it like any user scheme:

```sh
shelfmark scheme validate --path schemes/naics/naics.json
shelfmark scheme compile schemes/naics/naics.json   # preview; prints the commit command
shelfmark scheme compile schemes/naics/naics.json --commit --expected-revision <revision>
shelfmark scheme browse --scheme scheme:naics@2022 31-33
shelfmark scheme search --scheme scheme:naics@2022 "software publishers"
```

`shelfmark status` lists it as known but not compiled until then, with
this file as its source.

## Shape

- 2,125 classes: 20 sectors, 96 subsectors (3 digits), 308 industry
  groups (4), 689 NAICS industries (5) and 1,012 national industries (6).
- Each code sits under the code one digit shorter. Manufacturing, Retail
  Trade and Transportation and Warehousing are sector spans (`31-33`,
  `44-45`, `48-49`) with a range, and their subsectors sit under the span.
- Code grammar: `[0-9]{2}(-[0-9]{2}|[0-9]{1,4})?`.
- Labels are the official US titles (without the `T` trilateral marker).
- Vocabulary: about 20,000 entries from the official NAICS index file, as
  `naics-index` terms on their 6-digit industry, so "canola" finds
  111120 Oilseed (except Soybean) Farming.
- No LCC or LCSH links: NAICS publishes no crosswalk to either.

## License

US government work (OMB and the Census Bureau), not subject to copyright
in the United States (17 U.S.C. 105). It may ship inside an open-source
binary or package. Credit: "Source: U.S. Census Bureau, 2022 NAICS".
Details in [SOURCES.md](SOURCES.md).

## Rebuild

`builder/` is a standalone crate (`naics-scheme`) that downloads the
Census Bureau's 2-6 digit codes and index workbooks, locks their sha256 in
`sources.lock.json`, and regenerates `naics.json` and `SOURCES.md`
deterministically from the locked files. Commands are in
[SOURCES.md](SOURCES.md). `cargo test` in `builder/` validates the
committed scheme against the published schema and the scheme compiler, and
checks browse, code lookup and search.
