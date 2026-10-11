# shelfmark

An open-source, cross-platform, CPU-first materials classifier. It gives
every Work a permanent content-derived ID, places it into any
classification scheme supplied as a data file, and records each result as
an append-only placement claim with a confidence, a replayable trace, and
the hashes of its inputs.

Status: **v0.1.** The shared types are in `crates/core`; the scheme
compiler, the lexical tiers, the store, the CLI and gRPC are built on them.
The bundled scheme is the LCC outline with LCSH vocabulary
(`data/schemes/lcc/`). The `shortlist` profile's embedding tier
(BGE-small-en-v1.5 int8 through ONNX Runtime) is in builds with feature
`semantic-shortlist`; the `rerank` tier is still a stub. The store records
claims in an append-only log and also holds near-duplicate links, gold
labels and the fitted calibration. Inputs are `.txt`, `.md`, `.org`,
`.html` and, with the `pdf` feature, `.pdf`. `shelfmark eval` measures
every runnable profile against the gold set and fits the calibration. Not
yet built: the rerank tier and the FAST scheme.

## Profiles

| Profile | Tiers | Build | Peak RSS | Per item |
|---|---|---|---|---|
| `lexical` | identifier, vocab, rules | default | ~100 MB | under 50 ms |
| `openlibrary` | identifier, Open Library title+author lookup, vocab, rules | `--features openlibrary` (rustc 1.85+) | ~100 MB (estimate: lexical plus a few MB) | ~1 s per new lookup (paced); a cached one adds under 1 ms |
| `shortlist` | lexical, then an embedding shortlist | `--features semantic-shortlist` | ~300-360 MB | ~50-100 ms more |
| `rerank` | shortlist, then a cross-encoder | not available yet | | |

Measured on a 4-CPU Linux box, one item per process with a warm store
cache; process start-up is about 0.2 s. `shelfmark status` reports which
profiles this build and machine can run. The shortlist model (~65 MB)
downloads on first use into the user's cache (`FASTEMBED_CACHE_DIR`
overrides it). Each store keeps derived data in `<store>/cache/`
(compiled schemes, vocab matchers, label vectors); it is safe to delete.

The built-in profiles are at version 2 (`profile:lexical@2`). The `vocab`
tier (`vocab-2`) ignores class labels and vocabulary terms made only of
English stopwords, such as a class labelled `To`. Version 1 matched them,
so one such class could take most placements. Labels with at least one
other word, such as `History of the arts`, still match in full. Placements
made under version 1 report as stale.

### The `openlibrary` profile

`openlibrary` runs the identifier tier, then looks the book up on
[Open Library](https://openlibrary.org/dev/docs/api/search) by title and
author, then falls back to the vocabulary and rules tiers. The lookup
(`crates/tier-openlibrary`) sends
`search.json?title=<title>&author=<first author's family name>`, picks
the best match by normalized title and author similarity (threshold
0.8), reads the matched work's LC call numbers (`lcc`, and raw
`lc_classifications` when present; `ddc` is read but unused), and places
each in the deepest class of the active LCC scheme (`lcc` or `lcc-full`)
that covers it. A class's confidence is the match score scaled by its
share of the work's call numbers, and its evidence names the Open
Library work key and the call number.

It is the only profile that sends network requests, so it never runs
unless named: it is not recommended, and `eval` skips it unless given
`--profile openlibrary`. It needs a build with the feature:

```sh
cargo build --release --features openlibrary
shelfmark classify --profile openlibrary --title 'The Selfish Gene' --author 'Richard Dawkins'
```

Requests carry a User-Agent naming this repository, go out at most one
a second, time out after 10 s, and are retried at most twice (timeouts,
429, 5xx). Responses are cached in `<store>/cache/openlibrary/` with no
expiry, so a repeated lookup sends nothing; delete the directory to
refresh. When Open Library cannot be reached the tier is recorded
`unavailable` and the cascade carries on with the local tiers. Open
Library data is CC0.

Measured on 50 well-known books, classifying from title and author alone
into the bundled outline (list and method in
[`crates/tier-openlibrary/eval/`](crates/tier-openlibrary/eval/)):

| Profile | Class hit | Subclass hit | Decided (not flagged) |
|---|---|---|---|
| `lexical` | 0/50 (0%) | 3/50 (6%) | 0 |
| `openlibrary` | 45/50 (90%) | 45/50 (90%) | 43, all correct |

## Scheme ports

Schemes beyond the bundled LCC each live in `schemes/<name>/`, with a
builder, tests and a README. Load the built file with
`shelfmark scheme compile <file>`.

| Scheme | Directory | Source license | Bundled | Official page |
|---|---|---|---|---|
| `lcc` (LCC outline + LCSH links) | [`data/schemes/lcc/`](data/schemes/lcc/) | US government work, no US copyright | yes: data file committed and pre-compiled | <https://www.loc.gov/catdir/cpso/lcco/> |
| `lcc-full` (LCC complete schedules, 416,186 classes) | [`schemes/lcc-full/`](schemes/lcc-full/) | US government work, no US copyright | data file committed (`lcc-full.json.gz`, 7.5 MB) | <https://www.loc.gov/aba/publications/FreeLCC/freelcc.html> |
| `fast` (FAST topical) | [`schemes/fast/`](schemes/fast/) | ODC-By 1.0 | no: allowed, but at 79 MB it is built locally | <https://www.oclc.org/research/areas/data-science/fast.html> |
| `udc-summary` (UDC Summary) | [`schemes/udc-summary/`](schemes/udc-summary/) | CC BY-NC 4.0 | no: each user fetches and builds it | <https://udcsummary.info/> |
| `ipc` (IPC 2026.01) | [`schemes/ipc/`](schemes/ipc/) | CC BY 4.0 | data file committed (`ipc.json`) | <https://www.wipo.int/en/web/classification-ipc> |
| `cpc` (CPC 2026.08) | [`schemes/cpc/`](schemes/cpc/) | none; EPO website terms apply | no: each user fetches and builds it | <https://www.cooperativepatentclassification.org/home> |
| `msc` (MSC2020) | [`schemes/msc/`](schemes/msc/) | CC BY-NC-SA 4.0 | no: each user fetches and builds it | <https://msc2020.org/> |
| `acm-ccs` (ACM CCS 2012) | [`schemes/acm-ccs/`](schemes/acm-ccs/) | ACM copyright, not redistributable | no: each user fetches and builds it | <https://dl.acm.org/ccs> |
| `ilc2` (ILC2) | [`schemes/ilc2/`](schemes/ilc2/) | unsettled: CC BY 3.0 in the SKOS file, CC BY-SA 3.0 on BARTOC, none on the site | no: each user fetches and builds it | <https://www.iskoi.org/ilc/> |
| `jel` (JEL) | [`schemes/jel/`](schemes/jel/) | AEA, all rights reserved | no: each user fetches and builds it | <https://www.aeaweb.org/econlit/jelCodes.php> |
| `arxiv` (arXiv categories) | [`schemes/arxiv/`](schemes/arxiv/) | MIT (arxiv-base) | data file committed (`arxiv.json`) | <https://arxiv.org/category_taxonomy> |
| `bk` (Basisklassifikation) | [`schemes/bk/`](schemes/bk/) | CC0 1.0 | data file committed (`bk.json`) | <https://wiki.k10plus.de/spaces/K10PLUS/pages/118063148/Basisklassifikation> |
| `naics` (NAICS 2022) | [`schemes/naics/`](schemes/naics/) | US government work, no US copyright | data file committed (`naics.json`) | <https://www.census.gov/naics/> |
| `openalex-topics` (OpenAlex Topics) | [`schemes/openalex-topics/`](schemes/openalex-topics/) | CC0 1.0 | data file committed (`openalex-topics.json`, about 13 MB) | <https://docs.openalex.org/api-entities/topics> |
| `fdc` (Free Decimal Correspondence 0.06) | [`schemes/fdc/`](schemes/fdc/) | CC0 1.0 | data file committed (`fdc.json`) | <https://everybodyslibraries.com/free-decimal-correspondence/> |
| `eurovoc` (EuroVoc 4.24) | [`schemes/eurovoc/`](schemes/eurovoc/) | EU reuse terms unsettled (see crate README) | no: each user fetches and builds it | <https://op.europa.eu/en/web/eu-vocabularies/th-dataset/-/resource/dataset/eurovoc> |
| `unesco-thesaurus` (UNESCO Thesaurus 2026-08-31) | [`schemes/unesco-thesaurus/`](schemes/unesco-thesaurus/) | CC BY-SA 3.0 IGO | data file committed (`unesco-thesaurus.json`); not bundled into binaries (share-alike) | <https://vocabularies.unesco.org/> |

## The tower

| Layer | Noun | ID form |
|---|---|---|
| L0 | Work, Item | `work:nt1s256:<hex>`, `item:s256:<hex>` |
| L1 | Scheme, Class | `scheme:lcc@2024`, `class:lcc@2024:QA76.76.C65` |
| L2 | Profile, Tier | `profile:lexical@2` |
| L3 | Placement | `placement:s256:<hex>` |

Every ID names its kind, and a hash ID names its algorithm, so one
`resolve` verb accepts any of them.

## Contracts (`crates/core`)

- `id`: typed IDs and their derivation. A Work ID is the SHA-256 of the
  text under normalization v1 (lowercase; runs of non-alphanumerics become
  one space), so a PDF and its markdown conversion share a Work.
- `scheme`: the scheme data model, mirrored by
  [`schemas/scheme.schema.json`](schemas/scheme.schema.json), and the
  `CompiledIndex` trait that compiled schemes implement.
- `tier`: the `Tier` trait, classify inputs, candidates, profiles.
- `cascade`: tiers run in profile order. The first tier that reaches the
  target confidence (with the profile's minimum margin) decides.
  Otherwise the top-ranked class is taken and flagged. Ties break by code.
- `placement`: claims, traces, input hashes and memo keys, the status
  lattice (a human confirmation outranks any machine claim), and the
  `PlacementLog` trait.
- `error`, `envelope`: stable error codes, each with a next command;
  `{"ok": true, "data": ...}` / `{"ok": false, "error": ...}`; the
  `Service` trait the CLI and gRPC share; NDJSON batch lines with resume.
- `testkit` (feature): an in-memory index and a fixed-score tier for
  tests.

## Eval and calibration (`crates/eval`)

`shelfmark eval` re-runs each runnable profile on the gold set and
reports top-1/top-k accuracy, calibration error (ECE), flagged rate and
latency per profile, per deciding tier and per hierarchy level (R16). It
fits a per-tier calibration from the same run: raw scores in 10 bins,
made monotone by pool-adjacent-violators (plan PD3). Like any mutation it
previews first. `--commit` writes `calibration.json` and `eval.json` to
the store; `classify` then applies the calibration and `status` reports
both, including whether the calibration is stale against newer gold
(R17, R23). Gold comes from `confirm`/`correct`, which label the input the
Work was classified from, or from `--gold FILE`, an NDJSON file of gold
examples or mc-eval gold items, which a commit adds to the gold set once.
The metrics are in-sample: the calibration is scored on the examples it
was fitted on.

```sh
shelfmark eval --gold starter-v1.ndjson            # preview
shelfmark eval --gold starter-v1.ndjson --commit --expected-revision <rev>
```

[`proto/shelfmark/v1/shelfmark.proto`](proto/shelfmark/v1/shelfmark.proto)
carries the same requests and envelopes over gRPC. A test keeps its enums
in sync with the core.

## Development

```sh
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --features semantic-shortlist   # needs rustc 1.88+
```

`.cargo/config.toml` builds x86_64 for `x86-64-v3` (AVX2, 2013+ CPUs).

This workspace has no dependency outside its own directory. A test checks
that, and the directory can be split into its own repository unchanged.

License: MIT OR Apache-2.0.

Repository: https://gitlab.com/davidawad/shelfmark (a GitHub mirror may also exist).

## Gold set sources and attribution

The starter gold set (`research/eval/gold/starter-v1.ndjson`, 104 items) is
built only from public sources, each item recording its source and license:
arXiv metadata (CC0), Open Library records (CC0), and Wikipedia article
summaries. The roughly 30 Wikipedia-derived items are licensed CC BY-SA 4.0
(https://creativecommons.org/licenses/by-sa/4.0/); the article title and URL
recorded on each item are the attribution, and reuse of those items must keep
the same license and attribution. See `research/eval/README.md` and
`research/eval/gold/manifest.json`.
