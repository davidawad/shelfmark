#import "../template.typ": *
#import "../generated.typ": v

= A survey of the classification systems <sec:survey>

This section describes the thirteen scheme builds that the benchmark scores,
and gives DDC and UDC as context. The benchmark uses twelve distinct schemes:
LCC enters twice, as the published outline (`lcc`) and as the complete
schedules (`lcc-full`). Facts about structure and size are those recorded by
the scheme builders in the repository. Facts about licences are those recorded in the
scheme READMEs and in the licensing report of the project (checked 9--10
October 2026); they are a description of what each publisher states, not
legal advice.

#wide-table(
  {
    import "../generated.typ"
    let rows = (
      ("lcc", "Library of Congress", "2024", "7,392", "21 main classes (letters); subclasses and numbered spans (outline)", "US government work, 17 U.S.C. 105; LC's Classification Web notice calls LC classification copyrighted outside the USA. Bundled"),
      ("lcc-full", "Library of Congress", "2024", "416,186", "13 levels; 408,794 numbered entries from 44 FreeLCC text PDFs added to the outline; max 7,994 children", "As `lcc`. Bundled"),
      ("fdc", "J. M. Ockerbloom", "0.06 (2010-08-19)", "1,097", "6 levels; Dewey-shaped numbers 000 to 999.9999; written from DDC 11th ed. (1922)", "CC0 1.0. Frozen; author states no long-term maintenance. Bundled"),
      ("bk", "GBV (K10plus)", "export 2026-10-03", "2,140", "Five roots, two-digit headings, decimal classes; academic bias", "CC0 1.0. Bundled; weekly rolling export"),
      ("ilc2", "ISKO Italy (C. Gnoli and contributors)", "2019-10-17", "10,117", "Faceted, ordered by integrative levels; 9,174 concepts and 943 facet indicators", "Unsettled: CC BY 3.0 in the SKOS file, CC BY-SA 3.0 in BARTOC, no site page states a licence. Not bundled"),
      ("arxiv", "arXiv (arxiv-base)", "2024-04-01", "192", "8 groups, 38 archives (18 retired), categories", "MIT (arxiv-base). Bundled"),
      ("msc", "Mathematical Reviews (AMS) and zbMATH Open", "2020", "6,603", "63 sections, subsections, leaf codes", "CC BY-NC-SA 4.0. Not bundled (built from the official CSV)"),
      ("jel", "American Economic Association", "2026", "1,015", "20 categories, subcategories, codes (A, A1, A10)", "All rights reserved; terms grant personal use only. Not bundled"),
      ("acm-ccs", "ACM", "2012", "2,113", "13 top-level concepts, depth up to 6; 159 concepts have more than one parent", "ACM copyright; \"freely available for educational and research purposes\". No redistribution grant. Not bundled"),
      ("ipc", "WIPO", "2026.01", "80,145", "8 sections, 132 classes, 655 subclasses, 7,668 main groups, 71,682 subgroups", "CC BY 4.0 (WIPO Open Access Policy). File too large to commit; built by a pinned builder"),
      ("openalex-topics", "OpenAlex", "2026-10-09", "4,798", "4 domains, 26 fields, 252 subfields, 4,516 topics; citation-derived, LLM-named", "CC0 1.0. Bundled"),
      ("unesco-thesaurus", "UNESCO", "2026-08-31", "4,595", "7 domains, 88 microthesauri, about 4,500 concepts; 229 concepts have 2 to 6 broader terms", "CC BY-SA 3.0 IGO. Adaptation stays share-alike"),
      ("eurovoc", "Publications Office of the EU", "4.24 (2026-07-08)", "7,663", "21 domains, 127 microthesauri, 7,515 descriptors; 8 levels", "Sources disagree (Commission reuse notice, CC BY 4.0 for editorial content, CC0 in BARTOC). Not bundled"),
      ("DDC", "OCLC", "ed. 23", "n/a", "Decimal tree; 1,000 classes at three digits", "Copyright and trademark OCLC; full schedules only through paid WebDewey"),
      ("UDC", "UDC Consortium", "Summary", "n/a", "Faceted decimal; Summary has about 2,600 of 70,000+ entries", "Full reference file licensed to restricted communities; Summary CC BY-NC 4.0 from 2026 (see text)"),
    )
    tbl(
      ([scheme], [maintainer], [version], [classes], [structure], [licence and redistribution status]),
      rows,
      align: (left, left, left, right, left, left),
      columns: (auto, 1.35fr, auto, auto, 2fr, 2.4fr),
      size: 6.8pt,
    )
  },
  [The classification systems: thirteen benchmarked builds, with DDC and UDC as
   context. "Classes" is the number of classes in the build used in the
   benchmark. "Bundled" means the data file is committed in the repository.],
  label: <tab:survey>,
)

== Library classifications: LCC, DDC, UDC

*LCC* is maintained by the Library of Congress. Its schedules are published
as PDFs for letters A--Z @freelcc, and the Library describes its linked-data
service as a public-domain data set @idloc. The outline has 21 main classes
(the letters A--Z less I, O, W, X and Y). The complete schedules are not
available as one machine-readable current file: the project's report found
a 2016 MARC bulk snapshot of 649,332 records, about 2.5 million per-URI
records on id.loc.gov, and the PDFs, which the `lcc-full` builder parses.
Whether LCC is free to redistribute depends on jurisdiction. US government
works are not subject to copyright @usc105, but LC's Classification Web
notice says LC-originated classification data are copyrighted except within
the USA @classweblegal. LCC has been developed in successive editions since
the early twentieth century @fraser2020.

*DDC* is owned by OCLC @deweyrights. Full schedules are available through
the subscription WebDewey, and OCLC's terms for Dewey linked data prohibit
scraping, indexing and long-term storage; the base set is free to view only.
The DDC 22 Summaries PDF carries "All rights reserved". DDC is therefore absent from
the benchmark, although OCLC describes it as the world's most widely used way
to organize library collections @deweyrights. The
FDC was written from the out-of-copyright 1922 eleventh edition of DDC and
checked only for compatibility.

*UDC* is maintained by the UDC Consortium. The full Master Reference File is
licensed, with several licence types for restricted communities, and
embedding it in software requires a paid publishing licence @udclicences.
The licensing report records the UDC Summary (about 2,600 of 70,000+
entries) as CC BY-NC 4.0 from 2026, where earlier search snippets and the
survey report described a share-alike Creative Commons 3.0 licence. We use
the later statement. Either way the non-commercial condition, or the
share-alike condition, blocks use inside a permissively licensed tool.

== Open universal and near-universal schemes

No open-licensed scheme is at once universal, deep and equipped with a full
LCC or DDC crosswalk. The open candidates are shallow, or are not shelf
orders. The survey report records the main ones as follows.

*BK* (Basisklassifikation) is a hierarchical decimal subject classification
for scholarly works, maintained by the Gemeinsamer Bibliotheksverbund
@bk. It has 2,140 classes in this export (BARTOC lists 2,139, and the export
carries one class marked as deleted). Its data are CC0 and refreshed weekly.
It is academically biased, and its crosswalk is thin: the survey counted 130
mappings from BK in coli-conc, of which 49 point to DDC edition 23 and none to LCC. *FDC* is CC0 and
DDC-shaped, but frozen since 2010 @fdc. *ILC2* @ilc2 is universal and
faceted, orders phenomena by integrative levels rather than by library
convention, and has no settled licence. Its SKOS file declares CC BY 3.0
while the BARTOC record lists CC BY-SA 3.0, and no page on the maintainers'
site states a licence; the project therefore treats the data as CC BY-SA 3.0
if redistributed.

*OpenAlex Topics* is a four-level, citation-derived and LLM-named hierarchy
for research literature only, released as CC0 @openalextopics @priem2022
@openalexpricing.
*UNESCO Thesaurus* and *EuroVoc* are thesauri, narrower than a universal
scheme, with a domain--microthesaurus--descriptor layout. *IPC* is the
International Patent Classification, a deep scheme that is relevant here as
a large hierarchy over a technical domain. WIPO's open access policy issues
its online content under CC BY 4.0 unless marked otherwise, and the master
files carry no other notice @ipc @wipoterms. The remaining schemes are domain schemes: arXiv's category taxonomy
@arxivtaxonomy, MSC2020 @msc2020, JEL @jel and ACM CCS @acmccs. Of these, only
arXiv's carries an open software licence, and JEL publishes no open licence.

*ACM CCS, 1998 and 2012.* The 2012 system replaced the 1998 edition. ACM
says it mapped the old scheme to the new one and shows both terms on article
citation pages, but publishes no 1998-to-2012 mapping file; its public
download contains the 2012 tree only. We found no mapping file. This matters
below, because the arXiv `acm-class` field of the corpus carries CCS 1998
codes (@sec:corpus).

*Other surveyed resources.* The survey also recorded resources that are
supplements rather than shelf orders: Wikidata (CC0, a class graph, not a
shelf order), FAST (ODC-By headings), LCSH (headings with no class numbers),
GND and Thema. Wikidata is CC0 and holds 942 items with both a DDC and
an LCC statement, the only open DDC--LCC concordance the survey measured, and
a small one. The Open Shelves Classification was never
completed. The survey found no open machine-readable copy of the Colon, Bliss or
ICC classifications.
