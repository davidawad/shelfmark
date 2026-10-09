# vocab: LCSH and FAST vocabulary ingest (spike)

Spike 2 for the vocabulary tier of the
materials classifier. It is a standalone Rust crate (its own `[workspace]`) whose only
path dependencies are shelfmark's `core` and `scheme` crates, used by `enrich-scheme`.

It covers three things:

1. A **deterministic ingester** that turns the id.loc.gov LCSH bulk MADS dump
   and OCLC FAST records into one compact heading table.
2. **LCC link statistics**: how many LCSH headings carry an LCC number.
3. An **Aho-Corasick label matcher prototype** with an LCC class vote and a
   small accuracy check.

Large downloads and full outputs stay on the benchmark host and are never committed.
Committed: code, real record fixtures, table samples, stats and the eval report.

## Headline numbers (LCSH dump issued 2026-10-06, fetched 2026-10-07)

| | |
| --- | --- |
| LCSH authority records | 521,392 (7,922 deprecated) |
| **LCSH headings with at least one LCC link** | **91,580** (17.6% of all; 17.8% of the 513,470 active) |
| Of those, `Topic` / `ComplexSubject` | 77,342 / 13,392 |
| LCC links in total / ranges | 100,951 / 17,294 |
| Distinct LCC codes | 63,843 |
| Rejected class strings | 1 (`WP535.C9`, an NLM class, not LCC) |
| Headings with alt labels / broader | 180,310 / 229,987 |

All headings with LCC links are active. Full detail is in `stats/lcsh-stats.json`.
The 2026-10-07 dump has sha256 `f57e6dcf…eebd` (230 MB). The table it produces
has sha256 `d2f074cc…948b` (11 MB gz), and two runs give byte-identical output.

On the benchmark host (12 vCPU), the full LCSH ingest takes about 20 s with a peak RSS of
280 MB. Building the matcher over 1.11 M patterns and running the eval takes
about 7 s with a peak RSS of 1.4 GB.

## Heading table

The table is a TSV with a header row, sorted by `id`. List cells are sorted,
deduplicated and joined with `|`. A backslash escapes `\\`, `\t`, `\n` and
`\|` inside cells.

| column | meaning |
| --- | --- |
| `id` | `sh86007588` (LCSH) or `fst01001522` (FAST, zero-padded) |
| `vocab` | `lcsh` or `fast` |
| `kind` | MADS type (`Topic`, `ComplexSubject`, `Geographic`, ...) or FAST facet (`Topical`, ...) |
| `pref_label` | `madsrdf:authoritativeLabel` / `skos:prefLabel` (on deprecated LCSH records, the `variantLabel`) |
| `alt_labels` | `hasVariant/variantLabel` / `skos:altLabel` |
| `broader` | `hasBroaderAuthority` / `skos:broader` ids in the same vocabulary |
| `lcc` | canonical LCC numbers or ranges, e.g. `QA76.76.C65`, `QH101-QH198`, `D-F`, `PL772.9-PL772.988(Collections)` |
| `lcc_source` | `direct` (on the LCSH record) or `lcsh` (FAST, joined through `same_as`) |
| `same_as` | FAST: LCSH ids it links to. Deprecated LCSH: `useInstead` replacement ids |
| `deprecated` | `1` for `madsrdf:DeprecatedAuthority` |

### How LCC is read

In the MADS dump, LCC appears on `madsrdf:classification` in two forms: a blank
`lcc:ClassNumber` node with `madsrdf:code` (61,921 links), and an
`http://id.loc.gov/authorities/classification/<code>` IRI (39,031 links). Both
are read and deduplicated. `madsrdf:code` is also used for MARC geographic area
codes (`n-us-ca`), so a code counts as LCC only when it hangs off
`madsrdf:classification`.

Normalisation (`src/lcc.rs`) handles the hand-keyed forms found in the dump:

- lower-case class letters (`Ql430.P6`)
- a dot after the class letters (`P.325.5.O76`)
- bare-number range ends (`QH101-198` becomes `QH101-QH198`)
- dotted range ends (`PR9408.R8-.R83` becomes `PR9408.R8-PR9408.R83`)
- class-letter ranges (`B-BJ`)
- captions keyed after a hyphen or in parentheses (`QL638.S2-Zoology`)

The dump records each heading in a `# BEGIN` / `# END` block, and blank-node
labels are only unique within a block, so the ingester parses one record at a
time.

## FAST: blocked bulk download

The FAST bulk files (`researchworks.oclc.org/researchdata/fast/FAST*.{nt,marcxml}.zip`)
answer **HTTP 403 with a Cloudflare browser challenge** from the benchmark host, whatever
the user agent. A non-interactive client cannot get past it, so the full FAST
table could not be built here. What exists:

- `vocab-ingest fast --nt FASTTopical.nt` ingests the bulk N-Triples, using the
  predicates the per-record documents use (`skos:prefLabel`, `skos:altLabel`,
  `skos:broader`, `skos:inScheme` facet, `schema:sameAs` to LCSH). This path is
  unit-tested on hand-written triples only, because no bulk file could be fetched.
- `vocab-ingest fast --rdfxml DIR` ingests per-record RDF/XML from
  `id.worldcat.org/fast/<n>.rdf.xml`, which is reachable.
  `scripts/fetch-fast-sample.sh` collects such records through the FAST suggest
  API. **31 real Topical records** are committed in `sample/fast-records/`
  (FAST is ODC-By 1.0, © OCLC; attribution kept here).
- FAST records carry **no LCC numbers of their own**. LCC comes from joining
  `schema:sameAs` to the LCSH table. In the sample, 29 of 31 headings link to
  LCSH, all 29 links resolve, and 18 headings inherit LCC
  (`stats/fast-sample-stats.json`). FAST headings with subdivisions
  (`Education--Costs`) have no LCSH link and no LCC.

To finish FAST, fetch `FASTTopical.nt.zip` from a browser (or any machine
Cloudflare lets through), copy it to the benchmark host and run the `fast --nt` command
below. Then check that the bulk predicates match the ones assumed above.

## Matcher prototype

Implemented in `src/matcher.rs`, using `aho-corasick` with an overlapping
contiguous NFA:

- Labels and text are folded the same way: NFKD, combining marks dropped,
  lower case, and every non-alphanumeric run collapsed to one space.
- Each heading yields its pref and alt labels, a qualifier-free variant
  (`Songs (High voice) with harp` becomes `songs with harp`) and an uninverted
  variant (`Chemistry, Organic` becomes `organic chemistry`).
- Patterns under 3 characters, or made only of function words, are dropped.
- A hit must start and end on a word boundary. A hit wholly inside a longer hit
  is dropped; crossing hits are both kept.
- Each hit votes for its headings' LCC classes. The weight is the pattern's
  token count × the origin weight (pref 1.0, alt 0.9, uninverted 0.8,
  unqualified 0.7), shared across homonym headings. A heading without LCC
  borrows the nearest broader heading's LCC (up to 3 levels) at half weight.
- Votes are ranked by subclass (`QA`) and by main class (`Q`).

### Accuracy check (`stats/eval-lcsh.json`, full LCSH table)

**Fixtures:** 38 short passages in `fixtures/lcc-fixtures.jsonl`, each
labelled with one expected LCC subclass. They were written and labelled by the
agent from LCC schedule knowledge before the matcher was first run, not by a
cataloguer. Treat them as a smoke-level check.

| metric | result |
| --- | --- |
| subclass top-1 | **28 / 38 (74%)** |
| subclass in top 3 | 35 / 38 (92%) |
| main class top-1 | 33 / 38 (87%) |
| no class vote | 0 / 38 |

**Self-check:** 2,000 active `Topic` headings with direct LCC, chosen by FNV
hash of id, each embedded in filler text.

| metric | result |
| --- | --- |
| heading recovered | 1,997 / 1,997 |
| its own subclass wins | 1,966 / 1,997 (98.4%) |

(3 of the 2,000 sampled labels were under 3 characters after stripping and were skipped.)

The first run scored the same 28/38 on fixtures but recovered only 1,947/1,997
in the self-check. The misses were labels with a mid-label qualifier
(`Motivation (Psychology) in adolescence`), which the first version only
stripped when trailing. That run also showed function words (`and`, `for`)
matching as headings. Both were fixed (qualifier stripping anywhere, plus the
function-word filter), and the numbers above are from the fixed version.
Nothing else was tuned against the fixtures.

**Failure modes in the 10 fixture misses**, all expected for a pure-lexical tier:

- Near-sibling subclasses: HB vs HD/HG for macroeconomics, KF vs JK for US
  constitutional law, MT vs ML for music theory, GA vs G for cartography,
  NA vs NK for cathedrals, TG vs TA for bridges.
- Personal names and works: `Shakespeare`, `Hamlet` and `Julius Caesar` live in
  the LC Name Authority File, not LCSH.
- Generic single words outvoting the topic (`roman`, `empire`, `guide`).
- No singular/plural folding, so `compiler` does not hit `Compilers`.

Suggested next steps for the lexical tier: fold plurals, add IDF
weighting so generic single-token headings count for less (we already
specifies IDF), check range containment against the LCC outline, and add
name-authority identifiers for persons and works.

## Folding headings into the shelfmark scheme (`enrich-scheme`)

The bundled LCC outline (`data/schemes/lcc/lcc.json`, )
links a heading only when its LCC number is exactly an outline class: 1,343
links on 1,033 of 7,392 classes. `enrich-scheme` adds every active heading
from the table to the outline class that holds its number. The holder is the
class itself or the deepest span covering it, as shelfmark's own compiled
index decides (`check_code`, through a path dependency on
`crates/{core,scheme}`). The result is an ordinary scheme file
that shelfmark's vocab tier reads with no code change:

```sh
$B enrich-scheme --scheme data/schemes/lcc/lcc.json \
  --table $D/out/lcsh.tsv.gz --out $D/out/lcc-lcsh.json --stats $D/out/enrich-stats.json
shelfmark scheme compile $D/out/lcc-lcsh.json   # preview, then --commit
```

On the 2026-10-07 dump, 91,499 of 91,580 headings with LCC land in a class
(63,754 of 63,843 numbers; the rest are letter spans like `D-F`). That adds
97,848 terms and gives 3,647 classes vocabulary, in 9.0 MB of JSON.
`--alt true` also adds alternate and uninverted labels: 250,859 terms in
21.6 MB.

Measured with the lexical profile on the eval starter gold set (104 items,
`research/eval`, no identifier table), with subclass
top-1 meaning LCC subclass level:

| scheme | class top-1 | subclass top-1 | subclass top-5 |
|---|---|---|---|
| bundled outline | 31.7% | 15.4% | 27.9% |
| enriched, preferred labels | 47.1% | 34.6% | 57.7% |
| enriched, `--alt true` | 45.2% | 36.5% | 63.5% |
| majority-subclass baseline | 33.7% | 18.3% | 33.7% |

**Why it is not bundled yet.** Every `shelfmark` process compiles its
bundled scheme, and the vocab tier builds its automaton on first use. For
the preferred-label file that is 0.79 s to compile (0.42 s of it schema
validation) plus about 0.8 s for the automaton, so one interactive classify
takes about 1.6 s on a 12-vCPU benchmark host. The bundled outline takes 0.25 s. The
the success criterion is well under a second on a laptop CPU, and the
binary would grow by 9 MB. Bundling needs a persisted compiled form and a persisted or lazily built automaton first. Until then
the file is generated data, like the identifier tables: build it
from the public LCSH dump and compile it into a store.

## Commands

```sh
cargo build --release      # standalone workspace; set CARGO_TARGET_DIR as you like
B=target/release/vocab-ingest
D=/work/data/vocab         # big files live here, outside git

curl -sSLo $D/lcsh/subjects.madsrdf.nt.gz https://id.loc.gov/download/authorities/subjects.madsrdf.nt.gz
$B lcsh --input $D/lcsh/subjects.madsrdf.nt.gz --out $D/out/lcsh.tsv.gz --stats stats/lcsh-stats.json

scripts/fetch-fast-sample.sh $D/fast/records
$B fast --rdfxml $D/fast/records --lcsh $D/out/lcsh.tsv.gz --out $D/out/fast-sample.tsv --stats stats/fast-sample-stats.json
# full FAST, once FASTTopical.nt is obtained:
# $B fast --nt $D/fast/FASTTopical.nt --lcsh $D/out/lcsh.tsv.gz --out $D/out/fast.tsv.gz --stats stats/fast-stats.json

$B eval  --table $D/out/lcsh.tsv.gz --fixtures fixtures/lcc-fixtures.jsonl --self-check 2000 --out stats/eval-lcsh.json
$B match --table $D/out/lcsh.tsv.gz [--table $D/out/fast.tsv.gz] --text "Organic chemistry of heterocycles"
$B sample --table $D/out/lcsh.tsv.gz --every 2500 --ids sh86007588,sh85082139,sh85029524 --out sample/lcsh-sample.tsv
```

## Layout

| path | content |
| --- | --- |
| `src/ntriples.rs` | line N-Triples parser |
| `src/lcsh.rs` | MADS block parser and stats |
| `src/lcc.rs` | LCC normalisation |
| `src/fast.rs` | FAST builder with LCSH join |
| `src/rdfxml.rs` | targeted RDF/XML reader |
| `src/table.rs` | TSV table |
| `src/matcher.rs` | matcher |
| `src/eval.rs` | eval |
| `src/enrich.rs` | `enrich-scheme`: headings into a shelfmark scheme's vocabulary |
| `sample/lcsh-records.nt` | 7 real MADS record blocks (citation triples removed) used by tests |
| `sample/lcsh-sample.tsv` | 212 rows, every 2,500th row plus named ids |
| `sample/fast-records/`, `sample/fast-sample.tsv` | 31 real FAST records and their table |
| `stats/` | LCSH stats, FAST sample stats, eval report, `enrich-stats.json` |
| `fixtures/lcc-fixtures.jsonl` | 38 labelled passages |

## Sources and licences

- LCSH: Library of Congress, id.loc.gov bulk download. US government work; no
  explicit licence text was found (recorded as unverified).
- FAST: OCLC, Open Data Commons Attribution License (ODC-By) 1.0.
  "FAST (Faceted Application of Subject Terminology) data, © OCLC."
