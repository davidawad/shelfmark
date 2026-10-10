# Scheme benchmark results, 2026-10-10

Every scheme is scored as a hierarchy over a fixed similarity graph: Dasgupta cost and Moseley-Wang revenue (both normalized to [0, 1]; lower cost and higher revenue are better), skill relative to the average-linkage baseline built on the same graph (`(1 - cost_norm) / (1 - ref_cost_norm)`; 1.0 matches the baseline), placement coverage, and structure and navigation statistics. Items without a placement attach to the root, so low coverage pushes cost toward 1.

- Corpus: 10,000 arXiv papers, `corpus.jsonl` sha256 `7685685d0de25d6e8a614859783dcb5376d81e4fe336a65f22b550f4c7527b8d`.
- Holdout: 500 Open Library editions, `holdout.jsonl` sha256 `3e057eb4a257488ca1668a1ed370306e74b5b21a79cf7b4fdbd2f5b21873c833`.
- Model: `sentence-transformers/all-MiniLM-L6-v2` (fastembed 0.9.0 / ONNX Runtime CPU), weights `Qdrant/all-MiniLM-L6-v2-onnx` revision `d13954661f83248295ba75c1ed411eef3b7b936e`, `model.onnx` sha256 `bbd7b466f6d58e646fdc2bd5fd67b2f5e93c0b687011bd4548c420f7bd46f0c5`.
- Graph: k = 20 mean-offset kNN cosine. Average-linkage baseline cost_norm: corpus 0.1518, holdout 0.3014.
- Harness: `shelfmark-bench` at commit `092494850adc3755f09e225bfd10d76870f10df9`.

## Classifier-bias caveat

Gold and shelfmark-placed results are reported separately and should not be ranked against each other. Placed results come from shelfmark's own lexical classifier (`profile:lexical@1`), so they measure the scheme *as reached by that classifier*: a scheme with rich vocabulary links gets more and better placements than one with bare labels, independent of how well its hierarchy fits the corpus. Low-confidence placements (below the 0.8 target) are kept; fallback placements, where no tier produced any candidate and shelfmark files the item under the first top-level class, are counted as unplaced. Gold results use the labels the source supplied (arXiv categories, MSC, JEL; LC call numbers for the holdout) and cover only the items that carry them.

Column notes: `classes used` is the effective number of classes (exp of placement entropy); `nav log a=1` is the expected menu cost `sum(1 + log2 fanout)` from root to item; `nav scan` adds a linear menu scan and a scan of the final class shelf.

## Gold placements

arXiv corpus:

| scheme | version | coverage | cost_norm | revenue_norm | skill vs avg-link | classes used | mean depth | max depth | max branching | gini | nav log a=1 | nav scan |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| arxiv | 2024-04-01 | 1.0000 | 0.3333 | 0.6667 | 0.7861 | 36.4817 | 3.8976 | 4 | 797 | 0.7951 | 18.3324 | 291.2549 |
| msc | 2020 | 0.1314 | 0.9844 | 0.0156 | 0.0183 | 461.2949 | 1.3942 | 4 | 25 | 0.5983 | 15.2032 | 4375.8784 |
| jel | 2026 | 0.0002 | 1.0000 | 0.0000 | 0.0000 | 6.0000 | 1.0006 | 4 | 1 | 0.0000 | 14.2877 | 5001.5000 |

Book holdout (LC call numbers placed at the deepest covering class):

| scheme | version | coverage | cost_norm | revenue_norm | skill vs avg-link | classes used | mean depth | max depth | max branching | gini | nav log a=1 | nav scan |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| lcc-full | 2024 | 0.9220 | 0.7057 | 0.2943 | 0.4212 | 85.1981 | 5.1940 | 10 | 68 | 0.7182 | 19.7094 | 55.6390 |
| lcc | 2024 | 0.9220 | 0.7076 | 0.2924 | 0.4185 | 34.9857 | 4.1960 | 9 | 89 | 0.7907 | 18.5868 | 69.3980 |

## shelfmark-placed (profile:lexical@1)

arXiv corpus:

| scheme | version | coverage | cost_norm | revenue_norm | skill vs avg-link | classes used | mean depth | max depth | max branching | gini | nav log a=1 | nav scan |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| openalex-topics | 2026-10-09 | 0.9735 | 0.6415 | 0.3585 | 0.4227 | 77.5481 | 4.4463 | 5 | 554 | 0.8923 | 26.8417 | 310.3965 |
| unesco-thesaurus | 2026-08-31 | 0.9886 | 0.7318 | 0.2682 | 0.3162 | 124.4921 | 4.7884 | 9 | 1763 | 0.8633 | 25.2894 | 487.9767 |
| ipc | 2026.01 | 0.9941 | 0.7807 | 0.2193 | 0.2586 | 73.0291 | 4.9521 | 10 | 5974 | 0.8804 | 26.2321 | 3642.6847 |
| lcc-full | 2024 | 1.0000 | 0.7995 | 0.2005 | 0.2364 | 179.1898 | 7.0525 | 11 | 2213 | 0.8877 | 25.5919 | 742.3935 |
| ilc2 | 2019-10-17 | 1.0000 | 0.8249 | 0.1751 | 0.2065 | 21.4783 | 3.5177 | 13 | 7370 | 0.9188 | 20.1563 | 5499.9046 |
| acm-ccs | 2012 | 0.7445 | 0.8306 | 0.1694 | 0.1997 | 84.8556 | 3.6124 | 7 | 877 | 0.8606 | 23.7331 | 1476.2154 |
| fdc | 0.06 | 0.7696 | 0.8337 | 0.1663 | 0.1961 | 54.5437 | 3.2127 | 7 | 1103 | 0.8362 | 25.5635 | 1413.4729 |
| eurovoc | 4.24 | 0.9744 | 0.8483 | 0.1517 | 0.1788 | 271.4245 | 4.9803 | 8 | 350 | 0.8200 | 24.7547 | 264.7705 |
| lcc | 2024 | 0.8840 | 0.8484 | 0.1516 | 0.1788 | 129.4629 | 4.1087 | 10 | 659 | 0.8374 | 25.4687 | 787.6918 |
| arxiv | 2024-04-01 | 0.4260 | 0.8912 | 0.1088 | 0.1282 | 14.5259 | 2.2150 | 4 | 1665 | 0.8558 | 20.3414 | 3231.3667 |
| jel | 2026 | 0.5171 | 0.9015 | 0.0985 | 0.1161 | 33.0759 | 2.4011 | 4 | 967 | 0.7960 | 19.8550 | 2689.4884 |
| bk | 2026-10-03 | 0.4806 | 0.9019 | 0.0981 | 0.1157 | 74.9934 | 3.1522 | 8 | 788 | 0.8414 | 20.9355 | 2736.4071 |
| msc | 2020 | 0.3916 | 0.9498 | 0.0502 | 0.0592 | 114.3688 | 1.9990 | 4 | 380 | 0.8004 | 17.8237 | 3128.4689 |

Book holdout (general schemes only):

| scheme | version | coverage | cost_norm | revenue_norm | skill vs avg-link | classes used | mean depth | max depth | max branching | gini | nav log a=1 | nav scan |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| lcc-full | 2024 | 0.6100 | 0.9170 | 0.0830 | 0.1188 | 327.3908 | 4.4400 | 11 | 13 | 0.5180 | 14.4406 | 114.7330 |
| ilc2 | 2019-10-17 | 0.5680 | 0.9340 | 0.0660 | 0.0945 | 180.7624 | 3.1060 | 11 | 39 | 0.5476 | 12.9276 | 132.3280 |
| unesco-thesaurus | 2026-08-31 | 0.1600 | 0.9888 | 0.0112 | 0.0161 | 95.2030 | 1.6560 | 7 | 8 | 0.4359 | 10.8339 | 215.5390 |
| lcc | 2024 | 0.0840 | 0.9964 | 0.0036 | 0.0052 | 63.0415 | 1.2860 | 8 | 6 | 0.3288 | 10.2067 | 236.8060 |
| fdc | 0.06 | 0.0520 | 0.9981 | 0.0019 | 0.0027 | 31.0513 | 1.1380 | 7 | 4 | 0.3141 | 10.0966 | 242.6660 |
| bk | 2026-10-03 | 0.0440 | 0.9985 | 0.0015 | 0.0021 | 30.7884 | 1.2260 | 7 | 7 | 0.4391 | 10.1429 | 242.2530 |

Placement counts (fallback = no candidate, counted as unplaced):

| scheme | corpus | placed | flagged | fallback | failed | top class | top share | top terms | classify s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 7445 | 6854 | 2555 | 0 | `10002944.10011123.10011674` | 0.1178 | performance (877) | 24.6 |
| arxiv | corpus | 4260 | 3325 | 5740 | 0 | `cs.PF` | 0.3908 | performance (1665) | 24.2 |
| bk | corpus | 4806 | 3980 | 5194 | 0 | `01.00` | 0.1640 | general (788) | 24.3 |
| eurovoc | corpus | 9744 | 9593 | 256 | 0 | `C2228` | 0.0359 | paper (350) | 26.7 |
| fdc | corpus | 7696 | 7023 | 2304 | 0 | `676` | 0.1433 | paper (1103) | 25.3 |
| ilc2 | corpus | 10000 | 10000 | 0 | 0 | `agy993` | 0.7370 | to (7370) | 39.7 |
| ipc | corpus | 9941 | 9867 | 59 | 0 | `C01G` | 0.6006 | to (5971) | 30.4 |
| jel | corpus | 5171 | 4317 | 4829 | 0 | `M53` | 0.1870 | training (967) | 29.1 |
| lcc | corpus | 8840 | 8429 | 1160 | 0 | `QA299.6-433` | 0.0745 | analysis (659) | 30.0 |
| lcc-full | corpus | 10000 | 10000 | 0 | 0 | `PL2813` | 0.2213 | to (2213) | 657.1 |
| msc | corpus | 3916 | 2999 | 6084 | 0 | `51-XX` | 0.0968 | geometry (379) | 25.4 |
| openalex-topics | corpus | 9735 | 9621 | 265 | 0 | `F21` | 0.0565 | energy (550) | 27.5 |
| unesco-thesaurus | corpus | 9886 | 9777 | 114 | 0 | `c493` | 0.1780 | is (1760) | 27.8 |
| bk | holdout | 22 | 22 | 478 | 0 | `71.32` | 0.3182 | man (7) | 1.6 |
| fdc | holdout | 26 | 26 | 474 | 0 | `750` | 0.1154 | art (3) | 1.5 |
| ilc2 | holdout | 284 | 281 | 216 | 0 | `agqs99` | 0.1373 | of (39) | 3.6 |
| lcc | holdout | 42 | 42 | 458 | 0 | `QH501-531` | 0.0714 | life (3) | 2.0 |
| lcc-full | holdout | 305 | 305 | 195 | 0 | `BQ940` | 0.0393 | a (12) | 31.1 |
| unesco-thesaurus | holdout | 80 | 78 | 420 | 0 | `c155` | 0.0500 | time (4) | 1.8 |

Single-class captures: the lexical vocab tier matches common words against class labels and vocabulary, so in some schemes one class absorbs a large share of the corpus. These placements are kept as the classifier made them, and they shape the placed scores above: `arxiv-placed` files 39.1% of its placed items under `cs.PF` (Performance) by the term(s) "performance"; `ilc2-placed` files 73.7% of its placed items under `agy993` (to) by the term(s) "to"; `ipc-placed` files 60.1% of its placed items under `C01G` (COMPOUNDS CONTAINING METALS NOT COVERED BY SUBCLASSES C01D O) by the term(s) "to"; `lcc-full-placed` files 22.1% of its placed items under `PL2813` (To) by the term(s) "to". All 500 holdout rows are titles only (3.5 words on average, no description), which explains the low holdout coverage.

## Skipped

- `acm-ccs-gold`: no acm_ccs label in corpus is a code of the scheme (10000 rows checked); the corpus acm_ccs field (433 values on 132 papers) holds MSC-style codes and Primary/Secondary markers copied from arXiv's ACM-class field; none is an ACM CCS 2012 concept path, and no CCS 1998 to 2012 crosswalk is used

## Runtime

- graph_embedding_corpus: 6m13s for 10,000 rows (372.6 s embedding, 4 ONNX threads, shared CPU with concurrent builds) plus exact tau and kNN
- graph_embedding_holdout: about 5 s for 500 rows
- scheme_builds: 2m42s for 13 schemes (fetch, compile and build; IPC refetch and build included)
- average_linkage_baseline: 58.4 s on the corpus graph, 0.1 s on the holdout graph (computed once, reused via --reference-cost)
- placement_and_scoring: 25m12s for all jobs, 4 in parallel; lcc-full placement (416,186 classes) dominated at about 11 min, other schemes 24-40 s per 10,000 classifications
- gold_lcc_rerun: 18 s (deep LC call-number lookup through one CLI batch)
- total_wall_clock: about 45 min on 8 CPUs, 15 GB RAM
