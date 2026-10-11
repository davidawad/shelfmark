---
title: "Scoring universal classification schemes with hierarchical-clustering objectives"
author: "David Awad"
status: "DRAFT. Not submitted, posted or shared."
bibliography: references.bib
---

<!--
DRAFT. Every number in the tables, and every number in the prose wrapped in a
`v:` marker, is generated from bench/results/2026-10-10 (gold),
bench/results/2026-10-10-r3 (gold rescored on the fixed corpus) and
bench/results/2026-10-10-r2 (shelfmark-placed), and, for Sections 5.6 to 5.11,
from docs/validity, bench/results/2026-10-10-covered, -embed, -semantic,
-learned, 2026-10-11-falsejoin, -stability and -book-stability, through
paper/common/extra_results.py, all run by scripts/make_tables.py.
Do not edit generated text by hand; rerun the script. Citation keys refer to
references.bib, the cited subset of docs/research/references.bib written
by docs/research/subset_bib.py.
-->

# Abstract

Library classification schemes are compared mostly by description: their
history, notation, coverage and licence. Quantitative comparisons exist, but
they measure retrieval effectiveness on a test collection
[@cleverdon1960; @carpenter1978], the scatter of a topic or class across
schemes [@oneill1987; @connaway1996], or coverage of a reference map of
knowledge [@zins2011]. We found no study that scores the hierarchy of
several universal or near-universal schemes on one structural objective
over the same set of works. We propose such a comparison, built on
objectives from the theory of hierarchical clustering, and we test how far
it can be trusted. A scheme, restricted to the classes that a corpus
actually uses, is a rooted tree over the works. We score that tree with
Dasgupta's cost [@dasgupta2016] and Moseley and Wang's revenue
[@moseley2023] on a fixed k-nearest-neighbour cosine graph of text
embeddings, and normalize revenue against an average-linkage tree. We call
this ratio skill. The corpus is <!-- v:n_corpus -->10,000<!-- /v --> arXiv
papers with CC0 metadata, a holdout of <!-- v:n_holdout -->500<!-- /v -->
Open Library editions with librarian-assigned LC call numbers, and a sample
of <!-- v:x_books_n -->3,000<!-- /v --> books that also carry Dewey numbers. We keep source-assigned
("gold") placements apart from placements made by one lexical classifier. On
gold labels, arXiv's own taxonomy reaches skill
<!-- v:gold_arxiv_skill -->0.786<!-- /v --> on arXiv papers and LCC reaches
<!-- v:gold_lcc_skill -->0.419<!-- /v --> on the book holdout. With the
classifier's placements, the same two schemes fall to
<!-- v:placed_arxiv_corpus_skill -->0.128<!-- /v --> and
<!-- v:placed_lcc_holdout_skill -->0.005<!-- /v -->. The findings are mostly
cautions. (1) Coverage drives most classifier-placed rankings: across
<!-- v:n_placed_corpus -->13<!-- /v --> schemes, skill ranges from
<!-- v:placed_worst_skill -->0.059<!-- /v -->
(<!-- v:placed_worst -->msc<!-- /v -->) to
<!-- v:placed_best_skill -->0.423<!-- /v -->
(<!-- v:placed_best -->openalex-topics<!-- /v -->), its Spearman correlation
with coverage is <!-- v:rho_cov_skill -->0.76<!-- /v -->, and coverage
explains <!-- v:x_cov_rho_sq -->59%<!-- /v --> of the rank variance of cost. (2) The metric
agrees only weakly with a blind LLM judge of librarian-style decisions
(Cohen's $\kappa$ = <!-- v:x_val_kappa -->0.27<!-- /v --> on <!-- v:x_val_n -->953<!-- /v --> pairs). A false-join term raises
$\kappa$ to <!-- v:x_val_kappa_fj -->0.48<!-- /v -->, but it does not raise agreement with citation or
subject-heading signals. (3) Rankings of the schemes on the papers are
stable across embedding model, k and graph threshold within one classifier
(minimum Spearman $\rho$ <!-- v:x_stab_cost_min_same -->0.90<!-- /v -->) and not across classifier tiers
(minimum <!-- v:x_stab_cost_min_diff -->0.04<!-- /v -->), and objectives disagree about FDC against LCC. (4) On
<!-- v:x_books_n -->3,000<!-- /v --> books, Dewey sections against LCC is a tie that depends on the
setting. (5) A learned scheme's lead over every human scheme survives
rescoring on two independent embedding models, is partly home advantage,
and on books does not survive the false-join correction. (6) Under a
coverage-floor rule, the LCC outline with LCSH vocabulary is the default
among open schemes in <!-- v:x_def_wins -->27<!-- /v --> of <!-- v:x_def_settings -->27<!-- /v --> book settings, partly through its vocabulary. The measurement is cheap and its inputs are pinned by hash. Its
weak points are the classifier and, for now, the single annotator behind
the validity check. We also report a classifier fix found by the benchmark:
the largest single-class capture in ILC2 fell from
<!-- v:sw_ilc2_before -->73.7%<!-- /v --> to
<!-- v:sw_ilc2_after -->4.2%<!-- /v --> of placed papers.

# 1. Introduction

A universal classification scheme gives every work one address in a single
tree. Library of Congress Classification (LCC), Dewey (DDC), UDC, the Basic
Classification (BK), the Integrative Levels Classification (ILC) and others
each claim to arrange all of knowledge. A practitioner choosing among them, or
a researcher designing a new one, needs to ask which arrangement keeps
related works close together, and at what navigation cost. The literature
answers this mostly qualitatively, by notation, facet structure, history and
institutional adoption [@nocetti1978; @zins2011; @oh2023; @wu2023].
Quantitative comparisons have measured retrieval performance
[@cleverdon1960; @keen1973; @carpenter1978] or the scatter of a subject
across schemes [@oneill1987; @connaway1996], not the structure of the
hierarchy itself (Section 2). A structured search of JASIST, Knowledge
Organization, Cataloging & Classification Quarterly, ISKO and SIG/CR
proceedings, JCDL and TPDL (research/reports/8-prior-work-search.md) found
no study that scores several universal schemes on a common hierarchical
objective over the same corpus, and no use of Dasgupta cost or Moseley–Wang
revenue on a human-built scheme. The search covered titles and abstracts,
not full texts (see Limitations), so this is absence of evidence, not
proof.

Three developments make the question more pressing.

1. **The main lookup service is gone.** OCLC Classify returned the most
   common DDC and LCC numbers for a work from WorldCat holdings. OCLC
   discontinued it on 31 January 2024 [@oclcclassify]. No free service
   replaced it. Its successor endpoint is in the WorldCat Metadata API,
   which is open only to subscribers.
2. **Automatic placement has not closed the gap.** Annif, the most mature open
   subject-indexing toolkit, is trained separately for each vocabulary
   [@annif]. On more than 230,000 Swedish union-catalogue records its
   best (ensemble) configuration reaches 66.82% accuracy at three-digit
   DDC [@libris2024ddc]. Recent work that uses large language models (LLMs)
   for LCC and DDC assignment reports weakness with more granular
   classes [@llmlcc2026] and a lack of specificity [@nasko2025], and
   prompted LLMs fall well short of fine-tuned smaller models on DDC
   classes for research data [@llm4ddc2025]. The Library of Congress's
   own experiments predicted subject and genre with under 50% accuracy,
   short of a 95% automation threshold [@lcecd2025]. Placement is getting
   cheaper, so the open question shifts towards *which scheme* to place
   into.
3. **Licences constrain open use.** DDC is copyrighted by OCLC
   [@deweyrights]. The full UDC is licensed [@udclicences]. Openly licensed
   universal schemes are shallow, such as BK [@bk] and FDC [@fdc], or their
   licence terms are unsettled, as for ILC2 [@ilc2]. Data-driven
   vocabularies such as OpenAlex Topics are open but cover research
   literature only [@openalextopics; @openalexpricing]. A quantitative comparison helps
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
bias is calibrated where both exist (Section 5.1 to 5.4). (iv) A methods
note on a classifier defect that the benchmark exposed, with its
before-and-after effect (Section 5.5). (v) A validity check of the cost
against a blind judge, a false-join correction, and non-LLM checks
(Sections 5.6 and 5.7). (vi) Stability tests: a coverage control, a second
classifier tier, and a sweep over embedding model, k and graph threshold on
the papers (Sections 5.8 and 5.9), and the same sweep for the claims of a
book-structure proposal on <!-- v:x_books_n -->3,000<!-- /v --> books with Dewey numbers (Sections
5.10 and 5.11).

# 2. Related work

**Comparing classification schemes.** The Cranfield project compared the
Universal Decimal Classification, an alphabetical subject catalogue, a
faceted classification and Uniterm coordinate indexing, applied to 18,000
aeronautics documents [@cleverdon1960; @jahoda1961]. Keen compared five
index languages built for a test collection on retrieval effectiveness
[@keen1973], and Carpenter et al. compared five patent classifications on
recall and precision for the same questions [@carpenter1978]. O'Neill et
al. measured how the classes of LCC and DDC disperse into each other on
MARC records [@oneill1987], and Connaway and Sievert compared the scatter
of one subject across LCC, DDC and the NLM Classification [@connaway1996].
Zins and Santos mapped the top two levels of LCC, DDC and UDC onto one
reference model of knowledge [@zins2011]. In bibliometrics, research
classifications are scored on citation-based criteria: algorithmic journal
maps against the ISI Subject Categories [@rafols2009], the Web of Science
and Scopus journal classifications against each other [@wang2016], and
taxonomies built from different citation relations against one gold
standard [@klavans2017]. These studies score schemes by retrieval, by the
scatter of a mapping or by citation coherence. None scores the tree itself.
For a single vocabulary, corpus-weighted pruning measures how much of a
subject tree a collection needs [@julien2013], checking functions detect
quality defects in SKOS vocabularies [@mader2012], and structural analysis
is one of the established ways to evaluate a thesaurus [@owens2004].

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
human use [@rosch1976]. User studies of menu hierarchies disagree on the
best shape. Speed and accuracy improved with breadth in one study
[@snowberry1983], and task time rose with depth for web links
[@zaphiris2000], but others found an optimum at moderate breadth
[@miller1981; @larson1998]. Landauer and Nachbar fit the time per menu
screen as $k + c \log b$ for $b$ choices [@landauer1985], the form of our
`nav_log` model (Section 3.5). For a two-level card file, the optimum number of records
per guide is proportional to the square root of the file size
[@lipetz1970]. We report simple, uniform-use navigation costs. We fit no
user model.

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
pinned ONNX revision [@reimers2019; @wang2020minilm; @minilmcard; @minilm], and vectors are L2-normalized. Raw cosine
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
the classifier's optional embedding tier (BGE-small, used only in the
semantic-tier comparison of Section 5.8).
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

## 3.7 Validity check and false-join term

Dasgupta cost charges a scheme for separating similar items. It charges
nothing for joining dissimilar items, except through the graph edges the
join happens to contain. To ask whether the cost tracks what a librarian
would call a good decision, we drew <!-- v:x_val_n -->953<!-- /v --> pairs from eight runs in three
strata (similar pairs the scheme splits at depth one or less, dissimilar
pairs it joins deep, and random placed pairs) and had one LLM judge label
each blind under a rubric fixed before judging (`docs/validity`). The
metric flags a pair when its cost contribution exceeds the run's mean
contribution per graph edge. Agreement is Cohen's $\kappa$ [@cohen1960],
with percentile bootstrap intervals that resample pairs within strata
[@efron1979]. The correction adds a false-join term. For a seeded sample of
non-edge pairs whose cosine is below $\tau$, weighted by dissimilarity
$d = \tau - \cos$, the specificity form is
$\mathrm{FJs} = \sum d \ln(n/N(\mathrm{lca})) / (\ln n \sum d)$, so
that deep joins of dissimilar items cost more than shallow ones. The
corrected objective is $J' = C + \beta\,\mathrm{FJs} + \lambda N$ with
$\beta = 1$, $\lambda = 0.002$ and $N$ the `nav_log` cost in bits. An
outside-mass form is also computed. Paired differences between runs use an
item bootstrap on the same draws, and multiple comparisons are
Holm-adjusted [@holm1979].

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
| econ | 185 | 1.9% |
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
the array. Open Library asks contributors to share under CC0 1.0
[@openlibraryfaq]. Both
files are pinned by SHA-256 and by ID lists.

**Book sample with Dewey numbers.** A third file holds <!-- v:x_books_n -->3,000<!-- /v --> Open
Library editions that each carry an LC call number, a Dewey number and
subjects (<!-- v:x_books_desc -->320<!-- /v --> also have a description). The 21 LCC main classes
have equal quotas, and no edition shares a work with the 500-book holdout.
The graph text is the description followed by up to 30 subject headings, so
it is mostly Library of Congress subject headings
(`bench/corpus/holdout-books.jsonl`). The sample's $\tau$ is <!-- v:x_books_tau -->0.157<!-- /v -->.
Dewey numbers are used as notation only: the tree is the digit prefixes,
and no Dewey caption, schedule or table is read (Section 7). A work-level
lookup finds a Dewey number for many more of the 500-book holdout's works
than its edition records carry (research/reports/9, Section 3).

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
publications [@freelcc; @idloc; @usc105]. Sources for the others: IPC
[@ipc; @wipoterms], ACM CCS [@acmccs], EuroVoc [@eurovoc], UNESCO Thesaurus
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
| `lcc` | 2024 | 87.7% | 0.849 | 0.151 | 0.178 | 129.0 | 4.08 | 10 | 661 | 0.837 | 25.4 |
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
(Sections 5.3 and 5.8).

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
so Table 9 shows the trade-off and does not settle it. The menu studies
cited in Section 2 disagree on the best depth and breadth
[@miller1981; @snowberry1983; @larson1998], so they do not fix the
parameters either.

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
| `lcc` | corpus | 7.5% `QA299.6-433` | 7.5% `QA299.6-433` | 88.4% → 87.7% | 0.848 → 0.849 | 0.179 → 0.178 | +0.000 |
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

## 5.6 Does the cost agree with a judge?

Table 11 is the pooled result of the validity check (Section 3.7). The
judge labels a pair's scheme decision correct or incorrect. Over
<!-- v:x_val_n -->953<!-- /v --> pairs the metric's flag agrees with the judge at $\kappa$ =
<!-- v:x_val_kappa -->0.27<!-- /v --> [<!-- v:x_val_kappa_lo -->0.22<!-- /v -->, <!-- v:x_val_kappa_hi -->0.32<!-- /v -->], with agreement
<!-- v:x_val_agree -->0.63<!-- /v --> and an AUC of <!-- v:x_val_auc -->0.65<!-- /v --> for the raw contribution. That is
weak agreement. The judge is stable against itself ($\kappa$ =
<!-- v:x_val_self_kappa -->0.92<!-- /v --> on a blind second pass of <!-- v:x_val_self_n -->238<!-- /v --> pairs). Strata show where
the gap is. Among similar pairs the scheme splits (stratum A), the judge
calls <!-- v:x_val_A_incorrect -->0.70<!-- /v --> incorrect and the metric flags <!-- v:x_val_A_flagged -->0.84<!-- /v -->. Among
dissimilar pairs the scheme joins deep (stratum B), the judge also calls
<!-- v:x_val_B_incorrect -->0.70<!-- /v --> incorrect and the metric flags <!-- v:x_val_B_flagged -->0.00<!-- /v -->: those pairs have zero
weight, so the cost is blind to them by construction. Shares are over a
stratified sample, one third per stratum, not over all pairs. Across the
eight runs, the Spearman correlation between a run's `cost_norm` and the
judge's error rate on random pairs is <!-- v:x_val_rho_c -->0.23<!-- /v -->, and over all strata
<!-- v:x_val_rho_all -->0.42<!-- /v -->, from eight points. This is descriptive only. The pair sample
is small per run (<!-- v:x_val_run_n -->120<!-- /v --> pairs), so per-run intervals are wide.

Table 11. Agreement of the metric's flag with the judge, pooled and with
the false-join variants (same 953 pairs and same labels).

<!-- table:validity_variants -->
| variant | flagged | kappa [95% CI] | AUC [95% CI] |
| --- | ---: | ---: | ---: |
| Dasgupta cost C | 0.28 | 0.27 [0.22, 0.32] | 0.65 [0.62, 0.67] |
| FJs alone | 0.34 | 0.21 [0.15, 0.26] | 0.61 [0.59, 0.64] |
| C + 0.5 FJs | 0.60 | 0.49 [0.44, 0.55] | 0.80 [0.77, 0.83] |
| C + FJs | 0.62 | 0.48 [0.43, 0.53] | 0.78 [0.75, 0.81] |
| C + 4 FJs | 0.65 | 0.45 [0.40, 0.51] | 0.76 [0.73, 0.79] |
| C + FJ (outside mass) | 0.61 | 0.47 [0.41, 0.52] | 0.78 [0.75, 0.81] |
<!-- /table:validity_variants -->

## 5.7 A cost that sees false joins

Adding the specificity false-join term to the cost raises agreement with
the judge to $\kappa$ = <!-- v:x_val_kappa_fj -->0.48<!-- /v --> [<!-- v:x_val_kappa_fj_lo -->0.43<!-- /v -->, <!-- v:x_val_kappa_fj_hi -->0.53<!-- /v -->], a paired
change of <!-- v:x_val_dkappa_fj -->+0.21<!-- /v --> [<!-- v:x_val_dkappa_fj_lo -->+0.15<!-- /v -->, <!-- v:x_val_dkappa_fj_hi -->+0.27<!-- /v -->], and the AUC from <!-- v:x_val_auc_base -->0.65<!-- /v --> to
<!-- v:x_val_auc_fj -->0.78<!-- /v --> (Table 11). The false-join term alone does not do it
($\kappa$ = <!-- v:x_val_kappa_fjs_alone -->0.21<!-- /v -->); the gain comes from the sum. Weights from 0.25 to 8
give $\kappa$ between <!-- v:x_val_kappa_fjs_min -->0.45<!-- /v --> and
<!-- v:x_val_kappa_fjs_max -->0.49<!-- /v -->, so the choice of $\beta$ is not
critical in this range. Table 12 breaks the change down by
run. The correction helps the placed runs that join dissimilar items: every
placed run except the learned scheme gains, with intervals that exclude
zero. It does nothing for arXiv's own taxonomy or for the learned scheme,
where the judge rarely finds a false join.

Table 12. Judge agreement before and after adding the false-join term, per
run.

<!-- table:validity_runs -->
| run | kappa, C [95% CI] | kappa, C + FJs [95% CI] | change in kappa |
| --- | ---: | ---: | ---: |
| arxiv-gold | 0.24 [0.05, 0.43] | 0.11 [0.05, 0.18] | -0.13 [-0.28, +0.04] |
| msc-gold | 0.20 [0.01, 0.38] | 0.33 [0.21, 0.46] | +0.13 [-0.07, +0.33] |
| fdc-placed | 0.29 [0.21, 0.38] | 0.63 [0.48, 0.78] | +0.34 [+0.19, +0.47] |
| ilc2-placed | 0.25 [0.18, 0.33] | 0.66 [0.50, 0.79] | +0.40 [+0.28, +0.50] |
| lcc-full-placed | 0.25 [0.19, 0.32] | 0.68 [0.54, 0.81] | +0.43 [+0.33, +0.52] |
| openalex-topics-placed | 0.26 [0.16, 0.35] | 0.66 [0.52, 0.79] | +0.40 [+0.28, +0.50] |
| unesco-thesaurus-placed | 0.24 [0.17, 0.32] | 0.54 [0.38, 0.69] | +0.30 [+0.16, +0.43] |
| learned-placed | 0.33 [0.16, 0.50] | 0.30 [0.18, 0.43] | -0.03 [-0.20, +0.14] |
<!-- /table:validity_runs -->

Two checks that do not use an LLM temper this. The first signal is a
shared Library of Congress subject heading between books. The second is
bibliographic coupling or co-citation between arXiv papers, from OpenAlex
reference lists (research/reports/9, Section 4.6). Table 13 gives the
agreement of the pair-level flag with each signal, and the separation each
signal finds between an FDC backbone and an LCC backbone. The false-join
term does *not* raise pair-level agreement with any of the three
(coupling <!-- v:x_ext_coupling_k0 -->0.13<!-- /v --> to <!-- v:x_ext_coupling_k1 -->0.06<!-- /v -->, co-citation <!-- v:x_ext_cocitation_k0 -->0.23<!-- /v --> to <!-- v:x_ext_cocitation_k1 -->0.19<!-- /v -->, subject headings
<!-- v:x_ext_lcsh_k0 -->0.24<!-- /v --> to <!-- v:x_ext_lcsh_k1 -->0.21<!-- /v -->). At scheme level, rank agreement is unchanged. So the
corrected metric is better supported than the original, by one annotator
model, and it is not better supported by independent signals. Citation
and subject-heading signals have their own limits: most 2025 to 2026
preprints in the corpus have no OpenAlex reference list, and LC assigns
subject headings and call numbers in one record, so that signal is not
independent of LCC.

Table 13. Pair-level agreement ($\kappa$) and scheme-level rank
agreement ($\rho$) of the cost and the corrected cost with non-LLM signals.

<!-- table:external -->
| external signal | kappa, C | kappa, C + FJs | scheme-level rho, C | scheme-level rho, C + FJs | FDC-backbone minus LCC-backbone separation [95% CI] |
| --- | ---: | ---: | ---: | ---: | ---: |
| shared LCSH heading (books) | 0.24 | 0.21 | 0.88 | 0.89 | ddc-section - lcc-gold: -0.002 [-0.011, +0.007] |
| bibliographic coupling (papers) | 0.13 | 0.06 | 0.90 | 0.90 | fdc-arxiv-nested - lcc-arxiv-nested: +0.004 [-0.001, +0.010] |
| co-citation (papers) | 0.23 | 0.19 | 0.87 | 0.87 | fdc-arxiv-nested - lcc-arxiv-nested: +0.001 [-0.026, +0.025] |
<!-- /table:external -->

Part of the judge's verdict is also about the schemes. The judge's
together-or-apart call agrees with whether the pair is an edge of the graph
at $\kappa$ = <!-- v:x_val_together_kappa -->0.62<!-- /v -->, and disagrees with whether the scheme put the pair
together ($\kappa$ = <!-- v:x_val_scheme_together_kappa -->-0.31<!-- /v -->). The judge finds, on its sample, a false-join share
of <!-- v:x_val_false_join -->0.24<!-- /v --> and a false-split share of <!-- v:x_val_false_split -->0.27<!-- /v --> of all sampled pairs, shares
that depend on how the strata were drawn.

## 5.8 Coverage control and the classifier tier

A gold run's unlabelled items hang under the root and are charged the
maximum, so coverage enters the cost directly. The covered-subset analysis
rescored the placed runs on each run's own covered items. Coverage explains
<!-- v:x_cov_rho_sq -->59%<!-- /v --> of the rank variance of Dasgupta cost over the
<!-- v:x_cov_runs -->14<!-- /v --> placed runs on the papers ($\rho$ = <!-- v:x_cov_rho -->-0.77<!-- /v -->), and the ranking by cost on
all items and on each run's own covered items barely agrees (Kendall
$\tau$ = <!-- v:x_cov_tau -->0.10<!-- /v -->). Low-coverage schemes move from the bottom to the middle once
their unplaced papers are dropped. The sparse graph is not the cause: on random subsets of 1,000 to 5,000
papers, the kNN-sparse cost and a dense cost rank the placed runs with
Spearman $\rho \ge$ <!-- v:x_dense_min_rho -->0.92<!-- /v -->.

A second classifier tier tests the same point from the other side. The
shortlist profile adds a BGE-small embedding shortlist after the lexical
tiers and files every item, so coverage is <!-- v:x_sem_min_cov -->1.000<!-- /v --> for all
<!-- v:x_sem_n -->13<!-- /v --> schemes. The rerank profile is unavailable in this build and
reproduces the shortlist placements. Table 14 compares the tiers on the
same graph and schemes. Skill rises for <!-- v:x_sem_up -->12<!-- /v --> of <!-- v:x_sem_n -->13<!-- /v --> schemes, by much more where
the lexical tier placed few papers. arXiv's own taxonomy moves from
<!-- v:x_sem_arxiv_lex -->0.128<!-- /v --> to <!-- v:x_sem_arxiv_sh -->0.502<!-- /v --> and now leads, with openalex-topics second
(<!-- v:x_sem_second_skill -->0.465<!-- /v -->), and the Spearman correlation between the two tiers' skill values
is <!-- v:x_sem_rho -->0.14<!-- /v -->. The ranking of schemes under a classifier is therefore a
property of the classifier-scheme pair. The numbers are read from the
committed JSON of that directory. The tier that places by embedding uses a
different model from the graph's.

Table 14. Lexical and shortlist placement on the papers (`profile:lexical@2`
and `profile:shortlist@2`; bench/results/2026-10-10-semantic).

<!-- table:semantic_tier -->
| scheme | coverage lexical | coverage shortlist | skill lexical | skill shortlist | change in skill |
| --- | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | 0.745 | 1.000 | 0.200 | 0.278 | +0.078 |
| arxiv | 0.426 | 1.000 | 0.128 | 0.502 | +0.373 |
| bk | 0.481 | 1.000 | 0.116 | 0.360 | +0.244 |
| eurovoc | 0.974 | 1.000 | 0.179 | 0.175 | -0.004 |
| fdc | 0.770 | 1.000 | 0.196 | 0.258 | +0.062 |
| ilc2 | 1.000 | 1.000 | 0.214 | 0.244 | +0.031 |
| ipc | 0.980 | 1.000 | 0.272 | 0.296 | +0.023 |
| jel | 0.479 | 1.000 | 0.115 | 0.303 | +0.189 |
| lcc | 0.876 | 1.000 | 0.178 | 0.212 | +0.033 |
| lcc-full | 1.000 | 1.000 | 0.209 | 0.217 | +0.008 |
| msc | 0.392 | 1.000 | 0.059 | 0.230 | +0.171 |
| openalex-topics | 0.973 | 1.000 | 0.423 | 0.465 | +0.042 |
| unesco-thesaurus | 0.973 | 1.000 | 0.329 | 0.338 | +0.009 |
<!-- /table:semantic_tier -->

## 5.9 Rank stability on the papers

The stability sweep rescored the 13 schemes on the papers under
<!-- v:x_stab_settings -->54<!-- /v --> controlled settings: three embedding models, k of 10, 20 and 50,
three graph thresholds, and the lexical and shortlist tiers. A ranking
claim is called robust when the minimum Spearman $\rho$ over every pair of
settings is at least <!-- v:x_stab_threshold -->0.8<!-- /v --> (the sweep's own rule). Table 15 gives the minimum
and median $\rho$ between settings that use the same classifier tier and
between settings that do not.

Table 15. Rank agreement of the 13 schemes between controlled settings, by
objective, and the number of settings in which FDC ranks above LCC.

<!-- table:stability -->
| objective | min rho, same tier | median rho, same tier | min rho, across tiers | median rho, across tiers | FDC above LCC (settings) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Dasgupta cost | 0.896 | 0.978 | 0.038 | 0.154 | 0 of 54 |
| Moseley-Wang revenue | 0.896 | 0.978 | 0.038 | 0.154 | 0 of 54 |
| skill | 0.896 | 0.978 | 0.038 | 0.154 | 0 of 54 |
| triplet accuracy | 0.808 | 0.984 | -0.280 | -0.104 | 27 of 54 |
| shelf-order locality | 0.780 | 0.962 | 0.549 | 0.736 | 0 of 54 |
<!-- /table:stability -->

Within a tier, rankings are stable: cost, revenue and skill have a minimum
$\rho$ of <!-- v:x_stab_cost_min_same -->0.90<!-- /v --> over the embedding, k and threshold sweep. Across tiers they
are not: the minimum is <!-- v:x_stab_cost_min_diff -->0.04<!-- /v --> and the median <!-- v:x_stab_cost_med_diff -->0.15<!-- /v -->. The leader under
cost is <!-- v:x_stab_lead_lexical -->openalex-topics<!-- /v --> under the lexical tier and <!-- v:x_stab_lead_shortlist -->arxiv<!-- /v --> under the
shortlist tier. Objectives also disagree. FDC ranks above LCC by cost,
revenue, skill and locality in <!-- v:x_stab_cost_fdc_wins -->0<!-- /v --> settings. By triplet accuracy it
ranks above LCC in <!-- v:x_stab_triplet_fdc_wins -->27<!-- /v --> of <!-- v:x_stab_settings -->54<!-- /v -->, and the triplet ranking can be
reversed between settings (minimum $\rho$ <!-- v:x_stab_triplet_min_diff -->-0.28<!-- /v -->). A claim that one
scheme beats another on these papers therefore depends on the classifier
and on the objective. The graph settings matter much less.

## 5.10 Books with Dewey numbers: Dewey against LCC

On the <!-- v:x_books_n -->3,000<!-- /v --> books every item has both a Dewey number and an LC call
number from librarians, so the Dewey and LCC runs are gold on the same
items and coverage is 1.000 for both. Table 16 gives the runs on the MiniLM
$k = 20$ graph. Table 17 gives the comparisons that a book-structure
proposal drew from them, rechecked in each of <!-- v:x_dl_settings -->27<!-- /v --> settings (three
embedding models, three values of k and three graph thresholds, with <!-- v:x_bs_resamples -->1,000<!-- /v -->
paired item-bootstrap draws each).

Table 16. Books, MiniLM, $k = 20$, published graph. $C$ is `cost_norm`,
FJs enters at $\beta = 1$, $J'$ adds $0.002 N$.

<!-- table:book_main -->
| run | structure | coverage | C | C + FJs | nav bits | J' | skill |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ddc-gold` | Dewey, librarian number, full depth | 1.000 | 0.493 | 0.520 | 19.3 | 0.558 | 0.658 |
| `ddc-section` | Dewey, cut to section (3 digits) | 1.000 | 0.494 | 0.521 | 17.1 | 0.555 | 0.657 |
| `fdc-d3` | FDC from Dewey, depth 3 | 1.000 | 0.496 | 0.523 | 17.7 | 0.559 | 0.654 |
| `fdc-d2` | FDC from Dewey, depth 2 | 1.000 | 0.502 | 0.528 | 16.1 | 0.561 | 0.647 |
| `lcc-gold` | LCC, librarian call number (outline) | 1.000 | 0.523 | 0.531 | 16.2 | 0.563 | 0.619 |
| `lcc-full-gold` | LCC, librarian call number (complete schedules) | 1.000 | 0.522 | 0.530 | 16.8 | 0.563 | 0.620 |
| `learned-own` | learned scheme, own insertion | 1.000 | 0.339 | 0.391 | 29.7 | 0.450 | 0.858 |
| `fdc-placed` | FDC, lexical classifier | 0.839 | 0.654 | 0.680 | 23.4 | 0.727 | 0.450 |
| `lcc-lcsh-placed` | LCC outline with LCSH vocabulary, lexical classifier | 0.985 | 0.674 | 0.689 | 18.6 | 0.726 | 0.423 |
| `lcc-placed` | LCC outline, lexical classifier | 0.915 | 0.733 | 0.750 | 20.6 | 0.791 | 0.346 |
| `unesco-thesaurus-placed` | UNESCO Thesaurus, lexical classifier | 0.925 | 0.629 | 0.656 | 22.2 | 0.700 | 0.482 |
<!-- /table:book_main -->

Table 17. Pair claims across <!-- v:x_dl_settings -->27<!-- /v --> settings. "Claimed" is the sign the
proposal reported. "Sign matches" counts settings where the difference has
that sign. "Significant" counts settings whose 95% interval excludes zero,
for and against the claim. A tie claim fails when any interval excludes
zero.

<!-- table:book_claims -->
| comparison (a vs b) | objective | claimed | sign matches | significant for / against | range of a - b | verdict |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Dewey sections vs LCC, librarian numbers | C | a < b | 13 of 27 | 10 / 9 | [-0.0753, +0.0750] | not stable |
| Dewey sections vs LCC, librarian numbers | J' | tie | 8 of 27 | - / 19 | [-0.0564, +0.0948] | not stable |
| Dewey sections vs FDC depth 3 | C | a < b | 27 of 27 | 27 / 0 | [-0.0044, -0.0010] | stable |
| FDC depth 3 vs depth 2 | J' | a < b | 27 of 27 | 24 / 0 | [-0.0085, -0.0001] | stable |
| FDC depth 3 vs full FDC number | J' | a < b | 15 of 27 | 9 / 1 | [-0.0005, +0.0004] | not stable |
| Full Dewey number vs section | J' | a > b | 27 of 27 | 27 / 0 | [+0.0013, +0.0036] | stable |
| Learned scheme vs Dewey sections | J' | a < b | 19 of 27 | 13 / 6 | [-0.1520, +0.0917] | not stable |
| Placed: FDC vs LCC outline | J' | a < b | 27 of 27 | 24 / 0 | [-0.0897, -0.0237] | stable |
| Placed: FDC vs LCC outline + LCSH | J' | a < b | 9 of 27 | 5 / 8 | [-0.0434, +0.0729] | not stable |
<!-- /table:book_claims -->

Dewey sections beat LCC on cost on the published graph for all three
embedding models (<!-- v:x_dl_c_minilm -->-0.0292<!-- /v --> [<!-- v:x_dl_c_minilm_lo -->-0.0438<!-- /v -->, <!-- v:x_dl_c_minilm_hi -->-0.0141<!-- /v -->] on MiniLM,
<!-- v:x_dl_c_bgesmall -->-0.0411<!-- /v --> on bge-small, <!-- v:x_dl_c_nomic -->-0.0239<!-- /v --> on nomic, at $k = 20$). Across the
<!-- v:x_dl_settings -->27<!-- /v --> settings the sign matches in <!-- v:x_dl_c_match -->13<!-- /v -->, with <!-- v:x_dl_c_for -->10<!-- /v --> significant for and
<!-- v:x_dl_c_against -->9<!-- /v --> significant against, and the difference ranges from <!-- v:x_dl_c_min -->-0.0753<!-- /v --> to
<!-- v:x_dl_c_max -->+0.0750<!-- /v -->. Under the corrected objective the claim becomes a tie, and the tie
holds in <!-- v:x_dl_jp_match -->8<!-- /v --> settings with <!-- v:x_dl_jp_against -->19<!-- /v --> significant departures. The
worst-case regret that earlier favoured Dewey sections (<!-- v:x_regret_ddc9 -->0.060<!-- /v --> against
<!-- v:x_regret_lcc9 -->0.067<!-- /v --> for LCC over the nine published graphs) reverses over all
<!-- v:x_dl_settings -->27<!-- /v --> settings (<!-- v:x_regret_ddc27 -->0.116<!-- /v --> against <!-- v:x_regret_lcc27 -->0.067<!-- /v -->). The non-LLM signals do not separate them
either: shared subject headings <!-- v:x_ext_lcsh_diff -->-0.002<!-- /v --> [<!-- v:x_ext_lcsh_lo -->-0.011<!-- /v -->, <!-- v:x_ext_lcsh_hi -->+0.007<!-- /v -->], and for papers shelved
under an FDC or an LCC backbone, coupling <!-- v:x_ext_coupling_diff -->+0.004<!-- /v -->
[<!-- v:x_ext_coupling_lo -->-0.001<!-- /v -->, <!-- v:x_ext_coupling_hi -->+0.010<!-- /v -->] and co-citation <!-- v:x_ext_cocitation_diff -->+0.001<!-- /v -->
[<!-- v:x_ext_cocitation_lo -->-0.026<!-- /v -->, <!-- v:x_ext_cocitation_hi -->+0.025<!-- /v -->]. We read this as a
tie that depends on the setting: Dewey and LCC are not separated by these
metrics on these books. The sample is stratified equally over LCC's main
classes, which favours LCC's false-join term, and its text is mostly LC
subject headings.

Smaller comparisons are stable in every setting. A three-digit Dewey
section beats FDC depth 3 by less than
<!-- v:x_sec_fdc_max -->0.0044<!-- /v --> of cost (a caption-versus-number
difference). Three levels beat two. The full Dewey number gains cost and
loses on $J'$. Through the same classifier, FDC beats the LCC outline
without subject-heading vocabulary in all <!-- v:x_dl_settings -->27<!-- /v --> settings. It does not
beat the bundled LCC outline with LCSH vocabulary, which places
<!-- v:x_floor_cov_lcclcsh -->0.99<!-- /v --> of the books against <!-- v:x_floor_cov_fdc -->0.84<!-- /v --> for FDC: that comparison
turns on vocabulary and coverage, not on the scheme. On the 500-book
title-only holdout no placed run reaches the coverage floor of
<!-- v:x_floor_floor -->0.80<!-- /v -->, and the best reaches <!-- v:x_hold_best_cov -->0.57<!-- /v -->.

## 5.11 The learned scheme and home advantage

A scheme fitted to the objective by order-preserving dynamic programming
and local search (the companion draft in `paper/paper2`) scores far better
than every human scheme on its own corpus: in-sample skill
<!-- v:x_l_insample_skill -->0.976<!-- /v --> on the papers, and
<!-- v:x_l_heldout_skill -->0.922<!-- /v --> on a held-out 20% split. Through the
lexical classifier it falls to <!-- v:x_l_placed_skill -->0.732<!-- /v -->. On books, its cost is lower than Dewey sections' on every graph, but the
margin is largest on the graph it was fitted on (Table 18). Refits on the
other two models score best on their own graph (Table 19): on the
held-out split, a fit scored on its own model's graph averages
<!-- v:x_cf_diag_mean -->0.926<!-- /v --> skill against <!-- v:x_cf_off_mean -->0.904<!-- /v --> elsewhere, and is best on its own graph for
<!-- v:x_cf_diag_wins -->3<!-- /v --> of 3 models. Part of the learned scheme's lead is therefore home
advantage. Across the <!-- v:x_dl_settings -->27<!-- /v --> settings, the claim that it beats Dewey sections is
not stable (sign matches in <!-- v:x_learn_match -->19<!-- /v --> of
<!-- v:x_dl_settings -->27<!-- /v --> under $J'$, with
<!-- v:x_learn_against -->6<!-- /v --> significant reversals; Table 17). It also needs the embedding model to file a new item
and has an opaque notation. We use it as a reference for how low a fitted
tree can go, not as a candidate scheme.

Two further checks bound the claim. Rescored on graphs from the two other
embedding models at k = 10, 20 and 50, it still ranks first by skill in all
<!-- v:x_edge_conditions -->9<!-- /v --> conditions for the held-out split, the in-sample
fit, the inserted book holdout and the placed run, with paired intervals of
the margin above zero (tightest: <!-- v:x_edge_tight -->0.155<!-- /v --> [<!-- v:x_edge_tight_lo -->0.122<!-- /v -->,
<!-- v:x_edge_tight_hi -->0.189<!-- /v -->] on <!-- v:x_edge_tight_cond -->bge-small-k10<!-- /v -->). Under the corrected
objective, against the best non-learned run of each universe on the
false-join rescoring's graphs, it leads on the papers in
<!-- v:x_lfj_corpus_jp_ahead -->3<!-- /v --> of <!-- v:x_lfj_corpus_n -->3<!-- /v --> graphs and on the
title-only holdout in <!-- v:x_lfj_holdout_jp_ahead -->3<!-- /v --> of <!-- v:x_lfj_holdout_n -->3<!-- /v -->.
On the <!-- v:x_books_n -->3,000<!-- /v --> books it leads in <!-- v:x_lfj_books_jp_ahead -->6<!-- /v --> of
<!-- v:x_lfj_books_n -->9<!-- /v -->, ties in <!-- v:x_lfj_books_jp_tie -->2<!-- /v --> and is behind in
<!-- v:x_lfj_books_jp_behind -->1<!-- /v -->; under the outside-mass form it is behind in
<!-- v:x_lfj_books_cfj_behind -->6<!-- /v --> of <!-- v:x_lfj_books_n -->9<!-- /v -->. Its edge survives a
change of embedding and does not survive the false-join correction on
books. Paper 2, Section 6.6, gives the tables.

Table 18. Learned scheme against Dewey sections on books, by graph (all
$k = 20$; the scheme was fitted on MiniLM).

<!-- table:learned_margin -->
| graph (books, k = 20) | C learned | C Dewey sections | C difference | J' difference |
| --- | ---: | ---: | ---: | ---: |
| minilm (fit graph) | 0.339 | 0.494 | -0.155 | -0.105 |
| bge-small | 0.376 | 0.483 | -0.108 | -0.052 |
| nomic | 0.361 | 0.473 | -0.112 | -0.058 |
<!-- /table:learned_margin -->

Table 19. Held-out skill of each learned fit on each model's $k = 20$ papers
graph.

<!-- table:cross_fit -->
| learned fit | scored on minilm | scored on bge-small | scored on nomic |
| --- | ---: | ---: | ---: |
| fitted on minilm | 0.922 | 0.901 | 0.891 |
| fitted on bge-small | 0.913 | 0.938 | 0.909 |
| fitted on nomic | 0.899 | 0.912 | 0.918 |
<!-- /table:cross_fit -->

## 5.12 The default scheme under a stated rule

The tool needs one default scheme for a general library of mostly books,
filed by its classifier alone. The decision record
(`docs/decisions/default-scheme.md`) states the rule: among
placed runs whose coverage is at least a floor, take the highest skill;
every run whose paired difference to it has a 95% interval covering zero is
tied; the tied run with the lowest navigation cost wins. The floor is
<!-- v:x_def_floor -->0.90<!-- /v -->, so that at most one book in ten waits for a person. The
rule was applied in each of the <!-- v:x_def_settings -->27<!-- /v --> book settings of
Section 5.10. Table 20 gives the winners by floor, over all placed schemes and
over the open schemes the tool may bundle (FDC, the LCC outline with and
without LCSH vocabulary, LCC full, BK and the learned scheme).

Table 20. Winner of the default-scheme rule on the <!-- v:x_books_n -->3,000<!-- /v --> books,
settings won out of <!-- v:x_def_settings -->27<!-- /v -->, by coverage floor.

<!-- table:default_floors -->
| coverage floor | winner, all placed schemes (settings) | winner, bundleable open schemes (settings) |
| --- | ---: | ---: |
| 0.70 | `lcc-lcsh-placed` 15, `unesco-thesaurus-placed` 12 | `lcc-lcsh-placed` 18, `fdc-placed` 9 |
| 0.80 | `lcc-lcsh-placed` 15, `unesco-thesaurus-placed` 12 | `lcc-lcsh-placed` 18, `fdc-placed` 9 |
| 0.85 | `lcc-lcsh-placed` 15, `unesco-thesaurus-placed` 12 | `lcc-lcsh-placed` 27 |
| 0.90 | `lcc-lcsh-placed` 15, `unesco-thesaurus-placed` 12 | `lcc-lcsh-placed` 27 |
| 0.95 | `lcc-lcsh-placed` 27 | `lcc-lcsh-placed` 27 |
<!-- /table:default_floors -->

At the <!-- v:x_def_floor -->0.90<!-- /v --> floor, `<!-- v:x_def_winner -->lcc-lcsh-placed<!-- /v -->`, the LCC outline
with LCSH headings as search vocabulary, wins all <!-- v:x_def_wins -->27<!-- /v --> of
<!-- v:x_def_settings -->27<!-- /v --> settings among the open schemes, and it is the only
winner at every floor from <!-- v:x_def_min_floor_unique -->0.85<!-- /v --> up. Over all placed
schemes it wins <!-- v:x_def_all_lcclcsh -->15<!-- /v --> and the UNESCO Thesaurus
<!-- v:x_def_all_unesco -->12<!-- /v -->; the thesaurus is not among the schemes the tool may
bundle. Part of the win is vocabulary, not scheme: LCSH lifts the outline's
coverage from <!-- v:x_def_cov_lcc -->0.915<!-- /v --> to <!-- v:x_def_cov_lcclcsh -->0.985<!-- /v -->, while FDC
places <!-- v:x_def_cov_fdc -->0.839<!-- /v --> with its captions alone and falls below the
floor. The bundled default of the tool is this scheme (`lcc@2024`). The
decision record also documents an alternative for a catalogue-first
pipeline, FDC cut to Dewey sections, which rests on the gold-number tie of
Section 5.10 and on openness and notation length rather than on a metric
result.

# 6. Discussion

**What the measurement supports.** The method gives one scale for schemes of
different size, depth and domain. It costs little: no scoring run took more
than <!-- v:score_seconds_max -->6.9<!-- /v --> s, and the reference tree took
<!-- v:ref_seconds_corpus -->58<!-- /v --> s on the corpus graph. Its inputs are pinned by hash. On gold labels it separates a
well-fitted scheme (arXiv on arXiv) from a general scheme on books (LCC on
the holdout) and from schemes with too little gold to judge (MSC, JEL). Within
one classifier, the order of schemes on the papers does not depend on the
embedding model, k or graph threshold (Section 5.9). Several book-side
comparisons are stable in every setting we tried: the cut depth of a
Dewey-derived scheme, and the cost of the full number against its
three-digit section (Section 5.10).

**What it does not support.** It does not support a ranking of universal
schemes, for four reasons that each have a number behind them.

1. *Coverage.* Classifier placement is the only way to reach most of the
   schemes, and coverage explains <!-- v:x_cov_rho_sq -->59%<!-- /v --> of the rank variance of cost over the
   placed runs (Section 5.8). Gold placement does not escape this: MSC's and
   JEL's gold scores measure how few papers carry a code.
2. *The classifier.* The order of schemes changes when the classifier does:
   the Spearman correlation between the lexical and shortlist tiers' skill
   is <!-- v:x_sem_rho -->0.14<!-- /v -->, and the leader changes from <!-- v:x_stab_lead_lexical -->openalex-topics<!-- /v --> to <!-- v:x_stab_lead_shortlist -->arxiv<!-- /v --> (Sections 5.8
   and 5.9). The stopword fix shows the same risk from the other side: a
   classifier defect can change placements drastically while skill barely
   moves, or moves the wrong way.
3. *The objective.* FDC ranks above LCC under triplet accuracy in
   <!-- v:x_stab_triplet_fdc_wins -->27<!-- /v --> of <!-- v:x_stab_settings -->54<!-- /v --> settings and under cost in <!-- v:x_stab_cost_fdc_wins -->0<!-- /v -->. Dewey sections against LCC on
   books is a setting-dependent tie (<!-- v:x_dl_c_match -->13<!-- /v --> of <!-- v:x_dl_settings -->27<!-- /v --> settings match the claimed sign
   under cost, <!-- v:x_dl_jp_match -->8<!-- /v --> the tie under the corrected objective).
4. *The metric's validity.* The cost agrees with a blind judge at $\kappa$ =
   <!-- v:x_val_kappa -->0.27<!-- /v -->. The false-join correction raises that to <!-- v:x_val_kappa_fj -->0.48<!-- /v --> against the
   judge and not against citation or subject-heading signals, and the judge is
   one model (Sections 5.6 and 5.7).

A scheme's rank on this metric is therefore a statement about a similarity
graph, a classifier and an objective. It is not a statement about the
scheme's quality in use. The benchmark's own consumers should read it that
way. The default scheme follows a stated rule rather than a
ranking: the bundled LCC outline with LCSH vocabulary wins it in
<!-- v:x_def_wins -->27<!-- /v --> of <!-- v:x_def_settings -->27<!-- /v --> book settings among the open
schemes (Section 5.12), and that win is partly a vocabulary effect. On
librarian-assigned numbers the metrics do not separate Dewey from LCC, so a
catalogue-first pipeline could choose either on other grounds
(`docs/decisions/default-scheme.md`).

**The learned scheme.** A tree fitted to the objective sets the floor for
how low the cost can go on a given graph. Its lead over human schemes is real
in the sense that it holds on graphs it was not fitted to, and partly home
advantage: it is larger on the fit graph (Section 5.11). It cannot be a
scheme in the library sense without a notation and a way to file a new item
that does not need the embedding model.

**Toward a fair comparison.** The benchmark design lays out controls
(research/benchmark/DESIGN.md §2.5 and §3). A coverage control and a second
classifier tier, an embedding, k and threshold sweep, item bootstrap
intervals and a judge-based validity check have now been run. A
permutation null that keeps each scheme's shape and class-size histogram, a
label-intrinsic track that needs no works, a noise-injection curve that
calibrates classifier error on LCC, and a placement by a classifier other
than shelfmark's (Annif or a prompted LLM) have not. The largest remaining
gain would come from a second annotator model and a human annotator on the
validity sample, and from a graph that is not built from embeddings (co-use
or citations).

**Interpretation of skill.** Skill compares a scheme with a tree fitted to
the same similarity graph. A scheme encodes many relations besides textual
similarity: discipline, form, audience, place, period. A low skill can mean
the scheme organizes by something the embedding does not see. Our score
measures agreement with one notion of relatedness. It does not measure
quality in every sense that matters to libraries.

# 7. Limitations

- **Classifier bias.** Every non-gold placement comes from shelfmark's own
  classifier: the lexical profile for the main tables, and a BGE-small
  shortlist profile for Section 5.8. The lexical profile matches class
  labels and attached vocabulary, so a scheme with rich vocabulary is
  reached better than one with bare labels, independent of its structure.
  Most placements are below the classifier's own confidence target
  (Table 7). Table 8 and Section 5.8 show the classifier effect is as
  large as the effects we want to measure. No classifier outside this
  project (for example Annif or a prompted LLM) was run for comparison. The
  rerank profile is still a stub, so only the shortlist profile was
  measured.
- **One annotator for validity.** The validity check used one LLM judge, on
  <!-- v:x_val_n -->953<!-- /v --> pairs from <!-- v:x_val_runs -->8<!-- /v --> runs in three strata, so its shares describe the
  sample and not the population of pairs. The judge is self-consistent
  ($\kappa$ = <!-- v:x_val_self_kappa -->0.92<!-- /v -->), but self-consistency is not validity. There is no human
  annotator and no second model. The non-LLM signals (subject headings,
  citations) are partial, and the citation signal is thin for recent
  preprints.
- **Science-heavy corpus.** <!-- v:share_physical_math_cs -->86.4%<!-- /v -->
  of the corpus is computer science, mathematics or physics. Humanities,
  law, arts and general nonfiction appear only in the 500-record holdout,
  which is itself <!-- v:holdout_p_share -->76.4%<!-- /v --> class P, and
  in the <!-- v:x_books_n -->3,000<!-- /v -->-book sample, whose graph text is mostly LC subject headings.
  Domain schemes (arXiv, MSC, ACM CCS) are scored on their home ground and
  general schemes are not, so no cross-type comparison is fair.
- **Low gold coverage for MSC and JEL.** Only
  <!-- v:gold_msc_cov -->13.2%<!-- /v --> of papers carry a valid MSC code
  and <!-- v:gold_jel_cov -->0.02%<!-- /v --> a JEL code. Unplaced items
  hang under the root, so these gold scores measure coverage, not
  structure. The covered-subset analysis (Section 5.8) is the control for
  this, and its subset scores use a restricted graph.
- **ACM CCS gold missing, and no official crosswalk.** The corpus field
  meant to carry ACM CCS labels holds the ACM classes submitted to arXiv,
  which are CCS 1998 codes. No value is a CCS 2012 concept path, so the gold
  run was skipped. ACM publishes no CCS 1998 to CCS 2012 mapping: it says it
  built one for its own digital library and exposes it only as terms on
  each article's page, and the published CCS 2012 SKOS file holds no
  mapping or 1998 codes (research/reports/5). We did not build a mapping,
  because any we made would be ours and not ACM's, so the ACM CCS placed run
  has no gold counterpart.
- **Scheme licensing and the DDC position.** Several schemes cannot be
  redistributed: ACM CCS (educational and research use only), JEL (no open
  licence), MSC (non-commercial, share-alike), EuroVoc (terms stated
  inconsistently) and ILC2 (conflicting licence statements) (Table 2).
  Their files are built locally from the publishers' downloads and verified
  by hash, but they are not published. Exact reproduction therefore depends
  on the upstream downloads staying available and unchanged. DDC, which OCLC
  describes as the world's most widely used way to organize library
  collections, is owned by OCLC [@deweyrights]. We use Dewey class *numbers*
  from public catalogue records, and the tree is the notation's digit
  prefixes. No Dewey caption, schedule, table, Manual entry or relative-index
  term from any edition is used, and nothing is taken from OCLC services. The
  position rests on class numbers being short numeric strings in public
  catalogue data; it is an engineering position, not legal advice, and it is
  written for the United States and the European Union only
  (research/reports/7). The Dewey-derived scheme that can be bundled and
  used to *place* items is FDC [@fdc]. Even LCC's open status depends on
  jurisdiction. The Library of Congress calls its linked data a
  public-domain data set [@idloc], but its Classification Web notice says
  LC-originated classification data are copyrighted except within the USA
  [@classweblegal].
- **Titles-only holdout.** The book holdout has no descriptions. Its
  similarity graph is built from titles averaging
  <!-- v:holdout_title_words -->3.5<!-- /v --> words, and the
  classifier finds candidates for few of them. The holdout is also small
  (n = <!-- v:n_holdout -->500<!-- /v -->). Tables 3 to 10 give point values;
  intervals for the placed runs are in the bootstrap runs of
  `bench/results/2026-10-10-covered` and `-semantic`.
- **Method limits.** The kNN-sparsified cost agrees with a dense cost on
  random subsets (Section 5.8) but has not been checked on the full corpus.
  The embedding-model, k and threshold sweep uses three models; the
  similarity is always an embedding. Polyhierarchies are reduced to one
  parent per class. Only one placement per item is used, although 35.5% of a
  200-work Open Library sample had more than one LCC value (DESIGN.md §0).
  The balance statistics count each item at every ancestor class (Section
  3.5). The navigation models are unfitted. The 3,000-book sample is
  stratified equally over LCC's main classes, which favours LCC's
  false-join term.
- **Search for prior work.** The novelty claim rests on the search in
  research/reports/8-prior-work-search.md: 94 Crossref queries over the
  JASIS/JASIST family, Knowledge Organization, Cataloging & Classification
  Quarterly, ISKO, SIG/CR and NASKO proceedings, JCDL and TPDL/ECDL, plus
  five web searches. It screened titles and abstracts only. Full texts and
  non-English literature were not searched, one person screened, and
  Crossref's coverage of proceedings is incomplete (ISKO proceedings before
  2012 are missing).

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
- **Later results.** `docs/validity/` (sample, rubric, labels and tables),
  `bench/results/2026-10-10-covered`, `-embed`, `-semantic`, `-learned`,
  `2026-10-11-falsejoin`, `-stability` and `-book-stability`, each with a
  `REPRODUCE.md`. The judge's labels are in `docs/validity/sample.jsonl`;
  rerunning the judge needs the same model and rubric.
- **Determinism.** With the same graph and scheme files, gold scores
  reproduce exactly. Placed scores reproduce exactly with the same
  classifier build.
- **This paper.** `paper/paper1/scripts/make_tables.py` regenerates every
  table and marked number from the JSON, using `paper/common/extra_results.py`
  for the validity, stability, book and semantic-tier values. `--check` fails
  if `paper.md` is stale. It also checks that the r2 and r3 runs used the
  2026-10-10 graphs and scheme files, and that the corpus files match their
  recorded hashes.

# Acknowledgements

The author thanks Hetzner for offering an API for autonomous server
provisioning; DeepSeek for releasing open-source models; and Anthropic for
Claude, which assisted with the analysis, code and drafting. The validity
labels were produced by an Anthropic model acting as the judge (Section
3.7).

# References

<!-- Rendered from references.bib by the build tool (e.g. pandoc --citeproc). -->
