#import "../template.typ": *
#import "../generated.typ": *

#let main-header = (
  [scheme], [cov.], [cost\_norm], [revenue\ \_norm], [skill], [eff.\ classes],
  [mean\ depth], [max\ depth], [max\ fan-out], [Gini], [nav\_log\ ($a=1$)],
)
#let main-align = (left,) + (right,) * 10
#let result-table(rows) = tbl(main-header, rows, align: main-align, size: 7.4pt)

= Results <sec:results>

Every table in this section is generated from the committed result files
(@sec:repro). Placed rows are sorted by $"cost"_"norm"$, best first. All
scores are on the graphs of @sec:graph; skill is relative to the
average-linkage reference.

== Gold placements <sec:gold>

#wide-table(
  result-table(gold-corpus),
  [Gold placements on the arXiv corpus. The `msc` row is the corrected gold
   run (see text); `arxiv` and `jel` are from the original run.],
  label: <tab:gold-corpus>,
)

#wide-table(
  result-table(gold-holdout),
  [Gold placements on the book holdout. LC call numbers are resolved to the
   deepest covering class.],
  label: <tab:gold-holdout>,
)

arXiv's own taxonomy, applied to arXiv papers by their authors, reaches skill
#v.gold_arxiv_skill with full coverage (@tab:gold-corpus). This is the
highest score in the study. We read it as an upper reference for what a
well-fitted scheme with correct placements achieves on this graph, not as
evidence that arXiv's taxonomy is a good _universal_ scheme. The arXiv
categories and the abstracts come from the same authors, and the corpus was
sampled by arXiv group.

On the books, LCC call numbers resolved into the outline reach skill
#v.gold_lcc_skill and into the complete schedules #v.gold_lccfull_skill, at
the same coverage (#v.gold_lcc_cov, @tab:gold-holdout). The complete
schedules use more effective classes (#v.gold_lccfull_eff against
#v.gold_lcc_eff) and are deeper (mean depth #v.gold_lccfull_depth against
#v.gold_lcc_depth). Yet cost barely moves (#v.gold_lccfull_cost against
#v.gold_lcc_cost). With 500 short titles and a holdout that is three quarters
class P, the title-embedding graph does not reward the finer LCC
distinctions. This does not mean the distinctions are useless; it is a limit
of what the graph can see.

MSC and JEL gold results are uninformative. With #v.gold_msc_cov and
#v.gold_jel_cov coverage, almost every pair involves an item under the root.
The MSC gold run was rescored after a correction of the corpus's `msc` field
(it had held markers and words from the `msc-class` text) and of the MSC
normalizer, which now accepts section and form codes such as `65-02`. Coverage
moved from #v.gold_msc_cov_r1 to #v.gold_msc_cov and skill from
#v.gold_msc_skill_r1 to #v.gold_msc_skill. The corrected run is reported here.
ACM CCS still has no gold run. The corpus's `acm_ccs` field originally held
the MSC-style `msc-class` text copied from arXiv's ACM-class field. After the
correction it holds 690 ACM CCS 1998 codes such as `I.2.6` on 317 papers, but
the scheme is CCS 2012, whose codes are paths of 8-digit concept IDs, and no
1998-to-2012 crosswalk is available (@sec:survey). The run log records the
run as skipped for that reason.

== Placement by the lexical classifier <sec:placed>

#wide-table(
  result-table(placed-corpus),
  [Placed by shelfmark (`profile:lexical@2`) on the arXiv corpus.],
  label: <tab:placed-corpus>,
)

#wide-table(
  result-table(placed-holdout),
  [Placed by shelfmark on the book holdout (general schemes only).],
  label: <tab:placed-holdout>,
)

#wide-table(
  tbl(
    ([scheme], [set], [placed], [flagged\ (\<0.8)], [unplaced], [top class], [top\ share], [top matched term]),
    placement-stats,
    align: (left, left, right, right, right, left, right, left),
    size: 7.2pt,
  ),
  [Placement statistics for the `lexical@2` runs. "Flagged" placements are
   below the 0.8 confidence target but kept. "Unplaced" counts fallbacks. "Top
   matched term" is the vocabulary term that sent items to the top class.],
  label: <tab:placement>,
)

On the corpus, skill under classifier placement runs from #v.placed_worst_skill
(#v.placed_worst) to #v.placed_best_skill (#v.placed_best). Second is
#v.placed_second at #v.placed_second_skill. Coverage explains much of the
order: across the 13 schemes the Spearman correlation between coverage and
skill is #v.rho (@fig:cov). Among the #v.n_highcov schemes with at least 95%
coverage, skill still spreads from #v.highcov_min to #v.highcov_max, so
coverage does not explain everything. Within that group, OpenAlex Topics
scores best. It was built from research literature, the same kind of text as
the corpus @openalextopics. EuroVoc, a policy and legal thesaurus, scores
lowest. That fits the expectation that a vocabulary matched to the corpus
domain organizes it better. But @tab:placement shows the classifier produces
many low-confidence placements: in #v.n_flagged90 of the 13 corpus runs, over
90% of placed items are flagged. The classifier's reach is itself part of the
score (@sec:bias).

#figure(
  {
    let pts = placed-corpus.map(r => {
      let cov = float(r.at(1).replace("%", ""))
      let sk = float(r.at(4))
      let lab = r.at(0)
      let dy = 0pt
      if lab == "ilc2" { dy = -3pt }
      if lab == "lcc-full" { dy = 4pt }
      if lab == "bk" { dy = 4pt }
      if lab == "jel" { dy = -4pt }
      let anchor = if lab == "acm-ccs" { "l" } else { "r" }
      (lab, (cov - 30) / 90, sk / 0.46, anchor, dy)
    })
    scatter(
      pts,
      height: 4.8cm,
      xlabel: [coverage (%)],
      ylabel: [skill],
      xticks: ((str(40), 10 / 90), (str(60), 30 / 90), (str(80), 50 / 90), (str(100), 70 / 90)),
      yticks: (([0.0], 0), ([0.1], 0.1 / 0.46), ([0.2], 0.2 / 0.46), ([0.3], 0.3 / 0.46), ([0.4], 0.4 / 0.46)),
    )
  },
  caption: [Coverage and skill for the 13 schemes placed by the lexical
    classifier on the arXiv corpus (data of @tab:placed-corpus).],
  placement: auto,
  kind: image,
  supplement: [Figure],
) <fig:cov>

The holdout results are near zero for every scheme except `lcc-full` and
`ilc2` (@tab:placed-holdout). On titles of 3.5 words, the lexical classifier
finds a candidate for only #v.placed_bk_holdout_cov (BK) to
#v.placed_lccfull_holdout_cov (complete LCC) of the books.

== How large is the classifier effect? <sec:bias>

@tab:bias compares gold and classifier placements for the same scheme on the
same items and graph.

#col-table(
  tbl(
    ([scheme], [set], [gold\ cov.], [placed\ cov.], [gold\ skill], [placed\ skill], [placed\ $-$ gold]),
    bias,
    align: (left, left, right, right, right, right, right),
    size: 7pt,
  ),
  [Gold against classifier placement on identical items and graph.],
  label: <tab:bias>,
)

Where gold coverage is high (arXiv, LCC), the classifier loses most of the
skill. For arXiv, skill falls from #v.gold_arxiv_skill to
#v.placed_arxiv_skill. The classifier places only #v.placed_arxiv_cov of the
papers and sends 39.1% of those to `cs.PF` (Performance) on the word
"performance" (@tab:placement). For the LCC outline on books, skill falls
from #v.gold_lcc_skill to #v.placed_lcc_holdout_skill. Where gold coverage is
tiny (MSC, JEL), the classifier scores _higher_ than the gold labels, only
because it places more items. Neither direction measures scheme quality. In
this run, the classifier, not the scheme, accounts for most of the
differences in classifier-placed skill. The ranking in @tab:placed-corpus is
therefore a ranking of _scheme as reached by this classifier_. We do not
present it as a ranking of schemes.

== Navigation <sec:nav>

#wide-table(
  tbl(
    ([scheme], [labels], [set], [p50\ depth], [p95\ depth], [root\ fan-out], [nav\_log\ $a=0$], [$a=1$], [$a=2$], [nav\_scan]),
    navigation,
    align: (left, left, left, right, right, right, right, right, right, right),
    size: 7pt,
  ),
  [Depth and navigation cost under uniform use (items in the root menu
   included). $a$ is the per-level overhead of @eq:navlog.],
  label: <tab:nav>,
)

The navigation models answer a different question from cost: how much
reading it takes to reach an item, not whether related items are near each
other. With unplaced items in the root menu, the two models diverge. $"nav"_"log"$
_drops_ as coverage falls, because one large root menu costs only $log_2 b$.
$"nav"_"scan"$ rises steeply, because the root must be scanned linearly. Among
high-coverage runs, the complete LCC schedules have the lowest scan cost
($"nav"_"scan"$ #v.lccfull_navscan on the corpus) and the most effective
classes. This is the deep, narrow-shelf trade-off: more levels to descend,
but short shelves at the bottom. The models' parameters are assumptions, so
@tab:nav shows the trade-off and does not settle it. The menu studies cited
in @sec:related disagree on the best depth and breadth @miller1981
@snowberry1983 @larson1998, so they do not fix the parameters either.
