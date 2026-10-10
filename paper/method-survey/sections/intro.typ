#import "../template.typ": *
#import "../generated.typ": v

= Introduction <sec:intro>

A universal classification scheme gives every work one address in a single
tree. The Library of Congress Classification (LCC), the Dewey Decimal
Classification (DDC), the Universal Decimal Classification (UDC), the Basic
Classification (BK), the Integrative Levels Classification (ILC) and others
each claim to arrange all of knowledge. A practitioner choosing among them,
or a researcher designing a new one, needs to ask which arrangement keeps
related works close together, and at what navigation cost. The literature
answers this mostly qualitatively, by notation, facet structure, history and
institutional adoption @nocetti1978 @zins2011 @oh2023 @wu2023. Quantitative
comparisons have measured retrieval performance @cleverdon1960 @keen1973
@carpenter1978 or the scatter of a subject across schemes @oneill1987
@connaway1996, not the structure of the hierarchy itself (@sec:related). A
structured search of JASIST, Knowledge Organization, Cataloging &
Classification Quarterly, ISKO and SIG/CR proceedings, JCDL and TPDL (project
report 8, `research/reports/8-prior-work-search.md`) found no study that
scores several universal schemes on a common hierarchical objective over the
same corpus, and no use of Dasgupta cost or Moseley--Wang revenue on a
human-built scheme. The search covered titles and abstracts, not full texts
(@sec:limits), so this is absence of evidence, not proof.

Three developments make the question more pressing.

+ *The main lookup service is gone.* OCLC Classify returned the most common
  DDC and LCC numbers for a work from WorldCat holdings. OCLC discontinued it
  on 31 January 2024 @oclcclassify. No free service replaced it.
+ *Automatic placement has not closed the gap.* Annif, the most mature open
  subject-indexing toolkit, is trained separately for each vocabulary
  @annif. On more than 230,000 Swedish union-catalogue records its best
  (ensemble) configuration reaches 66.82% accuracy at three-digit DDC
  @libris2024ddc. Recent work that uses large language models (LLMs) for LCC
  and DDC assignment reports weakness with more granular classes @llmlcc2026
  and a lack of specificity @nasko2025, and prompted LLMs fall well short of
  fine-tuned smaller models on DDC classes for research data @llm4ddc2025. The Library of Congress's own experiments
  predicted subject and genre with under 50% accuracy, short of a 95%
  automation threshold @lcecd2025. Placement is getting cheaper, so the open
  question shifts towards _which scheme_ to place into.
+ *Licences constrain open use.* DDC is copyrighted by OCLC @deweyrights.
  The full UDC is licensed @udclicences. Openly licensed universal schemes
  are shallow, such as BK @bk and the Free Decimal Correspondence (FDC)
  @fdc, or their licence terms are unsettled, as for ILC2 @ilc2.
  Data-driven vocabularies such as OpenAlex Topics are open but cover
  research literature only @openalextopics. A quantitative comparison helps
  decide whether an open scheme is good enough to adopt.

*Contributions.* (i) A survey of the thirteen scheme builds benchmarked here,
with DDC and UDC as context, covering structure, size, maintainer and
redistribution status (@sec:survey). (ii) A scheme-agnostic scoring method.
It induces a tree from a scheme and a set of placements, and scores the tree
with Dasgupta cost and Moseley--Wang revenue on a fixed similarity graph.
The score is normalized against an average-linkage reference tree on the same
graph, and coverage, structure and navigation statistics are reported with it
(@sec:method). (iii) A pinned, CC0-metadata corpus of 10,000 arXiv papers and
a 500-edition LC holdout (@sec:corpus). (iv) Results for 13 scheme builds.
Gold placements and classifier placements are kept apart, and the classifier
bias is calibrated where both exist (@sec:results). (v) A methods note on a
classifier defect that the benchmark exposed, with its before-and-after
effect (@sec:stopword). (vi) A short account of an order-preserving scheme
fitted to the same objective, with its caveats (@sec:learned).

== Related work <sec:related>

*Comparing classification schemes.* The Cranfield project compared the
Universal Decimal Classification, an alphabetical subject catalogue, a faceted
classification and Uniterm coordinate indexing, applied to 18,000 aeronautics
documents @cleverdon1960 @jahoda1961. Keen compared five index languages built
for a test collection on retrieval effectiveness @keen1973, and Carpenter et
al. compared five patent classifications on recall and precision for the same
questions @carpenter1978. O'Neill et al. measured how the classes of LCC and
DDC disperse into each other on MARC records @oneill1987, and Connaway and
Sievert compared the scatter of one subject across LCC, DDC and the NLM
Classification @connaway1996. Zins and Santos mapped the top two levels of
LCC, DDC and UDC onto one reference model of knowledge @zins2011. In
bibliometrics, research classifications are scored on citation-based
criteria: algorithmic journal maps against the ISI Subject Categories
@rafols2009, the Web of Science and Scopus journal classifications against
each other @wang2016, and taxonomies built from different citation relations
against one gold standard @klavans2017. These studies score schemes by
retrieval, by the scatter of a mapping or by citation coherence. None scores
the tree itself. For a single vocabulary, corpus-weighted pruning measures how
much of a subject tree a collection needs @julien2013, checking functions
detect quality defects in SKOS vocabularies @mader2012, and structural
analysis is one of the established ways to evaluate a thesaurus @owens2004.

*Objectives for hierarchical clustering.* Dasgupta defined the cost of a tree
$T$ over items with pairwise similarities $w_(i j)$ as
$sum_(i<j) w_(i j) |"leaves"(T[i or j])|$, where $i or j$ is the lowest
common ancestor (LCA) of $i$ and $j$ @dasgupta2016. Similar pairs should be
separated as deep in the tree as possible. A flat tree has the maximum cost.
Moseley and Wang studied the dual, revenue,
$sum w_(i j) (n - |"leaves"(T[i or j])|)$, so that cost plus revenue equals
$n sum w_(i j)$. They showed that average linkage is a $1/3$-approximation to
the maximum revenue @moseley2017 @moseley2023. Optimizing Dasgupta cost
exactly is NP-hard. Approximation algorithms use sparsest cut and spreading
metrics @charikar2016 @roy2016. Cohen-Addad et al. give an axiomatic family of
admissible objectives and show when average linkage and its variants recover
planted hierarchies @cohenaddad2017. Hyperbolic embeddings give a
differentiable relaxation @chami2020, and structural constraints can be added
to the search @chatziafratis2018. scikit-network ships a Dasgupta cost and
score for binary dendrograms @sknetwork. This line of work treats the
objective as something to _optimize_. We use it first to _evaluate_ trees
that people built.

*No objective-free optimum.* Kleinberg's impossibility theorem says no
clustering function satisfies scale invariance, richness and consistency
together @kleinberg2002. Carlsson and Mémoli characterize single linkage
axiomatically among hierarchical methods @carlsson2010. One consequence for
us: a score is always relative to the similarity function. We therefore fix
and publish the graph and treat it as part of the measurement.

*Navigation.* Optimal search trees and Huffman coding minimize expected depth
under an access distribution @huffman1952 @knuth1971. The hierarchical-identity
model of Watts, Dodds and Newman makes greedy navigation on a hierarchy a
measurable success criterion @watts2002. Basic-level categories explain why a
natural middle level carries most human use @rosch1976. User studies of menu
hierarchies disagree on the best shape. Speed and accuracy improved with
breadth in one study @snowberry1983, and task time rose with depth for web
links @zaphiris2000, but others found an optimum at moderate breadth
@miller1981 @larson1998. Landauer and Nachbar fit the time per menu screen as
$k + c log b$ for $b$ choices @landauer1985, the form of our
$"nav"_"log"$ model. For a two-level card file, the optimum number of records
per guide is proportional to the square root of the file size @lipetz1970.
We report simple, uniform-use navigation costs and fit no user model.

*Data-driven and LLM-built taxonomies.* Waltman and van Eck cluster
publications on direct citation at several resolutions @waltman2012, using
the Leiden algorithm @traag2019. OpenAlex Topics come from citation
clustering and are named by an LLM, and their upper levels are mapped from
Scopus ASJC categories @priem2022 @openalextopics. TaxoGen and TaxoExpan
build and extend topic taxonomies from text @zhang2018taxogen
@shen2020taxoexpan. LLM pipelines generate layers or labels: Chain-of-Layer,
TnT-LLM and TaxoAdapt @zeng2024chainoflayer @wan2024tntllm
@kargupta2025taxoadapt. None of these optimizes a global objective.
Taxonomy-quality work catalogues attributes and measures
@unterkalmsteiner2023. It includes a reference-free robustness score that
correlates semantic and taxonomic similarity @wullschleger2025, which is
closest in spirit to ours.

*Automatic subject classification.* Annif @annif, its SemEval-2025 Task 5
entry @annifsemeval2025 and the task overview @semeval2025task5 cover subject
indexing with thesauri such as GND. They report results per vocabulary, not
across schemes. An open LCC classifier covers 15 of the 21 top-level classes
@autolcc. We found no public leaderboard that compares classification
schemes, as opposed to classifiers.
