# Report 8: Systematic search for prior work on quantitative comparison of classification schemes

Date of search: 2026-10-10.

## 1. Purpose

paper/paper1/paper.md (Abstract, Section 1 and Section 7) claims that no published study scores several universal or
near-universal library classification schemes on one quantitative objective over the same set of works. Section 7
concedes that the search behind the claim (research/reports/1-4) was not systematic, and Section 1 carries two
`TODO(cite)` markers: one for a knowledge-organization survey of scheme evaluation criteria, one for a systematic
search of JASIST, Knowledge Organization and ISKO proceedings. Section 5.4 has a third marker for menu depth/breadth
user studies. This report records a reproducible search that addresses all three and tests the novelty claim.

Two questions were searched:

- (a) quantitative comparisons of library or bibliographic classification schemes against each other (structure,
  depth, breadth, balance, coverage, scatter, retrieval performance, "classification scheme evaluation", "hierarchy
  quality");
- (b) hierarchy or taxonomy quality objectives applied to knowledge organization systems (KOS): Dasgupta-type tree
  costs, structural metrics of KOS, and depth/breadth trade-offs in browsing hierarchies.

Secondary targets: a citable source for the history of LCC and DDC, a KO survey of evaluation criteria, and the menu
depth/breadth studies (Miller 1981, Kiger 1984, Larson and Czerwinski 1998), to be verified, not assumed.

## 2. Method

### 2.1 Sources and venue filters

All bibliographic searches used the Crossref REST API (`https://api.crossref.org/works`) with
`query.bibliographic=<query>`, `rows=30-200`, `select=DOI,title,author,issued,container-title,type`, run with curl and
a small Python wrapper on 2026-10-10. Raw JSON for every query was kept locally (not committed). ISSNs were looked up
with `https://api.crossref.org/journals?query=...` on the same day.

| Label | Venue | Crossref restriction |
| --- | --- | --- |
| F-JASIS | Journal of the American Society for Information Science (1970-2000), JASIS&T (2000-2013), JASIST (2014-) | `filter=issn:0002-8231,issn:1097-4571,issn:1532-2882,issn:1532-2890,issn:2330-1635,issn:2330-1643` (Crossref holds 3,857 / 2,895 / 2,334 records under the three title runs) |
| F-AmDoc | American Documentation (predecessor of JASIS, 1950-1969) | `filter=issn:0096-946X` |
| F-KO | Knowledge Organization (incl. its predecessor title International Classification, 1974-1992, which Crossref files under the same ISSN) | `filter=issn:0943-7444,issn:2942-3309` (2,034 records, 1974-2026) |
| F-CCQ | Cataloging & Classification Quarterly | `filter=issn:0163-9374,issn:1544-4554` (2,361 records) |
| F-ISKO | ISKO international conference proceedings (Ergon/Nomos "Advances in Knowledge Organization" volumes 2012, 2016, 2018, 2019, 2020, 2022, 2024 present in Crossref) | `filter=prefix:10.5771,type:book-chapter`, then a post-filter on container-title `/Knowledge Organi[sz]ation\|Human Position in an Artificial World/` |
| F-Nomos | same as F-ISKO without the post-filter (first five ISKO queries; returned many non-ISKO Nomos/Rowman book chapters) | `filter=prefix:10.5771,type:book-chapter` |
| F-ACRO | Advances in Classification Research Online (ASIS&T SIG/CR) and NASKO proceedings (University of Washington Libraries) | `filter=prefix:10.7152` |
| F-JCDL | ACM/IEEE Joint Conference on Digital Libraries | `query.container-title=Joint Conference on Digital Libraries`, rows=100, post-filter container-title `/joint conf[a-z]* on digital librar\|JCDL/` |
| F-TPDL | TPDL and its predecessor ECDL (Springer LNCS "Research and Advanced Technology for Digital Libraries", "Theory and Practice of Digital Libraries", "Linking Theory and Practice of Digital Libraries", "Digital Libraries for Open Knowledge") | `query.container-title=Research and Advanced Technology for Digital Libraries Theory and Practice of Digital Libraries`, rows=100, post-filter container-title `/Research and Advanced Technology for Digital Libraries\|Theory and Practice of Digital Libraries\|Digital Libraries for Open Knowledge/` |
| none | Crossref without venue filter, used for known-item lookups | - |

Other sources:

- ISKO Encyclopedia of Knowledge Organization, index page https://www.isko.org/cyclo/ (HTTP 200, fetched with curl). It
  has entries on DDC, Library classification (part 1, premodern only), Hierarchy, KOS, Classification, Literary
  warrant and others, but no entry on LCC or UDC and no entry on KOS evaluation. The entries ddc, library_classification,
  hierarchy, kos, classification and literary_warrant were downloaded and searched for "evaluat", "compar",
  "quantitative", "empirical" and founding dates.
- OpenAlex (`https://api.openalex.org/works/doi:<doi>`) for abstracts missing from Crossref; arXiv API for two
  abstracts; publisher or repository landing pages and PDFs where open (IMR Press for Knowledge Organization, ACRL for
  College & Research Libraries).
- Web search (five queries, Section 2.3) for coverage outside the indexed venues.

### 2.2 Screening procedure

For each query, every returned title was read (for F-JCDL, F-TPDL and F-ISKO, every title surviving the venue
post-filter). For JC1-JC9 the in-venue titles were first narrowed with a case-insensitive title regex
(`classif|taxonom|hierarch|dewey|congress|brows|navig|knowledge org|thesaur|ontolog|subject|categor|menu|depth|cluster`)
and the matches read; K6-K11 dropped untitled items (book-review and "Classification Literature" bibliography
records). Candidates were then fetched individually from `https://api.crossref.org/works/<doi>`; title, authors,
year, venue, volume, issue and pages below are copied from those records. The abstract was read from the Crossref
`abstract` field, or from OpenAlex, arXiv or the first page of the open PDF where stated. Items were classified:

- **A**: directly comparable prior work: quantitative comparison of several schemes on a common measure;
- **B**: related: quantitative structural or quality measures for a single scheme or KOS, scheme-to-scheme mappings,
  evaluation frameworks, or descriptive comparisons that bear on the claim;
- **C**: not relevant (automatic text classification, cataloguing codes, OPAC interfaces, bibliometrics unrelated to
  classification structure, book reviews, editorials).

Totals: 94 Crossref queries; 2,636 title rows screened (with duplicates across queries; distinct candidates fetched
individually: 94). Hit counts are Crossref `total-results`, which for relevance-ranked `query.bibliographic` counts
any record matching any query word; within one journal it is often the size of the journal (all seven CCQ queries
report 2,361-2,362). Only the top-ranked rows were screened, so "screened" is the meaningful number.

### 2.3 Web searches (2026-10-10)

| # | Query | Result |
| --- | --- | --- |
| W1 | `"Retrieval tests on five classification schemes" Carpenter Jones Oppenheim` | found the IMR Press landing page and PDF for Carpenter et al. 1978 |
| W2 | `Library of Congress Classification history 1897 J.C.M. Hanson Charles Martel developed site:loc.gov` | no loc.gov page returned; only secondary sources (Wikipedia, LIS blogs), not used |
| W3 | `quantitative comparison Dewey Decimal Classification Library of Congress Classification Universal Decimal Classification structure depth balance same collection` (extended) | only Zins and Santos 2011 (already found) and general web pages |
| W4 | `"Dasgupta cost" evaluate existing taxonomy OR ontology OR "classification scheme" human-built hierarchy` (extended) | no use of Dasgupta cost on a human-built scheme; returned taxonomy-evaluation preprints incl. the already cited wullschleger2025 |
| W5 | `library classification LCC OR DDC evaluated against document similarity OR citation network "classification" coherence empirical test collocation study` (extended) | no study testing LCC/DDC against document similarity or citation coherence |

### 2.4 Crossref search log (every query)

Venue filters are the labels of Section 2.1. All queries run 2026-10-10.

| ID | Venue | Filter / venue restriction | Query string (`query.bibliographic`) | Hits (Crossref total-results) | Screened (titles read) |
| --- | --- | --- | --- | --- | --- |
| J1 | JASIS-family | F-JASIS | comparison of library classification schemes | 599 | 40 |
| J2 | JASIS-family | F-JASIS | Dewey Decimal Classification Library of Congress Classification comparison | 597 | 40 |
| J3 | JASIS-family | F-JASIS | evaluation of classification schemes quantitative | 356 | 40 |
| J4 | JASIS-family | F-JASIS | Universal Decimal Classification structure analysis | 675 | 40 |
| J5 | JASIS-family | F-JASIS | hierarchy depth breadth classification structure measures | 269 | 40 |
| J6 | JASIS-family | F-JASIS | structural metrics knowledge organization systems thesaurus | 716 | 40 |
| J7 | JASIS-family | F-JASIS | menu depth breadth hierarchy navigation | 33 | 33 |
| J8 | JASIS-family | F-JASIS | hierarchical clustering objective taxonomy evaluation | 316 | 40 |
| J9 | JASIS-family | F-JASIS | document clustering compared with library classification | 680 | 40 |
| J10 | JASIS-family | F-JASIS | classification scheme collocation of similar documents | 229 | 40 |
| J11 | JASIS-family | F-JASIS | history of the Library of Congress Classification development | 679 | 30 |
| J12 | JASIS-family | F-JASIS | Dewey Decimal Classification history editions | 196 | 30 |
| J13 | JASIS-family | F-JASIS | evaluation criteria for knowledge organization systems | 847 | 30 |
| J14 | JASIS-family | F-JASIS | Dasgupta cost hierarchical clustering | 109 | 30 |
| J15 | JASIS-family | F-JASIS | balance of classification hierarchy number of classes per level | 208 | 30 |
| J16 | JASIS-family | F-JASIS | browsing hierarchy subject categories depth versus breadth user study | 746 | 30 |
| K1 | Knowledge Organization | F-KO | comparison of classification schemes | 281 | 40 |
| K2 | Knowledge Organization | F-KO | Dewey Decimal Classification Library of Congress Classification Universal Decimal Classification comparison | 282 | 40 |
| K3 | Knowledge Organization | F-KO | evaluation of classification systems quantitative measures | 353 | 40 |
| K4 | Knowledge Organization | F-KO | structure of classification schemes depth breadth hierarchy statistics | 304 | 40 |
| K5 | Knowledge Organization | F-KO | quality of knowledge organization systems evaluation criteria | 2034 | 40 |
| K6 | Knowledge Organization | F-KO | Library of Congress Classification history development | 345 | 40 |
| K7 | Knowledge Organization | F-KO | graph theoretic measures of classification hierarchy | 293 | 40 |
| K8 | Knowledge Organization | F-KO | bibliometric analysis of class sizes in classification schedules | 365 | 40 |
| K9 | Knowledge Organization | F-KO | menu browsing hierarchy depth breadth navigation | 14 | 14 |
| K10 | Knowledge Organization | F-KO | clustering of documents compared with library classification | 329 | 40 |
| K11 | Knowledge Organization | F-KO | classification scheme coverage collection literary warrant measurement | 286 | 40 |
| C1 | CCQ | F-CCQ | comparison of classification schemes | 2362 | 40 |
| C2 | CCQ | F-CCQ | Dewey Decimal Classification Library of Congress Classification comparison | 2362 | 40 |
| C3 | CCQ | F-CCQ | evaluation of classification schemes quantitative | 2362 | 40 |
| C4 | CCQ | F-CCQ | structure of classification schedules depth breadth hierarchy | 2362 | 40 |
| C5 | CCQ | F-CCQ | Library of Congress Classification history development | 2362 | 40 |
| C6 | CCQ | F-CCQ | Dewey Decimal Classification history editorial | 2362 | 40 |
| C7 | CCQ | F-CCQ | classification scheme browsing shelf collocation effectiveness | 2362 | 40 |
| JC0 | JCDL | F-JCDL without post-filter | classification scheme | 149705 | 40 |
| JC1 | JCDL | F-JCDL | comparison of library classification schemes | 195622 | 100 returned, 62 in venue |
| JC2 | JCDL | F-JCDL | Dewey Decimal Classification Library of Congress Classification | 147721 | 100 returned, 24 in venue |
| JC3 | JCDL | F-JCDL | evaluation of hierarchy taxonomy quality structure | 320995 | 100 returned, 34 in venue |
| JC4 | JCDL | F-JCDL | browsing hierarchy depth breadth navigation | 59667 | 100 returned, 43 in venue |
| JC5 | JCDL | F-JCDL | clustering documents compared with subject classification | 149869 | 100 returned, 63 in venue |
| JC6 | JCDL | F-JCDL | knowledge organization system structure metrics | 777610 | 100 returned, 60 in venue |
| T0 | TPDL/ECDL | F-TPDL | classification scheme | 93329 | 100 returned, 28 in venue |
| T1 | TPDL/ECDL | F-TPDL | comparison of library classification schemes | 235777 | 100 returned, 3 in venue |
| T2 | TPDL/ECDL | F-TPDL | Dewey Decimal Classification Library of Congress Classification Universal Decimal Classification | 172771 | 100 returned, 1 in venue |
| T3 | TPDL/ECDL | F-TPDL | evaluation of taxonomy hierarchy quality structure | 593672 | 100 returned, 8 in venue |
| T4 | TPDL/ECDL | F-TPDL | browsing hierarchy depth breadth navigation | 36223 | 100 returned, 25 in venue |
| T5 | TPDL/ECDL | F-TPDL | clustering documents compared with library classification | 152708 | 100 returned, 5 in venue |
| T6 | TPDL/ECDL | F-TPDL | knowledge organization system structure analysis | 1372042 | 100 returned, 1 in venue |
| T7 | TPDL/ECDL | F-TPDL | library classification | 118341 | 100 returned, 3 in venue |
| T8 | TPDL/ECDL | F-TPDL | subject hierarchy evaluation | 280013 | 100 returned, 24 in venue |
| T9 | TPDL/ECDL | F-TPDL | SKOS vocabulary quality metrics | 221704 | 100 returned, 26 in venue |
| T10 | TPDL/ECDL | F-TPDL | Dewey | 1250 | 100 returned, 1 in venue |
| JC7 | JCDL | F-JCDL | library classification | 131239 | 100 returned, 64 in venue |
| JC8 | JCDL | F-JCDL | subject hierarchy evaluation | 162401 | 100 returned, 44 in venue |
| JC9 | JCDL | F-JCDL | Dewey | 237 | 100 returned, 0 in venue |
| I1 | ISKO proceedings (Ergon/Nomos book chapters) | F-Nomos (no post-filter) | comparison of classification schemes | 514 | 40 |
| I2 | ISKO proceedings (Ergon/Nomos book chapters) | F-Nomos (no post-filter) | Dewey Decimal Classification Library of Congress Classification Universal Decimal Classification comparison | 180 | 40 |
| I3 | ISKO proceedings (Ergon/Nomos book chapters) | F-Nomos (no post-filter) | quantitative evaluation classification structure hierarchy | 1599 | 40 |
| I4 | ISKO proceedings (Ergon/Nomos book chapters) | F-Nomos (no post-filter) | quality evaluation knowledge organization systems criteria | 4640 | 40 |
| I5 | ISKO proceedings (Ergon/Nomos book chapters) | F-Nomos (no post-filter) | clustering documents compared with library classification | 1448 | 40 |
| I6 | ISKO proceedings | F-ISKO | comparison of classification schemes universal | 745 | 200 returned, 4 in venue |
| I7 | ISKO proceedings | F-ISKO | Dewey Decimal Classification Library of Congress Classification Universal Decimal Classification | 1870 | 200 returned, 50 in venue |
| I8 | ISKO proceedings | F-ISKO | evaluation of classification quantitative measures structure | 1772 | 200 returned, 4 in venue |
| I9 | ISKO proceedings | F-ISKO | browsing navigation hierarchy depth | 116 | 116 returned, 4 in venue |
| I10 | ISKO proceedings | F-ISKO | knowledge organization system evaluation quality | 5929 | 200 returned, 165 in venue |
| N1 | ACRO (ASIS&T SIG/CR) and NASKO | F-ACRO | comparison of classification schemes | 160 | 30 |
| N2 | ACRO (ASIS&T SIG/CR) and NASKO | F-ACRO | Dewey Decimal Classification Library of Congress Classification | 166 | 30 |
| N3 | ACRO (ASIS&T SIG/CR) and NASKO | F-ACRO | quantitative evaluation of classification structure | 162 | 30 |
| N4 | ACRO (ASIS&T SIG/CR) and NASKO | F-ACRO | browsing hierarchy navigation depth breadth | 6 | 6 |
| N5 | ACRO (ASIS&T SIG/CR) and NASKO | F-ACRO | evaluation of knowledge organization systems | 79 | 30 |
| X1 | any (Crossref) | none (general lookup) | Cleverdon comparative efficiency of indexing systems Cranfield | 6208680 | 15 |
| X2 | any (Crossref) | none (general lookup) | Cranfield tests on index language devices | 3262834 | 8 |
| M1 | any (Crossref) | none (general lookup) | Miller depth breadth tradeoff in hierarchical computer menus | 3180315 | 8 |
| M2 | any (Crossref) | none (general lookup) | Kiger depth breadth trade-off in the design of menu-driven user interfaces | 122530 | 6 |
| M3 | any (Crossref) | none (general lookup) | Larson Czerwinski web page design implications of memory structure and scent for information retrieval | 361353 | 6 |
| M4 | any (Crossref) | none (general lookup) | Landauer Nachbar selection from alphabetic and numeric menu trees touch screen breadth depth width | 62099 | 6 |
| M5 | any (Crossref) | none (general lookup) | Snowberry Parkinson Sisson computer display menus | 2819249 | 6 |
| M6 | any (Crossref) | none (general lookup) | Zaphiris depth vs breadth in the arrangement of web links | 1057229 | 6 |
| G1 | any (Crossref, no venue filter) | none (general lookup) | Wang Waltman large-scale analysis accuracy journal classification systems Web of Science Scopus | 2661578 | 8 |
| G2 | any (Crossref, no venue filter) | none (general lookup) | Shu Julien comparing journal and paper level classifications of science | 10702458 | 8 |
| G3 | any (Crossref, no venue filter) | none (general lookup) | Rafols Leydesdorff content-based and algorithmic classifications of journals indexer effects | 9999581 | 8 |
| G4 | any (Crossref, no venue filter) | none (general lookup) | Tartir OntoQA metric-based ontology quality analysis | 14147952 | 8 |
| G5 | any (Crossref, no venue filter) | none (general lookup) | Gangemi modelling ontology evaluation and validation | 3017338 | 8 |
| G6 | any (Crossref, no venue filter) | none (general lookup) | Golub framework for evaluating automatic indexing or classification in the context of retrieval | 2641655 | 8 |
| G7 | any (Crossref, no venue filter) | none (general lookup) | Owens Cochrane thesaurus evaluation | 2150868 | 8 |
| G8 | any (Crossref, no venue filter) | none (general lookup) | Losee relative shelf location of circulated books classification users browsing | 1544650 | 8 |
| G9 | any (Crossref, no venue filter) | none (general lookup) | Keen Digger report of an information science index languages test | 15328862 | 8 |
| G10 | any (Crossref, no venue filter) | none (general lookup) | Heller Ghahramani Bayesian hierarchical clustering dendrogram purity | 523384 | 8 |
| H1 | any (Crossref) | none (general lookup) | Library of Congress Classification Encyclopedia of Library and Information Sciences | 8803327 | 10 |
| H2 | any (Crossref) | none (general lookup) | history of the Library of Congress Classification Hanson Martel 1897 | 3588294 | 10 |
| H3 | any (Crossref) | none (general lookup) | Dewey Decimal Classification Encyclopedia of Library and Information Sciences Mitchell | 8564864 | 6 |
| J17 | JASIS-family (American Documentation) | F-AmDoc | comparison of classification schemes | 34 | 30 |
| J18 | JASIS-family (American Documentation) | F-AmDoc | comparative efficiency of indexing systems Universal Decimal Classification | 126 | 30 |
| J19 | JASIS-family (American Documentation) | F-AmDoc | evaluation of classification hierarchy structure | 55 | 30 |

## 3. Results

Of the 94 individually fetched records, 5 are classed A, 32 B and the rest C or used only as sources for specific
claims (Section 4). C items are not listed.

### 3.1 A: quantitative comparisons of several schemes on a common measure

None of these scores several *universal* schemes with a *structural* objective over a shared corpus. Each is
nonetheless a quantitative, same-material comparison of several schemes and should be acknowledged.

1. **Cleverdon, C.W. (1960).** The ASLIB Cranfield research project on the comparative efficiency of indexing systems.
   *Aslib Proceedings* 12(12): 421-431. doi:10.1108/eb049778. Crossref abstract: describes the origin of "a complete
   test of various indexing systems", with a National Science Foundation grant approved in July 1957. The systems tested in
   the first stage were confirmed from the full text of the review **Jahoda, G. (1961)**, *College & Research
   Libraries* 22(3): 228, doi:10.5860/crl_22_03_228 (open PDF read): the Universal Decimal Classification, an
   alphabetic subject catalogue, a faceted classification and a Uniterm coordinate index, applied to 18,000 aeronautics
   journal articles and reports, with retrieval to be tested on 1,600 questions. One universal scheme (UDC) against
   three other index languages; the measure is retrieval performance, not tree structure. The report itself
   (Cleverdon 1960, College of Aeronautics, 166 pp.) has no DOI and was not seen.
   Retrospective: **Cleverdon, C.W. (1991).** The significance of the Cranfield tests on index languages. *Proc. 14th
   ACM SIGIR*, 3-12. doi:10.1145/122860.122861 (no abstract in Crossref or OpenAlex; not read).
2. **Carpenter, A.M., Jones, M., Oppenheim, C. (1978).** Retrieval tests on five classification schemes. *Knowledge
   Organization* (then *International Classification*) 5(2): 73-80. doi:10.5771/0943-7444-1978-2-73. Open PDF from
   IMR Press, page 1 read: the UK Patent Classification, US Patent Classification, International Patent
   Classification, Derwent Manual Code and Derwent Punch Code were tested on the same patents in inorganic chemistry
   and biochemistry with 15 and 22 questions; recall and precision were computed; the Derwent codes gave substantially
   higher recall than the classification schemes. Domain (patent) schemes; retrieval measure.
3. **Connaway, L.S., Sievert, M.C. (1996).** Comparison of three classification systems for information on health
   insurance. *Cataloging & Classification Quarterly* 23(2): 89-104. doi:10.1300/j104v23n02_07. Abstract (OpenAlex):
   LCC, DDC and the NLM Classification were queried for materials on health insurance; hypotheses on the scatter of
   the three schemes and on the classes used where they overlap were tested; the schemes showed little overlap. Two
   general schemes and one domain scheme, one topic, scatter as the measure.
4. **Wang, Q., Waltman, L. (2016).** Large-scale analysis of the accuracy of the journal classification systems of Web
   of Science and Scopus. *Journal of Informetrics* 10(2): 347-364. doi:10.1016/j.joi.2016.02.003. Abstract (arXiv
   1511.00735): two citation-based criteria flag journals weakly connected to their assigned categories or missing
   from strongly connected ones; both systems are scored on the same criteria; Web of Science performs significantly
   better. Research (journal) classification systems, not library schemes, but the closest modern analogue in method:
   two schemes, one data-driven quality criterion, one corpus.
5. **Keen, E.M. (1973).** The Aberystwyth index languages test. *Journal of Documentation* 29(1): 1-35.
   doi:10.1108/eb026547. Crossref abstract: laboratory comparison of five index languages in library and information
   science (three post-coordinate, two pre-coordinate, including hierarchically structured ones) on effectiveness and
   efficiency, with eight test comparisons. Index languages built for the test, not existing universal schemes;
   retrieval measure. (Borderline A/B; listed here because it is a same-collection quantitative comparison.)

### 3.2 B: related work

**Quantitative work on LCC, DDC and UDC as schemes**

- **O'Neill, E.T., Dillon, M., Vizine-Goetz, D. (1987).** Class dispersion between the Library of Congress
  Classification and the Dewey Decimal Classification. *JASIS* 38(3): 197-205.
  doi:10.1002/(sici)1097-4571(198705)38:3<197::aid-asi8>3.0.co;2-2. Abstract (OpenAlex): two measures of the dispersion
  of one scheme's classes when mapped into another (a distance measure and a rank-frequency scatter measure), applied
  to the library-science portions of LCC and DDC using MARC records. This is the earliest quantitative LCC-DDC
  measurement found; it characterizes the mapping between two schemes, not the quality of either.
- **Zins, C., Santos, P.L.V.A.C. (2011).** Mapping the knowledge covered by library classification systems. *JASIST*
  62(5): 877-901. doi:10.1002/asi.21481. Abstract (OpenAlex): the subject classes in the first two levels of LCC, DDC
  and UDC are mapped onto the "10 Pillars of Knowledge" model, used "as an evaluative model"; the authors conclude the
  three systems are unsystematic, biased and incomplete at the top two levels. The nearest comparison of the three
  universal schemes against one common frame; it uses class labels, not a corpus, and no quantitative objective.
- **Hider, P., Coe, M. (2022).** Academic disciplines in the context of library classification: mapping university
  faculty structures to the DDC and LCC schemes. *CCQ* 60(2): 194-213. doi:10.1080/01639374.2022.2040675. Abstract
  (OpenAlex): faculty structures of ten Australian universities mapped to DDC and LCC; strong alignment, very few fields
  uncovered by either scheme. A coverage comparison of two schemes against one external structure.
- **Smiraglia, R.P. (2014).** Big classification: using the empirical power of classification interaction. *Advances
  in Classification Research Online* 24(1): 21. doi:10.7152/acro.v24i1.14673. Crossref abstract: nine million UDC
  numbers from WorldCat sampled and deconstructed; chi-squared associations with bibliographic record components.
  Single scheme, usage statistics.
- **Scharnhorst, A., Smiraglia, R.P. (2013).** Evolution of classification systems. *ACRO* 23(1): 56.
  doi:10.7152/acro.v23i1.14264. Crossref abstract: evolutionary analysis of UDC set against the Wikipedia category
  system. Single library scheme against a folksonomy.
- **Speretta, M., Gauch, S., Lakkaraju, P. (2008).** Computer classification system usage in CiteSeer. *Proc. 8th
  ACM/IEEE-CS JCDL*, 468. doi:10.1145/1378889.1379013. Abstract (OpenAlex): statistics on how ACM CCS categories cover
  CiteSeer papers, category size and growth. Single scheme coverage.
- **Osińska, V., Bala, P. (2010).** New methods for visualization and improvement of classification schemes: the case
  of computer science. *Knowledge Organization* 37(3): 157-172. doi:10.5771/0943-7444-2010-3-157; and **Osińska, V.
  (2010).** Visual analysis of classification scheme. *Knowledge Organization* 37(4): 299-306.
  doi:10.5771/0943-7444-2010-4-299. Abstracts (open PDFs, page 1): ACM CCS evaluated through co-class similarity maps
  over 37,543 ACM DL documents (353 classes), used for "content-related evaluation of the original scheme". Single
  scheme, corpus-based, visual rather than a scalar objective; the closest single-scheme analogue to the paper's use of
  a document-similarity structure.

**Bibliometric comparisons of research classification systems**

- **Rafols, I., Leydesdorff, L. (2009).** Content-based and algorithmic classifications of journals: perspectives on
  the dynamics of scientific communication and indexer effects. *JASIST* 60(9): 1823-1835. doi:10.1002/asi.21086.
  Crossref abstract: two algorithmic decompositions of the journal citation matrix tested against the ISI Subject
  Categories and the Glänzel-Schubert classification; algorithms maximize within-category over between-category
  citations.
- **Klavans, R., Boyack, K.W. (2017).** Which type of citation analysis generates the most accurate taxonomy of
  scientific and technical knowledge? *JASIST* 68(4): 984-998. doi:10.1002/asi.23734. Crossref abstract: accuracy of
  topic-level taxonomies from direct citation, bibliographic coupling and co-citation compared on a gold standard of
  papers with at least 100 references; direct citation concentrates references best. Several data-driven taxonomies
  on one criterion and one corpus.
- **Perianes-Rodriguez, A., Ruiz-Castillo, J. (2018).** The impact of classification systems in the evaluation of the
  research performance of the Leiden Ranking universities. *JASIST* 69(8): 1046-1053. doi:10.1002/asi.24017. Crossref
  abstract: WoS subject categories vs. algorithmic G8 and G6 systems; effect on university rankings.

**Structural and quality measures for a single KOS or hierarchy**

- **Julien, C.-A., Tirilly, P., Dinneen, J.D., Guastavino, C. (2013).** Reducing subject tree browsing complexity.
  *JASIST* 64(11): 2201-2223. doi:10.1002/asi.22915. Crossref abstract: LCSH and MeSH subject trees pruned using the
  uneven distribution of real collections; LCSH reduced to 49% of its complexity while keeping access to 83% of the
  collection, MeSH to 45% with 97%. A corpus-weighted structural measure of a subject hierarchy.
- **Losee, R. (2016).** Thesaurus structure, descriptive parameters, and scale. *JASIST* 67(9): 2156-2165.
  doi:10.1002/asi.23544. Crossref abstract: thousands of simulated thesauri; relationships among tree height, width,
  number of roots and vocabulary size, and how they scale.
- **Losee, R.M. (2017).** Improving collection browsing: small world networking and Gray code ordering. *CCQ* 55(4):
  229-246. doi:10.1080/01639374.2017.1292415. Abstract (OpenAlex): circulation data with LCC and LCSH ordering used to
  evaluate linear document arrangements for browsing.
- **Mader, C., Haslhofer, B., Isaac, A. (2012).** Finding quality issues in SKOS vocabularies. In *Theory and Practice
  of Digital Libraries* (TPDL 2012), LNCS, 222-233. doi:10.1007/978-3-642-33290-6_25. Abstract (arXiv 1206.1339):
  quantifiable quality issues formalized as checking functions (qSKOS) and run on 15 vocabularies; issues found in
  all. Several KOS on common, but defect-oriented, checks; no corpus.
- **Amirhosseini, M. (2010).** Theoretical base of quantitative evaluation of unity in a thesaurus term network based on
  Kant's epistemology. *Knowledge Organization* 37(3): 185-202. doi:10.5771/0943-7444-2010-3-185. Abstract (open PDF):
  ratio-based quantitative evaluation of thesaurus structure (counts of preferred, non-preferred and cross-reference
  terms), a tradition the abstract dates to 1976 (Lancaster).
- **Tartir, S., Arpinar, I.B. (2007).** Ontology evaluation and ranking using OntoQA. *ICSC 2007*, 185-192.
  doi:10.1109/icsc.2007.19. Abstract (OpenAlex): ranks ontologies by a tunable set of structural metrics.
- **Bergman, M.K. (2022).** Hierarchy in knowledge systems. *Knowledge Organization* 49(1): 40-66.
  doi:10.5771/0943-7444-2022-1-40 (also the ISKO Encyclopedia entry "Hierarchy"). Crossref abstract: survey of
  hierarchy in taxonomies, classification systems, thesauri and ontologies.

**Navigation and browsing in classification hierarchies**

- **Wang, Z., Khoo, C.S.G., Chaudhry, A.S. (2014).** Evaluation of the navigation effectiveness of an organizational
  taxonomy built on a general classification scheme and domain thesauri. *JASIST* 65(5): 948-963.
  doi:10.1002/asi.23017. Crossref abstract: scenario-based navigation exercises on a DDC-based taxonomy; errors traced to
  construction steps.
- **Koch, T., Golub, K., Ardö, A. (2006).** Users browsing behaviour in a DDC-based web service: a log analysis. *CCQ*
  42(3-4): 163-186. doi:10.1300/j104v42n03_07. Abstract (OpenAlex): Renardus logs; browsing dominates; extensive
  hierarchical browsing through the DDC.
- **McKay, D., Smith, W., Chang, S. (2014).** Lend me some sugar: borrowing rates of neighbouring books as evidence for
  browsing. *IEEE/ACM JCDL 2014*, 145-154. doi:10.1109/jcdl.2014.6970161. Abstract (OpenAlex): consortium circulation
  data show books are loaned in shelf clusters.
- **Krowne, A., Halbert, M. (2005).** An initial evaluation of automated organization for digital library browsing.
  *Proc. 5th ACM/IEEE-CS JCDL*, 246-255. doi:10.1145/1065385.1065442. Abstract (OpenAlex): clustering and
  classification methods for browse interfaces evaluated by user activity.
- **Lipetz, B.-A., Song, C.T. (1970).** How many cards per file guide? Optimizing the two-level file. *JASIS* 21(2):
  140-141. doi:10.1002/asi.4630210206. Crossref abstract: optimum records per guide in a two-level file is
  proportional to the square root of file size (exactly the square root when scan rates are equal). An early
  analytical breadth optimum for a browsing hierarchy, directly relevant to `nav_scan`.

**Evaluation frameworks and criteria**

- **Owens, L.A., Cochrane, P.A. (2004).** Thesaurus evaluation. *CCQ* 37(3-4): 87-102. doi:10.1300/j104v37n03_07.
  Abstract (OpenAlex): structural, formative, observational and comparative evaluation techniques for thesauri, with
  sources of evaluative literature.
- **Golub, K., Soergel, D., Buchanan, G., Tudhope, D., Lykke, M., Hiom, D. (2016).** A framework for evaluating
  automatic indexing or classification in the context of retrieval. *JASIST* 67(1): 3-16. doi:10.1002/asi.23600.
  Crossref abstract: reviews evaluation approaches and proposes a framework; argues against a single gold standard.
- **Gnoli, C., Ledl, A., Park, Z., Trzmielewski, M. (2018).** Phenomenon-based vs. disciplinary classification:
  possibilities for evaluating and for mapping. In *Challenges and Opportunities for Knowledge Organization in the
  Digital Age* (ISKO 2018), 653-661. doi:10.5771/9783956504211-653. Only the opening text (OpenAlex) was read: it
  frames the question of evaluating phenomenon-based schemes such as ILC against disciplinary ones.
- **Hjørland, B. (2025).** Dewey Decimal Classification (DDC). *Em Questão* 31, article e-145068.
  doi:10.1590/1808-5245.31.145068 (also the ISKO Encyclopedia entry). Crossref abstract: basic characteristics,
  history, theory and criticism of DDC; the encyclopedia text states that it describes DDC "in comparison to other
  bibliographic classifications" and frames KO as "evaluating different KOSs".

**Descriptive (qualitative) comparisons of schemes.** These support the paper's statement that schemes are compared
mostly by description.

- **Nocetti, M. (1978).** Agricultural soil science in universal classification systems: a comparative analysis.
  *Knowledge Organization* 5(1): 15-20. doi:10.5771/0943-7444-1978-1-15 (open PDF, page 1): DDC, BBC and LCC compared
  on presentation, structure and relationships for one field.
- **Oh, D.-G. (2023).** Comparative analysis of national classification systems: cases of KDC and NDC. *Knowledge
  Organization* 50(2): 83-98. doi:10.5771/0943-7444-2023-2-83 (Crossref abstract).
- **Wu, Y. (2023).** Organization of complex topics in comprehensive classification schemes: case studies of disaster
  and security. *Knowledge Organization* 50(2): 99-110. doi:10.5771/0943-7444-2023-2-99 (Crossref abstract: LCC and
  the Classification Scheme for Chinese Libraries against two home-grown taxonomies).
- **Wu, Y. (2024).** Political stances and priorities in national level general classification schemes. *Knowledge
  Organization* 51(2): 73-82. doi:10.5771/0943-7444-2024-2-73 (Crossref abstract).
- **Choi, I. (2017).** Visualizations of cross-cultural bibliographic classification: comparative studies of the KDC
  and the DDC. *NASKO* 6(1): 39-55. doi:10.7152/nasko.v6i1.15229 (OpenAlex abstract: "descriptive method").

### 3.3 C: not relevant

All other screened titles. The bulk were automatic text classification (e.g. Larson 1992 on automatic LCC; Wang 2009
on automatic DDC; Ho 2024 at TPDL on LLM-based DDC), cataloguing rules and OPAC studies, critical and ethical studies
of specific classes (race, gender, eugenics ontogeny), book reviews, editorials, and bibliometric papers unrelated to
classification structure. Lund and Agbaji (2018, *Knowledge Organization* 45(5): 380-392,
doi:10.5771/0943-7444-2018-5-380) is a survey of librarians' LCC/DDC preferences and was classed C.

## 4. Sources for specific claims

### 4.1 Knowledge-organization survey of scheme evaluation criteria (Section 1 TODO)

None found that surveys evaluation criteria for universal classification schemes specifically. The ISKO Encyclopedia
has no entry on KOS evaluation. Verified candidates for a narrower citation:

- Owens and Cochrane 2004 (thesaurus evaluation techniques: structural, formative, observational, comparative);
- Golub et al. 2016 (evaluation framework for subject indexing and classification in retrieval);
- Hjørland 2025 (critical evaluation of DDC against alternatives).

Recommendation: drop "survey of scheme evaluation criteria" and instead cite examples of descriptive comparisons
(Nocetti 1978; Zins and Santos 2011; Oh 2023; Wu 2023) for "the literature answers this mostly qualitatively", plus
Owens and Cochrane 2004 for the existence of structural evaluation of KOS.

### 4.2 History of LCC and DDC

- DDC: Comaromi 1976 (*Knowledge Organization* 3(1): 11-15, doi:10.5771/0943-7444-1976-1-11; open PDF read) traces the
  conception of the decimal plan and the first and second (1885) editions. Hjørland 2025 (doi above) states, in
  its ISKO version, that DDC was "first published by Melvil Dewey in 1876" and divides its history into editions 1-6
  (1876-1899), 7-15 (1911-1952), 16-21 (1958-1996) and the digital period.
- LCC: Fraser 2020, Mathematics in library and review classification systems: an historical overview. *Knowledge
  Organization* 47(4): 334-356, doi:10.5771/0943-7444-2020-4-334. Crossref abstract: library classification received
  "sustained attention in the period from 1870 to the present"; the article focuses "on the Library of Congress
  Classification in its various iterations from 1905 to the present". This is the only peer-reviewed, verified
  statement of LCC's time span found. Chan 1990 and 1995 (CCQ) discuss LCC online and alternative numbers, not
  history. The Encyclopedia of Library and Information Sciences entry "Library of Congress Classification (LCC)"
  (3rd ed. 2009, doi:10.1081/e-elis3-120043714, pp. 3383-3391; 4th ed. 2017, doi:10.1081/e-elis4-120043714,
  pp. 2847-2855) exists, but Crossref lists no authors and the text was not read.
- UDC (if needed): McIlwaine 1997, *JASIS* 48(4): 331-339,
  doi:10.1002/(sici)1097-4571(199704)48:4<331::aid-asi6>3.0.co;2-x. Abstract (OpenAlex): UDC "developed in 1895" from
  DDC; history of its editorship and revision.

Suggested wording: "DDC has been published since 1876 [hjorland2025; comaromi1976] and LCC since the first years of
the twentieth century [fraser2020]." The common 1897 start date for LCC work (Hanson and Martel) was found only in
secondary web sources and is not recommended without a primary source.

### 4.3 Menu depth/breadth studies (Section 5.4 TODO)

All verified against Crossref; findings from the abstract source named.

- **Miller, D.P. (1981).** The depth/breadth tradeoff in hierarchical computer menus. *Proc. Human Factors Society
  Annual Meeting* 25(1): 296-300. doi:10.1177/107118138102500179. Crossref abstract: 64 goals, depth 1-6, breadth 2-64;
  acquisition time U-shaped with the minimum at two levels of eight choices; errors agreed.
- **Kiger, J.I. (1984).** The depth/breadth trade-off in the design of menu-driven user interfaces. *International
  Journal of Man-Machine Studies* 20(2): 201-213. doi:10.1016/s0020-7373(84)80018-8. Metadata verified; no abstract
  in Crossref or OpenAlex, ScienceDirect returned HTTP 403. Findings not verified; cite only for existence.
- **Snowberry, K., Parkinson, S.R., Sisson, N. (1983).** Computer display menus. *Ergonomics* 26(7): 699-712.
  doi:10.1080/00140138308963390. Abstract (OpenAlex): used Miller's (1981) hierarchies; speed and accuracy improved with
  breadth.
- **Landauer, T.K., Nachbar, D.W. (1985).** Selection from alphabetic and numeric menu trees using a touch screen.
  *Proc. CHI '85*, 73-78. doi:10.1145/317456.317470. Abstract (OpenAlex): time per screen fits T = k + c log b,
  consistent with Hick-Hyman and Fitts; this favours breadth. Direct support for the `nav_log` model form.
- **Jacko, J.A., Salvendy, G. (1996).** Hierarchical menu design: breadth, depth, and task complexity. *Perceptual and
  Motor Skills* 82(3 suppl.): 1187-1201. doi:10.2466/pms.1996.82.3c.1187. Crossref abstract: perceived complexity rose
  with depth.
- **Larson, K., Czerwinski, M. (1998).** Web page design [Crossref title; the subtitle "implications of memory,
  structure and scent for information retrieval" is not in the Crossref record]. *Proc. CHI '98*, 25-32.
  doi:10.1145/274644.274649. Abstract (OpenAlex): increased depth harmed web search, but a medium depth/breadth
  structure beat the broadest, shallowest one.
- **Zaphiris, P.G. (2000).** Depth vs breath [sic, Crossref title] in the arrangement of web links. *Proc. HFES Annual
  Meeting* 44(4): 453-456. doi:10.1177/154193120004400414. Crossref abstract: completion time rises with depth.
- **Hochheiser, H., Lazar, J. (2010).** Revisiting breadth vs. depth in menu structures for blind users of screen
  readers. *Interacting with Computers* 22(5): 389-398. doi:10.1016/j.intcom.2010.02.003. Abstract (OpenAlex): broad,
  shallow hierarchies outperformed narrow, deep ones.
- Analytical precedent in JASIS: Lipetz and Song 1970 (square-root rule for a two-level file).

Suggested wording for Section 5.4: "User studies of menu hierarchies favour broad, shallow trees over deep ones
[snowberry1983; landauer1985; zaphiris2000], with some finding an optimum at moderate breadth [miller1981;
larson1998]. Landauer and Nachbar fit a per-screen time of k + c log b [landauer1985], the form of `nav_log`."

## 5. Conclusion: does the novelty claim survive?

In narrow form, yes. No screened study scores several universal or near-universal library classification schemes with
a hierarchical-clustering objective (or any tree-structural objective) on a similarity graph over the same corpus. No
use of Dasgupta cost or Moseley-Wang revenue on a human-built KOS was found in any venue searched.

In the broad form used in the Abstract ("We are not aware of a published comparison that scores several universal or
near-universal schemes on one quantitative objective over the same set of works") the claim is too strong. The A
items are quantitative comparisons of several schemes on a common measure over the same material: Cranfield I
(UDC among four index languages, retrieval), Carpenter et al. 1978 (five patent schemes, recall and precision),
Connaway and Sievert 1996 (LCC, DDC and NLM, scatter of one topic), and Wang and Waltman 2016 (two research
classification systems, citation criteria). O'Neill et al. 1987 measured LCC-DDC dispersion on MARC records, and Zins
and Santos 2011 compared LCC, DDC and UDC against one evaluative model. A reviewer from KO or IR would likely cite
Cranfield.

Recommended rewording.

- Abstract, sentences 1-2: "Library classification schemes are compared mostly by description: their history,
  notation, coverage and licence. Quantitative comparisons exist, but they measure retrieval effectiveness on a test
  collection [cleverdon1960; carpenter1978], the scatter of a topic or class across schemes [oneill1987;
  connaway1996], or coverage of a reference map of knowledge [zins2011]. We found no study that scores the hierarchy
  of several universal schemes on one structural objective over the same set of works."
- Section 1, paragraph 1: replace the two TODO markers with: "The literature answers this mostly qualitatively, by
  notation, facet structure, history and institutional adoption [nocetti1978; zins2011; oh2023; wu2023].
  Quantitative comparisons have measured retrieval performance [cleverdon1960; keen1973; carpenter1978] or scatter
  [oneill1987; connaway1996], not the structure of the hierarchy itself. A structured search of JASIST, Knowledge
  Organization, CCQ, ISKO and SIG/CR proceedings, JCDL and TPDL (research/reports/8) found no study that scores
  several universal schemes on a common hierarchical objective over the same corpus. The search covered titles and
  abstracts, not full texts, so this is absence of evidence, not proof."
- Section 2: add a paragraph "Comparing classification schemes" covering Cranfield I, Carpenter et al. 1978,
  O'Neill et al. 1987, Connaway and Sievert 1996, Zins and Santos 2011, and, from bibliometrics, Rafols and
  Leydesdorff 2009, Wang and Waltman 2016 and Klavans and Boyack 2017. One sentence: these score schemes by
  retrieval, mapping scatter or citation coherence, but none scores the tree. In the "Navigation" paragraph, add the
  menu studies of Section 4.3 and Lipetz and Song 1970, and cite Julien et al. 2013 as corpus-weighted structural
  analysis of a subject tree. In the taxonomy-quality sentence, Mader et al. 2012 and Owens and Cochrane 2004 can sit
  next to unterkalmsteiner2023.
- Section 7, "Search for prior work": "The novelty claim rests on the search in research/reports/8: 94 Crossref
  queries over eight venue groups plus five web searches, screening titles and abstracts. Full texts and non-English
  literature were not searched, and Crossref's coverage of proceedings (JCDL before 2001, ISKO before 2012) is
  incomplete."

## 6. Limitations of this search

- Titles and abstracts only. Full text was read only for Carpenter et al. 1978, Jahoda 1961, Comaromi 1976, Nocetti
  1978, Osińska 2010 (two papers) and Amirhosseini 2010, and only their first pages. A comparison reported inside a
  paper on another topic would be missed.
- Many records have no abstract (most pre-2000 Wiley JASIS records, most Knowledge Organization records, Springer
  LNCS chapters). For these, abstracts came from OpenAlex, arXiv or open PDFs where possible; items with no readable
  abstract were not classified A or B unless content was confirmed another way.
- Paywalls and bot blocks: ScienceDirect (HTTP 403), Wiley and SAGE landing pages were not used; Kiger 1984 and Shu et
  al. 2019 abstracts could not be read.
- Crossref relevance ranking is word-based; only the top 30-200 rows per query were screened. `query.container-title`
  for JCDL and TPDL is fuzzy, so most returned rows were from other venues (in-venue rows per query: 0-64 for JCDL,
  1-28 for TPDL). Crossref coverage of ISKO proceedings starts in 2012 (earlier "Advances in Knowledge Organization"
  volumes are absent), and JCDL coverage before 2001 (ACM DL and ADL predecessors) was not checked.
- Venues outside the brief were sampled only by known-item lookups: Journal of Documentation, Information Processing &
  Management, Journal of Informetrics, Scientometrics, Library Resources & Technical Services, Library Quarterly,
  ontology-evaluation venues (ISWC/ESWC/EKAW), and HCI venues. A further pass over Journal of Documentation and LRTS
  is the most likely place for more A items (Cranfield follow-ups, LCC/DDC shelf-browsing studies).
- English-language queries only.
- One searcher, no second screener.

## 7. Unverified leads (not cited above; check before use)

- Cleverdon, C.W. (1960). *Report on the first stage of an investigation into the comparative efficiency of indexing
  systems*. Cranfield: College of Aeronautics, 166 pp. Known only from the Jahoda 1961 review; no DOI; not seen.
- Shu, F., Julien, C.-A., Zhang, L., Qiu, J., Zhang, J., Larivière, V. (2019). Comparing journal and paper level
  classifications of science. *Journal of Informetrics* 13(1): 202-225. doi:10.1016/j.joi.2018.12.005. Crossref
  metadata fetched; abstract not available; content not verified. Likely a B or A item for research classifications.
- Hall, M., Clough, P., Stevenson, M. (2012). Evaluating the use of clustering for automatically organising digital
  library collections. TPDL 2012, LNCS, 323-334. doi:10.1007/978-3-642-33290-6_35. No abstract read.
- Pfeffer, M., Eckert, K., Stuckenschmidt, H. (2008). Visual analysis of classification systems and library
  collections. ECDL 2008, LNCS, 436-439. doi:10.1007/978-3-540-87599-4_57. No abstract read.
- Akdag Salah, A., Gao, C., Suchecki, K., Scharnhorst, A., Smiraglia, R.P. (2012). The evolution of classification
  systems: ontogeny of the UDC. ISKO 2012, 51-57. doi:10.5771/9783956504402-51 (Crossref author string is malformed).
  No abstract read.
- Molholt, P. (1996). Qualities of classification schemes for the information superhighway. *CCQ* 21(2): 19-22.
  doi:10.1300/j104v21n02_04. No abstract exists; content unknown; title suggests evaluation criteria.
- Leazer, G.H., Montoya, R., Furner, J. (2020). Numerical classification and complexity: developing a classification
  of classifications. ISKO 2020, 217-225. doi:10.5771/9783956507762-217. No abstract read.
- Lancaster, F.W., Mills, J. (1964). Testing indexes and index language devices: the ASLIB Cranfield project.
  *American Documentation* 15(1): 4-13. doi:10.1002/asi.5090150104. One-sentence Crossref abstract only.
- Cleverdon, C. (1967). The Cranfield tests on index language devices. *Aslib Proceedings* 19(6): 173-194.
  doi:10.1108/eb050097. Abstract (OpenAlex) read: Cranfield II, on index language devices (term specificity), not
  on classification schemes; probably not needed.
- Encyclopedia of Library and Information Sciences entries "Library of Congress Classification (LCC)" and "Dewey
  Decimal Classification (DDC)" (doi:10.1081/e-elis3-120043714, 10.1081/e-elis4-120043714, 10.1081/e-elis4-120043240).
  Crossref lists no authors; texts not read.
- Losee, R.M. (1993). "The relative shelf location of circulated books: a study of classification, users, and
  browsing", *Library Resources & Technical Services*. Recalled, not found in Crossref (G8); unverified.
- LCC 1897 start date (Hanson and Martel): web secondary sources only (W2).

## 8. Proposed BibTeX

Fields are copied from the Crossref records fetched on 2026-10-10. Changes: all-caps titles and author names in
Crossref (Cleverdon 1960, Keen 1973, Snowberry et al. 1983) were set in normal case; the Crossref author field for
Keen 1973 ("MICHAEL KEEN, E.") was rewritten as Keen, E. Michael. Knowledge Organization volumes 1-19 appeared as
*International Classification*; Crossref files them under the current title, which is kept here with a note. Keys
follow surname+year and do not collide with docs/research/references.bib.

```bibtex
@article{cleverdon1960,
  author = {Cleverdon, C. W.},
  title = {The {ASLIB} {Cranfield} research project on the comparative efficiency of indexing systems},
  journal = {Aslib Proceedings},
  volume = {12},
  number = {12},
  pages = {421--431},
  year = {1960},
  doi = {10.1108/eb049778},
  note = {Crossref title in capitals; systems tested confirmed via jahoda1961}
}

@article{jahoda1961,
  author = {Jahoda, Gerald},
  title = {{ASLIB} {Cranfield} Research Project, Report on the First Stage of an Investigation into the Comparative Efficiency of Indexing Systems (Book Review)},
  journal = {College \& Research Libraries},
  volume = {22},
  number = {3},
  pages = {228},
  year = {1961},
  doi = {10.5860/crl_22_03_228}
}

@inproceedings{cleverdon1991,
  author = {Cleverdon, Cyril W.},
  title = {The significance of the {Cranfield} tests on index languages},
  booktitle = {Proceedings of the 14th annual international ACM SIGIR conference on Research and development in information retrieval},
  pages = {3--12},
  year = {1991},
  doi = {10.1145/122860.122861},
  note = {not read}
}

@article{carpenter1978,
  author = {Carpenter, Anne M. and Jones, Meriel and Oppenheim, Charles},
  title = {Retrieval tests on five classification schemes},
  journal = {Knowledge Organization},
  volume = {5},
  number = {2},
  pages = {73--80},
  year = {1978},
  doi = {10.5771/0943-7444-1978-2-73},
  note = {published as International Classification 5(2); Crossref files it under Knowledge Organization}
}

@article{keen1973,
  author = {Keen, E. Michael},
  title = {The {Aberystwyth} index languages test},
  journal = {Journal of Documentation},
  volume = {29},
  number = {1},
  pages = {1--35},
  year = {1973},
  doi = {10.1108/eb026547},
  note = {Crossref author field reads "MICHAEL KEEN, E."}
}

@article{oneill1987,
  author = {O'Neill, Edward T. and Dillon, Martin and Vizine-Goetz, Diane},
  title = {Class dispersion between the {Library of Congress Classification} and the {Dewey Decimal Classification}},
  journal = {Journal of the American Society for Information Science},
  volume = {38},
  number = {3},
  pages = {197--205},
  year = {1987},
  doi = {10.1002/(sici)1097-4571(198705)38:3<197::aid-asi8>3.0.co;2-2}
}

@article{connaway1996,
  author = {Connaway, Lynn Silipigni and Sievert, MaryEllen C.},
  title = {Comparison of Three Classification Systems for Information on Health Insurance},
  journal = {Cataloging \& Classification Quarterly},
  volume = {23},
  number = {2},
  pages = {89--104},
  year = {1996},
  doi = {10.1300/j104v23n02_07}
}

@article{zins2011,
  author = {Zins, Chaim and Santos, Pl{\'a}cida L. V. A. C.},
  title = {Mapping the knowledge covered by library classification systems},
  journal = {Journal of the American Society for Information Science and Technology},
  volume = {62},
  number = {5},
  pages = {877--901},
  year = {2011},
  doi = {10.1002/asi.21481}
}

@article{wang2016,
  author = {Wang, Qi and Waltman, Ludo},
  title = {Large-scale analysis of the accuracy of the journal classification systems of {Web of Science} and {Scopus}},
  journal = {Journal of Informetrics},
  volume = {10},
  number = {2},
  pages = {347--364},
  year = {2016},
  doi = {10.1016/j.joi.2016.02.003}
}

@article{rafols2009,
  author = {Rafols, Ismael and Leydesdorff, Loet},
  title = {Content-based and algorithmic classifications of journals: Perspectives on the dynamics of scientific communication and indexer effects},
  journal = {Journal of the American Society for Information Science and Technology},
  volume = {60},
  number = {9},
  pages = {1823--1835},
  year = {2009},
  doi = {10.1002/asi.21086}
}

@article{klavans2017,
  author = {Klavans, Richard and Boyack, Kevin W.},
  title = {Which Type of Citation Analysis Generates the Most Accurate Taxonomy of Scientific and Technical Knowledge?},
  journal = {Journal of the Association for Information Science and Technology},
  volume = {68},
  number = {4},
  pages = {984--998},
  year = {2017},
  doi = {10.1002/asi.23734}
}

@article{julien2013,
  author = {Julien, Charles-Antoine and Tirilly, Pierre and Dinneen, Jesse David and Guastavino, Catherine},
  title = {Reducing subject tree browsing complexity},
  journal = {Journal of the American Society for Information Science and Technology},
  volume = {64},
  number = {11},
  pages = {2201--2223},
  year = {2013},
  doi = {10.1002/asi.22915}
}

@incollection{mader2012,
  author = {Mader, Christian and Haslhofer, Bernhard and Isaac, Antoine},
  title = {Finding Quality Issues in {SKOS} Vocabularies},
  booktitle = {Theory and Practice of Digital Libraries},
  series = {Lecture Notes in Computer Science},
  publisher = {Springer Berlin Heidelberg},
  pages = {222--233},
  year = {2012},
  doi = {10.1007/978-3-642-33290-6_25},
  isbn = {9783642332890}
}

@article{owens2004,
  author = {Owens, Leslie Ann and Cochrane, Pauline Atherton},
  title = {Thesaurus Evaluation},
  journal = {Cataloging \& Classification Quarterly},
  volume = {37},
  number = {3-4},
  pages = {87--102},
  year = {2004},
  doi = {10.1300/j104v37n03_07}
}

@article{golub2016,
  author = {Golub, Koraljka and Soergel, Dagobert and Buchanan, George and Tudhope, Douglas and Lykke, Marianne and Hiom, Debra},
  title = {A framework for evaluating automatic indexing or classification in the context of retrieval},
  journal = {Journal of the Association for Information Science and Technology},
  volume = {67},
  number = {1},
  pages = {3--16},
  year = {2016},
  doi = {10.1002/asi.23600}
}

@article{nocetti1978,
  author = {Nocetti, Milton},
  title = {Agricultural soil science in universal classification systems. A comparative analysis},
  journal = {Knowledge Organization},
  volume = {5},
  number = {1},
  pages = {15--20},
  year = {1978},
  doi = {10.5771/0943-7444-1978-1-15},
  note = {published as International Classification 5(1)}
}

@article{oh2023,
  author = {Oh, Dong-Geun},
  title = {Comparative Analysis of National Classification Systems: Cases of {Korean Decimal Classification} ({KDC}) and {Nippon Decimal Classification} ({NDC})},
  journal = {Knowledge Organization},
  volume = {50},
  number = {2},
  pages = {83--98},
  year = {2023},
  doi = {10.5771/0943-7444-2023-2-83}
}

@article{wu2023,
  author = {Wu, Yejun},
  title = {Organization of Complex Topics in Comprehensive Classification Schemes: Case Studies of Disaster and Security},
  journal = {Knowledge Organization},
  volume = {50},
  number = {2},
  pages = {99--110},
  year = {2023},
  doi = {10.5771/0943-7444-2023-2-99}
}

@article{hjorland2025,
  author = {Hj{\o}rland, Birger},
  title = {Dewey Decimal Classification ({DDC})},
  journal = {Em Quest{\~a}o},
  volume = {31},
  pages = {e-145068},
  year = {2025},
  doi = {10.1590/1808-5245.31.145068},
  note = {Crossref article number e-145068; also ISKO Encyclopedia of Knowledge Organization, https://www.isko.org/cyclo/ddc}
}

@article{comaromi1976,
  author = {Comaromi, John P.},
  title = {Conception and development of the {Dewey Decimal Classification}},
  journal = {Knowledge Organization},
  volume = {3},
  number = {1},
  pages = {11--15},
  year = {1976},
  doi = {10.5771/0943-7444-1976-1-11},
  note = {published as International Classification 3(1)}
}

@article{fraser2020,
  author = {Fraser, Craig},
  title = {Mathematics in Library and Review Classification Systems: An Historical Overview},
  journal = {Knowledge Organization},
  volume = {47},
  number = {4},
  pages = {334--356},
  year = {2020},
  doi = {10.5771/0943-7444-2020-4-334}
}

@article{mcilwaine1997,
  author = {McIlwaine, I. C.},
  title = {The {Universal Decimal Classification}: Some factors concerning its origins, development, and influence},
  journal = {Journal of the American Society for Information Science},
  volume = {48},
  number = {4},
  pages = {331--339},
  year = {1997},
  doi = {10.1002/(sici)1097-4571(199704)48:4<331::aid-asi6>3.0.co;2-x}
}

@article{miller1981,
  author = {Miller, Dwight P.},
  title = {The Depth/Breadth Tradeoff in Hierarchical Computer Menus},
  journal = {Proceedings of the Human Factors Society Annual Meeting},
  volume = {25},
  number = {1},
  pages = {296--300},
  year = {1981},
  doi = {10.1177/107118138102500179}
}

@article{kiger1984,
  author = {Kiger, John I.},
  title = {The depth/breadth trade-off in the design of menu-driven user interfaces},
  journal = {International Journal of Man-Machine Studies},
  volume = {20},
  number = {2},
  pages = {201--213},
  year = {1984},
  doi = {10.1016/s0020-7373(84)80018-8},
  note = {metadata verified; abstract and findings not read}
}

@article{snowberry1983,
  author = {Snowberry, Kathleen and Parkinson, Stanley R. and Sisson, Norwood},
  title = {Computer display menus},
  journal = {Ergonomics},
  volume = {26},
  number = {7},
  pages = {699--712},
  year = {1983},
  doi = {10.1080/00140138308963390},
  note = {Crossref author names in capitals}
}

@inproceedings{landauer1985,
  author = {Landauer, T. K. and Nachbar, D. W.},
  title = {Selection from alphabetic and numeric menu trees using a touch screen},
  booktitle = {Proceedings of the SIGCHI conference on Human factors in computing systems - CHI '85},
  publisher = {ACM Press},
  pages = {73--78},
  year = {1985},
  doi = {10.1145/317456.317470}
}

@article{jacko1996,
  author = {Jacko, Julie A. and Salvendy, Gavriel},
  title = {Hierarchical Menu Design: Breadth, Depth, and Task Complexity},
  journal = {Perceptual and Motor Skills},
  volume = {82},
  number = {3\_suppl},
  pages = {1187--1201},
  year = {1996},
  doi = {10.2466/pms.1996.82.3c.1187}
}

@inproceedings{larson1998,
  author = {Larson, Kevin and Czerwinski, Mary},
  title = {Web page design},
  booktitle = {Proceedings of the SIGCHI conference on Human factors in computing systems - CHI '98},
  publisher = {ACM Press},
  pages = {25--32},
  year = {1998},
  doi = {10.1145/274644.274649},
  note = {Crossref title has no subtitle; check the ACM page before adding one}
}

@article{zaphiris2000,
  author = {Zaphiris, Panayiotis G.},
  title = {Depth vs Breath in the Arrangement of Web Links},
  journal = {Proceedings of the Human Factors and Ergonomics Society Annual Meeting},
  volume = {44},
  number = {4},
  pages = {453--456},
  year = {2000},
  doi = {10.1177/154193120004400414},
  note = {"Breath" as in the Crossref record}
}

@article{lipetz1970,
  author = {Lipetz, Ben-Ami and Song, Czetong T.},
  title = {How many cards per file guide? optimizing the two-level file},
  journal = {Journal of the American Society for Information Science},
  volume = {21},
  number = {2},
  pages = {140--141},
  year = {1970},
  doi = {10.1002/asi.4630210206}
}

@article{koch2006,
  author = {Koch, Traugott and Golub, Koraljka and Ard{\"o}, Anders},
  title = {Users Browsing Behaviour in a {DDC}-Based Web Service: A Log Analysis},
  journal = {Cataloging \& Classification Quarterly},
  volume = {42},
  number = {3-4},
  pages = {163--186},
  year = {2006},
  doi = {10.1300/j104v42n03_07}
}
```

## 9. Use in the papers

The DOIs in Section 8 were checked again against Crossref when they were added to
`docs/research/references.bib`; all 31 resolve to the titles and authors given. 25 of them are now cited.
Paper 1 and the method-survey draft cite them in the abstract, in Section 1 and in the related-work and limitations
sections. Paper 2 cites them for the LCC/DDC history and the menu-study limit. Not cited: `cleverdon1991` and
`kiger1984` (findings not read), `golub2016`, `jacko1996`, `mcilwaine1997` and `koch2006`. The Cranfield facts
(four systems, 18,000 documents) were re-read in the open PDF of `jahoda1961`.
