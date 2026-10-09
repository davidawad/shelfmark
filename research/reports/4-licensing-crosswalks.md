# Open general-purpose classification: licensing and crosswalk facts

Researched 2026-10-09. Read-only. Every URL below was fetched by me this session unless marked UNVERIFIED. "Quote" = verbatim from the fetched page. I am not a lawyer; where a license is silent I say so rather than infer.

## 1. Licensing table

Columns: data free to download = full machine-readable dataset, no payment. Redistributable = inside an MIT/Apache repo or tool. Free to assign = you may put the scheme's class numbers on your own documents.

| Scheme | Data free to download | Redistributable in open-source tool | Free to assign numbers | Primary source |
|---|---|---|---|---|
| LCC schedules (print-ready PDFs) | Yes, PDFs for all letters A-Z (data selected May 2024). Not machine-readable. The page carries no rights statement. | Page is silent. The Classification Web legal notice says LC-originated classification is "copyrighted, except within the USA". See sec. 3 for 17 USC 105. | No restriction found on the PDF page or the legal notice (silence, not a grant) | https://www.loc.gov/aba/publications/FreeLCC/freelcc.html ; https://classweb.loc.gov/legal.html |
| LCC as linked data (id.loc.gov/authorities/classification) | Per-URI only (RDF/XML, JSON-LD, N-Triples). About 2.53M URIs in the sitemap. NOT in the bulk download page (checked every dataset heading). | LC states the service is "a public domain data set" (as is, no warranties) | Not addressed | https://id.loc.gov/about/ ; https://id.loc.gov/download/ ; https://id.loc.gov/techcenter/ |
| LCC as MARC Classification records (MDSConnect open-access) | YES, bulk, free: 649,332 records (2016 set, 33 MB gz) | LC says the sets are "made available primarily for research and development usage". No license text found. | Not addressed | https://www.loc.gov/cds/products/MDSConnect-classification.html ; https://www.loc.gov/cds/products/marcDist.php |
| Classification Web (paid) | No, subscription. Per-record MARC download only. | Governed by "Classification Web Plus Terms and Conditions of Use". The PDF was not retrieved. UNVERIFIED. | n/a | https://classweb.loc.gov/legal.html |
| LCSH | Yes, bulk: MADS/RDF and SKOS in JSON-LD, NT, TTL, XML (SKOS NT is 101 MB gz; MADS NT 231 MB gz). About 468k subject authorities. | "public domain data set" per LC. | Not addressed | https://id.loc.gov/download/ ; https://id.loc.gov/about/ |
| FAST (OCLC) | Yes: MARC XML 198 MB zip, RDF N-Triples 266 MB zip, ISO MARC 203 MB zip, by facet or all | ODC-By (Open Data Commons Attribution). Quote: "made available by OCLC ... under the Open Data Commons Attribution License (ODC-By)". Attribution required. | Not addressed. Plain reading of ODC-By: free with attribution when you republish the data. | https://www.oclc.org/research/areas/data-science/fast/download.html ; https://www.oclc.org/research/areas/data-science/fast.html ("approximately 1.8 million headings") |
| Dewey DDC (full) | No. WebDewey is subscription. | No. Quote: "All copyright rights in the Dewey Decimal Classification system are owned by OCLC, Inc." Dewey is a registered trademark. | NOT STATED. OCLC refers to "licensing the Dewey name or classification numbers for a legitimate business purpose" and says to contact them. Whether a personal catalog needs a license is not answered by any page I could fetch. | https://www.oclc.org/en/dewey/resources ; https://www.oclc.org/en/dewey/resources/linkeddata.html |
| DDC Summaries (top 3 levels) | PDF downloadable (the file I fetched is DDC 22 summaries, 2003) | The PDF says: "© 2003 OCLC ... All rights reserved. No part of this publication may be reproduced ... without the prior permission of OCLC". OCLC's WebDewey Schedule 8 lets a subscriber post the DDC23 Summaries "solely for the internal, noncommercial purpose" with a verbatim credit line. Older CC BY-NC-ND claim: see Dewey linked data row. | See Schedule 8: subscribers' records "may display DDC numbers, but shall not display DDC captions" | https://www.oclc.org/content/dam/oclc/dewey/resources/summaries/deweysummaries.pdf ; https://www.oclc.org/content/dam/legal/schedules/en_us/Schedule-08-WebDewey-EN-US.pdf |
| Dewey linked data | Free "base set" viewable only. Quote: "Anyone is free to view and access a base set of Dewey linked data." Full set needs WebDewey subscription. | No. OCLC Terms prohibit "scraping, retrieving, or indexing", "permanent or long-term storage", and non-authorised commercial use. No CC license mentioned there. | n/a | https://policies.oclc.org/en/terms/dewey-linked-data.html. The 2012 InfoDocket post says CC BY-NC-ND for dewey.info (secondary, old): https://www.infodocket.com/2012/06/19/new-ddc-23-dewey-decimal-classification-released-as-linked-data-at-dewey-info/ |
| dewey.info | UNVERIFIED. http://dewey.info/ and https://dewey.info/ did not respond from this machine (curl timeout, exit 28, twice). Current status unknown. | UNVERIFIED | UNVERIFIED | - |
| UDC full (Master Reference File) | No. Licensed. | No. Several licence types, all for "restricted communities ... in a non-public environment" (pre-publication, intranet, research, education, personal use). Software embedding requires a publishing licence, paid annually. The page is dated 2009. | MRF licence types exist for "personal use ... study and research". Not tested for assigning numbers. | https://udcc.org/licences_terms.htm |
| UDC Summary (about 2,600 of 70,000+ entries) | Browsable free. Export files are "available on demand" by email (Word/txt, Excel), not an open download. | CC BY-NC 4.0 "from 2026". Non-commercial only. "Any redistribution should be under the same licence." | Not addressed | https://udcsummary.info/php/index.php ; https://udcsummary.info/about.htm ; https://udcsummary.info/exports.htm |
| BISAC | Free to browse and select a code. Paid licence needed "to download the entire list for use in business systems." | No. "No part of these lists ... may be distributed or reproduced in any manner whatsoever" without BISG permission. | Yes for selecting a code for your own publication. Quote: "free service from BISG ... to select a BISAC code to categorize your publication". | https://www.bisg.org/complete-bisac-subject-headings-list |
| Thema (EDItEUR) | Yes: v1.6 codes in Excel, HTML, XML, JSON, many languages. Full DB only for EDItEUR members. | EDItEUR licence: royalty-free, copying and onward distribution allowed (para 3.2.3) with attribution and notices intact. But "You may not translate, abridge, abstract from or otherwise modify" without permission (4.2), except wholly internal systems (4.3). So unmodified redistribution only, which is not an OSI-style licence. The Thema page itself does not state a licence, so applying this to Thema is my inference from "EDItEUR Standards" including controlled vocabularies. | Yes, 3.1.2 allows implementing "whether for commercial use or otherwise" | https://www.editeur.org/151/Thema/ ; https://doi.org/10.4400/nwgj (resolves to the EDItEUR IPR licence PDF) |
| ACM CCS 2012 | HTML tree free; a SKOS file existed per the archived ACM page. Live acm.org / dl.acm.org returned HTTP 403 (Cloudflare) to me, so I used the Wayback copy dated 2024-01. | No license stated. Quote: "freely available for educational and research purposes in HTML format." | Not addressed | https://web.archive.org/web/2023/https://www.acm.org/publications/class-2012 ; https://web.archive.org/web/2023/https://dl.acm.org/ccs |
| Colon Classification | UNVERIFIED. No machine-readable open dataset found. | UNVERIFIED. Ranganathan lived 1892-1972 (HathiTrust record via search). Term-of-life arithmetic is my inference, not sourced. | UNVERIFIED | - |
| Bliss BC2 | Some schedules online as PDF/DOCX on the BCA site; "not all currently available in computer format" (via search snippet; I did not re-fetch). | No. Pages read: "Copyright © Bliss Classification Association ... All rights reserved." | Not addressed | https://blissclassification.org.uk/bchist.shtml ; https://blissclassification.org.uk/bclink.shtml |
| ILC2 (Integrative Levels Classification) | Yes: Turtle, N-Triples, RDF/XML SKOS, 9,174 concepts (my count of `skos:Concept` in the .ttl) | CC BY 3.0 declared inside the TTL: `cc:license <http://creativecommons.org/licenses/by/3.0>`. Caution: BARTOC's record says BY-SA 3.0 (conflict, see below). | Yes under BY (attribution) | https://www.iskoi.org/ilc/skos.php ; https://www.iskoi.org/ilc/2/skos/ILC2.ttl ; https://bartoc.org/en/node/472 |

Notes on the table:
- BARTOC record for ILC: I fetched the API result and it listed `license ... creativecommons.org/licenses/by-sa/3.0/`, while the TTL itself says `by/3.0`. Treat as unresolved; ask the maintainers (udcs/Gnoli) or use the file's own statement with attribution.
- UDC Summary previously described as "Creative Commons 3.0" by search snippets; the live page now says CC BY-NC 4.0. NC is the blocker for an open tool.
- The FreeLCC page itself says earlier editions "should not be used for cataloging".

## 2. LCC licence question (17 U.S.C. 105)

- Statute, 105(a): "Copyright protection under this title is not available for any work of the United States Government, but the United States Government is not precluded from receiving and holding copyrights transferred to it by assignment, bequest, or otherwise." Source: https://www.law.cornell.edu/uscode/text/17/105 (Cornell also shows newer 105(b)/(c) text I did not analyse).
- LC's position (Classification Web legal notice, quote): "The classification, subject headings, and bibliographic records in Classification Web Plus that originate with the Library of Congress are copyrighted, except within the USA." and "It is your obligation to determine and satisfy copyright or other use restrictions when publishing or otherwise distributing material found on this site." https://classweb.loc.gov/legal.html
- id.loc.gov quote: "The Library of Congress has prepared this linked data system and is making it available as a public domain data set." https://id.loc.gov/about/ This is the strongest primary statement for LCC and LCSH data in linked-data form. It does not grant rights outside the US; it just states public domain.
- I found no primary LC page that expressly says "LCC schedules are public domain under 105". The practical position (public domain in the US, possibly claimed abroad) is an inference from the two quotes above. Not legal advice.

## 3. Most complete open LCC dataset

Three real sources, none perfect.

1. MDSConnect Classification, MARC 21 Classification Data, free bulk. Best for a complete offline copy.
   - Page: https://www.loc.gov/cds/products/MDSConnect-classification.html . Files listed there: `Class.2016.part01.xml.gz` (32,816,878 bytes), `Class.2016.part01.utf8.gz` (29,879,560), and older `Class.thru.d130228.records.{xml,utf8,marc8}.gz` (xml 31,654,920). Base path https://www.loc.gov/cds/downloads/MDSConnect/ . All returned HTTP 200 with those sizes. Last-modified: 2016 set 2019-06-21; older set 2017-03-06.
   - I downloaded the 2016 XML and counted: 649,332 `<record>` lines; 649,328 carry a tag 153 (class number plus caption hierarchy in `$h`), 14,179 carry tag 253 (see-references/table expansions). 536 distinct alphabetic class prefixes; all 21 letter classes present. Counts by first letter in 153: P 105,624; K 73,306; D 41,146; B 40,389; G 21,468; Q 21,054; H 20,708; T 18,839; Z 18,507; N 16,422; F 12,460; R 12,433; L 11,417; J 10,513; S 10,366; M 8,368; C 6,049; V 5,961; U 4,948; E 4,654; A 2,018.
   - Caveat: it is a 2016 snapshot, so 10 years stale. The data.gov catalog entry (https://catalog.data.gov/dataset/classification-0f547) links `Class.thru.d130228.xml.gz`, which returns 404 today. The real file name is `...records.xml.gz`.
   - Licence: only "primarily for research and development usage" (https://www.loc.gov/cds/products/marcDist.php). No explicit licence file. Pair with the id.loc.gov public-domain statement to judge redistribution.
   - Sample record: `153 $a E585.I75 $h United States $h The Civil War ... $j Irish`. This is the full-detail Cutter-level data, not just outline.
2. id.loc.gov Linked Data Service, current but per-URI.
   - Sitemap index lists 51 classification sitemaps: https://id.loc.gov/sitemapindex.xml , e.g. https://id.loc.gov/sitemaps/authorities/classification/0001.xml . Sitemaps 0001-0050 have 50,000 `loc` entries each; 0051 has 26,846. Total about 2,526,846 URIs (includes spans like `QA75.5-QA76.95`, cutter ranges, and opaque ids like `pt21294940`, `cf91006787`, `ct96212087` that I did not map to a meaning). Lastmod 2026-10-04.
   - Each URI serves RDF/XML (`.skos.rdf`), JSON (`.json`), `.nt` etc. Verified for `QA76` (skos:notation, prefLabel "General works, treatises, and textbooks", broader `QA75.5-QA76.95`).
   - There is NO bulk classification dump on https://id.loc.gov/download/ . Headings there are: LCNAF, LCSH, Children's, LCGFT, LCMPT, LCDGT, BIBFRAME Hubs, Cataloging, Relators, MARC Languages/Countries/GAC, ISO639, Cultural Heritage Orgs, TGM, AFS, RBMS, Extra RDF, and Experimental Embeddings by LCC. The embeddings are of bibliographic records grouped by LCC letter (2 GB-32 GB each), NOT the schedules.
   - So a harvest needs about 2.5M HTTP requests. Rate limits and acceptable use: UNVERIFIED (not found).
3. Outline-level JSON from the free PDFs: https://github.com/thisismattmiller/lcc-pdf-to-json , MIT code, last push 2020-04-02, Python 2.7. `results.json` (2.3 MB, fetched): 497 prefix keys, about 8,212 entries (sum of list lengths). Fields: id, parents, prefix, start, stop, subject. README says some classmarks were excluded or changed (`problems.json`). The underlying text is LC's, so the MIT label covers the code only. Other derivatives named by search, not inspected: cclauss/library-of-congress-classifier-hack, sethwoodworth/LCC.

Completeness vs outline:
- Outline: 21 letter classes (A-Z minus I, O, W, X, Y). Outline PDFs are the "first tier" only. LC outline index: https://www.loc.gov/catdir/cpso/lcco/
- Full: about 649k MARC class records in the 2016 snapshot; about 2.5M URIs on id.loc.gov today (different granularity: id.loc.gov carries cutter and span entries separately). So the full data is roughly 100x-300x the community outline JSON (8,212 entries). I did not find an authoritative single "number of LCC classes" statement; these are my own counts from the files above.

## 4. Crosswalks

All Wikidata counts are from live SPARQL (query.wikidata.org) on 2026-10-09. Wikidata data is CC0 (https://www.wikidata.org/wiki/Wikidata:Licensing , "All data in Wikidata has a CC0 license").

| Mapping | Entries | How measured | Open? |
|---|---|---|---|
| Wikidata P1036 (Dewey Decimal Classification) | 24,611 statements on 18,983 items | `COUNT(*)`, `COUNT(DISTINCT ?s)` | Yes, CC0. Note: this encodes DDC numbers in an open DB, whereas OCLC calls the DDC copyrighted. I did not find OCLC addressing it. |
| Wikidata P1149 (Library of Congress Classification) | 6,166 statements on 5,752 items | same | Yes, CC0 |
| Items with BOTH P1036 and P1149 | 942 | SPARQL join | Yes. This is the only open DDC-LCC concordance I measured, and it is tiny. |
| Wikidata P1190 (Universal Decimal Classification) | 467 statements on 451 items | same | Yes, CC0 |
| Wikidata P244 (Library of Congress Authorities ID, i.e. LCNAF and LCSH ids) | 1,764,521 statements | `COUNT(*)` | Yes, CC0 |
| Wikidata P2163 (FAST ID) | 467,319 statements on 460,913 items | same | Yes, CC0 |
| Items with FAST (P2163) AND LC authority (P244) | 460,913 | SPARQL join | Yes. Essentially all FAST-bearing items also carry a LC id. |
| Items with FAST (P2163) AND DDC (P1036) | 3,000 | SPARQL join | Yes |
| LC `externallinks.nt` (LC authority to external sources): total triples | 16,606,638 triples (2.2 GB uncompressed; 287 MB zip) | I downloaded and parsed https://id.loc.gov/download/externallinks.nt.zip | Public-domain statement applies to the service. |
| ...LCSH headings (sh ids) with a FAST link (close or broader) | 345,759 headings (353,747 close + 181,594 broader link triples) out of about 468,135 subject authorities, about 74 percent | grep/awk on the file | same |
| ...LCSH headings with a Wikidata link | 58,755 | same | same |
| ...LC names (LCNAF) with a Wikidata link | 1,704,086 | same | same |
| LCSH headings carrying an LCC number (`madsrdf:classification`) | 91,581 headings (100,952 link triples), about 19.6 percent of 468,135 | parsed `subjects.madsrdf.nt.gz` (this is the 053 field of subject authority records) | Yes, bulk public file https://id.loc.gov/download/authorities/subjects.madsrdf.nt.gz |
| LCSH to FAST in the bulk LCSH dump | 0. The FAST links live only in `externallinks.nt` and per-record views. | grep of the dump found no `id.worldcat.org/fast` string | - |
| LCC classification URIs with external links | 2 | externallinks.nt | essentially none |
| DDC to LCSH associations | exist inside Dewey Linked Data ("associates Dewey numbers with Library of Congress Subject Headings"). Count not stated. | https://help.oclc.org/Metadata_Services/WebDewey/WebDewey_Linked_Data/About_Dewey_Linked_Data | Base set only free to view; storage banned by the Terms. NOT usable as an open crosswalk. |
| OCLC Classify (work-level Dewey/LCC numbers from WorldCat) | "more than 46 million work summaries" but DISCONTINUED 31 January 2024 | https://www.oclc.org/research/areas/data-science/classify.html | gone |
| UDC to anything | No open UDC crosswalk found. UDC Summary site has a Mappings tab in its nav, but https://udcsummary.info/mappings.htm returned 404, so contents UNVERIFIED. | - | - |
| LCC to DDC bulk concordance | Not found as a download. Search returned lookup guides only. A derived concordance is feasible from co-occurring 050 (LCC) and 082 (DDC) fields in the MDSConnect open-access bibliographic records (MARC bib set, about 12M records per the lc_data_dump README: https://github.com/reeset/lc_data_dump ; I did not download or verify this). Using DDC numbers derived from LC records still raises the OCLC-copyright question for DDC. | - | - |

## 5. Bottom line for an open library classification

- Cleanest permissive options: LCSH and LCC data via id.loc.gov (US public-domain statement), FAST (ODC-By, attribution only), ILC2 (CC BY 3.0 in-file, tiny community), Wikidata (CC0).
- Not usable in an open tool: DDC data, BISAC list, full UDC, BC2 (all reserved), UDC Summary (NC), Thema (no modification), ACM CCS (educational/research use only).
- LCC can be fetched in full only as (a) 2016 MARC bulk, 649k records, or (b) ~2.5M per-URI fetches from id.loc.gov. There is no current single-file LCC dump. A fresh refresh is the missing piece.

## Sources (all fetched this session unless marked)

- https://www.loc.gov/aba/publications/FreeLCC/freelcc.html
- https://classweb.loc.gov/legal.html
- https://www.law.cornell.edu/uscode/text/17/105
- https://id.loc.gov/about/ ; https://id.loc.gov/download/ ; https://id.loc.gov/techcenter/ ; https://id.loc.gov/sitemapindex.xml ; https://id.loc.gov/sitemaps/authorities/classification/0001.xml ; https://id.loc.gov/authorities/classification/QA76.skos.rdf ; https://id.loc.gov/download/externallinks.nt.zip ; https://id.loc.gov/download/authorities/subjects.madsrdf.nt.gz
- Note: WebFetch got 403 from id.loc.gov; curl with a browser User-Agent got 200. https://id.loc.gov/authorities/classification.html returned 403 to curl.
- https://www.loc.gov/cds/products/MDSConnect-classification.html ; https://www.loc.gov/cds/products/marcDist.php ; https://www.loc.gov/cds/downloads/MDSConnect/Class.2016.part01.xml.gz ; https://catalog.data.gov/dataset/classification-0f547 ; https://www.loc.gov/catdir/cpso/lcco/
- https://github.com/thisismattmiller/lcc-pdf-to-json ; https://raw.githubusercontent.com/thisismattmiller/lcc-pdf-to-json/master/results.json ; https://github.com/reeset/lc_data_dump
- https://www.oclc.org/research/areas/data-science/fast/download.html ; https://www.oclc.org/research/areas/data-science/fast.html
- https://www.oclc.org/en/dewey/resources ; https://www.oclc.org/en/dewey/resources/linkeddata.html ; https://www.oclc.org/en/dewey/resources/licensing.html ; https://policies.oclc.org/en/terms/dewey-linked-data.html ; https://www.oclc.org/content/dam/legal/schedules/en_us/Schedule-08-WebDewey-EN-US.pdf ; https://www.oclc.org/content/dam/oclc/dewey/resources/summaries/deweysummaries.pdf ; https://help.oclc.org/Metadata_Services/WebDewey/WebDewey_Linked_Data/About_Dewey_Linked_Data ; https://www.infodocket.com/2012/06/19/new-ddc-23-dewey-decimal-classification-released-as-linked-data-at-dewey-info/ ; https://www.oclc.org/research/areas/data-science/classify.html
- https://udcsummary.info/php/index.php ; https://udcsummary.info/about.htm ; https://udcsummary.info/exports.htm ; https://udcc.org/licences_terms.htm
- https://www.bisg.org/complete-bisac-subject-headings-list
- https://www.editeur.org/151/Thema/ ; https://doi.org/10.4400/nwgj
- https://web.archive.org/web/2023/https://www.acm.org/publications/class-2012 ; https://web.archive.org/web/2023/https://dl.acm.org/ccs
- https://blissclassification.org.uk/bchist.shtml ; https://blissclassification.org.uk/bclink.shtml
- https://www.iskoi.org/ilc/skos.php ; https://www.iskoi.org/ilc/2/skos/ILC2.ttl ; https://bartoc.org/en/node/472 (and its API record)
- https://www.wikidata.org/wiki/Wikidata:Licensing ; https://query.wikidata.org/sparql (queries run: counts for P1036, P1149, P1190, P244, P2163 and joins)

UNVERIFIED / not retrieved:
- https://dewey.info/ (no response from this machine)
- Classification Web Plus Terms and Conditions PDF
- https://acm.org and https://dl.acm.org/ccs live (HTTP 403 Cloudflare; Wayback copy used)
- https://udcc.org/index.php/site/page?view=about_udc (HTTP 404)
- Colon Classification copyright and any dataset
- Thema code total (about 3,000 appears only in search-result text, not on a fetched primary page)
- id.loc.gov bulk-harvest rate limits or acceptable-use policy
- the 14,179 tag-253 records' exact meaning (assumed see-references/table expansions from MARC Classification format; not checked)
