# mc-eval — materials classifier eval harness

Scores classifier predictions against a labelled gold set (plan R15–R17 and
Success Criteria in `docs/plans/2026-10-07-001-feat-materials-classifier-and-kb-resource-plan.md`).
It is a standalone Rust crate with its own `[workspace]` and no path into the
surrounding repo, so it moves with the open-source tool when that is split out.

```bash
cargo run -- describe                                   # everything an agent needs, as JSON
cargo run -- run --gold gold/starter-v1.ndjson --predictions preds.ndjson
cargo run -- validate --gold gold/starter-v1.ndjson --predictions preds.ndjson
cargo run -- gold stats --gold gold/starter-v1.ndjson
```

When stdout is not a terminal, output is JSON (`mc-eval/report@1`); `--format text`
forces the table view. Errors go to stderr as
`{"error":{"code","message","next"}}` with exit 2 (usage), 3 (bad data) or 4 (I/O).

## Inputs

**Gold set** (`schemas/gold-item.schema.json`): one NDJSON line per item:
`id`, `title`, `text`, `identifiers`, `acceptable` (LCC codes or caption ranges, any
one of which counts as correct), optional `fast` headings, and `source` provenance
(kind, record, URL, retrieval time, license, labelling rule, raw snapshot and its
SHA-256).

**Predictions** (`schemas/prediction.schema.json`): one NDJSON line per item and
profile: `item` (gold `id` or L0 `item:` ID), `profile`, deciding `tier`, ranked
`candidates` (`class` may be an LCC code, a range or a typed `class:lcc@<v>:<code>`
ID), `confidence` (defaults to the top score), `flagged`, `latency_ms`, `fast`.
Several files may be passed; a second line for the same item and profile is an
error. Name comparison runs `experimental:<name>` (for example
`experimental:laya-decision`): they are reported, but marked as comparisons.

## Metrics

Hierarchy levels come from the code itself: 1 class (`Q`), 2 subclass (`QA`),
3 caption (`QA76`, or a range such as `QB630-638.8`), 4 number (`QA76.73`),
5 cutter (`QA76.73.C15`). A candidate is correct at level L when it agrees with
some acceptable class at every level up to L. Items whose gold does not reach
L are outside that level's denominator; the report gives `n` per level.

| Field | Meaning |
|---|---|
| `levels[].top1`, `topk` | top-1 / top-k accuracy per level |
| `hierarchical` | accuracy = matched depth / gold depth (hierarchical recall), precision = matched / predicted depth, f1 |
| `levels[].ece`, `brier` | calibration of top-1 confidence against correctness at that level (equal-width bins, top-label Brier) |
| `reliability` | calibration bins at `--level` (default 2, subclass) |
| `risk_coverage`, `aurc` | error rate as items are accepted in confidence order; area under that curve |
| `thresholds` | for each `--precision` target, the lowest confidence whose accepted set meets it, with its coverage; this is where R17's thresholds come from |
| `flagged_rate`, `accuracy_flagged/unflagged` | how much is flagged and whether flagging tracks errors |
| `latency_ms` | mean, p50, p95, max |
| `fast` | micro precision/recall/F1 when gold items carry FAST tags |

Each profile is reported overall and per deciding tier (`tiers[]`, with the
tier's share of predictions). Every scalar carries a percentile bootstrap
interval (`--bootstrap 1000`, `--confidence 0.95`) over items resampled with
replacement; the RNG is seeded (`--seed`), so a rerun gives identical output.
By default an unpredicted gold item counts as wrong at confidence 0
(`--missing as-wrong`); `--missing skip` drops it instead.

## Scoring shelfmark

`scripts/shelfmark-predictions.sh GOLD OUT [PROFILE] [TARGET]` runs a
`shelfmark` binary (`$SHELFMARK`) over a gold set as one NDJSON batch in a
fresh store and writes one prediction per item. Set `$SHELFMARK_SCHEME` to
compile a scheme file into the store first; it replaces a bundled scheme
with the same ID. By default no identifier table is loaded, because the
Open Library gold labels come from the records such a table is built
from. `$SHELFMARK_TABLE` loads one anyway. To score the identifier tier
honestly, build it from the editions dump with the gold records removed,
so a gold item is only found through other records that carry the same
ISBN or LCCN (`identifier-tables` reads plain text from a FIFO):

```bash
jq -r '.identifiers.openlibrary // empty | "\t/books/" + . + "\t"' gold/starter-v1.ndjson > gold-records.txt
mkfifo editions.txt
zcat ol_dump_editions_YYYY-MM-DD.txt.gz | grep -v -F -f gold-records.txt > editions.txt &
identifier-tables build editions.txt --out loo/
identifier-tables shelfmark-table loo/ --out loo.jsonl
SHELFMARK_TABLE=loo.jsonl scripts/shelfmark-predictions.sh gold/starter-v1.ndjson preds.ndjson
```

The record's source URL is not passed either, because the label came from
there.

```bash
SHELFMARK=../../src/shelfmark/target/release/shelfmark \
  scripts/shelfmark-predictions.sh gold/starter-v1.ndjson preds.ndjson profile:lexical@1
cargo run -- run --gold gold/starter-v1.ndjson --predictions preds.ndjson
```

Class codes may be LCC outline spans. A span across subclasses
(`KPG1-KPH4999.9`) scores at the main class only. A cutter span within one
number (`BX840.A1-840.Z`) scores as that number.

## Starter gold set

`gold/starter-v1.ndjson` holds 104 items across 21 LCC main classes and 57
subclasses: 40 arXiv abstracts, 34 Open Library editions and 30 Wikipedia
articles. 31 items have more than one acceptable class. Each label comes only
from the record's own metadata:

- **arXiv**: the primary category, mapped through `gold/arxiv-lcc-crosswalk.json`.
- **Open Library**: the class number in each `lc_classifications` call number of
  the pinned edition. A topical cutter attached to the number (`QH545.P4`) is
  kept, and the book number (`QA76.6 .A255`) is dropped.
- **Wikipedia**: every non-deprecated Wikidata P1149 (LCC) value of the article's item.

`gold/manifest.json` pins every record and records how each was selected.
`gold/fetch.sh` (needs curl and jq) downloads missing snapshots into `gold/raw/`
and records retrieval times in `gold/raw/retrieved.tsv`. Use `--refresh` to
download them again. The build itself is offline and deterministic:

```bash
gold/fetch.sh
cargo run -- gold build --manifest gold/manifest.json --out gold/starter-v1.ndjson
```

`tests/starter_gold.rs` checks the size and coverage targets. It also checks that
each item's `raw_sha256` matches its snapshot and that a rebuild reproduces the
shipped file byte for byte.

Known limits of these labels:

- The arXiv crosswalk was written by hand from memory of the LCC schedules. It
  has not yet been checked line by line against the FreeLCC PDFs.
- arXiv labels follow the primary category even when the paper's topic
  disagrees. For example, a bibliometrics paper filed under `cs.CL` is labelled
  P98 / QA76.9.N38.
- Wikidata P1149 is often a broad caption range, so 62 items stop at level 3.
  Only 42 items reach level 4 and 31 reach level 5.
- Some Open Library editions have no description, so their `text` is empty and
  only the title and identifiers (ISBN, LCCN) remain. This is realistic for
  identifier-tier evaluation (AE1).
- No item carries FAST tags yet. FAST metrics appear once corrections add them.

Licensing: arXiv metadata is CC0. Wikipedia extracts are CC BY-SA 4.0, with the
article and revision recorded per item. Wikidata is CC0. Open Library data is
stated as CC0, but the dump license is unverified (see the plan's Dependencies).

`fixtures/predictions-majority-baseline.ndjson` is a no-model reference: every
item is predicted as the five most frequent subclasses, with their frequencies
as scores. It scores 18.3% subclass top-1 on the starter set. Any real profile
should beat it.

Benchmark latency and throughput on remote boxes only, never on a laptop.
