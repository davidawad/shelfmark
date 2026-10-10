# Which knowledge-base structure for a personal library: a proposal from the benchmark

Written 2026-10-10. This report recommends one classification scheme and
one hierarchy for the owner's personal library: mostly books, plus papers
and other specimens. The recommendation is precise enough for another
agent to reorganise the collection. Every number in a table is
regenerated from committed results by
`research/reports/scripts/kb_proposal_tables.py`, which exits 1 if any
table drifts. Each table names its sources in the text above it. Numbers
quoted from other runs cite their results file inline.

Inputs:

- `bench/results/2026-10-10-ddc/`: Dewey, LCC, FDC and placed schemes on
  3,000 Open Library books and on the 500-book holdout (report 7).
- `bench/results/2026-10-10-embed/`: the three embedding models
  (all-MiniLM-L6-v2, bge-small-en-v1.5, nomic-embed-text-v1.5) at k = 10,
  20 and 50.
- `bench/results/2026-10-10-covered/`: covered-subset scores and the
  coverage analysis.
- `bench/results/2026-10-10-scores/` and report 6: the scoring catalogue.
- `bench/results/2026-10-10-proposal/`: new runs made for this report by
  `research/reports/scripts/kb_proposal_runs.py` and
  `ol_work_numbers.py`. These are the depth cuts, availability pipelines,
  work-level lookups, papers nested under a book backbone, the books graphs
  for the other two models, and paired bootstraps for every universe and
  graph. They are described in that directory's `README.md`.

The semantic-tier rerun (`bench/results/2026-10-10-semantic/`) published
no results and is **pending** (`bench/results/README.md`). Every
classifier number below is therefore the lexical classifier
(`profile:lexical@2`).

## Summary

**Recommendation.** Every item gets the Dewey number that librarians
gave it, taken from catalogue data. The directory tree is that number's
first three digits: main class, then division, then section. Each level
is labelled with the caption of the
Free Decimal Correspondence (FDC, CC0) class with the same number. The
full Dewey number is kept only in each item's metadata, where it sets the
order inside a section. Where no Dewey number exists, the item gets one
from its LC call number through a measured crosswalk, or from its arXiv
category for papers. Otherwise it goes to a review queue. The classifier
never decides on its own.

Why:

1. **Arrangement.** On 3,000 books, Dewey sections beat LCC on the
   decision objective J (normalised Dasgupta cost plus a navigation term,
   §1). The difference is −0.0273 [−0.0421, −0.0123], and it holds at
   k = 20 and k = 50 for all three embedding models. At k = 10 the two do
   not differ significantly after correction.
2. **The learned scheme.** It is better still: −0.155 cost. But it fails
   two admissibility constraints (opaque notation, and placement tied to
   one embedding model). Part of its lead is home advantage. Nesting it
   inside Dewey sections buys almost nothing.
3. **Depth.** Three digits is the minimax-regret cut over the plausible
   range of the navigation weight in six of nine conditions. Two digits
   wins the other three, and it is the better cut for libraries under
   about 2,000 items.
4. **Placement.** A work-level Open Library lookup gives a Dewey number
   for 91% of the holdout's works, against 22% from the edition record.
   The LCC-to-FDC crosswalk then places 76% of the remaining books with
   87% main-to-section accuracy (cross-validated). The lexical classifier
   agrees with librarians on only 54% of main classes.

## 1. The decision objective

### What is being chosen

A *structure* S = (T, c, π) has three parts:

- a classification scheme T, a rooted tree of classes;
- a cut c, which keeps only the top c levels of T as browsable nodes;
- a placement policy π, which maps an item and its identifiers to one
  node of the cut tree, or to "unplaced".

Applying S to a library L of n items gives a tree 𝒯(S, L). Its internal
nodes are the kept classes and its leaves are the items. Unplaced items
hang from the root.

### The objective

Let G = (L, E, w) be a similarity graph on the items: k nearest neighbours
under a text-embedding model, with weights w. The browsing cost of S is

  J_λ(S; G) = C(𝒯; G) + λ · N(𝒯)

where:

- C is the normalised Dasgupta cost (§2.1). It is the expected share of
  the library under the lowest common ancestor of two similar items,
  which is what a user scans when going from one item to related ones.
- N is the expected navigation cost in bits (§2.4). It is what a user
  pays to walk from the root to a known item.

**Coverage and placement accuracy are not separate terms.** 𝒯 is built
from the *realised* placements of π. An unplaced item sits at the root
and pays the maximum, C's term n, on every one of its edges. A misplaced
item meets its true neighbours high in the tree and raises C. The
pipelines of §5.3 measure exactly this end to end: the same books, with
librarian numbers for a share of them and the classifier for the rest.
The review queue is the one cost J does not see, so it is reported next
to J as the share of items sent to review.

**Admissibility constraints.** J is minimised only over structures that
satisfy all four:

- **A1, open labels.** The captions shown to the owner must be
  redistributable. This excludes Dewey captions (report 7; `deweyrights`).
  FDC is CC0 (`fdc`) and LCC is a US government work (`usc105`).
- **A2, stable readable notation.** An item's code must be short enough
  to write on a spine or in a file name. New items must enter without
  renumbering existing classes: hospitality, report 6.
- **A3, external anchoring.** An item's class must follow from public
  catalogue data, not from one embedding model. Otherwise a model change
  would reshuffle the library, and the score is measured with the same
  kind of embedding that placed the items.
- **A4, reproducible placement.** The policy is a fixed procedure, so
  another agent applying it gets the same tree.

### The weight λ

Suppose a session has r browsing moves (item to related item) per
known-item lookup. Scanning one item costs s, and one bit of menu choice
costs b. The expected session cost is then

  r·s·n·C + b·N, which is proportional to C + (b / (r·s·n)) · N.

So λ = κ / n with κ = b / (r·s): the number of items one could scan for
the price of one bit of navigation, per browsing move. The benchmark
books are n = 3,000. λ = 0.002 corresponds to κ = 6, and the range
λ ∈ [0.0005, 0.005] to κ ∈ [1.5, 15].

The Hick–Hyman law (`hick1952`, `hyman1953`) makes a menu decision cost
linear in bits. The harness's a = 1 term adds a constant per level, as in
the menu-depth studies (`miller1981`, `snowberry1983`, `landauer1985`,
`zaphiris2000`, `larson1998`). None of those studies measures κ for a
personal library, so κ is not calibrated. **The recommendation therefore
does not rest on λ = 0.002.** Every choice is checked by its worst-case
regret over the whole range λ ∈ [0.0005, 0.005]:

  regret(S) = max_λ [ J_λ(S) − min_S′ J_λ(S′) ]

The structure with the lowest regret is chosen. Because λ = κ / n, a
smaller library has a larger λ. That is where the depth rule of §6.2
comes from.

**Why C and not another score as the primary term.** C is the
hierarchical-clustering objective with an approximation theory
(`dasgupta2016`, `cohenaddad2017`). It reads the whole tree, and its
value has a direct meaning in items scanned. Triplet accuracy,
shelf-order locality and TSD (§2.5) are reported as secondary scores. On
the books they disagree with C in one place: LCC has better locality
(§4.3). §4.3 gives the locality weight at which that would flip the
decision.

## 2. Metrics

Report 6 defines the full catalogue. These are the scores used here.
Bootstrap and test procedures follow in §2.7.

### 2.1 Dasgupta cost and its normalisation

For a tree 𝒯 whose leaves are the items, and a graph with weights w_ij,

  cost(𝒯) = Σ_(i,j)∈E w_ij · |leaves(LCA(i, j))|

`cost_norm` = cost / (n · Σ w), which lies in (0, 1]. A tree that puts
every item at the root scores 1. Lower is better (`dasgupta2016`).
Values are comparable only on the same graph: same items, model and k.
That is why every comparison here is paired on one graph.

### 2.2 Moseley–Wang revenue and skill

The revenue is Σ w_ij (n − |leaves(LCA)|) (`moseley2017`, `moseley2023`),
so `revenue_norm` = 1 − `cost_norm`. Skill rescales revenue against the
average-linkage tree built on the same graph:

  skill = (1 − cost_norm) / (1 − cost_norm(average linkage))

The reference tree is recomputed on every bootstrap draw. Skill is 1 at
average-linkage quality and can exceed 1. No classification can match a
binary tree's resolution, so skill < 1 is expected. **Average linkage**
merges the two clusters with the highest mean edge weight. It is the
reference the harness builds (`crates/bench/src/reference.rs`), and its
revenue is a constant-factor approximation of the Moseley–Wang optimum
(`moseley2017`).

### 2.3 Coverage and covered-subset scores

Coverage is the share of items placed. Covered-subset scores (report 6;
`bench/results/2026-10-10-covered/`) rescore a run on a subset of items,
keeping only the graph edges between them. The *both-place* subset is the
items two runs both place, so neither pays the root penalty and the
comparison is paired. Here, every gold run on the 3,000 books has
coverage 1.000 (`books-main`), so the main comparison is
coverage-controlled by construction. The 500-book holdout is analysed
both raw and on its both-place subset (§5.2).

### 2.4 Navigation cost and depth

`nav_log_a1` is the expected cost, in bits, of the root-to-item path,
weighted by item. Each node with fan-out f > 1 on the path costs
1 + log₂ f, and the last step chooses among a class's own items, so a
large class is not free (`crates/bench/src/metrics.rs`,
`log_branch_cost`). This is the Hick–Hyman menu model with an intercept
of 1 (report 6, `hick1952`, `landauer1985`).

The **expected navigation depth** (`mean_depth`) is the mean number of
tree levels above an item. **Effective classes** is exp of the entropy of
class sizes.

### 2.5 Secondary scores (report 6)

- **Triplet accuracy.** For each graph edge (i, j), one non-neighbour k
  is sampled. The triplet is correct when LCA(i, j) is deeper than
  LCA(i, k); a tie counts one half. A flat tree scores 0.5.
- **Shelf-order locality.** Classes are put in pre-order, with an item
  after its class's own items. Locality is 1 − (weighted mean of |Δ
  position| over edges) / E|U − V|. It is 0 for a random order and 1 when
  neighbours are adjacent on the linear shelf.
- **Tree-sampling divergence (TSD).** The KL divergence, in nats, between
  the LCA distribution of a sampled edge and that of two independently
  sampled items (Charpentier and Bonald 2019, report 6). Higher is
  better.
- **Hospitality in array, splits per 1,000 inserts, notation length and
  expected search bits.** These come from report 6's librarian and
  computer-science blocks.

### 2.6 Agreement and availability

- **Agreement with a librarian** is the share of items whose placed class
  has the same depth-d ancestor as the librarian's number.
- **Availability** is the share of works for which a catalogue gives a
  number.

Both are binomial proportions with 95% Wilson intervals (Wilson 1927,
https://doi.org/10.1080/01621459.1927.10502953).

### 2.7 Bootstrap and tests

All J and cost intervals come from the paired **vertex (item)
bootstrap** of `shelfmark-bench bootstrap`, with 1,000 draws and seed
20261010 (`bootstrap/<universe>-<model>-k<k>.json`). Each draw takes n
items with replacement. A drawn item gets one copy per draw under its
original parent, the graph keeps every edge between drawn copies, and the
average-linkage reference is rescored on the same draw. Every run of a
universe and graph is scored on the same draws, and the per-draw values
are committed (`.draws.json.gz`). A difference between two runs is
therefore computed draw by draw:

- the interval is the 2.5% and 97.5% percentiles of the paired
  differences (Efron 1979, https://doi.org/10.1214/aos/1176344552);
- p = 2 · min(share ≤ 0, share ≥ 0), shown as "<0.002" when no draw
  crosses zero;
- within each table, p values are Holm-adjusted (Holm 1979,
  *Scandinavian Journal of Statistics* 6: 65–70; TODO(cite): no stable
  copy was verified). A p of 0 counts as 1/1,000 in the adjustment, which
  makes 0.018 the smallest Holm p for a table of 18 tests.

The catalogue scores (triplet, locality, TSD) use the vertex replicates of
`shelfmark-bench scores --boot 1000` with the same seed
(`scores/*.json`, key `catalogue.boot`). The vertex bootstrap gives
intervals about three times wider than the edge bootstrap on the books
(`bench/results/2026-10-10-covered/results.md`, "Bootstrap intervals").

## 3. Availability of numbers

What a structure can achieve depends on how many items arrive with a
librarian's number. The rows below come from:

- the edition records of the 500-book holdout (`bench/corpus/holdout-ddc.jsonl`);
- one Open Library work-level search per work, which aggregates `ddc` and
  `lcc` over all editions of the work. These are cached and committed by
  `ol_work_numbers.py` (`bench/results/2026-10-10-proposal/availability/`).

The "50 well-known books" sample is the one used to evaluate the Open
Library lookup tier, which is on another base and not merged here. On
those 50 books that tier resolved 45 of 50 books against 0 of 50 without
it. The source is the unmerged ref `dot-4xu6b.6`, read with
`git show dot-4xu6b.6:<path>` under `crates/tier-openlibrary/eval/` (the
sample is `crates/tier-openlibrary/eval/books-50.json`). That ref is not
present in this checkout, so its 45/50 could not be re-derived here; the
work-level numbers below were.

<!-- table:availability -->
| sample | level | number | count | share | 95% Wilson interval | source |
| --- | --- | --- | ---: | ---: | ---: | --- |
| holdout (500 editions) | edition record | Dewey number | 110/500 | 0.220 | [0.186, 0.258] | bench/corpus/holdout-ddc.jsonl |
| holdout (500 works) | work (all editions) | Dewey number | 455/500 | 0.910 | [0.882, 0.932] | bench/results/2026-10-10-proposal/availability/holdout-works.jsonl |
| holdout (500 works) | work (all editions) | LC call number | 500/500 | 1.000 | [0.992, 1.000] | bench/results/2026-10-10-proposal/availability/holdout-works.jsonl |
| holdout (500 works) | work (all editions) | both | 455/500 | 0.910 | [0.882, 0.932] | bench/results/2026-10-10-proposal/availability/holdout-works.jsonl |
| 50 well-known books | work (all editions) | Dewey number | 50/50 | 1.000 | [0.929, 1.000] | bench/results/2026-10-10-proposal/availability/books50-works.jsonl |
| 50 well-known books | work (all editions) | LC call number | 50/50 | 1.000 | [0.929, 1.000] | bench/results/2026-10-10-proposal/availability/books50-works.jsonl |
| 50 well-known books | work (all editions) | both | 50/50 | 1.000 | [0.929, 1.000] | bench/results/2026-10-10-proposal/availability/books50-works.jsonl |
| holdout (500 works) | edition, else work consensus | FDC class reached | 455/500 | 0.910 | [0.882, 0.932] | bench/results/2026-10-10-proposal/placements/holdout/fdc-work.jsonl |
<!-- /table:availability -->

**The work level matters.** A single edition record carries a Dewey
number for 22% of the holdout. The work (all editions) carries one for
91%, and LC call numbers for 100% (that holdout was sampled for having
one). The table's last row checks that the edition-else-work consensus
number reaches an FDC class for every work that has one. The consensus is
the majority three-digit section, then the longest common prefix of its
numbers: `consensus_ddc` in `kb_proposal_runs.py`. Using it raises the
holdout's Dewey cost from 0.976 to 0.70–0.72 (`holdout`, row
`fdc-work` − `fdc-ddc`, −0.25 to −0.27 on all three graphs).

## 4. Scheme comparison on 3,000 books

Sources: `bootstrap/books-minilm-k20.json` (the 2026-10-10-ddc graph and
placements, plus the cuts of this report) and `scores/books-*.json`. All
gold runs have coverage 1.000. The graph text is mostly Library of
Congress subject headings, which, if anything, favours LCC
(`bench/results/2026-10-10-ddc/results.md`, Caveats).

### 4.1 Main results

<!-- table:books-main -->
| run | structure | coverage | cost_norm [95% CI] | skill [95% CI] | nav bits | mean depth | effective classes | J (λ = 0.002) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ddc-gold` | Dewey, librarian number, full depth | 1.000 | 0.4929 [0.4752, 0.5113] | 0.658 [0.636, 0.680] | 19.31 | 7.82 | 1656.7 | 0.5315 |
| `ddc-section` | Dewey, cut to section (3 digits) | 1.000 | 0.4941 [0.4767, 0.5125] | 0.657 [0.635, 0.678] | 17.14 | 4.00 | 123.1 | 0.5284 |
| `fdc-ddc` | FDC from Dewey, full FDC depth | 1.000 | 0.4958 [0.4786, 0.5139] | 0.654 [0.633, 0.676] | 18.07 | 3.94 | 109.7 | 0.5320 |
| `fdc-d3` | FDC from Dewey, depth 3 | 1.000 | 0.4964 [0.4791, 0.5144] | 0.654 [0.632, 0.676] | 17.69 | 3.77 | 92.4 | 0.5318 |
| `fdc-d2` | FDC from Dewey, depth 2 | 1.000 | 0.5017 [0.4845, 0.5198] | 0.647 [0.626, 0.668] | 16.12 | 2.99 | 36.0 | 0.5339 |
| `lcc-gold` | LCC, librarian call number | 1.000 | 0.5233 [0.5060, 0.5413] | 0.619 [0.598, 0.639] | 16.19 | 4.12 | 298.2 | 0.5557 |
| `learned-own` | learned scheme, own insertion | 1.000 | 0.3390 [0.3248, 0.3541] | 0.858 [0.839, 0.878] | 29.72 | 11.31 | 37.8 | 0.3984 |
| `fdc-placed` | FDC, lexical@2 classifier | 0.839 | 0.6537 [0.6369, 0.6711] | 0.450 [0.428, 0.471] | 23.37 | 3.57 | 69.8 | 0.7005 |
| `lcc-placed` | LCC, lexical@2 classifier | 0.915 | 0.7334 [0.7184, 0.7490] | 0.346 [0.326, 0.365] | 20.55 | 4.11 | 271.0 | 0.7745 |
| `unesco-thesaurus-placed` | UNESCO thesaurus, lexical@2 classifier | 0.925 | 0.6287 [0.6116, 0.6456] | 0.482 [0.459, 0.504] | 22.17 | 4.50 | 196.4 | 0.6731 |
<!-- /table:books-main -->

### 4.1.1 Query workloads, information bounds and assignment null

The following uses the same 3,000 sampled editions and all three k=20
embedding graphs. Menu access, depth and item-scan expectations use uniform
queries; a seeded Zipf(s=1) workload and its item-bootstrap intervals are
available in the run records. The corpus has one sampled edition per work,
so it supplies no edition-count popularity proxy. Logarithms are base 2.
The alphabetic optimum uses the Knuth-optimized O(n²) dynamic program.
Permutation nulls keep the tree shape and class sizes while shuffling
item-to-class assignments (200 permutations, seed 20261010). All displayed
intervals are 95% bootstrap intervals; the theoretical entropy and alphabetic
bounds condition on the observed leaf distribution. The baseline MiniLM
cost_norm values reproduce exactly, so the original scheme ranking and cut
recommendation do not change; these additions quantify query workloads,
lower-bound overhead and how far each real assignment sits from its null.

<!-- table:metrics-upgrade -->
<!-- /table:metrics-upgrade -->

### 4.2 Paired tests

Δ is a − b: negative means a is better. The last column is the effect as
a share of b's cost. In items, the Dewey-section-versus-LCC cost gap of
0.0292 (§5.1) is 0.0292 × 3,000 ≈ 88 fewer items under the meeting point
of an average similar pair.

<!-- table:paired-books -->
| comparison | score | Δ | 95% CI | p | p (Holm) | Δ / cost of b |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `fdc-ddc` − `lcc-gold` | cost_norm | -0.0275 | [-0.0421, -0.0122] | 0.002 | 0.014 | -5.3% |
| `fdc-ddc` − `lcc-gold` | J (λ = 0.002) | -0.0238 | [-0.0385, -0.0087] | 0.002 | 0.014 |  |
| `fdc-d3` − `lcc-gold` | cost_norm | -0.0270 | [-0.0415, -0.0118] | 0.002 | 0.014 | -5.2% |
| `fdc-d3` − `lcc-gold` | J (λ = 0.002) | -0.0240 | [-0.0386, -0.0091] | 0.002 | 0.014 |  |
| `fdc-d3` − `ddc-section` | cost_norm | +0.0022 | [+0.0016, +0.0030] | <0.002 | 0.014 | +0.5% |
| `fdc-d3` − `ddc-section` | J (λ = 0.002) | +0.0033 | [+0.0026, +0.0041] | <0.002 | 0.014 |  |
| `fdc-d3` − `fdc-ddc` | cost_norm | +0.0005 | [+0.0003, +0.0008] | <0.002 | 0.014 | +0.1% |
| `fdc-d3` − `fdc-ddc` | J (λ = 0.002) | -0.0002 | [-0.0004, -0.0000] | 0.032 | 0.032 |  |
| `fdc-d3` − `fdc-d2` | cost_norm | -0.0053 | [-0.0061, -0.0046] | <0.002 | 0.014 | -1.1% |
| `fdc-d3` − `fdc-d2` | J (λ = 0.002) | -0.0022 | [-0.0030, -0.0015] | <0.002 | 0.014 |  |
| `fdc-d3` − `fdc-placed` | cost_norm | -0.1573 | [-0.1783, -0.1370] | <0.002 | 0.014 | -24.1% |
| `fdc-d3` − `fdc-placed` | J (λ = 0.002) | -0.1687 | [-0.1898, -0.1487] | <0.002 | 0.014 |  |
| `learned-own` − `fdc-d3` | cost_norm | -0.1574 | [-0.1792, -0.1355] | <0.002 | 0.014 | -31.7% |
| `learned-own` − `fdc-d3` | J (λ = 0.002) | -0.1333 | [-0.1565, -0.1131] | <0.002 | 0.014 |  |
<!-- /table:paired-books -->

The DDC run's own paired differences (`bench/results/2026-10-10-ddc/bootstrap/books.json`)
agree with this:

<!-- table:ddc-reference -->
| comparison | score | Δ | 95% CI | share of draws a lower |
| --- | --- | ---: | ---: | ---: |
| `ddc-gold` − `ddc-section` | cost_norm | -0.0012 | [-0.0015, -0.0010] | 1.000 |
| `ddc-gold` − `lcc-gold` | cost_norm | -0.0304 | [-0.0449, -0.0155] | 0.999 |
| `ddc-gold` − `fdc-ddc` | cost_norm | -0.0029 | [-0.0037, -0.0023] | 1.000 |
| `ddc-gold` − `learned-own` | cost_norm | +0.1539 | [+0.1322, +0.1758] | 0.000 |
| `fdc-ddc` − `fdc-placed` | cost_norm | -0.1579 | [-0.1787, -0.1376] | 1.000 |
| `ddc-gold` − `ddc-section` | nav_log_a1 | +2.17 | [+2.06, +2.25] | 0.000 |
| `ddc-gold` − `lcc-gold` | nav_log_a1 | +3.12 | [+2.88, +3.19] | 0.000 |
| `ddc-gold` − `fdc-ddc` | nav_log_a1 | +1.24 | [+1.07, +1.41] | 0.000 |
| `ddc-gold` − `learned-own` | nav_log_a1 | -10.41 | [-10.66, -7.81] | 1.000 |
| `fdc-ddc` − `fdc-placed` | nav_log_a1 | -5.30 | [-5.70, -5.04] | 1.000 |
<!-- /table:ddc-reference -->

### 4.3 Secondary scores

Source: `scores/books-*.json`, MiniLM k = 20.

<!-- table:catalogue-books -->
| score | `fdc-d3` | `fdc-ddc` | `ddc-section` | `lcc-gold` | `learned-own` | `fdc-placed` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cost_norm (lower better) | 0.4964 | 0.4958 | 0.4941 | 0.5233 | 0.3390 | 0.6537 |
| triplet accuracy (higher better) | 0.6945 | 0.6948 | 0.6957 | 0.6991 | 0.7432 | 0.6117 |
| shelf-order locality (higher better) | 0.4118 | 0.4125 | 0.4130 | 0.4734 | 0.4982 | 0.3285 |
| tree-sampling divergence, nats (higher better) | 0.885 | 0.905 | 0.933 | 1.242 | 0.938 | 0.504 |
| hospitality in array | 0.857 | 0.857 | 0.701 | 1.000 | 0.000 | 0.857 |
| splits per 1,000 inserts | 3.333 | 2.333 | 2.333 | 1.667 | 11.333 | 4.766 |
| notation length, characters | 3.265 | 3.286 | 3.000 | 7.644 | 29.923 | 3.563 |
| expected search bits | 9.276 | 9.780 | 9.337 | 8.742 | 13.297 | 9.022 |
<!-- /table:catalogue-books -->

<!-- table:catalogue-tests -->
| comparison | score | Δ | 95% CI (vertex) | p | p (Holm) |
| --- | --- | ---: | ---: | ---: | ---: |
| `ddc-section` − `lcc-gold` | cost_norm | -0.0292 | [-0.0443, -0.0135] | <0.002 | 0.012 |
| `ddc-section` − `lcc-gold` | triplet accuracy | -0.0034 | [-0.0119, +0.0053] | 0.410 | 0.864 |
| `ddc-section` − `lcc-gold` | shelf-order locality | -0.0604 | [-0.0886, -0.0311] | <0.002 | 0.012 |
| `ddc-section` − `lcc-gold` | tree-sampling divergence, nats | -0.3090 | [-0.3457, -0.2573] | <0.002 | 0.012 |
| `fdc-d3` − `lcc-gold` | cost_norm | -0.0270 | [-0.0419, -0.0113] | 0.002 | 0.012 |
| `fdc-d3` − `lcc-gold` | triplet accuracy | -0.0046 | [-0.0131, +0.0042] | 0.288 | 0.864 |
| `fdc-d3` − `lcc-gold` | shelf-order locality | -0.0616 | [-0.0906, -0.0322] | <0.002 | 0.012 |
| `fdc-d3` − `lcc-gold` | tree-sampling divergence, nats | -0.3572 | [-0.3938, -0.3052] | <0.002 | 0.012 |
| `learned-own` − `ddc-section` | cost_norm | -0.1551 | [-0.1773, -0.1321] | <0.002 | 0.012 |
| `learned-own` − `ddc-section` | triplet accuracy | +0.0476 | [+0.0336, +0.0614] | <0.002 | 0.012 |
| `learned-own` − `ddc-section` | shelf-order locality | +0.0852 | [+0.0445, +0.1231] | <0.002 | 0.012 |
| `learned-own` − `ddc-section` | tree-sampling divergence, nats | +0.0050 | [-0.0650, +0.0762] | 0.876 | 0.876 |
<!-- /table:catalogue-tests -->

Dewey (sections, or FDC depth 3) beats LCC on cost. The two tie on
triplet accuracy. **LCC is better on shelf-order locality (by 0.06) and
on TSD.** LCC's longer call numbers order books more finely along a line,
and its 298 effective classes concentrate edge mass. The scores therefore
disagree on what "better" means:

- C rewards meeting low in the *tree*;
- locality rewards adjacency on a *line*.

For directories and notes, the user moves through a tree, so C and N are
the right costs. For one long physical shelf, locality matters too. The
table below gives the weight μ on locality at which J − μ · locality
would prefer LCC:

<!-- table:locality-weight -->
| comparison | Δ J (λ = 0.002) | Δ locality | μ* (LCC preferred above) |
| --- | ---: | ---: | ---: |
| `ddc-section` − `lcc-gold` | -0.0273 | -0.0604 | 0.45 |
| `fdc-d3` − `lcc-gold` | -0.0240 | -0.0616 | 0.39 |
<!-- /table:locality-weight -->

LCC would win only if one unit of locality were worth more than about
0.45 units of normalised cost. Locality runs from 0 (random) to 1 (every
neighbour adjacent), and the whole J spread between the gold schemes is
0.03. Such a weight would make the tree almost irrelevant, which does not
fit a tree-structured knowledge base. The learned scheme beats both on
cost, triplet and locality.

### 4.4 Learned scheme and hybrids

`learned-own` is the learned scheme (`schemes/learned`), with each book
inserted at its nearest leaf centroid using the MiniLM vectors. It is
unconstrained-optimal on J: 0.3984 against 0.5284 for Dewey sections at
λ = 0.002. That lead holds on all nine graphs (`embedding`). But:

- **It fails A2.** Its notation is 29.9 characters, its hospitality is 0,
  it makes 11.3 splits per 1,000 inserts, and it needs 29.7 bits of
  navigation (`catalogue-books`).
- **It fails A3.** Placement needs the embedding model, and part of the
  lead is home advantage. The scheme was fitted on MiniLM. Its margin
  shrinks off that graph: −0.157 on MiniLM against −0.109 and −0.114 on
  bge-small and nomic at k = 20 (`embedding`). Refits on other models
  each win on their own graph (`bench/results/2026-10-10-embed/results.md`,
  "Cross-fit").

A hybrid would keep Dewey sections as the readable backbone and nest the
learned paths inside them. `fdc-learned{1,2,3}` takes each book's FDC
depth-3 section and puts the first 1, 2 or 3 levels of its learned path
beneath it (`schemes/fdc-learned*.json`):

<!-- table:hybrid -->
| graph (k = 20) | comparison | cost a | nav bits a | Δ cost | 95% CI | p (Holm) | Δ J (λ = 0.002) | p (Holm) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| minilm | `fdc-learned1` − `fdc-d3` | 0.4956 | 17.85 | -0.0008 | [-0.0010, -0.0005] | 0.024 | -0.0004 | 0.024 |
| minilm | `fdc-learned2` − `fdc-d3` | 0.4955 | 18.72 | -0.0009 | [-0.0011, -0.0007] | 0.024 | +0.0012 | 0.054 |
| minilm | `fdc-learned3` − `fdc-d3` | 0.4955 | 18.95 | -0.0009 | [-0.0012, -0.0007] | 0.024 | +0.0016 | 0.024 |
| minilm | `learned-own` − `fdc-learned3` | 0.3390 | 29.72 | -0.1564 | [-0.1782, -0.1345] | 0.024 | -0.1349 | 0.024 |
| bge-small | `fdc-learned1` − `fdc-d3` | 0.4846 | 17.85 | -0.0006 | [-0.0009, -0.0005] | 0.024 | -0.0003 | 0.024 |
| bge-small | `fdc-learned2` − `fdc-d3` | 0.4844 | 18.72 | -0.0008 | [-0.0010, -0.0006] | 0.024 | +0.0013 | 0.054 |
| bge-small | `fdc-learned3` − `fdc-d3` | 0.4844 | 18.95 | -0.0008 | [-0.0010, -0.0006] | 0.024 | +0.0017 | 0.024 |
| bge-small | `learned-own` − `fdc-learned3` | 0.3758 | 29.72 | -0.1086 | [-0.1302, -0.0867] | 0.024 | -0.0871 | 0.024 |
| nomic | `fdc-learned1` − `fdc-d3` | 0.4748 | 17.85 | -0.0007 | [-0.0009, -0.0005] | 0.024 | -0.0004 | 0.024 |
| nomic | `fdc-learned2` − `fdc-d3` | 0.4747 | 18.72 | -0.0008 | [-0.0011, -0.0006] | 0.024 | +0.0012 | 0.054 |
| nomic | `fdc-learned3` − `fdc-d3` | 0.4746 | 18.95 | -0.0009 | [-0.0011, -0.0007] | 0.024 | +0.0016 | 0.024 |
| nomic | `learned-own` − `fdc-learned3` | 0.3613 | 29.72 | -0.1134 | [-0.1360, -0.0914] | 0.024 | -0.0918 | 0.024 |
<!-- /table:hybrid -->

Nesting buys 0.0006 to 0.0009 of cost on every graph. Once it adds two or
more levels it raises J instead: by about 0.0012 at two levels (Holm
p = 0.054) and significantly at three. The learned scheme's advantage comes from
grouping books *across* Dewey sections, which no nesting under them can
recover. **The hybrid is not recommended.** The learned scheme stays
useful as a "related items" view next to the tree, where A2 and A3 do
not apply.

## 5. Sensitivity

### 5.1 Embedding model and k

Sources: `bootstrap/books-<model>-k<k>.json`, 9 graphs, plus the books
graphs this report built for bge-small and nomic
(`graphs/*.manifest.json`). In this table, "minimax depth" is computed
over the FDC cuts only.

<!-- table:embedding -->
| graph | FDC d3 | LCC | Δ FDC − LCC | 95% CI | p (Holm) | learned | Δ learned − FDC | 95% CI | p (Holm) | minimax depth | λ* FDC = LCC |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| minilm k = 10 | 0.4164 | 0.4022 | +0.0142 | [-0.0030, +0.0311] | 0.224 | 0.3142 | -0.1022 | [-0.1280, -0.0768] | 0.018 | `fdc-d3` | never |
| minilm k = 20 | 0.4964 | 0.5233 | -0.0270 | [-0.0415, -0.0118] | 0.018 | 0.3390 | -0.1574 | [-0.1792, -0.1355] | 0.018 | `fdc-d3` | 0.0180 |
| minilm k = 50 | 0.5841 | 0.6460 | -0.0618 | [-0.0736, -0.0498] | 0.018 | 0.3805 | -0.2036 | [-0.2205, -0.1851] | 0.018 | `fdc-d2` | 0.0413 |
| bge-small k = 10 | 0.4046 | 0.4011 | +0.0035 | [-0.0125, +0.0202] | 0.704 | 0.3445 | -0.0602 | [-0.0836, -0.0354] | 0.018 | `fdc-d3` | never |
| bge-small k = 20 | 0.4852 | 0.5244 | -0.0392 | [-0.0534, -0.0253] | 0.018 | 0.3758 | -0.1094 | [-0.1310, -0.0876] | 0.018 | `fdc-d3` | 0.0262 |
| bge-small k = 50 | 0.5727 | 0.6470 | -0.0743 | [-0.0859, -0.0626] | 0.018 | 0.4187 | -0.1540 | [-0.1712, -0.1359] | 0.018 | `fdc-d2` | 0.0497 |
| nomic k = 10 | 0.3953 | 0.3732 | +0.0221 | [+0.0050, +0.0387] | 0.030 | 0.3352 | -0.0601 | [-0.0857, -0.0347] | 0.018 | `fdc-d3` | never |
| nomic k = 20 | 0.4755 | 0.4972 | -0.0216 | [-0.0365, -0.0058] | 0.024 | 0.3613 | -0.1142 | [-0.1369, -0.0924] | 0.018 | `fdc-d3` | 0.0145 |
| nomic k = 50 | 0.5637 | 0.6231 | -0.0594 | [-0.0715, -0.0467] | 0.018 | 0.4003 | -0.1635 | [-0.1814, -0.1455] | 0.018 | `fdc-d2` | 0.0397 |
<!-- /table:embedding -->

Dewey's three-digit sections against the FDC-tree cut, and minimax regret
over every FDC and Dewey cut:

<!-- table:dewey-cut -->
| graph | Δ cost `ddc-section` − `fdc-d3` | 95% CI | p (Holm) | Δ J (λ = 0.002) | 95% CI | p (Holm) | minimax cut | max regret `ddc-section` | max regret `ddc-d2` | max regret `fdc-d3` | λ where `ddc-d2` = `ddc-section` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| minilm k = 10 | -0.0031 | [-0.0040, -0.0023] | 0.018 | -0.0042 | [-0.0051, -0.0033] | 0.018 | `ddc-section` | 0.0009 | 0.0069 | 0.0068 | 0.0044 |
| minilm k = 20 | -0.0022 | [-0.0030, -0.0016] | 0.018 | -0.0033 | [-0.0041, -0.0026] | 0.018 | `ddc-section` | 0.0029 | 0.0045 | 0.0079 | 0.0032 |
| minilm k = 50 | -0.0013 | [-0.0018, -0.0009] | 0.018 | -0.0024 | [-0.0030, -0.0019] | 0.018 | `ddc-d2` | 0.0047 | 0.0023 | 0.0088 | 0.0020 |
| bge-small k = 10 | -0.0028 | [-0.0038, -0.0020] | 0.018 | -0.0039 | [-0.0049, -0.0030] | 0.018 | `ddc-section` | 0.0009 | 0.0070 | 0.0065 | 0.0044 |
| bge-small k = 20 | -0.0018 | [-0.0026, -0.0012] | 0.018 | -0.0029 | [-0.0036, -0.0023] | 0.018 | `ddc-section` | 0.0030 | 0.0045 | 0.0076 | 0.0031 |
| bge-small k = 50 | -0.0010 | [-0.0016, -0.0005] | 0.018 | -0.0021 | [-0.0028, -0.0015] | 0.018 | `ddc-d2` | 0.0048 | 0.0023 | 0.0086 | 0.0019 |
| nomic k = 10 | -0.0030 | [-0.0039, -0.0022] | 0.018 | -0.0041 | [-0.0050, -0.0033] | 0.018 | `ddc-section` | 0.0009 | 0.0073 | 0.0063 | 0.0046 |
| nomic k = 20 | -0.0023 | [-0.0030, -0.0016] | 0.018 | -0.0034 | [-0.0041, -0.0027] | 0.018 | `ddc-section` | 0.0027 | 0.0048 | 0.0077 | 0.0033 |
| nomic k = 50 | -0.0014 | [-0.0019, -0.0009] | 0.018 | -0.0025 | [-0.0030, -0.0019] | 0.018 | `ddc-d2` | 0.0046 | 0.0026 | 0.0087 | 0.0021 |
<!-- /table:dewey-cut -->

Dewey sections against LCC on every graph:

<!-- table:section-lcc -->
| graph | `ddc-section` | `lcc-gold` | Δ cost | 95% CI | p (Holm) | Δ J (λ = 0.002) | 95% CI | p (Holm) | λ where J equal |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| minilm k = 10 | 0.4133 | 0.4022 | +0.0111 | [-0.0060, +0.0281] | 0.678 | +0.0130 | [-0.0043, +0.0299] | 0.600 | never |
| minilm k = 20 | 0.4941 | 0.5233 | -0.0292 | [-0.0438, -0.0141] | 0.020 | -0.0273 | [-0.0421, -0.0123] | 0.020 | 0.0309 |
| minilm k = 50 | 0.5828 | 0.6460 | -0.0632 | [-0.0748, -0.0513] | 0.018 | -0.0613 | [-0.0731, -0.0495] | 0.018 | 0.0669 |
| bge-small k = 10 | 0.4018 | 0.4011 | +0.0007 | [-0.0151, +0.0175] | 1.000 | +0.0026 | [-0.0131, +0.0191] | 1.000 | never |
| bge-small k = 20 | 0.4834 | 0.5244 | -0.0411 | [-0.0551, -0.0272] | 0.018 | -0.0392 | [-0.0534, -0.0256] | 0.018 | 0.0435 |
| bge-small k = 50 | 0.5716 | 0.6470 | -0.0753 | [-0.0867, -0.0636] | 0.018 | -0.0734 | [-0.0849, -0.0619] | 0.018 | 0.0798 |
| nomic k = 10 | 0.3923 | 0.3732 | +0.0191 | [+0.0021, +0.0356] | 0.140 | +0.0210 | [+0.0039, +0.0373] | 0.060 | never |
| nomic k = 20 | 0.4732 | 0.4972 | -0.0239 | [-0.0386, -0.0084] | 0.032 | -0.0220 | [-0.0368, -0.0066] | 0.032 | 0.0253 |
| nomic k = 50 | 0.5623 | 0.6231 | -0.0608 | [-0.0726, -0.0480] | 0.018 | -0.0589 | [-0.0709, -0.0462] | 0.018 | 0.0644 |
<!-- /table:section-lcc -->

What these three tables show:

- **Dewey sections against the FDC tree.** Sections beat the FDC tree cut
  at depth 3 on all nine graphs. The gap is 0.001 to 0.003 of cost: small
  but always significant. FDC's tree sometimes stops at a tens class or a
  range where the librarian's number names a section.
- **The best cut.** The minimax cut is three digits on six graphs and two
  digits at k = 50. Worst-case regret of the section cut is at most
  0.0048 in any condition.
- **Dewey sections against LCC.** At k = 20 and k = 50, Dewey sections
  beat LCC on every model, by 0.024 to 0.075 of cost. At k = 10, nomic
  favours LCC before correction (+0.019 [+0.002, +0.036]), and none of the
  k = 10 differences survives Holm's correction. With only the ten
  nearest neighbours, LCC's finer classes catch the closest pairs.
  Browsing a knowledge base reaches beyond the ten nearest items, so k = 20
  is the working condition.

### 5.2 Books vs papers corpora, and the title-only holdout

**The 500-book holdout** has a title-only graph from Paper 1. Source:
`bootstrap/holdout-<model>-k20.json` and `scores/holdout-both-*.json`, the
424 books that both `fdc-work` and `lcc-gold` place
(`scores/holdout-both-ids.txt`).

<!-- table:holdout -->
| graph (k = 20) | comparison | coverage a | cost a | coverage b | cost b | Δ | 95% CI | p | p (Holm) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| minilm | `fdc-work` − `lcc-gold` | 0.910 | 0.7237 | 0.922 | 0.7076 | +0.0161 | [-0.0087, +0.0423] | 0.268 | 1.000 |
| minilm | `fdc-pipeline` − `lcc-pipeline` | 0.912 | 0.7237 | 0.932 | 0.7071 | +0.0167 | [-0.0088, +0.0435] | 0.246 | 1.000 |
| minilm | `fdc-work` − `fdc-ddc` | 0.910 | 0.7237 | 0.220 | 0.9759 | -0.2521 | [-0.2791, -0.2244] | <0.002 | 0.012 |
| minilm | `fdc-work` − `lcc-gold`, the 424 books both place | 1.000 | 0.6970 | 1.000 | 0.7074 | -0.0103 | [-0.0424, +0.0189] | 0.524 | 1.000 |
| bge-small | `fdc-work` − `lcc-gold` | 0.910 | 0.7165 | 0.922 | 0.7132 | +0.0033 | [-0.0235, +0.0280] | 0.816 | 1.000 |
| bge-small | `fdc-pipeline` − `lcc-pipeline` | 0.912 | 0.7165 | 0.932 | 0.7128 | +0.0037 | [-0.0232, +0.0281] | 0.796 | 1.000 |
| bge-small | `fdc-work` − `fdc-ddc` | 0.910 | 0.7165 | 0.220 | 0.9760 | -0.2596 | [-0.2884, -0.2317] | <0.002 | 0.012 |
| bge-small | `fdc-work` − `lcc-gold`, the 424 books both place | 1.000 | 0.6966 | 1.000 | 0.7096 | -0.0131 | [-0.0428, +0.0139] | 0.374 | 1.000 |
| nomic | `fdc-work` − `lcc-gold` | 0.910 | 0.7021 | 0.922 | 0.7072 | -0.0052 | [-0.0323, +0.0212] | 0.660 | 1.000 |
| nomic | `fdc-pipeline` − `lcc-pipeline` | 0.912 | 0.7020 | 0.932 | 0.7070 | -0.0050 | [-0.0321, +0.0211] | 0.678 | 1.000 |
| nomic | `fdc-work` − `fdc-ddc` | 0.910 | 0.7021 | 0.220 | 0.9762 | -0.2742 | [-0.3045, -0.2452] | <0.002 | 0.012 |
| nomic | `fdc-work` − `lcc-gold`, the 424 books both place | 1.000 | 0.6861 | 1.000 | 0.7047 | -0.0186 | [-0.0479, +0.0071] | 0.166 | 1.000 |
<!-- /table:holdout -->

On titles alone, Dewey-via-FDC and LCC do not differ. This holds raw
(coverage 0.910 against 0.922) and on the 424 books both place, on every
model; all Holm p = 1.000. The coverage-controlled sign favours FDC on all
three models, but none is significant. The holdout cannot rank the two
schemes, and it agrees with the books' result only in sign.

**Papers** are the 10,000 arXiv papers. Each paper's arXiv category is
sent through a crosswalk written for this report from the FDC captions
(`ARXIV_FDC` in `kb_proposal_runs.py`, mirrored to LCC as `FDC_LCC`).
Each paper is either shelved at the FDC class (`fdc-arxiv`) or with its
arXiv category nested under that class (`fdc-arxiv-nested`). Sources:
`bootstrap/corpus-<model>-k20.json` and the `2026-10-10-embed` placements.

<!-- table:papers -->
| run | minilm cost / nav | bge-small cost / nav | nomic cost / nav | coverage |
| --- | ---: | ---: | ---: | ---: |
| `arxiv-gold` | 0.3333 / 18.33 | 0.3327 / 18.33 | 0.3465 / 18.33 | 1.000 |
| `fdc-arxiv-nested` | 0.3139 / 19.30 | 0.3105 / 19.30 | 0.3247 / 19.30 | 1.000 |
| `lcc-arxiv-nested` | 0.3396 / 22.74 | 0.3401 / 22.74 | 0.3549 / 22.74 | 1.000 |
| `fdc-arxiv` | 0.4000 / 18.35 | 0.3934 / 18.35 | 0.4048 / 18.35 | 1.000 |
| `lcc-arxiv` | 0.4448 / 24.06 | 0.4432 / 24.06 | 0.4505 / 24.06 | 1.000 |
| `learned-placed` | 0.3794 / 31.08 | 0.3852 / 31.08 | 0.3873 / 31.08 | 0.999 |
| `openalex-topics-placed` | 0.6414 / 26.85 | 0.6457 / 26.85 | 0.6434 / 26.85 | 0.973 |
| `fdc-placed` | 0.8337 / 25.56 | 0.8348 / 25.56 | 0.8351 / 25.56 | 0.770 |
<!-- /table:papers -->

<!-- table:papers-tests -->
| graph (k = 20) | comparison | Δ cost | 95% CI | p (Holm) | Δ J (λ = 0.002) | 95% CI | p (Holm) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| minilm | `fdc-arxiv-nested` − `arxiv-gold` | -0.0194 | [-0.0222, -0.0167] | 0.018 | -0.0174 | [-0.0203, -0.0148] | 0.018 |
| minilm | `fdc-arxiv-nested` − `lcc-arxiv-nested` | -0.0257 | [-0.0292, -0.0222] | 0.018 | -0.0326 | [-0.0361, -0.0290] | 0.018 |
| minilm | `fdc-arxiv-nested` − `fdc-arxiv` | -0.0861 | [-0.0907, -0.0817] | 0.018 | -0.0842 | [-0.0888, -0.0799] | 0.018 |
| bge-small | `fdc-arxiv-nested` − `arxiv-gold` | -0.0222 | [-0.0254, -0.0192] | 0.018 | -0.0203 | [-0.0235, -0.0172] | 0.018 |
| bge-small | `fdc-arxiv-nested` − `lcc-arxiv-nested` | -0.0296 | [-0.0340, -0.0255] | 0.018 | -0.0365 | [-0.0408, -0.0324] | 0.018 |
| bge-small | `fdc-arxiv-nested` − `fdc-arxiv` | -0.0829 | [-0.0874, -0.0787] | 0.018 | -0.0810 | [-0.0855, -0.0770] | 0.018 |
| nomic | `fdc-arxiv-nested` − `arxiv-gold` | -0.0218 | [-0.0250, -0.0188] | 0.018 | -0.0199 | [-0.0231, -0.0169] | 0.018 |
| nomic | `fdc-arxiv-nested` − `lcc-arxiv-nested` | -0.0302 | [-0.0342, -0.0263] | 0.018 | -0.0371 | [-0.0410, -0.0332] | 0.018 |
| nomic | `fdc-arxiv-nested` − `fdc-arxiv` | -0.0801 | [-0.0845, -0.0760] | 0.018 | -0.0782 | [-0.0826, -0.0743] | 0.018 |
<!-- /table:papers-tests -->

Nesting arXiv categories under the book backbone gives papers a lower
cost than arXiv's own taxonomy on every model. It also beats the same
nesting under LCC, and it beats FDC without nesting by 0.08. So **papers
join the same tree, one level below their FDC class**. They need their
own leaf vocabulary, because FDC alone is too coarse for them (`fdc-arxiv`
0.39–0.40).

### 5.3 Coverage-controlled vs raw, and the classifier

The covered-subset run (`bench/results/2026-10-10-covered/results.md`,
Conclusions) found that coverage explains 59% of the rank variance of
cost_norm across placed paper runs. On the 500-book holdout's placed runs,
the cost ranking *is* the coverage ranking (ρ = −1.00). Rankings of placed
schemes must not be read as rankings of the schemes. This report avoids
that in three ways:

- the scheme comparison uses gold numbers at coverage 1.000 (§4);
- the holdout comparison is repeated on both-place items (§5.2);
- the classifier's effect is measured end to end as a pipeline.

In a pipeline, a seeded share of books keeps its librarian number, the
same books for both schemes (`placements/books/{fdc,lcc}-pipe-NN.jsonl`),
and the lexical classifier places the rest:

<!-- table:pipelines -->
| books with a librarian number | FDC coverage | FDC cost | LCC coverage | LCC cost | Δ cost FDC − LCC | 95% CI | p (Holm) | Δ J (λ = 0.002) | p (Holm) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 20% | 0.877 | 0.6516 | 0.934 | 0.7396 | -0.0880 | [-0.1061, -0.0680] | 0.014 | -0.0820 | 0.014 |
| 40% | 0.905 | 0.6357 | 0.950 | 0.7206 | -0.0850 | [-0.1034, -0.0652] | 0.014 | -0.0788 | 0.014 |
| 60% | 0.938 | 0.6011 | 0.967 | 0.6780 | -0.0769 | [-0.0942, -0.0580] | 0.014 | -0.0703 | 0.014 |
| 80% | 0.968 | 0.5610 | 0.981 | 0.6218 | -0.0607 | [-0.0774, -0.0426] | 0.014 | -0.0543 | 0.014 |
| 90% | 0.986 | 0.5309 | 0.991 | 0.5764 | -0.0455 | [-0.0621, -0.0285] | 0.014 | -0.0395 | 0.014 |
| 100% (gold) | 1.000 | 0.4958 | 1.000 | 0.5233 | -0.0275 | [-0.0421, -0.0122] | 0.014 | -0.0238 | 0.014 |
| 0% (classifier only) | 0.839 | 0.6537 | 0.915 | 0.7334 | -0.0797 | [-0.0985, -0.0594] | 0.014 | -0.0741 | 0.014 |
<!-- /table:pipelines -->

FDC stays ahead of LCC at every share. The more books depend on the
classifier, the worse both get: from 0.4958 to 0.6537 for FDC. The
classifier itself, against the librarians' numbers on the 3,000 books
(`2026-10-10-ddc/placements/books/*-placed.jsonl` and `.stats.json`):

<!-- table:classifier -->
| scheme | depth | placed / books | decided without flag | same class as librarian | agreement | 95% Wilson interval |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| FDC | 1 | 2518/3000 | 623 | 1371/2518 | 0.544 | [0.525, 0.564] |
| FDC | 2 | 2518/3000 | 623 | 883/2518 | 0.351 | [0.332, 0.370] |
| FDC | 3 | 2518/3000 | 623 | 529/2518 | 0.210 | [0.195, 0.226] |
| LCC | 1 | 2744/3000 | 195 | 1021/2744 | 0.372 | [0.354, 0.390] |
| LCC | 2 | 2744/3000 | 195 | 655/2744 | 0.239 | [0.223, 0.255] |
| LCC | 3 | 2744/3000 | 195 | 425/2744 | 0.155 | [0.142, 0.169] |
<!-- /table:classifier -->

Lexical placement into FDC agrees with the librarian on the main class
54% of the time and on the depth-3 class 21% of the time. It leaves 16%
of books unplaced. The classifier is not a placement method for this
library, only a hint for the review queue.

**Lexical vs semantic classifier.** The semantic-tier rerun is pending,
so this comparison cannot be made. What would change if it happens is in
§8.

### 5.4 A better fallback: LCC call number to FDC

Nearly every book carries an LC call number, including books with no
Dewey number (§3). An LCC class can therefore be mapped to FDC by
majority vote over the books that carry both. The vote is learned on the
3,000 books (`lcc-gold` and `fdc-ddc` placements). It is tested two ways:

- 5-fold cross-validation, with folds by a hash of the edition id;
- trained on all 3,000 and tested on the 424 holdout works with both
  numbers, a disjoint sample of works.

<!-- table:crosswalk -->
| test | FDC depth | same class as the Dewey number | accuracy | 95% Wilson interval |
| --- | ---: | ---: | ---: | ---: |
| 3,000 books, 5-fold | 1 | 2604/3000 | 0.868 | [0.855, 0.880] |
| 500-book holdout works, trained on the 3,000 | 1 | 379/424 | 0.894 | [0.861, 0.920] |
| 3,000 books, 5-fold | 2 | 2238/3000 | 0.746 | [0.730, 0.761] |
| 500-book holdout works, trained on the 3,000 | 2 | 288/424 | 0.679 | [0.633, 0.722] |
| 3,000 books, 5-fold | 3 | 1838/3000 | 0.613 | [0.595, 0.630] |
| 500-book holdout works, trained on the 3,000 | 3 | 165/424 | 0.389 | [0.344, 0.436] |
<!-- /table:crosswalk -->

At the main class the crosswalk is right 87–89% of the time, against the
classifier's 54%. Deeper, it degrades, so it is applied with a confidence
rule. Find the deepest LCC ancestor of the book's class that has at least
5 training books. Take the deepest FDC depth whose modal class holds at
least 80% of those books. If none qualifies, the book goes to review.
The threshold was picked from a grid over τ ∈ {0.6, 0.7, 0.8, 0.9} and
minimum support ∈ {3, 5, 10}, as the smallest τ with cross-validated
accuracy of at least 0.85 (the selection is not itself held out).

<!-- table:crosswalk-rule -->
| test | decided | share decided | at FDC depth 1 / 2 / 3 | correct | accuracy | 95% Wilson interval |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 3,000 books, 5-fold | 2287/3000 | 0.762 | 748 / 581 / 958 | 1990/2287 | 0.870 | [0.856, 0.883] |
| 500-book holdout works, trained on the 3,000 | 365/424 | 0.861 | 32 / 203 / 130 | 264/365 | 0.723 | [0.675, 0.767] |
<!-- /table:crosswalk-rule -->

On the holdout the rule is less accurate than in cross-validation (0.72).
The holdout's books are spread differently over classes, and its deeper
decisions fail more often. A crosswalk placement is therefore recorded
as provisional. The trained crosswalk is committed as
`bench/results/2026-10-10-proposal/crosswalk/lcc-fdc.json`. The table
script recomputes it and fails if it differs.

## 6. Recommendation

### 6.1 Scheme

- **Dewey notation** is the hierarchy: digits only, the numbers
  librarians assigned, taken from catalogue records (report 7).
- **FDC** (`schemes/fdc/fdc.json`, `fdc@0.06`, CC0) gives the captions
  and serves as the target scheme for every item that has no Dewey
  number.

FDC codes are Dewey-shaped decimal numbers, so both feed one tree.
Captions: of the 392 sections the 3,000 books use, most have an FDC class
of the same number. A section without one takes the caption of the FDC
class that contains it, written as "<number> (in <FDC caption>)".

<!-- table:captions -->
| sections used | with an FDC class of the same number | books in those sections |
| ---: | ---: | ---: |
| 392 | 371 (0.946) | 2721/3000 (0.907) |
<!-- /table:captions -->

Not chosen:

- **LCC** loses on J at k ≥ 20 by 0.024–0.075 (`section-lcc`) and wins
  only on locality.
- **Full-depth Dewey** gains 0.0012 of cost for 2.2 more bits of
  navigation (`ddc-reference`), and its J is worse.
- **The learned scheme** fails A2 and A3 (§4.4).
- **Hybrids** gain under 0.001 (§4.4).

### 6.2 Hierarchy depth (cut rule)

- **Libraries of about 2,000 items or more:** three levels, which are
  main class (`8xx`), division (`82x`) and section (`823`).
- **Smaller libraries:** two levels (main class and division). With
  λ = κ / n and κ = 6, the J of two digits equals that of three at
  λ = 0.0031–0.0033 on the k = 20 graphs (`dewey-cut`, last column),
  which is n ≈ 6 / 0.0032 ≈ 1,900.
- **Papers:** one more level, the arXiv category, below their section or
  division (§5.2).

Sources: `bootstrap/books-minilm-k20.json` (`placements/books/fdc-d*`,
`fdc-m*`, `ddc-d*`). Regret is over λ ∈ [0.0005, 0.005] among the FDC
cuts. The last column is the λ at which a run's J equals that of
`fdc-d3`.

<!-- table:depth -->
| run | cost_norm | nav bits | eff. classes | J, λ = 0.0 | J, λ = 0.0005 | J, λ = 0.002 | J, λ = 0.005 | J, λ = 0.01 | max regret, FDC cuts | λ where J = J(`fdc-d3`) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `fdc-d1` | 0.5536 | 14.05 | 7.1 | 0.5536 | 0.5606 | 0.5817 | 0.6238 | 0.6940 | 0.0557 | 0.0157 |
| `fdc-d2` | 0.5017 | 16.12 | 36.0 | 0.5017 | 0.5097 | 0.5339 | 0.5823 | 0.6629 | 0.0049 | 0.0034 |
| `fdc-d3` | 0.4964 | 17.69 | 92.4 | 0.4964 | 0.5052 | 0.5318 | 0.5848 | 0.6733 | 0.0025 |  |
| `fdc-ddc` | 0.4958 | 18.07 | 109.7 | 0.4958 | 0.5049 | 0.5320 | 0.5862 | 0.6765 | 0.0039 | 0.0014 |
| `fdc-m5` | 0.4960 | 18.39 | 88.2 | 0.4960 | 0.5052 | 0.5327 | 0.5879 | 0.6799 | 0.0056 | 0.0006 |
| `fdc-m10` | 0.4969 | 18.74 | 67.6 | 0.4969 | 0.5063 | 0.5344 | 0.5906 | 0.6843 | 0.0083 |  |
| `fdc-m25` | 0.5021 | 18.90 | 35.2 | 0.5021 | 0.5116 | 0.5399 | 0.5966 | 0.6911 | 0.0143 |  |
| `fdc-m50` | 0.5078 | 18.24 | 22.9 | 0.5078 | 0.5170 | 0.5443 | 0.5990 | 0.6902 | 0.0167 |  |
| `ddc-d1` | 0.5536 | 14.05 | 7.1 | 0.5536 | 0.5606 | 0.5817 | 0.6238 | 0.6940 |  | 0.0157 |
| `ddc-d2` | 0.4991 | 15.57 | 37.0 | 0.4991 | 0.5069 | 0.5302 | 0.5770 | 0.6548 |  | 0.0013 |
| `ddc-section` | 0.4941 | 17.14 | 123.1 | 0.4941 | 0.5027 | 0.5284 | 0.5798 | 0.6655 |  |  |
| `ddc-d4` | 0.4932 | 18.24 | 283.7 | 0.4932 | 0.5024 | 0.5297 | 0.5844 | 0.6756 |  | 0.0058 |
| `ddc-gold` | 0.4929 | 19.31 | 1656.7 | 0.4929 | 0.5026 | 0.5315 | 0.5895 | 0.6860 |  | 0.0021 |
<!-- /table:depth -->

Never cut deeper: digits beyond the section cost about 1.1 to 2.2 bits
of navigation for at most 0.0012 of cost (`depth`). The full number is
kept as metadata and orders items within their directory, so a later
change of cut needs no reclassification. Size-driven roll-ups
(`fdc-m5`…`fdc-m50`, "place at the deepest class with ≥ m items") were
tested and lose to the fixed cut (`depth`, regret column).

### 6.3 Placement rule

The rule runs per item and in order. The first step that yields a class
wins. Each item records `placement_source` and, where given,
`confidence`.

1. **Identify.**
   - Books: normalise ISBN-10 to ISBN-13, and keep LCCN or OCLC numbers
     if present.
   - Papers: DOI, and arXiv id when there is one.
   - Items with no identifier: look up title + first author in Open
     Library search and accept a match only when exactly one work matches
     both. Otherwise go to step 7.
2. **Open Library edition.** Fetch the edition by ISBN and read
   `dewey_decimal_class` and `lc_classifications`. Keep the first value
   that parses as a number (`ddc_number` in
   `crates/bench/scripts/ddc_numbers.py`).
3. **Open Library work.** If the edition has no Dewey number, run one
   search with `q=key:"/works/<id>"` and fields `ddc,lcc`. Take the
   consensus number: the majority three-digit section, then the longest
   common prefix of its numbers (`consensus_ddc`). Source `work`; it
   reaches 91% of works (§3). Requests must be at least 1 s apart; Open
   Library data is CC0 (`openlibrarylicensing`).
4. **Dewey number → directory.**
   - Strip the number's prefixes and suffixes (`823/.912` → `823.912`).
   - The directory is its first digit, its first two digits and its
     first three digits, cut by §6.2.
   - The caption is the FDC class of the same number, else of the FDC
     class containing it (`fdc_class` in `ddc_numbers.py`).
   - Source `edition` or `work`, no review.
5. **Arrival without Dewey, with an LC call number.**
   - Resolve the call number to the bench LCC class (the deepest class
     whose span covers it, as in Paper 1).
   - Apply `crosswalk/lcc-fdc.json`: start from the deepest LCC ancestor
     listed, and take the deepest FDC depth whose entry has share ≥ 0.8.
     Entries exist only for classes with ≥ 5 books.
   - Map the FDC class to the Dewey directory: a three-digit FDC number
     gives that section; a tens class or a range (`017-019`) gives its
     division; a hundreds class gives the main class. The item sits at
     that level as one of its own items.
   - Source `crosswalk`, confidence = the entry's share. Expect about 87%
     accuracy in-sample and 72% out of sample (§5.4).
   - Add the item to the review queue when the source is `crosswalk` and
     its depth is below the cut.
6. **Papers with an arXiv category.**
   - Map the primary category with `ARXIV_FDC`, where a category entry
     overrides its archive.
   - File the paper under that FDC class's directory, in a subdirectory
     named by the arXiv category.
   - Source `arxiv`. Papers with only a DOI go to step 7.
7. **Classifier hint, then review.**
   - Run shelfmark `profile:lexical@2` into `fdc`.
   - Never file by the classifier alone. Put the item in
     `_review/<main class>/` when the classifier names a main class and
     did not flag it, else in `_review/unsorted/`.
   - Source `classifier`. Its main-class agreement is 54% (§5.3).
8. **Review queue.** The owner, or an agent with the item in hand,
   assigns a Dewey or FDC number (for papers, an FDC class and arXiv
   category). The item then moves by step 4, with source `manual`.
   Measure the queue as the share of items with source `classifier` or
   `crosswalk` below the cut.

### 6.4 Inserting new items without renumbering

- Directories are named by numbers that are fixed by the notation, so
  adding an item never changes another item's path.
- A new section directory is created the first time an item needs it.
  Its number already exists in the notation.
- Nothing is renumbered. Hospitality in array is 0.701 for Dewey
  sections and 0.857 for FDC (`catalogue-books`); a decimal notation
  always admits a further digit.
- Items within a directory are ordered by the full Dewey number, then by
  author surname, then by title. That order is the file-name prefix.
- If the library crosses about 2,000 items under the two-level rule,
  split each division into sections. Every item's section is already in
  its metadata, so the split is mechanical.

### 6.5 Mapping to directories and notes

```
library/
  8xx Literature/                         # main class: FDC caption of 800
    82x English & Old English literatures/ # division: FDC caption of 820
      823 English fiction/                 # section: FDC caption of 823
        823.912 Woolf - To the Lighthouse.md
  5xx Science/
    53x Physics/
      530 Physics/
        quant-ph/                          # papers: arXiv category under the section
          2101.00001 Author - Title.md
  _review/
    3xx/ ... unsorted/ ...
```

Each item note has YAML front matter with these keys:

- identifiers: `id`, `isbn`, `doi`, `arxiv`, `ol_edition`, `ol_work`;
- numbers: `ddc` (full number as catalogued), `lcc`, `fdc` (FDC class
  reached);
- placement: `shelf` (the directory number at the cut),
  `placement_source` (`edition|work|crosswalk|arxiv|classifier|manual`),
  `confidence`, `placed_on`.

Two rules for every name and caption:

- Directory captions come from FDC only, never from Dewey schedules
  (A1; report 7).
- Do not call the structure "Dewey Decimal Classification". It uses
  Dewey numbers with FDC captions (report 7, §7). The directory captions
  above are illustrative; the agent must read the real ones from
  `schemes/fdc/fdc.json`.

## 7. Failure modes and risks

- **Coverage-driven rankings.** Any comparison of placed schemes mostly
  measures coverage (§5.3). The recommendation rests on gold runs at
  coverage 1.000 and on both-place subsets. Classifier-placed rankings
  were not used to choose a scheme.
- **The learned scheme's home advantage.** Its lead is measured with the
  same kind of text embedding that placed the books. The margin shrinks
  off the MiniLM graph, and refits win on their own graphs (§4.4). The
  graphs reward *embedding* similarity, which is not the same as the
  owner's sense of relatedness.
- **Dewey licence limits.** Numbers from catalogue records and their
  digit hierarchy are used, and captions are not (report 7). That
  position has residual risk: *ADA*, and no ruling on a dataset of
  numbers. Mitigation: captions come from FDC; nothing is fetched from
  OCLC services (`oclcclassify`, `deweyrights`).
- **FDC maintenance.** FDC has been frozen since 2010 and its author
  promises no maintenance (`schemes/fdc/README.md`). Its captions follow
  the 1922 Dewey edition, so some modern sections are captioned by a
  broader class (`captions`). The Dewey numbers themselves are current;
  only labels can lag.
- **Classifier error rates.** Lexical placement into FDC is right on the
  main class only 54% of the time, and on the section 21% (§5.3).
  Allowing it to file items would raise cost toward 0.6537 from 0.4958
  (`pipelines`). It is confined to the review queue.
- **Crosswalk errors.** The crosswalk is 87% accurate in-sample and 72%
  out of sample (`crosswalk-rule`). Its placements are marked
  `crosswalk` and reviewed when deeper than the main class.
- **Items with no identifiers.** Manuscripts, offprints, grey literature
  and objects get no number from any catalogue, so they always reach
  review. For the 50 well-known books every work carries both numbers
  (`availability`), but an older or rarer collection will have fewer: the
  holdout's work-level Dewey rate is 91%.
- **Sampling.** The 3,000 books are stratified equally over LCC main
  classes, not like any real collection. Their graph text is mostly
  subject headings, which favours LCC if anything. The papers are arXiv
  only. Effects between Dewey cuts are small (≤ 0.005 of cost) even where
  they are significant.
- **Weights.** κ, and so λ, is not calibrated (§1). The cut rule's 2,000
  threshold scales with κ: n ≈ κ / 0.0032.

## 8. What would change the recommendation

- **A semantic classifier.** Suppose a semantic classifier (the pending
  `2026-10-10-semantic` rerun) reached ≥ 0.85 main-class agreement on the
  3,000 books with coverage ≥ 0.95. It could then replace review for
  items without numbers. The pipelines show that would move the 0% row
  of `pipelines` toward the gold row.
- **A non-embedding similarity graph.** Suppose LCC beat Dewey sections
  on a graph built from co-use, citations or the owner's own links, at
  k ≥ 20. The scheme choice would then flip to LCC with the same
  placement steps: LCC call numbers are available for 100% of the
  holdout's works.
- **A learned scheme without home advantage.** Suppose the learned
  scheme kept a J lead of ≥ 0.05 on such a graph, and its notation were
  made hospitable and short. It would become a candidate primary
  structure despite A3.
- **A physical, single-shelf library.** If the owner browses mostly by
  walking one shelf, so that locality weighs more than about 0.45 units
  of cost (`locality-weight`), LCC order is preferred.
- **A small library.** Under about 2,000 items, use two digits (§6.2).
  Measuring κ on the owner's own use would replace that threshold.

## Reproduction

```sh
cargo build --release -p shelfmark-bench
python3 -I research/reports/scripts/ol_work_numbers.py              # cached Open Library answers
python3 -I research/reports/scripts/kb_proposal_runs.py all --harness target/release/shelfmark-bench
python3 -I research/reports/scripts/kb_proposal_tables.py           # exit 1 on any drift
```

`kb_proposal_runs.py graphs` needs the bge-small and nomic books graphs
built by `crates/bench/scripts/build_graph_fastembed.py` in
`bench/corpus/cache/proposal/graphs/`. The committed `.json.gz` graphs
and manifests replace that step for rescoring.

## References

Bibliography keys refer to `docs/research/references.bib`: `dasgupta2016`,
`cohenaddad2017`, `moseley2017`, `moseley2023`, `hick1952`, `hyman1953`,
`miller1981`, `snowberry1983`, `landauer1985`, `zaphiris2000`,
`larson1998`, `fdc`, `deweyrights`, `oclcclassify`, `usc105`,
`openlibrarylicensing`.

Others:

- Efron 1979, https://doi.org/10.1214/aos/1176344552
- Wilson 1927, https://doi.org/10.1080/01621459.1927.10502953
- Charpentier and Bonald 2019, https://doi.org/10.24963/ijcai.2019/286 (as in report 6)
- Holm 1979, TODO(cite)

Reports 6 (scoring catalogue) and 7 (Dewey licensing position) of this
directory.
