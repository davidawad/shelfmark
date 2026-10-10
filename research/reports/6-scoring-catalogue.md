# Scoring catalogue for classification schemes

Written 2026-10-10. This report defines every score that `shelfmark-bench
scores` computes, and catalogues the requested scores that the current data
cannot support, naming the missing data. The results are in
`bench/results/2026-10-10-scores/`. Score names below are the keys in that
directory's `results.json` (`runs.<run>.catalogue.scores`).

Citations: each source has a DOI or stable URL that was resolved on
2026-10-10, through Crossref plus doi.org, or by fetching the venue page.
`TODO(cite)` marks a primary source with no verified stable copy.

## Setting and conventions

- **Items and trees.** A run is one scheme plus one placement of the
  benchmark items: the 10,000 arXiv papers (`corpus`) or the 500 Open
  Library books (`holdout`). Each item sits at its *primary* class (the first
  source label, or the highest-mass placement). Unplaced items sit at a
  virtual root. The tree is the scheme restricted to the used classes and
  their ancestors (`crates/bench/src/hier.rs`). Class nodes are numbered in
  shelf order: pre-order, with siblings in scheme-file order. Every
  benchmark placement holds one class per item, so the primary-class tree is
  the tree the Paper 1 harness scores. The catalogue recomputes Dasgupta
  cost and checks that it equals the harness value for every run.
- **Similarity graph.** These are the committed kNN graphs
  (`bench/results/2026-10-10/graphs/`): all-MiniLM-L6-v2 cosine, k = 20,
  mean-pair offset, max-symmetrised.
- **Embeddings.** These are the same model's unit vectors (n x 384),
  regenerated with `build_graph_fastembed.py --vectors`. The vector files are
  pinned by SHA-256 in `results.json`. They are not committed, because each
  is over 14 MB and rebuildable.
- **External gold.** For papers this is the arXiv primary category (fine)
  and its top-level group (coarse). For books it is the LCC class letters of
  the first LC call number (fine) and its main class letter (coarse). This
  gold is independent of the scheme being scored, except for arXiv and LCC
  themselves, where it measures self-consistency.
- **Tracks.** Gold runs use the source labels (arXiv, MSC and JEL on the
  papers; LCC and LCC-full on the books). For the learned scheme, its own
  placements stand in for gold. Placed runs use shelfmark
  `profile:lexical@2` placements of all 13 schemes plus the learned scheme.
- **Uncertainty.** 95% percentile bootstrap intervals (Efron 1979,
  https://doi.org/10.1214/aos/1176344552), with 200 replicates. Graph scores
  resample similarity edges and label scores resample items. Null models and
  noise use 10 draws each. All randomness comes from a seeded SplitMix64
  (seed 20261010), so results reproduce exactly.

"Cost" below is asymptotic, with n items, m graph edges, d tree depth,
C classes, s the embedding sample (2,000) and D the embedding dimension.

## Data-science scores

| Score (key) | Definition | Rewards / punishes | Cost | Gold? | Embedding? | Failure modes | Citation |
|---|---|---|---|---|---|---|---|
| Dasgupta cost, unnormalised (`dasgupta.cost`) and normalised (`dasgupta.cost_norm`) | `Σ_edges w_ij · |items under LCA(i,j)|`, normalised by `n · Σ w`. Lower is better. The Paper 1 headline. | Rewards similar items meeting low in the tree. Punishes flat trees, large catch-all classes and unplaced items, which meet at the root. | O(m·d) | no | graph | Coverage dominates it: unplaced items cost the most. It is not comparable across graphs. | Dasgupta 2016, https://doi.org/10.1145/2897518.2897527 |
| Moseley–Wang revenue and skill (`structure`, `dasgupta.skill`) | revenue = `n·Σw − cost`; skill = `(1 − cost_norm)/(1 − ref)` against the average-linkage tree. Carried over from the harness. | As above, on a scale where the average-linkage tree scores 1. | O(m·d) plus HAC once | no | graph | The reference is a binary tree, which no classification can match. | Moseley & Wang 2017/2023, https://jmlr.org/papers/v24/18-080.html |
| Binarized Dasgupta (`dasgupta.binarized_cost_norm`) | Every multiway node becomes a weight-balanced binary tree over its slots (own items first, then subclasses) in shelf order. The edge pays the items under its LCA in that tree. | Separates "flat but well ordered" from "flat and unordered": order only matters inside the binarisation. Rewards fine leaves. | O(m·d·log fan-out) | no | graph | The balanced split is one fixed choice, not the optimal binarisation (finding that is NP-hard in general), so this is an upper bound on the best binarised cost. | Dasgupta 2016 (cost); construction ours |
| Per-level Dasgupta (`dasgupta.level{1,2,3}_cost_norm`) | Dasgupta cost with the tree cut below depth L, so classes below collapse into their depth-L ancestor. | Shows how much of the score comes from the top levels versus deep classes. | O(m·d) | no | graph | Depth means different things in different schemes (arXiv depth 1 is a group of archives). | Dasgupta 2016 |
| Tree-sampling divergence (`tsd.nats`) | KL divergence, in nats, between the LCA distribution of a sampled edge and that of two items sampled independently by weighted degree. Higher is better. | Rewards trees whose internal nodes concentrate edge mass beyond chance. Unplaced items at the root add little. | O(m·d + C) | no | graph | Unbounded and not comparable across graphs. It is insensitive to the order of children. | Charpentier & Bonald 2019, https://doi.org/10.24963/ijcai.2019/286 |
| Triplet accuracy (`triplet.accuracy`) | For each graph edge (i, j), one non-neighbour k of i is sampled. The triplet is correct when LCA(i, j) is deeper than LCA(i, k); a tie counts one half. | Rewards placing embedding neighbours lower than random pairs. A root-only tree scores 0.5. | O(m·d) | no | graph | One sample per edge. Ties are common in shallow trees, which pulls the score toward 0.5. | Triplet constraints: Schultz & Joachims 2003, https://proceedings.neurips.cc/paper/2003/hash/d3b1fb02964aa64e257f9f26a31f72cf-Abstract.html ; Tamuz et al. 2011, https://arxiv.org/abs/1105.1033 |
| Cophenetic correlation (`embed.cophenetic`) | Pearson r between cosine similarity and LCA depth over all pairs in a fixed sample (every 5th paper, every item of the holdout). | Rewards deeper shared ancestry for more similar pairs. | O(s²·(D + d)) | no | yes | LCA depth stands in for merge height, because scheme trees have no heights. Unplaced items all sit at depth 0. | Sokal & Rohlf 1962, https://doi.org/10.2307/1217208 |
| Silhouette per level (`embed.silhouette_d{1,2,3}`) | Mean silhouette with cosine distance, on the same sample, with classes cut at depth d. Items placed above d or unplaced are excluded. Singletons score 0. | Rewards classes that are tight and well separated in embedding space. | O(s²·D) | no | yes | Near zero for any topical scheme on short abstracts. It ignores the hierarchy below d. | Rousseeuw 1987, https://doi.org/10.1016/0377-0427(87)90125-7 |
| Davies–Bouldin per level (`embed.davies_bouldin_d{1,2,3}`) | Mean over classes of the worst `(S_a + S_b)/‖c_a − c_b‖`, Euclidean. Lower is better. | Punishes overlapping classes. | O(n·D + K²·D) | no | yes | Singleton classes have zero scatter, so fine schemes look better. | Davies & Bouldin 1979, https://doi.org/10.1109/TPAMI.1979.4766909 |
| Label predictability, kNN (`probe.knn_d{1,2,3}`) | Leave-one-out weighted vote of graph neighbours that predicts the depth-d class. Reported with `probe.majority_d*` (majority baseline) and `probe.labelled_d*` (labelled share). | Rewards classes recoverable from the embedding neighbourhood. | O(m) | no | graph | High for schemes that use few classes: compare it with the majority baseline. It covers only labelled items. | Cover & Hart 1967, https://doi.org/10.1109/TIT.1967.1053964 |
| Label predictability, linear probe (`probe.centroid_d{1,2,3}`) | A nearest-centroid classifier, which is linear in the embedding, with 5 folds by item index. | Same as the kNN probe. | O(n·K·D) | no | yes | Nearest-centroid is a weak linear probe, so this is a lower bound on linear separability. | Alain & Bengio 2016, https://arxiv.org/abs/1610.01644 |
| ARI, AMI, V-measure, homogeneity, completeness, purity vs gold (`gold.{ari,ami,v_measure,homogeneity,completeness,purity}_{fine,coarse}`) | Flat partition agreement between the scheme's leaf classes (fine) or depth-1 classes (coarse) and the external gold. Unplaced items form one cluster. AMI uses arithmetic-mean normalisation with the exact hypergeometric expected MI. | Rewards partitions that match the gold. Purity rewards fragmentation; completeness punishes it. | O(n + R·C·min) for AMI | yes | no | Unplaced items form one large cluster that dominates on low-coverage runs. ARI is near 0 for very fine partitions even when they nest inside the gold. | Hubert & Arabie 1985, https://doi.org/10.1007/BF01908075 ; Vinh, Epps & Bailey 2010, https://jmlr.org/papers/v11/vinh10a.html ; Rosenberg & Hirschberg 2007, https://aclanthology.org/D07-1043/ |
| Dendrogram purity (`gold.dendrogram_purity_{fine,coarse}`) | Mean, over same-gold pairs, of the gold class's share among the gold-labelled items under the pair's LCA. | Rewards trees in which same-gold items meet in pure subtrees. Insensitive to fragmenting a gold class across sibling subtrees. | O(n·d) | yes | no | Pairs meeting at the root score the gold class's overall share, so it is never zero. | Heller & Ghahramani 2005, https://doi.org/10.1145/1102351.1102389 ; Kobren et al. 2017, https://doi.org/10.1145/3097983.3098079 |
| Hierarchical F (`gold.hierarchical_f_{fine,coarse}`) | For each gold class, the best F1 over all tree nodes and single items; class-size weighted mean. | Rewards some node matching each gold class at some level. | O(n·d) | yes | no | One good node per class suffices: the rest of the tree is ignored. | Larsen & Aone 1999, https://doi.org/10.1145/312129.312186 |
| Null models (`catalogue.null.{shuffled_labels,random_shape}`) | z-scores of the core scores (cost_norm, triplet, TSD, locality, kNN d1, dendrogram purity, ARI) against two nulls. *Shuffled labels* permutes the placed items' classes, which keeps class sizes and tree. *Random shape* reassigns each depth's classes to the same multiset of parent slots, which keeps level sizes and fan-outs. | Shows how much of a score comes from placement and nesting rather than from tree shape and class sizes. | 10 x core scores | no | graph | z is huge when the null variance is tiny. Read it together with the null mean. | (standard permutation nulls) |
| Placement noise (`catalogue.noise.{0.05,0.1,0.2}`) | Core scores after each placed item moves, with probability p, to the class of a random placed item. | Shows degradation per unit of misplacement and how fragile the scheme's score is. | 30 x core scores | no | graph | Moves follow class frequency, so large classes absorb the noise. | ours |

## Computer-science scores

| Score (key) | Definition | Rewards / punishes | Cost | Gold? | Embedding? | Failure modes | Citation |
|---|---|---|---|---|---|---|---|
| Expected search cost (`search.scheme_bits`) and worst path (`search.worst_bits`) | Expected number of binary questions to reach an item's shelf. Each node's options (its own-items slot, then subclasses that hold items, in shelf order) become a balanced binary tree. The cost is item-weighted. | Rewards shallow, narrow, well-used trees. Punishes wide nodes and long chains. | O(C) | no | no | Uniform within a node: it ignores usage when it splits options. | Knuth 1971, https://doi.org/10.1007/BF00264289 |
| Bounds: entropy, Huffman, Hu–Tucker (`search.{entropy,huffman,hu_tucker}_bits`), gap (`search.gap_bits` = scheme − Hu–Tucker) | Over the same shelf weights: entropy H; Huffman, the optimal code that ignores order; and Hu–Tucker, the optimal alphabetic code that keeps shelf order, computed by Garsia–Wachs. `H ≤ Huffman ≤ Hu–Tucker ≤ scheme`. | The gap is the price of the scheme's shape compared with the best order-preserving search tree over its own shelves. | O(S log S) and O(S²) for S shelves | no | no | The bounds depend on the scheme's own shelves, so a coarse scheme can have a small gap and still give poor discrimination. | Huffman 1952, https://doi.org/10.1109/JRPROC.1952.273898 ; Hu & Tucker 1971, https://doi.org/10.1137/0121057 ; Garsia & Wachs 1977, https://doi.org/10.1137/0206045 |
| Menu navigation (`structure.nav_scan`, `structure.nav_log_a{0,1,2}`) | Harness scan and log-menu costs: Hick-style `a + log2(fan-out)` per level. | Same as search cost, under a human-menu model. | O(n·d) | no | no | Scan cost explodes on root fan-out when coverage is low. | Hick 1952, https://doi.org/10.1080/17470215208416600 ; Landauer & Nachbar 1985, https://doi.org/10.1145/317456.317470 |
| MDL (`mdl.model_bits`, `mdl.data_bits_per_item`, `mdl.total_bits_per_item`) | Two-part code. The model is one parent pointer per scheme class, `C·log2(C+1)` bits. The data is the items' shelves at their empirical entropy. The total is per item. | Rewards small schemes whose classes compress the placements. Punishes huge schemes (LCC-full, IPC) and diffuse placements. | O(C) | no | no | Crude model code: labels and notation are not charged. At n = 10⁴ the model term dominates for large schemes. | Rissanen 1978, https://doi.org/10.1016/0005-1098(78)90005-5 ; Grünwald 2007, https://doi.org/10.7551/mitpress/4643.001.0001 |
| Order preservation (`order.locality`, `order.kendall_tau`) | Shelf position: classes in pre-order, own items first. Locality is `1 − weighted mean |Δpos| over edges / E|U−V|` (0 = random, 1 = adjacent). τ-b is computed between edge similarity and negative shelf distance over 3,000 sampled edges plus 3,000 random pairs. | Rewards keeping neighbours near each other on the linear shelf: Ranganathan's helpful sequence. | O(m) / O(s²) | no | graph | Only placed items count, so a low-coverage run is judged on its easy part. | Kendall 1938, https://doi.org/10.1093/biomet/30.1-2.81 ; Knight 1966, https://doi.org/10.1080/01621459.1966.10480879 |
| Insertion stability (`insertion.splits_per_1000`, `insertion.churn_per_insert`) | Items arrive in corpus order. A class must be subdivided every time its own items pass a multiple of 50, and every item it holds is re-shelved. The scores are splits per 1000 insertions and re-shelved items per insertion. | Rewards fine classes that absorb growth. Punishes catch-alls. | O(n) | no | no | The capacity of 50 is arbitrary, and fixed enumerative schemes never *renumber*. For the learned scheme the real insertion report is quoted in the results (8 splits in 500 inserts, no renumbering of existing codes). | ours |
| Depth, balance and fan-out distributions, Gini (`structure.*`) | From the harness: depth p50/p95/max, branching mean/p95/max, root fan-out, nodes with over 50 children, normalised entropy, effective classes, the sorted-size Gini over class placement counts (kept as is), and the largest class share. | Descriptive. Gini and largest share punish concentration. | O(n·d) | no | no | Descriptive only; no direction of "better". | Gini via Dorfman 1979, https://doi.org/10.2307/1924845 (original Gini 1912: TODO(cite)) |
| Latency (`latency.ms_per_item`) | shelfmark classification wall time per item, from the placement stats: one batch, 4 schemes in parallel on 8 CPUs. | Rewards cheap placement. | - | no | no | Wall clock on a shared machine; it measures the classifier, not the scheme. Placed runs only. | - |

## Librarian scores

| Score (key) | Definition | Rewards / punishes | Cost | Gold? | Embedding? | Failure modes | Citation |
|---|---|---|---|---|---|---|---|
| Literary warrant (`warrant.useful_depth_share`) | Share of all items placed at depth ≥ 2 (below a main class). | Rewards schemes with classes that fit what the corpus contains. | O(n) | no | no | Depth 2 means different specificity in different schemes. | Hulme 1911 via ISKO, https://www.isko.org/cyclo/literary_warrant (original: TODO(cite)) |
| Hospitality in array (`hospitality.array`, `hospitality.ordered_siblings`) | Share of adjacent siblings, in ascending natural order, between which the scheme's code grammar admits an unused code: next integer, decimal or digit extension, letter extension. `ordered_siblings` is the share of sibling pairs listed in ascending natural order. | Rewards notation where new classes fit between existing ones without renumbering. Appending after the last sibling is possible in every scheme here and is not counted. | O(C) | no | no | It is a heuristic over a few candidate codes, so it can under-count. Dense fixed-width notation (the learned scheme's `01.02`) scores 0 although it can always append. Opaque IDs (ACM CCS concept IDs, EuroVoc) are always hospitable but not ordered. | Ranganathan 1967, http://hdl.handle.net/10150/106370 |
| Notation length, expressiveness, regularity (`notation.length`, `notation.expressiveness`, `notation.regularity`, `notation.length_depth_spearman`, `notation.lengthening`) | Item-weighted characters per code. Expressiveness: the share of child codes that extend the parent code. Regularity: the share of codes whose shape (letters as A, digits as 9, runs collapsed) is the modal shape at their depth. The Spearman correlation of code length with depth. `lengthening`: the share of children whose code is no shorter than the parent's. | Short, regular, hierarchical notation is easier to read and shelve. | O(C log C) | no | no | Expressiveness is 0 for opaque IDs by construction. Shape regularity ignores meaning (mnemonics). | Ranganathan 1967 (canons of notation); Svenonius 2000, https://doi.org/10.7551/mitpress/3828.001.0001 |
| Granularity (`granularity.classes_per_1000`, `granularity.works_per_leaf_{mean,median}`, `granularity.empty_class_share`, `granularity.empty_leaf_share`) | Classes holding items per 1000 placed works. Own items per used leaf. The share of all scheme classes, and of scheme leaves, that receive no item. | Shows whether a scheme is too coarse (many works per leaf) or too fine (mostly empty) for this corpus. | O(C) | no | no | Empty share is high for any general scheme on a 10k topical corpus. | Svenonius 2000 |
| Exclusiveness (`canon.exclusiveness`) | Share of placed items whose source gives exactly one class. | Ranganathan's canon of exclusiveness: one place per item. | O(n) | no | no | shelfmark placements are single by construction (1.0). It is informative only for gold labels. | Ranganathan 1967 |
| Collocation by author (`collocation.author_same_class`, `collocation.author_same_class_chance`, `collocation.author_pairs`, `collocation.author_locality`) | Pairs of papers sharing an exact author string (authors with more than 50 papers are dropped as name collisions). Reports the share shelved in the same class, the same share for random items, and their shelf locality. | Rewards bringing an author's works together, the collocation objective of the catalogue. | O(pairs) | no | no | Name strings are not disambiguated. The books carry no authors, so this is computed on papers only. | Cutter 1876, https://archive.org/details/rulesforaprinte00cuttgoog |
| Inter-classifier consistency (`consistency.{items,exact,kappa_exact,depth1,kappa_depth1,depth2,ari}`) | On items labelled by both: exact class agreement, agreement of the depth-1 and depth-2 ancestors, Cohen's kappa (exact and depth 1), and ARI. Source labels vs shelfmark for arXiv, MSC, JEL, LCC and LCC-full; learned own placements vs shelfmark for the learned scheme. | Rewards a scheme that two independent classifiers apply the same way. | O(n·d) | yes | no | JEL has 2 labelled papers. The source labels of the books are deep LCC call numbers resolved to classes. | Cohen 1960, https://doi.org/10.1177/001316446002000104 |
| Ranganathan canons | See the next section. | | | | | | Ranganathan 1967 |

## Ranganathan's canons: which are checkable here

From the *Prolegomena* (3rd ed., Ranganathan 1967,
http://hdl.handle.net/10150/106370), each canon is either mapped to a score
above or listed as not checkable, with the data it would need.

| Canon | Status | Score or missing data |
|---|---|---|
| Exhaustiveness (of an array) | proxy | `coverage` and `warrant.useful_depth_share` |
| Exclusiveness | checked | `canon.exclusiveness` |
| Helpful sequence | checked | `order.locality`, `order.kendall_tau`, `dasgupta.binarized_cost_norm` |
| Consistent sequence | proxy | `notation.regularity`, `hospitality.ordered_siblings` |
| Hospitality in array | checked | `hospitality.array` |
| Hospitality in chain | proxy | `insertion.*` (subdivision pressure) and the learned scheme's insertion report |
| Decreasing extension | proxy | `notation.lengthening`, `notation.length_depth_spearman` |
| Relevant succession / modulation | not checkable | Needs the scheme's characteristics of division, which no benchmark scheme publishes |
| Currency (of terms) | not checkable | Needs dated edit histories of class labels |
| Reticence, enumeration, context | not checkable | Needs labelled-facet data. Captions are free text |
| Mnemonics | not checkable | Needs a mapping from notation segments to meanings (e.g. form divisions) |
| Canon of brevity (notation) | checked | `notation.length` |

Ranganathan's *Five Laws* (1931, https://archive.org/details/in.ernet.dli.2015.283188;
edition unconfirmed) are service principles, not scheme properties. They are
not operationalised.

## Requested scores that the data cannot support

| Score | Why not | Data needed |
|---|---|---|
| Collocation by citation | No citation graph is committed. The arXiv corpus holds no references, and the Open Library holdout holds none. | A citation or co-citation edge list for the corpus (for example from OpenAlex `referenced_works`). Shared author strings are used as the defensible proxy instead. Co-listing in arXiv categories was rejected as a proxy because it is the arXiv scheme's own label. |
| Crosswalk agreement | No open crosswalk links any two benchmark schemes. `research/reports/4-licensing-crosswalks.md` §4 lists the open ones: Wikidata DDC to LCC (942 items), LCSH to LCC (91,581 headings) and FAST to LCSH. None maps arXiv, MSC, JEL, ACM CCS, IPC, EuroVoc, ILC2, BK, FDC, OpenAlex topics or UNESCO to each other, and DDC is not benchmarked. | An open LCC to another-benchmark-scheme concordance, or LCSH on the holdout books joined through the LCSH-LCC links. |
| Silhouette / probes on the full corpus | They are computed on all items for DB and the probes, and on a fixed 2,000-paper sample for silhouette and cophenetic correlation, to keep them O(s²). | - |
| ID churn for fixed schemes | Enumerative schemes never renumber existing codes when a class is added, so churn is 0 by construction. The simulation reports re-shelving pressure instead. | Edition-to-edition change logs (e.g. MSC2010 to MSC2020) to measure real renumbering. |
| Maintenance and licence | Not a computed score. See below. | - |

## Maintenance and licence (from report 4)

This is catalogued, not computed. The licence facts are from
`research/reports/4-licensing-crosswalks.md` §1. The scheme-level terms are
those recorded in `bench/results/2026-10-10/results.json` (`schemes.*.license`).

| Scheme | Licence or terms (as recorded) | Maintenance and provenance |
|---|---|---|
| LCC, LCC-full | US government work, no US copyright (17 U.S.C. 105); id.loc.gov calls its data "a public domain data set"; Classification Web is paid | Library of Congress; no current single-file LCC dump (report 4 §3, §5) |
| OpenAlex topics | CC0 1.0 | `schemes/openalex-topics/README.md` |
| FDC | CC0 1.0 | `schemes/fdc/README.md`, `SOURCES.md` |
| BK | CC0 1.0 | `schemes/bk/README.md` |
| UNESCO Thesaurus | CC BY-SA 3.0 IGO | `schemes/unesco-thesaurus/README.md` |
| ILC2 | CC BY 3.0 in the TTL; BARTOC says BY-SA 3.0 (unresolved, report 4) | `schemes/ilc2/README.md` |
| IPC | CC BY 4.0; built locally, not committed (size) | `schemes/ipc/README.md`, `SOURCES.md` |
| ACM CCS 2012 | "freely available for educational and research purposes"; not redistributable | `schemes/acm-ccs/README.md`; report 5 on the 1998/2012 gap |
| EuroVoc | reuse terms stated inconsistently; built from the Publications Office download | `schemes/eurovoc/README.md` |
| MSC 2020, JEL | fetch-only, not bundled | `schemes/msc/`, `schemes/jel/` READMEs |
| arXiv | bundled taxonomy | `schemes/arxiv/README.md`, `sources/LICENSE` |
| learned | generated in this repo from the CC0 corpus | `schemes/learned/README.md` |

## Implementation map

| Module (`crates/bench/src/`) | Scores |
|---|---|
| `hier.rs` | primary-class hierarchy, shelf order, labels per depth |
| `treescore.rs` | Dasgupta (plain, binarized, per level), TSD, triplets, seeded RNG |
| `partition.rs` | ARI, AMI, V-measure, purity, Cohen's kappa |
| `goldtree.rs` | dendrogram purity, hierarchical F |
| `embedscore.rs` | silhouette, Davies–Bouldin, kNN and centroid probes, cophenetic correlation |
| `search.rs` | search cost, Huffman, Garsia–Wachs (Hu–Tucker), MDL |
| `shelf.rs` | locality, Kendall τ-b, author collocation, insertion simulation |
| `notation.rs` | notation, hospitality, granularity, warrant |
| `robust.rs` | null models, noise, bootstrap helpers |
| `agree.rs` | Spearman, Kendall, clustering order for the agreement heatmap |
| `catalogue.rs` | assembles all scores, CIs, nulls, noise; consistency |

Each module has unit tests on hand-checkable trees (the five-item toy tree
in `hier.rs` and small hand-computed cases).
