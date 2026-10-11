#import "../template.typ": *
#import "../generated.typ": *

= Books with Dewey numbers: Dewey against LCC <sec:books>

A third file holds #v.x_books_n Open Library editions that each carry an LC
call number, a Dewey number and subjects (#v.x_books_desc also have a
description). The 21 LCC main classes have equal quotas, and no edition shares
a work with the 500-book holdout. The graph text is the description followed
by up to 30 subject headings, so it is mostly Library of Congress subject
headings, and $tau = #v.x_books_tau$. Dewey numbers are used as notation
only: the tree is the digit prefixes, and no Dewey caption, schedule or table
is read (@sec:limits). Every item has both a Dewey number and an LC call
number from librarians, so the Dewey and LCC runs are gold on the same items
and coverage is 1.000 for both. @tab:books gives the runs on the MiniLM
$k = 20$ graph. @tab:claims gives the comparisons that a book-structure
proposal drew from them, rechecked in each of #v.x_dl_settings settings (three
embedding models, three values of $k$ and three graph thresholds, with
#v.x_bs_resamples paired item-bootstrap draws each).

#wide-table(
  tbl(
    book-main-header,
    book-main,
    align: (left, left, right, right, right, right, right, right),
    size: 7pt,
  ),
  [Books, MiniLM, $k = 20$, published graph. $C$ is `cost_norm`; FJs enters at
   $beta = 1$; $J'$ adds $0.002 N$.],
  label: <tab:books>,
)

#wide-table(
  tbl(
    book-claims-header,
    book-claims,
    align: (left, left, left, right, right, left, left),
    mono-first: false,
    size: 7pt,
  ),
  [Pair claims across #v.x_dl_settings settings. "Claimed" is the sign the
   proposal reported; "sign matches" counts settings with that sign;
   "significant" counts settings whose 95% interval excludes zero, for and
   against the claim. A tie claim fails when any interval excludes zero.],
  label: <tab:claims>,
)

Dewey sections beat LCC on cost on the published graph for all three
embedding models (#v.x_dl_c_minilm [#v.x_dl_c_minilm_lo, #v.x_dl_c_minilm_hi]
on MiniLM, #v.x_dl_c_bgesmall on bge-small, #v.x_dl_c_nomic on nomic, at
$k = 20$). Across the #v.x_dl_settings settings the sign matches in
#v.x_dl_c_match, with #v.x_dl_c_for significant for and #v.x_dl_c_against
significant against, and the difference ranges from #v.x_dl_c_min to
#v.x_dl_c_max. Under the corrected objective the claim becomes a tie, which
holds in #v.x_dl_jp_match settings with #v.x_dl_jp_against significant
departures. The worst-case regret that earlier favoured Dewey sections
(#v.x_regret_ddc9 against #v.x_regret_lcc9 for LCC over the nine published
graphs) reverses over all #v.x_dl_settings settings (#v.x_regret_ddc27
against #v.x_regret_lcc27). The non-LLM signals do not separate them either
(@sec:external): shared subject headings #v.x_ext_lcsh_diff
[#v.x_ext_lcsh_lo, #v.x_ext_lcsh_hi], and for papers shelved under an FDC or an
LCC backbone, coupling #v.x_ext_coupling_diff [#v.x_ext_coupling_lo,
#v.x_ext_coupling_hi] and co-citation #v.x_ext_cocitation_diff
[#v.x_ext_cocitation_lo, #v.x_ext_cocitation_hi]. We read this as a tie that
depends on the setting. The sample is stratified equally over LCC's main
classes, which favours LCC's false-join term, and its text is mostly LC subject
headings.

Smaller comparisons are stable in every setting. A three-digit Dewey section
beats FDC depth 3 by less than #v.x_sec_fdc_max of cost, a
caption-versus-number difference. Three levels beat two. The full Dewey number
gains cost and loses on $J'$. Through the same classifier, FDC beats the LCC
outline without subject-heading vocabulary in all #v.x_dl_settings settings.
It does not beat the bundled LCC outline with LCSH vocabulary, which places
#v.x_floor_cov_lcclcsh of the books against #v.x_floor_cov_fdc for FDC: that
comparison turns on vocabulary and coverage, not on the scheme. On the
500-book title-only holdout no placed run reaches the coverage floor of
#v.x_floor_floor, and the best reaches #v.x_hold_best_cov.

*The default scheme.* The tool needs one default scheme for a general
library of mostly books, filed by its classifier alone. Its decision record
states a rule: among placed runs whose coverage is at least a floor, take the
highest skill; every run whose paired difference to it has a 95% interval
covering zero is tied; the tied run with the lowest navigation cost wins. The
floor is #v.x_def_floor, so that at most one book in ten waits for a person.
@tab:default gives the winners in the #v.x_def_settings settings. At the
#v.x_def_floor floor the LCC outline with LCSH headings as search vocabulary
(`lcc-lcsh`) wins all #v.x_def_wins of #v.x_def_settings settings among the
open schemes the tool may bundle (FDC, the LCC outline with and without LCSH
vocabulary, LCC full, BK and the fitted scheme), and it is the only winner at
every floor from #v.x_def_min_floor_unique up. Over all placed schemes it wins
#v.x_def_all_lcclcsh and the UNESCO Thesaurus, which may not be bundled,
#v.x_def_all_unesco. Part of the win is vocabulary: LCSH lifts the outline's
coverage from #v.x_def_cov_lcc to #v.x_def_cov_lcclcsh, while FDC places
#v.x_def_cov_fdc with its captions alone. The bundled default of the tool is
this scheme. The decision record also documents a catalogue-first
alternative, FDC cut to Dewey sections, which rests on the gold-number tie
above and on openness and notation length rather than on a metric result.

#col-table(
  tbl(
    default-floors-header,
    default-floors,
    align: (right, left, left),
    mono-first: false,
    size: 7pt,
  ),
  [Winner of the default-scheme rule on the books: settings won out of
   #v.x_def_settings, by coverage floor, over all placed schemes and over the
   open schemes the tool may bundle.],
  label: <tab:default>,
)
