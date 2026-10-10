// Scoring library classification schemes with hierarchical-clustering
// objectives: a survey of open schemes and a common benchmark.
// Build: typst compile paper/method-survey/main.typ paper/method-survey/main.pdf
#import "template.typ": *
#import "generated.typ": v

#show: paper.with(
  title: [Scoring Library Classification Schemes with Hierarchical-Clustering Objectives: A Survey of Open Schemes and a Common Benchmark],
  short-title: [Scoring classification schemes with hierarchical-clustering objectives],
  authors: "AUTHOR TBD",
  keywords: (
    "library classification",
    "hierarchical clustering",
    "Dasgupta cost",
    "knowledge organization",
    "open data",
    "benchmark",
  ),
  abstract: [
    Library classification schemes are compared mostly by description: their
    history, notation, coverage and licence. Quantitative comparisons exist,
    but they measure retrieval effectiveness, the scatter of a subject across
    schemes or coverage of a reference map of knowledge. We found no study
    that scores the hierarchy of several universal or near-universal schemes
    on one structural objective over the same set of works. We survey thirteen open
    or openly obtainable scheme builds and propose such a comparison, built on
    objectives from the theory of hierarchical clustering. A scheme, restricted
    to the classes a corpus uses, is a rooted tree over the works. We score
    that tree with Dasgupta's cost and Moseley and Wang's revenue on a fixed
    $k$-nearest-neighbour graph of text embeddings, and normalize revenue
    against an average-linkage tree on the same graph; the ratio is called
    skill. The corpus is 10,000 arXiv papers with CC0 metadata plus a holdout
    of 500 Open Library editions with librarian-assigned LC call numbers. We
    keep source-assigned ("gold") placements apart from placements made by one
    lexical classifier. On gold labels, arXiv's own taxonomy reaches skill
    #v.gold_arxiv_skill on arXiv papers and LCC reaches #v.gold_lcc_skill on
    the books. With the classifier's placements the same two schemes fall to
    #v.placed_arxiv_skill and #v.placed_lcc_holdout_skill. Across 13 schemes
    placed by the classifier, skill ranges from #v.placed_worst_skill
    (#v.placed_worst) to #v.placed_best_skill (#v.placed_best), and correlates
    with coverage at Spearman $rho = #v.rho$. The classifier, not the scheme,
    accounts for most of the differences in classifier-placed scores. We also
    report a classifier defect that the benchmark exposed, and a short
    experiment in fitting an order-preserving scheme to the same objective,
    whose results are in-sample or share the graph's embedding and are not
    evidence of generalization.
  ],
)

#include "sections/intro.typ"
#include "sections/survey.typ"
#include "sections/method.typ"
#include "sections/corpus.typ"
#include "sections/results.typ"
#include "sections/stopword.typ"
#include "sections/learned.typ"
#include "sections/discussion.typ"
#include "sections/repro.typ"

#unnumbered[Acknowledgements]
To be added.

#show bibliography: set text(size: 8pt)
#show bibliography: set par(justify: false)
#bibliography("references.bib", title: [References], style: "ieee")
