# Benchmarking classification schemes with Dasgupta cost: survey and design

Research and design only. Date 2026-10-09.
The shelfmark repository was read to inspect the schema, the gold set and the CLI.

Verification legend: "fetched" = HTTP 200 and content read this session. "200 only" = HTTP 200, content not read. "403" = publisher bot wall, unverified. Statements marked (inference) or (estimate) are mine, not measured.

## 0. Findings that shape the design

1. The shelfmark gold set (`research/eval/gold/starter-v1.ndjson`, 104 lines, fetched) is LCC-only: each item has `acceptable` LCC codes plus `identifiers` (isbn, lccn, openlibrary, openlibrary_work). It carries no gold for any other scheme. So a gold-only cross-scheme comparison is impossible on it; 104 items is also too few for a cost estimate (about 5k pairs, kNN graph of ~1k edges). Use it as a smoke test and to measure classifier confusion, not to rank. Its identifiers let us join each item to Open Library and LoC records and pull DDC for the same book.
2. Real librarian-assigned labels in two general schemes exist at scale: Open Library's search index (queried 2026-10-09) reports 14.7M docs with `lcc`, 5.86M with `ddc`, 4.99M with both, 3.91M with both and `language:eng`. Of the English sample I drew (200 random docs): 64% had at least 3 subject strings, 35.5% had more than one LCC value, 7.5% had more than one DDC value. Works with a prose description of at least 80 characters: 18 of 100 sampled (small sample, wide error bar). Consequence: gold-label comparison is effectively LCC vs DDC (plus FAST/LCSH where present); every other scheme needs shelfmark-placed labels. This is the central confound, handled in section 4.
3. Description text is scarce (about 18%), so the primary corpus is the "rich" subset (title plus description), roughly 0.7M works, far more than needed. A title-plus-subjects variant on the full set is the sensitivity run.
4. `schemas/scheme.schema.json` (fetched): classes have one optional `parent`, so every scheme is a tree or forest by construction. EuroVoc and the UNESCO Thesaurus are polyhierarchies; the port has already chosen one parent per class. That choice is hidden bias; the port should record dropped parents (inference: open question for the port beads).
5. No Rust crate computes Dasgupta cost or Moseley-Wang revenue (section 1). The needed code is about 150 lines; we write it.
6. Shelfmark's classifier shortlist tier embeds with BGE-small-en-v1.5 (README, fetched). If the benchmark also uses BGE-small for similarity, schemes whose labels embed well get a circular advantage. Use a different embedding model for similarity than for placement (section 2.3).
7. Dewey is not a shelfmark scheme (OCLC-owned labels). For the gold track we need only its tree shape, which the decimal digits define (510 -> 510.1 -> 510.15), so cost needs no labels (inference from the notation; verify on a sample before relying on it).

## 1. Survey of open implementations

| Item | What it gives | Language / license | Status | Complexity | Link status |
|---|---|---|---|---|---|
| Dasgupta 2016 | Cost: sum over pairs of w_ij times the number of leaves under LCA(i,j). Original definition | paper | n/a | n/a | arXiv 1510.05043 fetched (200); ACM DOI 403 |
| Moseley-Wang (JMLR 24, 2023) | Revenue (reward): sum w_ij (n - leaves under LCA). Paper states cost + reward = n * sum w_ij; average linkage is a 1/3-approximation to max reward; also gives divisive 1/3-approx algorithms; bisecting k-means has a poor bound | paper | n/a | n/a | JMLR PDF fetched and read |
| Cohen-Addad, Kanade, Mallmann-Trenn, Mathieu (JACM 2019) | Axiomatic family of admissible objectives; Dasgupta cost is one; average linkage and variants are optimal on planted hierarchies | paper (no code found) | n/a | n/a | arXiv 1704.02147 fetched and read |
| Charikar-Chatziafratis | O(sqrt(log n))-approx via sparsest cut | paper | n/a | n/a | arXiv 1609.09548 fetched |
| HazyResearch/HypHC | Official PyTorch HypHC. `utils/metrics.py` has `dasgupta_cost` (fast, Cython-reordered, binary) and `dasgupta_cost_iterative` (also works on non-binary trees, recursion-free). Cython `mst` and `unionfind` packages | Python 3.7 tested, PyTorch. GitHub API reports no license (cannot vendor) | 209 stars, last push 2023-10-05; bundled datasets are small UCI sets (zoo, iris, glass) | Cost is O(n^2) on a dense similarity matrix (sums submatrices per node) | repo, README, source fetched |
| scikit-network `sknetwork.hierarchy` | `dasgupta_cost`, `dasgupta_score` (1 - normalized cost), `tree_sampling_divergence`, `Paris` (graph average-linkage-like), `cut_balanced` | Python, BSD, v0.33.5 | maintained | Sparse-graph input; cost in one pass over merges with an aggregate-graph (dendrogram must be binary scipy-style) | docs and source fetched |
| scipy `cluster.hierarchy.linkage` | Dense average linkage, `to_tree` | Python | stable | O(n^2) memory | docs 200 only |
| fastcluster | Fast dense linkage | Python/R/C++ | stable | O(n^2) | PyPI page 200 only |
| Rust `kodama` 0.3.0 | Agglomerative clustering incl. average linkage (generic nn-chain) | Rust | 608k downloads (crates.io API) | O(n^2) memory (condensed matrix) | docs.rs 200 only |
| Rust `linfa-hierarchical` 0.8.1 | Agglomerative | Rust | 17.7k downloads | O(n^2) | docs.rs 200 only |
| Rust crate for Dasgupta cost / revenue / HypHC | None found. crates.io searches for dasgupta, hierarchical-clustering, moseley returned nothing relevant (2026-10-09) | n/a | n/a | n/a | crates.io API queried |
| google/graph-mining | Parallel HAC on sparse graphs (poly-log-depth HAC, TeraHAC), C++ | C++ | active | near-linear in edges | README fetched; TeraHAC arXiv 2308.03578 fetched (title only) |
| Rust kNN: `hnsw_rs`, `instant-distance` | Approximate kNN graph | Rust | n/a | ~O(n log n) | docs.rs 200 only |

### Computing the cost of a fixed tree

Definition for any rooted tree T (n-ary allowed, internal nodes may also hold leaves): cost(T) = sum over unordered pairs {i,j} of w_ij * |L(lca(i,j))|, where |L(v)| is the number of items under v. Moseley-Wang's pairing identity gives revenue = n * W - cost with W = sum w_ij (stated in the JMLR text for the general cost; checked).

Algorithm, given a sparse weight graph with m edges (kNN, k about 20 so m is about 10 n after symmetrizing):
1. Build T over the used classes; attach each item to its class node; items placed at an internal class become direct leaf children of it.
2. For each edge (i,j,w), find v = lca(i,j) and add w to accumulator `pair_w[v]`. Class trees are shallow (depth at most about 10 for the schemes here), so walking up the parent pointers is O(depth) per edge; for the HAC dendrogram (depth up to n) use Euler tour plus sparse table, O(n log n) build and O(1) per query, or offline Tarjan LCA in O(n + m alpha).
3. cost = sum over internal v of |L(v)| * pair_w[v]. Total O(m * depth) or O(n log n + m). For n = 20k, m about 400k: milliseconds.
4. Equivalent bottom-up form that avoids LCA: A(v) = total weight of edges with both endpoints under v; cost = sum_v |L(v)| (A(v) - sum_children A(c)). A(v) is accumulated by DSU/small-to-large in O(m log n).

The O(n^2) in the literature is the dense similarity matrix. With kNN sparsification the sum is truncated to the retained pairs; every scheme sees the same sparsified w, so comparisons stay paired. Validate on n <= 5k by comparing the sparse cost to the dense cost (rank agreement across schemes, not equality).

Average linkage on a sparse w (missing pairs = 0, cluster similarity = sum of edge weights / (|A||B|)) is exactly Moseley-Wang's average linkage, so a sparse implementation is the reference tree with its 1/3 reward guarantee, in O(m log n) with a lazy heap (inference: standard implementation).

## 2. Harness design

### 2.1 Corpora (public, reproducible)

| Corpus | Source | Gold labels | Size | Purpose |
|---|---|---|---|---|
| G (gold books) | Open Library editions/works dump (dumps page 200; licensing page 200, CC0 claim not re-read; shelfmark's own gold notes say "unverified for dumps"), filter: English, has LCC and DDC, has description of at least 80 chars, dedupe by work, seeded uniform sample | LCC (call number, parse class part), DDC | 20k main, 100k scale run | Gold-only track. Pin the dump date and SHA-256 of the file |
| G2 (cross-check) | LoC MARC "Books All" free distribution (marcDist page 200): LC catalogers assign both 050 and 082; 650 gives LCSH | LCC, DDC | 20k | Check that OL cataloger-quality does not drive results |
| S (shelfmark smoke) | The 104-item gold set, joined to OL for DDC | LCC (plus DDC via join) | 104 | Smoke test; classifier confusion matrix |
| P (placed) | Corpus G text run through `shelfmark classify` for each scheme | none; classifier output | same items as G | All-scheme track |
| D (domain, later) | arXiv metadata (carries arXiv category, MSC, ACM class where the author gave them; inference, not re-verified) plus OpenAlex works | native domain labels | 20k | Domain schemes (MSC, ACM CCS, arXiv) on their home turf; not in the general ranking |

"All kinds of works" claim: corpus G is books only. A general-purpose ranking should add a second stratum (papers via D) and report each stratum, not average them; state this limit up front.

### 2.2 Text and fixed similarity

- Text: `title + ". " + description` (truncate 256 tokens). Primary run excludes subject strings, because LCSH headings are linked into the LCC scheme vocabulary by shelfmark and would leak label information into similarity. Sensitivity run S2 includes subjects.
- Embedding for similarity: one pinned model not used by the shelfmark placement tier. BGE-small is the placement model, so use a different open model (candidates: e5-base-v2, gte-base, nomic-embed-text; pick one before running and pin the revision hash; I did not benchmark them). Second similarity S3: IDF-weighted Jaccard over subject strings (no embedding at all). Third (domain corpus D only): bibliographic coupling from OpenAlex citations.
- Weights: cosine on L2-normalized vectors; w_ij = max(0, cos_ij - tau), tau = mean cosine of 100k random pairs (embedding cosines carry a large constant offset that would otherwise make every pair "related"). Pre-register tau rule and kNN k; sensitivity at k in {10, 20, 50}.
- kNN: exact brute-force blocked matmul with rayon for n <= 100k (n^2 d = 3.8e12 flops at 100k, d=384; about minutes on 12 cores, estimate); `hnsw_rs` above that. Symmetrize by taking max. Store the graph once (`graph.bin`) so every scheme and baseline reads the identical w.
- Robustness rule: report Kendall tau of the scheme ranking across similarity choices (S1, S2, S3, two k values). A ranking that flips between similarities is reported as unresolved.

### 2.3 Inducing a tree from a scheme

1. Load scheme JSON (`classes[].code`, `parent`). Forest roots get a virtual root.
2. Restrict to used classes: keep classes with at least one item and all their ancestors; delete unused subtrees. Unary chains do not change Dasgupta cost (|L| is equal along them) so cost uses the compressed tree; navigation metrics use the uncompressed tree because each level is a real menu step (compress only if the menu would show one option).
3. Item placement: item attaches as a leaf under its class node. Items in one class tie: LCA of two same-class items is the class node, so all pairs inside a class pay |class|. This is the star (worst) arrangement inside a class and it is the right semantics for "you scan the class". Report also `cost_refined`: same tree, but each class's interior replaced by the average-linkage subtree (the cost if the shelf order inside a class is optimized). Scheme quality = `cost`; shelf-order quality = `cost_refined`.
4. Unplaced items (no candidate above the confidence threshold; for gold, code not in the scheme) hang directly under the virtual root. Pairs involving them have LCA = root, |L| = n. Coverage is therefore penalized inside the cost; it is also reported separately.
5. Multiple placements (OL: 35.5% of sampled works have more than one LCC). Three modes, all reported:
   - `primary` (headline): the first-listed code for gold; top-1 for classifier output.
   - `fractional`: work i has masses m_ia = 1/p_i over its p_i placements; the copies are leaves of mass m; pair term is w_ij * sum_{a,b} m_ia m_jb * mass(lca(a,b)); mass(root) = n. This is the "buy one copy per class" expectation.
   - `best-pair`: for each related pair, use the closest pair of placements (min over a,b). Lower bound on what multi-shelving buys.
   Also report the multi-placement rate per scheme (a measure of how often the scheme cannot decide).

### 2.4 Metrics

Per (scheme, track, similarity, mode):

| Metric | Definition |
|---|---|
| `cost_norm` | cost / (n W). Flat tree (everything under root) = 1. Interpreted as the weighted-average fraction of the corpus you must scan to see a related pair together |
| `revenue_norm` | 1 - cost_norm (Moseley-Wang reward over n W) |
| Reference tree | Sparse average-linkage binary tree HAC on the same w; `cost_norm_HAC`. Terminology: HAC is an achievable cost (upper bound on the optimum), not a lower bound; no tractable lower bound exists (exact optimization is NP-hard per the survey in research/2). Where affordable, take the best of HAC, HAC plus local search, HypHC (n <= 5k only) as `cost_best_known` |
| `skill` | (1 - cost_norm_T) / (1 - cost_norm_ref), 0 for flat, about 1 for a learned tree. Can exceed 1 because the reference is approximate; report as is |
| Matched-K gap | See 2.5 |
| Permutation null | Shuffle items across the scheme's leaf slots (keeps tree shape and class-size histogram), 1000 permutations; report z = (mean_null - cost) / sd_null. Controls for scheme size and shape without any HAC |
| Expected tree distance | sum w_ij * dist_T(i,j) / W (hops up to LCA and down). Same LCA pass |
| Depth | Item depth: mean, p50, p95, max (uncompressed tree) |
| Branching | Children per used internal node: mean, p95, max, root fan-out, count of nodes above 50 children |
| Balance | Normalized entropy of class mass H(c)/log K_used; effective classes exp(H); Gini of class sizes; share of items in the largest class |
| Coverage | Fraction placed; fraction in residual classes (label matching general/miscellaneous/other, listed in a config file); fraction placed at a non-leaf class (hedge rate) |
| Expected navigation | See 2.6 |

### 2.5 Fairness across schemes of very different sizes

A scheme with more leaf classes has smaller ties, so its raw cost falls without being a better organization. Controls, in order of importance:
1. Cost-resolution curve. Truncate each scheme at depth d = 1..D; the truncation is a partition with K_d classes. Plot (K_d, cost_norm). The reference curve is the HAC dendrogram cut into K clusters (best flat K-partition by the cut that minimizes cost, through the cut with the tree above intact). Score = vertical gap between scheme and reference at the same K, summarized as area between curves over log K. This compares at matched leaf counts.
2. Permutation null (above): same K and same size histogram by construction.
3. Cap each scheme at a common maximum resolution (K <= K_min over compared schemes) via depth truncation and report that as the "equal-resolution" ranking next to the "native-resolution" ranking.
4. Bootstrap over items (200 resamples on the fixed graph; paired across schemes) for 95% intervals; do not call a ranking unless intervals separate.
5. Scheme-size sweep: also run the benchmark at n = 2k, 5k, 20k to show whether the ranking is stable as n grows past the point where small schemes saturate.

### 2.6 Navigation cost

Model: a seeker descends from the root; at node v with b(v) children (uncompressed, b = 1 costs nothing), the cost is c(b). At the leaf class holding n_c items the seeker scans the class. Report three c:
- `nav_scan`: c(b) = (b+1)/2 (linear scan of a menu or shelf) plus (n_c+1)/2 at the class. Finite optimum fan-out; this is the one that blocks the "flat is cheapest" degeneracy.
- `nav_log`: c(b) = a + log2 b with a in {0, 1, 2} bits-equivalents of per-level overhead (Hick-Hyman style); a = 0 collapses to log2 n for any perfectly balanced tree and measures only imbalance, so a > 0 is required for depth to matter. The model parameters are assumptions, not fitted; the benchmark reports the sweep and does not choose a.
- `nav_related`: expected cost to reach a related item after finding one: walk up to LCA (hops) then down with c(b), weighted by w_ij. Ties navigation to the Dasgupta pair weights.
The usage prior p_i is uniform by default; sensitivity with p_i proportional to log(1 + edition count) or OL reading-log counts (inference: available in the OL dump; not verified).

## 3. Controlling classifier bias

Two tracks are always reported side by side; a ranking claim needs agreement across them.

- Track A, gold-only: corpus G, labels straight from the catalog. Schemes available: LCC and DDC (and LCSH/FAST-based trees where present). No classifier in the loop. Answers "is LCC or DDC better organized on real librarian placements", nothing about the other 15 schemes.
- Track P, shelfmark-placed: the same G text placed by shelfmark into each scheme. Covers all schemes. Confounds: (i) classifier accuracy differs by scheme (LCC has gold-tuned vocabulary; the ports often have only label text; the README shows LCC subclass top-1 as low as 15.4% with the bundled outline and 34.6% with LCSH enrichment); (ii) classifiers retreat to coarse parents when unsure, which lowers resolution and raises cost; (iii) circularity with the embedding model (section 2.2).
- Calibration of the bias using LCC, the only scheme with both tracks: Delta_LCC = cost_P - cost_A on identical items. Then (a) noise-injection curve: degrade the gold LCC labels toward the classifier's measured confusion matrix at several accuracies, record cost vs accuracy; (b) read each other scheme's placed cost against that curve at its own measured accuracy (accuracy measured only where gold exists, so for the other schemes use the classifier's self-confidence calibration from `shelfmark eval`, which is in-sample per README, so treat as optimistic).
- Fixed-resolution placement: truncate all placed labels to a common depth before costing, removing the parent-retreat effect.
- Soft placement: use top-k candidates with calibrated confidences as masses (fractional mode), which degrades gracefully when the classifier is unsure.
- Label-intrinsic track L, no works and no classifier: items are the scheme's own classes (label plus aliases embedded), w from embedding similarity of labels, tree = the scheme itself, compared to HAC on the same class set. Measures whether siblings are closer than cousins. Weakness: terse or numeric labels ("General works"), and it ignores which classes actually get used.
- Report per scheme: A (if gold exists), P, L; the shift cost_P - cost_A for LCC; plus the confusion-corrected P estimate. A scheme that wins only on P and has no A or L support is flagged "classifier-dependent".

## 4. Implementation recommendation

Crate `crates/bench` in the shelfmark workspace (`shelfmark-bench`, binary `shelfmark-bench`, optionally exposed later as `shelfmark bench`). The workspace has no dependency outside its own directory (README, tested), so keep it self-contained.

Dependencies: `shelfmark-core` (SchemeDoc), `serde`, `serde_json`, `rayon`, `ndarray` or `matrixmultiply` for blocked dot products, `rand` plus `rand_chacha` (seeded), `clap`, `flate2`. Optional feature `embed` using `fastembed` (already in `tier-shortlist`, needs rustc 1.88+, above the workspace MSRV 1.82, so keep it behind a feature) and `hnsw_rs` for large n. No Python required except optional HypHC at n <= 5k through a subprocess.

Pipeline as subcommands; every stage reads and writes JSON/NDJSON and a content hash, so stages are cacheable and reproducible:

| Subcommand | Input | Output |
|---|---|---|
| `corpus build` | OL dump path, seed, n, filters | `corpus.ndjson`: `{id, title, text, gold: {lcc:[...], ddc:[...]}}` + manifest with dump SHA-256 |
| `embed` | corpus, model id | `vectors.f32`, manifest (model revision) |
| `graph` | vectors, k, tau rule | `graph.bin` (CSR: n, edges, weights), manifest |
| `place` | corpus, scheme files, mode | `placements/<scheme>.ndjson`: `{id, classes:[{code, mass}]}`; for gold track derived from corpus; for track P produced by running `shelfmark classify` in NDJSON batch mode |
| `tree` | scheme JSON, placements | `tree/<scheme>.json`: used-class tree with counts (also dumpable for inspection) |
| `reference` | graph | `ref/hac.json` (binary dendrogram), `ref/cut_curve.json` (cost vs K) |
| `score` | graph, tree(s), reference | `scores.json` |
| `report` | scores | `report.md` and `report.json` |
| `run` | config.toml | all stages, resumable |

`scores.json` shape (one object per scheme x track x mode x similarity):
`{scheme, version, track: "A"|"P"|"L", mode: "primary"|"fractional"|"best-pair", sim, n, k, cost_norm, revenue_norm, cost_refined_norm, ref_cost_norm, skill, null: {mean, sd, z}, curve: [{depth, K, cost_norm, ref_cost_norm}], depth: {...}, branching: {...}, balance: {...}, coverage: {...}, nav: {scan, log: {a0,a1,a2}, related}, ci95: {...}, hashes: {graph, tree, scheme}}`.

Edge cases to test: scheme where all items land at the root (cost_norm must equal 1); star-vs-binary identity on a toy graph (cost + revenue = n W); a planted two-level hierarchy where the true tree must score best; unary chains; items at internal nodes; duplicate codes; forest schemes; sparse vs dense agreement at n = 2k; sknetwork `dasgupta_cost` cross-check on binary dendrograms (golden fixtures generated once in Python).

Runtime (estimates, not measured):
- Costing: O(m depth); n = 20k, about 400k edges: under 10 ms per scheme-mode; with 1000 permutations and 200 bootstraps still seconds. Sparse HAC: seconds at 20k, about a minute at 100k.
- kNN: exact at 20k about 10-30 s on 12 cores; 100k about several minutes.
- Embedding: base-size model on CPU, estimate 50-200 texts/s, so 20k texts is 2-7 minutes; dominates the cheap stages.
- Placement for track P: README quotes under 50 ms per item for the lexical profile and 50-100 ms more for shortlist, plus 0.2 s process start. With batch NDJSON that is about 17 min per scheme for 20k items single-threaded, so 17 schemes is about 5 hours single-core or about 25-30 minutes on 12 cores. Launching one process per item would add 20k * 0.2 s = 67 min per scheme and must be avoided.
- Whole benchmark at n = 20k: under an hour wall clock on a 12-core machine after the dump is downloaded (dump download and filtering are the long pole and are one-time; size not checked).

Build order: (1) cost/revenue/LCA core with toy tests and the sknetwork cross-check; (2) corpus + graph + gold tracks for LCC and DDC (a first real answer, no classifier involved); (3) HAC reference and the cost-resolution curve; (4) placement integration and calibration; (5) label-intrinsic track; (6) domain corpus.

## 5. Open questions

- Which similarity embedding model, and a measured check that its offset subtraction (tau) does not change rankings.
- DDC digit-prefix tree: confirm against a labeled DDC outline that depth and fan-out are faithful.
- HathiTrust hathifiles as a third gold source: documentation page returned 403 from this host, not evaluated.
- Whether non-book strata can be built with gold labels in more than one general scheme; the sources found carry gold for only one domain scheme each.
- Which parent the polyhierarchy ports dropped.

## Sources

Fetched and read this session (2026-10-09):
- Dasgupta, "A cost function for similarity-based hierarchical clustering", arXiv: https://arxiv.org/abs/1510.05043 (200). ACM DOI https://dl.acm.org/doi/10.1145/2897518.2897527 returned 403.
- Moseley and Wang, JMLR 24 (2023): https://jmlr.org/papers/v24/18-080.html (200); PDF https://jmlr.org/papers/volume24/18-080/18-080.pdf (read: cost + reward = n sum w; reward upper bound (n-2) sum w; average linkage 1/3-approx).
- Cohen-Addad et al.: https://arxiv.org/abs/1704.02147 (read; this arXiv version does not use the revenue term)
- Charikar and Chatziafratis: https://arxiv.org/abs/1609.09548
- Chami et al., HypHC paper: https://arxiv.org/abs/2010.00402
- HypHC code: https://github.com/HazyResearch/HypHC (README, file tree, `utils/metrics.py` read; API: no license, 209 stars, last push 2023-10-05)
- scikit-network hierarchy docs: https://scikit-network.readthedocs.io/en/latest/reference/hierarchy.html ; repo https://github.com/sknetwork-team/scikit-network ; source of `hierarchy/metrics.py` read; PyPI JSON: v0.33.5, BSD
- google/graph-mining: https://github.com/google/graph-mining (README read); TeraHAC: https://arxiv.org/abs/2308.03578 (title read)
- Shelfmark repository files read: README.md, schemas/scheme.schema.json, schemes/registry.json, crates/core/src/scheme.rs, research/eval/README.md, research/eval/gold/starter-v1.ndjson, research/vocab/README.md
- Open Library search API (queried 2026-10-09 for the counts and samples in section 0): https://openlibrary.org/search.json ; dumps page https://openlibrary.org/developers/dumps (200 only); licensing https://openlibrary.org/developers/licensing (200, CC0 statement not located in the extracted text)
- crates.io API queries (kodama 0.3.0, 607,808 downloads; linfa-hierarchical 0.8.1, 17,690)

HTTP 200 only (content not read): https://docs.rs/kodama/0.3.0/kodama/ , https://docs.rs/linfa-hierarchical/0.8.1/linfa_hierarchical/ , https://docs.rs/hnsw_rs/latest/hnsw_rs/ , https://docs.rs/instant-distance/latest/instant_distance/ , https://docs.rs/fastembed/latest/fastembed/ , https://docs.rs/rayon/latest/rayon/ , https://pypi.org/project/fastcluster/ , https://docs.scipy.org/doc/scipy/reference/generated/scipy.cluster.hierarchy.linkage.html , https://www.loc.gov/cds/products/marcDist.php , https://huggingface.co/BAAI/bge-small-en-v1.5 , https://www.loc.gov/catdir/cpso/lcco/ , https://docs.openalex.org/api-entities/topics

Unverified: HathiTrust hathifiles page (403); ACM DOI (403); arXiv metadata carrying msc/acm classes (recalled, not re-fetched); OL reading-log counts in dumps.
