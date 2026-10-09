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
| `shortlist` | lexical, then an embedding shortlist | `--features semantic-shortlist` | ~300-360 MB | ~50-100 ms more |
| `rerank` | shortlist, then a cross-encoder | not available yet | | |

Measured on a 4-CPU Linux box, one item per process with a warm store
cache; process start-up is about 0.2 s. `shelfmark status` reports which
profiles this build and machine can run. The shortlist model (~65 MB)
downloads on first use into the user's cache (`FASTEMBED_CACHE_DIR`
overrides it). Each store keeps derived data in `<store>/cache/`
(compiled schemes, vocab matchers, label vectors); it is safe to delete.

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
| L2 | Profile, Tier | `profile:lexical@1` |
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
