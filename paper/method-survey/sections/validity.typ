#import "../template.typ": *
#import "../generated.typ": *

= Validity of the cost <sec:validity>

The cost scores agreement with a similarity graph. Whether it tracks what a
librarian would call a good decision is a separate question, and we test it
rather than assume it.

== A blind judge, and a false-join term <sec:judge>

We drew #v.x_val_n pairs from eight runs in three strata: similar pairs the
scheme splits at depth one or less (A), dissimilar pairs it joins deep (B),
and random placed pairs (C). One LLM judge labelled each pair blind under a
rubric fixed before judging (`docs/validity`). The metric flags a pair when
its cost contribution $w |"leaves"("lca")| slash n$ exceeds the run's mean
contribution per graph edge. We report Cohen's $kappa$ @cohen1960 and the AUC
of the raw contribution, with percentile intervals that resample pairs within
strata @efron1979. Over all pairs, the flag agrees with the judge at
$kappa = #v.x_val_kappa$ [#v.x_val_kappa_lo, #v.x_val_kappa_hi], with
agreement #v.x_val_agree and AUC #v.x_val_auc. That is weak agreement. The
judge is stable against itself ($kappa = #v.x_val_self_kappa$ on a blind
second pass of #v.x_val_self_n pairs).

The strata show where the gap is. Among similar pairs the scheme splits (A),
the judge calls #v.x_val_A_incorrect incorrect and the metric flags
#v.x_val_A_flagged. Among dissimilar pairs the scheme joins deep (B), the
judge also calls #v.x_val_B_incorrect incorrect and the metric flags
#v.x_val_B_flagged: those pairs have zero weight, so the cost is blind to
them by construction. Shares describe a stratified sample, one third per
stratum, not the population of pairs. Over the eight runs, the Spearman
correlation of a run's `cost_norm` with the judge's error rate on random pairs
is #v.x_val_rho_c, and over all strata #v.x_val_rho_all; with eight points
this is descriptive only.

The correction adds a false-join term. For a seeded sample of non-edge pairs
whose cosine is below $tau$, weighted by dissimilarity $d = tau - cos$, the
specificity form is
$ "FJs" = (sum d ln(n slash N("lca"))) / (ln n sum d), $ <eq:fjs>
so that deep joins of dissimilar items cost more than shallow ones. The
corrected objective is $J' = C + beta "FJs" + lambda N$ with $beta = 1$,
$lambda = 0.002$ and $N$ the $"nav"_"log"$ cost in bits. An outside-mass form
is also computed. Adding the term raises agreement to
$kappa = #v.x_val_kappa_fj$ [#v.x_val_kappa_fj_lo, #v.x_val_kappa_fj_hi], a
paired change of #v.x_val_dkappa_fj [#v.x_val_dkappa_fj_lo,
#v.x_val_dkappa_fj_hi], and the AUC from #v.x_val_auc_base to #v.x_val_auc_fj
(@tab:validity-variants). The term alone does not do it
($kappa = #v.x_val_kappa_fjs_alone$). Weights from 0.25 to 8 give $kappa$
between #v.x_val_kappa_fjs_min and #v.x_val_kappa_fjs_max, so $beta$ is not
critical in that range. Every placed run except the learned scheme gains, with
intervals that exclude zero; arXiv's own taxonomy and the learned scheme do
not, because the judge rarely finds a false join in them
(@tab:validity-runs).

#col-table(
  tbl(
    validity-variants-header,
    validity-variants,
    align: (left, right, right, right),
    mono-first: false,
    size: 7.2pt,
  ),
  [Agreement of the metric's flag with the judge, pooled over all pairs, for
   Dasgupta cost and the false-join variants (same pairs and labels).],
  label: <tab:validity-variants>,
)

#wide-table(
  tbl(
    validity-runs-header,
    validity-runs,
    align: (left, right, right, right),
    size: 7.2pt,
  ),
  [Judge agreement before and after adding the false-join term, per run.],
  label: <tab:validity-runs>,
)

== Checks that do not use the judge <sec:external>

Two signals are independent of the LLM judge: a shared Library of Congress
subject heading between books, and bibliographic coupling or co-citation
between arXiv papers from OpenAlex reference lists. @tab:external gives the
pair-level agreement ($kappa$) of each flag with the signal, the scheme-level
rank agreement ($rho$) and the separation each signal finds between an FDC
backbone and an LCC backbone. The false-join term does _not_ raise pair-level
agreement with any of the three (coupling #v.x_ext_coupling_k0 to
#v.x_ext_coupling_k1, co-citation #v.x_ext_cocitation_k0 to
#v.x_ext_cocitation_k1, subject headings #v.x_ext_lcsh_k0 to
#v.x_ext_lcsh_k1), and leaves the scheme-level rank agreement where it was.
The corrected metric is better supported than the original by one annotator
model, and not by independent signals. These signals have their own limits:
most 2025 to 2026 preprints in the corpus have no OpenAlex reference list, and
LC assigns subject headings and call numbers in one record, so that signal is
not independent of LCC.

#wide-table(
  tbl(
    external-header,
    external,
    align: (left, right, right, right, right, left),
    mono-first: false,
    size: 7pt,
  ),
  [Agreement of the cost and the corrected cost with non-LLM signals.
   Separation: difference in AUC between the FDC and LCC backbones, with a 95%
   interval from paired draws.],
  label: <tab:external>,
)

The judge's together-or-apart call agrees with whether the pair is an edge of
the graph at $kappa = #v.x_val_together_kappa$, and disagrees with whether the
scheme put the pair together ($kappa = #v.x_val_scheme_together_kappa$). On
its sample the judge finds a false-join share of #v.x_val_false_join and a
false-split share of #v.x_val_false_split of all sampled pairs, shares that
depend on how the strata were drawn.
