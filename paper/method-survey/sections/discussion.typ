#import "../template.typ": *
#import "../generated.typ": v

= Discussion <sec:discussion>

*What the measurement supports.* The method gives one scale for schemes of
different size, depth and domain. It costs little: the average-linkage
reference took #v.c_ref_seconds_corpus s on the corpus graph and
#v.ref_holdout_seconds s on the holdout graph, and no scoring run took more
than #v.c_score_seconds_max s. Its inputs are pinned by hash. On gold labels
it separates a well-fitted scheme (arXiv on arXiv) from a general scheme on
books (LCC on the holdout) and from schemes with too little gold to judge
(MSC, JEL). Within one classifier, the order of schemes on the papers does not
depend on the embedding model, $k$ or graph threshold (@sec:rankstab). Several
book-side comparisons are stable in every setting we tried: the cut depth of a
Dewey-derived scheme, and the full number against its three-digit section
(@sec:books).

*What it does not support.* It does not support a ranking of universal
schemes, for four reasons that each have a number behind them.

+ _Coverage._ Classifier placement is the only way to reach most schemes, and
  coverage explains #v.x_cov_rho_sq of the rank variance of cost over the
  placed runs (@sec:coverage). Gold placement does not escape this: the MSC
  and JEL gold scores measure how few papers carry a code.
+ _The classifier._ The order of schemes changes when the classifier does: the
  Spearman correlation between the lexical and shortlist tiers' skill is
  #v.x_sem_rho, and the leader changes from #v.x_stab_lead_lexical to
  #v.x_stab_lead_shortlist (@sec:tier, @sec:rankstab). The stopword defect shows
  the same risk from the other side: a classifier defect can change placements
  drastically while skill barely moves, or moves it the wrong way
  (@sec:stopword).
+ _The objective._ FDC ranks above LCC under triplet accuracy in
  #v.x_stab_triplet_fdc_wins of #v.x_stab_settings settings and under cost in
  #v.x_stab_cost_fdc_wins. Dewey sections against LCC on books is a
  setting-dependent tie: #v.x_dl_c_match of #v.x_dl_settings settings match the
  claimed sign under cost, and #v.x_dl_jp_match the tie under the corrected
  objective.
+ _The metric's validity._ The cost agrees with a blind judge at
  $kappa = #v.x_val_kappa$. The false-join correction raises that to
  #v.x_val_kappa_fj against the judge and not against citation or
  subject-heading signals, and the judge is one model (@sec:validity).

A scheme's rank on this metric is therefore a statement about a similarity
graph, a classifier and an objective. It is not a statement about the
scheme's quality in use. The default scheme follows a stated rule
rather than a ranking: the LCC outline with LCSH vocabulary wins it in
#v.x_def_wins of #v.x_def_settings book settings among the open schemes, and
the win is partly a vocabulary effect (@sec:books). On librarian-assigned
numbers the metrics do not separate Dewey from LCC.

*Toward a fair comparison.* The benchmark design lays out controls. A coverage
control, a second classifier tier, an embedding, $k$ and threshold sweep, item
bootstrap intervals and a judge-based validity check have now been run. A
permutation null that keeps each scheme's shape and class-size histogram, a
label-intrinsic track that needs no works, a noise-injection curve that
calibrates classifier error on LCC, and placement by a classifier other than
shelfmark's (Annif or a prompted LLM) have not. The largest remaining gain
would come from a second annotator model and a human annotator on the
validity sample, and from a graph that is not built from embeddings (co-use
or citations).

*Interpretation of skill.* Skill compares a scheme with a tree fitted to the
same similarity graph. A scheme encodes many relations besides textual
similarity: discipline, form, audience, place, period. A low skill can mean
the scheme organizes by something the embedding does not see. Our score
measures agreement with one notion of relatedness. It does not measure
quality in every sense that matters to libraries.

*On the fitted scheme.* A scheme fitted to the benchmark's own objective
scores well on that objective, as it must. The result is a floor for how low
the cost can go on a given graph, not a ranking: its lead is partly home
advantage and is not stable across settings (@sec:learned). Whether an open,
learned, order-preserving classification can become a general one is open.

= Limitations <sec:limits>

- *Classifier bias.* Every non-gold placement comes from shelfmark's own
  classifier: the lexical profile for the main tables and a BGE-small
  shortlist profile for @sec:tier. The lexical profile matches class labels
  and attached vocabulary, so a scheme with rich vocabulary is reached better
  than one with bare labels, independent of its structure. Most placements are
  below the classifier's own confidence target (@tab:placement). @tab:bias and
  @sec:tier show the classifier effect is as large as the effects we want to
  measure. No classifier from outside this project (for example Annif or an
  LLM) was run for comparison. The semantic tier's rerank profile is still a
  stub, so only the shortlist profile was measured.
- *One annotator for validity.* The validity check used one LLM judge, on
  #v.x_val_n pairs from #v.x_val_runs runs in three strata, so its shares
  describe the sample and not the population of pairs. The judge is
  self-consistent ($kappa = #v.x_val_self_kappa$), but self-consistency is not
  validity. There is no human annotator and no second model. The non-LLM
  signals (subject headings, citations) are partial, and the citation signal
  is thin for recent preprints.
- *Science-heavy corpus.* #v.c_share_physical_math_cs of the corpus is
  computer science, mathematics or physics. Humanities, law, arts and general
  nonfiction appear only in the 500-record holdout, which is itself
  #v.c_holdout_p_share class P, and in the sample of #v.x_books_n books, whose
  graph text is mostly LC subject headings. Domain schemes (arXiv, MSC, ACM
  CCS) are scored on their home ground and general schemes are not, so no
  cross-type comparison is fair.
- *Embedding as the only similarity.* The graph is always built from a text
  embedding. The sweep covers three models and three values of $k$, and a
  similarity from citations, co-use or subject headings could reorder
  results. The fitted scheme of @sec:learned is the most exposed to this
  circularity.
- *Low gold coverage for MSC and JEL.* Only #v.gold_msc_cov of papers carry a
  valid MSC code and #v.gold_jel_cov a JEL code. Unplaced items hang under the
  root, so these gold scores measure coverage, not structure. The
  covered-subset analysis (@sec:coverage) is the control, and its subset
  scores use a restricted graph.
- *ACM CCS gold missing, and no official crosswalk.* The corpus carries CCS
  1998 codes, the scheme is CCS 2012, and ACM publishes no 1998-to-2012
  mapping: it says it built one for its digital library and exposes it only
  as terms on each article's page, and the published CCS 2012 SKOS file holds
  no mapping or 1998 codes (@sec:survey). The gold run was skipped, and we
  did not build a mapping, because any we made would be ours and not ACM's.
- *Scheme licensing and the DDC position.* Several schemes cannot be
  redistributed: ACM CCS (educational and research use only), JEL (no open
  licence), MSC (non-commercial, share-alike), EuroVoc (terms stated
  inconsistently) and ILC2 (conflicting licence statements) (@tab:survey).
  Their files are built locally from the publishers' downloads and verified
  by hash, but are not published. Exact reproduction therefore depends on the
  upstream downloads staying available and unchanged. DDC is owned by OCLC
  @deweyrights. We use Dewey class _numbers_ from public catalogue records,
  and the tree is the notation's digit prefixes. No Dewey caption, schedule,
  table, Manual entry or relative-index term from any edition is used, and
  nothing is taken from OCLC services. The position rests on class numbers
  being short numeric strings in public catalogue data. It is an engineering
  position, not legal advice, and it is written for the United States and the
  European Union only. The Dewey-derived scheme that can be bundled and used
  to _place_ items is FDC @fdc. Even LCC's open status depends on
  jurisdiction (@sec:survey).
- *Titles-only holdout.* The book holdout has no descriptions. Its similarity
  graph is built from titles averaging #v.c_holdout_title_words words, and the
  classifier finds candidates for few of them. The holdout is also small
  ($n = 500$). The main tables give point values; intervals for the placed
  runs are in the bootstrap runs of the covered and semantic result
  directories.
- *Navigation mixes coverage with structure.* Unplaced items sit under the
  root, so low coverage inflates root fan-out and every navigation cost.
  Navigation columns must be read next to coverage (@tab:placement).
- *Method limits.* The $k$NN-sparsified cost agrees with a dense cost on
  random subsets (@sec:coverage) but has not been checked on the full corpus.
  Polyhierarchies are reduced to one parent per class. Only one placement per
  item is used, although 35.5% of a 200-work Open Library sample of English
  documents had more than one LCC value. The balance statistics count each
  item at every ancestor class (@sec:structure). The navigation models are
  unfitted. The sample of #v.x_books_n books is stratified equally over LCC's main
  classes, which favours LCC's false-join term.
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
