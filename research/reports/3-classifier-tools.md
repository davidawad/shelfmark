# Existing subject/classification tools vs shelfmark

Date: 2026-10-09. Method: web search + page fetch. "Fetched" = page loaded and read this session (via a summarizing fetch tool, so figures are as-reported by that summary; PDFs with no extractable text were not read). SNIPPET = number seen only in a search-result summary. UNVERIFIED = link or claim not confirmed by fetch.

## Comparison table

| Tool | What it does | License | Lang | Schemes | Reported accuracy (metric, dataset) | Maintenance | Link status |
|---|---|---|---|---|---|---|---|
| Annif (NatLibFi) | Vocabulary-agnostic subject indexing toolkit; train per vocabulary on indexed records; REST/CLI | Apache-2.0 (optional YAKE dep is GPLv3) | Python 3.11-3.13 | Any SKOS vocab: YSO, GND, LCSH, Brinkman; DDC in papers | No single headline number; see rows below | Active (3,505 commits, 271 stars; release date not shown) | github.com/NatLibFi/Annif fetched |
| Annif backends: tfidf, fastText, Omikuji, MLLM, STWFSA, YAKE, SVC, ensemble, PAV, nn_ensemble, HTTP | MLLM = Maui-like lexical matching, features (tf-idf, term spread, vocab relations), 10 decision trees; needs a few hundred to a few thousand training docs. Omikuji = Parabel/Bonsai label trees. nn_ensemble = learned fusion | Apache-2.0 | Python | as above | Wiki pages give no accuracy numbers | Active | Sidebar of MLLM wiki fetched. XTransformer backend NOT in that sidebar list, so its existence is UNVERIFIED |
| Annif on DDC, Swedish Libris (J. Documentation 80(5)) | Annif lexical + SVC + fastText + Omikuji Bonsai + ensemble | paper | n/a | DDC | Ensemble 66.82% accuracy at 3-digit DDC; 44.87% precision@1 at full DDC; 230k Libris records, ~12k distinct DDC classes | 2024 | emerald.com abstract fetched |
| Annif on MARC DDC (Atlantis ICMBL-24 / JGU) | Annif on ~200k MARC-21 records (082/245/520/650) | paper | n/a | DDC | F1@5 and NDCG; sources disagree on winner (NN ensemble vs Omikuji); numbers not obtained | 2024 | search snippets only; UNVERIFIED |
| Annif at SemEval-2025 Task 5 | Annif XMTC + LLM translation + synthetic data, monolingual models merged | repo CC0-1.0 (task data CC-BY-4.0) | Python | GND (TIBKAT, en/de) | Rank 1 all-subjects, rank 2 tib-core, rank 4 qualitative; P/R/F1 values not retrievable (PDF unreadable) | repo 93 commits, 1 star | arxiv.org/abs/2504.19675 and github.com/NatLibFi/Annif-LLMs4Subjects fetched |
| DNB-AI-Project at SemEval-2025 Task 5 | Few-shot prompting of several open-weight LLMs, ensemble, map keywords to GND via embeddings | not checked | not checked | GND | 4th quantitative (all-subjects), best in expert qualitative ranking (per search summary of arXiv abstract) | n/a | arxiv.org/abs/2504.21589 seen in search, not fetched; SNIPPET |
| SemEval-2025 Task 5 overview (D'Souza et al.) | Shared task: en/de TIBKAT records, GND, P/R/F1 plus expert eval | paper | n/a | GND | Abstract only: LLM ensembles, synthetic data, multilingual processing helped; leaderboard numbers not obtained | 2025 | arxiv.org/abs/2504.07199 fetched (abstract) |
| DNB Annif vs Averbis pilot (Uhlmann 2020 slides) | Compared Averbis and Annif for GND | slides | n/a | GND | F1@5 ~0.286 Averbis vs ~0.338 Annif, 434 records (value pairing inferred by tool, low confidence) | 2020 | search result; UNVERIFIED |
| Finto AI | Hosted Annif service from NLF | service | n/a | YSO etc. | no numbers retrieved | live since 2020 per JLIS.it paper | UNVERIFIED |
| KB Netherlands (KB Lab) | Annif models on Dutch e-book metadata with Brinkman thesaurus (Omikuji, fastText, ensemble 3:1; Vowpal Wabbit for genre); "Demosaurus" demo | not checked | Python | Brinkman | none found | n/a | lab.kb.nl/tool/assisted-keyword-assignment-using-annif seen in search only; UNVERIFIED. No NBC-classification work found |
| OCLC Classify | Returned most popular DDC/LCC for a work | proprietary | n/a | DDC, LCC | none published | SHUT DOWN 2024-01-31. No free replacement. Subscribers: WorldCat Metadata API endpoint for most popular LC and Dewey numbers by OCLC number; others: WorldCat.org call numbers | help.oclc.org page fetched |
| LC Labs Exploring Computational Description (ECD1/ECD2) | ~5 ML/LLM models predicting eBook metadata; HITL prototypes | n/a | n/a | MARC fields, subjects, genre | Title/author up to 99%; subject and genre below 50%; 95% automation threshold not met (IFLA WLIC 2025) | phases 1-2 done | repository.ifla.org/handle/20.500.14598/6861 fetched; labs.loc.gov/work/experiments/ECD fetch timed out (UNVERIFIED) |
| Qualle (ZBW) | Not a classifier; estimates per-document recall of a multi-label result | Apache-2.0 | Python 3.10-3.13 | scheme-agnostic (built around STW) | n/a | 447 commits, 7 stars; last date not shown | github.com/zbw/qualle fetched |
| Magpie (INSPIRE/CERN) | NN multi-label tagging and keyword extraction for HEP abstracts | MIT | Python | HEP categories (custom-trained) | none found | 196 commits, 47 open issues, last date not shown; likely dormant (UNVERIFIED) | github.com/inspirehep/magpie fetched |
| Omikuji | Parabel/Bonsai partitioned label trees, XMC | MIT | Rust + Python binding | any label set | P@1: EURLex-4K 82.1 (Parabel mode)/83.0 (Bonsai mode); Amazon-670K 44.8/45.6; WikiLSHTC-325K 64.8/66.6 | stable; release date not shown | github.com/tomtung/omikuji fetched |
| PECOS (Amazon) incl. XR-Transformer, X-Linear | XMC framework | Apache-2.0 | Python/C++ | any label set | see X-Transformer | 221 commits, 555 stars, no releases listed | github.com/amzn/pecos fetched |
| X-Transformer (KDD 2020) | Fine-tune transformers for XMC | paper | n/a | any | P@1 77.28% Wiki-500K vs Parabel 68.70% | frozen | arxiv.org/abs/1905.02331 fetched |
| auto_lcc (ahmad-PH) | NB/SVM/MLP/LSTM on title+synopsis embeddings to LCC | MIT | Python | LCC top-level letters, 15 of 21 classes | LSTM + BERT 76.03% accuracy; MLP+BERT 75.60%; SVC+BERT 75.13%; NB TF-IDF 33.87%; MARC21 records | 140 commits, last date not shown | github.com/ahmad-PH/auto_lcc fetched |
| LLM LCC classifiers (J. Information Science, Llama3 best of 6) | LLM classifiers into LCC at several granularities | paper | n/a | LCC | Abstract: struggles on fine-grained classes and long text; numbers need full paper | 2026 | synapsesocial abstract fetched; DOI 10.1177/01655515261425547 not fetched |
| Title-only LCC (ICCED 2025, Project Gutenberg) | TF-IDF+LR vs DistilBERT, one-stage vs two-stage (letter then two-letter class) | paper | n/a | LCC 1-2 letters | DistilBERT best Macro-F1; no number retrieved | 2025 | SNIPPET; scholar.unair.ac.id fetch failed (DNS); UNVERIFIED |
| Frank and Paynter (JASIST 2004) | LCSH sets to LCC via ML | paper | n/a | LCC from LCSH | tested on 50,000 LCSH/LCC pairs; numbers not obtained | 2004 | SNIPPET; UNVERIFIED |
| Washington NASKO: ChatGPT and Copilot cataloging | Manual evaluation for LCSH, Sears, LCC, DDC, Cutter | paper | n/a | LCC, DDC | Qualitative only: lacks specificity, wrong classes; no statistics | 2025 | journals.lib.washington.edu fetched |
| LLM4DDC (Mannheim) | Prompted LLMs vs fine-tuned RoBERTa on research-data DDC | models/code on GitHub | Python | DDC | Qualitative: prompted LLMs worse than fine-tuned small models; no numbers in abstract | 2025 | madoc.bib.uni-mannheim.de/71151 fetched |
| Hawaii ChatGPT course mapping | GPT-4 with reference data, courses to LCC | paper | n/a | LCC | precision 73%; hallucination 48% to 4% | 2024 | SNIPPET; UNVERIFIED |
| IMLS proposal (LLM LCC) | zero-/few-shot GPT-3.5: 0.48 / 0.57 on 21 first-level classes; fine-tuned up to 0.85 | proposal | n/a | LCC | as stated | n/a | SNIPPET (imls.gov); UNVERIFIED |
| 110-book DDC study (ChatGPT-4o, DeepSeek, Gemini 2.0) | LLM DDC assignment on 110 titles | paper | n/a | DDC | mean match score 56.43 (DeepSeek), 51.82 (ChatGPT-4o), 45.73 (Gemini); good through 2nd-3rd digit | ~2025 | SNIPPET; UNVERIFIED |
| Calibre "Library Codes" plugin | Per-book LCC/DDC lookup (v2.0.1, 2024-07, uses LoC SRU + MODS), maps to genre | not checked | Python | LCC, DDC | n/a (lookup) | v2.0.2 2024-07-22; a Feb 2024 forum post said the earlier OCLC route broke | MobileRead threads in search only; UNVERIFIED |
| Zotero | Imports call numbers from MARC 050/082/090; no classifier plugin found | not checked | JS | n/a | n/a | n/a | forum snippets only |
| Open Library / Internet Archive | No evidence of LCC/DDC inference from text; they hold cataloger-supplied classes and subjects | n/a | n/a | n/a | n/a | n/a | nothing found; absence of evidence, not proof |
| text2ddc | fastText DDC tool; F 87% (level 2), 78% (level 3) per a search summary | ? | ? | DDC | as stated | ? | repo not found; UNVERIFIED |

## What shelfmark should reuse or benchmark against

1. Benchmark against Annif first. It is the one real incumbent. Its MLLM backend is the same idea as shelfmark's lexical vocabulary match (candidates from vocab labels, learned ranker). Run Annif (tfidf, MLLM, Omikuji, ensemble) on the same 104-item gold set and compare top-1 and F1@5. Caveat: Annif needs training records per scheme; shelfmark's pitch is no per-scheme training, so report both "Annif trained on N records" and shelfmark.
2. Context numbers (different data, not comparable): Libris DDC 3-digit 66.82% (Annif ensemble); auto_lcc 76% at 15 top-level LCC letters. shelfmark's 52% is at LCC subclass (hundreds of classes), a harder level; no published subclass-level LCC top-1 found.
3. Reuse: Annif's metrics and corpus format; Omikuji (Rust, MIT, same stack) as an optional learned backend if MARC training data is used; Qualle-style quality/recall estimation to decide when shelfmark abstains; the SemEval trick of LLM translation/synthetic training data.
4. Data: LoC MARC distribution records (050 = LCC, 082 = DDC); the MARC21 set behind auto_lcc (Google Drive link in its README). Library Codes' route (LoC SRU/MODS) shows free LoC lookup is viable after Classify.
5. LLM baseline: run one zero-shot top-3 LCC prompt on the gold set. Published LLM results are weak at fine granularity (see rows above), so a retrieval-constrained approach should beat naive prompting.

## Is there a gap shelfmark fills

Yes, narrowly. I found no open, CPU-first tool that (a) takes an arbitrary scheme as a data file with no per-scheme training, (b) chains identifier lookup, lexical match, rules, and embedding shortlist, and (c) targets LCC/DDC placement of individual books.
- Annif is closest and mature, but trained per vocabulary and framed as multi-label subject indexing (F1@5), not single best class placement. DDC/LCC use is research-paper territory.
- OCLC Classify was the only service doing book-level DDC/LCC lookup; shut down 2024-01-31, no free replacement. Vacated niche.
- GitHub LCC classifier found (auto_lcc) covers only 15 of 21 top-level letters, not subclasses.
- LLM work (2023-2026) is mostly evaluation papers, weak at fine granularity; no packaged tool.
- LC's own ECD experiment: subject/genre under 50%, not automatable at 95%. 52% at subclass is plausibly near state of the art, though metrics are not comparable.
- No public LCC/DDC benchmark with a class/subclass leaderboard found (Open Library, HathiTrust, WorldCat: none). Publishing the gold set, ideally enlarged from LoC MARC, would fill a gap itself.

Limitations: one US-only search pass. Not confirmed: Annif XTransformer backend, DNB production indexing details (only a 2020 slide), last-release dates for PECOS/Magpie/Annif, Zotero/Open Library absence.

## Sources

- https://github.com/NatLibFi/Annif (fetched)
- https://github.com/NatLibFi/Annif/wiki/Backend%3A-MLLM (fetched)
- https://github.com/NatLibFi/Annif/wiki/Backend%3A-Omikuji (fetched)
- https://github.com/NatLibFi/Annif-LLMs4Subjects (fetched)
- https://arxiv.org/abs/2504.19675 (fetched)
- https://arxiv.org/abs/2504.07199 (fetched)
- https://arxiv.org/abs/2504.21589 (search result only)
- https://www.emerald.com/jd/article/80/5/1057/1236186/Automated-Dewey-Decimal-Classification-of-Swedish (fetched)
- https://help.oclc.org/Librarian_Toolbox/Troubleshooting/Why_is_OCLC_discontinuing_OCLC_Classify (fetched)
- https://repository.ifla.org/handle/20.500.14598/6861 (fetched)
- https://labs.loc.gov/work/experiments/ECD (fetch failed)
- https://github.com/zbw/qualle (fetched)
- https://github.com/inspirehep/magpie (fetched)
- https://github.com/tomtung/omikuji (fetched)
- https://github.com/amzn/pecos (fetched)
- https://arxiv.org/abs/1905.02331 (fetched)
- https://github.com/ahmad-PH/auto_lcc (fetched)
- https://www.synapsesocial.com/papers/69d893406c1944d70ce04452 (fetched)
- https://journals.lib.washington.edu/index.php/nasko/article/view/16375 (fetched)
- https://madoc.bib.uni-mannheim.de/71151/ (fetched)
- https://arxiv.org/abs/2502.15745 (fetched; FORC dataset, Llama 3.1 up to 0.82; not LCC/DDC)
- https://jlis.it/index.php/jlis/article/download/437/430/445 (search result)
- https://lab.kb.nl/tool/assisted-keyword-assignment-using-annif (search result)
- https://pure.jgu.edu.in/id/eprint/9881/ (search result)
- https://www.atlantis-press.com/proceedings/icmbl-24/126011042 (search result)
- https://scholar.unair.ac.id/en/publications/ai-based-book-classification-using-book-titles-investigating-two--2/ (fetch failed)
- https://scholarspace.manoa.hawaii.edu/items/4ca12838-3164-447d-a2c0-16d076e9f5c7 (search result)
- https://imls.gov/sites/default/files/project-proposals/lg-256666-ols-24-preliminary-proposal.pdf (search result)
- https://www.mobileread.com/forums/showthread.php?p=4439964 (search result; Library Codes plugin)
