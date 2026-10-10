#import "../template.typ": *
#import "../generated.typ": v

= Discussion <sec:discussion>

*What the measurement supports.* The method gives one scale for schemes of
different size, depth and domain. It costs little: the average-linkage
reference took 58.4 s on the corpus graph and 0.1 s on the holdout graph, and
the whole first run, including embedding and scheme builds, took about 45
minutes on 8 CPUs, with placement into the 416,186-class `lcc-full` the
longest single job.
Its inputs are pinned by hash. On gold labels it separates a well-fitted
scheme (arXiv on arXiv) from a general scheme on books (LCC on the holdout)
and from schemes with too little gold to judge (MSC, JEL).

*What it does not yet support.* It does not support a ranking of universal
schemes. The general schemes that matter most for libraries (LCC, BK, FDC,
ILC2) have gold labels only for LCC, and only on a small title-only holdout.
Classifier placement, the only way to reach the other schemes, moves skill by
as much as the differences we would want to detect (@tab:bias). The stopword
defect shows a second risk. A classifier defect can change placements
drastically while skill barely moves, or moves it the wrong way
(@sec:stopword).

*Toward a fair comparison.* The benchmark design lays out the controls
needed. These are a cost--resolution curve against the reference cut at
matched class counts; a permutation null that keeps each scheme's shape and
class-size histogram; item bootstrap intervals; a label-intrinsic track that
needs no works; and a noise-injection curve that calibrates classifier error
on LCC, where both tracks exist. None of these was run for this draft. The
largest gain would come from a large gold set of books that carry both LCC
and DDC. The benchmark design counted about 3.91 million Open Library search
documents with both labels and `language:eng` (queried 9 October 2026). DDC could be scored by its digit tree alone, which needs no
captions.

*Interpretation of skill.* Skill compares a scheme with a tree fitted to the
same similarity graph. A scheme encodes many relations besides textual
similarity: discipline, form, audience, place, period. A low skill can mean
the scheme organizes by something the embedding does not see. Our score
measures agreement with one notion of relatedness. It does not measure
quality in every sense that matters to libraries.

*On the fitted scheme.* A scheme fitted to the benchmark's own objective
scores well on that objective, as it must. The result is useful as a
reference point, not as a ranking: a hand-built scheme can be described as
a given distance from a fitted scheme with the same number of levels on the
same graph. Whether an open, learned, order-preserving classification can
become a general one is open (@sec:learned).

= Limitations <sec:limits>

- *Classifier bias.* Every non-gold placement comes from shelfmark's own
  lexical classifier. It matches class labels and attached vocabulary, so a
  scheme with rich vocabulary is reached better than one with bare labels,
  independent of its structure. Most placements are below the classifier's
  own confidence target (@tab:placement). @tab:bias shows the classifier
  effect is as large as the effects we want to measure. No other classifier
  (for example Annif or an LLM) was run for comparison.
- *Science-heavy corpus.* 86.4% of the corpus is computer science,
  mathematics or physics. Humanities, law, arts and general nonfiction appear
  only in the 500-record holdout, which is itself 76.4% class P. Domain
  schemes (arXiv, MSC, ACM CCS) are scored on their home ground and general
  schemes are not, so no cross-type comparison is fair.
- *Single embedding.* The graph is built from one embedding model, one value
  of $k$ and one mean offset, with no sensitivity runs. A different
  embedding, or a similarity from citations or subject headings, could
  reorder results. The fitted scheme of @sec:learned is the most exposed to
  this circularity.
- *Low gold coverage for MSC and JEL.* Only #v.gold_msc_cov of papers carry a
  valid MSC code and #v.gold_jel_cov a JEL code. Unplaced items hang under
  the root, so these gold scores measure coverage, not structure. A
  covered-subset score, restricted to labelled items and their edges, is not
  yet implemented.
- *ACM CCS gold missing.* The corpus carries CCS 1998 codes, the scheme is CCS
  2012, and no official crosswalk exists (@sec:survey). The gold run was
  skipped, and we did not build a mapping by hand.
- *Scheme licensing.* Several schemes cannot be redistributed: ACM CCS
  (educational and research use only), JEL (no open licence), MSC
  (non-commercial, share-alike), EuroVoc (terms stated inconsistently) and
  ILC2 (conflicting licence statements) (@tab:survey). Their files are built
  locally from the publishers' downloads and verified by hash, but are not
  published. Exact reproduction therefore depends on the upstream downloads
  staying available and unchanged. DDC, which OCLC describes as the
  world's most widely used way to organize library collections, is absent
  because OCLC holds its copyright @deweyrights. Even
  LCC's open status depends on jurisdiction (@sec:survey).
- *Titles-only holdout.* The book holdout has no descriptions. Its similarity
  graph is built from titles averaging 3.5 words, and the classifier finds
  candidates for few of them. The holdout is also small ($n = 500$), and no
  confidence intervals are reported.
- *No uncertainty or null model on the headline numbers.* Skill is a point
  estimate relative to average linkage alone, so tree shape and class-size
  distribution are confounded with scheme content, and no interval is
  reported. A permutation null, bootstrap intervals, a rank-stability report
  over $k$, embedding and objective, and a librarian-validity sample are
  specified but not run (project beads dot-8mj8m.33 to .36). Rankings in
  this draft are therefore indicative.
- *Navigation mixes coverage with structure.* Unplaced items sit under the
  root, so low coverage inflates root fan-out and every navigation cost.
  Navigation columns must be read next to coverage (@tab:placement).
- *Method limits.* The $k$NN-sparsified cost has not been checked against the
  dense cost. Polyhierarchies are reduced to one parent per class. Only one
  placement per item is used, although 35.5% of a 200-work Open Library
  sample of English documents had more than one LCC value. The balance statistics count each item
  at every ancestor class (@sec:structure). The navigation models are
  unfitted.
- *Self-embedding circularity of the fitted scheme.* Its insertion and the
  graph share an embedding, its corpus rows are in sample, and its labels
  were written by the AI coding assistant that built it and have not been
  evaluated by people (@sec:learned).
- *Search for prior work.* The novelty claim rests on the search in project
  report 8: 94 Crossref queries over the JASIS/JASIST family, Knowledge
  Organization, Cataloging & Classification Quarterly, ISKO, SIG/CR and NASKO
  proceedings, JCDL and TPDL/ECDL, plus five web searches. It screened titles
  and abstracts only. Full texts and non-English literature were not
  searched, one person screened, and Crossref's coverage of proceedings is
  incomplete (ISKO proceedings before 2012 are missing).
