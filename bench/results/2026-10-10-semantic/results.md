# Scheme benchmark, lexical and semantic classifier tiers, 2026-10-10-semantic

The shelfmark-placed part of the benchmark, run once per classifier profile on the same graphs, scheme files and average-linkage baselines: `lexical` = `profile:lexical@2`, `shortlist` = `profile:shortlist@2`, `rerank` = `profile:rerank@2`. Placed scores measure a scheme *as reached by a classifier*; comparing tiers on the same scheme separates what the scheme's hierarchy is worth from how much of the corpus the classifier can file into it.

- Corpus: 10,000 arXiv papers, `corpus.jsonl` sha256 `1027bf9d34928f470ed8b0d7b5ddc5c983fde49bc0cd33eaa162bbd1f830fea5`.
- Holdout: 500 Open Library editions (titles only), `holdout.jsonl` sha256 `3e057eb4a257488ca1668a1ed370306e74b5b21a79cf7b4fdbd2f5b21873c833`.
- Graph: k = 20 mean-offset kNN cosine over `sentence-transformers/all-MiniLM-L6-v2` embeddings. Average-linkage baseline cost_norm: corpus 0.1518, holdout 0.3014.
- shelfmark and harness at commit `9a130f99ca74af30732ccde1352142f7747ad818`, shelfmark built with feature `semantic-shortlist` (BGE-small-en-v1.5 int8 through ONNX Runtime).
- Intervals: 200 item-bootstrap resamples, seed 1 (item bootstrap, percentile interval; paired across runs with the same corpus and seed; scheme tree re-induced per resample; ref_cost_norm held at its full-sample value). Deltas are paired: the interval is of the per-resample difference.

## Lexical vs semantic

Skill is `(1 - cost_norm) / (1 - ref_cost_norm)`; 1.0 matches the average-linkage tree fitted to the graph. Items without a placement attach to the root.

`lexical` → `shortlist`:

| scheme | corpus | coverage lexical | coverage shortlist | skill lexical | skill shortlist | Δ skill | Δ cost_norm |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 0.7445 | 1.0000 | 0.1997 [0.1914, 0.2081] | 0.2781 [0.2706, 0.2853] | +0.0784 [+0.0705, +0.0863] | -0.0665 [-0.0732, -0.0598] |
| arxiv | corpus | 0.4260 | 1.0000 | 0.1282 [0.1204, 0.1368] | 0.5017 [0.4920, 0.5134] | +0.3734 [+0.3615, +0.3876] | -0.3167 [-0.3287, -0.3066] |
| bk | corpus | 0.4806 | 1.0000 | 0.1157 [0.1086, 0.1235] | 0.3600 [0.3510, 0.3705] | +0.2443 [+0.2345, +0.2535] | -0.2072 [-0.2150, -0.1989] |
| eurovoc | corpus | 0.9737 | 1.0000 | 0.1794 [0.1734, 0.1860] | 0.1749 [0.1699, 0.1816] | -0.0044 [-0.0083, -0.0003] | +0.0038 [+0.0003, +0.0071] |
| fdc | corpus | 0.7696 | 1.0000 | 0.1961 [0.1877, 0.2026] | 0.2581 [0.2505, 0.2644] | +0.0620 [+0.0577, +0.0667] | -0.0526 [-0.0566, -0.0489] |
| ilc2 | corpus | 0.9999 | 1.0000 | 0.2135 [0.2061, 0.2210] | 0.2441 [0.2357, 0.2548] | +0.0305 [+0.0219, +0.0420] | -0.0259 [-0.0357, -0.0186] |
| ipc | corpus | 0.9799 | 1.0000 | 0.2724 [0.2646, 0.2805] | 0.2956 [0.2877, 0.3030] | +0.0232 [+0.0170, +0.0292] | -0.0197 [-0.0248, -0.0144] |
| jel | corpus | 0.4786 | 1.0000 | 0.1146 [0.1074, 0.1232] | 0.3033 [0.2958, 0.3102] | +0.1887 [+0.1796, +0.1966] | -0.1601 [-0.1668, -0.1523] |
| lcc | corpus | 0.8765 | 1.0000 | 0.1783 [0.1706, 0.1849] | 0.2116 [0.2049, 0.2189] | +0.0332 [+0.0288, +0.0386] | -0.0282 [-0.0328, -0.0244] |
| lcc-full | corpus | 0.9999 | 1.0000 | 0.2089 [0.2016, 0.2164] | 0.2166 [0.2093, 0.2232] | +0.0077 [-0.0003, +0.0149] | -0.0065 [-0.0126, +0.0002] |
| msc | corpus | 0.3916 | 1.0000 | 0.0592 [0.0531, 0.0640] | 0.2305 [0.2209, 0.2393] | +0.1713 [+0.1631, +0.1792] | -0.1453 [-0.1520, -0.1383] |
| openalex-topics | corpus | 0.9732 | 1.0000 | 0.4228 [0.4149, 0.4316] | 0.4645 [0.4559, 0.4742] | +0.0417 [+0.0370, +0.0479] | -0.0354 [-0.0406, -0.0314] |
| unesco-thesaurus | corpus | 0.9731 | 1.0000 | 0.3289 [0.3213, 0.3366] | 0.3376 [0.3302, 0.3446] | +0.0087 [+0.0064, +0.0113] | -0.0074 [-0.0096, -0.0054] |
| bk | holdout | 0.0440 | 1.0000 | 0.0021 [0.0003, 0.0049] | 0.3411 [0.3117, 0.3658] | +0.3389 [+0.3090, +0.3645] | -0.2368 [-0.2546, -0.2158] |
| fdc | holdout | 0.0520 | 1.0000 | 0.0027 [0.0003, 0.0073] | 0.2595 [0.2277, 0.2919] | +0.2568 [+0.2233, +0.2893] | -0.1794 [-0.2021, -0.1560] |
| ilc2 | holdout | 0.4560 | 1.0000 | 0.0655 [0.0462, 0.0876] | 0.0646 [0.0444, 0.0825] | -0.0008 [-0.0285, +0.0206] | +0.0006 [-0.0144, +0.0199] |
| lcc | holdout | 0.0840 | 1.0000 | 0.0052 [0.0013, 0.0109] | 0.2548 [0.2189, 0.2954] | +0.2495 [+0.2151, +0.2892] | -0.1743 [-0.2021, -0.1503] |
| lcc-full | holdout | 0.5660 | 1.0000 | 0.1074 [0.0801, 0.1414] | 0.2112 [0.1808, 0.2483] | +0.1037 [+0.0719, +0.1398] | -0.0725 [-0.0977, -0.0503] |
| unesco-thesaurus | holdout | 0.1580 | 1.0000 | 0.0161 [0.0093, 0.0271] | 0.3071 [0.2776, 0.3367] | +0.2910 [+0.2611, +0.3232] | -0.2033 [-0.2258, -0.1824] |

`lexical` → `rerank`:

| scheme | corpus | coverage lexical | coverage rerank | skill lexical | skill rerank | Δ skill | Δ cost_norm |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 0.7445 | 1.0000 | 0.1997 [0.1914, 0.2081] | 0.2781 [0.2706, 0.2853] | +0.0784 [+0.0705, +0.0863] | -0.0665 [-0.0732, -0.0598] |
| arxiv | corpus | 0.4260 | 1.0000 | 0.1282 [0.1204, 0.1368] | 0.5017 [0.4920, 0.5134] | +0.3734 [+0.3615, +0.3876] | -0.3167 [-0.3287, -0.3066] |
| bk | corpus | 0.4806 | 1.0000 | 0.1157 [0.1086, 0.1235] | 0.3600 [0.3510, 0.3705] | +0.2443 [+0.2345, +0.2535] | -0.2072 [-0.2150, -0.1989] |
| eurovoc | corpus | 0.9737 | 1.0000 | 0.1794 [0.1734, 0.1860] | 0.1749 [0.1699, 0.1816] | -0.0044 [-0.0083, -0.0003] | +0.0038 [+0.0003, +0.0071] |
| fdc | corpus | 0.7696 | 1.0000 | 0.1961 [0.1877, 0.2026] | 0.2581 [0.2505, 0.2644] | +0.0620 [+0.0577, +0.0667] | -0.0526 [-0.0566, -0.0489] |
| ilc2 | corpus | 0.9999 | 1.0000 | 0.2135 [0.2061, 0.2210] | 0.2441 [0.2357, 0.2548] | +0.0305 [+0.0219, +0.0420] | -0.0259 [-0.0357, -0.0186] |
| ipc | corpus | 0.9799 | 1.0000 | 0.2724 [0.2646, 0.2805] | 0.2956 [0.2877, 0.3030] | +0.0232 [+0.0170, +0.0292] | -0.0197 [-0.0248, -0.0144] |
| jel | corpus | 0.4786 | 1.0000 | 0.1146 [0.1074, 0.1232] | 0.3033 [0.2958, 0.3102] | +0.1887 [+0.1796, +0.1966] | -0.1601 [-0.1668, -0.1523] |
| lcc | corpus | 0.8765 | 1.0000 | 0.1783 [0.1706, 0.1849] | 0.2116 [0.2049, 0.2189] | +0.0332 [+0.0288, +0.0386] | -0.0282 [-0.0328, -0.0244] |
| lcc-full | corpus | 0.9999 | 1.0000 | 0.2089 [0.2016, 0.2164] | 0.2166 [0.2093, 0.2232] | +0.0077 [-0.0003, +0.0149] | -0.0065 [-0.0126, +0.0002] |
| msc | corpus | 0.3916 | 1.0000 | 0.0592 [0.0531, 0.0640] | 0.2305 [0.2209, 0.2393] | +0.1713 [+0.1631, +0.1792] | -0.1453 [-0.1520, -0.1383] |
| openalex-topics | corpus | 0.9732 | 1.0000 | 0.4228 [0.4149, 0.4316] | 0.4645 [0.4559, 0.4742] | +0.0417 [+0.0370, +0.0479] | -0.0354 [-0.0406, -0.0314] |
| unesco-thesaurus | corpus | 0.9731 | 1.0000 | 0.3289 [0.3213, 0.3366] | 0.3376 [0.3302, 0.3446] | +0.0087 [+0.0064, +0.0113] | -0.0074 [-0.0096, -0.0054] |
| bk | holdout | 0.0440 | 1.0000 | 0.0021 [0.0003, 0.0049] | 0.3411 [0.3117, 0.3658] | +0.3389 [+0.3090, +0.3645] | -0.2368 [-0.2546, -0.2158] |
| fdc | holdout | 0.0520 | 1.0000 | 0.0027 [0.0003, 0.0073] | 0.2595 [0.2277, 0.2919] | +0.2568 [+0.2233, +0.2893] | -0.1794 [-0.2021, -0.1560] |
| ilc2 | holdout | 0.4560 | 1.0000 | 0.0655 [0.0462, 0.0876] | 0.0646 [0.0444, 0.0825] | -0.0008 [-0.0285, +0.0206] | +0.0006 [-0.0144, +0.0199] |
| lcc | holdout | 0.0840 | 1.0000 | 0.0052 [0.0013, 0.0109] | 0.2548 [0.2189, 0.2954] | +0.2495 [+0.2151, +0.2892] | -0.1743 [-0.2021, -0.1503] |
| lcc-full | holdout | 0.5660 | 1.0000 | 0.1074 [0.0801, 0.1414] | 0.2112 [0.1808, 0.2483] | +0.1037 [+0.0719, +0.1398] | -0.0725 [-0.0977, -0.0503] |
| unesco-thesaurus | holdout | 0.1580 | 1.0000 | 0.0161 [0.0093, 0.0271] | 0.3071 [0.2776, 0.3367] | +0.2910 [+0.2611, +0.3232] | -0.2033 [-0.2258, -0.1824] |

## Coverage and skill

Correlation of coverage with skill across the placed runs of each tier (corpus: 13 schemes; holdout: the 6 general schemes). Intervals are over paired bootstrap replicates; a correlation is undefined in a replicate where every coverage is equal.

| tier | runs | n | Spearman | Pearson | undefined rounds |
| --- | --- | ---: | ---: | ---: | ---: |
| lexical | corpus | 13 | 0.762 [0.714, 0.835] | 0.750 [0.735, 0.763] | 0 |
| lexical | holdout | 6 | 1.000 [0.829, 1.000] | 0.986 [0.937, 0.998] | 0 |
| lexical | all | 19 | 0.907 [0.889, 0.933] | 0.883 [0.876, 0.891] | 0 |
| shortlist | corpus | 13 | n/a [n/a, n/a] | n/a [n/a, n/a] | 200 |
| shortlist | holdout | 6 | n/a [n/a, n/a] | n/a [n/a, n/a] | 200 |
| shortlist | all | 19 | n/a [n/a, n/a] | n/a [n/a, n/a] | 200 |
| rerank | corpus | 13 | n/a [n/a, n/a] | n/a [n/a, n/a] | 200 |
| rerank | holdout | 6 | n/a [n/a, n/a] | n/a [n/a, n/a] | 200 |
| rerank | all | 19 | n/a [n/a, n/a] | n/a [n/a, n/a] | 200 |

Correlation of the coverage gain with the skill gain over `lexical`, run by run (a tier that files every item has constant coverage, so its own coverage/skill correlation is undefined; this asks instead whether the schemes where it adds the most coverage are the ones where skill rises most):

| tiers | runs | n | Spearman | Pearson | undefined rounds |
| --- | --- | ---: | ---: | ---: | ---: |
| lexical → shortlist | corpus | 13 | 0.861 [0.775, 0.890] | 0.896 [0.888, 0.903] | 0 |
| lexical → shortlist | holdout | 6 | 0.771 [0.543, 0.886] | 0.874 [0.805, 0.919] | 0 |
| lexical → shortlist | all | 19 | 0.801 [0.742, 0.840] | 0.845 [0.815, 0.864] | 0 |
| lexical → rerank | corpus | 13 | 0.861 [0.775, 0.890] | 0.896 [0.888, 0.903] | 0 |
| lexical → rerank | holdout | 6 | 0.771 [0.543, 0.886] | 0.874 [0.805, 0.919] | 0 |
| lexical → rerank | all | 19 | 0.801 [0.742, 0.840] | 0.845 [0.815, 0.864] | 0 |

Skill-rank agreement with `lexical` (Spearman over schemes): `shortlist` corpus 0.137, holdout -0.771; `rerank` corpus 0.137, holdout -0.771.

Identical placements: `lexical=shortlist`: 0 of 19 placement files byte-identical; `lexical=rerank`: 0 of 19 placement files byte-identical; `shortlist=rerank`: 19 of 19 placement files byte-identical.

Baseline check: `lexical` coverage, cost_norm and skill against `bench/results/2026-10-10-r2`: 0 of 19 runs differ.

## `lexical` (profile:lexical@2): all metrics

Value [95% interval]. `classes used` is exp of placement entropy; `nav log a=1` is the expected `sum(1 + log2 fanout)` menu cost from root to item; `nav scan` adds a linear menu scan and a scan of the final class shelf. † marks an interval that excludes the full-sample value. A resample holds about 63% distinct items, so classes with few items drop out of it; statistics of the set of used classes (classes used, gini, and the navigation costs, which depend on fan-out) are biased in resamples, and their intervals show resampling spread around a shifted centre. `results.json` gives every metric's bootstrap bias (`bootstrap_bias`, resample mean minus full-sample value). Coverage, cost_norm, revenue_norm and skill are averages over items and pairs and are not affected this way.

| scheme | corpus | coverage | cost_norm | revenue_norm | skill | classes used | mean depth | gini | nav log a=1 | nav scan |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 0.7445 [0.7357, 0.7527] | 0.8306 [0.8235, 0.8377] | 0.1694 [0.1623, 0.1765] | 0.1997 [0.1914, 0.2081] | 84.9 [81.2, 86.1] | 3.61 [3.58, 3.65] | 0.861 [0.838, 0.849] † | 23.73 [23.48, 23.78] | 1476.2 [1443.7, 1514.7] |
| arxiv | corpus | 0.4260 [0.4166, 0.4368] | 0.8912 [0.8840, 0.8979] | 0.1088 [0.1021, 0.1160] | 0.1282 [0.1204, 0.1368] | 14.5 [14.1, 14.9] | 2.21 [2.18, 2.25] | 0.856 [0.837, 0.854] † | 20.34 [20.16, 20.49] | 3231.4 [3197.4, 3267.2] |
| bk | corpus | 0.4806 [0.4717, 0.4906] | 0.9019 [0.8953, 0.9079] | 0.0981 [0.0921, 0.1047] | 0.1157 [0.1086, 0.1235] | 75.0 [72.0, 76.7] | 3.15 [3.11, 3.20] | 0.841 [0.818, 0.831] † | 20.94 [20.55, 21.00] | 2736.4 [2691.0, 2780.3] |
| eurovoc | corpus | 0.9737 [0.9710, 0.9773] | 0.8479 [0.8422, 0.8530] | 0.1521 [0.1470, 0.1578] | 0.1794 [0.1734, 0.1860] | 269.3 [260.2, 269.5] | 4.98 [4.96, 5.00] | 0.821 [0.795, 0.804] † | 24.80 [24.43, 24.76] † | 268.7 [251.9, 283.5] |
| fdc | corpus | 0.7696 [0.7618, 0.7771] | 0.8337 [0.8282, 0.8408] | 0.1663 [0.1592, 0.1718] | 0.1961 [0.1877, 0.2026] | 54.5 [53.3, 55.7] | 3.21 [3.19, 3.24] | 0.836 [0.811, 0.824] † | 25.56 [25.28, 25.70] | 1413.5 [1382.1, 1449.2] |
| ilc2 | corpus | 0.9999 [0.9997, 1.0000] | 0.8189 [0.8125, 0.8252] | 0.1811 [0.1748, 0.1875] | 0.2135 [0.2061, 0.2210] | 295.1 [282.7, 295.5] | 5.09 [5.06, 5.12] | 0.824 [0.800, 0.808] † | 23.38 [22.84, 23.18] † | 135.6 [127.8, 136.8] |
| ipc | corpus | 0.9799 [0.9775, 0.9825] | 0.7690 [0.7621, 0.7756] | 0.2310 [0.2244, 0.2379] | 0.2724 [0.2646, 0.2805] | 317.8 [303.0, 318.6] | 6.31 [6.29, 6.35] | 0.838 [0.813, 0.822] † | 27.31 [26.75, 27.27] † | 322.1 [306.2, 339.3] |
| jel | corpus | 0.4786 [0.4698, 0.4877] | 0.9028 [0.8955, 0.9089] | 0.0972 [0.0911, 0.1045] | 0.1146 [0.1074, 0.1232] | 28.9 [28.0, 29.6] | 2.27 [2.25, 2.30] | 0.806 [0.776, 0.802] † | 19.41 [19.27, 19.50] | 2875.9 [2840.6, 2916.7] |
| lcc | corpus | 0.8765 [0.8690, 0.8831] | 0.8487 [0.8432, 0.8553] | 0.1513 [0.1447, 0.1568] | 0.1783 [0.1706, 0.1849] | 129.0 [124.4, 130.1] | 4.08 [4.05, 4.11] | 0.837 [0.810, 0.823] † | 25.36 [25.07, 25.34] † | 824.9 [793.8, 861.1] |
| lcc-full | corpus | 0.9999 [0.9997, 1.0000] | 0.8228 [0.8165, 0.8290] | 0.1772 [0.1710, 0.1835] | 0.2089 [0.2016, 0.2164] | 657.2 [614.6, 638.2] † | 6.06 [6.04, 6.08] | 0.803 [0.780, 0.787] † | 22.86 [22.39, 22.70] † | 52.3 [50.8, 53.6] |
| msc | corpus | 0.3916 [0.3818, 0.4009] | 0.9498 [0.9457, 0.9550] | 0.0502 [0.0450, 0.0543] | 0.0592 [0.0531, 0.0640] | 114.4 [107.4, 116.1] | 2.00 [1.97, 2.03] | 0.800 [0.770, 0.787] † | 17.82 [17.65, 17.90] | 3128.5 [3085.5, 3174.0] |
| openalex-topics | corpus | 0.9732 [0.9705, 0.9764] | 0.6414 [0.6339, 0.6481] | 0.3586 [0.3519, 0.3661] | 0.4228 [0.4149, 0.4316] | 77.6 [74.6, 77.7] | 4.45 [4.43, 4.46] | 0.892 [0.878, 0.883] † | 26.85 [26.55, 26.94] | 311.1 [295.7, 324.2] |
| unesco-thesaurus | corpus | 0.9731 [0.9703, 0.9755] | 0.7210 [0.7145, 0.7275] | 0.2790 [0.2725, 0.2855] | 0.3289 [0.3213, 0.3366] | 146.7 [141.3, 147.4] | 4.90 [4.87, 4.92] | 0.852 [0.830, 0.840] † | 26.68 [26.28, 26.67] † | 304.7 [291.0, 318.3] |
| bk | holdout | 0.0440 [0.0260, 0.0600] | 0.9985 [0.9966, 0.9998] | 0.0015 [0.0002, 0.0034] | 0.0021 [0.0003, 0.0049] | 30.8 [17.2, 32.3] | 1.23 [1.14, 1.31] | 0.439 [0.299, 0.518] | 10.14 [10.05, 10.24] | 242.3 [238.5, 246.6] |
| fdc | holdout | 0.0520 [0.0379, 0.0741] | 0.9981 [0.9949, 0.9998] | 0.0019 [0.0002, 0.0051] | 0.0027 [0.0003, 0.0073] | 31.1 [15.3, 29.5] † | 1.14 [1.10, 1.20] | 0.314 [0.235, 0.433] | 10.10 [10.04, 10.21] | 242.7 [236.9, 246.1] |
| ilc2 | holdout | 0.4560 [0.4159, 0.4920] | 0.9543 [0.9388, 0.9677] | 0.0457 [0.0323, 0.0612] | 0.0655 [0.0462, 0.0876] | 188.1 [133.4, 169.9] † | 2.94 [2.76, 3.15] | 0.527 [0.498, 0.557] | 12.46 [12.15, 12.80] | 155.5 [144.9, 163.1] |
| lcc | holdout | 0.0840 [0.0640, 0.1080] | 0.9964 [0.9924, 0.9991] | 0.0036 [0.0009, 0.0076] | 0.0052 [0.0013, 0.0109] | 63.0 [35.4, 56.5] † | 1.29 [1.21, 1.37] | 0.329 [0.294, 0.425] | 10.21 [10.12, 10.36] | 236.8 [230.5, 240.8] |
| lcc-full | holdout | 0.5660 [0.5160, 0.6040] | 0.9250 [0.9012, 0.9441] | 0.0750 [0.0559, 0.0988] | 0.1074 [0.0801, 0.1414] | 340.8 [242.3, 289.9] † | 4.14 [3.84, 4.40] | 0.492 [0.484, 0.539] | 13.98 [13.58, 14.46] | 124.9 [114.5, 136.1] |
| unesco-thesaurus | holdout | 0.1580 [0.1280, 0.1881] | 0.9888 [0.9811, 0.9935] | 0.0112 [0.0065, 0.0189] | 0.0161 [0.0093, 0.0271] | 93.8 [64.1, 81.1] † | 1.65 [1.52, 1.79] | 0.439 [0.409, 0.488] | 10.82 [10.61, 11.07] | 216.0 [208.8, 223.3] |

Placement counts (fallback = no candidate, counted as unplaced; `deciding` is the tier that decided, or for a flagged item the tier of its top candidate):

| scheme | corpus | placed | flagged | fallback | failed | deciding | top class | classify s |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 7445 | 6854 | 2555 | 0 | vocab 7445 | 11.8% `10002944.10011123.10011674` | 39 |
| arxiv | corpus | 4260 | 3325 | 5740 | 0 | vocab 4260 | 39.1% `cs.PF` | 42 |
| bk | corpus | 4806 | 3980 | 5194 | 0 | vocab 4806 | 16.4% `01.00` | 42 |
| eurovoc | corpus | 9737 | 9582 | 263 | 0 | vocab 9737 | 3.6% `C2228` | 40 |
| fdc | corpus | 7696 | 7023 | 2304 | 0 | vocab 7696 | 14.3% `676` | 39 |
| ilc2 | corpus | 9999 | 9999 | 1 | 0 | vocab 9999 | 4.2% `edqam` | 54 |
| ipc | corpus | 9799 | 9725 | 201 | 0 | vocab 9799 | 8.1% `G09B23/00` | 49 |
| jel | corpus | 4786 | 3854 | 5214 | 0 | vocab 4786 | 20.3% `M53` | 38 |
| lcc | corpus | 8765 | 8320 | 1235 | 0 | vocab 8765 | 7.5% `QA299.6-433` | 46 |
| lcc-full | corpus | 9999 | 9999 | 1 | 0 | vocab 9999 | 1.2% `QC243.3.E5` | 895 |
| msc | corpus | 3916 | 2999 | 6084 | 0 | vocab 3916 | 9.7% `51-XX` | 36 |
| openalex-topics | corpus | 9732 | 9616 | 268 | 0 | vocab 9732 | 5.7% `F21` | 43 |
| unesco-thesaurus | corpus | 9731 | 9529 | 269 | 0 | vocab 9731 | 5.5% `c155` | 46 |
| bk | holdout | 22 | 22 | 478 | 0 | vocab 22 | 31.8% `71.32` | 2 |
| fdc | holdout | 26 | 26 | 474 | 0 | vocab 26 | 11.5% `750` | 2 |
| ilc2 | holdout | 228 | 226 | 272 | 0 | vocab 228 | 11.0% `edqam` | 4 |
| lcc | holdout | 42 | 42 | 458 | 0 | vocab 42 | 7.1% `QH501-531` | 3 |
| lcc-full | holdout | 283 | 283 | 217 | 0 | vocab 283 | 2.1% `K23` | 47 |
| unesco-thesaurus | holdout | 79 | 77 | 421 | 0 | vocab 79 | 5.1% `c155` | 3 |

## `shortlist` (profile:shortlist@2): all metrics

Value [95% interval]. `classes used` is exp of placement entropy; `nav log a=1` is the expected `sum(1 + log2 fanout)` menu cost from root to item; `nav scan` adds a linear menu scan and a scan of the final class shelf. † marks an interval that excludes the full-sample value. A resample holds about 63% distinct items, so classes with few items drop out of it; statistics of the set of used classes (classes used, gini, and the navigation costs, which depend on fan-out) are biased in resamples, and their intervals show resampling spread around a shifted centre. `results.json` gives every metric's bootstrap bias (`bootstrap_bias`, resample mean minus full-sample value). Coverage, cost_norm, revenue_norm and skill are averages over items and pairs and are not affected this way.

| scheme | corpus | coverage | cost_norm | revenue_norm | skill | classes used | mean depth | gini | nav log a=1 | nav scan |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 1.0000 [1.0000, 1.0000] | 0.7641 [0.7580, 0.7705] | 0.2359 [0.2295, 0.2420] | 0.2781 [0.2706, 0.2853] | 81.3 [78.2, 82.4] | 4.06 [4.04, 4.08] | 0.866 [0.845, 0.855] † | 21.51 [20.96, 21.56] | 493.6 [473.4, 514.3] |
| arxiv | corpus | 1.0000 [1.0000, 1.0000] | 0.5745 [0.5645, 0.5827] | 0.4255 [0.4173, 0.4355] | 0.5017 [0.4920, 0.5134] | 22.0 [21.6, 22.3] | 3.30 [3.29, 3.31] | 0.827 [0.809, 0.825] † | 19.57 [19.39, 19.69] | 740.6 [718.1, 764.8] |
| bk | corpus | 1.0000 [1.0000, 1.0000] | 0.6947 [0.6857, 0.7023] | 0.3053 [0.2977, 0.3143] | 0.3600 [0.3510, 0.3705] | 57.8 [56.0, 58.8] | 4.19 [4.17, 4.22] | 0.870 [0.852, 0.861] † | 21.02 [20.65, 21.03] | 690.6 [665.5, 715.8] |
| eurovoc | corpus | 1.0000 [1.0000, 1.0000] | 0.8516 [0.8460, 0.8559] | 0.1484 [0.1441, 0.1540] | 0.1749 [0.1699, 0.1816] | 255.0 [246.4, 255.6] | 4.68 [4.65, 4.70] | 0.815 [0.790, 0.798] † | 21.58 [21.31, 21.60] | 147.5 [143.1, 153.2] |
| fdc | corpus | 1.0000 [1.0000, 1.0000] | 0.7811 [0.7757, 0.7875] | 0.2189 [0.2125, 0.2243] | 0.2581 [0.2505, 0.2644] | 61.3 [59.9, 62.1] | 3.67 [3.66, 3.69] | 0.846 [0.824, 0.834] † | 21.70 [21.43, 21.75] | 347.3 [334.2, 359.9] |
| ilc2 | corpus | 1.0000 [1.0000, 1.0000] | 0.7930 [0.7839, 0.8001] | 0.2070 [0.1999, 0.2161] | 0.2441 [0.2357, 0.2548] | 108.8 [102.9, 109.8] | 2.25 [2.24, 2.25] | 0.816 [0.787, 0.802] † | 17.56 [17.36, 17.49] † | 421.5 [398.9, 423.3] |
| ipc | corpus | 1.0000 [1.0000, 1.0000] | 0.7493 [0.7430, 0.7560] | 0.2507 [0.2440, 0.2570] | 0.2956 [0.2877, 0.3030] | 255.2 [243.1, 256.8] | 5.47 [5.44, 5.51] | 0.840 [0.815, 0.825] † | 22.23 [21.82, 22.25] | 220.7 [213.0, 230.1] |
| jel | corpus | 1.0000 [1.0000, 1.0000] | 0.7427 [0.7369, 0.7491] | 0.2573 [0.2509, 0.2631] | 0.3033 [0.2958, 0.3102] | 31.9 [31.1, 32.4] | 3.23 [3.22, 3.24] | 0.849 [0.832, 0.845] † | 19.42 [19.29, 19.44] | 784.8 [766.6, 806.8] |
| lcc | corpus | 1.0000 [1.0000, 1.0000] | 0.8206 [0.8144, 0.8262] | 0.1794 [0.1738, 0.1856] | 0.2116 [0.2049, 0.2189] | 135.1 [130.2, 136.1] | 4.14 [4.12, 4.17] | 0.839 [0.815, 0.826] † | 21.61 [21.39, 21.65] | 212.8 [207.2, 219.7] |
| lcc-full | corpus | 1.0000 [1.0000, 1.0000] | 0.8163 [0.8107, 0.8225] | 0.1837 [0.1775, 0.1893] | 0.2166 [0.2093, 0.2232] | 499.3 [463.8, 485.3] † | 4.80 [4.77, 4.84] | 0.803 [0.781, 0.788] † | 21.09 [20.83, 21.02] † | 115.2 [109.2, 120.4] |
| msc | corpus | 1.0000 [1.0000, 1.0000] | 0.8045 [0.7970, 0.8126] | 0.1955 [0.1874, 0.2030] | 0.2305 [0.2209, 0.2393] | 189.9 [182.5, 190.4] | 3.22 [3.21, 3.23] | 0.804 [0.778, 0.788] † | 18.76 [18.59, 18.73] † | 162.1 [157.5, 166.6] |
| openalex-topics | corpus | 1.0000 [1.0000, 1.0000] | 0.6060 [0.5978, 0.6134] | 0.3940 [0.3866, 0.4022] | 0.4645 [0.4559, 0.4742] | 57.7 [55.6, 58.0] | 4.15 [4.14, 4.17] | 0.902 [0.889, 0.893] † | 22.24 [22.10, 22.31] | 457.4 [440.9, 473.3] |
| unesco-thesaurus | corpus | 1.0000 [1.0000, 1.0000] | 0.7136 [0.7077, 0.7200] | 0.2864 [0.2800, 0.2923] | 0.3376 [0.3302, 0.3446] | 143.4 [137.8, 144.0] | 4.84 [4.81, 4.86] | 0.851 [0.829, 0.839] † | 22.46 [22.11, 22.53] | 176.1 [171.1, 183.5] |
| bk | holdout | 1.0000 [1.0000, 1.0000] | 0.7617 [0.7445, 0.7823] | 0.2383 [0.2177, 0.2555] | 0.3411 [0.3117, 0.3658] | 23.2 [20.2, 24.7] | 3.14 [3.08, 3.19] | 0.748 [0.696, 0.743] † | 12.95 [12.80, 13.07] | 53.9 [49.0, 62.4] |
| fdc | holdout | 1.0000 [1.0000, 1.0000] | 0.8187 [0.7961, 0.8409] | 0.1813 [0.1591, 0.2039] | 0.2595 [0.2277, 0.2919] | 47.1 [41.2, 47.3] | 3.03 [3.01, 3.06] | 0.698 [0.651, 0.688] † | 12.87 [12.71, 12.95] | 28.3 [26.2, 30.6] |
| ilc2 | holdout | 1.0000 [1.0000, 1.0000] | 0.9548 [0.9424, 0.9690] | 0.0452 [0.0310, 0.0576] | 0.0646 [0.0444, 0.0825] | 277.0 [195.0, 233.5] † | 2.68 [2.56, 2.80] | 0.416 [0.406, 0.455] | 11.61 [11.56, 11.81] | 93.6 [72.9, 80.8] † |
| lcc | holdout | 1.0000 [1.0000, 1.0000] | 0.8220 [0.7936, 0.8471] | 0.1780 [0.1529, 0.2064] | 0.2548 [0.2189, 0.2954] | 85.5 [70.4, 81.7] † | 3.12 [3.08, 3.16] | 0.627 [0.590, 0.630] | 13.13 [13.00, 13.35] | 30.7 [27.3, 30.8] |
| lcc-full | holdout | 1.0000 [1.0000, 1.0000] | 0.8525 [0.8266, 0.8737] | 0.1475 [0.1263, 0.1734] | 0.2112 [0.1808, 0.2483] | 342.7 [248.2, 288.3] † | 4.61 [4.46, 4.78] | 0.503 [0.512, 0.549] † | 13.46 [13.47, 13.96] † | 32.8 [26.6, 30.1] † |
| unesco-thesaurus | holdout | 1.0000 [1.0000, 1.0000] | 0.7855 [0.7648, 0.8061] | 0.2145 [0.1939, 0.2352] | 0.3071 [0.2776, 0.3367] | 63.4 [53.2, 63.2] † | 3.34 [3.27, 3.41] | 0.695 [0.652, 0.687] † | 12.74 [12.63, 12.91] | 22.2 [21.5, 23.8] |

Placement counts (fallback = no candidate, counted as unplaced; `deciding` is the tier that decided, or for a flagged item the tier of its top candidate):

| scheme | corpus | placed | flagged | fallback | failed | deciding | top class | classify s |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 10000 | 9409 | 0 | 0 | shortlist 3043, vocab 6957 | 16.4% `10003752.10010061` | 1948 |
| arxiv | corpus | 10000 | 8891 | 0 | 0 | shortlist 6375, vocab 3625 | 14.1% `adap-org` | 1670 |
| bk | corpus | 10000 | 9174 | 0 | 0 | shortlist 5196, vocab 4804 | 18.3% `31` | 1694 |
| eurovoc | corpus | 10000 | 9845 | 0 | 0 | shortlist 1989, vocab 8011 | 4.8% `2036` | 2004 |
| fdc | corpus | 10000 | 9327 | 0 | 0 | shortlist 2338, vocab 7662 | 11.0% `676` | 1560 |
| ilc2 | corpus | 10000 | 9981 | 0 | 0 | shortlist 9934, vocab 66 | 9.7% `agx993` | 1726 |
| ipc | corpus | 10000 | 9926 | 0 | 0 | shortlist 2705, vocab 7295 | 6.9% `G09B23/00` | 2058 |
| jel | corpus | 10000 | 9068 | 0 | 0 | shortlist 6061, vocab 3939 | 15.4% `C3` | 1546 |
| lcc | corpus | 10000 | 9555 | 0 | 0 | shortlist 2502, vocab 7498 | 5.8% `QA299.6-433` | 1615 |
| lcc-full | corpus | 10000 | 10000 | 0 | 0 | shortlist 4092, vocab 5908 | 6.1% `JS8474-84749` | 2001 |
| msc | corpus | 10000 | 9083 | 0 | 0 | shortlist 6085, vocab 3915 | 3.8% `51-XX` | 1583 |
| openalex-topics | corpus | 10000 | 9884 | 0 | 0 | shortlist 2328, vocab 7672 | 8.9% `F26` | 1656 |
| unesco-thesaurus | corpus | 10000 | 9798 | 0 | 0 | shortlist 873, vocab 9127 | 5.0% `c155` | 1825 |
| bk | holdout | 500 | 500 | 0 | 0 | shortlist 478, vocab 22 | 22.0% `06` | 23 |
| fdc | holdout | 500 | 500 | 0 | 0 | shortlist 474, vocab 26 | 9.2% `208` | 27 |
| ilc2 | holdout | 500 | 486 | 0 | 0 | shortlist 406, vocab 94 | 4.0% `agx993` | 24 |
| lcc | holdout | 500 | 500 | 0 | 0 | shortlist 461, vocab 39 | 3.6% `PZ` | 26 |
| lcc-full | holdout | 500 | 500 | 0 | 0 | shortlist 274, vocab 226 | 1.2% `K170.2` | 57 |
| unesco-thesaurus | holdout | 500 | 498 | 0 | 0 | shortlist 421, vocab 79 | 4.2% `6.70` | 27 |

## `rerank` (profile:rerank@2): all metrics

Value [95% interval]. `classes used` is exp of placement entropy; `nav log a=1` is the expected `sum(1 + log2 fanout)` menu cost from root to item; `nav scan` adds a linear menu scan and a scan of the final class shelf. † marks an interval that excludes the full-sample value. A resample holds about 63% distinct items, so classes with few items drop out of it; statistics of the set of used classes (classes used, gini, and the navigation costs, which depend on fan-out) are biased in resamples, and their intervals show resampling spread around a shifted centre. `results.json` gives every metric's bootstrap bias (`bootstrap_bias`, resample mean minus full-sample value). Coverage, cost_norm, revenue_norm and skill are averages over items and pairs and are not affected this way.

| scheme | corpus | coverage | cost_norm | revenue_norm | skill | classes used | mean depth | gini | nav log a=1 | nav scan |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 1.0000 [1.0000, 1.0000] | 0.7641 [0.7580, 0.7705] | 0.2359 [0.2295, 0.2420] | 0.2781 [0.2706, 0.2853] | 81.3 [78.2, 82.4] | 4.06 [4.04, 4.08] | 0.866 [0.845, 0.855] † | 21.51 [20.96, 21.56] | 493.6 [473.4, 514.3] |
| arxiv | corpus | 1.0000 [1.0000, 1.0000] | 0.5745 [0.5645, 0.5827] | 0.4255 [0.4173, 0.4355] | 0.5017 [0.4920, 0.5134] | 22.0 [21.6, 22.3] | 3.30 [3.29, 3.31] | 0.827 [0.809, 0.825] † | 19.57 [19.39, 19.69] | 740.6 [718.1, 764.8] |
| bk | corpus | 1.0000 [1.0000, 1.0000] | 0.6947 [0.6857, 0.7023] | 0.3053 [0.2977, 0.3143] | 0.3600 [0.3510, 0.3705] | 57.8 [56.0, 58.8] | 4.19 [4.17, 4.22] | 0.870 [0.852, 0.861] † | 21.02 [20.65, 21.03] | 690.6 [665.5, 715.8] |
| eurovoc | corpus | 1.0000 [1.0000, 1.0000] | 0.8516 [0.8460, 0.8559] | 0.1484 [0.1441, 0.1540] | 0.1749 [0.1699, 0.1816] | 255.0 [246.4, 255.6] | 4.68 [4.65, 4.70] | 0.815 [0.790, 0.798] † | 21.58 [21.31, 21.60] | 147.5 [143.1, 153.2] |
| fdc | corpus | 1.0000 [1.0000, 1.0000] | 0.7811 [0.7757, 0.7875] | 0.2189 [0.2125, 0.2243] | 0.2581 [0.2505, 0.2644] | 61.3 [59.9, 62.1] | 3.67 [3.66, 3.69] | 0.846 [0.824, 0.834] † | 21.70 [21.43, 21.75] | 347.3 [334.2, 359.9] |
| ilc2 | corpus | 1.0000 [1.0000, 1.0000] | 0.7930 [0.7839, 0.8001] | 0.2070 [0.1999, 0.2161] | 0.2441 [0.2357, 0.2548] | 108.8 [102.9, 109.8] | 2.25 [2.24, 2.25] | 0.816 [0.787, 0.802] † | 17.56 [17.36, 17.49] † | 421.5 [398.9, 423.3] |
| ipc | corpus | 1.0000 [1.0000, 1.0000] | 0.7493 [0.7430, 0.7560] | 0.2507 [0.2440, 0.2570] | 0.2956 [0.2877, 0.3030] | 255.2 [243.1, 256.8] | 5.47 [5.44, 5.51] | 0.840 [0.815, 0.825] † | 22.23 [21.82, 22.25] | 220.7 [213.0, 230.1] |
| jel | corpus | 1.0000 [1.0000, 1.0000] | 0.7427 [0.7369, 0.7491] | 0.2573 [0.2509, 0.2631] | 0.3033 [0.2958, 0.3102] | 31.9 [31.1, 32.4] | 3.23 [3.22, 3.24] | 0.849 [0.832, 0.845] † | 19.42 [19.29, 19.44] | 784.8 [766.6, 806.8] |
| lcc | corpus | 1.0000 [1.0000, 1.0000] | 0.8206 [0.8144, 0.8262] | 0.1794 [0.1738, 0.1856] | 0.2116 [0.2049, 0.2189] | 135.1 [130.2, 136.1] | 4.14 [4.12, 4.17] | 0.839 [0.815, 0.826] † | 21.61 [21.39, 21.65] | 212.8 [207.2, 219.7] |
| lcc-full | corpus | 1.0000 [1.0000, 1.0000] | 0.8163 [0.8107, 0.8225] | 0.1837 [0.1775, 0.1893] | 0.2166 [0.2093, 0.2232] | 499.3 [463.8, 485.3] † | 4.80 [4.77, 4.84] | 0.803 [0.781, 0.788] † | 21.09 [20.83, 21.02] † | 115.2 [109.2, 120.4] |
| msc | corpus | 1.0000 [1.0000, 1.0000] | 0.8045 [0.7970, 0.8126] | 0.1955 [0.1874, 0.2030] | 0.2305 [0.2209, 0.2393] | 189.9 [182.5, 190.4] | 3.22 [3.21, 3.23] | 0.804 [0.778, 0.788] † | 18.76 [18.59, 18.73] † | 162.1 [157.5, 166.6] |
| openalex-topics | corpus | 1.0000 [1.0000, 1.0000] | 0.6060 [0.5978, 0.6134] | 0.3940 [0.3866, 0.4022] | 0.4645 [0.4559, 0.4742] | 57.7 [55.6, 58.0] | 4.15 [4.14, 4.17] | 0.902 [0.889, 0.893] † | 22.24 [22.10, 22.31] | 457.4 [440.9, 473.3] |
| unesco-thesaurus | corpus | 1.0000 [1.0000, 1.0000] | 0.7136 [0.7077, 0.7200] | 0.2864 [0.2800, 0.2923] | 0.3376 [0.3302, 0.3446] | 143.4 [137.8, 144.0] | 4.84 [4.81, 4.86] | 0.851 [0.829, 0.839] † | 22.46 [22.11, 22.53] | 176.1 [171.1, 183.5] |
| bk | holdout | 1.0000 [1.0000, 1.0000] | 0.7617 [0.7445, 0.7823] | 0.2383 [0.2177, 0.2555] | 0.3411 [0.3117, 0.3658] | 23.2 [20.2, 24.7] | 3.14 [3.08, 3.19] | 0.748 [0.696, 0.743] † | 12.95 [12.80, 13.07] | 53.9 [49.0, 62.4] |
| fdc | holdout | 1.0000 [1.0000, 1.0000] | 0.8187 [0.7961, 0.8409] | 0.1813 [0.1591, 0.2039] | 0.2595 [0.2277, 0.2919] | 47.1 [41.2, 47.3] | 3.03 [3.01, 3.06] | 0.698 [0.651, 0.688] † | 12.87 [12.71, 12.95] | 28.3 [26.2, 30.6] |
| ilc2 | holdout | 1.0000 [1.0000, 1.0000] | 0.9548 [0.9424, 0.9690] | 0.0452 [0.0310, 0.0576] | 0.0646 [0.0444, 0.0825] | 277.0 [195.0, 233.5] † | 2.68 [2.56, 2.80] | 0.416 [0.406, 0.455] | 11.61 [11.56, 11.81] | 93.6 [72.9, 80.8] † |
| lcc | holdout | 1.0000 [1.0000, 1.0000] | 0.8220 [0.7936, 0.8471] | 0.1780 [0.1529, 0.2064] | 0.2548 [0.2189, 0.2954] | 85.5 [70.4, 81.7] † | 3.12 [3.08, 3.16] | 0.627 [0.590, 0.630] | 13.13 [13.00, 13.35] | 30.7 [27.3, 30.8] |
| lcc-full | holdout | 1.0000 [1.0000, 1.0000] | 0.8525 [0.8266, 0.8737] | 0.1475 [0.1263, 0.1734] | 0.2112 [0.1808, 0.2483] | 342.7 [248.2, 288.3] † | 4.61 [4.46, 4.78] | 0.503 [0.512, 0.549] † | 13.46 [13.47, 13.96] † | 32.8 [26.6, 30.1] † |
| unesco-thesaurus | holdout | 1.0000 [1.0000, 1.0000] | 0.7855 [0.7648, 0.8061] | 0.2145 [0.1939, 0.2352] | 0.3071 [0.2776, 0.3367] | 63.4 [53.2, 63.2] † | 3.34 [3.27, 3.41] | 0.695 [0.652, 0.687] † | 12.74 [12.63, 12.91] | 22.2 [21.5, 23.8] |

Placement counts (fallback = no candidate, counted as unplaced; `deciding` is the tier that decided, or for a flagged item the tier of its top candidate):

| scheme | corpus | placed | flagged | fallback | failed | deciding | top class | classify s |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | 10000 | 9409 | 0 | 0 | shortlist 3043, vocab 6957 | 16.4% `10003752.10010061` | 1612 |
| arxiv | corpus | 10000 | 8891 | 0 | 0 | shortlist 6375, vocab 3625 | 14.1% `adap-org` | 1622 |
| bk | corpus | 10000 | 9174 | 0 | 0 | shortlist 5196, vocab 4804 | 18.3% `31` | 1651 |
| eurovoc | corpus | 10000 | 9845 | 0 | 0 | shortlist 1989, vocab 8011 | 4.8% `2036` | 1750 |
| fdc | corpus | 10000 | 9327 | 0 | 0 | shortlist 2338, vocab 7662 | 11.0% `676` | 1663 |
| ilc2 | corpus | 10000 | 9981 | 0 | 0 | shortlist 9934, vocab 66 | 9.7% `agx993` | 1793 |
| ipc | corpus | 10000 | 9926 | 0 | 0 | shortlist 2705, vocab 7295 | 6.9% `G09B23/00` | 1708 |
| jel | corpus | 10000 | 9068 | 0 | 0 | shortlist 6061, vocab 3939 | 15.4% `C3` | 1292 |
| lcc | corpus | 10000 | 9555 | 0 | 0 | shortlist 2502, vocab 7498 | 5.8% `QA299.6-433` | 1725 |
| lcc-full | corpus | 10000 | 10000 | 0 | 0 | shortlist 4092, vocab 5908 | 6.1% `JS8474-84749` | 3246 |
| msc | corpus | 10000 | 9083 | 0 | 0 | shortlist 6085, vocab 3915 | 3.8% `51-XX` | 1326 |
| openalex-topics | corpus | 10000 | 9884 | 0 | 0 | shortlist 2328, vocab 7672 | 8.9% `F26` | 1759 |
| unesco-thesaurus | corpus | 10000 | 9798 | 0 | 0 | shortlist 873, vocab 9127 | 5.0% `c155` | 1765 |
| bk | holdout | 500 | 500 | 0 | 0 | shortlist 478, vocab 22 | 22.0% `06` | 24 |
| fdc | holdout | 500 | 500 | 0 | 0 | shortlist 474, vocab 26 | 9.2% `208` | 23 |
| ilc2 | holdout | 500 | 486 | 0 | 0 | shortlist 406, vocab 94 | 4.0% `agx993` | 24 |
| lcc | holdout | 500 | 500 | 0 | 0 | shortlist 461, vocab 39 | 3.6% `PZ` | 23 |
| lcc-full | holdout | 500 | 500 | 0 | 0 | shortlist 274, vocab 226 | 1.2% `K170.2` | 78 |
| unesco-thesaurus | holdout | 500 | 498 | 0 | 0 | shortlist 421, vocab 79 | 4.2% `6.70` | 25 |

## Failures

None.

## Runtime

- scheme_builds: about 1 min for 13 schemes (fetch, compile and build; every file sha256 equal to 2026-10-10)
- average_linkage_baseline: not recomputed: ref_cost_norm taken from ../2026-10-10/results.json after checking each graph_sha256
- concurrency: at most 3 placement jobs at once over all tiers; tiers run one after another (shortlist, rerank, lexical); each semantic batch pinned to its own 5 CPUs (--cpus-per-job 5), bootstrap scoring on 2 threads (--score-threads 2)
- shortlist: about 2 h 35 min for 19 jobs: lcc-full alone first (36 min, to measure memory), then the other 18, 3 at a time (1 h 58 min); classify_seconds sum 23,071, longest 2,058 (lcc-full corpus)
- rerank: about 2 h 16 min for 19 jobs, 3 at a time; classify_seconds sum 23,110, longest 3,246
- lexical: about 20 min for 19 jobs, 3 at a time; classify_seconds sum 1,470, longest 895 (lcc-full corpus)
- peak_memory: 8.0 GiB summed RSS over all running shelfmark and harness processes (sampled every 5 s); the largest single process was the rerank lcc-full holdout batch at 6.9 GiB, which loads the store's 1.46 GB memo.jsonl left by the lcc-full corpus batch; a semantic corpus batch held about 1.2 GiB and a bootstrap scoring run at most 0.44 GiB
- total_wall_clock: about 5 h 10 min of placement and scoring on 16 CPUs, 31 GiB RAM
