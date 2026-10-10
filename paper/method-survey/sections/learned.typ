#import "../template.typ": *
#import "../generated.typ": *

= A fitted order-preserving scheme <sec:learned>

The benchmark makes a converse question natural: if a scheme can be scored,
can one be _fitted_ to the score, under constraints a library needs? We report
a first, short experiment; the full description is in a companion draft.

*Objective.* On the graph of @sec:graph, the fitted scheme minimizes
$ J(T) = "cost"_"norm" (T) + lambda N(T), quad lambda = 0.005, $ <eq:J>
where $"cost"_"norm"$ is the harness's cost with works under their class and
$N$ is $"nav"_"log"$ with $a = 1$ (@eq:navlog; a leaf class's fan-out is its
number of works). The constraints are 2 to 12 child classes per class, at
most 4 class levels, leaf classes of at least 5 works, and every class a
contiguous run of one fixed shelf order, the alphabetic-tree constraint of
optimal order-preserving codes @hu1971 @knuth1971. New works are filed by
splitting leaf classes only, so that every existing class keeps its code,
label and parent. The fit starts from the average-linkage tree, collapses it
to a bounded-shape class tree by an exact dynamic program, and applies
order-preserving local search; it takes about 2.5 s on the corpus. The
result has 110 classes (2, 5, 16 and 87 by level). Their labels were
written by hand from sheets of class terms and central titles, by the AI
coding assistant that built the prototype and not by a cataloguer.

#wide-table(
  tbl(
    ([evaluation set], [scheme], [placements], [coverage], [cost\_norm], [skill], [$J$], [nav\_log\ ($a=1$)]),
    learned,
    align: (left, left, left, right, right, right, right, right),
    mono-first: false,
    size: 7pt,
  ),
  [The fitted scheme against published schemes on the same graphs.
   "own, in-sample" rows are the fitted assignment of the corpus it was fitted
   to. "own insertion" rows file works by nearest leaf centroid in the
   embedding space of the graph. Rows through `lexical@2` use the classifier of
   @sec:placed. Lower is better for cost and $J$.],
  label: <tab:learned>,
)

*Results.* In sample, the fitted scheme reaches cost #v.l_own_cost (average
linkage #v.l_avg_cost, arXiv's own categories #v.gold_arxiv_cost), skill
#v.l_own_skill and $J$ #v.l_own_J (@tab:learned). Fitted on a random 80% and
filed on the other 20% by insertion, it scores #v.l_held_cost on the test
works, against #v.l_full_on_test_cost for the full fit that saw them. Reached
through the lexical classifier, it scores #v.l_lex_cost: a lower cost than
every published scheme's placed row, and a higher cost than arXiv's gold
labels. Inserting the
500 books as new works produced 16 classes and changed no existing
identifier; on the holdout graph the inserted scheme scores #v.l_book_cost
(skill #v.l_book_skill), against #v.gold_lcc_cost for LCC call numbers.
Through the classifier it places only #v.l_book_lex_cov of the books and
scores #v.l_book_lex_cost.

*Caveats.* These numbers flatter the method, and we do not read them as a win
over any published scheme.

- _In sample._ The corpus rows, and the classifier rows of the fitted
  scheme, use a tree or a vocabulary fitted on those same works. The fitted
  scheme's vocabulary is the corpus's own most distinctive terms.
- _Embedding circularity._ Insertion files a new work at the nearest leaf
  centroid in the same all-MiniLM-L6-v2 space from which the graph is built,
  so a new work is placed by exactly the geometry it is later scored on. This
  holds for the held-out rows and, more strongly, for the book holdout, where
  graph and insertion see only title embeddings. The classifier rows are the
  comparison without it. A fair test needs a second, independent similarity.
- _One corpus._ The scheme has seen only arXiv papers. Its top level splits
  computing from mathematics and the physical sciences, and it has no class
  for the humanities until insertion creates a weak one. Labels of inserted
  classes come from frequent title words and are not usable as captions.
- _The objective._ Dasgupta cost favours a binary split near the top, so the
  fitted root has two children. Library schemes have wide top levels for
  reasons of browsing that $J$ does not model.
- _Weights are modelling choices._ $lambda$, $a$ and the depth bound are not
  fitted. A sweep over depth bounds 3, 4 and 6 and $lambda$ from 0.001 to 0.05
  moves $"cost"_"norm"$ from 0.1577 to 0.2075 and the class count from 330 to
  33.
