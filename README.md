# shelfmark

An open-source, cross-platform, CPU-first materials classifier. It gives
every Work a permanent content-derived ID, places it into any
classification scheme supplied as a data file, and records each result as
an append-only placement claim with a confidence, a replayable trace, and
the hashes of its inputs.

Status: **v0.1 contracts.** The shared types every component builds against
are in `crates/core`; the scheme compiler, tiers, store, CLI and gRPC
surfaces are being built against them.

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

[`proto/shelfmark/v1/shelfmark.proto`](proto/shelfmark/v1/shelfmark.proto)
carries the same requests and envelopes over gRPC. A test keeps its enums
in sync with the core.

## Development

```sh
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

This workspace has no dependency outside its own directory. A test checks
that, and the directory can be split into its own repository unchanged.

License: MIT OR Apache-2.0.
