# DDC comparison, 2026-10-10-ddc

Gold Dewey Decimal Classification numbers against gold LCC, FDC and the
shelfmark-placed general schemes (`lcc`, `lcc-full`, `fdc`, `bk`, `ilc2`,
`unesco-thesaurus`, `learned`), scored by the Paper 1 harness on two book
holdouts. Every value has a paired item-bootstrap 95% interval (1,000
draws, seed 20261010). No text of the Dewey schedules is used: the Dewey
tree is built from the class numbers alone. See
`research/reports/7-ddc-position.md` for the licensing position and
`REPRODUCE.md` for the commands.

## Inputs

- `holdout`: the 500 Open Library editions of Paper 1 (titles only), sha256
  `3e057eb4a257...`, on the published 2026-10-10 graph
  (average-linkage baseline 0.3014). The Dewey numbers were re-read
  from the same 500 edition records (`bench/corpus/holdout-ddc.jsonl`): 134
  carry a `dewey_decimal_class` value and 110 of those hold a number, so
  gold DDC covers 22% of this holdout.
- `holdout-ddc`: those 110 editions, on the graph induced on them
  (baseline 0.2162), with the placements made on the whole holdout. All 110 also
  carry an LC call number.
- `books`: 3,000 new Open Library editions (`bench/corpus/holdout-books.jsonl`,
  sha256 `38e5463bf590...`). Each has an LC call number, a Dewey number and subjects; 320 also
  have a description. They are stratified equally over the 21 LCC main
  classes, and none shares a work with the 500-book holdout. The new graph
  uses the same all-MiniLM-L6-v2 weights and k = 20 as the committed graphs,
  over title, description and subjects (`graphs/books-k20.manifest.json`,
  graph sha256 `1d12c791d19a...`, tau 0.1572, baseline 0.2297).
- Dewey tree (`schemes/ddc.json`): 6,778 class numbers, being every number on
  the 3,110 books that carry one plus its notational prefixes. It has 10
  main classes, 93 divisions and 416 sections, then one level per further
  decimal digit. Labels are the notation itself.
- Runs: `ddc` (gold number at full depth), `ddc (sections)` (gold number cut to
  three digits), `lcc` and `lcc-full` gold (deepest class covering the call
  number, as in Paper 1), `fdc` gold (the Dewey number mapped to the deepest
  FDC class with a coinciding number or range: on `books` 316 exact, 2,401
  by leading digits, 283 by range), shelfmark `profile:lexical@2`
  placements into the seven general schemes, and `learned` own insertion
  (nearest leaf centroid).
- The 10 `holdout` runs shared with earlier results (gold `lcc` and
  `lcc-full`, the six placed Paper 1 schemes, `learned` placed and own
  insertion) equal the published 2026-10-10, 2026-10-10-r2 and
  2026-10-10-learned results exactly: scores, structure, and graph and tree
  hashes. Only the two `learned` scheme-file hashes differ, because the
  scheme's source note now pins the corrected corpus; the classes are the same.

## Findings

1. **On the 3,000 books, the Dewey numbers librarians assigned give a lower
   Dasgupta cost than their LC call numbers.** Gold DDC scores cost_norm 0.4929
   [0.4752, 0.5113] (skill 0.658) and gold LCC 0.5233 [0.5060, 0.5413] (skill
   0.619). The paired difference DDC - LCC is -0.0304 [-0.0449, -0.0155],
   negative in 99.9% of draws; against `lcc-full` it is -0.0294
   [-0.0439, -0.0144].
2. **The gap is not a depth effect.** Cut to its three-digit section, gold DDC
   still scores 0.4941, with paired difference -0.0292 [-0.0438, -0.0141]
   against LCC. That holds although sections give 123 effective classes
   against LCC's 298. Full-depth Dewey is only 0.0012 better than sections.
   It costs 2.2 more bits of navigation (nav log a=1: 19.31 against 17.14) and
   is 3.1 bits above LCC's 16.19.
3. **FDC carries almost all of it.** Mapping each Dewey number into Free
   Decimal Correspondence (CC0, 1,097 classes) gives 0.4958 [0.4786, 0.5139],
   only 0.0029 [0.0023, 0.0037] above gold DDC. That is still 0.0275
   [0.0122, 0.0421] below gold LCC. An open Dewey-shaped scheme keeps the
   arrangement advantage, given the right numbers.
4. **Classifier-placed schemes stay far behind the librarians' numbers.**
   The best are `unesco-thesaurus` 0.6287 [0.6116, 0.6456] (coverage 0.925) and
   `fdc` 0.6537 [0.6369, 0.6711] (coverage 0.839). Every placed run is at least
   0.13 above gold DDC (all intervals exclude 0). With the books' subjects to
   read, coverage is far above Paper 1's title-only holdout: `lcc` 0.915
   against 0.084.
5. **The 500-book holdout cannot rank DDC and LCC.** On the whole holdout,
   gold DDC leaves 78% of books unplaced (cost 0.9757). On the 110 books both
   systems classified, DDC 0.7255 and LCC 0.7328 differ by -0.0073
   [-0.0522, +0.0394], so the intervals do not separate.
6. `learned` own insertion scores 0.3390 on `books` and 0.4432 on `holdout`,
   below every gold label set. As in `../2026-10-10-learned/`, it files works
   by the same embedding vectors that define the graph, so this is not a
   like-for-like comparison with labels assigned without the graph.

## Caveats

- One graph, one model and one k per corpus, as in Paper 1; the ranking is a
  statement about this similarity, not about shelving quality.
- The `books` text is mostly Library of Congress subject headings (all 3,000
  rows have subjects, 320 a description). Those headings are usually
  assigned along with the LC call number, which, if anything, should favour
  LCC.
- `books` is stratified equally over LCC main classes, not proportionally to
  any collection. Its Dewey main classes are uneven (3xx: 888 books, 4xx: 35).
- Gold uses the first value that holds a number when a record carries
  several, as the harness uses the first label.
- Bootstrap intervals for class-size statistics (`effective_classes`,
  navigation costs, `largest_class_share`) are approximate. A drawn copy
  enlarges its class without adding a distinct work, so these intervals
  can sit off the point value (for example DDC effective classes on `books`:
  1,656.7 [1,343.0, 1,446.7]). Cost, revenue, skill and coverage intervals
  do not have this problem.
- Open Library records change. The pinned files, not a fresh harvest, define
  the holdouts.

Metrics, as in Paper 1: cost_norm is normalized Dasgupta cost (lower is
better); revenue_norm = 1 - cost_norm is the normalized Moseley-Wang revenue;
skill = (1 - cost_norm) / (1 - average-linkage cost_norm) on the same draw;
coverage is the share of works placed (unplaced works sit at the root);
nav log a=1 is the expected navigation cost in bits; structure is mean
depth, effective classes (exp of class-size entropy) and largest class
share. `results.json` and `bootstrap/*.json` hold every scalar with its
interval, including nav scan, nav log a=0 and a=2, depth quantiles,
branching, entropy and Gini, plus paired differences of cost_norm,
nav_log_a1 and largest_class_share for every pair of runs.

## holdout (n = 500)

| scheme | placement | coverage | cost_norm | revenue_norm | skill | nav log a=1 | mean depth | effective classes | largest class share |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ddc (numbers) | gold DDC | 0.220 [0.186, 0.254] | 0.9757 [0.9639, 0.9857] | 0.0243 [0.0143, 0.0361] | 0.035 [0.020, 0.051] | 12.06 [11.53, 12.46] | 2.12 [1.93, 2.30] | 66.2 [40.3, 71.0] | 0.146 [0.127, 0.166] |
| ddc (sections) | gold DDC, 3 digits | 0.220 [0.186, 0.254] | 0.9759 [0.9642, 0.9858] | 0.0241 [0.0142, 0.0358] | 0.035 [0.020, 0.051] | 11.72 [11.28, 12.02] | 1.66 [1.56, 1.76] | 20.6 [14.4, 23.1] | 0.248 [0.222, 0.275] |
| lcc | gold LCC | 0.922 [0.896, 0.944] | 0.7076 [0.6888, 0.7301] | 0.2924 [0.2699, 0.3112] | 0.419 [0.383, 0.451] | 18.59 [18.06, 18.88] | 4.20 [4.08, 4.31] | 35.0 [28.2, 37.3] | 0.239 [0.228, 0.251] |
| lcc-full | gold LCC | 0.922 [0.896, 0.944] | 0.7057 [0.6866, 0.7287] | 0.2943 [0.2713, 0.3134] | 0.421 [0.386, 0.454] | 19.71 [19.20, 20.16] | 5.19 [5.04, 5.34] | 85.2 [66.0, 84.5] | 0.182 [0.174, 0.191] |
| fdc | gold DDC mapped | 0.220 [0.186, 0.254] | 0.9759 [0.9642, 0.9858] | 0.0241 [0.0142, 0.0358] | 0.035 [0.020, 0.051] | 11.71 [11.28, 12.02] | 1.65 [1.55, 1.75] | 19.3 [13.6, 21.7] | 0.253 [0.227, 0.280] |
| lcc | shelfmark | 0.084 [0.062, 0.108] | 0.9964 [0.9916, 0.9990] | 0.0036 [0.0010, 0.0084] | 0.005 [0.001, 0.012] | 10.21 [10.12, 10.36] | 1.29 [1.20, 1.38] | 63.0 [36.0, 56.4] | 0.063 [0.051, 0.104] |
| lcc-full | shelfmark | 0.566 [0.524, 0.610] | 0.9250 [0.9030, 0.9450] | 0.0750 [0.0550, 0.0970] | 0.107 [0.078, 0.139] | 13.98 [13.59, 14.44] | 4.14 [3.87, 4.40] | 340.8 [243.5, 289.9] | 0.071 [0.061, 0.081] |
| fdc | shelfmark | 0.052 [0.034, 0.072] | 0.9981 [0.9951, 0.9998] | 0.0019 [0.0002, 0.0049] | 0.003 [0.000, 0.007] | 10.10 [10.03, 10.20] | 1.14 [1.09, 1.20] | 31.1 [16.6, 29.3] | 0.072 [0.067, 0.153] |
| bk | shelfmark | 0.044 [0.028, 0.062] | 0.9985 [0.9963, 0.9998] | 0.0015 [0.0002, 0.0037] | 0.002 [0.000, 0.005] | 10.14 [10.06, 10.26] | 1.23 [1.14, 1.32] | 30.8 [17.3, 33.0] | 0.115 [0.078, 0.154] |
| ilc2 | shelfmark | 0.456 [0.410, 0.500] | 0.9543 [0.9370, 0.9696] | 0.0457 [0.0304, 0.0630] | 0.065 [0.044, 0.090] | 12.46 [12.13, 12.81] | 2.94 [2.73, 3.15] | 188.1 [135.1, 173.8] | 0.042 [0.033, 0.054] |
| unesco-thesaurus | shelfmark | 0.158 [0.128, 0.190] | 0.9888 [0.9811, 0.9944] | 0.0112 [0.0056, 0.0189] | 0.016 [0.008, 0.027] | 10.82 [10.61, 11.10] | 1.65 [1.52, 1.80] | 93.8 [63.6, 82.3] | 0.058 [0.055, 0.086] |
| learned | shelfmark | 0.086 [0.062, 0.110] | 0.9915 [0.9836, 0.9968] | 0.0085 [0.0032, 0.0164] | 0.012 [0.005, 0.024] | 10.57 [10.34, 10.78] | 1.34 [1.25, 1.44] | 26.1 [19.7, 25.6] | 0.175 [0.138, 0.208] |
| learned | own insertion | 1.000 [1.000, 1.000] | 0.4432 [0.4129, 0.4764] | 0.5568 [0.5236, 0.5871] | 0.797 [0.754, 0.840] | 19.69 [19.05, 19.97] | 8.79 [8.64, 8.94] | 15.1 [14.3, 15.7] | 0.125 [0.123, 0.128] |

### Paired cost_norm differences against gold DDC

| a | b | cost_norm a - b | 95% CI | share of draws a lower |
| --- | --- | ---: | ---: | ---: |
| ddc-gold | ddc-section | -0.0002 | [-0.0004, -0.0000] | 1.000 |
| ddc-gold | lcc-gold | +0.2681 | [+0.2429, +0.2901] | 0.000 |
| ddc-gold | lcc-full-gold | +0.2700 | [+0.2442, +0.2920] | 0.000 |
| ddc-gold | fdc-ddc | -0.0002 | [-0.0004, -0.0000] | 1.000 |
| ddc-gold | lcc-placed | -0.0207 | [-0.0320, -0.0106] | 0.999 |
| ddc-gold | lcc-full-placed | +0.0507 | [+0.0286, +0.0750] | 0.001 |
| ddc-gold | fdc-placed | -0.0224 | [-0.0341, -0.0125] | 1.000 |
| ddc-gold | bk-placed | -0.0228 | [-0.0346, -0.0131] | 1.000 |
| ddc-gold | ilc2-placed | +0.0214 | [+0.0026, +0.0414] | 0.011 |
| ddc-gold | unesco-thesaurus-placed | -0.0131 | [-0.0258, -0.0005] | 0.979 |
| ddc-gold | learned-placed | -0.0158 | [-0.0279, -0.0036] | 0.993 |
| ddc-gold | learned-own | +0.5325 | [+0.4966, +0.5637] | 0.000 |

## books (n = 3000)

| scheme | placement | coverage | cost_norm | revenue_norm | skill | nav log a=1 | mean depth | effective classes | largest class share |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ddc (numbers) | gold DDC | 1.000 [1.000, 1.000] | 0.4929 [0.4752, 0.5113] | 0.5071 [0.4887, 0.5248] | 0.658 [0.636, 0.680] | 19.31 [19.10, 19.38] | 7.82 [7.73, 7.92] | 1656.7 [1343.0, 1446.7] | 0.043 [0.041, 0.046] |
| ddc (sections) | gold DDC, 3 digits | 1.000 [1.000, 1.000] | 0.4941 [0.4767, 0.5125] | 0.5059 [0.4875, 0.5233] | 0.657 [0.635, 0.678] | 17.14 [17.01, 17.16] | 4.00 [4.00, 4.00] | 123.1 [116.2, 123.0] | 0.099 [0.093, 0.105] |
| lcc | gold LCC | 1.000 [1.000, 1.000] | 0.5233 [0.5060, 0.5413] | 0.4767 [0.4587, 0.4940] | 0.619 [0.598, 0.639] | 16.19 [16.12, 16.29] | 4.12 [4.08, 4.16] | 298.2 [281.0, 294.9] | 0.017 [0.017, 0.019] |
| lcc-full | gold LCC | 1.000 [1.000, 1.000] | 0.5224 [0.5050, 0.5404] | 0.4776 [0.4596, 0.4950] | 0.620 [0.599, 0.640] | 16.77 [16.66, 16.84] | 5.16 [5.11, 5.20] | 608.0 [561.4, 585.2] | 0.012 [0.013, 0.015] |
| fdc | gold DDC mapped | 1.000 [1.000, 1.000] | 0.4958 [0.4786, 0.5139] | 0.5042 [0.4861, 0.5214] | 0.654 [0.633, 0.676] | 18.07 [17.85, 18.16] | 3.94 [3.92, 3.97] | 109.7 [103.6, 109.9] | 0.101 [0.095, 0.107] |
| lcc | shelfmark | 0.915 [0.905, 0.924] | 0.7334 [0.7184, 0.7490] | 0.2666 [0.2510, 0.2816] | 0.346 [0.326, 0.365] | 20.55 [20.30, 20.58] | 4.11 [4.05, 4.17] | 271.0 [249.2, 265.6] | 0.042 [0.039, 0.045] |
| lcc-full | shelfmark | 1.000 [1.000, 1.000] | 0.7622 [0.7474, 0.7762] | 0.2378 [0.2238, 0.2526] | 0.309 [0.292, 0.327] | 18.48 [18.27, 18.53] | 5.96 [5.91, 6.01] | 887.7 [751.4, 809.3] | 0.031 [0.029, 0.034] |
| fdc | shelfmark | 0.839 [0.827, 0.851] | 0.6537 [0.6369, 0.6711] | 0.3463 [0.3289, 0.3631] | 0.450 [0.428, 0.471] | 23.37 [23.07, 23.68] | 3.57 [3.51, 3.62] | 69.8 [64.5, 71.0] | 0.138 [0.132, 0.144] |
| bk | shelfmark | 0.554 [0.537, 0.572] | 0.7958 [0.7762, 0.8137] | 0.2042 [0.1863, 0.2238] | 0.265 [0.241, 0.290] | 19.76 [19.31, 19.96] | 3.67 [3.59, 3.76] | 78.8 [71.7, 81.9] | 0.099 [0.093, 0.103] |
| ilc2 | shelfmark | 1.000 [1.000, 1.000] | 0.7310 [0.7164, 0.7448] | 0.2690 [0.2552, 0.2836] | 0.349 [0.331, 0.368] | 19.30 [18.84, 19.18] | 4.63 [4.59, 4.68] | 222.1 [202.5, 219.9] | 0.049 [0.047, 0.054] |
| unesco-thesaurus | shelfmark | 0.925 [0.916, 0.935] | 0.6287 [0.6116, 0.6456] | 0.3713 [0.3544, 0.3884] | 0.482 [0.459, 0.504] | 22.17 [21.93, 22.24] | 4.50 [4.45, 4.55] | 196.4 [179.0, 194.6] | 0.092 [0.087, 0.097] |
| learned | shelfmark | 0.501 [0.483, 0.518] | 0.8272 [0.8104, 0.8429] | 0.1728 [0.1571, 0.1896] | 0.224 [0.204, 0.245] | 20.67 [20.26, 21.00] | 2.93 [2.85, 2.99] | 28.5 [27.4, 29.0] | 0.165 [0.159, 0.171] |
| learned | own insertion | 1.000 [1.000, 1.000] | 0.3390 [0.3248, 0.3541] | 0.6610 [0.6459, 0.6752] | 0.858 [0.839, 0.878] | 29.72 [27.03, 29.85] | 11.31 [11.24, 11.38] | 37.8 [37.2, 38.1] | 0.095 [0.094, 0.096] |

### Paired cost_norm differences against gold DDC

| a | b | cost_norm a - b | 95% CI | share of draws a lower |
| --- | --- | ---: | ---: | ---: |
| ddc-gold | ddc-section | -0.0012 | [-0.0015, -0.0010] | 1.000 |
| ddc-gold | lcc-gold | -0.0304 | [-0.0449, -0.0155] | 0.999 |
| ddc-gold | lcc-full-gold | -0.0294 | [-0.0439, -0.0144] | 0.999 |
| ddc-gold | fdc-ddc | -0.0029 | [-0.0037, -0.0023] | 1.000 |
| ddc-gold | lcc-placed | -0.2405 | [-0.2612, -0.2208] | 1.000 |
| ddc-gold | lcc-full-placed | -0.2693 | [-0.2895, -0.2501] | 1.000 |
| ddc-gold | fdc-placed | -0.1608 | [-0.1819, -0.1406] | 1.000 |
| ddc-gold | bk-placed | -0.3028 | [-0.3266, -0.2783] | 1.000 |
| ddc-gold | ilc2-placed | -0.2381 | [-0.2579, -0.2196] | 1.000 |
| ddc-gold | unesco-thesaurus-placed | -0.1358 | [-0.1547, -0.1165] | 1.000 |
| ddc-gold | learned-placed | -0.3343 | [-0.3559, -0.3112] | 1.000 |
| ddc-gold | learned-own | +0.1539 | [+0.1322, +0.1758] | 0.000 |

## holdout-ddc (n = 110)

| scheme | placement | coverage | cost_norm | revenue_norm | skill | nav log a=1 | mean depth | effective classes | largest class share |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| ddc (numbers) | gold DDC | 1.000 [1.000, 1.000] | 0.7255 [0.6649, 0.7884] | 0.2745 [0.2116, 0.3351] | 0.350 [0.272, 0.443] | 15.02 [13.48, 15.61] | 6.10 [5.81, 6.38] | 66.2 [39.8, 71.8] | 0.146 [0.127, 0.167] |
| ddc (sections) | gold DDC, 3 digits | 1.000 [1.000, 1.000] | 0.7419 [0.6872, 0.7996] | 0.2581 [0.2004, 0.3128] | 0.329 [0.257, 0.413] | 13.45 [12.05, 13.73] | 4.00 [4.00, 4.00] | 20.6 [14.4, 22.9] | 0.248 [0.221, 0.273] |
| lcc | gold LCC | 1.000 [1.000, 1.000] | 0.7328 [0.6665, 0.7942] | 0.2672 [0.2058, 0.3335] | 0.341 [0.267, 0.433] | 13.48 [12.60, 13.70] | 4.65 [4.50, 4.81] | 36.4 [24.9, 39.5] | 0.212 [0.187, 0.235] |
| lcc-full | gold LCC | 1.000 [1.000, 1.000] | 0.7313 [0.6649, 0.7937] | 0.2687 [0.2063, 0.3351] | 0.343 [0.268, 0.436] | 13.97 [13.19, 14.44] | 5.82 [5.62, 6.04] | 75.3 [50.3, 74.3] | 0.160 [0.142, 0.178] |
| fdc | gold DDC mapped | 1.000 [1.000, 1.000] | 0.7419 [0.6872, 0.7996] | 0.2581 [0.2004, 0.3128] | 0.329 [0.257, 0.413] | 13.43 [12.04, 13.72] | 3.95 [3.89, 4.00] | 19.3 [13.7, 21.5] | 0.253 [0.226, 0.280] |
| lcc | shelfmark | 0.073 [0.027, 0.127] | 0.9963 [0.9801, 1.0000] | 0.0037 [0.0000, 0.0199] | 0.005 [0.000, 0.026] | 7.83 [7.78, 8.01] | 1.24 [1.09, 1.41] | 21.0 [6.6, 20.1] | 0.077 [0.069, 0.214] |
| lcc-full | shelfmark | 0.509 [0.409, 0.600] | 0.9415 [0.8743, 0.9846] | 0.0585 [0.0154, 0.1257] | 0.075 [0.020, 0.158] | 9.60 [9.19, 10.40] | 3.96 [3.37, 4.52] | 158.5 [92.9, 127.5] | 0.064 [0.047, 0.086] |
| fdc | shelfmark | 0.064 [0.027, 0.109] | 0.9942 [0.9716, 1.0000] | 0.0058 [0.0000, 0.0284] | 0.007 [0.000, 0.037] | 7.87 [7.78, 8.07] | 1.15 [1.05, 1.27] | 11.0 [3.8, 10.9] | 0.176 [0.130, 0.364] |
| bk | shelfmark | 0.055 [0.018, 0.100] | 0.9987 [0.9919, 1.0000] | 0.0013 [0.0000, 0.0081] | 0.002 [0.000, 0.011] | 7.89 [7.78, 8.12] | 1.28 [1.09, 1.52] | 17.3 [6.0, 17.2] | 0.129 [0.094, 0.200] |
| ilc2 | shelfmark | 0.436 [0.345, 0.527] | 0.9455 [0.8844, 0.9866] | 0.0545 [0.0134, 0.1156] | 0.070 [0.018, 0.145] | 8.94 [8.64, 9.48] | 2.81 [2.45, 3.23] | 95.4 [56.6, 81.7] | 0.035 [0.032, 0.066] |
| unesco-thesaurus | shelfmark | 0.118 [0.064, 0.182] | 1.0000 [1.0000, 1.0000] | 0.0000 [0.0000, 0.0000] | 0.000 [0.000, 0.000] | 7.97 [7.85, 8.33] | 1.46 [1.23, 1.73] | 37.6 [14.6, 32.6] | 0.098 [0.065, 0.182] |
| learned | shelfmark | 0.082 [0.036, 0.136] | 0.9860 [0.9483, 1.0000] | 0.0140 [0.0000, 0.0517] | 0.018 [0.000, 0.066] | 8.07 [7.83, 8.43] | 1.33 [1.15, 1.55] | 16.3 [7.2, 16.3] | 0.194 [0.125, 0.250] |
| learned | own insertion | 1.000 [1.000, 1.000] | 0.4644 [0.4015, 0.5492] | 0.5356 [0.4508, 0.5985] | 0.683 [0.588, 0.784] | 16.60 [14.22, 16.93] | 8.66 [8.35, 8.95] | 14.7 [13.2, 15.8] | 0.127 [0.122, 0.132] |

### Paired cost_norm differences against gold DDC

| a | b | cost_norm a - b | 95% CI | share of draws a lower |
| --- | --- | ---: | ---: | ---: |
| ddc-gold | ddc-section | -0.0164 | [-0.0331, -0.0054] | 1.000 |
| ddc-gold | lcc-gold | -0.0073 | [-0.0522, +0.0394] | 0.649 |
| ddc-gold | lcc-full-gold | -0.0058 | [-0.0516, +0.0411] | 0.625 |
| ddc-gold | fdc-ddc | -0.0164 | [-0.0331, -0.0054] | 1.000 |
| ddc-gold | lcc-placed | -0.2708 | [-0.3325, -0.2054] | 1.000 |
| ddc-gold | lcc-full-placed | -0.2160 | [-0.2902, -0.1264] | 1.000 |
| ddc-gold | fdc-placed | -0.2687 | [-0.3315, -0.2015] | 1.000 |
| ddc-gold | bk-placed | -0.2732 | [-0.3336, -0.2101] | 1.000 |
| ddc-gold | ilc2-placed | -0.2200 | [-0.2929, -0.1414] | 1.000 |
| ddc-gold | unesco-thesaurus-placed | -0.2745 | [-0.3351, -0.2116] | 1.000 |
| ddc-gold | learned-placed | -0.2605 | [-0.3271, -0.1863] | 1.000 |
| ddc-gold | learned-own | +0.2611 | [+0.1688, +0.3431] | 0.000 |

