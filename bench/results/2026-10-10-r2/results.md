# Scheme benchmark results, 2026-10-10-r2

Every scheme is scored as a hierarchy over a fixed similarity graph: Dasgupta cost and Moseley-Wang revenue (both normalized to [0, 1]; lower cost and higher revenue are better), skill relative to the average-linkage baseline built on the same graph (`(1 - cost_norm) / (1 - ref_cost_norm)`; 1.0 matches the baseline), placement coverage, and structure and navigation statistics. Items without a placement attach to the root, so low coverage pushes cost toward 1.

- Corpus: 10,000 arXiv papers, `corpus.jsonl` sha256 `7685685d0de25d6e8a614859783dcb5376d81e4fe336a65f22b550f4c7527b8d`.
- Holdout: 500 Open Library editions, `holdout.jsonl` sha256 `3e057eb4a257488ca1668a1ed370306e74b5b21a79cf7b4fdbd2f5b21873c833`.
- Model: `sentence-transformers/all-MiniLM-L6-v2` (fastembed 0.9.0 / ONNX Runtime CPU), weights `Qdrant/all-MiniLM-L6-v2-onnx` revision `d13954661f83248295ba75c1ed411eef3b7b936e`, `model.onnx` sha256 `bbd7b466f6d58e646fdc2bd5fd67b2f5e93c0b687011bd4548c420f7bd46f0c5`.
- Graph: k = 20 mean-offset kNN cosine. Average-linkage baseline cost_norm: corpus 0.1518, holdout 0.3014.
- Harness: `shelfmark-bench` at commit `092494850adc3755f09e225bfd10d76870f10df9`.
- Classifier: `shelfmark` at commit `f4ce24b78eda67a03b17e80e8aef90768d0354eb`, `profile:lexical@2`.

## Classifier-bias caveat

Gold and shelfmark-placed results are reported separately and should not be ranked against each other. Placed results come from shelfmark's own lexical classifier (`profile:lexical@2`), so they measure the scheme *as reached by that classifier*: a scheme with rich vocabulary links gets more and better placements than one with bare labels, independent of how well its hierarchy fits the corpus. Low-confidence placements (below the 0.8 target) are kept; fallback placements, where no tier produced any candidate and shelfmark files the item under the first top-level class, are counted as unplaced. Gold results use the labels the source supplied (arXiv categories, MSC, JEL; LC call numbers for the holdout) and cover only the items that carry them.

Column notes: `classes used` is the effective number of classes (exp of placement entropy); `nav log a=1` is the expected menu cost `sum(1 + log2 fanout)` from root to item; `nav scan` adds a linear menu scan and a scan of the final class shelf.

## Gold placements

Not rerun. Gold placements use the source labels, not the classifier, so the gold results in `bench/results/2026-10-10/results.json` stand; they were scored on the same graphs against the same average-linkage baselines.

## shelfmark-placed (profile:lexical@2)

arXiv corpus:

| scheme | version | coverage | cost_norm | revenue_norm | skill vs avg-link | classes used | mean depth | max depth | max branching | gini | nav log a=1 | nav scan |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| openalex-topics | 2026-10-09 | 0.9732 | 0.6414 | 0.3586 | 0.4228 | 77.5892 | 4.4457 | 5 | 554 | 0.8921 | 26.8502 | 311.1478 |
| unesco-thesaurus | 2026-08-31 | 0.9731 | 0.7210 | 0.2790 | 0.3289 | 146.6634 | 4.8982 | 9 | 535 | 0.8524 | 26.6819 | 304.6872 |
| ipc | 2026.01 | 0.9799 | 0.7690 | 0.2310 | 0.2724 | 317.7829 | 6.3145 | 10 | 794 | 0.8376 | 27.3148 | 322.1472 |
| ilc2 | 2019-10-17 | 0.9999 | 0.8189 | 0.1811 | 0.2135 | 295.0633 | 5.0901 | 13 | 417 | 0.8242 | 23.3842 | 135.6010 |
| lcc-full | 2024 | 0.9999 | 0.8228 | 0.1772 | 0.2089 | 657.1964 | 6.0593 | 11 | 121 | 0.8030 | 22.8644 | 52.2667 |
| acm-ccs | 2012 | 0.7445 | 0.8306 | 0.1694 | 0.1997 | 84.8556 | 3.6124 | 7 | 877 | 0.8606 | 23.7331 | 1476.2154 |
| fdc | 0.06 | 0.7696 | 0.8337 | 0.1663 | 0.1961 | 54.5437 | 3.2127 | 7 | 1103 | 0.8362 | 25.5635 | 1413.4729 |
| eurovoc | 4.24 | 0.9737 | 0.8479 | 0.1521 | 0.1794 | 269.3480 | 4.9761 | 8 | 350 | 0.8211 | 24.7990 | 268.7044 |
| lcc | 2024 | 0.8765 | 0.8487 | 0.1513 | 0.1783 | 129.0405 | 4.0760 | 10 | 661 | 0.8368 | 25.3558 | 824.8570 |
| arxiv | 2024-04-01 | 0.4260 | 0.8912 | 0.1088 | 0.1282 | 14.5259 | 2.2150 | 4 | 1665 | 0.8558 | 20.3414 | 3231.3667 |
| bk | 2026-10-03 | 0.4806 | 0.9019 | 0.0981 | 0.1157 | 74.9934 | 3.1522 | 8 | 788 | 0.8414 | 20.9355 | 2736.4071 |
| jel | 2026 | 0.4786 | 0.9028 | 0.0972 | 0.1146 | 28.9472 | 2.2721 | 4 | 972 | 0.8061 | 19.4099 | 2875.8535 |
| msc | 2020 | 0.3916 | 0.9498 | 0.0502 | 0.0592 | 114.3688 | 1.9990 | 4 | 380 | 0.8004 | 17.8237 | 3128.4689 |

Book holdout (general schemes only):

| scheme | version | coverage | cost_norm | revenue_norm | skill vs avg-link | classes used | mean depth | max depth | max branching | gini | nav log a=1 | nav scan |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| lcc-full | 2024 | 0.5660 | 0.9250 | 0.0750 | 0.1074 | 340.7621 | 4.1400 | 11 | 13 | 0.4917 | 13.9829 | 124.8790 |
| ilc2 | 2019-10-17 | 0.4560 | 0.9543 | 0.0457 | 0.0655 | 188.1462 | 2.9440 | 11 | 25 | 0.5274 | 12.4639 | 155.4890 |
| unesco-thesaurus | 2026-08-31 | 0.1580 | 0.9888 | 0.0112 | 0.0161 | 93.8176 | 1.6520 | 7 | 8 | 0.4392 | 10.8246 | 216.0310 |
| lcc | 2024 | 0.0840 | 0.9964 | 0.0036 | 0.0052 | 63.0415 | 1.2860 | 8 | 6 | 0.3288 | 10.2067 | 236.8060 |
| fdc | 0.06 | 0.0520 | 0.9981 | 0.0019 | 0.0027 | 31.0513 | 1.1380 | 7 | 4 | 0.3141 | 10.0966 | 242.6660 |
| bk | 2026-10-03 | 0.0440 | 0.9985 | 0.0015 | 0.0021 | 30.7884 | 1.2260 | 7 | 7 | 0.4391 | 10.1429 | 242.2530 |

Placement counts (fallback = no candidate, counted as unplaced):

| scheme | corpus | placed | flagged | fallback | failed | top class | top share | top terms | classify s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 7445 | 6854 | 2555 | 0 | `10002944.10011123.10011674` | 0.1178 | performance (877) | 44.1 |
| arxiv | corpus | 4260 | 3325 | 5740 | 0 | `cs.PF` | 0.3908 | performance (1665) | 43.4 |
| bk | corpus | 4806 | 3980 | 5194 | 0 | `01.00` | 0.1640 | general (788) | 42.1 |
| eurovoc | corpus | 9737 | 9582 | 263 | 0 | `C2228` | 0.0359 | paper (350) | 47.0 |
| fdc | corpus | 7696 | 7023 | 2304 | 0 | `676` | 0.1433 | paper (1103) | 47.4 |
| ilc2 | corpus | 9999 | 9999 | 1 | 0 | `edqam` | 0.0417 | s (417) | 60.7 |
| ipc | corpus | 9799 | 9725 | 201 | 0 | `G09B23/00` | 0.0807 | models (791) | 57.2 |
| jel | corpus | 4786 | 3854 | 5214 | 0 | `M53` | 0.2031 | training (972) | 47.1 |
| lcc | corpus | 8765 | 8320 | 1235 | 0 | `QA299.6-433` | 0.0754 | analysis (661) | 54.9 |
| lcc-full | corpus | 9999 | 9999 | 1 | 0 | `QC243.3.E5` | 0.0121 | energy (121) | 468.8 |
| msc | corpus | 3916 | 2999 | 6084 | 0 | `51-XX` | 0.0968 | geometry (379) | 42.9 |
| openalex-topics | corpus | 9732 | 9616 | 268 | 0 | `F21` | 0.0565 | energy (550) | 52.5 |
| unesco-thesaurus | corpus | 9731 | 9529 | 269 | 0 | `c155` | 0.0550 | time (535) | 47.0 |
| bk | holdout | 22 | 22 | 478 | 0 | `71.32` | 0.3182 | man (7) | 2.5 |
| fdc | holdout | 26 | 26 | 474 | 0 | `750` | 0.1154 | art (3) | 2.6 |
| ilc2 | holdout | 228 | 226 | 272 | 0 | `edqam` | 0.1096 | s (25) | 4.9 |
| lcc | holdout | 42 | 42 | 458 | 0 | `QH501-531` | 0.0714 | life (3) | 3.6 |
| lcc-full | holdout | 283 | 283 | 217 | 0 | `K23` | 0.0212 | s (6) | 26.1 |
| unesco-thesaurus | holdout | 79 | 77 | 421 | 0 | `c155` | 0.0506 | time (4) | 2.9 |

Single-class captures: the lexical vocab tier matches words of the text against class labels and vocabulary, so in some schemes one class absorbs a large share of the corpus. These placements are kept as the classifier made them, and they shape the placed scores above: `arxiv-placed` files 39.1% of its placed items under `cs.PF` (Performance) by the term(s) "performance"; `jel-placed` files 20.3% of its placed items under `M53` (Training) by the term(s) "training". All 500 holdout rows are titles only (3.5 words on average, no description), which explains the low holdout coverage.

## Before and after (profile:lexical@1 → profile:lexical@2)

Against `bench/results/2026-10-10/results.json`, same graphs, scheme files and baselines; only the classifier changed. `top share` is the largest single-class share of the placed items.

| scheme | corpus | top share before | top share after | coverage | cost_norm | skill |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 11.8% `10002944.10011123.10011674` | 11.8% `10002944.10011123.10011674` | 0.7445 → 0.7445 | 0.8306 → 0.8306 | 0.1997 → 0.1997 |
| arxiv | corpus | 39.1% `cs.PF` | 39.1% `cs.PF` | 0.4260 → 0.4260 | 0.8912 → 0.8912 | 0.1282 → 0.1282 |
| bk | corpus | 16.4% `01.00` | 16.4% `01.00` | 0.4806 → 0.4806 | 0.9019 → 0.9019 | 0.1157 → 0.1157 |
| eurovoc | corpus | 3.6% `C2228` | 3.6% `C2228` | 0.9744 → 0.9737 | 0.8483 → 0.8479 | 0.1788 → 0.1794 |
| fdc | corpus | 14.3% `676` | 14.3% `676` | 0.7696 → 0.7696 | 0.8337 → 0.8337 | 0.1961 → 0.1961 |
| ilc2 | corpus | 73.7% `agy993` | 4.2% `edqam` | 1.0000 → 0.9999 | 0.8249 → 0.8189 | 0.2065 → 0.2135 |
| ipc | corpus | 60.1% `C01G` | 8.1% `G09B23/00` | 0.9941 → 0.9799 | 0.7807 → 0.7690 | 0.2586 → 0.2724 |
| jel | corpus | 18.7% `M53` | 20.3% `M53` | 0.5171 → 0.4786 | 0.9015 → 0.9028 | 0.1161 → 0.1146 |
| lcc | corpus | 7.5% `QA299.6-433` | 7.5% `QA299.6-433` | 0.8840 → 0.8765 | 0.8484 → 0.8487 | 0.1788 → 0.1783 |
| lcc-full | corpus | 22.1% `PL2813` | 1.2% `QC243.3.E5` | 1.0000 → 0.9999 | 0.7995 → 0.8228 | 0.2364 → 0.2089 |
| msc | corpus | 9.7% `51-XX` | 9.7% `51-XX` | 0.3916 → 0.3916 | 0.9498 → 0.9498 | 0.0592 → 0.0592 |
| openalex-topics | corpus | 5.6% `F21` | 5.7% `F21` | 0.9735 → 0.9732 | 0.6415 → 0.6414 | 0.4227 → 0.4228 |
| unesco-thesaurus | corpus | 17.8% `c493` | 5.5% `c155` | 0.9886 → 0.9731 | 0.7318 → 0.7210 | 0.3162 → 0.3289 |
| bk | holdout | 31.8% `71.32` | 31.8% `71.32` | 0.0440 → 0.0440 | 0.9985 → 0.9985 | 0.0021 → 0.0021 |
| fdc | holdout | 11.5% `750` | 11.5% `750` | 0.0520 → 0.0520 | 0.9981 → 0.9981 | 0.0027 → 0.0027 |
| ilc2 | holdout | 13.7% `agqs99` | 11.0% `edqam` | 0.5680 → 0.4560 | 0.9340 → 0.9543 | 0.0945 → 0.0655 |
| lcc | holdout | 7.1% `QH501-531` | 7.1% `QH501-531` | 0.0840 → 0.0840 | 0.9964 → 0.9964 | 0.0052 → 0.0052 |
| lcc-full | holdout | 3.9% `BQ940` | 2.1% `K23` | 0.6100 → 0.5660 | 0.9170 → 0.9250 | 0.1188 → 0.1074 |
| unesco-thesaurus | holdout | 5.0% `c155` | 5.1% `c155` | 0.1600 → 0.1580 | 0.9888 → 0.9888 | 0.0161 → 0.0161 |

## Skipped

None.

## Runtime

- scheme_builds: about 2 min for 13 schemes (fetch, compile and build; every file sha256 equal to 2026-10-10)
- average_linkage_baseline: not recomputed: ref_cost_norm taken from ../2026-10-10/results.json after checking each graph_sha256
- placement_and_scoring: about 21 min for 19 placed jobs, 4 in parallel; the first launch was stopped after 10m 32s with 17 jobs done and lcc-full corpus at 4,775 of 10,000 rows, and the rerun resumed that batch and finished in 9m38s
- lcc_full_classify: about 16 min for 10,000 corpus rows over the two launches; its classify_seconds (468.8) covers only the resumed 5,225 rows
- total_wall_clock: about 25 min on 8 CPUs, 15 GB RAM, plus a workspace cargo test run sharing the CPUs during the last 9 minutes
