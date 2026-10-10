#import "../template.typ": *
#import "../generated.typ": v

= Method <sec:method>

== Similarity graph <sec:graph>

Each item's text is the title and abstract joined by a full stop (titles only
for the book holdout, which has no descriptions). It is embedded with
`sentence-transformers/all-MiniLM-L6-v2` at a pinned ONNX revision @reimers2019 @wang2020minilm @minilmcard @minilm,
and vectors are L2-normalized. Raw cosine similarities between unrelated
texts are not centred on zero, so we subtract the exact mean cosine $tau$ over
all unordered pairs: $tau = 0.109$ on the arXiv corpus and $tau = 0.173$ on
the holdout. Each item keeps its $k = 20$ most similar items. The weight of
an edge is $w_(i j) = max(0, cos_(i j) - tau)$. The graph is symmetrized by
taking the maximum, and zero-weight edges are dropped. That leaves 148,111
edges on the corpus and 7,297 on the holdout. Every scheme and the reference
tree are scored on this one graph, so all comparisons on a corpus are paired.
The classifier's optional embedding tier uses a different model (BGE-small,
not used in this run). This follows the benchmark design, which aims to avoid
a circular advantage for schemes whose labels embed well.

== From a scheme and placements to a tree <sec:induced>

A scheme is a set of classes, each with at most one parent, so it is a tree
or a forest. For polyhierarchies such as EuroVoc and the UNESCO Thesaurus,
the port has already chosen one primary parent per concept. Given one
placement per item, we keep the used classes and all their ancestors. Forest
roots hang under a virtual root. Each item becomes a leaf under its class
node, including when that class is an internal node. An item with no
placement becomes a leaf directly under the root, so every pair involving it
pays the full corpus size. Coverage is therefore penalized _inside_ the cost,
and we also report it separately. All reported runs use one class per item:
the first listed code for gold labels and the classifier's top choice
otherwise. The harness also supports fractional (multi-copy) placement,
which we leave for later work.

== Cost, revenue and skill <sec:skill>

Let $n$ be the number of items (placed or not), $E$ the edges of the graph,
$W = sum_((i,j) in E) w_(i j)$, and $N(u)$ the number of items in the subtree
of node $u$. Then

$ "cost"(T) &= sum_((i,j) in E) w_(i j) N("lca"_T (i,j)), \
  "cost"_"norm" &= "cost"(T) / (n W), \
  "revenue"_"norm" &= 1 - "cost"_"norm". $ <eq:cost>

A tree with every item under the root has $"cost"_"norm" = 1$. By the
Moseley--Wang identity, $n W - "cost"(T)$ is the revenue, so
$"revenue"_"norm"$ is revenue over its trivial upper bound. The sum runs over
graph edges only. This is a $k$NN-sparsified Dasgupta cost: the pairs that
matter are those the embedding calls close.

Revenue depends on the corpus and the graph, so we normalize it against a
reference tree $T_"ref"$. That tree is a deterministic average-linkage (UPGMA)
dendrogram on the same graph, with missing edges treated as similarity 0:

$ "skill"(T) = (1 - "cost"_"norm" (T)) / (1 - "cost"_"norm" (T_"ref")). $ <eq:skill>

Skill is 0 for the flat tree and 1 for a tree as good as average linkage.
Average linkage is a $1/3$-approximation of maximum revenue @moseley2023, so
the reference is an achievable tree, not an optimum. Skill may exceed 1. The
reference reaches $"cost"_"norm"$ = #v.ref_corpus on the corpus and
#v.ref_holdout on the holdout.

The reference is a binary tree over individual items. A scheme's tree stops
at classes, and all items inside one class share an LCA. This tie is
intended: it models scanning the class shelf. It also means that no scheme
whose classes hold more than one item can reach skill 1 easily. Skill
compares a scheme with a data-fitted tree. It is not a pass mark.

== Gold and shelfmark-placed subsets <sec:subsets>

A score depends on the scheme _and_ on how works were placed into it. We keep
two subsets apart.

- *Gold (track A).* Labels supplied by the source: arXiv primary categories,
  author-supplied MSC and JEL codes on the arXiv corpus, and LC call numbers
  on the book holdout. Call numbers are resolved to the deepest covering
  class of the bundled LCC outline (`lcc`) and of the complete LCC schedules
  (`lcc-full`). The resolver tries the full call number, then the class
  number without cutters, then the class letters.
- *Placed (track P).* Labels assigned by shelfmark's lexical classifier
  (`profile:lexical@2`). Each scheme gets an isolated store with no
  hand-written rules. Placements below the classifier's 0.8 confidence target
  are kept and flagged. Fallback placements, where no tier produced a
  candidate, count as unplaced.

The two subsets are never ranked against each other. Where both exist for the
same scheme and items (arXiv, MSC and JEL on the corpus; LCC and `lcc-full`
on the holdout), their difference measures the classifier's effect
(@sec:bias).

== Structure, coverage and navigation <sec:structure>

For each run we report:

- *Coverage:* the share of items with a placement.
- *Depth:* the mean, median, 95th percentile and maximum item depth, in edges
  from the root to the item leaf. The root is depth 0, and an item placed in
  a top-level class has depth 2.
- *Branching:* the fan-out of used internal nodes. Item leaves count as
  children, so unplaced items inflate root fan-out.
- *Balance:* the effective number of classes $exp(H)$ and the Gini
  coefficient. Both use the distribution $p_u prop N(u)$ over used class
  nodes at _all_ levels, so an item counts once at each ancestor class.
  These are multi-level summaries, not leaf-partition statistics.
- *Expected navigation cost* under uniform use, with two menu models. For an
  item $i$ with ancestors $u$ of fan-out $b(u) > 1$,
  $ "nav"_"log" (a) = 1/n sum_i sum_u (a + log_2 b(u)) $ <eq:navlog>
  is a Hick--Hyman-style decision cost with per-level overhead
  $a in {0, 1, 2}$. The second model is
  $ "nav"_"scan" = 1/n sum_i [ sum_u (b(u)+1)/2 + (n_c+1)/2 ], $ <eq:navscan>
  a linear menu scan plus a scan of the $n_c$ items under the item's deepest
  class. These parameters are assumptions, not fitted values. We report the
  sweep over $a$.

== Implementation <sec:impl>

The harness is a Rust crate, `shelfmark-bench`. It walks parent pointers to
find LCAs, which the shallow class trees make cheap, and computes the
average-linkage reference once per graph with a size-weighted
(Lance--Williams) update and deterministic tie-breaking. Unit tests check a
hand-computed cost, the identity cost $+$ revenue $= n W$, that a root-only
tree has $"cost"_"norm" = 1$, and that the embedder, the graph and the
reference are deterministic. Each result records the SHA-256 of the graph,
the induced tree and the scheme file.
