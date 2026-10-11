# learned: a fitted, order-preserving general classification

`learned.json` is a shelfmark scheme that was not designed but optimized.
It is fitted to the Paper 1 benchmark corpus (10,000 arXiv papers,
`bench/corpus/corpus.jsonl`) by minimizing the same quantities the benchmark
harness (`crates/bench`) reports: Dasgupta similarity cost on the committed
kNN cosine graph, plus an expected navigation cost with a per-branch charge.
It is the prototype for the second paper's question: how far a learned
hierarchy gets on the objective that ranks the published schemes. The results
are in `bench/results/2026-10-10-learned/`.

| | |
| --- | --- |
| Scheme ID | `scheme:learned@2026-10-10` |
| Classes | 110: 2 top-level, 5, 16, then 87 at level 4; 88 leaf classes of 38-345 works (median 98) |
| Codes | `01`, `01.02`, `01.02.03`, ...: parent code plus two-digit position, so codes sort in shelf order (`[0-9]{2}(\.[0-9]{2})*`) |
| Vocabulary | each class's top 10 c-TF-IDF terms (source `ctfidf`, weight relative to the top term) |
| Licence | the generated scheme, its labels and the files in `data/`: CC0 1.0. Builder code: MIT OR Apache-2.0, like the rest of the repository |

Validate and load it with the shelfmark CLI:

```sh
shelfmark scheme validate --path schemes/learned/learned.json
shelfmark scheme compile schemes/learned/learned.json
```

## Objective

For a class tree T over the n corpus works, each work in one leaf class:

```
J(T) = D(T) + lambda * N(T)

D(T) = (1 / (n W)) * sum over graph edges (i, j) of w_ij * |items(lca_T(i, j))|
N(T) = (1 / n) * sum over nodes v with fanout(v) > 1 of |items(v)| * (a + log2 fanout(v))
```

- `w_ij` are the edge weights of `bench/results/2026-10-10/graphs/corpus-k20.json.gz`
  (all-MiniLM-L6-v2 cosine minus the mean pair cosine, k = 20, max-symmetrized),
  `W` their sum. Works hang under their leaf class, so two works in one class
  meet at that class and pay its size; a virtual root holds the top-level
  classes. `D` is the harness's `cost_norm` exactly.
- `fanout(v)` is the number of child classes of an internal class (or of the
  root) and the number of works in a leaf class. `N` is the mean over works
  of `a + log2 fanout` summed along the path from the root to the work, the
  harness's `nav_log_a`. With the per-branch cost `a = 1` it is
  `nav_log_a1`. A tree whose splits are balanced pays about `log2 n` bits
  plus `a` per level, so `a` is the price of a menu level and the `log2`
  part penalizes splits whose fan-out does not match the sizes below.
- Weights: `lambda = 0.005`, `a = 1`. One extra level for every work is worth
  a 0.005 drop in `cost_norm` (half a percent of the corpus scanned per
  related pair). Shape bounds: 2-12 child classes, at most 4 class levels
  (the target depth), leaf classes of at least 5 works.
- Order preservation: the shelf order is fixed before the classes are chosen
  (step 3 below), every class is a contiguous run of it, and siblings are
  ordered along it. This is the Hu-Tucker/optimal-alphabetic-tree constraint
  from `research/reports/2-math-formulation.md`: the leaf order of the tree
  is the physical shelf order.

The two terms pull against each other. Dasgupta cost falls with every extra
split and is lowest for a binary tree with one work per leaf; the navigation
term charges `a` bits per level for every work under it. The test
`rebuild_reproduces_the_committed_scheme_and_matches_the_harness` checks
that `D` and `N` computed by the builder equal the harness's `cost_norm` and
`nav_log_a1` for the committed scheme and its placements.

## Algorithm

`learned-scheme build` (`src/fit.rs`), deterministic for a fixed seed
(`20261010`, used by the local-search visiting orders):

1. **Average linkage** on the sparse graph (`src/binary.rs`): cluster
   similarity is the summed edge weight over the product of the sizes, with
   missing pairs at 0 (Moseley and Wang's average linkage, a 1/3
   approximation of the Dasgupta revenue). Ties go to the lowest cluster
   ids; disconnected components are joined smallest first. Its normalized
   Dasgupta cost, 0.15183392202502477, equals the harness's reference value.
2. **Rotation local search** on Dasgupta cost: at node `x = (A, C)` with
   `A = (A1, A2)`, try `((A1, C), A2)` and `((A2, C), A1)`, with exact local
   deltas. On this graph it makes no move. The reason is general:
   a rotation helps exactly when `w(A1, C) / (|A1||C|) > w(A1, A2) / (|A1||A2|)`,
   and average linkage is reducible, so when `A1` and `A2` merged no part of
   a later `C` was closer. Average-linkage trees are rotation-optimal; the
   pass certifies that and matters for other starting trees.
3. **Shelf order**: each merge's children are oriented so the child more
   tied to the works just shelved to its left comes first. The binary tree's
   leaf order becomes the fixed shelf order.
4. **Exact collapse** (`src/collapse.rs`): a dynamic program over the binary
   tree picks which binary nodes become classes, contracting the rest, so as
   to minimize `J` over all such trees with 2-12 children and at most 4
   levels. For a class `u` it chooses the frontier of descendant classes with
   a knapsack over the binary nodes in between (at most 11 contractions
   deep). Binary subtrees with fewer than 5 works are glued to their
   sibling and join the adjacent leaf class on the shelf. The DP is exact
   when nothing is glued above leaf level, and close to exact otherwise
   (0.264697 by the DP, 0.264730 exact here).
5. **Order-preserving local search** (`src/search.rs`) on exact `J`: shift a
   boundary work into the neighbouring leaf class on the shelf (incremental
   delta), hand the last child class to the next sibling (or the reverse),
   merge adjacent sibling leaves, flatten a class into its parent, group two
   or three adjacent siblings under a new class. Every move keeps the bounds
   and the shelf order. Rounds repeat until no move improves `J`.

On the corpus (about 2.5 s on one core):

| stage | D (cost_norm) | N (nav, a = 1) | J | classes |
| --- | ---: | ---: | ---: | ---: |
| average linkage (binary, n leaves) | 0.151834 | - | - | - |
| rotation local search (0 moves) | 0.151834 | - | - | - |
| exact collapse | 0.172309 | 18.4841 | 0.264730 | 129 |
| order-preserving local search (88 shifts, 20 structure moves) | 0.172156 | 18.4487 | 0.264399 | 110 |

The root has two children under every setting tried: Dasgupta cost always
prefers a binary split near the top, where the most weight crosses. A
sensitivity sweep over `lambda`, `a` and the depth bound is in the results
directory (`results.json`, `sweep`).

## Stable codes under insertion

`learned-scheme insert` (`src/insert.rs`) files new works without renaming
anything:

1. A new work goes to the leaf class whose centroid is nearest (cosine on
   unit all-MiniLM-L6-v2 vectors; the centroids are `data/centroids.json.gz`,
   one i8-quantized vector and work count per leaf class).
2. A leaf class tries a two-means split, anchored on its fitted centroid,
   when either (a) it has taken at least 12 new works and they form a
   distinct group (the two centroids' cosine is at most 0.5), or (b) its new
   works reach its capacity: its fitted size, or the median fitted leaf size
   (98) for a class created by insertion. Capacity splits start from the
   first principal direction of the new works (Boley's principal direction
   divisive partitioning). Both sides need at least 5 works.
3. A split turns leaf `C` into the parent of `C.01`, which keeps the core
   (C's earlier works and the new works near them), and `C.02`, which takes
   the other group. `C.01` is labelled "<C's label> (general)"; `C.02` is
   labelled from its works' most frequent title words.

The guarantee, checked by `insert::check_stable` on every run and proved
on the real data by `tests/insertion.rs` (the 500-record Open Library holdout,
`data/holdout-vectors.json.gz`):

- every existing class keeps its code, label, vocabulary, note and parent,
  and the existing classes keep their relative order;
- only leaf classes are split: a new class's parent is a class that was a
  leaf before insertion or was itself created by insertion, and a new code
  extends its parent's code;
- an existing work's class can only become a descendant of its old class
  (`C` to `C.01`), and its shelf position does not change, because `C.01`
  opens `C`'s run.

The inserted scheme takes the version `<version>.ins<count>`. Its codes and
parent links are a superset of the original's; shelfmark's stale check sees
only the split leaves as changed.

## Labels

No model and no API was used. The labels in `labels.tsv` were written by
hand, by the AI coding assistant that built this prototype, from the label
sheet (`learned-scheme build --sheet sheet.md`). The sheet shows, for every
class, its size, its top 10 c-TF-IDF terms (class-based TF-IDF as in
BERTopic: `tf(t, c) / tokens(c) * ln(1 + A / f(t))` over titles counted twice
and abstracts, unigrams and adjacent bigrams, terms in at least 5% of the
class's works) and its five most central titles (largest graph weight to
the class's other works). Each label line stores the class's top three terms
as a fingerprint. The builder refuses a label whose fingerprint no longer
matches, so a refit that moves classes cannot silently keep stale labels.
`--allow-auto-labels` falls back to the top terms ("Graph / retrieval /
query") for unlabelled fits such as the 80% split.

## Files

| file | what |
| --- | --- |
| `learned.json` | the scheme (CC0 1.0) |
| `labels.tsv` | hand-written labels with term fingerprints |
| `data/centroids.json.gz` | leaf centroids for insertion (derived from the corpus vectors) |
| `data/holdout-vectors.json.gz` | the holdout's vectors, i8-quantized, for the insertion test and run |
| `sources.lock.json` | sha256 of every input |
| `src/`, `tests/` | the builder and its tests |

## Rebuild

From the repository root:

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo run --release --manifest-path schemes/learned/Cargo.toml -- build \
  --corpus bench/corpus/corpus.jsonl \
  --graph bench/results/2026-10-10/graphs/corpus-k20.json.gz \
  --labels schemes/learned/labels.tsv --out schemes/learned/learned.json
cargo test --release --manifest-path schemes/learned/Cargo.toml
```

The scheme needs only committed files and rebuilds byte for byte (tested).
The centroid and vector files also need the embedding vectors, which
`crates/bench/scripts/build_graph_fastembed.py --vectors` writes (see
`bench/results/2026-10-10-learned/REPRODUCE.md`); add
`--vectors <corpus.f32> --centroids schemes/learned/data/centroids.json.gz`
to the build, and run `learned-scheme quantize` for the holdout.

## Limits

- Fitted on arXiv papers only: the top level splits computing from
  mathematics and physical sciences, and there is no place for the
  humanities until insertion creates one. It is a general method, not yet a
  general classification.
- Its own placements of the corpus are in-sample. The held-out 80/20 split in
  the results is the fair test.
- Insertion uses the same embedding model as the similarity graph, which
  favours it on that graph. The shelfmark-placed runs, which use only the
  labels and c-TF-IDF vocabulary through the lexical classifier, are the
  comparison that avoids this.
