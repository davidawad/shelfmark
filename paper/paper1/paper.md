---
title: "Scoring universal classification schemes with hierarchical-clustering objectives"
author: "AUTHOR TBD"
status: "DRAFT. Not submitted, posted or shared."
bibliography: references.bib
---

<!--
DRAFT. Every number in the tables, and every number in the prose wrapped in a
`v:` marker, is generated from bench/results/2026-10-10 (gold),
bench/results/2026-10-10-r3 (gold rescored on the fixed corpus) and
bench/results/2026-10-10-r2 (shelfmark-placed) by scripts/make_tables.py.
Do not edit generated text by hand; rerun the script. Citation keys refer to
references.bib. TODO(cite) marks a claim with no verified source yet.
-->

# Abstract

Library classification schemes are compared mostly by description: their
history, notation, coverage and licence. We are not aware of a published
comparison that scores several universal or near-universal schemes on one
quantitative objective over the same set of works. We propose such a
comparison, built on objectives from the theory of hierarchical
clustering. A scheme, restricted to the classes that a corpus actually uses,
is a rooted tree over the works. We score that tree with Dasgupta's cost
[@dasgupta2016] and Moseley and Wang's revenue [@moseley2023] on a fixed
k-nearest-neighbour cosine graph of text embeddings, and normalize revenue
against an average-linkage tree built on the same graph. We call this ratio skill. We
report coverage, structural statistics and expected navigation cost next to
skill. The corpus is <!-- v:n_corpus -->10,000<!-- /v --> arXiv papers with
CC0 metadata plus a holdout of <!-- v:n_holdout -->500<!-- /v --> Open
Library editions with librarian-assigned LC call numbers. We separate
source-assigned ("gold") placements from placements made by one lexical
classifier, shelfmark. On gold labels, arXiv's own taxonomy reaches skill
<!-- v:gold_arxiv_skill -->0.786<!-- /v --> on arXiv papers and LCC reaches
<!-- v:gold_lcc_skill -->0.419<!-- /v --> on the book holdout. With the
classifier's placements, the same two schemes fall to
<!-- v:placed_arxiv_corpus_skill -->0.128<!-- /v --> and
<!-- v:placed_lcc_holdout_skill -->0.005<!-- /v -->. Across
<!-- v:n_placed_corpus -->13<!-- /v --> schemes placed by the classifier,
skill ranges from <!-- v:placed_worst_skill -->0.059<!-- /v -->
(<!-- v:placed_worst -->msc<!-- /v -->) to
<!-- v:placed_best_skill -->0.423<!-- /v -->
(<!-- v:placed_best -->openalex-topics<!-- /v -->), and its Spearman
correlation with coverage is <!-- v:rho_cov_skill -->0.76<!-- /v -->. The measurement is
cheap and its inputs are pinned by hash. Its weak point is the classifier: in this
run, classifier-placed scores rank how well the classifier reaches a scheme
more than they rank the scheme itself. We also report a classifier fix found
by the benchmark. Before the fix, the classifier matched class labels made
only of stopwords; afterwards, the largest single-class capture in ILC2
fell from <!-- v:sw_ilc2_before -->73.7%<!-- /v --> to
<!-- v:sw_ilc2_after -->4.2%<!-- /v --> of placed papers.

# 1. Introduction

A universal classification scheme gives every work one address in a single
tree. Library of Congress Classification (LCC), Dewey (DDC), UDC, the Basic
Classification (BK), the Integrative Levels Classification (ILC) and others
each claim to arrange all of knowledge. A practitioner choosing among them, or
a researcher designing a new one, needs to ask which arrangement keeps
related works close together, and at what navigation cost. The literature
answers this mostly qualitatively, by notation, facet structure, history and
institutional adoption. TODO(cite: a knowledge-organization survey of scheme
evaluation criteria). In our search we found no published study that
scores several universal schemes on a common quantitative objective over the
same corpus. Our search was not systematic (see Limitations), so this is
absence of evidence, not proof. TODO(cite: systematic search of JASIST,
Knowledge Organization and ISKO proceedings).

Three developments make the question more pressing.

1. **The main lookup service is gone.** OCLC Classify returned the most
   common DDC and LCC numbers for a work from WorldCat holdings. OCLC
   discontinued it on 31 January 2024 [@oclcclassify]. No free service
   replaced it. Its successor endpoint is in the WorldCat Metadata API,
   which is open only to subscribers.
2. **Automatic placement has not closed the gap.** Annif, the most mature open
   subject-indexing toolkit, is trained separately for each vocabulary
   [@annif]. On Swedish national-bibliography records it reaches 66.82%
   accuracy at three-digit DDC and 44.87% precision@1 at full DDC
   [@libris2024ddc]. Recent work that uses large language models (LLMs)
   for LCC and DDC assignment reports weakness at fine-grained classes
   [@llmlcc2026; @llm4ddc2025; @nasko2025]. The Library of Congress's
   own experiments predicted subject and genre with under 50% accuracy,
   short of a 95% automation threshold [@lcecd2025]. Placement is getting
   cheaper, so the open question shifts towards *which scheme* to place
   into.
3. **Licences constrain open use.** DDC is copyrighted by OCLC
   [@deweyrights]. The full UDC is licensed [@udclicences]. Openly licensed
   universal schemes are shallow, such as BK [@bk] and FDC [@fdc], or their
   licence terms are unsettled, as for ILC2 [@ilc2]. Data-driven
   vocabularies such as OpenAlex Topics are open but cover research
   literature only [@openalextopics]. A quantitative comparison helps
   decide whether an open scheme is good enough to adopt.

**Contributions.** (i) A scheme-agnostic scoring method. It induces a tree
from a scheme and a set of placements, and scores the tree with Dasgupta
cost and Moseley–Wang revenue on a fixed similarity graph. The score is
normalized against an average-linkage reference tree on the same graph, and
coverage, structure and navigation statistics are reported with it
(Section 3). (ii) A pinned, CC0-metadata corpus of
<!-- v:n_corpus -->10,000<!-- /v --> arXiv papers and a
<!-- v:n_holdout -->500<!-- /v -->-edition LC holdout (Section 4).
(iii) Results for <!-- v:n_schemes -->13<!-- /v --> schemes. Gold
placements and classifier placements are kept apart, and the classifier
bias is calibrated where both exist (Section 5). (iv) A methods note on a
classifier defect that the benchmark exposed, with its before-and-after
effect (Section 5.5).

# 2. Related work

**Objectives for hierarchical clustering.** Dasgupta defined the cost of a
tree $T$ over items with pairwise similarities $w_{ij}$ as
$\sum_{i<j} w_{ij}\,|\mathrm{leaves}(T[i \vee j])|$, where $i \vee j$ is
the lowest common ancestor (LCA) of $i$ and $j$ [@dasgupta2016]. Similar
pairs should be separated as deep in the tree as possible. A flat tree has
the maximum cost. Moseley and Wang studied the dual, revenue,
$\sum w_{ij}(n - |\mathrm{leaves}(T[i\vee j])|)$, so that cost plus revenue
equals $n\sum w_{ij}$. They showed that average linkage is a
1/3-approximation to the maximum revenue [@moseley2017; @moseley2023].
Optimizing Dasgupta cost exactly is NP-hard. Approximation algorithms use
sparsest cut and spreading metrics [@charikar2016; @roy2016].
Cohen-Addad et al. give an axiomatic family of admissible objectives and
show when average linkage and its variants recover planted hierarchies
[@cohenaddad2017]. Hyperbolic embeddings give a differentiable relaxation
[@chami2020], and structural constraints can be added to the search
[@chatziafratis2018]. scikit-network ships a Dasgupta cost and score for
binary dendrograms [@sknetwork]. This line of work treats the
objective as something to *optimize*. We use it only to *evaluate* trees
that people built.

**No objective-free optimum.** Kleinberg's impossibility theorem says no
clustering function satisfies scale invariance, richness and consistency
together [@kleinberg2002]. Carlsson and Mémoli characterize single linkage
axiomatically among hierarchical methods [@carlsson2010]. One consequence
for us: a score is always relative to the similarity function. We therefore
fix and publish the graph and treat it as part of the measurement.

**Navigation.** Optimal search trees and Huffman coding minimize expected
depth under an access distribution [@huffman1952; @knuth1971]. The
hierarchical-identity model of Watts, Dodds and Newman makes greedy
navigation on a hierarchy a measurable success criterion [@watts2002].
Basic-level categories explain why a natural middle level carries most
human use [@rosch1976]. We report simple, uniform-use navigation costs. We
fit no user model.

**Data-driven and LLM-built taxonomies.** Waltman and van Eck cluster
publications on direct citation at several resolutions [@waltman2012],
using the Leiden algorithm [@traag2019]. OpenAlex Topics come from citation
clustering and are named by an LLM, and their upper levels are mapped from
Scopus ASJC categories [@priem2022; @openalextopics]. TaxoGen and
TaxoExpan build and extend topic taxonomies from text
[@zhang2018taxogen; @shen2020taxoexpan]. LLM pipelines generate layers or
labels: Chain-of-Layer, TnT-LLM and TaxoAdapt
[@zeng2024chainoflayer; @wan2024tntllm; @kargupta2025taxoadapt]. None of
these optimizes a global objective. Taxonomy-quality work catalogues
attributes and measures [@unterkalmsteiner2023]. It includes a
reference-free robustness score that correlates semantic and taxonomic
similarity [@wullschleger2025], which is closest in spirit to ours.

**Automatic subject classification.** Annif [@annif], its SemEval-2025
Task 5 entry [@annifsemeval2025] and the task overview
[@semeval2025task5] cover subject indexing with thesauri such as GND. They
report results per vocabulary, not across schemes. An open LCC classifier
covers 15 of the 21 top-level classes [@autolcc]. We found no public
leaderboard that compares classification schemes, as opposed to
classifiers.

# 3. Method

## 3.1 Similarity graph

Each item's text is `title + ". " + abstract` (titles only for the book
holdout, which has no descriptions). It is embedded with
<!-- v:model -->sentence-transformers/all-MiniLM-L6-v2<!-- /v --> at a
pinned ONNX revision [@minilm], and vectors are L2-normalized. Raw cosine
similarities between unrelated texts are not centred on zero, so we
subtract the exact mean cosine $\tau$ over all unordered pairs:
$\tau$ = <!-- v:tau_corpus -->0.109<!-- /v --> on the arXiv corpus and
<!-- v:tau_holdout -->0.173<!-- /v --> on the holdout. Each item keeps its
$k$ = <!-- v:k -->20<!-- /v --> most similar items. The weight of an edge is
$w_{ij} = \max(0, \cos_{ij} - \tau)$. The graph is symmetrized by taking the
maximum, and zero-weight edges are dropped. That leaves
<!-- v:edges_corpus -->148,111<!-- /v --> edges on the corpus and
<!-- v:edges_holdout -->7,297<!-- /v --> on the holdout. Every scheme and
the reference tree are scored on this one graph, so all comparisons on a
corpus are paired. We used a different embedding model for similarity than
the classifier's optional embedding tier (BGE-small, not used in this run).
This follows the benchmark design, which aims to avoid a circular advantage
for schemes whose labels embed well (research/benchmark/DESIGN.md §2.2).

## 3.2 From a scheme and placements to a tree

A scheme is a set of classes, each with at most one parent, so it is a tree
or a forest. For polyhierarchies such as EuroVoc and the UNESCO Thesaurus,
the port has already chosen one primary parent per concept. Given one
placement per item, we keep the used classes and all their ancestors. Forest
roots hang under a virtual root. Each item becomes a leaf under its class
node, including when that class is an internal node. An item with no
placement becomes a leaf directly under the root, so every pair involving
it pays the full corpus size. Coverage is therefore penalized *inside* the
cost, and we also report it separately. All reported runs use one class per
item, the first listed code for gold labels and the classifier's top choice
otherwise. The harness also supports fractional (multi-copy) placement,
which we leave for later work.

## 3.3 Cost, revenue and skill

Let $n$ be the number of items (placed or not), $E$ the edges of the graph,
$W = \sum_{(i,j)\in E} w_{ij}$, and $N(v)$ the number of items in the
subtree of node $v$. Then

$$\mathrm{cost}(T) = \sum_{(i,j)\in E} w_{ij}\, N(\mathrm{lca}_T(i,j)),\qquad
\mathrm{cost_{norm}} = \frac{\mathrm{cost}(T)}{nW},\qquad
\mathrm{revenue_{norm}} = 1 - \mathrm{cost_{norm}}.$$

A tree with every item under the root has $\mathrm{cost_{norm}} = 1$. By the
Moseley–Wang identity, $nW - \mathrm{cost}(T)$ is the revenue, so
$\mathrm{revenue_{norm}}$ is revenue over its trivial upper bound. The
sum runs over graph edges only. This is a kNN-sparsified Dasgupta cost: the
pairs that matter are those the embedding calls close.

Revenue depends on the corpus and the graph, so we normalize it against a
reference tree $T_\mathrm{ref}$. That tree is a deterministic
average-linkage (UPGMA) dendrogram on the same graph, with missing edges
treated as similarity 0:

$$\mathrm{skill}(T) = \frac{1-\mathrm{cost_{norm}}(T)}{1-\mathrm{cost_{norm}}(T_\mathrm{ref})}.$$

Skill is 0 for the flat tree and 1 for a tree as good as average linkage.
Average linkage is a 1/3-approximation of maximum revenue
[@moseley2023], so the reference is an achievable tree, not an optimum.
Skill may exceed 1. The reference reaches $\mathrm{cost_{norm}}$ =
<!-- v:ref_corpus -->0.152<!-- /v --> on the corpus and
<!-- v:ref_holdout -->0.301<!-- /v --> on the holdout.

The reference is a binary tree over individual items. A scheme's tree stops
at classes, and all items inside one class share an LCA. This tie is
intended: it models scanning the class shelf. It also means that no scheme
whose classes hold more than one item can reach skill 1 easily. Skill
compares a scheme with a data-fitted tree. It is not a pass mark.

## 3.4 Gold and shelfmark-placed subsets

A score depends on the scheme *and* on how works were placed into it. We
keep two subsets apart.

- **Gold (track A).** Labels supplied by the source. These are arXiv
  primary categories, author-supplied MSC and JEL codes on the arXiv
  corpus, and LC call numbers on the book holdout. Call numbers are resolved
  to the deepest covering class of the bundled LCC outline (`lcc`) and of
  the complete LCC schedules (`lcc-full`). The resolver tries the full call
  number, then the class number without cutters, then the class letters.
- **Placed (track P).** Labels assigned by shelfmark's lexical classifier
  (`profile:lexical@2`). Each scheme gets an isolated store with no
  hand-written rules. Placements below the classifier's 0.8 confidence
  target are kept and flagged. Fallback placements, where no tier produced a
  candidate, count as unplaced.

The two subsets are never ranked against each other. Where both exist for
the same scheme and items (arXiv, MSC and JEL on the corpus; LCC and
`lcc-full` on the holdout), their difference measures the classifier's
effect (Section 5.3).

## 3.5 Structure, coverage and navigation

For each run we report:

- **Coverage:** the share of items with a placement.
- **Depth:** the mean, median, 95th percentile and maximum item depth, in
  edges from the root to the item leaf. The root is depth 0, and an item
  placed in a top-level class has depth 2.
- **Branching:** the fan-out of used internal nodes. Item leaves count as
  children, so unplaced items inflate root fan-out.
- **Balance:** the effective number of classes $\exp(H)$ and the Gini
  coefficient. Both use the distribution $p_v \propto N(v)$ over used class
  nodes at *all* levels. An item therefore counts once at each ancestor
  class, so these are multi-level summaries, not leaf-partition statistics.
- **Expected navigation cost** under uniform use, with two menu models. For
  an item $i$ with ancestors $v$ of fan-out $b(v) > 1$:
  $\mathrm{nav_{log}}(a) = \frac1n\sum_i\sum_v (a + \log_2 b(v))$, a
  Hick–Hyman-style decision cost with per-level overhead
  $a\in\{0,1,2\}$. The second model is
  $\mathrm{nav_{scan}} = \frac1n\sum_i [\sum_v (b(v)+1)/2 + (n_c+1)/2]$:
  a linear menu scan plus a scan of the $n_c$ items under the item's
  deepest class. These parameters are assumptions, not fitted values. We
  report the sweep.

## 3.6 Implementation

The harness is a Rust crate, `shelfmark-bench`. It walks parent pointers
to find LCAs, which the shallow class trees make cheap, and computes the
average-linkage reference once per graph with a size-weighted
(Lance–Williams) update and deterministic tie-breaking. Unit tests check a
hand-computed cost, the identity cost + revenue $= nW$, that a root-only
tree has $\mathrm{cost_{norm}} = 1$, and that the embedder, the graph and
the reference are deterministic. Each result records the SHA-256 of the
graph, the induced tree and the scheme file.

# 4. Corpus

**arXiv corpus.** The arXiv OAI-PMH endpoint (`arXivRaw`) was harvested for
records with datestamps from 2026-01-01 to 2026-10-09 in eight top-level
groups. A seeded SHA-256 ranking then sampled
<!-- v:n_corpus -->10,000<!-- /v --> records, proportional to group size
with a floor of 100 per group (Table 1). arXiv applies CC0 1.0 to
metadata [@arxivlicense]. Datestamps follow record updates, so older papers
appear: <!-- v:share_2026_ids -->69.7%<!-- /v --> of identifiers carry a
2026 prefix. Computer science, mathematics and physics make up
<!-- v:share_physical_math_cs -->86.4%<!-- /v --> of the sample. Gold
labels in other schemes are sparse. <!-- v:n_msc_rows -->1,325<!-- /v -->
papers carry an MSC code, <!-- v:n_acm_rows -->317<!-- /v --> an ACM class
field and <!-- v:n_jel_rows -->2<!-- /v --> a JEL code.

Table 1. Composition of the arXiv corpus by primary category.

<!-- table:corpus -->
| arXiv top-level group (primary category) | papers | share |
| --- | ---: | ---: |
| cs | 3,968 | 39.7% |
| econ | 185 | 1.8% |
| eess | 519 | 5.2% |
| math | 2,042 | 20.4% |
| physics | 2,630 | 26.3% |
| q-bio | 172 | 1.7% |
| q-fin | 139 | 1.4% |
| stat | 345 | 3.5% |
| **total** | 10,000 | 100.0% |
<!-- /table:corpus -->

**Open Library holdout.** <!-- v:n_holdout -->500<!-- /v --> Open Library
edition records with a non-empty `lc_classifications` array were drawn
through the Open Library Search and editions APIs
[@openlibrarylicensing]. They hold titles only, with no description:
<!-- v:holdout_with_text -->0<!-- /v --> rows have text beyond the title,
and titles average <!-- v:holdout_title_words -->3.5<!-- /v --> words. The
holdout is skewed: <!-- v:holdout_p -->382<!-- /v --> records
(<!-- v:holdout_p_share -->76.4%<!-- /v -->) have a first call number in
class P (language and literature). The other
<!-- v:holdout_letter_count -->14<!-- /v --> letters seen are
<!-- v:holdout_other_top -->B 32, H 19, Q 8, D 5, M 4, R 3, U 2, N 2, C 2, J 1, G 1, E 1, L 1<!-- /v -->.
<!-- v:holdout_empty -->37<!-- /v --> records hold only an empty string in
the array. Open Library asks contributors to share under CC0 1.0. Both
files are pinned by SHA-256 and by ID lists.

**Schemes.** Table 2 lists the
<!-- v:n_schemes -->13<!-- /v --> schemes as built for the run. Five are
general or universal: LCC (outline and complete schedules), FDC, BK and
ILC2. The rest are thesauri or vocabularies (UNESCO Thesaurus, EuroVoc,
OpenAlex Topics) or domain schemes (arXiv, MSC, JEL, ACM CCS, IPC). Each
scheme file's SHA-256 is recorded, and the r2 run reused byte-identical
files.

Table 2. Schemes, class counts and licence as recorded by the build.

<!-- table:schemes -->
| scheme | version | classes | licence as recorded by the build |
| --- | --- | ---: | --- |
| `acm-ccs` | 2012 | 2,113 | ACM, Inc.: educational and research use; not redistributable, so not bundled |
| `arxiv` | 2024-04-01 | 192 | see scheme README |
| `bk` | 2026-10-03 | 2,140 | CC0 1.0 |
| `eurovoc` | 4.24 | 7,663 | EU reuse terms stated inconsistently (Commission reuse notice in dataset metadata, CC0 in a third-party registry); not bundled, built from the Publications Office download |
| `fdc` | 0.06 | 1,097 | CC0 1.0 |
| `ilc2` | 2019-10-17 | 10,117 | Unsettled: the publisher's SKOS file says CC BY 3.0, BARTOC lists CC BY-SA 3.0, no iskoi.org page states a license; not bundled |
| `ipc` | 2026.01 | 80,145 | CC BY 4.0 |
| `jel` | 2026 | 1,015 | see scheme README |
| `lcc` | 2024 | 7,392 | see scheme README |
| `lcc-full` | 2024 | 416,186 | US government work, no US copyright (17 U.S.C. 105) |
| `msc` | 2020 | 6,603 | see scheme README |
| `openalex-topics` | 2026-10-09 | 4,798 | CC0 1.0 |
| `unesco-thesaurus` | 2026-08-31 | 4,595 | CC BY-SA 3.0 IGO (https://creativecommons.org/licenses/by-sa/3.0/igo/) |
<!-- /table:schemes -->

The scheme READMEs give the licences for the rows marked "see scheme
README". arXiv's taxonomy comes from arxiv-base, under MIT
[@arxivtaxonomy]. MSC2020 is CC BY-NC-SA 4.0 [@msc2020]. JEL publishes no
open licence [@jel]. The LCC outline is built from Library of Congress
publications [@lccfree; @idloc; @usc105]. Sources for the others: IPC
[@ipc], ACM CCS [@acmccs], EuroVoc [@eurovoc], UNESCO Thesaurus
[@unesco].

# 5. Results

Every table in this section is generated from the published JSON (Section
7). Rows are sorted by $\mathrm{cost_{norm}}$, with the best first.

## 5.1 Gold placements

Table 3. Gold placements on the arXiv corpus (bench/results/2026-10-10;
`msc` rescored on the fixed corpus in bench/results/2026-10-10-r3).

<!-- table:gold_corpus -->
| scheme | version | coverage | cost_norm | revenue_norm | skill | eff. classes | mean depth | max depth | max fan-out | Gini | nav_log (a=1) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `arxiv` | 2024-04-01 | 100.0% | 0.333 | 0.667 | 0.786 | 36.5 | 3.90 | 4 | 797 | 0.795 | 18.3 |
| `msc` | 2020 | 13.2% | 0.984 | 0.016 | 0.018 | 461.5 | 1.39 | 4 | 25 | 0.599 | 15.2 |
| `jel` | 2026 | 0.02% | 1.000 | 0.000 | 0.000 | 6.0 | 1.00 | 4 | 1 | 0.000 | 14.3 |
<!-- /table:gold_corpus -->

Table 4. Gold placements on the book holdout. LC call numbers are resolved
to the deepest covering class.

<!-- table:gold_holdout -->
| scheme | version | coverage | cost_norm | revenue_norm | skill | eff. classes | mean depth | max depth | max fan-out | Gini | nav_log (a=1) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `lcc-full` | 2024 | 92.2% | 0.706 | 0.294 | 0.421 | 85.2 | 5.19 | 10 | 68 | 0.718 | 19.7 |
| `lcc` | 2024 | 92.2% | 0.708 | 0.292 | 0.419 | 35.0 | 4.20 | 9 | 89 | 0.791 | 18.6 |
<!-- /table:gold_holdout -->

arXiv's own taxonomy, applied to arXiv papers by their authors, reaches skill
<!-- v:gold_arxiv_skill -->0.786<!-- /v --> with full coverage. This is the
highest score in the study. We read it as an upper reference for what a
well-fitted scheme with correct placements achieves on this graph, not as
evidence that arXiv's taxonomy is a good *universal* scheme. The arXiv
categories and the abstracts come from the same authors, and the corpus was
sampled by arXiv group.

On the books, LCC call numbers resolved into the outline reach skill
<!-- v:gold_lcc_skill -->0.419<!-- /v --> and into the complete schedules
<!-- v:gold_lcc-full_skill -->0.421<!-- /v -->, at the same coverage
(<!-- v:gold_lcc_cov -->92.2%<!-- /v -->). The complete schedules use more
effective classes (<!-- v:g_lcc-full-gold_eff -->85<!-- /v --> vs
<!-- v:g_lcc-gold_eff -->35<!-- /v -->) and are deeper
(<!-- v:g_lcc-full-gold_depth -->5.19<!-- /v --> vs
<!-- v:g_lcc-gold_depth -->4.20<!-- /v --> mean depth). Yet cost barely
moves (<!-- v:g_lcc-full-gold_cost -->0.706<!-- /v --> vs
<!-- v:g_lcc-gold_cost -->0.708<!-- /v -->). With 500 short titles and a
holdout that is three quarters class P, the title-embedding graph does not
reward the finer LCC distinctions. This does not mean the distinctions are
useless; it is a limit of what the graph can see.

MSC and JEL gold results are uninformative. With
<!-- v:gold_msc_cov -->13.2%<!-- /v --> and
<!-- v:gold_jel_cov -->0.02%<!-- /v --> coverage, almost every pair
involves an item under the root. ACM CCS has no gold run at all. The
corpus's `acm_ccs` field holds the ACM classes submitted to arXiv, and no
value in it is a CCS 2012 concept path. The run log records: "<!-- v:skipped_acm_note -->the corpus acm_ccs field (690 values on 317 papers) holds ACM CCS 1998 codes such as `I.2.6`, as submitted to arXiv; the scheme is CCS 2012, whose codes are paths of 8-digit concept IDs, and no CCS 1998 to 2012 crosswalk is used<!-- /v -->".

## 5.2 shelfmark-placed results

Table 5. Placed by shelfmark (`profile:lexical@2`) on the arXiv corpus
(bench/results/2026-10-10-r2).

<!-- table:placed_corpus -->
| scheme | version | coverage | cost_norm | revenue_norm | skill | eff. classes | mean depth | max depth | max fan-out | Gini | nav_log (a=1) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `openalex-topics` | 2026-10-09 | 97.3% | 0.641 | 0.359 | 0.423 | 77.6 | 4.45 | 5 | 554 | 0.892 | 26.9 |
| `unesco-thesaurus` | 2026-08-31 | 97.3% | 0.721 | 0.279 | 0.329 | 146.7 | 4.90 | 9 | 535 | 0.852 | 26.7 |
| `ipc` | 2026.01 | 98.0% | 0.769 | 0.231 | 0.272 | 317.8 | 6.31 | 10 | 794 | 0.838 | 27.3 |
| `ilc2` | 2019-10-17 | 100.0% | 0.819 | 0.181 | 0.214 | 295.1 | 5.09 | 13 | 417 | 0.824 | 23.4 |
| `lcc-full` | 2024 | 100.0% | 0.823 | 0.177 | 0.209 | 657.2 | 6.06 | 11 | 121 | 0.803 | 22.9 |
| `acm-ccs` | 2012 | 74.5% | 0.831 | 0.169 | 0.200 | 84.9 | 3.61 | 7 | 877 | 0.861 | 23.7 |
| `fdc` | 0.06 | 77.0% | 0.834 | 0.166 | 0.196 | 54.5 | 3.21 | 7 | 1103 | 0.836 | 25.6 |
| `eurovoc` | 4.24 | 97.4% | 0.848 | 0.152 | 0.179 | 269.3 | 4.98 | 8 | 350 | 0.821 | 24.8 |
| `lcc` | 2024 | 87.6% | 0.849 | 0.151 | 0.178 | 129.0 | 4.08 | 10 | 661 | 0.837 | 25.4 |
| `arxiv` | 2024-04-01 | 42.6% | 0.891 | 0.109 | 0.128 | 14.5 | 2.21 | 4 | 1665 | 0.856 | 20.3 |
| `bk` | 2026-10-03 | 48.1% | 0.902 | 0.098 | 0.116 | 75.0 | 3.15 | 8 | 788 | 0.841 | 20.9 |
| `jel` | 2026 | 47.9% | 0.903 | 0.097 | 0.115 | 28.9 | 2.27 | 4 | 972 | 0.806 | 19.4 |
| `msc` | 2020 | 39.2% | 0.950 | 0.050 | 0.059 | 114.4 | 2.00 | 4 | 380 | 0.800 | 17.8 |
<!-- /table:placed_corpus -->

Table 6. Placed by shelfmark on the book holdout (general schemes only).

<!-- table:placed_holdout -->
| scheme | version | coverage | cost_norm | revenue_norm | skill | eff. classes | mean depth | max depth | max fan-out | Gini | nav_log (a=1) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `lcc-full` | 2024 | 56.6% | 0.925 | 0.075 | 0.107 | 340.8 | 4.14 | 11 | 13 | 0.492 | 14.0 |
| `ilc2` | 2019-10-17 | 45.6% | 0.954 | 0.046 | 0.065 | 188.1 | 2.94 | 11 | 25 | 0.527 | 12.5 |
| `unesco-thesaurus` | 2026-08-31 | 15.8% | 0.989 | 0.011 | 0.016 | 93.8 | 1.65 | 7 | 8 | 0.439 | 10.8 |
| `lcc` | 2024 | 8.4% | 0.996 | 0.004 | 0.005 | 63.0 | 1.29 | 8 | 6 | 0.329 | 10.2 |
| `fdc` | 0.06 | 5.2% | 0.998 | 0.002 | 0.003 | 31.1 | 1.14 | 7 | 4 | 0.314 | 10.1 |
| `bk` | 2026-10-03 | 4.4% | 0.999 | 0.001 | 0.002 | 30.8 | 1.23 | 7 | 7 | 0.439 | 10.1 |
<!-- /table:placed_holdout -->

Table 7. Placement statistics for the classifier runs. "Flagged" placements
are below the 0.8 confidence target but kept. "Top matched term" is the
vocabulary term that sent items to the top class.

<!-- table:placement -->
| scheme | corpus | rows | placed | flagged (<0.8) | unplaced | classes used | top class | top share | top matched term |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | --- |
| `acm-ccs` | corpus | 10,000 | 7,445 | 6,854 | 2,555 | 405 | `10002944.10011123.10011674` | 11.8% | "performance" |
| `arxiv` | corpus | 10,000 | 4,260 | 3,325 | 5,740 | 72 | `cs.PF` | 39.1% | "performance" |
| `bk` | corpus | 10,000 | 4,806 | 3,980 | 5,194 | 213 | `01.00` | 16.4% | "general" |
| `eurovoc` | corpus | 10,000 | 9,737 | 9,582 | 263 | 802 | `C2228` | 3.6% | "paper" |
| `fdc` | corpus | 10,000 | 7,696 | 7,023 | 2,304 | 231 | `676` | 14.3% | "paper" |
| `ilc2` | corpus | 10,000 | 9,999 | 9,999 | 1 | 1,103 | `edqam` | 4.2% | "s" |
| `ipc` | corpus | 10,000 | 9,799 | 9,725 | 201 | 853 | `G09B23/00` | 8.1% | "models" |
| `jel` | corpus | 10,000 | 4,786 | 3,854 | 5,214 | 62 | `M53` | 20.3% | "training" |
| `lcc` | corpus | 10,000 | 8,765 | 8,320 | 1,235 | 379 | `QA299.6-433` | 7.5% | "analysis" |
| `lcc-full` | corpus | 10,000 | 9,999 | 9,999 | 1 | 2,421 | `QC243.3.E5` | 1.2% | "energy" |
| `msc` | corpus | 10,000 | 3,916 | 2,999 | 6,084 | 324 | `51-XX` | 9.7% | "geometry" |
| `openalex-topics` | corpus | 10,000 | 9,732 | 9,616 | 268 | 1,020 | `F21` | 5.7% | "energy" |
| `unesco-thesaurus` | corpus | 10,000 | 9,731 | 9,529 | 269 | 675 | `c155` | 5.5% | "time" |
| `bk` | holdout | 500 | 22 | 22 | 478 | 13 | `71.32` | 31.8% | "man" |
| `fdc` | holdout | 500 | 26 | 26 | 474 | 20 | `750` | 11.5% | "art" |
| `ilc2` | holdout | 500 | 228 | 226 | 272 | 151 | `edqam` | 11.0% | "s" |
| `lcc` | holdout | 500 | 42 | 42 | 458 | 31 | `QH501-531` | 7.1% | "life" |
| `lcc-full` | holdout | 500 | 283 | 283 | 217 | 227 | `K23` | 2.1% | "s" |
| `unesco-thesaurus` | holdout | 500 | 79 | 77 | 421 | 60 | `c155` | 5.1% | "time" |
<!-- /table:placement -->

On the corpus, skill under classifier placement runs from
<!-- v:placed_worst_skill -->0.059<!-- /v -->
(<!-- v:placed_worst -->msc<!-- /v -->) to
<!-- v:placed_best_skill -->0.423<!-- /v -->
(<!-- v:placed_best -->openalex-topics<!-- /v -->). Second is
<!-- v:placed_second -->unesco-thesaurus<!-- /v --> at
<!-- v:placed_second_skill -->0.329<!-- /v -->. Coverage explains much of the
order. Across the <!-- v:n_placed_corpus -->13<!-- /v --> schemes, the
Spearman correlation between coverage and skill is
<!-- v:rho_cov_skill -->0.76<!-- /v -->. Among the
<!-- v:n_highcov -->6<!-- /v --> schemes with at least 95% coverage, skill
still spreads from <!-- v:highcov_skill_min -->0.179<!-- /v --> to
<!-- v:highcov_skill_max -->0.423<!-- /v -->, so coverage does not
explain everything. Within that group, OpenAlex Topics scores best. It was
built from research literature, the same kind of text as the corpus
[@openalextopics]. EuroVoc, a policy and legal thesaurus, scores lowest.
That fits the expectation that a vocabulary matched to the corpus domain
organizes it better. But Table 7 shows the classifier produces many
low-confidence placements: in <!-- v:n_flagged_over90 -->9<!-- /v --> of the
<!-- v:n_placed_corpus -->13<!-- /v --> corpus runs, over 90% of placed items are flagged. The classifier's reach is itself part of the score
(Section 5.3).

The holdout results are near zero for every scheme except `lcc-full` and
`ilc2`. On titles of <!-- v:holdout_title_words -->3.5<!-- /v --> words, the lexical classifier finds a candidate
for only <!-- v:p_bk-placed-holdout_cov -->4.4%<!-- /v --> (BK) to
<!-- v:p_lcc-full-placed-holdout_cov -->56.6%<!-- /v --> (complete LCC)
of the books.

## 5.3 How large is the classifier effect?

Table 8 compares gold and classifier placements for the same scheme on
the same items and graph.

Table 8. Gold vs classifier placement on identical items.

<!-- table:bias -->
| scheme | corpus | gold coverage | placed coverage | gold skill | placed skill | placed − gold |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `arxiv` | corpus | 100.0% | 42.6% | 0.786 | 0.128 | -0.658 |
| `msc` | corpus | 13.2% | 39.2% | 0.018 | 0.059 | +0.041 |
| `jel` | corpus | 0.02% | 47.9% | 0.000 | 0.115 | +0.115 |
| `lcc` | holdout | 92.2% | 8.4% | 0.419 | 0.005 | -0.413 |
| `lcc-full` | holdout | 92.2% | 56.6% | 0.421 | 0.107 | -0.314 |
<!-- /table:bias -->

Where gold coverage is high (arXiv, LCC), the classifier loses most of the
skill. For arXiv, skill falls from
<!-- v:gold_arxiv_skill -->0.786<!-- /v --> to
<!-- v:placed_arxiv_corpus_skill -->0.128<!-- /v -->. The classifier
places only <!-- v:placed_arxiv_corpus_cov -->42.6%<!-- /v --> of the papers
and sends <!-- v:arxiv_placed_top_share -->39.1%<!-- /v --> of those to
<!-- v:arxiv_placed_top_class -->cs.PF<!-- /v --> (Performance) on the
word "performance" (Table 7). For the LCC outline on books, skill falls from
<!-- v:gold_lcc_skill -->0.419<!-- /v --> to
<!-- v:placed_lcc_holdout_skill -->0.005<!-- /v -->. Where gold coverage is
tiny (MSC, JEL), the classifier scores *higher* than the gold labels, only
because it places more items. Neither direction measures scheme quality.
In this run, the classifier, not the scheme, accounts for most of the
differences in classifier-placed skill. The ranking in Table 5 is therefore
a ranking of *scheme as reached by this classifier*. We do not present it
as a ranking of schemes.

## 5.4 Navigation

Table 9. Depth and navigation cost under uniform use (items in the root
menu included).

<!-- table:navigation -->
| scheme | labels | corpus | p50 depth | p95 depth | root fan-out | nav_log a=0 | a=1 | a=2 | nav_scan |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `arxiv` | gold | corpus | 4 | 4 | 8 | 15.2 | 18.3 | 21.5 | 291 |
| `jel` | gold | corpus | 1 | 1 | 10000 | 13.3 | 14.3 | 15.3 | 5001 |
| `msc` | gold | corpus | 1 | 4 | 8738 | 13.9 | 15.2 | 16.5 | 4372 |
| `lcc` | gold | holdout | 4 | 6 | 53 | 14.8 | 18.6 | 22.4 | 69 |
| `lcc-full` | gold | holdout | 5 | 7 | 53 | 15.4 | 19.7 | 24.1 | 56 |
| `acm-ccs` | placed | corpus | 4 | 6 | 2568 | 20.5 | 23.7 | 27.0 | 1476 |
| `arxiv` | placed | corpus | 1 | 4 | 5748 | 18.5 | 20.3 | 22.2 | 3231 |
| `bk` | placed | corpus | 1 | 6 | 5199 | 18.3 | 20.9 | 23.5 | 2736 |
| `eurovoc` | placed | corpus | 5 | 6 | 284 | 20.2 | 24.8 | 29.4 | 269 |
| `fdc` | placed | corpus | 4 | 5 | 2314 | 22.4 | 25.6 | 28.7 | 1413 |
| `ilc2` | placed | corpus | 5 | 7 | 68 | 18.9 | 23.4 | 27.9 | 136 |
| `ipc` | placed | corpus | 6 | 9 | 209 | 22.3 | 27.3 | 32.3 | 322 |
| `jel` | placed | corpus | 1 | 4 | 5231 | 17.4 | 19.4 | 21.4 | 2876 |
| `lcc` | placed | corpus | 4 | 6 | 1255 | 21.7 | 25.4 | 29.0 | 825 |
| `lcc-full` | placed | corpus | 6 | 8 | 22 | 17.5 | 22.9 | 28.2 | 52 |
| `msc` | placed | corpus | 1 | 4 | 6141 | 16.0 | 17.8 | 19.7 | 3128 |
| `openalex-topics` | placed | corpus | 5 | 5 | 272 | 22.4 | 26.9 | 31.3 | 311 |
| `unesco-thesaurus` | placed | corpus | 5 | 7 | 276 | 22.0 | 26.7 | 31.4 | 305 |
| `bk` | placed | holdout | 1 | 1 | 481 | 9.1 | 10.1 | 11.2 | 242 |
| `fdc` | placed | holdout | 1 | 2 | 482 | 9.0 | 10.1 | 11.2 | 243 |
| `ilc2` | placed | holdout | 1 | 7 | 300 | 10.3 | 12.5 | 14.6 | 155 |
| `lcc` | placed | holdout | 1 | 4 | 470 | 9.1 | 10.2 | 11.3 | 237 |
| `lcc-full` | placed | holdout | 5 | 9 | 236 | 11.3 | 14.0 | 16.7 | 125 |
| `unesco-thesaurus` | placed | holdout | 1 | 6 | 427 | 9.4 | 10.8 | 12.2 | 216 |
<!-- /table:navigation -->

The navigation models answer a different question from cost: how much
reading it takes to reach an item, not whether related items are near
each other. With unplaced items in the root menu, the two models diverge.
`nav_log` *drops* as coverage falls, because one large root menu costs only
$\log_2 b$. `nav_scan` rises steeply, because the root must be scanned
linearly. Among high-coverage runs, the complete LCC schedules have the
lowest scan cost (`nav_scan` <!-- v:lccfull_placed_navscan -->52<!-- /v --> on the corpus) and the most effective
classes. This is the deep, narrow-shelf trade-off: more levels to descend,
but short shelves at the bottom. The models' parameters are assumptions,
so Table 9 shows the trade-off and does not settle it. TODO(cite: menu
depth/breadth user studies; research/reports/2 lists Miller 1981 and
Larson–Czerwinski 1998 as unverified).

## 5.5 Methods note: the stopword fix

The first placed run (bench/results/2026-10-10, `profile:lexical@1`)
showed single-class captures. The classifier's vocabulary tier matched
complete class labels and terms, and some of them consist only of English
function words. ILC2 has a class labelled "to", IPC a C01G catchword "TO"
and the complete LCC a class `PL2813` labelled "To". Any abstract
containing "to" matched them. The fix (classifier `vocab-2`,
`profile:lexical@2`) drops normalized labels and terms whose tokens are all
stopwords. Labels with at least one content word still match in full. We
then reran only the placed part, on the same graphs, scheme files
(byte-identical, checked by SHA-256) and reference costs. The gold results
do not depend on the classifier and were not rerun. Every placed number in
this paper comes from the rerun (bench/results/2026-10-10-r2).

Table 10. Effect of the stopword fix (before: `lexical@1`, after:
`lexical@2`).

<!-- table:stopword -->
| scheme | corpus | top class before | top class after | coverage | cost_norm | skill | Δ skill |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| `acm-ccs` | corpus | 11.8% `10002944.10011123.10011674` | 11.8% `10002944.10011123.10011674` | 74.5% → 74.5% | 0.831 → 0.831 | 0.200 → 0.200 | +0.000 |
| `arxiv` | corpus | 39.1% `cs.PF` | 39.1% `cs.PF` | 42.6% → 42.6% | 0.891 → 0.891 | 0.128 → 0.128 | +0.000 |
| `bk` | corpus | 16.4% `01.00` | 16.4% `01.00` | 48.1% → 48.1% | 0.902 → 0.902 | 0.116 → 0.116 | +0.000 |
| `eurovoc` | corpus | 3.6% `C2228` | 3.6% `C2228` | 97.4% → 97.4% | 0.848 → 0.848 | 0.179 → 0.179 | +0.001 |
| `fdc` | corpus | 14.3% `676` | 14.3% `676` | 77.0% → 77.0% | 0.834 → 0.834 | 0.196 → 0.196 | +0.000 |
| `ilc2` | corpus | 73.7% `agy993` | 4.2% `edqam` | 100.0% → 100.0% | 0.825 → 0.819 | 0.206 → 0.214 | +0.007 |
| `ipc` | corpus | 60.1% `C01G` | 8.1% `G09B23/00` | 99.4% → 98.0% | 0.781 → 0.769 | 0.259 → 0.272 | +0.014 |
| `jel` | corpus | 18.7% `M53` | 20.3% `M53` | 51.7% → 47.9% | 0.902 → 0.903 | 0.116 → 0.115 | -0.001 |
| `lcc` | corpus | 7.5% `QA299.6-433` | 7.5% `QA299.6-433` | 88.4% → 87.6% | 0.848 → 0.849 | 0.179 → 0.178 | +0.000 |
| `lcc-full` | corpus | 22.1% `PL2813` | 1.2% `QC243.3.E5` | 100.0% → 100.0% | 0.800 → 0.823 | 0.236 → 0.209 | -0.027 |
| `msc` | corpus | 9.7% `51-XX` | 9.7% `51-XX` | 39.2% → 39.2% | 0.950 → 0.950 | 0.059 → 0.059 | +0.000 |
| `openalex-topics` | corpus | 5.6% `F21` | 5.7% `F21` | 97.4% → 97.3% | 0.641 → 0.641 | 0.423 → 0.423 | +0.000 |
| `unesco-thesaurus` | corpus | 17.8% `c493` | 5.5% `c155` | 98.9% → 97.3% | 0.732 → 0.721 | 0.316 → 0.329 | +0.013 |
| `bk` | holdout | 31.8% `71.32` | 31.8% `71.32` | 4.4% → 4.4% | 0.999 → 0.999 | 0.002 → 0.002 | +0.000 |
| `fdc` | holdout | 11.5% `750` | 11.5% `750` | 5.2% → 5.2% | 0.998 → 0.998 | 0.003 → 0.003 | +0.000 |
| `ilc2` | holdout | 13.7% `agqs99` | 11.0% `edqam` | 56.8% → 45.6% | 0.934 → 0.954 | 0.095 → 0.065 | -0.029 |
| `lcc` | holdout | 7.1% `QH501-531` | 7.1% `QH501-531` | 8.4% → 8.4% | 0.996 → 0.996 | 0.005 → 0.005 | +0.000 |
| `lcc-full` | holdout | 3.9% `BQ940` | 2.1% `K23` | 61.0% → 56.6% | 0.917 → 0.925 | 0.119 → 0.107 | -0.011 |
| `unesco-thesaurus` | holdout | 5.0% `c155` | 5.1% `c155` | 16.0% → 15.8% | 0.989 → 0.989 | 0.016 → 0.016 | +0.000 |
<!-- /table:stopword -->

The fix removed the captures. ILC2's largest class fell from
<!-- v:sw_ilc2_before -->73.7%<!-- /v --> to
<!-- v:sw_ilc2_after -->4.2%<!-- /v --> of placed papers, IPC's from
<!-- v:sw_ipc_before -->60.1%<!-- /v --> to
<!-- v:sw_ipc_after -->8.1%<!-- /v -->, complete LCC's from
<!-- v:sw_lcc-full_before -->22.1%<!-- /v --> to
<!-- v:sw_lcc-full_after -->1.2%<!-- /v --> and the UNESCO Thesaurus's from
<!-- v:sw_unesco-thesaurus_before -->17.8%<!-- /v --> to
<!-- v:sw_unesco-thesaurus_after -->5.5%<!-- /v -->. Skill moved much less.
On the corpus, <!-- v:sw_changed -->6<!-- /v --> of
<!-- v:sw_n -->13<!-- /v --> schemes changed skill by at least 0.0005, and
Kendall's $\tau_b$ between the skill values before and after is
<!-- v:sw_kendall -->0.95<!-- /v -->. Complete LCC *lost* skill
(<!-- v:sw_lcc-full_skill_before -->0.236<!-- /v --> →
<!-- v:sw_lcc-full_skill_after -->0.209<!-- /v -->) even though its
placements became plausible. A plausible explanation, not tested here: the
wrong placements had piled most papers
that share "to" into a single class. Within such a pile, the edges the
graph keeps still connect similar papers, and the pile is far smaller than
the whole corpus, so its edges cost less than they would under the root.
A metric can reward a degenerate placement more than a spread of honest but
noisy placements. This is a known property of a cost that charges
same-class pairs the class size: one large class still beats the root. It is
also why we always report largest-class share and placement statistics
next to skill.

# 6. Discussion

**What the measurement supports.** The method gives one scale for schemes of
different size, depth and domain. It costs little: no scoring run took more
than <!-- v:score_seconds_max -->6.9<!-- /v --> s, and the reference tree took
<!-- v:ref_seconds_corpus -->58<!-- /v --> s on the corpus graph. Its inputs are pinned by hash. On gold labels it separates a
well-fitted scheme (arXiv on arXiv) from a general scheme on books (LCC on
the holdout) and from schemes with too little gold to judge (MSC, JEL).

**What it does not yet support.** It does not support a ranking of universal
schemes. The general schemes that matter most for libraries (LCC, BK, FDC,
ILC2) have gold labels only for LCC, and only on a small title-only
holdout. Classifier placement, the only way to reach the other schemes,
moves skill by as much as the differences we would want to detect
(Table 8). The stopword fix shows the second risk. A classifier defect can
change placements drastically while skill barely moves, or moves the
wrong way.

**Toward a fair comparison.** The benchmark design already lays out the
controls needed (research/benchmark/DESIGN.md §2.5 and §3). These are a
cost–resolution curve against the reference cut at matched class counts; a
permutation null that keeps each scheme's shape and class-size histogram;
item bootstrap intervals; a label-intrinsic track that needs no works; and
a noise-injection curve that calibrates classifier error on LCC, where
both tracks exist. None of these was run for this draft. The largest gain
would come from a large gold set of books that carry both LCC and DDC.
The design estimates about 3.9M English Open Library records with both
labels (DESIGN.md §0). DDC could be scored by its digit tree alone, which
needs no captions.

**Interpretation of skill.** Skill compares a scheme with a tree fitted to
the same similarity graph. A scheme encodes many relations besides textual
similarity: discipline, form, audience, place, period. A low skill can mean
the scheme organizes by something the embedding does not see. Our score
measures agreement with one notion of relatedness. It does not measure
quality in every sense that matters to libraries.

# 7. Limitations

- **Classifier bias.** Every non-gold placement comes from shelfmark's own
  lexical classifier. It matches class labels and attached vocabulary, so
  a scheme with rich vocabulary is reached better than one with bare
  labels, independent of its structure. Most placements are below the
  classifier's own confidence target (Table 7). Table 8 shows the classifier
  effect is as large as the effects we want to measure. No other
  classifier (for example Annif or an LLM) was run for comparison.
- **Science-heavy corpus.** <!-- v:share_physical_math_cs -->86.4%<!-- /v -->
  of the corpus is computer science, mathematics or physics. Humanities,
  law, arts and general nonfiction appear only in the 500-record holdout,
  which is itself <!-- v:holdout_p_share -->76.4%<!-- /v --> class P. Domain
  schemes (arXiv, MSC, ACM CCS) are scored on their home ground and general
  schemes are not, so no cross-type comparison is fair.
- **Low gold coverage for MSC and JEL.** Only
  <!-- v:gold_msc_cov -->13.2%<!-- /v --> of papers carry a valid MSC code
  and <!-- v:gold_jel_cov -->0.02%<!-- /v --> a JEL code. Unplaced items
  hang under the root, so these gold scores measure coverage, not
  structure. A covered-subset score, restricted to labelled items and their
  edges, is not yet implemented.
- **ACM CCS gold missing.** The corpus field meant to carry ACM CCS labels
  holds the ACM classes submitted to arXiv, which are CCS 1998 codes. No
  value is a CCS 2012 concept path, so the gold run was skipped.
- **Scheme licensing.** Several schemes cannot be redistributed: ACM CCS
  (educational and research use only), JEL (no open licence), MSC
  (non-commercial, share-alike), EuroVoc (terms stated inconsistently) and
  ILC2 (conflicting licence statements) (Table 2). Their files are built
  locally from the publishers' downloads and verified by hash, but they are
  not published. Exact reproduction therefore depends on the upstream
  downloads staying available and unchanged. DDC, the most widely used
  universal scheme, is absent because OCLC holds its copyright
  [@deweyrights]. Even LCC's open status depends on jurisdiction. The
  Library of Congress calls its linked data a public-domain data set
  [@idloc], but its Classification Web notice says LC-originated
  classification data are copyrighted except within the USA
  [@classweblegal].
- **Titles-only holdout.** The book holdout has no descriptions. Its
  similarity graph is built from titles averaging
  <!-- v:holdout_title_words -->3.5<!-- /v --> words, and the
  classifier finds candidates for few of them. The holdout is also small
  (n = <!-- v:n_holdout -->500<!-- /v -->), and no confidence intervals are reported.
- **Method limits.** The kNN-sparsified cost has not been checked against
  the dense cost (DESIGN.md §1 proposes a rank-agreement check at
  n ≤ 5k). There is one embedding model and one value of k, with no
  sensitivity runs. Polyhierarchies are reduced to one parent per class.
  Only one placement per item is used, although 35.5% of a 200-work Open
  Library sample had more than one LCC value (DESIGN.md §0). The balance
  statistics count each item at every ancestor class (Section 3.5). The
  navigation models are unfitted.
- **Search for prior work.** The claim that no common-objective comparison
  of universal schemes exists rests on the searches recorded in
  research/reports/1–4. Those searches were not systematic.

# 8. Reproducibility statement

All inputs and outputs are in the repository or pinned by hash.

- **Corpus.** `bench/corpus/corpus.jsonl` and `holdout.jsonl`, with ID lists
  and SHA-256 hashes (bench/corpus/README.md). Metadata only, CC0 1.0. The
  harvest scripts and the deterministic Rust sampler are in
  `bench/corpus/builder/`.
- **Similarity graphs.** Published gzipped, with manifests, in
  `bench/results/2026-10-10/graphs/`. The manifests give the model
  revision `d13954661f83248295ba75c1ed411eef3b7b936e`, the `model.onnx`
  SHA-256, $\tau$, $k$ and the graph SHA-256 (corpus graph
  <!-- v:graph_corpus_sha -->759d3b8461a9384d<!-- /v -->…).
- **Harness.** `crates/bench`, at the commit recorded in `results.json`
  (<!-- v:harness_commit -->092494850adc<!-- /v -->). The classifier for
  the placed run is at <!-- v:classifier_commit -->f4ce24b78eda<!-- /v -->
  (`profile:lexical@2`). Both binaries' SHA-256 hashes are recorded too.
- **Schemes.** Bundled schemes are in the repository. Fetch-only schemes
  are rebuilt by `crates/bench/scripts/build_schemes.sh` from the
  publishers' downloads, checked against committed `sources.lock.json`
  files, and must match the `file_sha256` recorded in `results.json`.
- **Commands.** `bench/results/2026-10-10/REPRODUCE.md`,
  `bench/results/2026-10-10-r2/REPRODUCE.md` and
  `bench/results/2026-10-10-r3/REPRODUCE.md` list every command. The first
  run took about 45 minutes and the r2 rerun about 25 minutes, both on 8
  CPUs.
- **Determinism.** With the same graph and scheme files, gold scores
  reproduce exactly. Placed scores reproduce exactly with the same
  classifier build.
- **This paper.** `paper/paper1/scripts/make_tables.py` regenerates every
  table and marked number from the JSON. `--check` fails if `paper.md` is
  stale. It also checks that the r2 and r3 runs used the 2026-10-10 graphs
  and scheme files, and that the corpus files match their recorded hashes.

# Acknowledgements

TODO.

# References

<!-- Rendered from references.bib by the build tool (e.g. pandoc --citeproc). -->
