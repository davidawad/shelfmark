# UDC Summary scheme (`udc-summary`)

Builds a shelfmark scheme file from the
[UDC Summary](https://udcsummary.info/), the UDC Consortium's selection
of about 2,600 classes from the Universal Decimal Classification. The
scheme is **not bundled**: each user fetches the source and builds the
file locally.

## License

The UDC Summary is © UDC Consortium. Its site has said since 2026 that
the data is released under
[CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/); the
older linked-data page (udcdata.info, now offline) still says CC BY-SA
3.0. Checked 2026-10-08 on `udcsummary.info/about.htm`, `exports.htm` and
`php/index.php`.

- **Attribution required.** Cite: *Multilingual Universal Decimal
  Classification Summary (UDCC Publication No. 088) © UDC Consortium,
  2013. Released for use and distribution under the Creative Commons
  Attribution-NonCommercial 4.0 International license. UDC Consortium, PO
  Box 90407, 2509 LK The Hague, The Netherlands, mail@udcc.org.* The built
  file carries this in `source.license`.
- **May not ship in shelfmark's binaries.** Shelfmark is MIT/Apache, and
  anyone may sell or embed it. NonCommercial data inside the binary would
  restrict that. So this directory commits only the tool, a lock of URLs
  and sha256 hashes, and synthetic test fixtures. The fetched pages and
  the built JSON stay in the user's cache and `out/`, which git ignores.
  Don't commit them, and don't use the built file commercially.

## Use

```sh
cd schemes/udc-summary
cargo run --release -- build --cache ~/.cache/udc-summary --fetch
shelfmark scheme compile out/udc-summary.json          # preview; prints the commit command
shelfmark scheme compile out/udc-summary.json --commit --expected-revision <rev>
```

`build --fetch` downloads the ten English tree pages
(`https://udcsummary.info/php/index.php?tag=<X>&lang=en`, one second
apart) into the cache. Without `--fetch` the build is offline. Either
way it refuses any page whose sha256 differs from `sources.lock.json`, so
one lock always builds the same file (`check` rebuilds and compares).
Maintainers run `fetch --cache DIR` to refetch and rewrite the lock. The
lock's `edition` is the scheme version and stays the same while the
hashes do.

## What the scheme holds

`scheme:udc-summary@<edition>`, 2,652 classes at edition 2026-10-08:

- **Classes.** The common auxiliary numbers (Tables 1c-1k: language
  `=`, form `(0...)`, place `(1/9)`, race and nationality `(=...)`, time
  `"..."`, general characteristics `-0...`) and the main classes 0-9 (4 is
  vacant). The site's hierarchy is kept, and captions are verbatim.
- **Left out.** The page header "COMMON AUXILIARY NUMBERS" is a
  grouping, so its tables become top-level classes. The seven common
  auxiliary signs (`+ / : :: [] * A/Z`) join numbers rather than name
  subjects, so they are not fetched.
- **Code grammar.** A digit or an auxiliary opener (`( = " -`), then
  UDC notation characters. Compound numbers such as `004.738:37(4)"19"`
  are grammatical.
- **Vocabulary.** The UDC Summary publishes no LCC or LCSH mappings: its
  mappings section is marked "under development". So there are no
  `lcc`/`lcsh` links. A caption is a list of equivalent terms
  (`Computer science and technology. Computing. Data processing`), and the
  lexical tier matches whole labels, so each term of a multi-term caption
  is also a link with source `udc-caption` and weight 0.9. That gives
  1,140 classes extra terms.
- **Provenance.** `source.description` names every page's URL and
  sha256 and one `s256:` digest of the whole lock.
- **No ranges and no notes.** Span codes like `004.01/.08` are classes
  with their own captions. Scope notes would need one request per class
  (`getrecord.php`), about 2,600 requests, so they are not fetched.

## Known limits

- Exact lookup, browse, ancestors and search follow the explicit
  hierarchy and are exact. The shelf-order key in `crates/scheme`
  (`code.rs`) is built for LCC: it ignores punctuation and reads `004` as
  `4`. So browse order mixes auxiliaries with main numbers, and a
  `within` answer from `scheme validate` for a code that is not a class
  is approximate (`004.738:37(4)` reports `within 004.738`).
- With the lexical profile, text that matches no caption term is flagged
  at confidence 0.

## Tests

```sh
cargo test                                         # parser, builder, fixtures
UDC_SUMMARY_CACHE=~/.cache/udc-summary cargo test  # also the real, locked pages
```

`tests/fixture.rs` builds two synthetic pages in the site's format
(UDC-shaped codes, invented captions). It checks the scheme against the
published JSON Schema, `SchemeDoc` and the compiler, then exercises
browse, search, code checks and the failure cases. `tests/real.rs`
builds the locked pages. Without `UDC_SUMMARY_CACHE` it is skipped and
logged to `TEST_SKIP_LOG`; with `UDC_SUMMARY_REQUIRE_CACHE=1` that skip
becomes a failure.
