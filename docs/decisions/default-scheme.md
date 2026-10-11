# Default scheme: decision record

Status: decided 2026-10-11 on existing results only; no new run. The
default stays the bundled `lcc@2024` (LCC outline + LCSH), which is what the
CLI already does, so no code change follows. This amends the first version of
this record, which chose FDC; that reasoning is kept below as the considered
alternative. The decision was made by the project manager; David can
overturn it.

All numbers below come from committed results. `BS` is
`bench/results/2026-10-11-book-stability/` (`results.md`, `results.json`,
`settings/books-<model>-k<k>-q<q>.json`). `R9` is
`research/reports/9-kb-structure-proposal.md`. Intervals are 95% paired
item-bootstrap percentiles (1,000 draws, seed 20261010) on the 3,000-book
sample, as in `BS`.

## Objective

Choose the scheme and depth the CLI files into by default, for a general
library of mostly books. Two objectives are in play.

- **C**: normalised Dasgupta cost (`cost_norm`). Lower is better.
  Skill is (1 − C) / (1 − C_ref) against the setting's reference, so within
  a setting it orders runs as C does (`BS`, "Bead rule").
- **J′ = C + FJs + 0.002·N**, with N = `nav_log_a1` (bits) and FJs the
  specificity false-join term at β = 1 (`R9` §2.8). It is the corrected
  objective: it agrees with the judge at κ = 0.48 [0.43, 0.53] against 0.27
  for C alone (`R9` Summary; `docs/validity/README.md`). J = C + 0.002·N
  and C + FJs are also reported in `BS`.

## The rule

> Highest placed skill with coverage above a floor, ties broken by
> `nav_log_a1`; one corpus, all schemes placed through the same classifier,
> with CIs.

Operational form (`BS`, "Bead rule"): among placed runs whose coverage is at
least the floor, take the highest skill; every run whose paired difference
to it has a 95% CI covering zero is tied; the tied run with the lowest N
wins. This is evaluated per setting (3 embeddings × k ∈ {10, 20, 50} × 3
graph thresholds = 27) on the 3,000 books.

**Floor: 0.90.** Coverage is the share of items the classifier places at
all. The rest wait in the review queue (`R9` §6.3), so a floor of 0.90
means at most one item in ten needs a person. A scheme that places fewer
is not usable as a fully automatic default no matter how well it scores on
the items it does place (`R9` §2.3: covered-subset scores flatter low
coverage). The result does not depend on the exact value: the table below
shows 0.70 to 0.95.

## Evidence

Books, MiniLM, k = 20, published graph (q00); `BS` `settings/books-minilm-k20-q00.json`.
Skill and J′ with 95% CI. N in bits.

| placed run | coverage | skill | C | J′ | N |
| --- | ---: | --- | --- | --- | ---: |
| `lcc-lcsh-placed` (LCC outline + LCSH vocabulary; bundled) | 0.985 | 0.423 [0.403, 0.445] | 0.674 [0.656, 0.690] | 0.726 [0.707, 0.742] | 18.61 |
| `unesco-thesaurus-placed` | 0.925 | 0.482 [0.459, 0.504] | 0.629 [0.612, 0.646] | 0.700 [0.682, 0.717] | 22.17 |
| `lcc-placed` (LCC outline, no LCSH vocabulary) | 0.915 | 0.346 [0.326, 0.365] | 0.733 [0.718, 0.749] | 0.791 [0.776, 0.807] | 20.55 |
| `fdc-placed` | 0.839 | 0.450 [0.428, 0.471] | 0.654 [0.637, 0.671] | 0.727 [0.711, 0.745] | 23.37 |
| `lcc-full-placed` | 1.000 | 0.309 [0.292, 0.327] | 0.762 [0.747, 0.776] | 0.819 [0.804, 0.833] | 18.48 |

Winner of the rule, settings won out of 27 (`BS` results.md, "Winners by
coverage floor"; `results.json` `bead_rule.books.floors`):

| floor | all placed runs | bundleable schemes only |
| ---: | --- | --- |
| 0.70 | `lcc-lcsh-placed` 15, `unesco-thesaurus-placed` 12 | `lcc-lcsh-placed` 18, `fdc-placed` 9 |
| 0.80 | `lcc-lcsh-placed` 15, `unesco-thesaurus-placed` 12 | `lcc-lcsh-placed` 18, `fdc-placed` 9 |
| 0.85 | `lcc-lcsh-placed` 15, `unesco-thesaurus-placed` 12 | `lcc-lcsh-placed` 27 |
| **0.90** | `lcc-lcsh-placed` 15, `unesco-thesaurus-placed` 12 | `lcc-lcsh-placed` 27 |
| 0.95 | `lcc-lcsh-placed` 27 | `lcc-lcsh-placed` 27 |

Pairwise, same setting (`settings/books-minilm-k20-q00.json`, `deltas`):

| pair (a − b) | objective | Δ [95% CI] | stable over 27 settings? (`BS` results.md) |
| --- | --- | --- | --- |
| `fdc-placed` − `lcc-lcsh-placed` | J′ | +0.0014 [−0.0177, +0.0217] | NOT STABLE: sign a < b in 9/27, 5 significant for, 8 against |
| `fdc-placed` − `lcc-lcsh-placed` | C | −0.0205 [−0.0396, −0.0001] | NOT STABLE: 16/27, Δ range [−0.0641, +0.0510] |
| `fdc-placed` − `lcc-placed` | J′ | −0.0641 [−0.0832, −0.0430] | STABLE: 27/27, 24 significant, none against |
| `ddc-section` − `lcc-gold` (librarian numbers) | J′ | −0.0080 [−0.0229, +0.0070] | NOT STABLE: 8/27 for the claimed tie, 19 significant against it |
| `ddc-section` − `lcc-gold` | C | −0.0292 [−0.0438, −0.0141] | NOT STABLE: 13/27, 10 significant for, 9 against, Δ range [−0.0753, +0.0750] |
| `learned-own` − `ddc-section` | J′ | −0.1048 [−0.1286, −0.0828] | NOT STABLE: 19/27, 13 significant for, 6 against |

Pair-level sign counts across settings: `BS` results.md, "books: pair
claims". Runs available in the 27 settings: skill of `fdc-placed`
0.425–0.545 (median 0.483), of `lcc-lcsh-placed` 0.364–0.576 (median
0.468), of `lcc-placed` 0.301–0.505 (median 0.384); `results.json`
`bead_rule.books.rows`.

## Stability verdict

27 settings, four objectives (`BS`).

Stable, 27/27 under every objective that applies:

- Dewey sections beat FDC depth 3 (`ddc-section` − `fdc-d3` ≤ 0, Δ range
  [−0.0055, −0.0010]); the margin is under 0.006, so this is a
  caption-versus-number difference, not a reason to leave FDC.
- Three levels beat two (`fdc-d3` − `fdc-d2`, Δ range [−0.0117, −0.0001]).
- Full-depth Dewey gains cost but loses on J and J′ (`ddc-gold` −
  `ddc-section`, J′ Δ range [+0.0013, +0.0036]).
- Through the same classifier, FDC beats the LCC outline without LCSH
  vocabulary (`fdc-placed` − `lcc-placed`, J′ Δ range [−0.0897, −0.0237]).
- One nested learned level helps, three raise J′.

Not stable:

- **Dewey sections against LCC on librarian numbers.** The sign flips with
  embedding model, k and threshold. This is the central finding: Dewey and
  LCC are a setting-dependent tie.
- FDC against the bundled LCC + LCSH, placed (above).
- The learned scheme against Dewey sections.
- The ranking of the eight gold-numbered structures (min ρ −0.143 under C,
  −0.667 under J; `BS` ranking claims, result NOT CLEARED).
- Three FDC levels against the full FDC number on J and J′: sign a < b in
  15/27, Δ range [−0.0005, +0.0004], a tie.

Max regret of J′ over λ ∈ [0.0005, 0.005] and β ∈ [0.5, 2] (`BS`, "Minimax
regret of the gold-numbered structures"):

| structure | 9 published graphs | all 27 settings |
| --- | ---: | ---: |
| `ddc-section` | 0.0597 | 0.1156 |
| `ddc-d2` | 0.0618 | 0.1221 |
| `fdc-d3` | 0.0655 | 0.1226 |
| `lcc-gold` | 0.0670 | 0.0670 |

Over the nine published graphs Dewey sections have the lowest worst-case
regret, the tie-break `R9` used (0.060 against 0.067). Over all 27 settings
LCC has the lowest (0.0670 against 0.1156). That tie-break is therefore no
reason to prefer Dewey.

## What the rule and `R9` each answer

- **The rule** asks which scheme a fully automatic pipeline files best:
  every scheme goes through the same lexical classifier, with nothing
  assigned by a librarian. It answers "which scheme does the classifier
  place best?". Applied above it picks `lcc-lcsh-placed`. Part of that win
  is vocabulary, not scheme: the bundled LCC has LCSH headings as search
  terms, which lifts coverage from 0.915 (`lcc-placed`) to 0.985, while
  FDC places at 0.839 with its captions only. No equivalent FDC vocabulary
  has been built.
- **`R9`** asks what structure a library should adopt when items carry
  librarian-assigned numbers (Open Library work-level Dewey numbers for 91%
  of the holdout, LC call numbers for 100%). The classifier is the last
  step and never decides alone (`R9` §6.3). Its comparisons are on gold
  numbers (`ddc-section` against `lcc-gold`) and on the pipelines
  (`fdc-pipe-90`, `lcc-pipe-90`).

They differ in placement quality (classifier at 54% main-class agreement
with librarians, `R9` Summary), not in corpus or metric. The rule's winner
is the right answer when the CLI must place from titles and descriptions
alone. `R9`'s is the right answer when catalogue numbers are available.
Neither answer survives the stability sweep as a metric result: the gold
comparison is a tie, and the placed comparison turns on the LCSH
vocabulary.

On the holdout (500 books, title-only graph) no placed run reaches any
floor (best coverage 0.566, `lcc-full-placed`; `BS` bead-rule table), so
the rule returns no winner there and says nothing about the default.

## Decision

1. **The default is the bundled LCC outline with LCSH vocabulary,
   `lcc@2024`** (`data/schemes/lcc/lcc-lcsh.json.gz`, US government work,
   compiled into the binary).
2. **Placement order**: LC call number from the catalogue when there is one,
   then the classifier, then the review queue (`R9` §6.3).
3. **FDC (`fdc` @ 0.06, CC0, depth 3) is the documented alternative**, and
   `lcc-full` remains available for call-number libraries that want the
   complete schedules.

Why LCC:

- **It wins the rule.** Among the bundleable schemes (open, allowed in the
  binary) `lcc-lcsh-placed` wins all 27 settings at the 0.90 floor, and at
  every floor from 0.85 up (Evidence, "Winners by coverage floor"). FDC
  places only 0.839 of the books, under the floor.
- **Catalogue numbers cover every book.** LC call numbers exist for 100% of
  the holdout's works against 91% with a Dewey number (`R9`), so the
  catalogue-first step finds an LC number more often than a Dewey one.
- **Switching later is cheap.** Placements are keyed by content id, not by
  where a file sits, so moving to another scheme is a new id-to-class
  rendering of the same items, not a re-filing by hand.
- The Dewey/LCC metric comparison is a setting-dependent tie (Stability
  verdict), so nothing in the metrics overrides the rule.

### The considered alternative: FDC at depth 3

The first version of this record chose FDC (`fdc` @ 0.06) filed to depth 3
(main class, division, section: `8xx`, `82x`, `823`), with the full Dewey
number kept as item metadata, catalogue Dewey first, LC call numbers through
the measured crosswalk second, and depth 2 for libraries under about 2,000
items (`R9` §6.2). Its reasons, which still hold as facts:

- Dewey against LCC is a tie that depends on setting, so the choice was made
  on other grounds.
- FDC is CC0 with open captions that can ship in the binary; the Dewey
  numbers themselves are not redistributed, and FDC supplies open captions
  for the same numbers.
- Three-digit notation is shorter than LCC's average 7.6 characters
  (`R9` §6.1, `catalogue-books`).
- Papers nest better under the FDC backbone than under LCC (`R9` §5.2).
- Depth 3 beats depth 2 in 27/27 settings and is tied with the full number
  on J′ (above).

It departed from the rule on judgement: the shipped pipeline was to be
catalogue-first, the gold and pipeline comparisons are ties, and the rule's
margin over FDC is partly a vocabulary margin (LCSH lifts LCC's coverage from
0.915 to 0.985; FDC has no search vocabulary yet). That judgement was not
kept because the rule's result is the only one in this record that is not a
tie, LC numbers are the more available catalogue numbers, and the choice is
reversible at the cost of a re-render.

## Where the code stands

The CLI already follows the decision. The default is the first compiled
scheme, the bundled LCC outline with LCSH vocabulary:
`crates/cli/src/wiring/mod.rs` (`schemes: vec![bundled_lcc_in(...)]`, data
in `LCC_GZ`, id `SchemeId::new("lcc", "2024")`), selection in
`crates/cli/src/service/mod.rs` (`index`, first compiled). The test
`crates/cli/tests/bundled.rs` pins `scheme:lcc@2024`. FDC is not compiled
into the binary. Sources, license and rebuild of the bundled file:
`docs/schemes/lcc-lcsh-sources.md`.

## What would overturn it

- **FDC gains a search vocabulary** that lifts its placed coverage to at
  least 0.90 with J′ no worse than `lcc-lcsh-placed` (+0.0014
  [−0.0177, +0.0217] today); the rule would then compare them on equal
  terms.
- **A stable Dewey win.** A graph not built from embeddings (co-use,
  citations, the owner's links) on which `ddc-section` beats `lcc-gold` in
  every setting with CIs excluding zero.
- **Mostly papers.** If the library is mostly papers rather than books, FDC's
  better nesting of papers (`R9` §5.2) weighs more.
- **Short shelf marks matter.** If the owner wants three-digit shelf marks
  over LCC's longer notation.
- **A semantic classifier** that reaches ≥ 0.85 main-class agreement at
  coverage ≥ 0.95 would re-open the classifier-only comparison on equal
  terms. The BGE-small shortlist tier does not reach that: it has coverage
  1.0 but agreement of 0.51 (FDC) and 0.56 (LCC + LCSH) on the 3,000 books
  (`bench/results/2026-10-11-books-semantic/`).
- **A repeat on a different corpus** (not the 3,000-book Open Library
  sample, stratified equally over LCC's 21 main classes, which `R9` §4.5
  flags as favouring LCC's false-join term).
