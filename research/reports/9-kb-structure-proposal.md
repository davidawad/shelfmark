<!-- table:availability -->
| sample | level | number | count | share | 95% Wilson interval | source |
| --- | --- | --- | ---: | ---: | ---: | --- |
| holdout (500 editions) | edition record | Dewey number | 110/500 | 0.220 | [0.186, 0.258] | bench/corpus/holdout-ddc.jsonl |
| holdout (500 works) | work (all editions) | Dewey number | 455/500 | 0.910 | [0.882, 0.932] | bench/results/2026-10-10-proposal/availability/holdout-works.jsonl |
| holdout (500 works) | work (all editions) | LC call number | 500/500 | 1.000 | [0.992, 1.000] | bench/results/2026-10-10-proposal/availability/holdout-works.jsonl |
| holdout (500 works) | work (all editions) | both | 455/500 | 0.910 | [0.882, 0.932] | bench/results/2026-10-10-proposal/availability/holdout-works.jsonl |
| 50 well-known books | work (all editions) | Dewey number | 50/50 | 1.000 | [0.929, 1.000] | bench/results/2026-10-10-proposal/availability/books50-works.jsonl |
| 50 well-known books | work (all editions) | LC call number | 50/50 | 1.000 | [0.929, 1.000] | bench/results/2026-10-10-proposal/availability/books50-works.jsonl |
| 50 well-known books | work (all editions) | both | 50/50 | 1.000 | [0.929, 1.000] | bench/results/2026-10-10-proposal/availability/books50-works.jsonl |
| holdout (500 works) | edition, else work consensus | FDC class reached | 455/500 | 0.910 | [0.882, 0.932] | bench/results/2026-10-10-proposal/placements/holdout/fdc-work.jsonl |
<!-- /table:availability -->

<!-- table:books-main -->
| run | structure | coverage | cost_norm [95% CI] | skill [95% CI] | nav bits | mean depth | effective classes | J (λ = 0.002) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ddc-gold` | Dewey, librarian number, full depth | 1.000 | 0.4929 [0.4752, 0.5113] | 0.658 [0.636, 0.680] | 19.31 | 7.82 | 1656.7 | 0.5315 |
| `ddc-section` | Dewey, cut to section (3 digits) | 1.000 | 0.4941 [0.4767, 0.5125] | 0.657 [0.635, 0.678] | 17.14 | 4.00 | 123.1 | 0.5284 |
| `fdc-ddc` | FDC from Dewey, full FDC depth | 1.000 | 0.4958 [0.4786, 0.5139] | 0.654 [0.633, 0.676] | 18.07 | 3.94 | 109.7 | 0.5320 |
| `fdc-d3` | FDC from Dewey, depth 3 | 1.000 | 0.4964 [0.4791, 0.5144] | 0.654 [0.632, 0.676] | 17.69 | 3.77 | 92.4 | 0.5318 |
| `fdc-d2` | FDC from Dewey, depth 2 | 1.000 | 0.5017 [0.4845, 0.5198] | 0.647 [0.626, 0.668] | 16.12 | 2.99 | 36.0 | 0.5339 |
| `lcc-gold` | LCC, librarian call number | 1.000 | 0.5233 [0.5060, 0.5413] | 0.619 [0.598, 0.639] | 16.19 | 4.12 | 298.2 | 0.5557 |
| `learned-own` | learned scheme, own insertion | 1.000 | 0.3390 [0.3248, 0.3541] | 0.858 [0.839, 0.878] | 29.72 | 11.31 | 37.8 | 0.3984 |
| `fdc-placed` | FDC, lexical@2 classifier | 0.839 | 0.6537 [0.6369, 0.6711] | 0.450 [0.428, 0.471] | 23.37 | 3.57 | 69.8 | 0.7005 |
| `lcc-placed` | LCC, lexical@2 classifier | 0.915 | 0.7334 [0.7184, 0.7490] | 0.346 [0.326, 0.365] | 20.55 | 4.11 | 271.0 | 0.7745 |
| `unesco-thesaurus-placed` | UNESCO thesaurus, lexical@2 classifier | 0.925 | 0.6287 [0.6116, 0.6456] | 0.482 [0.459, 0.504] | 22.17 | 4.50 | 196.4 | 0.6731 |
<!-- /table:books-main -->

<!-- table:paired-books -->
| comparison | score | Δ | 95% CI | p | p (Holm) | Δ / cost of b |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `fdc-ddc` − `lcc-gold` | cost_norm | -0.0275 | [-0.0421, -0.0122] | 0.002 | 0.014 | -5.3% |
| `fdc-ddc` − `lcc-gold` | J (λ = 0.002) | -0.0238 | [-0.0385, -0.0087] | 0.002 | 0.014 |  |
| `fdc-d3` − `lcc-gold` | cost_norm | -0.0270 | [-0.0415, -0.0118] | 0.002 | 0.014 | -5.2% |
| `fdc-d3` − `lcc-gold` | J (λ = 0.002) | -0.0240 | [-0.0386, -0.0091] | 0.002 | 0.014 |  |
| `fdc-d3` − `ddc-section` | cost_norm | +0.0022 | [+0.0016, +0.0030] | <0.002 | 0.014 | +0.5% |
| `fdc-d3` − `ddc-section` | J (λ = 0.002) | +0.0033 | [+0.0026, +0.0041] | <0.002 | 0.014 |  |
| `fdc-d3` − `fdc-ddc` | cost_norm | +0.0005 | [+0.0003, +0.0008] | <0.002 | 0.014 | +0.1% |
| `fdc-d3` − `fdc-ddc` | J (λ = 0.002) | -0.0002 | [-0.0004, -0.0000] | 0.032 | 0.032 |  |
| `fdc-d3` − `fdc-d2` | cost_norm | -0.0053 | [-0.0061, -0.0046] | <0.002 | 0.014 | -1.1% |
| `fdc-d3` − `fdc-d2` | J (λ = 0.002) | -0.0022 | [-0.0030, -0.0015] | <0.002 | 0.014 |  |
| `fdc-d3` − `fdc-placed` | cost_norm | -0.1573 | [-0.1783, -0.1370] | <0.002 | 0.014 | -24.1% |
| `fdc-d3` − `fdc-placed` | J (λ = 0.002) | -0.1687 | [-0.1898, -0.1487] | <0.002 | 0.014 |  |
| `learned-own` − `fdc-d3` | cost_norm | -0.1574 | [-0.1792, -0.1355] | <0.002 | 0.014 | -31.7% |
| `learned-own` − `fdc-d3` | J (λ = 0.002) | -0.1333 | [-0.1565, -0.1131] | <0.002 | 0.014 |  |
<!-- /table:paired-books -->

<!-- table:ddc-reference -->
| comparison | score | Δ | 95% CI | share of draws a lower |
| --- | --- | ---: | ---: | ---: |
| `ddc-gold` − `ddc-section` | cost_norm | -0.0012 | [-0.0015, -0.0010] | 1.000 |
| `ddc-gold` − `lcc-gold` | cost_norm | -0.0304 | [-0.0449, -0.0155] | 0.999 |
| `ddc-gold` − `fdc-ddc` | cost_norm | -0.0029 | [-0.0037, -0.0023] | 1.000 |
| `ddc-gold` − `learned-own` | cost_norm | +0.1539 | [+0.1322, +0.1758] | 0.000 |
| `fdc-ddc` − `fdc-placed` | cost_norm | -0.1579 | [-0.1787, -0.1376] | 1.000 |
| `ddc-gold` − `ddc-section` | nav_log_a1 | +2.17 | [+2.06, +2.25] | 0.000 |
| `ddc-gold` − `lcc-gold` | nav_log_a1 | +3.12 | [+2.88, +3.19] | 0.000 |
| `ddc-gold` − `fdc-ddc` | nav_log_a1 | +1.24 | [+1.07, +1.41] | 0.000 |
| `ddc-gold` − `learned-own` | nav_log_a1 | -10.41 | [-10.66, -7.81] | 1.000 |
| `fdc-ddc` − `fdc-placed` | nav_log_a1 | -5.30 | [-5.70, -5.04] | 1.000 |
<!-- /table:ddc-reference -->

<!-- table:catalogue-books -->
| score | `fdc-d3` | `fdc-ddc` | `ddc-section` | `lcc-gold` | `learned-own` | `fdc-placed` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cost_norm (lower better) | 0.4964 | 0.4958 | 0.4941 | 0.5233 | 0.3390 | 0.6537 |
| triplet accuracy (higher better) | 0.6945 | 0.6948 | 0.6957 | 0.6991 | 0.7432 | 0.6117 |
| shelf-order locality (higher better) | 0.4118 | 0.4125 | 0.4130 | 0.4734 | 0.4982 | 0.3285 |
| tree-sampling divergence, nats (higher better) | 0.885 | 0.905 | 0.933 | 1.242 | 0.938 | 0.504 |
| hospitality in array | 0.857 | 0.857 | 0.701 | 1.000 | 0.000 | 0.857 |
| splits per 1,000 inserts | 3.333 | 2.333 | 2.333 | 1.667 | 11.333 | 4.766 |
| notation length, characters | 3.265 | 3.286 | 3.000 | 7.644 | 29.923 | 3.563 |
| expected search bits | 9.276 | 9.780 | 9.337 | 8.742 | 13.297 | 9.022 |
<!-- /table:catalogue-books -->

<!-- table:catalogue-tests -->
| comparison | score | Δ | 95% CI (vertex) | p | p (Holm) |
| --- | --- | ---: | ---: | ---: | ---: |
| `fdc-d3` − `lcc-gold` | cost_norm | -0.0270 | [-0.0419, -0.0113] | 0.002 | 0.008 |
| `fdc-d3` − `lcc-gold` | triplet accuracy | -0.0046 | [-0.0131, +0.0042] | 0.288 | 0.296 |
| `fdc-d3` − `lcc-gold` | shelf-order locality | -0.0616 | [-0.0906, -0.0322] | <0.002 | 0.008 |
| `fdc-d3` − `lcc-gold` | tree-sampling divergence, nats | -0.3572 | [-0.3938, -0.3052] | <0.002 | 0.008 |
| `learned-own` − `fdc-d3` | cost_norm | -0.1574 | [-0.1798, -0.1340] | <0.002 | 0.008 |
| `learned-own` − `fdc-d3` | triplet accuracy | +0.0487 | [+0.0344, +0.0625] | <0.002 | 0.008 |
| `learned-own` − `fdc-d3` | shelf-order locality | +0.0864 | [+0.0456, +0.1247] | <0.002 | 0.008 |
| `learned-own` − `fdc-d3` | tree-sampling divergence, nats | +0.0532 | [-0.0173, +0.1249] | 0.148 | 0.296 |
<!-- /table:catalogue-tests -->

<!-- table:depth -->
| run | cost_norm | nav bits | eff. classes | J, λ = 0.0 | J, λ = 0.0005 | J, λ = 0.002 | J, λ = 0.005 | J, λ = 0.01 | max regret, FDC cuts |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `fdc-d1` | 0.5536 | 14.05 | 7.1 | 0.5536 | 0.5606 | 0.5817 | 0.6238 | 0.6940 | 0.0557 |
| `fdc-d2` | 0.5017 | 16.12 | 36.0 | 0.5017 | 0.5097 | 0.5339 | 0.5823 | 0.6629 | 0.0049 |
| `fdc-d3` | 0.4964 | 17.69 | 92.4 | 0.4964 | 0.5052 | 0.5318 | 0.5848 | 0.6733 | 0.0025 |
| `fdc-ddc` | 0.4958 | 18.07 | 109.7 | 0.4958 | 0.5049 | 0.5320 | 0.5862 | 0.6765 | 0.0039 |
| `fdc-m5` | 0.4960 | 18.39 | 88.2 | 0.4960 | 0.5052 | 0.5327 | 0.5879 | 0.6799 | 0.0056 |
| `fdc-m10` | 0.4969 | 18.74 | 67.6 | 0.4969 | 0.5063 | 0.5344 | 0.5906 | 0.6843 | 0.0083 |
| `fdc-m25` | 0.5021 | 18.90 | 35.2 | 0.5021 | 0.5116 | 0.5399 | 0.5966 | 0.6911 | 0.0143 |
| `fdc-m50` | 0.5078 | 18.24 | 22.9 | 0.5078 | 0.5170 | 0.5443 | 0.5990 | 0.6902 | 0.0167 |
| `ddc-d1` | 0.5536 | 14.05 | 7.1 | 0.5536 | 0.5606 | 0.5817 | 0.6238 | 0.6940 |  |
| `ddc-d2` | 0.4991 | 15.57 | 37.0 | 0.4991 | 0.5069 | 0.5302 | 0.5770 | 0.6548 |  |
| `ddc-section` | 0.4941 | 17.14 | 123.1 | 0.4941 | 0.5027 | 0.5284 | 0.5798 | 0.6655 |  |
| `ddc-d4` | 0.4932 | 18.24 | 283.7 | 0.4932 | 0.5024 | 0.5297 | 0.5844 | 0.6756 |  |
| `ddc-gold` | 0.4929 | 19.31 | 1656.7 | 0.4929 | 0.5026 | 0.5315 | 0.5895 | 0.6860 |  |
<!-- /table:depth -->

<!-- table:embedding -->
| graph | FDC d3 | LCC | Δ FDC − LCC | 95% CI | p (Holm) | learned | Δ learned − FDC | 95% CI | p (Holm) | minimax depth | λ* FDC = LCC |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| minilm k = 10 | 0.4164 | 0.4022 | +0.0142 | [-0.0030, +0.0311] | 0.224 | 0.3142 | -0.1022 | [-0.1280, -0.0768] | 0.018 | `fdc-d3` | never |
| minilm k = 20 | 0.4964 | 0.5233 | -0.0270 | [-0.0415, -0.0118] | 0.018 | 0.3390 | -0.1574 | [-0.1792, -0.1355] | 0.018 | `fdc-d3` | 0.0180 |
| minilm k = 50 | 0.5841 | 0.6460 | -0.0618 | [-0.0736, -0.0498] | 0.018 | 0.3805 | -0.2036 | [-0.2205, -0.1851] | 0.018 | `fdc-d2` | 0.0413 |
| bge-small k = 10 | 0.4046 | 0.4011 | +0.0035 | [-0.0125, +0.0202] | 0.704 | 0.3445 | -0.0602 | [-0.0836, -0.0354] | 0.018 | `fdc-d3` | never |
| bge-small k = 20 | 0.4852 | 0.5244 | -0.0392 | [-0.0534, -0.0253] | 0.018 | 0.3758 | -0.1094 | [-0.1310, -0.0876] | 0.018 | `fdc-d3` | 0.0262 |
| bge-small k = 50 | 0.5727 | 0.6470 | -0.0743 | [-0.0859, -0.0626] | 0.018 | 0.4187 | -0.1540 | [-0.1712, -0.1359] | 0.018 | `fdc-d2` | 0.0497 |
| nomic k = 10 | 0.3953 | 0.3732 | +0.0221 | [+0.0050, +0.0387] | 0.030 | 0.3352 | -0.0601 | [-0.0857, -0.0347] | 0.018 | `fdc-d3` | never |
| nomic k = 20 | 0.4755 | 0.4972 | -0.0216 | [-0.0365, -0.0058] | 0.024 | 0.3613 | -0.1142 | [-0.1369, -0.0924] | 0.018 | `fdc-d3` | 0.0145 |
| nomic k = 50 | 0.5637 | 0.6231 | -0.0594 | [-0.0715, -0.0467] | 0.018 | 0.4003 | -0.1635 | [-0.1814, -0.1455] | 0.018 | `fdc-d2` | 0.0397 |
<!-- /table:embedding -->

<!-- table:pipelines -->
| books with a librarian number | FDC coverage | FDC cost | LCC coverage | LCC cost | Δ cost FDC − LCC | 95% CI | p (Holm) | Δ J (λ = 0.002) | p (Holm) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 20% | 0.877 | 0.6516 | 0.934 | 0.7396 | -0.0880 | [-0.1061, -0.0680] | 0.014 | -0.0820 | 0.014 |
| 40% | 0.905 | 0.6357 | 0.950 | 0.7206 | -0.0850 | [-0.1034, -0.0652] | 0.014 | -0.0788 | 0.014 |
| 60% | 0.938 | 0.6011 | 0.967 | 0.6780 | -0.0769 | [-0.0942, -0.0580] | 0.014 | -0.0703 | 0.014 |
| 80% | 0.968 | 0.5610 | 0.981 | 0.6218 | -0.0607 | [-0.0774, -0.0426] | 0.014 | -0.0543 | 0.014 |
| 90% | 0.986 | 0.5309 | 0.991 | 0.5764 | -0.0455 | [-0.0621, -0.0285] | 0.014 | -0.0395 | 0.014 |
| 100% (gold) | 1.000 | 0.4958 | 1.000 | 0.5233 | -0.0275 | [-0.0421, -0.0122] | 0.014 | -0.0238 | 0.014 |
| 0% (classifier only) | 0.839 | 0.6537 | 0.915 | 0.7334 | -0.0797 | [-0.0985, -0.0594] | 0.014 | -0.0741 | 0.014 |
<!-- /table:pipelines -->

<!-- table:holdout -->
| graph (k = 20) | comparison | coverage a | cost a | coverage b | cost b | Δ | 95% CI | p | p (Holm) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| minilm | `fdc-work` − `lcc-gold` | 0.910 | 0.7237 | 0.922 | 0.7076 | +0.0161 | [-0.0087, +0.0423] | 0.268 | 1.000 |
| minilm | `fdc-pipeline` − `lcc-pipeline` | 0.912 | 0.7237 | 0.932 | 0.7071 | +0.0167 | [-0.0088, +0.0435] | 0.246 | 1.000 |
| minilm | `fdc-work` − `fdc-ddc` | 0.910 | 0.7237 | 0.220 | 0.9759 | -0.2521 | [-0.2791, -0.2244] | <0.002 | 0.012 |
| minilm | `fdc-work` − `lcc-gold`, the 424 books both place | 1.000 | 0.6970 | 1.000 | 0.7074 | -0.0103 | [-0.0424, +0.0189] | 0.524 | 1.000 |
| bge-small | `fdc-work` − `lcc-gold` | 0.910 | 0.7165 | 0.922 | 0.7132 | +0.0033 | [-0.0235, +0.0280] | 0.816 | 1.000 |
| bge-small | `fdc-pipeline` − `lcc-pipeline` | 0.912 | 0.7165 | 0.932 | 0.7128 | +0.0037 | [-0.0232, +0.0281] | 0.796 | 1.000 |
| bge-small | `fdc-work` − `fdc-ddc` | 0.910 | 0.7165 | 0.220 | 0.9760 | -0.2596 | [-0.2884, -0.2317] | <0.002 | 0.012 |
| bge-small | `fdc-work` − `lcc-gold`, the 424 books both place | 1.000 | 0.6966 | 1.000 | 0.7096 | -0.0131 | [-0.0428, +0.0139] | 0.374 | 1.000 |
| nomic | `fdc-work` − `lcc-gold` | 0.910 | 0.7021 | 0.922 | 0.7072 | -0.0052 | [-0.0323, +0.0212] | 0.660 | 1.000 |
| nomic | `fdc-pipeline` − `lcc-pipeline` | 0.912 | 0.7020 | 0.932 | 0.7070 | -0.0050 | [-0.0321, +0.0211] | 0.678 | 1.000 |
| nomic | `fdc-work` − `fdc-ddc` | 0.910 | 0.7021 | 0.220 | 0.9762 | -0.2742 | [-0.3045, -0.2452] | <0.002 | 0.012 |
| nomic | `fdc-work` − `lcc-gold`, the 424 books both place | 1.000 | 0.6861 | 1.000 | 0.7047 | -0.0186 | [-0.0479, +0.0071] | 0.166 | 1.000 |
<!-- /table:holdout -->

<!-- table:papers -->
| run | minilm cost / nav | bge-small cost / nav | nomic cost / nav | coverage |
| --- | ---: | ---: | ---: | ---: |
| `arxiv-gold` | 0.3333 / 18.33 | 0.3327 / 18.33 | 0.3465 / 18.33 | 1.000 |
| `fdc-arxiv-nested` | 0.3139 / 19.30 | 0.3105 / 19.30 | 0.3247 / 19.30 | 1.000 |
| `lcc-arxiv-nested` | 0.3396 / 22.74 | 0.3401 / 22.74 | 0.3549 / 22.74 | 1.000 |
| `fdc-arxiv` | 0.4000 / 18.35 | 0.3934 / 18.35 | 0.4048 / 18.35 | 1.000 |
| `lcc-arxiv` | 0.4448 / 24.06 | 0.4432 / 24.06 | 0.4505 / 24.06 | 1.000 |
| `learned-placed` | 0.3794 / 31.08 | 0.3852 / 31.08 | 0.3873 / 31.08 | 0.999 |
| `openalex-topics-placed` | 0.6414 / 26.85 | 0.6457 / 26.85 | 0.6434 / 26.85 | 0.973 |
| `fdc-placed` | 0.8337 / 25.56 | 0.8348 / 25.56 | 0.8351 / 25.56 | 0.770 |
<!-- /table:papers -->

<!-- table:papers-tests -->
| graph (k = 20) | comparison | Δ cost | 95% CI | p (Holm) | Δ J (λ = 0.002) | 95% CI | p (Holm) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| minilm | `fdc-arxiv-nested` − `arxiv-gold` | -0.0194 | [-0.0222, -0.0167] | 0.018 | -0.0174 | [-0.0203, -0.0148] | 0.018 |
| minilm | `fdc-arxiv-nested` − `lcc-arxiv-nested` | -0.0257 | [-0.0292, -0.0222] | 0.018 | -0.0326 | [-0.0361, -0.0290] | 0.018 |
| minilm | `fdc-arxiv-nested` − `fdc-arxiv` | -0.0861 | [-0.0907, -0.0817] | 0.018 | -0.0842 | [-0.0888, -0.0799] | 0.018 |
| bge-small | `fdc-arxiv-nested` − `arxiv-gold` | -0.0222 | [-0.0254, -0.0192] | 0.018 | -0.0203 | [-0.0235, -0.0172] | 0.018 |
| bge-small | `fdc-arxiv-nested` − `lcc-arxiv-nested` | -0.0296 | [-0.0340, -0.0255] | 0.018 | -0.0365 | [-0.0408, -0.0324] | 0.018 |
| bge-small | `fdc-arxiv-nested` − `fdc-arxiv` | -0.0829 | [-0.0874, -0.0787] | 0.018 | -0.0810 | [-0.0855, -0.0770] | 0.018 |
| nomic | `fdc-arxiv-nested` − `arxiv-gold` | -0.0218 | [-0.0250, -0.0188] | 0.018 | -0.0199 | [-0.0231, -0.0169] | 0.018 |
| nomic | `fdc-arxiv-nested` − `lcc-arxiv-nested` | -0.0302 | [-0.0342, -0.0263] | 0.018 | -0.0371 | [-0.0410, -0.0332] | 0.018 |
| nomic | `fdc-arxiv-nested` − `fdc-arxiv` | -0.0801 | [-0.0845, -0.0760] | 0.018 | -0.0782 | [-0.0826, -0.0743] | 0.018 |
<!-- /table:papers-tests -->

<!-- table:classifier -->
| scheme | depth | placed / books | decided without flag | same class as librarian | agreement | 95% Wilson interval |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| FDC | 1 | 2518/3000 | 623 | 1371/2518 | 0.544 | [0.525, 0.564] |
| FDC | 2 | 2518/3000 | 623 | 883/2518 | 0.351 | [0.332, 0.370] |
| FDC | 3 | 2518/3000 | 623 | 529/2518 | 0.210 | [0.195, 0.226] |
| LCC | 1 | 2744/3000 | 195 | 1021/2744 | 0.372 | [0.354, 0.390] |
| LCC | 2 | 2744/3000 | 195 | 655/2744 | 0.239 | [0.223, 0.255] |
| LCC | 3 | 2744/3000 | 195 | 425/2744 | 0.155 | [0.142, 0.169] |
<!-- /table:classifier -->
