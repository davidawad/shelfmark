#import "../template.typ": *
#import "../generated.typ": *

= Coverage, classifier tier and rank stability <sec:stability>

== Coverage control <sec:coverage>

A gold run's unlabelled items hang under the root and are charged the maximum,
so coverage enters the cost directly. The covered-subset analysis rescored the
placed runs on each run's own covered items. Coverage explains #v.x_cov_rho_sq
of the rank variance of Dasgupta cost over the #v.x_cov_runs placed runs on
the papers ($rho = #v.x_cov_rho$), and the ranking by cost on all items and on
each run's own covered items barely agrees (Kendall $tau = #v.x_cov_tau$).
Low-coverage schemes move from the bottom to the middle once their unplaced
papers are dropped. The sparse graph is not the cause: on random subsets of
1,000 to 5,000 papers, the $k$NN-sparse cost and a dense cost rank the placed
runs with Spearman $rho >= #v.x_dense_min_rho$.

== A second classifier tier <sec:tier>

The shortlist profile adds a BGE-small embedding shortlist after the lexical
tiers and files every item, so coverage is #v.x_sem_min_cov for all
#v.x_sem_n schemes. The rerank profile is unavailable in this build and
reproduces the shortlist placements. @tab:semantic compares the tiers on the
same graph and schemes. Skill rises for #v.x_sem_up of #v.x_sem_n schemes, by
much more where the lexical tier placed few papers. arXiv's own taxonomy moves
from #v.x_sem_arxiv_lex to #v.x_sem_arxiv_sh and now leads, with
#raw(v.x_sem_second) second (#v.x_sem_second_skill), and the Spearman
correlation between the two tiers' skill values is #v.x_sem_rho. The order of
schemes under a classifier is a property of the classifier--scheme pair. The
run's directory is marked pending in the results index, so the numbers are
read from its committed JSON and should be treated as preliminary. The tier
that places by embedding also uses a different model from the graph's.

#wide-table(
  tbl(
    semantic-tier-header,
    semantic-tier,
    align: (left, right, right, right, right, right),
    size: 7.2pt,
  ),
  [Lexical (`profile:lexical@2`) and shortlist (`profile:shortlist@2`)
   placement on the arXiv corpus.],
  label: <tab:semantic>,
)

== Rank stability on the papers <sec:rankstab>

The stability sweep rescored the 13 schemes under #v.x_stab_settings controlled
settings: three embedding models, $k$ of 10, 20 and 50, three graph
thresholds, and the lexical and shortlist tiers. A ranking claim is called
robust when the minimum Spearman $rho$ over every pair of settings is at least
#v.x_stab_threshold. @tab:stability gives the minimum and median $rho$ between
settings that use the same tier and between settings that do not.

#wide-table(
  tbl(
    stability-header,
    stability,
    align: (left, right, right, right, right, right),
    mono-first: false,
    size: 7.2pt,
  ),
  [Rank agreement of the 13 schemes between controlled settings, by
   objective, and the number of settings in which FDC ranks above LCC.],
  label: <tab:stability>,
)

Within a tier the rankings are stable: cost, revenue and skill have a minimum
$rho$ of #v.x_stab_cost_min_same over the embedding, $k$ and threshold sweep.
Across tiers they are not: the minimum is #v.x_stab_cost_min_diff and the
median #v.x_stab_cost_med_diff. The leader under cost is
#raw(v.x_stab_lead_lexical) under the lexical tier and
#raw(v.x_stab_lead_shortlist) under the shortlist tier. Objectives also
disagree. FDC ranks above LCC by cost, revenue, skill and locality in
#v.x_stab_cost_fdc_wins settings. By triplet accuracy it ranks above LCC in
#v.x_stab_triplet_fdc_wins of #v.x_stab_settings, and that ranking can reverse
between settings (minimum $rho$ #v.x_stab_triplet_min_diff). A claim that one
scheme beats another on these papers therefore depends on the classifier and
on the objective. The graph settings matter much less.
