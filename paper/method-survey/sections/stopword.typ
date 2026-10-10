#import "../template.typ": *
#import "../generated.typ": *

= A methods note: the stopword defect <sec:stopword>

The first placed run (`profile:lexical@1`) showed single-class captures. The
classifier's vocabulary tier matched complete class labels and terms, and some
of them consist only of English function words. ILC2 has a class labelled
"to", IPC a C01G catchword "TO" and the complete LCC a class `PL2813`
labelled "To". Any abstract containing "to" matched them. The fix
(classifier `vocab-2`, `profile:lexical@2`) drops normalized labels and
terms whose tokens are all stopwords. Labels with at least one content word
still match in full. We then reran only the placed part, on the same graphs,
scheme files (byte-identical, checked by SHA-256) and reference costs. The
gold results do not depend on the classifier and were not rerun on this
account. Every placed number in this paper comes from the rerun.

#wide-table(
  tbl(
    ([scheme], [set], [top class before], [top class after], [coverage], [cost\_norm], [skill], [$Delta$ skill]),
    stopword,
    align: (left, left, left, left, right, right, right, right),
    size: 6.8pt,
  ),
  [Effect of the stopword fix (before: `lexical@1`; after: `lexical@2`).
   "Top class" gives the largest single-class share of placed items and its
   code.],
  label: <tab:stopword>,
)

The fix removed the captures (@tab:stopword). ILC2's largest class fell from
#v.sw_ilc2_corpus_before to #v.sw_ilc2_corpus_after of placed papers, IPC's
from #v.sw_ipc_corpus_before to #v.sw_ipc_corpus_after, complete LCC's from
#v.sw_lcc_full_corpus_before to #v.sw_lcc_full_corpus_after and the UNESCO
Thesaurus's from #v.sw_unesco_thesaurus_corpus_before to
#v.sw_unesco_thesaurus_corpus_after. Skill moved much less. On the corpus,
#v.sw_changed of 13 schemes changed skill by at least 0.0005, and Kendall's
$tau_b$ between the skill values before and after is #v.sw_kendall. Complete
LCC _lost_ skill (#v.sw_lcc_full_corpus_skill_before $arrow.r$
#v.sw_lcc_full_corpus_skill_after) even though its placements became
plausible. A plausible explanation, not tested here, is that the wrong
placements had piled most papers that share "to" into a single class. Within
such a pile, the edges the graph keeps still connect similar papers, and the
pile is far smaller than the whole corpus, so its edges cost less than they
would under the root. A metric can reward a degenerate placement more than a
spread of honest but noisy placements. This is a known property of a cost
that charges same-class pairs the class size: one large class still beats the
root. It is also why we always report largest-class share and placement
statistics next to skill.
