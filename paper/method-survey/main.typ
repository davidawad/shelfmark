// Scoring library classification schemes with hierarchical-clustering
// objectives: a survey of open schemes and a common benchmark.
// Build: typst compile paper/method-survey/main.typ paper/method-survey/main.pdf
#import "template.typ": *
#import "generated.typ": v

#show: paper.with(
  title: [Scoring Library Classification Schemes with Hierarchical-Clustering Objectives: A Survey of Open Schemes and a Common Benchmark],
  short-title: [Scoring classification schemes with hierarchical-clustering objectives],
  authors: "David Awad",
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
    on one structural objective over the same set of works. We survey thirteen
    open or openly obtainable scheme builds and propose such a comparison,
    built on objectives from the theory of hierarchical clustering, and we test
    how far it can be trusted. A scheme, restricted to the classes a corpus
    uses, is a rooted tree over the works. We score that tree with Dasgupta's
    cost and Moseley and Wang's revenue on a fixed $k$-nearest-neighbour graph
    of text embeddings, and normalize revenue against an average-linkage tree
    on the same graph; the ratio is called skill. The corpus is
    #v.c_n_corpus arXiv papers with CC0 metadata, a holdout of #v.c_n_holdout
    Open Library editions with librarian-assigned LC call numbers and a sample
    of #v.x_books_n books that also carry Dewey numbers. We keep
    source-assigned ("gold") placements apart from placements made by one
    lexical classifier. On gold labels, arXiv's own taxonomy reaches skill
    #v.gold_arxiv_skill on arXiv papers and LCC reaches #v.gold_lcc_skill on
    the books. With the classifier's placements the same two schemes fall to
    #v.placed_arxiv_skill and #v.placed_lcc_holdout_skill. The findings are
    mostly cautions. Coverage drives most classifier-placed rankings: across 13
    schemes skill correlates with coverage at Spearman $rho = #v.rho$, and
    coverage explains #v.x_cov_rho_sq of the rank variance of cost. The metric
    agrees only weakly with a blind LLM judge ($kappa = #v.x_val_kappa$ on
    #v.x_val_n pairs); a false-join term raises $kappa$ to #v.x_val_kappa_fj,
    but not agreement with citation or subject-heading signals. Rankings are
    stable across embedding model, $k$ and graph threshold within one
    classifier (minimum $rho$ #v.x_stab_cost_min_same) and not across
    classifier tiers (minimum #v.x_stab_cost_min_diff), and objectives
    disagree about FDC against LCC. On #v.x_books_n books, Dewey sections
    against LCC is a tie that depends on the setting. Under a coverage-floor
    rule, the LCC outline with LCSH vocabulary is the default among open
    schemes in #v.x_def_wins of #v.x_def_settings book settings, partly
    through its vocabulary. A fitted order-preserving scheme leads every
    human scheme on its own graph; the lead survives rescoring on two
    independent embedding models, part of it is home advantage, and on books
    it does not survive the false-join correction. We also report a
    classifier defect that the benchmark exposed.
  ],
)

#include "sections/intro.typ"
#include "sections/survey.typ"
#include "sections/method.typ"
#include "sections/corpus.typ"
#include "sections/results.typ"
#include "sections/stopword.typ"
#include "sections/validity.typ"
#include "sections/stability.typ"
#include "sections/books.typ"
#include "sections/learned.typ"
#include "sections/discussion.typ"
#include "sections/repro.typ"

#unnumbered[Acknowledgements]
The author thanks Hetzner for offering an API for autonomous server
provisioning; DeepSeek for
releasing open-source models; and Anthropic for Claude, which assisted with
the analysis, code and drafting.

#show bibliography: set text(size: 8pt)
#show bibliography: set par(justify: false)
#bibliography("references.bib", title: [References], style: "ieee")
