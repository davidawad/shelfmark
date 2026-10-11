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
order-preserving local search; it takes about #v.l_fit_seconds s on the corpus.
The result has #v.l_classes classes (#v.l_by_depth by level). Their labels were
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
500 books as new works produced #v.l_ins_new classes and changed
#v.l_ins_stable; on the holdout graph the inserted scheme scores #v.l_book_cost
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
  comparison without it. Rescoring on graphs from two independent embedding
  models (below) removes the circularity of the scoring graph, not of
  filing.
- _One corpus._ The scheme has seen only arXiv papers. Its top level splits
  computing from mathematics and the physical sciences, and it has no class
  for the humanities until insertion creates a weak one. Labels of inserted
  classes come from frequent title words and are not usable as captions.
- _The objective._ Dasgupta cost favours a binary split near the top, so the
  fitted root has two children. Library schemes have wide top levels for
  reasons of browsing that $J$ does not model.
- _Weights are modelling choices._ $lambda$, $a$ and the depth bound are not
  fitted. A sweep over depth bounds #v.l_sweep_depths and $lambda$ from
  #v.l_sweep_lam_min to #v.l_sweep_lam_max moves $"cost"_"norm"$ from
  #v.l_sweep_cost_min to #v.l_sweep_cost_max and the class count from
  #v.l_sweep_classes_max down to #v.l_sweep_classes_min.

*Home advantage.* The fitted scheme's lead is largest on the graph it was
fitted on. On the books of @sec:books, its cost is lower than that of Dewey
sections on every graph, but the margin shrinks off the fit graph
(@tab:homeadv). Refits on the other two embedding models score best on their
own graph (@tab:crossfit): on the held-out split, a fit scored on its own
model's graph averages #v.x_cf_diag_mean skill against #v.x_cf_off_mean
elsewhere, and it is best on its own graph for #v.x_cf_diag_wins of 3 models.
In the sample, the held-out and the classifier-placed rows read #v.x_l_insample_skill,
#v.x_l_heldout_skill and #v.x_l_placed_skill skill. The claim that the fitted
scheme beats Dewey sections is not stable across the #v.x_dl_settings book
settings: the sign matches in #v.x_learn_match under $J'$, with
#v.x_learn_against significant reversals (@tab:claims). We use the fitted
scheme as a reference for how low a tree can go on a graph, not as a
candidate scheme.

*Independent embeddings and false joins.* Rescored on graphs from the two
other embedding models at $k$ = 10, 20 and 50, the fitted scheme still ranks
first by skill in all #v.x_edge_conditions conditions, for the held-out split,
the in-sample fit, the inserted book holdout and the classifier-placed run,
with paired intervals of the margin above zero (@tab:edge). The tightest case
is the held-out split on #v.x_edge_tight_cond, a margin of #v.x_edge_tight
[#v.x_edge_tight_lo, #v.x_edge_tight_hi]. Under the corrected objective $J'$,
against the best non-learned run of each universe on the false-join
rescoring's graphs (@tab:learnedfj), it leads on the papers in
#v.x_lfj_corpus_jp_ahead of #v.x_lfj_corpus_n graphs and on the title-only
holdout in #v.x_lfj_holdout_jp_ahead of #v.x_lfj_holdout_n. On the
#v.x_books_n books it leads in #v.x_lfj_books_jp_ahead of #v.x_lfj_books_n,
ties in #v.x_lfj_books_jp_tie and is behind in #v.x_lfj_books_jp_behind; under
the outside-mass form it is behind in #v.x_lfj_books_cfj_behind of
#v.x_lfj_books_n. Its edge survives a change of embedding. On books it does
not survive the false-join correction.

#wide-table(
  tbl(
    learned-edge-header,
    learned-edge,
    align: (left, right, right, right, left),
    mono-first: false,
    size: 7pt,
  ),
  [The fitted scheme against the best other row in each of
   #v.x_edge_conditions graph conditions. $J$ is the fitted objective.],
  label: <tab:edge>,
)

#wide-table(
  tbl(
    learned-fj-header,
    learned-fj,
    align: (left, left, right, left, right, left, right),
    mono-first: false,
    size: 7pt,
  ),
  [Fitted scheme minus the best non-learned run under $J'$ and under
   $C + "FJ"$, both at $beta = 1$, with paired 95% intervals over 1,000 item
   draws. Negative means the fitted scheme is ahead.],
  label: <tab:learnedfj>,
)

#col-table(
  tbl(
    learned-margin-header,
    learned-margin,
    align: (left, right, right, right, right),
    mono-first: false,
    size: 7.2pt,
  ),
  [Fitted scheme against Dewey sections on books, by graph ($k = 20$). The
   scheme was fitted on MiniLM.],
  label: <tab:homeadv>,
)

#col-table(
  tbl(
    cross-fit-header,
    cross-fit,
    align: (left, right, right, right),
    mono-first: false,
    size: 7.2pt,
  ),
  [Held-out skill of each fit on each model's $k = 20$ papers graph.],
  label: <tab:crossfit>,
)
