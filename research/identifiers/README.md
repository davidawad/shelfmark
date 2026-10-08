# identifier-tables

Data for the materials classifier's identifier tier (plan R7, AE1): the
exact-lookup step that runs before any vocabulary matching or model.

This is a standalone Cargo workspace with no dependency on the surrounding
repo, so it can be split out with the classifier.

| Path | What it is |
|---|---|
| `src/` | Deterministic builder: Open Library editions dump to an ISBN/LCCN/OCLC lookup table for LCC and DDC |
| `data/arxiv-lcc.tsv` | arXiv category to LCC map: all 155 current categories, the 6 arXiv aliases, 11 bare archives and 18 discontinued legacy archives |
| `data/arxiv-taxonomy.tsv` | Snapshot of https://arxiv.org/category_taxonomy (2026-10-07), used to check the map covers every category |
| `data/url-domain-rules.tsv` | Starter URL-domain to class rules (54 rules; `decide` or `hint`) |
| `samples/` | 79 real dump lines and their golden build output (tests compare against it) |
| `FINDINGS.md` | Coverage, size, conflict and license findings from the full 2026-09-30 dump |

## Build the table

```sh
cargo build --release
identifier-tables build ol_dump_editions_YYYY-MM-DD.txt.gz --out out/ --progress-every 10000000
identifier-tables check   # validate the arXiv map and the domain rules
```

`build` streams the gzipped dump (one reader thread, `--jobs` JSON parsers)
and writes three files. They are sorted and byte-identical for identical
input at any `--jobs`. Timings go to stderr only.

- `lookup.tsv`: `key  scheme  prefix  full  support`. There is one row per
  identifier and distinct class number. `key` is `isbn:<13 digits>` (ISBN-10s
  are converted), `lccn:<normalized>` or `oclc:<digits>`. `prefix` is the LCC
  class (`QA76.76`) or the DDC section (`005`). `full` is the normalized call
  or class number. `support` is the number of edition records asserting it.
- `conflicts.tsv`: `key  scheme  level  editions  candidates`. A key appears
  here when its class numbers disagree at the `prefix` level (LCC class /
  DDC section) or the `top` level (LCC subclass letters / DDC main class).
  `editions` is the number of classified editions carrying the key. When it
  is 1, Open Library merged several libraries' call numbers into one record.
- `stats.json`: field validity counts with the smallest rejected samples,
  edition coverage, per-key-kind coverage and conflict counts, provenance
  (`source_records` prefix) of classified editions, and table sizes.

### Normalization

- ISBN: separators are ignored and the check digit must verify. Output is
  ISBN-13.
- LCCN: follows the LoC normalization rules
  (https://www.loc.gov/marc/lccn-namespace.html).
- OCLC: `(OCoLC)`, `ocm`, `ocn` and `on` prefixes and leading zeros are
  dropped.
- LCC: whitespace is collapsed and the space before a Cutter's period is
  removed (`GE145 .A53` becomes `GE145.A53`). The value must start with
  an LCC subclass from the LC Classification Outline, followed by a class
  number, or be a bare subclass. Class K is accepted by pattern, because its
  jurisdiction tables are too many to list. LC's `LAW`, `MLC`/`MLCS` and
  `CPB` shelf marks, other local marks and NLM `W` numbers are rejected.
- DDC: prime marks and brackets are removed, and trailing edition numbers
  are dropped. A `/` directly after the three-digit section stands for the
  decimal point. `[E]`, `B`, `Fic` and similar marks are rejected.

Empty strings are skipped. Every other rejected value is counted, never
guessed.

## Rule files

Both files are `#`-commented TSV with a header row, and `check` validates
them. The tests also enforce that the arXiv map covers the taxonomy snapshot.
New-style arXiv IDs (`2101.00001`) do not carry a category, so resolving one
needs arXiv metadata (API or OAI-PMH). Old-style IDs (`hep-th/9901001`)
carry it, and `rules::arxiv_category_from_id` extracts it.

URL-domain rules match a host or any of its subdomains plus an optional path
prefix, and the most specific rule wins. A `decide` rule may place a Work on
its own. A `hint` rule is only a prior for later tiers.

Both files were hand-assigned and need librarian review before they are
trusted at full confidence.
