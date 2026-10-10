# Open universal / near-universal classification schemes (survey, 2026-10-09)

Method: every URL in the Sources list returned HTTP 200 to curl or was read via WebFetch/WebSearch, except where marked. BARTOC (bartoc.org) JSON API records were used for license/extent/modified metadata where the scheme's own site was silent; those are marked (BARTOC). Anything not directly verified is marked UNVERIFIED.

Bottom line first: no open-licensed scheme exists that is BOTH universal AND deep AND has a full LCC/DDC crosswalk. Universal + open candidates are shallow (BK 2,139 classes, FDC ~1,000 DDC-like numbers, ILC ~10.8k classes) or are non-shelf-order vocabularies (OpenAlex Topics, Wikidata).

## Comparison table

| Scheme | License (exact) | Coverage | Size | Structure | Machine-readable download | Maintenance | Crosswalk to LCC/DDC |
|---|---|---|---|---|---|---|---|
| ILC (Integrative Levels Classification, Gnoli/ISKO Italy) | CC BY-SA 3.0 per BARTOC record (http://creativecommons.org/licenses/by-sa/3.0/); the iskoi.org pages themselves state no license (UNVERIFIED on-site) | Universal (all knowledge, ordered by integrative levels) | ILC2: 10,845 classes and facets (iskoi.org); ~10,851 per DCMI paper | Faceted, hierarchical by integrative level; freely combinable facets | SKOS version of ILC2 linked from iskoi.org/ilc; BARTOC lists SKOS, Skosmos at skosmos.bartoc.org/472/. Exact file URL not captured (UNVERIFIED) | ILC2 declared stable; site footer dated 2024-09-24; BARTOC modified 2023-02-20; DCMI 2026 paper co-authored by Gnoli (active research) | None found on ILC site; not in coli-conc counts I checked. UNVERIFIED |
| ICC (Information Coding Classification, Dahlberg) | Not open: 1982 edition notice "all rights reserved" (INDEKS/FID); BARTOC lists no license | Universal (9 levels, ~6,500 fields) | ~6,500 knowledge fields (Wikipedia) | Hierarchical, nine ontical levels, 9x9 | Only scanned/PDF (isko.org/icc.pdf); no data download | Dormant (Dahlberg d. 2025? UNVERIFIED); no release | None found |
| Colon Classification (Ranganathan) | Print copyright, status unverified; no open license found (UNVERIFIED) | Universal, 42 main classes | 42 main classes + facets | Faceted (PMEST) | None found | 7th ed. 1971 (6 later editions after 1933 per Wikipedia); not maintained as data | None |
| Bliss BC2 | Bliss Classification Association / K.G. Saur copyright; schedules free to read as PDF on blissclassification.org.uk; no open license found | Universal | 15 of ~28 volumes published (Wikipedia) | Faceted | PDF per class only. Community SKOS conversion in Code4lib Journal 59 (2024, Bartholomew) is third-party, license not checked (UNVERIFIED) | BCA still publishes amendments; volumes 1977-1999 | None |
| Basic Classification (BK / Basisklassifikation, Netherlands, run by GBV VZG) | CC0 1.0 (BARTOC record; http://creativecommons.org/publicdomain/zero/1.0/) | Universal, academic-oriented | 2,139 classes (BARTOC) | Hierarchical decimal tree, enumerative | JSKOS / NDJSON / JSON from api.dante.gbv.de (HTTP 200 verified, JSON endpoint) and coli-conc.gbv.de/api | BARTOC modified 2026-09-22 (actively maintained) | Partial: coli-conc holds 130 mappings from BK; 49 to DDC e23, 30 to DDC (BARTOC node 241), 19 to RVK; no LCC. Thin, not a full crosswalk |
| Open Shelves Classification (LibraryThing) | Intended public domain (2008 announcements); no license file | Intended universal, public-library focus | Never completed | Intended tree | None | Dormant; LibraryThing group "officially dormant"; BARTOC record modified 2019 | None |
| Free Decimal Correspondence (Ockerbloom) | Public domain, CC0 (author states "explicitly placed ... in the public domain") | Universal, Dewey-compatible 3-digit plus decimals | 000-999 with finer decimals; count not stated | Decimal tree | HTML + structured text at everybodyslibraries.com/free-decimal-correspondence/ (200 verified) | v0.06, 2010-08-19; author says no long-term maintenance | Intended to be roughly compatible with DDC numbers (based on DDC 11th ed. 1922 ideas); no LCC mapping |
| OpenAlex Topics | Data CC0 (per OpenAlex developer docs as relayed by search; license text page not fetched, UNVERIFIED on exact page) | Research literature only (all academic fields) | 4 domains, 26 fields, 252 subfields, 4,516 topics | 4-level tree (topics have multi-parent via works) | Snapshot (JSONL/parquet, S3) and API; free snapshot cadence conflicting (quarterly vs monthly) | Active | None to LCC/DDC |
| OpenAlex Concepts (deprecated) | Same CC0 data (UNVERIFIED on page) | Research literature | 65,026 concepts, 6 levels | DAG-ish tree | Included in snapshot, frozen | Deprecated/frozen, replaced by Topics in 2024; origin MAG | None |
| Microsoft Academic Graph fields of study | MAG data was ODC-BY (UNVERIFIED, not confirmed in sources) | Research literature | ~hundreds of thousands (UNVERIFIED; OpenAlex kept 65k) | Hierarchy up to 6 levels | MAG dumps retired | Retired 2021-12-31 (microsoft.com project page) | None |
| Wikidata class hierarchy | CC0 1.0 (Wikidata:Licensing, verified) | Everything; far from a shelf order (subclass of graph is messy, cyclic in places) | Millions of items; class count not checked | DAG (P279/P31) | Dumps (JSON/RDF) from Wikimedia; SPARQL | Continuous | Properties for DDC/LCC identifiers exist on items (from my background knowledge, UNVERIFIED here); sparse |
| Wikipedia category graph | CC BY-SA (Wikipedia text license; UNVERIFIED by fetch for categories) | All topics | ~2M categories (UNVERIFIED) | Loose DAG with cycles | Wikimedia dumps | Continuous | None official |
| Wikipedia Vital Articles levels | CC BY-SA | 11 Level-4 sublists (People, History, Geography, Arts, Philosophy and religion, Everyday life, Society and social sciences, Biological and health sciences, Physical sciences, Technology, Mathematics) | 10 / 100 / 1,000 / 10,000 / 50,000 by level | Curated tree | Wikitext only | Active | None |
| DBpedia ontology | CC BY-SA 3.0 and GFDL (dbpedia.org/about) | General, Wikipedia-derived, ontology not subject classification | 768 classes, ~3,000 properties | Subsumption tree | OWL via archivo.dbpedia.org; databus (HTTP 500 at time of check) | Monthly releases, 2025-06 latest seen | None |
| UNESCO Thesaurus | CC BY-SA 3.0 IGO (vocabularies.unesco.org/en/about) | Education, science, culture, social sci, information (not all knowledge) | 4,499 concepts (DCMI page; not on UNESCO site) | Thesaurus: 7 subject areas, 72 microthesauri | SKOS Turtle, RDF/XML, REST, SPARQL | Last modified 2026-02-27 (DCMI) | None found |
| EuroVoc | CC0 1.0 per BARTOC record (http://creativecommons.org/publicdomain/zero/1.0/); op.europa.eu page states no license | Policy/legal/EU domains, 21 domains | 7,000+ concepts, 127 microthesauri (DCMI) | Thesaurus | SKOS/XML via op.europa.eu "Downloads" tab (URL not captured, UNVERIFIED) | v4.24 labelled LATEST; BARTOC modified 2026-02-13 | None (mapping to STW exists, not LCC/DDC) |
| LCSH | US federal work; public-domain claim from secondary source only (Wikipedia infobox); id.loc.gov/about states no license (UNVERIFIED) | Universal subject headings (not a shelf order) | 300k+ in 2008 per history source; current count not found | Thesaurus with BT/NT, pre-coordinated strings | id.loc.gov/download/ : MADS/RDF and SKOS/RDF, JSON-LD and N-Triples (verified 200) | Continuous, weekly-ish | Strong, via LC records and Classification Web, but not an open dataset |
| FAST (OCLC) | ODC-BY 1.0 (OCLC FAST page; https://opendatacommons.org/licenses/by/1-0/ verified) | Universal subject headings, faceted LCSH derivative | Not stated on page | Faceted (not hierarchical) | MARCXML, ISO MARC, RDF N-Triples (researchworks.oclc.org/researchdata/fast/FASTAll.nt.zip; direct curl returned 403, UNVERIFIED download) | Updated 2024-10-10 | Derived from LCSH; no class numbers |
| GND subject headings | CC0 (BARTOC record, DNB announcements) | Universal for German libraries | ~9.5M GND authority records total (NDL report, undated); subject-heading subset count not found | Thesaurus/authority, not a shelf order | RDF Turtle bulk on DNB; DNB node URL 404 so exact link UNVERIFIED | BARTOC modified 2026-02-09 | Via DNB/DDC-Sachgruppen (not verified) |
| Thema (EDItEUR) | "free of charge for all to use, subject to EDItEUR's normal, highly-permissive licence" (editeur.org/206/About-Thema/); licence PDF (doi.org/10.4400/nwgj) not read | Book trade, all subjects | 20 top-level areas; total not found; v1.6 added 131 categories and 411 qualifiers | Tree plus qualifiers | Excel, HTML, XML, JSON; SKOS at ns.editeur.org/themacomplete.rdf (200 verified) | v1.6, 2024-10-31 | Mapping files to BISAC/BIC published by EDItEUR; not to LCC/DDC |
| BISAC | BISG copyright, licensed (not open); page not fetched (UNVERIFIED) | Book trade | ~50 top-level, ~4,000 headings (background, UNVERIFIED) | Tree | Paid/licensed list | Annual | Not applicable |
| DDC | Copyright and trademark OCLC (oclc.org Dewey page); BARTOC lists CC BY-NC-ND 3.0 + InC for the open summaries; full schedules via paid WebDewey | Universal | 1,000 at 3 digits, ~50k+ full (background) | Decimal tree | Base linked data was published at dewey.info (host unreachable on 2026-10-10; see report 4); full via WebDewey | Active, e23 | n/a |
| UDC Summary (already bundled) | CC BY-SA 3.0 (udcc.org/index.php/faq/7) | Universal | ~2,600 classes | Faceted decimal | SKOS at udcdata.info | Last updated 2012 for the Summary | Via UDC Consortium, not open |
| Newer GitHub/GitLab projects 2020-2026 | None found: searches for an open universal scheme returned only tools (nichtich/wikidata-taxonomy, last updated ~2017) and domain taxonomies | | | | | | |

## Verdict: best open candidates for a general-purpose shelf order

1. Basic Classification (BK): the only CC0, maintained (modified 2026-09), machine-readable (JSKOS/NDJSON), universal decimal tree. Only 2,139 classes, academic-biased, thin crosswalk (49 DDC mappings, no LCC).
2. ILC2: universal, 10.8k classes, SKOS, CC BY-SA 3.0 (via BARTOC; confirm on-site). Its order is by integrative levels not library convention, and the license is share-alike.
3. FDC: CC0 and DDC-shaped, but based on 1922 DDC ideas, frozen in 2010.
4. Supplements, not general shelf orders: OpenAlex Topics (CC0, 4,516 topics, literature only), Wikidata (CC0, DAG not a shelf order), UNESCO/EuroVoc (thesauri, narrower), FAST/GND/LCSH (headings, no class numbers).
5. Not usable as open: DDC, ICC, Colon, Bliss BC2 (no machine-readable open copy), BISAC, OSC (never built). No relevant 2020-2026 project found.

Practical read: nothing replaces LCC. If one is built, BK (CC0) or ILC2 are the nearest seeds; no existing scheme gives both open license and LCC/DDC crosswalk coverage.

## Sources (all fetched unless marked)

- ILC: https://www.iskoi.org/ilc/ , https://www.iskoi.org/ilc/2/scheme , https://bartoc.org/en/node/472 , https://isko.org/cyclo/ilc
- BK: https://bartoc.org/en/node/18785 , https://uri.gbv.de/terminology/bk/ , https://api.dante.gbv.de/voc/concepts?uri=http%3A%2F%2Fbartoc.org%2Fen%2Fnode%2F18785&download=json , https://coli-conc.gbv.de/api/mappings?fromScheme=http://bartoc.org/en/node/18785&limit=130
- ICC: https://bartoc.org/en/node/1030 , https://en.wikipedia.org/wiki/Information_Coding_Classification , https://www.isko.org/icc.pdf (from search, not fetched)
- Colon: https://en.wikipedia.org/wiki/Colon_classification
- Bliss: https://en.wikipedia.org/wiki/Bliss_bibliographic_classification , https://blissclassification.org.uk/bchist.shtml (from search), https://journal.code4lib.org/articles/18073 (from search)
- OSC: https://bartoc.org/en/node/1939 , https://blog.librarything.com/2008/08/open-shelves-classification-update-and-summary/ (from search)
- FDC: https://everybodyslibraries.com/about-the-free-decimal-correspondence/ , https://everybodyslibraries.com/free-decimal-correspondence/
- OpenAlex: https://help.openalex.org/api-entities/topics , https://help.openalex.org/data/concepts/ , https://help.openalex.org/api/deprecations/ (from search)
- MAG: https://www.microsoft.com/en-us/research/project/microsoft-academic-graph/
- Wikidata: https://www.wikidata.org/wiki/Wikidata:Licensing , https://creativecommons.org/publicdomain/zero/1.0/
- Wikipedia Vital Articles: https://en.wikipedia.org/wiki/Wikipedia:Vital_articles/Level/4
- DBpedia: https://www.dbpedia.org/about/ , https://www.dbpedia.org/resources/ontology/
- UNESCO: https://vocabularies.unesco.org/en/about , https://msi.dublincore.org/standards/unesco-thesaurus
- EuroVoc: https://op.europa.eu/en/web/eu-vocabularies/th-dataset/-/resource/dataset/eurovoc , https://bartoc.org/en/node/15 , https://msi.dublincore.org/standards/eurovoc
- LCSH: https://id.loc.gov/download/ , https://id.loc.gov/about/
- FAST: https://www.oclc.org/research/areas/data-science/fast/download.html , https://opendatacommons.org/licenses/by/1-0/
- GND: https://bartoc.org/en/node/430
- Thema: https://www.editeur.org/151/Thema , https://www.editeur.org/206/About-Thema/ , https://ns.editeur.org/themacomplete.rdf
- DDC: https://www.oclc.org/en/dewey/resources/summaries.html
- UDC: https://udcc.org/index.php/faq/7
- Newer-project search: https://github.com/nichtich/wikidata-taxonomy (from search, not fetched)
