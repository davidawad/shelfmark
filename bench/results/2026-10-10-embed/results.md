# Cross-embedding robustness, 2026-10-10-embed

Every published score so far used one similarity graph: all-MiniLM-L6-v2 at
k = 20. The learned scheme (`schemes/learned`) was fitted on that same graph.
This run checks whether the rankings, and the learned scheme's lead, depend on
that choice. The corpus and holdout are re-embedded with two independent open
models from other families: BAAI/bge-small-en-v1.5 (int8 ONNX) and
nomic-ai/nomic-embed-text-v1.5. Graphs are built at k = 10, 20 and 50, giving
nine conditions. Each placement is fixed (placements do not depend on the
embedding) and rescored on all nine graphs, with 95% item-bootstrap intervals
from 1,000 paired resamples. 468 runs and 27 references; 0 failures.

## Verdict

**The learned scheme's edge survives the cross-embedding test.** In all nine
conditions, it ranks first by skill and by J among every Paper 1 row, in every
comparison:

- held out: the 2,000 works inserted into the 80% fit;
- in sample: the full fit's own placements;
- the 500 holdout books, inserted;
- placed by shelfmark lexical@2.

The paired 95% intervals of both the skill margin and the J margin lie above
zero everywhere. The tightest case is the held-out split on bge-small-en-v1.5
at k = 10: skill 0.859 vs 0.704 for the best other row (arXiv gold), margin
0.155 [0.122, 0.188]. On the held-out split, every one of the 1,000 resamples
favours the learned scheme in every condition.

The lead does shrink off its home graph. On the held-out split at k = 20, the
skill margin is 0.190 on MiniLM, 0.166 on bge-small and 0.167 on nomic. The
held-out skill itself falls from 0.922 to 0.901 and 0.892. The two refits show
the mirror image: a scheme fitted on bge-small or nomic scores best on its own
model's graph and loses 0.02 to 0.04 held-out skill on the others (see
Cross-fit). Part of the published margin is therefore home advantage. That part
is about 0.02 to 0.03 skill, roughly a seventh of the margin; the rest holds on
graphs the scheme never saw.

## Ranking stability

The rankings of the Paper 1 schemes and the learned scheme barely move across
models and k. Over all 36 pairs of conditions, the minimum Spearman / Kendall
tau-b of the skill rankings is:

| universe | rows | Spearman | Kendall |
| --- | ---: | ---: | ---: |
| corpus | 18 | 0.992 | 0.948 |
| holdout | 10 | 0.976 | 0.911 |
| 20% split | 18 | 0.973 | 0.908 |

The J rankings give 0.998 / 0.987, 0.952 / 0.867 and 0.973 / 0.908.

On the corpus and the split, the top six rows hold the same ranks in every
condition. On the split, the learned scheme's held-out and in-sample rows
trade first and second place on MiniLM k = 10 and nomic k = 10. Every gold row
keeps its rank everywhere. The changes are moves of one to three places among
the middling lexical@2 rows (ranks 7 to 14: ilc2, lcc-full, fdc, acm-ccs,
eurovoc, lcc, arxiv and bk placed). On the holdout, the top five never move.
Under bge-small, two pairs of neighbours swap: learned-placed with
unesco-thesaurus-placed (6/7) and fdc-placed with bk-placed (9/10); nomic
swaps fdc-placed and bk-placed too. Some percentile
intervals of the rank correlations exclude their point value (for example
1.000 [0.969, 0.998]). The point value is the full-sample ranking; when it is
perfect, no resample can exceed it.

## Notes on this run

- **nomic-embed-text-v1.5 was built with every text capped at 512 tokens.** The
  previous attempt used the model's 8192-token limit with batches of 64 and
  vectors kept in memory, and was OOM-killed at 16 GiB on the 10k corpus. With
  the 512-token cap, batches of 16 and vectors streamed to a memmap, it peaked
  at 4.23 GiB (2,687 s). Only 114 of the 10,000 corpus texts are longer than
  512 tokens (the longest is 871); no holdout text is. Manifests record
  `max_length_tokens: 512`.
- MiniLM and bge-small vectors were re-embedded for the refits. They rebuild
  the committed bge-small k = 20 and MiniLM k = 10 graphs byte for byte, so
  those graphs (committed earlier) were not recomputed.
- The MiniLM k = 20 condition is the published 2026-10-10 graph. 45 of 46
  runs rescored on it equal the published cost_norm exactly. The exception is
  `split/msc-gold` (0.980538 vs 0.980498). The 2026-10-10-learned run used the
  MSC gold labels from before the r3 normalizer fix. The fixed labels cover one
  more of the 2,000 test works (coverage 0.137 vs 0.136), and the
  corpus-wide `msc-gold` equals r3 exactly.
- A bug in `shelfmark-bench --reference-replicates` was found and fixed in this
  run. It read the replicates of the reference run's scheme tree (`cost_norm`)
  instead of the average-linkage tree's (`ref_cost_norm`), which made every
  skill interval meaningless (0 or ~1e13). Point values were unaffected. All
  runs were rescored with the fix, and the published intervals come from that
  rescoring.
- Refits (same parameters, automatic labels) on bge-small and nomic k = 20
  were also scored on every other model's graphs: see Cross-fit.

## Graphs

| model | graph | k | n | edges | tau | average-linkage cost_norm [95% CI] |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| all-MiniLM-L6-v2 | corpus | 10 | 10000 | 75227 | 0.1086 | 0.1341 [0.1288, 0.1396] |
| all-MiniLM-L6-v2 | holdout | 10 | 500 | 3718 | 0.1731 | 0.2571 [0.2286, 0.2898] |
| all-MiniLM-L6-v2 | 20% split (induced) | 10 | 2000 | - | - | 0.0478 [0.0389, 0.0588] |
| all-MiniLM-L6-v2 | corpus | 20 | 10000 | 148111 | 0.1086 | 0.1518 [0.1460, 0.1577] |
| all-MiniLM-L6-v2 | holdout | 20 | 500 | 7297 | 0.1731 | 0.3014 [0.2745, 0.3314] |
| all-MiniLM-L6-v2 | 20% split (induced) | 20 | 2000 | - | - | 0.1024 [0.0919, 0.1147] |
| all-MiniLM-L6-v2 | corpus | 50 | 10000 | 362651 | 0.1086 | 0.1863 [0.1803, 0.1925] |
| all-MiniLM-L6-v2 | holdout | 50 | 500 | 17468 | 0.1731 | 0.3554 [0.3324, 0.3821] |
| all-MiniLM-L6-v2 | 20% split (induced) | 50 | 2000 | - | - | 0.1553 [0.1438, 0.1701] |
| bge-small-en-v1.5 | corpus | 10 | 10000 | 75145 | 0.6146 | 0.1226 [0.1175, 0.1287] |
| bge-small-en-v1.5 | holdout | 10 | 500 | 3722 | 0.5507 | 0.2814 [0.2527, 0.3137] |
| bge-small-en-v1.5 | 20% split (induced) | 10 | 2000 | - | - | 0.0374 [0.0303, 0.0483] |
| bge-small-en-v1.5 | corpus | 20 | 10000 | 147792 | 0.6146 | 0.1409 [0.1357, 0.1469] |
| bge-small-en-v1.5 | holdout | 20 | 500 | 7320 | 0.5507 | 0.3411 [0.3134, 0.3729] |
| bge-small-en-v1.5 | 20% split (induced) | 20 | 2000 | - | - | 0.0959 [0.0854, 0.1087] |
| bge-small-en-v1.5 | corpus | 50 | 10000 | 362815 | 0.6146 | 0.1690 [0.1634, 0.1754] |
| bge-small-en-v1.5 | holdout | 50 | 500 | 17503 | 0.5507 | 0.4156 [0.3927, 0.4397] |
| bge-small-en-v1.5 | 20% split (induced) | 50 | 2000 | - | - | 0.1449 [0.1334, 0.1589] |
| nomic-embed-text-v1.5 | corpus | 10 | 10000 | 74188 | 0.5628 | 0.1343 [0.1288, 0.1406] |
| nomic-embed-text-v1.5 | holdout | 10 | 500 | 3748 | 0.6580 | 0.2402 [0.2118, 0.2739] |
| nomic-embed-text-v1.5 | 20% split (induced) | 10 | 2000 | - | - | 0.0500 [0.0415, 0.0616] |
| nomic-embed-text-v1.5 | corpus | 20 | 10000 | 146751 | 0.5628 | 0.1554 [0.1496, 0.1619] |
| nomic-embed-text-v1.5 | holdout | 20 | 500 | 7359 | 0.6580 | 0.2802 [0.2529, 0.3112] |
| nomic-embed-text-v1.5 | 20% split (induced) | 20 | 2000 | - | - | 0.1058 [0.0950, 0.1196] |
| nomic-embed-text-v1.5 | corpus | 50 | 10000 | 363048 | 0.5628 | 0.1956 [0.1897, 0.2023] |
| nomic-embed-text-v1.5 | holdout | 50 | 500 | 17780 | 0.6580 | 0.3629 [0.3382, 0.3961] |
| nomic-embed-text-v1.5 | 20% split (induced) | 50 | 2000 | - | - | 0.1605 [0.1487, 0.1746] |

## Ranking stability

Rankings by skill (equivalently cost_norm within one graph) and by J. Rows: Paper 1 schemes (gold and lexical@2 placed) plus the learned scheme fitted on MiniLM; the refits are left out. Spearman rho and Kendall tau-b of each condition's ranking against all-MiniLM-L6-v2 k = 20, with paired item-bootstrap 95% intervals (each resample ranks the same resampled works on both graphs), and the minimum over all 36 pairs of conditions.

### corpus (18 rows)

| condition | Spearman (skill) | Kendall (skill) | Spearman (J) | Kendall (J) |
| --- | --- | --- | --- | --- |
| minilm-k10 | 1.000 [0.994, 1.000] | 1.000 [0.974, 1.000] | 1.000 [0.996, 1.000] | 1.000 [0.974, 1.000] |
| minilm-k50 | 0.996 [0.994, 1.000] | 0.974 [0.974, 1.000] | 0.998 [0.996, 1.000] | 0.987 [0.974, 1.000] |
| bge-small-k10 | 0.998 [0.990, 1.000] | 0.987 [0.948, 1.000] | 1.000 [0.992, 1.000] | 1.000 [0.960, 1.000] |
| bge-small-k20 | 0.998 [0.992, 1.000] | 0.987 [0.961, 1.000] | 0.998 [0.994, 1.000] | 0.987 [0.961, 1.000] |
| bge-small-k50 | 0.998 [0.992, 1.000] | 0.987 [0.961, 1.000] | 0.998 [0.996, 1.000] | 0.987 [0.974, 1.000] |
| nomic-k10 | 0.998 [0.986, 1.000] | 0.987 [0.935, 1.000] | 0.998 [0.990, 1.000] | 0.987 [0.948, 1.000] |
| nomic-k20 | 0.996 [0.992, 1.000] | 0.974 [0.948, 1.000] | 0.998 [0.994, 1.000] | 0.987 [0.961, 1.000] |
| nomic-k50 | 0.998 [0.992, 1.000] | 0.987 [0.961, 1.000] | 0.998 [0.996, 1.000] | 0.987 [0.974, 1.000] |
| min over all pairs | 0.992 | 0.948 | 0.998 | 0.987 |

Rank by skill under each condition:

| row | minilm-k10 | minilm-k20 | minilm-k50 | bge-small-k10 | bge-small-k20 | bge-small-k50 | nomic-k10 | nomic-k20 | nomic-k50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-own | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| arxiv-gold | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| learned-placed | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| openalex-topics-placed | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| unesco-thesaurus-placed | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| ipc-placed | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| ilc2-placed | 7 | 7 | 7 | 7 | 7 | 7 | 8 | 8 | 8 |
| lcc-full-placed | 8 | 8 | 8 | 8 | 8 | 8 | 7 | 7 | 7 |
| acm-ccs-placed | 9 | 9 | 9 | 10 | 10 | 9 | 9 | 10 | 9 |
| fdc-placed | 10 | 10 | 10 | 9 | 9 | 10 | 10 | 9 | 10 |
| eurovoc-placed | 11 | 11 | 12 | 11 | 11 | 12 | 11 | 11 | 11 |
| lcc-placed | 12 | 12 | 11 | 12 | 12 | 11 | 12 | 12 | 12 |
| arxiv-placed | 13 | 13 | 13 | 13 | 13 | 13 | 13 | 13 | 13 |
| bk-placed | 14 | 14 | 15 | 14 | 14 | 14 | 14 | 14 | 14 |
| jel-placed | 15 | 15 | 14 | 15 | 15 | 15 | 15 | 15 | 15 |
| msc-placed | 16 | 16 | 16 | 16 | 16 | 16 | 16 | 16 | 16 |
| msc-gold | 17 | 17 | 17 | 17 | 17 | 17 | 17 | 17 | 17 |
| jel-gold | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 |

### holdout (10 rows)

| condition | Spearman (skill) | Kendall (skill) | Spearman (J) | Kendall (J) |
| --- | --- | --- | --- | --- |
| minilm-k10 | 1.000 [0.964, 1.000] | 1.000 [0.911, 1.000] | 1.000 [0.952, 1.000] | 1.000 [0.867, 1.000] |
| minilm-k50 | 1.000 [0.976, 1.000] | 1.000 [0.911, 1.000] | 1.000 [0.952, 1.000] | 1.000 [0.867, 1.000] |
| bge-small-k10 | 0.976 [0.939, 1.000] | 0.911 [0.867, 1.000] | 0.952 [0.915, 1.000] | 0.867 [0.822, 1.000] |
| bge-small-k20 | 0.976 [0.952, 1.000] | 0.911 [0.867, 1.000] | 0.976 [0.915, 1.000] | 0.911 [0.822, 1.000] |
| bge-small-k50 | 0.976 [0.952, 1.000] | 0.911 [0.867, 1.000] | 0.976 [0.915, 1.000] | 0.911 [0.822, 1.000] |
| nomic-k10 | 0.988 [0.952, 1.000] | 0.956 [0.867, 1.000] | 1.000 [0.915, 1.000] | 1.000 [0.822, 1.000] |
| nomic-k20 | 0.988 [0.963, 1.000] | 0.956 [0.910, 1.000] | 1.000 [0.927, 1.000] | 1.000 [0.866, 1.000] |
| nomic-k50 | 0.988 [0.952, 1.000] | 0.956 [0.867, 1.000] | 0.988 [0.915, 1.000] | 0.956 [0.822, 1.000] |
| min over all pairs | 0.976 | 0.911 | 0.952 | 0.867 |

Rank by skill under each condition:

| row | minilm-k10 | minilm-k20 | minilm-k50 | bge-small-k10 | bge-small-k20 | bge-small-k50 | nomic-k10 | nomic-k20 | nomic-k50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-own | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| lcc-full-gold | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| lcc-gold | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| lcc-full-placed | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| ilc2-placed | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| unesco-thesaurus-placed | 6 | 6 | 6 | 7 | 7 | 7 | 6 | 6 | 6 |
| learned-placed | 7 | 7 | 7 | 6 | 6 | 6 | 7 | 7 | 7 |
| lcc-placed | 8 | 8 | 8 | 8 | 8 | 8 | 8 | 8 | 8 |
| fdc-placed | 9 | 9 | 9 | 10 | 10 | 10 | 10 | 10 | 10 |
| bk-placed | 10 | 10 | 10 | 9 | 9 | 9 | 9 | 9 | 9 |

### split (18 rows)

| condition | Spearman (skill) | Kendall (skill) | Spearman (J) | Kendall (J) |
| --- | --- | --- | --- | --- |
| minilm-k10 | 0.996 [0.975, 1.000] | 0.974 [0.908, 1.000] | 0.996 [0.975, 1.000] | 0.974 [0.908, 1.000] |
| minilm-k50 | 0.994 [0.975, 1.000] | 0.974 [0.908, 1.000] | 0.998 [0.977, 1.000] | 0.987 [0.908, 1.000] |
| bge-small-k10 | 0.994 [0.959, 0.998] | 0.961 [0.882, 0.987] | 0.986 [0.959, 0.998] | 0.935 [0.869, 0.987] |
| bge-small-k20 | 0.996 [0.965, 0.998] | 0.974 [0.895, 0.987] | 0.992 [0.967, 0.998] | 0.961 [0.895, 0.987] |
| bge-small-k50 | 1.000 [0.969, 0.998] | 1.000 [0.895, 0.987] | 0.998 [0.971, 0.998] | 0.987 [0.895, 0.987] |
| nomic-k10 | 0.986 [0.950, 0.996] | 0.935 [0.856, 0.974] | 0.981 [0.953, 0.996] | 0.922 [0.856, 0.974] |
| nomic-k20 | 0.992 [0.959, 0.998] | 0.961 [0.869, 0.987] | 0.994 [0.961, 0.998] | 0.974 [0.882, 0.987] |
| nomic-k50 | 1.000 [0.967, 0.998] | 1.000 [0.895, 0.987] | 0.998 [0.967, 0.998] | 0.987 [0.895, 0.987] |
| min over all pairs | 0.973 | 0.908 | 0.973 | 0.908 |

Rank by skill under each condition:

| row | minilm-k10 | minilm-k20 | minilm-k50 | bge-small-k10 | bge-small-k20 | bge-small-k50 | nomic-k10 | nomic-k20 | nomic-k50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-heldout | 2 | 1 | 1 | 1 | 1 | 1 | 2 | 1 | 1 |
| learned-own | 1 | 2 | 2 | 2 | 2 | 2 | 1 | 2 | 2 |
| arxiv-gold | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| openalex-topics-placed | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| unesco-thesaurus-placed | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| ipc-placed | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| ilc2-placed | 7 | 7 | 7 | 8 | 7 | 7 | 7 | 7 | 7 |
| lcc-full-placed | 9 | 8 | 10 | 7 | 8 | 8 | 8 | 8 | 8 |
| fdc-placed | 8 | 9 | 8 | 9 | 9 | 9 | 10 | 9 | 9 |
| acm-ccs-placed | 10 | 10 | 9 | 11 | 11 | 10 | 12 | 12 | 10 |
| eurovoc-placed | 11 | 11 | 11 | 10 | 10 | 11 | 9 | 10 | 11 |
| lcc-placed | 12 | 12 | 12 | 12 | 12 | 12 | 11 | 11 | 12 |
| arxiv-placed | 13 | 13 | 13 | 14 | 14 | 13 | 14 | 14 | 13 |
| bk-placed | 14 | 14 | 14 | 13 | 13 | 14 | 13 | 13 | 14 |
| jel-placed | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 | 15 |
| msc-placed | 16 | 16 | 16 | 16 | 16 | 16 | 16 | 16 | 16 |
| msc-gold | 17 | 17 | 17 | 17 | 17 | 17 | 17 | 17 | 17 |
| jel-gold | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 |

## The learned scheme's edge

Margin = learned skill minus the best other row's skill on the same graph (positive: learned ahead); J margin = best other J minus learned J (positive: learned ahead). Intervals are paired item-bootstrap 95% intervals of the difference.

### split-heldout: MiniLM 80% fit, held-out works inserted with MiniLM vectors, vs every Paper 1 scheme on the same 2,000 works

| condition | learned skill [95% CI] | best other (skill) | skill margin [95% CI] | best other (J) | J margin [95% CI] |
| --- | --- | --- | --- | --- | --- |
| minilm-k10 | 0.8825 [0.8632, 0.9031] | arxiv-gold 0.7065 | 0.1760 [0.1469, 0.2077] | arxiv-gold 0.4069 | 0.1667 [0.1388, 0.1963] |
| minilm-k20 | 0.9222 [0.9045, 0.9404] | arxiv-gold 0.7327 | 0.1895 [0.1637, 0.2179] | arxiv-gold 0.4220 | 0.1692 [0.1462, 0.1940] |
| minilm-k50 | 0.9557 [0.9405, 0.9720] | arxiv-gold 0.7562 | 0.1994 [0.1759, 0.2237] | arxiv-gold 0.4408 | 0.1676 [0.1479, 0.1877] |
| bge-small-k10 | 0.8585 [0.8361, 0.8813] | arxiv-gold 0.7038 | 0.1547 [0.1217, 0.1891] | arxiv-gold 0.4021 | 0.1480 [0.1163, 0.1797] |
| bge-small-k20 | 0.9010 [0.8815, 0.9206] | arxiv-gold 0.7353 | 0.1657 [0.1381, 0.1933] | arxiv-gold 0.4148 | 0.1489 [0.1242, 0.1737] |
| bge-small-k50 | 0.9321 [0.9167, 0.9470] | arxiv-gold 0.7566 | 0.1756 [0.1521, 0.1986] | arxiv-gold 0.4327 | 0.1492 [0.1286, 0.1682] |
| nomic-k10 | 0.8513 [0.8273, 0.8753] | arxiv-gold 0.6886 | 0.1626 [0.1332, 0.1958] | arxiv-gold 0.4254 | 0.1536 [0.1253, 0.1849] |
| nomic-k20 | 0.8915 [0.8710, 0.9110] | arxiv-gold 0.7241 | 0.1674 [0.1417, 0.1960] | arxiv-gold 0.4321 | 0.1488 [0.1261, 0.1733] |
| nomic-k50 | 0.9258 [0.9103, 0.9416] | arxiv-gold 0.7469 | 0.1789 [0.1568, 0.2032] | arxiv-gold 0.4527 | 0.1493 [0.1308, 0.1697] |

Survives every condition (both margins' intervals above 0): **yes**.

### corpus-own: MiniLM full fit, own placements (in-sample works), vs every Paper 1 scheme

| condition | learned skill [95% CI] | best other (skill) | skill margin [95% CI] | best other (J) | J margin [95% CI] |
| --- | --- | --- | --- | --- | --- |
| minilm-k10 | 0.9821 [0.9762, 0.9881] | arxiv-gold 0.7864 | 0.1957 [0.1849, 0.2056] | arxiv-gold 0.4107 | 0.1688 [0.1597, 0.1774] |
| minilm-k20 | 0.9760 [0.9753, 0.9768] | arxiv-gold 0.7861 | 0.1900 [0.1806, 0.1989] | arxiv-gold 0.4250 | 0.1606 [0.1527, 0.1683] |
| minilm-k50 | 0.9725 [0.9676, 0.9774] | arxiv-gold 0.7914 | 0.1810 [0.1724, 0.1898] | arxiv-gold 0.4476 | 0.1467 [0.1398, 0.1539] |
| bge-small-k10 | 0.9406 [0.9336, 0.9475] | arxiv-gold 0.7753 | 0.1653 [0.1545, 0.1759] | arxiv-gold 0.4114 | 0.1445 [0.1353, 0.1537] |
| bge-small-k20 | 0.9394 [0.9329, 0.9463] | arxiv-gold 0.7768 | 0.1626 [0.1525, 0.1723] | arxiv-gold 0.4243 | 0.1391 [0.1303, 0.1470] |
| bge-small-k50 | 0.9377 [0.9315, 0.9445] | arxiv-gold 0.7778 | 0.1599 [0.1506, 0.1692] | arxiv-gold 0.4453 | 0.1323 [0.1245, 0.1397] |
| nomic-k10 | 0.9432 [0.9369, 0.9501] | arxiv-gold 0.7709 | 0.1723 [0.1627, 0.1828] | arxiv-gold 0.4243 | 0.1486 [0.1404, 0.1574] |
| nomic-k20 | 0.9432 [0.9370, 0.9496] | arxiv-gold 0.7737 | 0.1695 [0.1602, 0.1791] | arxiv-gold 0.4382 | 0.1426 [0.1348, 0.1504] |
| nomic-k50 | 0.9535 [0.9468, 0.9604] | arxiv-gold 0.7853 | 0.1683 [0.1597, 0.1772] | arxiv-gold 0.4600 | 0.1348 [0.1282, 0.1419] |

Survives every condition (both margins' intervals above 0): **yes**.

### holdout-own: MiniLM fit, 500 books inserted with MiniLM vectors, vs every Paper 1 scheme

| condition | learned skill [95% CI] | best other (skill) | skill margin [95% CI] | best other (J) | J margin [95% CI] |
| --- | --- | --- | --- | --- | --- |
| minilm-k10 | 0.7599 [0.7124, 0.8040] | lcc-full-gold 0.4026 | 0.3574 [0.2915, 0.4117] | lcc-gold 0.7959 | 0.2620 [0.2084, 0.3042] |
| minilm-k20 | 0.7971 [0.7530, 0.8361] | lcc-full-gold 0.4212 | 0.3759 [0.3143, 0.4250] | lcc-gold 0.8006 | 0.2590 [0.2138, 0.2962] |
| minilm-k50 | 0.8223 [0.7802, 0.8583] | lcc-full-gold 0.4447 | 0.3776 [0.3206, 0.4244] | lcc-gold 0.8079 | 0.2395 [0.2004, 0.2732] |
| bge-small-k10 | 0.7124 [0.6693, 0.7561] | lcc-full-gold 0.4155 | 0.2969 [0.2316, 0.3523] | lcc-gold 0.7965 | 0.2101 [0.1632, 0.2506] |
| bge-small-k20 | 0.7508 [0.7118, 0.7897] | lcc-full-gold 0.4382 | 0.3127 [0.2577, 0.3638] | lcc-gold 0.8061 | 0.2024 [0.1660, 0.2368] |
| bge-small-k50 | 0.7935 [0.7532, 0.8337] | lcc-full-gold 0.4729 | 0.3207 [0.2668, 0.3710] | lcc-gold 0.8182 | 0.1834 [0.1523, 0.2120] |
| nomic-k10 | 0.6808 [0.6279, 0.7280] | lcc-full-gold 0.4004 | 0.2804 [0.2142, 0.3397] | lcc-gold 0.7909 | 0.2097 [0.1574, 0.2558] |
| nomic-k20 | 0.6974 [0.6485, 0.7456] | lcc-full-gold 0.4093 | 0.2881 [0.2270, 0.3446] | lcc-gold 0.8002 | 0.2037 [0.1579, 0.2449] |
| nomic-k50 | 0.7402 [0.6912, 0.7868] | lcc-full-gold 0.4462 | 0.2939 [0.2365, 0.3482] | lcc-gold 0.8101 | 0.1832 [0.1441, 0.2190] |

Survives every condition (both margins' intervals above 0): **yes**.

### corpus-placed: learned scheme placed by shelfmark lexical@2 vs every Paper 1 placed scheme

| condition | learned skill [95% CI] | best other (skill) | skill margin [95% CI] | best other (J) | J margin [95% CI] |
| --- | --- | --- | --- | --- | --- |
| minilm-k10 | 0.7267 [0.7157, 0.7360] | openalex-topics-placed 0.4274 | 0.2993 [0.2863, 0.3117] | openalex-topics-placed 0.7641 | 0.2380 [0.2271, 0.2486] |
| minilm-k20 | 0.7317 [0.7223, 0.7408] | openalex-topics-placed 0.4228 | 0.3090 [0.2973, 0.3205] | openalex-topics-placed 0.7757 | 0.2409 [0.2308, 0.2503] |
| minilm-k50 | 0.7473 [0.7381, 0.7559] | openalex-topics-placed 0.4236 | 0.3237 [0.3125, 0.3344] | openalex-topics-placed 0.7895 | 0.2423 [0.2329, 0.2508] |
| bge-small-k10 | 0.7082 [0.6976, 0.7176] | openalex-topics-placed 0.4153 | 0.2929 [0.2798, 0.3057] | openalex-topics-placed 0.7698 | 0.2359 [0.2244, 0.2471] |
| bge-small-k20 | 0.7157 [0.7048, 0.7250] | openalex-topics-placed 0.4125 | 0.3032 [0.2917, 0.3153] | openalex-topics-placed 0.7799 | 0.2393 [0.2295, 0.2496] |
| bge-small-k50 | 0.7258 [0.7163, 0.7345] | openalex-topics-placed 0.4093 | 0.3165 [0.3062, 0.3271] | openalex-topics-placed 0.7942 | 0.2419 [0.2327, 0.2505] |
| nomic-k10 | 0.7159 [0.7057, 0.7249] | openalex-topics-placed 0.4259 | 0.2901 [0.2774, 0.3021] | openalex-topics-placed 0.7656 | 0.2300 [0.2188, 0.2406] |
| nomic-k20 | 0.7254 [0.7156, 0.7344] | openalex-topics-placed 0.4222 | 0.3032 [0.2920, 0.3146] | openalex-topics-placed 0.7777 | 0.2350 [0.2254, 0.2444] |
| nomic-k50 | 0.7463 [0.7363, 0.7551] | openalex-topics-placed 0.4255 | 0.3208 [0.3096, 0.3319] | openalex-topics-placed 0.7920 | 0.2369 [0.2281, 0.2458] |

Survives every condition (both margins' intervals above 0): **yes**.

### split-heldout-bge-small: bge-small 80% fit, held out (bge-small insertion vectors), vs every Paper 1 scheme

| condition | learned skill [95% CI] | best other (skill) | skill margin [95% CI] | best other (J) | J margin [95% CI] |
| --- | --- | --- | --- | --- | --- |
| minilm-k10 | 0.8836 [0.8634, 0.9032] | arxiv-gold 0.7065 | 0.1771 [0.1474, 0.2100] | arxiv-gold 0.4069 | 0.1679 [0.1394, 0.1983] |
| minilm-k20 | 0.9134 [0.8956, 0.9311] | arxiv-gold 0.7327 | 0.1807 [0.1554, 0.2091] | arxiv-gold 0.4220 | 0.1614 [0.1386, 0.1867] |
| minilm-k50 | 0.9410 [0.9258, 0.9575] | arxiv-gold 0.7562 | 0.1847 [0.1621, 0.2095] | arxiv-gold 0.4408 | 0.1553 [0.1365, 0.1756] |
| bge-small-k10 | 0.8985 [0.8790, 0.9164] | arxiv-gold 0.7038 | 0.1947 [0.1616, 0.2290] | arxiv-gold 0.4021 | 0.1866 [0.1546, 0.2185] |
| bge-small-k20 | 0.9379 [0.9209, 0.9549] | arxiv-gold 0.7353 | 0.2026 [0.1744, 0.2322] | arxiv-gold 0.4148 | 0.1824 [0.1568, 0.2080] |
| bge-small-k50 | 0.9641 [0.9501, 0.9788] | arxiv-gold 0.7566 | 0.2075 [0.1831, 0.2333] | arxiv-gold 0.4327 | 0.1767 [0.1561, 0.1976] |
| nomic-k10 | 0.8786 [0.8563, 0.8993] | arxiv-gold 0.6886 | 0.1899 [0.1603, 0.2243] | arxiv-gold 0.4254 | 0.1796 [0.1515, 0.2125] |
| nomic-k20 | 0.9092 [0.8892, 0.9282] | arxiv-gold 0.7241 | 0.1851 [0.1566, 0.2149] | arxiv-gold 0.4321 | 0.1648 [0.1390, 0.1902] |
| nomic-k50 | 0.9370 [0.9223, 0.9531] | arxiv-gold 0.7469 | 0.1902 [0.1672, 0.2145] | arxiv-gold 0.4527 | 0.1589 [0.1398, 0.1798] |

Survives every condition (both margins' intervals above 0): **yes**.

### split-heldout-nomic: nomic 80% fit, held out (nomic insertion vectors), vs every Paper 1 scheme

| condition | learned skill [95% CI] | best other (skill) | skill margin [95% CI] | best other (J) | J margin [95% CI] |
| --- | --- | --- | --- | --- | --- |
| minilm-k10 | 0.8652 [0.8424, 0.8858] | arxiv-gold 0.7065 | 0.1587 [0.1285, 0.1902] | arxiv-gold 0.4069 | 0.1498 [0.1212, 0.1797] |
| minilm-k20 | 0.8987 [0.8781, 0.9187] | arxiv-gold 0.7327 | 0.1660 [0.1402, 0.1933] | arxiv-gold 0.4220 | 0.1477 [0.1248, 0.1719] |
| minilm-k50 | 0.9303 [0.9148, 0.9464] | arxiv-gold 0.7562 | 0.1741 [0.1531, 0.1985] | arxiv-gold 0.4408 | 0.1457 [0.1281, 0.1655] |
| bge-small-k10 | 0.8661 [0.8433, 0.8875] | arxiv-gold 0.7038 | 0.1623 [0.1301, 0.1954] | arxiv-gold 0.4021 | 0.1549 [0.1238, 0.1870] |
| bge-small-k20 | 0.9117 [0.8928, 0.9313] | arxiv-gold 0.7353 | 0.1764 [0.1508, 0.2052] | arxiv-gold 0.4148 | 0.1582 [0.1346, 0.1840] |
| bge-small-k50 | 0.9428 [0.9284, 0.9572] | arxiv-gold 0.7566 | 0.1863 [0.1640, 0.2104] | arxiv-gold 0.4327 | 0.1580 [0.1389, 0.1778] |
| nomic-k10 | 0.8762 [0.8528, 0.8987] | arxiv-gold 0.6886 | 0.1875 [0.1578, 0.2188] | arxiv-gold 0.4254 | 0.1768 [0.1484, 0.2062] |
| nomic-k20 | 0.9178 [0.8991, 0.9366] | arxiv-gold 0.7241 | 0.1937 [0.1682, 0.2211] | arxiv-gold 0.4321 | 0.1719 [0.1495, 0.1955] |
| nomic-k50 | 0.9531 [0.9388, 0.9680] | arxiv-gold 0.7469 | 0.2062 [0.1849, 0.2303] | arxiv-gold 0.4527 | 0.1718 [0.1540, 0.1910] |

Survives every condition (both margins' intervals above 0): **yes**.

## Cross-fit: each learned fit scored on each model's k = 20 graph

| fit | scored on | corpus skill (in-sample works) | split skill (held out) | split J | holdout skill |
| --- | --- | --- | --- | ---: | --- |
| minilm | minilm-k20 | 0.9760 [0.9753, 0.9768] | 0.9222 [0.9045, 0.9404] | 0.2528 | 0.7971 [0.7530, 0.8361] |
| minilm | bge-small-k20 | 0.9394 [0.9329, 0.9463] | 0.9010 [0.8815, 0.9206] | 0.2659 | 0.7508 [0.7118, 0.7897] |
| minilm | nomic-k20 | 0.9432 [0.9370, 0.9496] | 0.8915 [0.8710, 0.9110] | 0.2833 | 0.6974 [0.6485, 0.7456] |
| bge-small | minilm-k20 | 0.9464 [0.9398, 0.9531] | 0.9134 [0.8956, 0.9311] | 0.2606 | 0.6916 [0.6495, 0.7312] |
| bge-small | bge-small-k20 | 0.9730 [0.9714, 0.9747] | 0.9379 [0.9209, 0.9549] | 0.2324 | 0.7839 [0.7384, 0.8321] |
| bge-small | nomic-k20 | 0.9481 [0.9417, 0.9548] | 0.9092 [0.8892, 0.9282] | 0.2674 | 0.6418 [0.6006, 0.6797] |
| nomic | minilm-k20 | 0.9453 [0.9386, 0.9513] | 0.8987 [0.8781, 0.9187] | 0.2743 | 0.7957 [0.7592, 0.8271] |
| nomic | bge-small-k20 | 0.9516 [0.9451, 0.9581] | 0.9117 [0.8928, 0.9313] | 0.2566 | 0.8074 [0.7608, 0.8523] |
| nomic | nomic-k20 | 0.9754 [0.9746, 0.9763] | 0.9178 [0.8991, 0.9366] | 0.2602 | 0.8373 [0.7994, 0.8707] |

## Every run

### corpus

| condition | row | coverage [95% CI] | cost_norm [95% CI] | MW revenue_norm [95% CI] | skill [95% CI] | J | eff. classes [95% CI] | mean depth | nav log a=1 [95% CI] | nav scan [95% CI] |
| --- | --- | --- | --- | --- | --- | ---: | --- | ---: | --- | --- |
| minilm-k10 | learned-own | 1.0000 [1.0000, 1.0000] | 0.1496 [0.1441, 0.1554] | 0.8504 [0.8446, 0.8559] | 0.9821 [0.9762, 0.9881] | 0.2418 | 39.7 [39.5, 39.9] | 4.98 | 18.45 [18.43, 18.46] | 156.5 [155.0, 159.3] |
| minilm-k10 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.1757 [0.1694, 0.1824] | 0.8243 [0.8176, 0.8306] | 0.9519 [0.9452, 0.9589] | 0.2690 | 30.4 [30.1, 30.6] | 5.00 | 18.67 [18.65, 18.69] | 211.4 [209.0, 214.7] |
| minilm-k10 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.1778 [0.1717, 0.1845] | 0.8222 [0.8155, 0.8283] | 0.9495 [0.9426, 0.9567] | 0.2701 | 38.2 [37.9, 38.4] | 4.98 | 18.46 [18.44, 18.47] | 181.5 [179.4, 184.7] |
| minilm-k10 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3190 [0.3092, 0.3285] | 0.6810 [0.6715, 0.6908] | 0.7864 [0.7760, 0.7966] | 0.4107 | 36.5 [35.8, 37.0] | 3.90 | 18.33 [18.29, 18.38] | 291.3 [284.2, 300.2] |
| minilm-k10 | learned-placed | 0.9993 [0.9987, 0.9997] | 0.3707 [0.3625, 0.3805] | 0.6293 [0.6195, 0.6375] | 0.7267 [0.7157, 0.7360] | 0.5261 | 33.5 [33.1, 33.7] | 4.38 | 31.08 [30.92, 31.24] | 1036.7 [1018.3, 1054.4] |
| minilm-k10 | openalex-topics-placed | 0.9732 [0.9701, 0.9762] | 0.6299 [0.6216, 0.6377] | 0.3701 [0.3623, 0.3784] | 0.4274 [0.4185, 0.4369] | 0.7641 | 77.6 [74.8, 77.6] | 4.45 | 26.85 [26.77, 26.93] | 311.1 [306.7, 316.8] |
| minilm-k10 | unesco-thesaurus-placed | 0.9731 [0.9698, 0.9762] | 0.7110 [0.7031, 0.7188] | 0.2890 [0.2812, 0.2969] | 0.3337 [0.3247, 0.3431] | 0.8444 | 146.7 [141.3, 147.7] | 4.90 | 26.68 [26.58, 26.78] | 304.7 [300.6, 309.9] |
| minilm-k10 | ipc-placed | 0.9799 [0.9772, 0.9826] | 0.7617 [0.7546, 0.7683] | 0.2383 [0.2317, 0.2454] | 0.2752 [0.2672, 0.2841] | 0.8982 | 317.8 [303.2, 318.8] | 6.31 | 27.31 [27.22, 27.42] | 322.1 [314.8, 331.0] |
| minilm-k10 | ilc2-placed | 0.9999 [0.9997, 1.0000] | 0.8117 [0.8040, 0.8198] | 0.1883 [0.1802, 0.1960] | 0.2175 [0.2083, 0.2267] | 0.9286 | 295.1 [282.7, 295.0] | 5.09 | 23.38 [23.28, 23.48] | 135.6 [133.3, 139.3] |
| minilm-k10 | lcc-full-placed | 0.9999 [0.9997, 1.0000] | 0.8150 [0.8081, 0.8219] | 0.1850 [0.1781, 0.1919] | 0.2136 [0.2055, 0.2219] | 0.9294 | 657.2 [614.5, 640.4] | 6.06 | 22.86 [22.78, 22.96] | 52.3 [52.0, 53.6] |
| minilm-k10 | acm-ccs-placed | 0.7445 [0.7358, 0.7528] | 0.8247 [0.8163, 0.8319] | 0.1753 [0.1681, 0.1837] | 0.2024 [0.1938, 0.2122] | 0.9434 | 84.9 [81.5, 86.5] | 3.61 | 23.73 [23.59, 23.88] | 1476.2 [1469.2, 1484.4] |
| minilm-k10 | fdc-placed | 0.7696 [0.7615, 0.7777] | 0.8258 [0.8173, 0.8333] | 0.1742 [0.1667, 0.1827] | 0.2012 [0.1925, 0.2110] | 0.9536 | 54.5 [53.0, 55.3] | 3.21 | 25.56 [25.40, 25.73] | 1413.5 [1404.0, 1423.7] |
| minilm-k10 | eurovoc-placed | 0.9737 [0.9707, 0.9768] | 0.8366 [0.8296, 0.8439] | 0.1634 [0.1561, 0.1704] | 0.1886 [0.1803, 0.1969] | 0.9606 | 269.3 [259.0, 270.1] | 4.98 | 24.80 [24.71, 24.89] | 268.7 [265.9, 272.2] |
| minilm-k10 | lcc-placed | 0.8765 [0.8696, 0.8826] | 0.8405 [0.8327, 0.8472] | 0.1595 [0.1528, 0.1673] | 0.1841 [0.1767, 0.1930] | 0.9673 | 129.0 [124.6, 130.5] | 4.08 | 25.36 [25.22, 25.48] | 824.9 [819.6, 830.7] |
| minilm-k10 | arxiv-placed | 0.4260 [0.4164, 0.4357] | 0.8892 [0.8813, 0.8962] | 0.1108 [0.1038, 0.1187] | 0.1279 [0.1200, 0.1375] | 0.9909 | 14.5 [14.1, 14.9] | 2.21 | 20.34 [20.18, 20.51] | 3231.4 [3214.2, 3251.4] |
| minilm-k10 | bk-placed | 0.4806 [0.4712, 0.4899] | 0.8974 [0.8899, 0.9043] | 0.1026 [0.0957, 0.1101] | 0.1185 [0.1103, 0.1272] | 1.0021 | 75.0 [72.0, 76.4] | 3.15 | 20.94 [20.77, 21.09] | 2736.4 [2730.0, 2743.8] |
| minilm-k10 | jel-placed | 0.4786 [0.4697, 0.4881] | 0.8991 [0.8923, 0.9062] | 0.1009 [0.0938, 0.1077] | 0.1165 [0.1085, 0.1247] | 0.9962 | 28.9 [28.0, 29.7] | 2.27 | 19.41 [19.29, 19.52] | 2875.9 [2866.7, 2887.3] |
| minilm-k10 | msc-placed | 0.3916 [0.3819, 0.4008] | 0.9453 [0.9403, 0.9503] | 0.0547 [0.0497, 0.0597] | 0.0631 [0.0574, 0.0691] | 1.0345 | 114.4 [107.3, 116.4] | 2.00 | 17.82 [17.71, 17.93] | 3128.5 [3125.8, 3131.9] |
| minilm-k10 | msc-gold | 0.1322 [0.1254, 0.1385] | 0.9829 [0.9798, 0.9860] | 0.0171 [0.0140, 0.0202] | 0.0198 [0.0162, 0.0233] | 1.0589 | 461.5 [373.5, 406.4] | 1.39 | 15.21 [15.15, 15.27] | 4371.9 [4372.3, 4372.5] |
| minilm-k10 | jel-gold | 0.0002 [0.0000, 0.0005] | 1.0000 [1.0000, 1.0000] | 0.0000 [0.0000, 0.0000] | 0.0000 [0.0000, 0.0000] | 1.0714 | 6.0 [1.0, 6.0] | 1.00 | 14.29 [14.29, 14.29] | 5001.5 [5002.0, 5002.0] |
| minilm-k20 | learned-own | 1.0000 [1.0000, 1.0000] | 0.1722 [0.1662, 0.1781] | 0.8278 [0.8219, 0.8338] | 0.9760 [0.9753, 0.9768] | 0.2644 | 39.7 [39.5, 39.9] | 4.98 | 18.45 [18.43, 18.46] | 156.5 [155.0, 159.3] |
| minilm-k20 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.1973 [0.1909, 0.2038] | 0.8027 [0.7962, 0.8091] | 0.9464 [0.9398, 0.9531] | 0.2906 | 30.4 [30.1, 30.6] | 5.00 | 18.67 [18.65, 18.69] | 211.4 [209.0, 214.7] |
| minilm-k20 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.1983 [0.1920, 0.2052] | 0.8017 [0.7948, 0.8080] | 0.9453 [0.9386, 0.9513] | 0.2906 | 38.2 [37.9, 38.4] | 4.98 | 18.46 [18.44, 18.47] | 181.5 [179.4, 184.7] |
| minilm-k20 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3333 [0.3240, 0.3424] | 0.6667 [0.6576, 0.6760] | 0.7861 [0.7770, 0.7954] | 0.4250 | 36.5 [35.8, 37.0] | 3.90 | 18.33 [18.29, 18.38] | 291.3 [284.2, 300.2] |
| minilm-k20 | learned-placed | 0.9993 [0.9987, 0.9997] | 0.3794 [0.3712, 0.3884] | 0.6206 [0.6116, 0.6288] | 0.7317 [0.7223, 0.7408] | 0.5347 | 33.5 [33.1, 33.7] | 4.38 | 31.08 [30.92, 31.24] | 1036.7 [1018.3, 1054.4] |
| minilm-k20 | openalex-topics-placed | 0.9732 [0.9701, 0.9762] | 0.6414 [0.6343, 0.6485] | 0.3586 [0.3515, 0.3657] | 0.4228 [0.4144, 0.4313] | 0.7757 | 77.6 [74.8, 77.6] | 4.45 | 26.85 [26.77, 26.93] | 311.1 [306.7, 316.8] |
| minilm-k20 | unesco-thesaurus-placed | 0.9731 [0.9698, 0.9762] | 0.7210 [0.7143, 0.7280] | 0.2790 [0.2720, 0.2857] | 0.3289 [0.3207, 0.3370] | 0.8544 | 146.7 [141.3, 147.7] | 4.90 | 26.68 [26.58, 26.78] | 304.7 [300.6, 309.9] |
| minilm-k20 | ipc-placed | 0.9799 [0.9772, 0.9826] | 0.7690 [0.7631, 0.7748] | 0.2310 [0.2252, 0.2369] | 0.2724 [0.2651, 0.2798] | 0.9056 | 317.8 [303.2, 318.8] | 6.31 | 27.31 [27.22, 27.42] | 322.1 [314.8, 331.0] |
| minilm-k20 | ilc2-placed | 0.9999 [0.9997, 1.0000] | 0.8189 [0.8124, 0.8256] | 0.1811 [0.1744, 0.1876] | 0.2135 [0.2055, 0.2212] | 0.9358 | 295.1 [282.7, 295.0] | 5.09 | 23.38 [23.28, 23.48] | 135.6 [133.3, 139.3] |
| minilm-k20 | lcc-full-placed | 0.9999 [0.9997, 1.0000] | 0.8228 [0.8169, 0.8286] | 0.1772 [0.1714, 0.1831] | 0.2089 [0.2019, 0.2155] | 0.9372 | 657.2 [614.5, 640.4] | 6.06 | 22.86 [22.78, 22.96] | 52.3 [52.0, 53.6] |
| minilm-k20 | acm-ccs-placed | 0.7445 [0.7358, 0.7528] | 0.8306 [0.8233, 0.8375] | 0.1694 [0.1625, 0.1767] | 0.1997 [0.1915, 0.2083] | 0.9493 | 84.9 [81.5, 86.5] | 3.61 | 23.73 [23.59, 23.88] | 1476.2 [1469.2, 1484.4] |
| minilm-k20 | fdc-placed | 0.7696 [0.7615, 0.7777] | 0.8337 [0.8265, 0.8402] | 0.1663 [0.1598, 0.1735] | 0.1961 [0.1883, 0.2047] | 0.9615 | 54.5 [53.0, 55.3] | 3.21 | 25.56 [25.40, 25.73] | 1413.5 [1404.0, 1423.7] |
| minilm-k20 | eurovoc-placed | 0.9737 [0.9707, 0.9768] | 0.8479 [0.8422, 0.8534] | 0.1521 [0.1466, 0.1578] | 0.1794 [0.1727, 0.1863] | 0.9718 | 269.3 [259.0, 270.1] | 4.98 | 24.80 [24.71, 24.89] | 268.7 [265.9, 272.2] |
| minilm-k20 | lcc-placed | 0.8765 [0.8696, 0.8826] | 0.8487 [0.8426, 0.8544] | 0.1513 [0.1456, 0.1574] | 0.1783 [0.1718, 0.1854] | 0.9755 | 129.0 [124.6, 130.5] | 4.08 | 25.36 [25.22, 25.48] | 824.9 [819.6, 830.7] |
| minilm-k20 | arxiv-placed | 0.4260 [0.4164, 0.4357] | 0.8912 [0.8840, 0.8977] | 0.1088 [0.1023, 0.1160] | 0.1282 [0.1206, 0.1368] | 0.9929 | 14.5 [14.1, 14.9] | 2.21 | 20.34 [20.18, 20.51] | 3231.4 [3214.2, 3251.4] |
| minilm-k20 | bk-placed | 0.4806 [0.4712, 0.4899] | 0.9019 [0.8947, 0.9083] | 0.0981 [0.0917, 0.1053] | 0.1157 [0.1082, 0.1241] | 1.0066 | 75.0 [72.0, 76.4] | 3.15 | 20.94 [20.77, 21.09] | 2736.4 [2730.0, 2743.8] |
| minilm-k20 | jel-placed | 0.4786 [0.4697, 0.4881] | 0.9028 [0.8965, 0.9087] | 0.0972 [0.0913, 0.1035] | 0.1146 [0.1078, 0.1223] | 0.9998 | 28.9 [28.0, 29.7] | 2.27 | 19.41 [19.29, 19.52] | 2875.9 [2866.7, 2887.3] |
| minilm-k20 | msc-placed | 0.3916 [0.3819, 0.4008] | 0.9498 [0.9454, 0.9544] | 0.0502 [0.0456, 0.0546] | 0.0592 [0.0538, 0.0643] | 1.0389 | 114.4 [107.3, 116.4] | 2.00 | 17.82 [17.71, 17.93] | 3128.5 [3125.8, 3131.9] |
| minilm-k20 | msc-gold | 0.1322 [0.1254, 0.1385] | 0.9843 [0.9817, 0.9869] | 0.0157 [0.0131, 0.0183] | 0.0185 [0.0154, 0.0215] | 1.0604 | 461.5 [373.5, 406.4] | 1.39 | 15.21 [15.15, 15.27] | 4371.9 [4372.3, 4372.5] |
| minilm-k20 | jel-gold | 0.0002 [0.0000, 0.0005] | 1.0000 [1.0000, 1.0000] | 0.0000 [0.0000, 0.0000] | 0.0000 [0.0000, 0.0000] | 1.0714 | 6.0 [1.0, 6.0] | 1.00 | 14.29 [14.29, 14.29] | 5001.5 [5002.0, 5002.0] |
| minilm-k50 | learned-own | 1.0000 [1.0000, 1.0000] | 0.2087 [0.2028, 0.2148] | 0.7913 [0.7852, 0.7972] | 0.9725 [0.9676, 0.9774] | 0.3009 | 39.7 [39.5, 39.9] | 4.98 | 18.45 [18.43, 18.46] | 156.5 [155.0, 159.3] |
| minilm-k50 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.2281 [0.2219, 0.2352] | 0.7719 [0.7648, 0.7781] | 0.9485 [0.9415, 0.9546] | 0.3204 | 38.2 [37.9, 38.4] | 4.98 | 18.46 [18.44, 18.47] | 181.5 [179.4, 184.7] |
| minilm-k50 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.2290 [0.2225, 0.2359] | 0.7710 [0.7641, 0.7775] | 0.9475 [0.9408, 0.9544] | 0.3224 | 30.4 [30.1, 30.6] | 5.00 | 18.67 [18.65, 18.69] | 211.4 [209.0, 214.7] |
| minilm-k50 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3560 [0.3473, 0.3646] | 0.6440 [0.6354, 0.6527] | 0.7914 [0.7825, 0.8004] | 0.4476 | 36.5 [35.8, 37.0] | 3.90 | 18.33 [18.29, 18.38] | 291.3 [284.2, 300.2] |
| minilm-k50 | learned-placed | 0.9993 [0.9987, 0.9997] | 0.3919 [0.3843, 0.4003] | 0.6081 [0.5997, 0.6157] | 0.7473 [0.7381, 0.7559] | 0.5472 | 33.5 [33.1, 33.7] | 4.38 | 31.08 [30.92, 31.24] | 1036.7 [1018.3, 1054.4] |
| minilm-k50 | openalex-topics-placed | 0.9732 [0.9701, 0.9762] | 0.6553 [0.6490, 0.6617] | 0.3447 [0.3383, 0.3510] | 0.4236 [0.4163, 0.4313] | 0.7895 | 77.6 [74.8, 77.6] | 4.45 | 26.85 [26.77, 26.93] | 311.1 [306.7, 316.8] |
| minilm-k50 | unesco-thesaurus-placed | 0.9731 [0.9698, 0.9762] | 0.7366 [0.7311, 0.7424] | 0.2634 [0.2576, 0.2689] | 0.3237 [0.3164, 0.3304] | 0.8700 | 146.7 [141.3, 147.7] | 4.90 | 26.68 [26.58, 26.78] | 304.7 [300.6, 309.9] |
| minilm-k50 | ipc-placed | 0.9799 [0.9772, 0.9826] | 0.7766 [0.7714, 0.7818] | 0.2234 [0.2182, 0.2286] | 0.2745 [0.2677, 0.2816] | 0.9132 | 317.8 [303.2, 318.8] | 6.31 | 27.31 [27.22, 27.42] | 322.1 [314.8, 331.0] |
| minilm-k50 | ilc2-placed | 0.9999 [0.9997, 1.0000] | 0.8302 [0.8249, 0.8364] | 0.1698 [0.1636, 0.1751] | 0.2086 [0.2011, 0.2153] | 0.9471 | 295.1 [282.7, 295.0] | 5.09 | 23.38 [23.28, 23.48] | 135.6 [133.3, 139.3] |
| minilm-k50 | lcc-full-placed | 0.9999 [0.9997, 1.0000] | 0.8323 [0.8273, 0.8373] | 0.1677 [0.1627, 0.1727] | 0.2060 [0.1997, 0.2124] | 0.9467 | 657.2 [614.5, 640.4] | 6.06 | 22.86 [22.78, 22.96] | 52.3 [52.0, 53.6] |
| minilm-k50 | acm-ccs-placed | 0.7445 [0.7358, 0.7528] | 0.8398 [0.8338, 0.8456] | 0.1602 [0.1544, 0.1662] | 0.1969 [0.1894, 0.2049] | 0.9584 | 84.9 [81.5, 86.5] | 3.61 | 23.73 [23.59, 23.88] | 1476.2 [1469.2, 1484.4] |
| minilm-k50 | fdc-placed | 0.7696 [0.7615, 0.7777] | 0.8433 [0.8374, 0.8491] | 0.1567 [0.1509, 0.1626] | 0.1926 [0.1859, 0.2002] | 0.9711 | 54.5 [53.0, 55.3] | 3.21 | 25.56 [25.40, 25.73] | 1413.5 [1404.0, 1423.7] |
| minilm-k50 | lcc-placed | 0.8765 [0.8696, 0.8826] | 0.8600 [0.8549, 0.8646] | 0.1400 [0.1354, 0.1451] | 0.1720 [0.1663, 0.1781] | 0.9868 | 129.0 [124.6, 130.5] | 4.08 | 25.36 [25.22, 25.48] | 824.9 [819.6, 830.7] |
| minilm-k50 | eurovoc-placed | 0.9737 [0.9707, 0.9768] | 0.8612 [0.8568, 0.8657] | 0.1388 [0.1343, 0.1432] | 0.1705 [0.1650, 0.1763] | 0.9852 | 269.3 [259.0, 270.1] | 4.98 | 24.80 [24.71, 24.89] | 268.7 [265.9, 272.2] |
| minilm-k50 | arxiv-placed | 0.4260 [0.4164, 0.4357] | 0.8940 [0.8871, 0.9002] | 0.1060 [0.0998, 0.1129] | 0.1303 [0.1225, 0.1391] | 0.9957 | 14.5 [14.1, 14.9] | 2.21 | 20.34 [20.18, 20.51] | 3231.4 [3214.2, 3251.4] |
| minilm-k50 | jel-placed | 0.4786 [0.4697, 0.4881] | 0.9062 [0.9001, 0.9118] | 0.0938 [0.0882, 0.0999] | 0.1153 [0.1082, 0.1228] | 1.0032 | 28.9 [28.0, 29.7] | 2.27 | 19.41 [19.29, 19.52] | 2875.9 [2866.7, 2887.3] |
| minilm-k50 | bk-placed | 0.4806 [0.4712, 0.4899] | 0.9077 [0.9015, 0.9130] | 0.0923 [0.0870, 0.0985] | 0.1135 [0.1069, 0.1213] | 1.0123 | 75.0 [72.0, 76.4] | 3.15 | 20.94 [20.77, 21.09] | 2736.4 [2730.0, 2743.8] |
| minilm-k50 | msc-placed | 0.3916 [0.3819, 0.4008] | 0.9579 [0.9544, 0.9616] | 0.0421 [0.0384, 0.0456] | 0.0518 [0.0472, 0.0562] | 1.0470 | 114.4 [107.3, 116.4] | 2.00 | 17.82 [17.71, 17.93] | 3128.5 [3125.8, 3131.9] |
| minilm-k50 | msc-gold | 0.1322 [0.1254, 0.1385] | 0.9870 [0.9848, 0.9891] | 0.0130 [0.0109, 0.0152] | 0.0160 [0.0134, 0.0187] | 1.0631 | 461.5 [373.5, 406.4] | 1.39 | 15.21 [15.15, 15.27] | 4371.9 [4372.3, 4372.5] |
| minilm-k50 | jel-gold | 0.0002 [0.0000, 0.0005] | 1.0000 [1.0000, 1.0000] | -0.0000 [-0.0000, -0.0000] | -0.0000 [-0.0000, -0.0000] | 1.0714 | 6.0 [1.0, 6.0] | 1.00 | 14.29 [14.29, 14.29] | 5001.5 [5002.0, 5002.0] |
| bge-small-k10 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.1423 [0.1368, 0.1482] | 0.8577 [0.8518, 0.8632] | 0.9776 [0.9712, 0.9835] | 0.2356 | 30.4 [30.1, 30.6] | 5.00 | 18.67 [18.65, 18.69] | 211.4 [209.0, 214.7] |
| bge-small-k10 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.1624 [0.1566, 0.1695] | 0.8376 [0.8305, 0.8434] | 0.9546 [0.9485, 0.9612] | 0.2547 | 38.2 [37.9, 38.4] | 4.98 | 18.46 [18.44, 18.47] | 181.5 [179.4, 184.7] |
| bge-small-k10 | learned-own | 1.0000 [1.0000, 1.0000] | 0.1747 [0.1682, 0.1817] | 0.8253 [0.8183, 0.8318] | 0.9406 [0.9336, 0.9475] | 0.2670 | 39.7 [39.5, 39.9] | 4.98 | 18.45 [18.43, 18.46] | 156.5 [155.0, 159.3] |
| bge-small-k10 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3198 [0.3103, 0.3303] | 0.6802 [0.6697, 0.6897] | 0.7753 [0.7647, 0.7855] | 0.4114 | 36.5 [35.8, 37.0] | 3.90 | 18.33 [18.29, 18.38] | 291.3 [284.2, 300.2] |
| bge-small-k10 | learned-placed | 0.9993 [0.9987, 0.9997] | 0.3786 [0.3705, 0.3885] | 0.6214 [0.6115, 0.6295] | 0.7082 [0.6976, 0.7176] | 0.5340 | 33.5 [33.1, 33.7] | 4.38 | 31.08 [30.92, 31.24] | 1036.7 [1018.3, 1054.4] |
| bge-small-k10 | openalex-topics-placed | 0.9732 [0.9701, 0.9762] | 0.6356 [0.6275, 0.6437] | 0.3644 [0.3563, 0.3725] | 0.4153 [0.4063, 0.4249] | 0.7698 | 77.6 [74.8, 77.6] | 4.45 | 26.85 [26.77, 26.93] | 311.1 [306.7, 316.8] |
| bge-small-k10 | unesco-thesaurus-placed | 0.9731 [0.9698, 0.9762] | 0.7115 [0.7036, 0.7189] | 0.2885 [0.2811, 0.2964] | 0.3288 [0.3201, 0.3379] | 0.8449 | 146.7 [141.3, 147.7] | 4.90 | 26.68 [26.58, 26.78] | 304.7 [300.6, 309.9] |
| bge-small-k10 | ipc-placed | 0.9799 [0.9772, 0.9826] | 0.7669 [0.7599, 0.7739] | 0.2331 [0.2261, 0.2401] | 0.2657 [0.2575, 0.2740] | 0.9034 | 317.8 [303.2, 318.8] | 6.31 | 27.31 [27.22, 27.42] | 322.1 [314.8, 331.0] |
| bge-small-k10 | ilc2-placed | 0.9999 [0.9997, 1.0000] | 0.8127 [0.8053, 0.8200] | 0.1873 [0.1800, 0.1947] | 0.2135 [0.2052, 0.2219] | 0.9296 | 295.1 [282.7, 295.0] | 5.09 | 23.38 [23.28, 23.48] | 135.6 [133.3, 139.3] |
| bge-small-k10 | lcc-full-placed | 0.9999 [0.9997, 1.0000] | 0.8154 [0.8088, 0.8224] | 0.1846 [0.1776, 0.1912] | 0.2104 [0.2027, 0.2179] | 0.9297 | 657.2 [614.5, 640.4] | 6.06 | 22.86 [22.78, 22.96] | 52.3 [52.0, 53.6] |
| bge-small-k10 | fdc-placed | 0.7696 [0.7615, 0.7777] | 0.8271 [0.8187, 0.8350] | 0.1729 [0.1650, 0.1813] | 0.1970 [0.1880, 0.2067] | 0.9550 | 54.5 [53.0, 55.3] | 3.21 | 25.56 [25.40, 25.73] | 1413.5 [1404.0, 1423.7] |
| bge-small-k10 | acm-ccs-placed | 0.7445 [0.7358, 0.7528] | 0.8273 [0.8187, 0.8348] | 0.1727 [0.1652, 0.1813] | 0.1968 [0.1881, 0.2065] | 0.9460 | 84.9 [81.5, 86.5] | 3.61 | 23.73 [23.59, 23.88] | 1476.2 [1469.2, 1484.4] |
| bge-small-k10 | eurovoc-placed | 0.9737 [0.9707, 0.9768] | 0.8373 [0.8307, 0.8443] | 0.1627 [0.1557, 0.1693] | 0.1855 [0.1775, 0.1932] | 0.9613 | 269.3 [259.0, 270.1] | 4.98 | 24.80 [24.71, 24.89] | 268.7 [265.9, 272.2] |
| bge-small-k10 | lcc-placed | 0.8765 [0.8696, 0.8826] | 0.8401 [0.8327, 0.8470] | 0.1599 [0.1530, 0.1673] | 0.1823 [0.1744, 0.1910] | 0.9669 | 129.0 [124.6, 130.5] | 4.08 | 25.36 [25.22, 25.48] | 824.9 [819.6, 830.7] |
| bge-small-k10 | arxiv-placed | 0.4260 [0.4164, 0.4357] | 0.8994 [0.8921, 0.9060] | 0.1006 [0.0940, 0.1079] | 0.1147 [0.1072, 0.1230] | 1.0011 | 14.5 [14.1, 14.9] | 2.21 | 20.34 [20.18, 20.51] | 3231.4 [3214.2, 3251.4] |
| bge-small-k10 | bk-placed | 0.4806 [0.4712, 0.4899] | 0.9005 [0.8932, 0.9074] | 0.0995 [0.0926, 0.1068] | 0.1134 [0.1055, 0.1218] | 1.0052 | 75.0 [72.0, 76.4] | 3.15 | 20.94 [20.77, 21.09] | 2736.4 [2730.0, 2743.8] |
| bge-small-k10 | jel-placed | 0.4786 [0.4697, 0.4881] | 0.9067 [0.9001, 0.9129] | 0.0933 [0.0871, 0.0999] | 0.1064 [0.0992, 0.1141] | 1.0037 | 28.9 [28.0, 29.7] | 2.27 | 19.41 [19.29, 19.52] | 2875.9 [2866.7, 2887.3] |
| bge-small-k10 | msc-placed | 0.3916 [0.3819, 0.4008] | 0.9437 [0.9383, 0.9490] | 0.0563 [0.0510, 0.0617] | 0.0642 [0.0581, 0.0705] | 1.0328 | 114.4 [107.3, 116.4] | 2.00 | 17.82 [17.71, 17.93] | 3128.5 [3125.8, 3131.9] |
| bge-small-k10 | msc-gold | 0.1322 [0.1254, 0.1385] | 0.9812 [0.9777, 0.9844] | 0.0188 [0.0156, 0.0223] | 0.0214 [0.0178, 0.0254] | 1.0573 | 461.5 [373.5, 406.4] | 1.39 | 15.21 [15.15, 15.27] | 4371.9 [4372.3, 4372.5] |
| bge-small-k10 | jel-gold | 0.0002 [0.0000, 0.0005] | 1.0000 [1.0000, 1.0000] | -0.0000 [-0.0000, -0.0000] | -0.0000 [-0.0000, -0.0000] | 1.0714 | 6.0 [1.0, 6.0] | 1.00 | 14.29 [14.29, 14.29] | 5001.5 [5002.0, 5002.0] |
| bge-small-k20 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.1641 [0.1586, 0.1704] | 0.8359 [0.8296, 0.8414] | 0.9730 [0.9714, 0.9747] | 0.2574 | 30.4 [30.1, 30.6] | 5.00 | 18.67 [18.65, 18.69] | 211.4 [209.0, 214.7] |
| bge-small-k20 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.1825 [0.1764, 0.1894] | 0.8175 [0.8106, 0.8236] | 0.9516 [0.9451, 0.9581] | 0.2748 | 38.2 [37.9, 38.4] | 4.98 | 18.46 [18.44, 18.47] | 181.5 [179.4, 184.7] |
| bge-small-k20 | learned-own | 1.0000 [1.0000, 1.0000] | 0.1930 [0.1863, 0.1998] | 0.8070 [0.8002, 0.8137] | 0.9394 [0.9329, 0.9463] | 0.2852 | 39.7 [39.5, 39.9] | 4.98 | 18.45 [18.43, 18.46] | 156.5 [155.0, 159.3] |
| bge-small-k20 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3327 [0.3241, 0.3421] | 0.6673 [0.6579, 0.6759] | 0.7768 [0.7674, 0.7864] | 0.4243 | 36.5 [35.8, 37.0] | 3.90 | 18.33 [18.29, 18.38] | 291.3 [284.2, 300.2] |
| bge-small-k20 | learned-placed | 0.9993 [0.9987, 0.9997] | 0.3852 [0.3775, 0.3946] | 0.6148 [0.6054, 0.6225] | 0.7157 [0.7048, 0.7250] | 0.5406 | 33.5 [33.1, 33.7] | 4.38 | 31.08 [30.92, 31.24] | 1036.7 [1018.3, 1054.4] |
| bge-small-k20 | openalex-topics-placed | 0.9732 [0.9701, 0.9762] | 0.6457 [0.6383, 0.6529] | 0.3543 [0.3471, 0.3617] | 0.4125 [0.4044, 0.4210] | 0.7799 | 77.6 [74.8, 77.6] | 4.45 | 26.85 [26.77, 26.93] | 311.1 [306.7, 316.8] |
| bge-small-k20 | unesco-thesaurus-placed | 0.9731 [0.9698, 0.9762] | 0.7242 [0.7173, 0.7308] | 0.2758 [0.2692, 0.2827] | 0.3210 [0.3135, 0.3293] | 0.8576 | 146.7 [141.3, 147.7] | 4.90 | 26.68 [26.58, 26.78] | 304.7 [300.6, 309.9] |
| bge-small-k20 | ipc-placed | 0.9799 [0.9772, 0.9826] | 0.7731 [0.7674, 0.7788] | 0.2269 [0.2212, 0.2326] | 0.2641 [0.2572, 0.2712] | 0.9097 | 317.8 [303.2, 318.8] | 6.31 | 27.31 [27.22, 27.42] | 322.1 [314.8, 331.0] |
| bge-small-k20 | ilc2-placed | 0.9999 [0.9997, 1.0000] | 0.8206 [0.8140, 0.8274] | 0.1794 [0.1726, 0.1860] | 0.2088 [0.2008, 0.2165] | 0.9375 | 295.1 [282.7, 295.0] | 5.09 | 23.38 [23.28, 23.48] | 135.6 [133.3, 139.3] |
| bge-small-k20 | lcc-full-placed | 0.9999 [0.9997, 1.0000] | 0.8220 [0.8163, 0.8277] | 0.1780 [0.1723, 0.1837] | 0.2072 [0.2005, 0.2141] | 0.9363 | 657.2 [614.5, 640.4] | 6.06 | 22.86 [22.78, 22.96] | 52.3 [52.0, 53.6] |
| bge-small-k20 | fdc-placed | 0.7696 [0.7615, 0.7777] | 0.8348 [0.8279, 0.8415] | 0.1652 [0.1585, 0.1721] | 0.1923 [0.1846, 0.2006] | 0.9626 | 54.5 [53.0, 55.3] | 3.21 | 25.56 [25.40, 25.73] | 1413.5 [1404.0, 1423.7] |
| bge-small-k20 | acm-ccs-placed | 0.7445 [0.7358, 0.7528] | 0.8350 [0.8277, 0.8415] | 0.1650 [0.1585, 0.1723] | 0.1920 [0.1841, 0.2005] | 0.9537 | 84.9 [81.5, 86.5] | 3.61 | 23.73 [23.59, 23.88] | 1476.2 [1469.2, 1484.4] |
| bge-small-k20 | eurovoc-placed | 0.9737 [0.9707, 0.9768] | 0.8488 [0.8433, 0.8548] | 0.1512 [0.1452, 0.1567] | 0.1760 [0.1689, 0.1826] | 0.9728 | 269.3 [259.0, 270.1] | 4.98 | 24.80 [24.71, 24.89] | 268.7 [265.9, 272.2] |
| bge-small-k20 | lcc-placed | 0.8765 [0.8696, 0.8826] | 0.8491 [0.8431, 0.8548] | 0.1509 [0.1452, 0.1569] | 0.1756 [0.1689, 0.1827] | 0.9759 | 129.0 [124.6, 130.5] | 4.08 | 25.36 [25.22, 25.48] | 824.9 [819.6, 830.7] |
| bge-small-k20 | arxiv-placed | 0.4260 [0.4164, 0.4357] | 0.9024 [0.8959, 0.9081] | 0.0976 [0.0919, 0.1041] | 0.1137 [0.1070, 0.1212] | 1.0041 | 14.5 [14.1, 14.9] | 2.21 | 20.34 [20.18, 20.51] | 3231.4 [3214.2, 3251.4] |
| bge-small-k20 | bk-placed | 0.4806 [0.4712, 0.4899] | 0.9058 [0.8994, 0.9114] | 0.0942 [0.0886, 0.1006] | 0.1097 [0.1029, 0.1172] | 1.0105 | 75.0 [72.0, 76.4] | 3.15 | 20.94 [20.77, 21.09] | 2736.4 [2730.0, 2743.8] |
| bge-small-k20 | jel-placed | 0.4786 [0.4697, 0.4881] | 0.9104 [0.9047, 0.9159] | 0.0896 [0.0841, 0.0953] | 0.1043 [0.0980, 0.1112] | 1.0075 | 28.9 [28.0, 29.7] | 2.27 | 19.41 [19.29, 19.52] | 2875.9 [2866.7, 2887.3] |
| bge-small-k20 | msc-placed | 0.3916 [0.3819, 0.4008] | 0.9491 [0.9446, 0.9538] | 0.0509 [0.0462, 0.0554] | 0.0592 [0.0538, 0.0644] | 1.0383 | 114.4 [107.3, 116.4] | 2.00 | 17.82 [17.71, 17.93] | 3128.5 [3125.8, 3131.9] |
| bge-small-k20 | msc-gold | 0.1322 [0.1254, 0.1385] | 0.9831 [0.9801, 0.9857] | 0.0169 [0.0143, 0.0199] | 0.0197 [0.0166, 0.0232] | 1.0592 | 461.5 [373.5, 406.4] | 1.39 | 15.21 [15.15, 15.27] | 4371.9 [4372.3, 4372.5] |
| bge-small-k20 | jel-gold | 0.0002 [0.0000, 0.0005] | 1.0000 [1.0000, 1.0000] | -0.0000 [-0.0000, -0.0000] | -0.0000 [-0.0000, -0.0000] | 1.0714 | 6.0 [1.0, 6.0] | 1.00 | 14.29 [14.29, 14.29] | 5001.5 [5002.0, 5002.0] |
| bge-small-k50 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.2001 [0.1944, 0.2067] | 0.7999 [0.7933, 0.8056] | 0.9626 [0.9571, 0.9683] | 0.2934 | 30.4 [30.1, 30.6] | 5.00 | 18.67 [18.65, 18.69] | 211.4 [209.0, 214.7] |
| bge-small-k50 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.2119 [0.2056, 0.2188] | 0.7881 [0.7812, 0.7944] | 0.9484 [0.9426, 0.9542] | 0.3042 | 38.2 [37.9, 38.4] | 4.98 | 18.46 [18.44, 18.47] | 181.5 [179.4, 184.7] |
| bge-small-k50 | learned-own | 1.0000 [1.0000, 1.0000] | 0.2208 [0.2141, 0.2278] | 0.7792 [0.7722, 0.7859] | 0.9377 [0.9315, 0.9445] | 0.3130 | 39.7 [39.5, 39.9] | 4.98 | 18.45 [18.43, 18.46] | 156.5 [155.0, 159.3] |
| bge-small-k50 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3537 [0.3451, 0.3629] | 0.6463 [0.6371, 0.6549] | 0.7778 [0.7686, 0.7872] | 0.4453 | 36.5 [35.8, 37.0] | 3.90 | 18.33 [18.29, 18.38] | 291.3 [284.2, 300.2] |
| bge-small-k50 | learned-placed | 0.9993 [0.9987, 0.9997] | 0.3969 [0.3896, 0.4057] | 0.6031 [0.5943, 0.6104] | 0.7258 [0.7163, 0.7345] | 0.5523 | 33.5 [33.1, 33.7] | 4.38 | 31.08 [30.92, 31.24] | 1036.7 [1018.3, 1054.4] |
| bge-small-k50 | openalex-topics-placed | 0.9732 [0.9701, 0.9762] | 0.6599 [0.6536, 0.6665] | 0.3401 [0.3335, 0.3464] | 0.4093 [0.4018, 0.4169] | 0.7942 | 77.6 [74.8, 77.6] | 4.45 | 26.85 [26.77, 26.93] | 311.1 [306.7, 316.8] |
| bge-small-k50 | unesco-thesaurus-placed | 0.9731 [0.9698, 0.9762] | 0.7391 [0.7338, 0.7446] | 0.2609 [0.2554, 0.2662] | 0.3139 [0.3069, 0.3209] | 0.8725 | 146.7 [141.3, 147.7] | 4.90 | 26.68 [26.58, 26.78] | 304.7 [300.6, 309.9] |
| bge-small-k50 | ipc-placed | 0.9799 [0.9772, 0.9826] | 0.7825 [0.7775, 0.7874] | 0.2175 [0.2126, 0.2225] | 0.2618 [0.2555, 0.2686] | 0.9190 | 317.8 [303.2, 318.8] | 6.31 | 27.31 [27.22, 27.42] | 322.1 [314.8, 331.0] |
| bge-small-k50 | ilc2-placed | 0.9999 [0.9997, 1.0000] | 0.8310 [0.8256, 0.8370] | 0.1690 [0.1630, 0.1744] | 0.2034 [0.1960, 0.2098] | 0.9479 | 295.1 [282.7, 295.0] | 5.09 | 23.38 [23.28, 23.48] | 135.6 [133.3, 139.3] |
| bge-small-k50 | lcc-full-placed | 0.9999 [0.9997, 1.0000] | 0.8311 [0.8262, 0.8362] | 0.1689 [0.1638, 0.1738] | 0.2032 [0.1972, 0.2092] | 0.9454 | 657.2 [614.5, 640.4] | 6.06 | 22.86 [22.78, 22.96] | 52.3 [52.0, 53.6] |
| bge-small-k50 | acm-ccs-placed | 0.7445 [0.7358, 0.7528] | 0.8453 [0.8390, 0.8508] | 0.1547 [0.1492, 0.1610] | 0.1862 [0.1792, 0.1940] | 0.9639 | 84.9 [81.5, 86.5] | 3.61 | 23.73 [23.59, 23.88] | 1476.2 [1469.2, 1484.4] |
| bge-small-k50 | fdc-placed | 0.7696 [0.7615, 0.7777] | 0.8454 [0.8390, 0.8513] | 0.1546 [0.1487, 0.1610] | 0.1861 [0.1789, 0.1936] | 0.9732 | 54.5 [53.0, 55.3] | 3.21 | 25.56 [25.40, 25.73] | 1413.5 [1404.0, 1423.7] |
| bge-small-k50 | lcc-placed | 0.8765 [0.8696, 0.8826] | 0.8615 [0.8566, 0.8660] | 0.1385 [0.1340, 0.1434] | 0.1667 [0.1610, 0.1726] | 0.9883 | 129.0 [124.6, 130.5] | 4.08 | 25.36 [25.22, 25.48] | 824.9 [819.6, 830.7] |
| bge-small-k50 | eurovoc-placed | 0.9737 [0.9707, 0.9768] | 0.8637 [0.8593, 0.8681] | 0.1363 [0.1319, 0.1407] | 0.1641 [0.1587, 0.1697] | 0.9877 | 269.3 [259.0, 270.1] | 4.98 | 24.80 [24.71, 24.89] | 268.7 [265.9, 272.2] |
| bge-small-k50 | arxiv-placed | 0.4260 [0.4164, 0.4357] | 0.9063 [0.9003, 0.9115] | 0.0937 [0.0885, 0.0997] | 0.1128 [0.1064, 0.1201] | 1.0080 | 14.5 [14.1, 14.9] | 2.21 | 20.34 [20.18, 20.51] | 3231.4 [3214.2, 3251.4] |
| bge-small-k50 | bk-placed | 0.4806 [0.4712, 0.4899] | 0.9121 [0.9067, 0.9171] | 0.0879 [0.0829, 0.0933] | 0.1058 [0.0997, 0.1124] | 1.0168 | 75.0 [72.0, 76.4] | 3.15 | 20.94 [20.77, 21.09] | 2736.4 [2730.0, 2743.8] |
| bge-small-k50 | jel-placed | 0.4786 [0.4697, 0.4881] | 0.9156 [0.9107, 0.9206] | 0.0844 [0.0794, 0.0893] | 0.1016 [0.0955, 0.1079] | 1.0127 | 28.9 [28.0, 29.7] | 2.27 | 19.41 [19.29, 19.52] | 2875.9 [2866.7, 2887.3] |
| bge-small-k50 | msc-placed | 0.3916 [0.3819, 0.4008] | 0.9571 [0.9535, 0.9608] | 0.0429 [0.0392, 0.0465] | 0.0517 [0.0473, 0.0561] | 1.0462 | 114.4 [107.3, 116.4] | 2.00 | 17.82 [17.71, 17.93] | 3128.5 [3125.8, 3131.9] |
| bge-small-k50 | msc-gold | 0.1322 [0.1254, 0.1385] | 0.9857 [0.9834, 0.9879] | 0.0143 [0.0121, 0.0166] | 0.0172 [0.0145, 0.0200] | 1.0618 | 461.5 [373.5, 406.4] | 1.39 | 15.21 [15.15, 15.27] | 4371.9 [4372.3, 4372.5] |
| bge-small-k50 | jel-gold | 0.0002 [0.0000, 0.0005] | 1.0000 [1.0000, 1.0000] | 0.0000 [-0.0000, -0.0000] | 0.0000 [-0.0000, -0.0000] | 1.0714 | 6.0 [1.0, 6.0] | 1.00 | 14.29 [14.29, 14.29] | 5001.5 [5002.0, 5002.0] |
| nomic-k10 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.1524 [0.1472, 0.1586] | 0.8476 [0.8414, 0.8528] | 0.9792 [0.9738, 0.9844] | 0.2447 | 38.2 [37.9, 38.4] | 4.98 | 18.46 [18.44, 18.47] | 181.5 [179.4, 184.7] |
| nomic-k10 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.1755 [0.1691, 0.1825] | 0.8245 [0.8175, 0.8309] | 0.9524 [0.9459, 0.9592] | 0.2688 | 30.4 [30.1, 30.6] | 5.00 | 18.67 [18.65, 18.69] | 211.4 [209.0, 214.7] |
| nomic-k10 | learned-own | 1.0000 [1.0000, 1.0000] | 0.1835 [0.1770, 0.1906] | 0.8165 [0.8094, 0.8230] | 0.9432 [0.9369, 0.9501] | 0.2758 | 39.7 [39.5, 39.9] | 4.98 | 18.45 [18.43, 18.46] | 156.5 [155.0, 159.3] |
| nomic-k10 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3327 [0.3232, 0.3425] | 0.6673 [0.6575, 0.6768] | 0.7709 [0.7604, 0.7806] | 0.4243 | 36.5 [35.8, 37.0] | 3.90 | 18.33 [18.29, 18.38] | 291.3 [284.2, 300.2] |
| nomic-k10 | learned-placed | 0.9993 [0.9987, 0.9997] | 0.3803 [0.3722, 0.3892] | 0.6197 [0.6108, 0.6278] | 0.7159 [0.7057, 0.7249] | 0.5356 | 33.5 [33.1, 33.7] | 4.38 | 31.08 [30.92, 31.24] | 1036.7 [1018.3, 1054.4] |
| nomic-k10 | openalex-topics-placed | 0.9732 [0.9701, 0.9762] | 0.6313 [0.6229, 0.6394] | 0.3687 [0.3606, 0.3771] | 0.4259 [0.4169, 0.4350] | 0.7656 | 77.6 [74.8, 77.6] | 4.45 | 26.85 [26.77, 26.93] | 311.1 [306.7, 316.8] |
| nomic-k10 | unesco-thesaurus-placed | 0.9731 [0.9698, 0.9762] | 0.7092 [0.7013, 0.7170] | 0.2908 [0.2830, 0.2987] | 0.3359 [0.3270, 0.3455] | 0.8426 | 146.7 [141.3, 147.7] | 4.90 | 26.68 [26.58, 26.78] | 304.7 [300.6, 309.9] |
| nomic-k10 | ipc-placed | 0.9799 [0.9772, 0.9826] | 0.7631 [0.7567, 0.7698] | 0.2369 [0.2302, 0.2433] | 0.2737 [0.2656, 0.2816] | 0.8996 | 317.8 [303.2, 318.8] | 6.31 | 27.31 [27.22, 27.42] | 322.1 [314.8, 331.0] |
| nomic-k10 | lcc-full-placed | 0.9999 [0.9997, 1.0000] | 0.8126 [0.8058, 0.8193] | 0.1874 [0.1807, 0.1942] | 0.2165 [0.2086, 0.2245] | 0.9269 | 657.2 [614.5, 640.4] | 6.06 | 22.86 [22.78, 22.96] | 52.3 [52.0, 53.6] |
| nomic-k10 | ilc2-placed | 0.9999 [0.9997, 1.0000] | 0.8159 [0.8082, 0.8238] | 0.1841 [0.1762, 0.1918] | 0.2126 [0.2037, 0.2218] | 0.9329 | 295.1 [282.7, 295.0] | 5.09 | 23.38 [23.28, 23.48] | 135.6 [133.3, 139.3] |
| nomic-k10 | acm-ccs-placed | 0.7445 [0.7358, 0.7528] | 0.8278 [0.8195, 0.8351] | 0.1722 [0.1649, 0.1805] | 0.1989 [0.1903, 0.2085] | 0.9465 | 84.9 [81.5, 86.5] | 3.61 | 23.73 [23.59, 23.88] | 1476.2 [1469.2, 1484.4] |
| nomic-k10 | fdc-placed | 0.7696 [0.7615, 0.7777] | 0.8279 [0.8194, 0.8357] | 0.1721 [0.1643, 0.1806] | 0.1989 [0.1903, 0.2085] | 0.9557 | 54.5 [53.0, 55.3] | 3.21 | 25.56 [25.40, 25.73] | 1413.5 [1404.0, 1423.7] |
| nomic-k10 | eurovoc-placed | 0.9737 [0.9707, 0.9768] | 0.8318 [0.8247, 0.8398] | 0.1682 [0.1602, 0.1753] | 0.1943 [0.1854, 0.2028] | 0.9558 | 269.3 [259.0, 270.1] | 4.98 | 24.80 [24.71, 24.89] | 268.7 [265.9, 272.2] |
| nomic-k10 | lcc-placed | 0.8765 [0.8696, 0.8826] | 0.8386 [0.8314, 0.8453] | 0.1614 [0.1547, 0.1686] | 0.1865 [0.1786, 0.1947] | 0.9653 | 129.0 [124.6, 130.5] | 4.08 | 25.36 [25.22, 25.48] | 824.9 [819.6, 830.7] |
| nomic-k10 | arxiv-placed | 0.4260 [0.4164, 0.4357] | 0.8950 [0.8873, 0.9017] | 0.1050 [0.0983, 0.1127] | 0.1213 [0.1135, 0.1305] | 0.9967 | 14.5 [14.1, 14.9] | 2.21 | 20.34 [20.18, 20.51] | 3231.4 [3214.2, 3251.4] |
| nomic-k10 | bk-placed | 0.4806 [0.4712, 0.4899] | 0.8982 [0.8902, 0.9058] | 0.1018 [0.0942, 0.1098] | 0.1176 [0.1089, 0.1271] | 1.0029 | 75.0 [72.0, 76.4] | 3.15 | 20.94 [20.77, 21.09] | 2736.4 [2730.0, 2743.8] |
| nomic-k10 | jel-placed | 0.4786 [0.4697, 0.4881] | 0.9012 [0.8941, 0.9079] | 0.0988 [0.0921, 0.1059] | 0.1141 [0.1064, 0.1225] | 0.9983 | 28.9 [28.0, 29.7] | 2.27 | 19.41 [19.29, 19.52] | 2875.9 [2866.7, 2887.3] |
| nomic-k10 | msc-placed | 0.3916 [0.3819, 0.4008] | 0.9429 [0.9369, 0.9485] | 0.0571 [0.0515, 0.0631] | 0.0660 [0.0595, 0.0730] | 1.0320 | 114.4 [107.3, 116.4] | 2.00 | 17.82 [17.71, 17.93] | 3128.5 [3125.8, 3131.9] |
| nomic-k10 | msc-gold | 0.1322 [0.1254, 0.1385] | 0.9847 [0.9816, 0.9872] | 0.0153 [0.0128, 0.0184] | 0.0177 [0.0147, 0.0213] | 1.0607 | 461.5 [373.5, 406.4] | 1.39 | 15.21 [15.15, 15.27] | 4371.9 [4372.3, 4372.5] |
| nomic-k10 | jel-gold | 0.0002 [0.0000, 0.0005] | 1.0000 [1.0000, 1.0000] | -0.0000 [-0.0000, -0.0000] | -0.0000 [-0.0000, -0.0000] | 1.0714 | 6.0 [1.0, 6.0] | 1.00 | 14.29 [14.29, 14.29] | 5001.5 [5002.0, 5002.0] |
| nomic-k20 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.1761 [0.1704, 0.1826] | 0.8239 [0.8174, 0.8296] | 0.9754 [0.9746, 0.9763] | 0.2684 | 38.2 [37.9, 38.4] | 4.98 | 18.46 [18.44, 18.47] | 181.5 [179.4, 184.7] |
| nomic-k20 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.1992 [0.1929, 0.2059] | 0.8008 [0.7941, 0.8071] | 0.9481 [0.9417, 0.9548] | 0.2925 | 30.4 [30.1, 30.6] | 5.00 | 18.67 [18.65, 18.69] | 211.4 [209.0, 214.7] |
| nomic-k20 | learned-own | 1.0000 [1.0000, 1.0000] | 0.2033 [0.1967, 0.2102] | 0.7967 [0.7898, 0.8033] | 0.9432 [0.9370, 0.9496] | 0.2956 | 39.7 [39.5, 39.9] | 4.98 | 18.45 [18.43, 18.46] | 156.5 [155.0, 159.3] |
| nomic-k20 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3465 [0.3377, 0.3561] | 0.6535 [0.6439, 0.6623] | 0.7737 [0.7642, 0.7834] | 0.4382 | 36.5 [35.8, 37.0] | 3.90 | 18.33 [18.29, 18.38] | 291.3 [284.2, 300.2] |
| nomic-k20 | learned-placed | 0.9993 [0.9987, 0.9997] | 0.3873 [0.3793, 0.3960] | 0.6127 [0.6040, 0.6207] | 0.7254 [0.7156, 0.7344] | 0.5427 | 33.5 [33.1, 33.7] | 4.38 | 31.08 [30.92, 31.24] | 1036.7 [1018.3, 1054.4] |
| nomic-k20 | openalex-topics-placed | 0.9732 [0.9701, 0.9762] | 0.6434 [0.6362, 0.6509] | 0.3566 [0.3491, 0.3638] | 0.4222 [0.4140, 0.4305] | 0.7777 | 77.6 [74.8, 77.6] | 4.45 | 26.85 [26.77, 26.93] | 311.1 [306.7, 316.8] |
| nomic-k20 | unesco-thesaurus-placed | 0.9731 [0.9698, 0.9762] | 0.7190 [0.7122, 0.7256] | 0.2810 [0.2744, 0.2878] | 0.3327 [0.3246, 0.3410] | 0.8524 | 146.7 [141.3, 147.7] | 4.90 | 26.68 [26.58, 26.78] | 304.7 [300.6, 309.9] |
| nomic-k20 | ipc-placed | 0.9799 [0.9772, 0.9826] | 0.7695 [0.7634, 0.7753] | 0.2305 [0.2247, 0.2366] | 0.2729 [0.2659, 0.2805] | 0.9060 | 317.8 [303.2, 318.8] | 6.31 | 27.31 [27.22, 27.42] | 322.1 [314.8, 331.0] |
| nomic-k20 | lcc-full-placed | 0.9999 [0.9997, 1.0000] | 0.8218 [0.8163, 0.8273] | 0.1782 [0.1727, 0.1837] | 0.2110 [0.2043, 0.2178] | 0.9361 | 657.2 [614.5, 640.4] | 6.06 | 22.86 [22.78, 22.96] | 52.3 [52.0, 53.6] |
| nomic-k20 | ilc2-placed | 0.9999 [0.9997, 1.0000] | 0.8245 [0.8183, 0.8312] | 0.1755 [0.1688, 0.1817] | 0.2078 [0.1998, 0.2157] | 0.9414 | 295.1 [282.7, 295.0] | 5.09 | 23.38 [23.28, 23.48] | 135.6 [133.3, 139.3] |
| nomic-k20 | fdc-placed | 0.7696 [0.7615, 0.7777] | 0.8351 [0.8281, 0.8417] | 0.1649 [0.1583, 0.1719] | 0.1952 [0.1877, 0.2040] | 0.9629 | 54.5 [53.0, 55.3] | 3.21 | 25.56 [25.40, 25.73] | 1413.5 [1404.0, 1423.7] |
| nomic-k20 | acm-ccs-placed | 0.7445 [0.7358, 0.7528] | 0.8354 [0.8279, 0.8417] | 0.1646 [0.1583, 0.1721] | 0.1949 [0.1876, 0.2037] | 0.9541 | 84.9 [81.5, 86.5] | 3.61 | 23.73 [23.59, 23.88] | 1476.2 [1469.2, 1484.4] |
| nomic-k20 | eurovoc-placed | 0.9737 [0.9707, 0.9768] | 0.8446 [0.8387, 0.8507] | 0.1554 [0.1493, 0.1613] | 0.1840 [0.1771, 0.1911] | 0.9686 | 269.3 [259.0, 270.1] | 4.98 | 24.80 [24.71, 24.89] | 268.7 [265.9, 272.2] |
| nomic-k20 | lcc-placed | 0.8765 [0.8696, 0.8826] | 0.8491 [0.8432, 0.8546] | 0.1509 [0.1454, 0.1568] | 0.1787 [0.1720, 0.1859] | 0.9758 | 129.0 [124.6, 130.5] | 4.08 | 25.36 [25.22, 25.48] | 824.9 [819.6, 830.7] |
| nomic-k20 | arxiv-placed | 0.4260 [0.4164, 0.4357] | 0.8982 [0.8913, 0.9043] | 0.1018 [0.0957, 0.1087] | 0.1206 [0.1131, 0.1289] | 0.9999 | 14.5 [14.1, 14.9] | 2.21 | 20.34 [20.18, 20.51] | 3231.4 [3214.2, 3251.4] |
| nomic-k20 | bk-placed | 0.4806 [0.4712, 0.4899] | 0.9028 [0.8955, 0.9095] | 0.0972 [0.0905, 0.1045] | 0.1150 [0.1071, 0.1235] | 1.0075 | 75.0 [72.0, 76.4] | 3.15 | 20.94 [20.77, 21.09] | 2736.4 [2730.0, 2743.8] |
| nomic-k20 | jel-placed | 0.4786 [0.4697, 0.4881] | 0.9055 [0.8994, 0.9117] | 0.0945 [0.0883, 0.1006] | 0.1119 [0.1047, 0.1192] | 1.0026 | 28.9 [28.0, 29.7] | 2.27 | 19.41 [19.29, 19.52] | 2875.9 [2866.7, 2887.3] |
| nomic-k20 | msc-placed | 0.3916 [0.3819, 0.4008] | 0.9489 [0.9438, 0.9538] | 0.0511 [0.0462, 0.0562] | 0.0605 [0.0547, 0.0667] | 1.0380 | 114.4 [107.3, 116.4] | 2.00 | 17.82 [17.71, 17.93] | 3128.5 [3125.8, 3131.9] |
| nomic-k20 | msc-gold | 0.1322 [0.1254, 0.1385] | 0.9862 [0.9837, 0.9884] | 0.0138 [0.0116, 0.0163] | 0.0163 [0.0138, 0.0192] | 1.0623 | 461.5 [373.5, 406.4] | 1.39 | 15.21 [15.15, 15.27] | 4371.9 [4372.3, 4372.5] |
| nomic-k20 | jel-gold | 0.0002 [0.0000, 0.0005] | 1.0000 [1.0000, 1.0000] | -0.0000 [-0.0000, -0.0000] | -0.0000 [-0.0000, -0.0000] | 1.0714 | 6.0 [1.0, 6.0] | 1.00 | 14.29 [14.29, 14.29] | 5001.5 [5002.0, 5002.0] |
| nomic-k50 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.2132 [0.2074, 0.2198] | 0.7868 [0.7802, 0.7926] | 0.9781 [0.9717, 0.9839] | 0.3055 | 38.2 [37.9, 38.4] | 4.98 | 18.46 [18.44, 18.47] | 181.5 [179.4, 184.7] |
| nomic-k50 | learned-own | 1.0000 [1.0000, 1.0000] | 0.2330 [0.2264, 0.2401] | 0.7670 [0.7599, 0.7736] | 0.9535 [0.9468, 0.9604] | 0.3252 | 39.7 [39.5, 39.9] | 4.98 | 18.45 [18.43, 18.46] | 156.5 [155.0, 159.3] |
| nomic-k50 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.2333 [0.2271, 0.2404] | 0.7667 [0.7596, 0.7729] | 0.9531 [0.9467, 0.9592] | 0.3266 | 30.4 [30.1, 30.6] | 5.00 | 18.67 [18.65, 18.69] | 211.4 [209.0, 214.7] |
| nomic-k50 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3683 [0.3599, 0.3770] | 0.6317 [0.6230, 0.6401] | 0.7853 [0.7757, 0.7941] | 0.4600 | 36.5 [35.8, 37.0] | 3.90 | 18.33 [18.29, 18.38] | 291.3 [284.2, 300.2] |
| nomic-k50 | learned-placed | 0.9993 [0.9987, 0.9997] | 0.3997 [0.3923, 0.4078] | 0.6003 [0.5922, 0.6077] | 0.7463 [0.7363, 0.7551] | 0.5550 | 33.5 [33.1, 33.7] | 4.38 | 31.08 [30.92, 31.24] | 1036.7 [1018.3, 1054.4] |
| nomic-k50 | openalex-topics-placed | 0.9732 [0.9701, 0.9762] | 0.6577 [0.6518, 0.6641] | 0.3423 [0.3359, 0.3482] | 0.4255 [0.4180, 0.4335] | 0.7920 | 77.6 [74.8, 77.6] | 4.45 | 26.85 [26.77, 26.93] | 311.1 [306.7, 316.8] |
| nomic-k50 | unesco-thesaurus-placed | 0.9731 [0.9698, 0.9762] | 0.7360 [0.7307, 0.7415] | 0.2640 [0.2585, 0.2693] | 0.3283 [0.3215, 0.3355] | 0.8694 | 146.7 [141.3, 147.7] | 4.90 | 26.68 [26.58, 26.78] | 304.7 [300.6, 309.9] |
| nomic-k50 | ipc-placed | 0.9799 [0.9772, 0.9826] | 0.7784 [0.7730, 0.7835] | 0.2216 [0.2165, 0.2270] | 0.2755 [0.2691, 0.2824] | 0.9149 | 317.8 [303.2, 318.8] | 6.31 | 27.31 [27.22, 27.42] | 322.1 [314.8, 331.0] |
| nomic-k50 | lcc-full-placed | 0.9999 [0.9997, 1.0000] | 0.8321 [0.8271, 0.8370] | 0.1679 [0.1630, 0.1729] | 0.2088 [0.2025, 0.2152] | 0.9464 | 657.2 [614.5, 640.4] | 6.06 | 22.86 [22.78, 22.96] | 52.3 [52.0, 53.6] |
| nomic-k50 | ilc2-placed | 0.9999 [0.9997, 1.0000] | 0.8350 [0.8295, 0.8408] | 0.1650 [0.1592, 0.1705] | 0.2051 [0.1980, 0.2124] | 0.9519 | 295.1 [282.7, 295.0] | 5.09 | 23.38 [23.28, 23.48] | 135.6 [133.3, 139.3] |
| nomic-k50 | acm-ccs-placed | 0.7445 [0.7358, 0.7528] | 0.8443 [0.8377, 0.8498] | 0.1557 [0.1502, 0.1623] | 0.1935 [0.1866, 0.2013] | 0.9630 | 84.9 [81.5, 86.5] | 3.61 | 23.73 [23.59, 23.88] | 1476.2 [1469.2, 1484.4] |
| nomic-k50 | fdc-placed | 0.7696 [0.7615, 0.7777] | 0.8459 [0.8399, 0.8518] | 0.1541 [0.1482, 0.1601] | 0.1916 [0.1847, 0.1993] | 0.9737 | 54.5 [53.0, 55.3] | 3.21 | 25.56 [25.40, 25.73] | 1413.5 [1404.0, 1423.7] |
| nomic-k50 | eurovoc-placed | 0.9737 [0.9707, 0.9768] | 0.8601 [0.8553, 0.8652] | 0.1399 [0.1348, 0.1447] | 0.1739 [0.1679, 0.1800] | 0.9841 | 269.3 [259.0, 270.1] | 4.98 | 24.80 [24.71, 24.89] | 268.7 [265.9, 272.2] |
| nomic-k50 | lcc-placed | 0.8765 [0.8696, 0.8826] | 0.8614 [0.8564, 0.8661] | 0.1386 [0.1339, 0.1436] | 0.1723 [0.1665, 0.1784] | 0.9882 | 129.0 [124.6, 130.5] | 4.08 | 25.36 [25.22, 25.48] | 824.9 [819.6, 830.7] |
| nomic-k50 | arxiv-placed | 0.4260 [0.4164, 0.4357] | 0.9021 [0.8959, 0.9079] | 0.0979 [0.0921, 0.1041] | 0.1217 [0.1144, 0.1296] | 1.0038 | 14.5 [14.1, 14.9] | 2.21 | 20.34 [20.18, 20.51] | 3231.4 [3214.2, 3251.4] |
| nomic-k50 | bk-placed | 0.4806 [0.4712, 0.4899] | 0.9099 [0.9042, 0.9153] | 0.0901 [0.0847, 0.0958] | 0.1120 [0.1053, 0.1188] | 1.0146 | 75.0 [72.0, 76.4] | 3.15 | 20.94 [20.77, 21.09] | 2736.4 [2730.0, 2743.8] |
| nomic-k50 | jel-placed | 0.4786 [0.4697, 0.4881] | 0.9105 [0.9052, 0.9161] | 0.0895 [0.0839, 0.0948] | 0.1113 [0.1044, 0.1182] | 1.0075 | 28.9 [28.0, 29.7] | 2.27 | 19.41 [19.29, 19.52] | 2875.9 [2866.7, 2887.3] |
| nomic-k50 | msc-placed | 0.3916 [0.3819, 0.4008] | 0.9566 [0.9525, 0.9605] | 0.0434 [0.0395, 0.0475] | 0.0539 [0.0492, 0.0591] | 1.0457 | 114.4 [107.3, 116.4] | 2.00 | 17.82 [17.71, 17.93] | 3128.5 [3125.8, 3131.9] |
| nomic-k50 | msc-gold | 0.1322 [0.1254, 0.1385] | 0.9888 [0.9869, 0.9905] | 0.0112 [0.0095, 0.0131] | 0.0139 [0.0119, 0.0163] | 1.0649 | 461.5 [373.5, 406.4] | 1.39 | 15.21 [15.15, 15.27] | 4371.9 [4372.3, 4372.5] |
| nomic-k50 | jel-gold | 0.0002 [0.0000, 0.0005] | 1.0000 [1.0000, 1.0000] | 0.0000 [-0.0000, -0.0000] | 0.0000 [-0.0000, -0.0000] | 1.0714 | 6.0 [1.0, 6.0] | 1.00 | 14.29 [14.29, 14.29] | 5001.5 [5002.0, 5002.0] |

### holdout

| condition | row | coverage [95% CI] | cost_norm [95% CI] | MW revenue_norm [95% CI] | skill [95% CI] | J | eff. classes [95% CI] | mean depth | nav log a=1 [95% CI] | nav scan [95% CI] |
| --- | --- | --- | --- | --- | --- | ---: | --- | ---: | --- | --- |
| minilm-k10 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.4267 [0.3915, 0.4673] | 0.5733 [0.5327, 0.6085] | 0.7717 [0.7263, 0.8114] | 0.5241 | 15.4 [14.9, 15.8] | 9.11 | 19.49 [19.26, 19.68] | 65.7 [63.9, 68.4] |
| minilm-k10 | learned-own | 1.0000 [1.0000, 1.0000] | 0.4355 [0.4024, 0.4790] | 0.5645 [0.5210, 0.5976] | 0.7599 [0.7124, 0.8040] | 0.5339 | 15.1 [14.3, 15.6] | 8.79 | 19.69 [19.40, 19.96] | 73.1 [70.6, 76.6] |
| minilm-k10 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.5062 [0.4738, 0.5418] | 0.4938 [0.4582, 0.5262] | 0.6648 [0.6245, 0.7040] | 0.6152 | 13.5 [12.8, 14.1] | 7.93 | 21.81 [21.44, 22.16] | 89.7 [85.9, 94.3] |
| minilm-k10 | lcc-full-gold | 0.9220 [0.8960, 0.9440] | 0.7010 [0.6737, 0.7282] | 0.2990 [0.2718, 0.3263] | 0.4026 [0.3653, 0.4449] | 0.7995 | 85.2 [65.8, 84.8] | 5.19 | 19.71 [19.17, 20.24] | 55.6 [53.0, 59.7] |
| minilm-k10 | lcc-gold | 0.9220 [0.8960, 0.9440] | 0.7030 [0.6757, 0.7300] | 0.2970 [0.2700, 0.3243] | 0.3998 [0.3624, 0.4420] | 0.7959 | 35.0 [28.4, 37.3] | 4.20 | 18.59 [18.13, 19.04] | 69.4 [65.5, 74.7] |
| minilm-k10 | lcc-full-placed | 0.5660 [0.5220, 0.6080] | 0.9137 [0.8850, 0.9379] | 0.0863 [0.0621, 0.1150] | 0.1162 [0.0842, 0.1555] | 0.9836 | 340.8 [243.6, 291.0] | 4.14 | 13.98 [13.52, 14.42] | 124.9 [124.9, 125.9] |
| minilm-k10 | ilc2-placed | 0.4560 [0.4120, 0.4980] | 0.9423 [0.9190, 0.9624] | 0.0577 [0.0376, 0.0810] | 0.0776 [0.0518, 0.1093] | 1.0047 | 188.1 [134.7, 173.2] | 2.94 | 12.46 [12.10, 12.81] | 155.5 [155.2, 156.8] |
| minilm-k10 | unesco-thesaurus-placed | 0.1580 [0.1280, 0.1920] | 0.9876 [0.9784, 0.9945] | 0.0124 [0.0055, 0.0216] | 0.0167 [0.0074, 0.0299] | 1.0417 | 93.8 [63.5, 81.8] | 1.65 | 10.82 [10.60, 11.06] | 216.0 [216.3, 216.8] |
| minilm-k10 | learned-placed | 0.0860 [0.0620, 0.1120] | 0.9890 [0.9764, 0.9962] | 0.0110 [0.0038, 0.0236] | 0.0149 [0.0051, 0.0325] | 1.0418 | 26.1 [19.5, 25.7] | 1.34 | 10.57 [10.37, 10.79] | 231.6 [231.9, 232.4] |
| minilm-k10 | lcc-placed | 0.0840 [0.0620, 0.1080] | 0.9947 [0.9881, 0.9988] | 0.0053 [0.0012, 0.0119] | 0.0071 [0.0016, 0.0161] | 1.0458 | 63.0 [36.3, 56.7] | 1.29 | 10.21 [10.11, 10.31] | 236.8 [237.2, 237.4] |
| minilm-k10 | fdc-placed | 0.0520 [0.0340, 0.0720] | 0.9972 [0.9925, 0.9998] | 0.0028 [0.0002, 0.0075] | 0.0038 [0.0003, 0.0105] | 1.0477 | 31.1 [16.1, 29.1] | 1.14 | 10.10 [10.03, 10.17] | 242.7 [243.1, 243.3] |
| minilm-k10 | bk-placed | 0.0440 [0.0260, 0.0640] | 0.9985 [0.9951, 1.0000] | 0.0015 [0.0000, 0.0049] | 0.0020 [0.0000, 0.0067] | 1.0492 | 30.8 [17.5, 32.6] | 1.23 | 10.14 [10.05, 10.25] | 242.3 [242.6, 242.9] |
| minilm-k20 | learned-own | 1.0000 [1.0000, 1.0000] | 0.4432 [0.4133, 0.4796] | 0.5568 [0.5204, 0.5867] | 0.7971 [0.7530, 0.8361] | 0.5416 | 15.1 [14.3, 15.6] | 8.79 | 19.69 [19.40, 19.96] | 73.1 [70.6, 76.6] |
| minilm-k20 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.4442 [0.4122, 0.4803] | 0.5558 [0.5197, 0.5878] | 0.7957 [0.7592, 0.8271] | 0.5416 | 15.4 [14.9, 15.8] | 9.11 | 19.49 [19.26, 19.68] | 65.7 [63.9, 68.4] |
| minilm-k20 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.5169 [0.4885, 0.5502] | 0.4831 [0.4498, 0.5115] | 0.6916 [0.6495, 0.7312] | 0.6259 | 13.5 [12.8, 14.1] | 7.93 | 21.81 [21.44, 22.16] | 89.7 [85.9, 94.3] |
| minilm-k20 | lcc-full-gold | 0.9220 [0.8960, 0.9440] | 0.7057 [0.6825, 0.7286] | 0.2943 [0.2714, 0.3175] | 0.4212 [0.3845, 0.4602] | 0.8043 | 85.2 [65.8, 84.8] | 5.19 | 19.71 [19.17, 20.24] | 55.6 [53.0, 59.7] |
| minilm-k20 | lcc-gold | 0.9220 [0.8960, 0.9440] | 0.7076 [0.6847, 0.7303] | 0.2924 [0.2697, 0.3153] | 0.4185 [0.3822, 0.4569] | 0.8006 | 35.0 [28.4, 37.3] | 4.20 | 18.59 [18.13, 19.04] | 69.4 [65.5, 74.7] |
| minilm-k20 | lcc-full-placed | 0.5660 [0.5220, 0.6080] | 0.9250 [0.9037, 0.9446] | 0.0750 [0.0554, 0.0963] | 0.1074 [0.0793, 0.1378] | 0.9949 | 340.8 [243.6, 291.0] | 4.14 | 13.98 [13.52, 14.42] | 124.9 [124.9, 125.9] |
| minilm-k20 | ilc2-placed | 0.4560 [0.4120, 0.4980] | 0.9543 [0.9375, 0.9693] | 0.0457 [0.0307, 0.0625] | 0.0655 [0.0445, 0.0891] | 1.0166 | 188.1 [134.7, 173.2] | 2.94 | 12.46 [12.10, 12.81] | 155.5 [155.2, 156.8] |
| minilm-k20 | unesco-thesaurus-placed | 0.1580 [0.1280, 0.1920] | 0.9888 [0.9817, 0.9943] | 0.0112 [0.0057, 0.0183] | 0.0161 [0.0082, 0.0262] | 1.0429 | 93.8 [63.5, 81.8] | 1.65 | 10.82 [10.60, 11.06] | 216.0 [216.3, 216.8] |
| minilm-k20 | learned-placed | 0.0860 [0.0620, 0.1120] | 0.9915 [0.9832, 0.9967] | 0.0085 [0.0033, 0.0168] | 0.0122 [0.0047, 0.0242] | 1.0443 | 26.1 [19.5, 25.7] | 1.34 | 10.57 [10.37, 10.79] | 231.6 [231.9, 232.4] |
| minilm-k20 | lcc-placed | 0.0840 [0.0620, 0.1080] | 0.9964 [0.9920, 0.9991] | 0.0036 [0.0009, 0.0080] | 0.0052 [0.0013, 0.0117] | 1.0474 | 63.0 [36.3, 56.7] | 1.29 | 10.21 [10.11, 10.31] | 236.8 [237.2, 237.4] |
| minilm-k20 | fdc-placed | 0.0520 [0.0340, 0.0720] | 0.9981 [0.9949, 0.9998] | 0.0019 [0.0002, 0.0051] | 0.0027 [0.0003, 0.0074] | 1.0486 | 31.1 [16.1, 29.1] | 1.14 | 10.10 [10.03, 10.17] | 242.7 [243.1, 243.3] |
| minilm-k20 | bk-placed | 0.0440 [0.0260, 0.0640] | 0.9985 [0.9962, 0.9998] | 0.0015 [0.0002, 0.0038] | 0.0021 [0.0002, 0.0054] | 1.0492 | 30.8 [17.5, 32.6] | 1.23 | 10.14 [10.05, 10.25] | 242.3 [242.6, 242.9] |
| minilm-k50 | learned-own | 1.0000 [1.0000, 1.0000] | 0.4699 [0.4442, 0.5025] | 0.5301 [0.4975, 0.5558] | 0.8223 [0.7802, 0.8583] | 0.5684 | 15.1 [14.3, 15.6] | 8.79 | 19.69 [19.40, 19.96] | 73.1 [70.6, 76.6] |
| minilm-k50 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.4792 [0.4514, 0.5099] | 0.5208 [0.4901, 0.5486] | 0.8079 [0.7733, 0.8419] | 0.5767 | 15.4 [14.9, 15.8] | 9.11 | 19.49 [19.26, 19.68] | 65.7 [63.9, 68.4] |
| minilm-k50 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.5349 [0.5093, 0.5634] | 0.4651 [0.4366, 0.4907] | 0.7216 [0.6821, 0.7618] | 0.6439 | 13.5 [12.8, 14.1] | 7.93 | 21.81 [21.44, 22.16] | 89.7 [85.9, 94.3] |
| minilm-k50 | lcc-full-gold | 0.9220 [0.8960, 0.9440] | 0.7134 [0.6940, 0.7333] | 0.2866 [0.2667, 0.3060] | 0.4447 [0.4101, 0.4824] | 0.8119 | 85.2 [65.8, 84.8] | 5.19 | 19.71 [19.17, 20.24] | 55.6 [53.0, 59.7] |
| minilm-k50 | lcc-gold | 0.9220 [0.8960, 0.9440] | 0.7150 [0.6953, 0.7348] | 0.2850 [0.2652, 0.3047] | 0.4422 [0.4084, 0.4795] | 0.8079 | 35.0 [28.4, 37.3] | 4.20 | 18.59 [18.13, 19.04] | 69.4 [65.5, 74.7] |
| minilm-k50 | lcc-full-placed | 0.5660 [0.5220, 0.6080] | 0.9347 [0.9186, 0.9513] | 0.0653 [0.0487, 0.0814] | 0.1013 [0.0762, 0.1277] | 1.0046 | 340.8 [243.6, 291.0] | 4.14 | 13.98 [13.52, 14.42] | 124.9 [124.9, 125.9] |
| minilm-k50 | ilc2-placed | 0.4560 [0.4120, 0.4980] | 0.9678 [0.9575, 0.9778] | 0.0322 [0.0222, 0.0425] | 0.0500 [0.0352, 0.0659] | 1.0301 | 188.1 [134.7, 173.2] | 2.94 | 12.46 [12.10, 12.81] | 155.5 [155.2, 156.8] |
| minilm-k50 | unesco-thesaurus-placed | 0.1580 [0.1280, 0.1920] | 0.9917 [0.9870, 0.9955] | 0.0083 [0.0045, 0.0130] | 0.0128 [0.0069, 0.0204] | 1.0459 | 93.8 [63.5, 81.8] | 1.65 | 10.82 [10.60, 11.06] | 216.0 [216.3, 216.8] |
| minilm-k50 | learned-placed | 0.0860 [0.0620, 0.1120] | 0.9936 [0.9877, 0.9973] | 0.0064 [0.0027, 0.0123] | 0.0100 [0.0043, 0.0191] | 1.0464 | 26.1 [19.5, 25.7] | 1.34 | 10.57 [10.37, 10.79] | 231.6 [231.9, 232.4] |
| minilm-k50 | lcc-placed | 0.0840 [0.0620, 0.1080] | 0.9979 [0.9955, 0.9994] | 0.0021 [0.0006, 0.0045] | 0.0033 [0.0009, 0.0072] | 1.0489 | 63.0 [36.3, 56.7] | 1.29 | 10.21 [10.11, 10.31] | 236.8 [237.2, 237.4] |
| minilm-k50 | fdc-placed | 0.0520 [0.0340, 0.0720] | 0.9988 [0.9969, 0.9998] | 0.0012 [0.0002, 0.0031] | 0.0019 [0.0003, 0.0048] | 1.0493 | 31.1 [16.1, 29.1] | 1.14 | 10.10 [10.03, 10.17] | 242.7 [243.1, 243.3] |
| minilm-k50 | bk-placed | 0.0440 [0.0260, 0.0640] | 0.9990 [0.9975, 0.9999] | 0.0010 [0.0001, 0.0025] | 0.0015 [0.0002, 0.0039] | 1.0497 | 30.8 [17.5, 32.6] | 1.23 | 10.14 [10.05, 10.25] | 242.3 [242.6, 242.9] |
| bge-small-k10 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.4406 [0.4045, 0.4839] | 0.5594 [0.5161, 0.5955] | 0.7784 [0.7252, 0.8254] | 0.5380 | 15.4 [14.9, 15.8] | 9.11 | 19.49 [19.26, 19.68] | 65.7 [63.9, 68.4] |
| bge-small-k10 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.4710 [0.4407, 0.5035] | 0.5290 [0.4965, 0.5593] | 0.7361 [0.6866, 0.7849] | 0.5801 | 13.5 [12.8, 14.1] | 7.93 | 21.81 [21.44, 22.16] | 89.7 [85.9, 94.3] |
| bge-small-k10 | learned-own | 1.0000 [1.0000, 1.0000] | 0.4880 [0.4571, 0.5225] | 0.5120 [0.4775, 0.5429] | 0.7124 [0.6693, 0.7561] | 0.5865 | 15.1 [14.3, 15.6] | 8.79 | 19.69 [19.40, 19.96] | 73.1 [70.6, 76.6] |
| bge-small-k10 | lcc-full-gold | 0.9220 [0.8960, 0.9440] | 0.7014 [0.6692, 0.7304] | 0.2986 [0.2696, 0.3308] | 0.4155 [0.3745, 0.4607] | 0.8000 | 85.2 [65.8, 84.8] | 5.19 | 19.71 [19.17, 20.24] | 55.6 [53.0, 59.7] |
| bge-small-k10 | lcc-gold | 0.9220 [0.8960, 0.9440] | 0.7036 [0.6719, 0.7323] | 0.2964 [0.2677, 0.3281] | 0.4124 [0.3720, 0.4583] | 0.7965 | 35.0 [28.4, 37.3] | 4.20 | 18.59 [18.13, 19.04] | 69.4 [65.5, 74.7] |
| bge-small-k10 | lcc-full-placed | 0.5660 [0.5220, 0.6080] | 0.9211 [0.8917, 0.9441] | 0.0789 [0.0559, 0.1083] | 0.1098 [0.0785, 0.1497] | 0.9910 | 340.8 [243.6, 291.0] | 4.14 | 13.98 [13.52, 14.42] | 124.9 [124.9, 125.9] |
| bge-small-k10 | ilc2-placed | 0.4560 [0.4120, 0.4980] | 0.9540 [0.9327, 0.9712] | 0.0460 [0.0288, 0.0673] | 0.0640 [0.0411, 0.0921] | 1.0163 | 188.1 [134.7, 173.2] | 2.94 | 12.46 [12.10, 12.81] | 155.5 [155.2, 156.8] |
| bge-small-k10 | learned-placed | 0.0860 [0.0620, 0.1120] | 0.9871 [0.9732, 0.9955] | 0.0129 [0.0045, 0.0268] | 0.0179 [0.0062, 0.0373] | 1.0399 | 26.1 [19.5, 25.7] | 1.34 | 10.57 [10.37, 10.79] | 231.6 [231.9, 232.4] |
| bge-small-k10 | unesco-thesaurus-placed | 0.1580 [0.1280, 0.1920] | 0.9872 [0.9781, 0.9941] | 0.0128 [0.0059, 0.0219] | 0.0179 [0.0083, 0.0310] | 1.0413 | 93.8 [63.5, 81.8] | 1.65 | 10.82 [10.60, 11.06] | 216.0 [216.3, 216.8] |
| bge-small-k10 | lcc-placed | 0.0840 [0.0620, 0.1080] | 0.9958 [0.9893, 0.9992] | 0.0042 [0.0008, 0.0107] | 0.0059 [0.0012, 0.0149] | 1.0468 | 63.0 [36.3, 56.7] | 1.29 | 10.21 [10.11, 10.31] | 236.8 [237.2, 237.4] |
| bge-small-k10 | bk-placed | 0.0440 [0.0260, 0.0640] | 0.9959 [0.9898, 0.9995] | 0.0041 [0.0005, 0.0102] | 0.0057 [0.0007, 0.0144] | 1.0467 | 30.8 [17.5, 32.6] | 1.23 | 10.14 [10.05, 10.25] | 242.3 [242.6, 242.9] |
| bge-small-k10 | fdc-placed | 0.0520 [0.0340, 0.0720] | 0.9973 [0.9925, 0.9999] | 0.0027 [0.0001, 0.0075] | 0.0037 [0.0002, 0.0103] | 1.0478 | 31.1 [16.1, 29.1] | 1.14 | 10.10 [10.03, 10.17] | 242.7 [243.1, 243.3] |
| bge-small-k20 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.4680 [0.4367, 0.5057] | 0.5320 [0.4943, 0.5633] | 0.8074 [0.7608, 0.8523] | 0.5654 | 15.4 [14.9, 15.8] | 9.11 | 19.49 [19.26, 19.68] | 65.7 [63.9, 68.4] |
| bge-small-k20 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.4835 [0.4556, 0.5157] | 0.5165 [0.4843, 0.5444] | 0.7839 [0.7384, 0.8321] | 0.5925 | 13.5 [12.8, 14.1] | 7.93 | 21.81 [21.44, 22.16] | 89.7 [85.9, 94.3] |
| bge-small-k20 | learned-own | 1.0000 [1.0000, 1.0000] | 0.5052 [0.4777, 0.5351] | 0.4948 [0.4649, 0.5223] | 0.7508 [0.7118, 0.7897] | 0.6037 | 15.1 [14.3, 15.6] | 8.79 | 19.69 [19.40, 19.96] | 73.1 [70.6, 76.6] |
| bge-small-k20 | lcc-full-gold | 0.9220 [0.8960, 0.9440] | 0.7113 [0.6858, 0.7357] | 0.2887 [0.2643, 0.3142] | 0.4382 [0.4020, 0.4775] | 0.8098 | 85.2 [65.8, 84.8] | 5.19 | 19.71 [19.17, 20.24] | 55.6 [53.0, 59.7] |
| bge-small-k20 | lcc-gold | 0.9220 [0.8960, 0.9440] | 0.7132 [0.6881, 0.7373] | 0.2868 [0.2627, 0.3119] | 0.4353 [0.3994, 0.4745] | 0.8061 | 35.0 [28.4, 37.3] | 4.20 | 18.59 [18.13, 19.04] | 69.4 [65.5, 74.7] |
| bge-small-k20 | lcc-full-placed | 0.5660 [0.5220, 0.6080] | 0.9298 [0.9071, 0.9490] | 0.0702 [0.0510, 0.0929] | 0.1065 [0.0785, 0.1415] | 0.9997 | 340.8 [243.6, 291.0] | 4.14 | 13.98 [13.52, 14.42] | 124.9 [124.9, 125.9] |
| bge-small-k20 | ilc2-placed | 0.4560 [0.4120, 0.4980] | 0.9647 [0.9498, 0.9762] | 0.0353 [0.0238, 0.0502] | 0.0536 [0.0365, 0.0759] | 1.0270 | 188.1 [134.7, 173.2] | 2.94 | 12.46 [12.10, 12.81] | 155.5 [155.2, 156.8] |
| bge-small-k20 | learned-placed | 0.0860 [0.0620, 0.1120] | 0.9897 [0.9801, 0.9960] | 0.0103 [0.0040, 0.0199] | 0.0156 [0.0062, 0.0301] | 1.0426 | 26.1 [19.5, 25.7] | 1.34 | 10.57 [10.37, 10.79] | 231.6 [231.9, 232.4] |
| bge-small-k20 | unesco-thesaurus-placed | 0.1580 [0.1280, 0.1920] | 0.9899 [0.9830, 0.9951] | 0.0101 [0.0049, 0.0170] | 0.0153 [0.0074, 0.0255] | 1.0441 | 93.8 [63.5, 81.8] | 1.65 | 10.82 [10.60, 11.06] | 216.0 [216.3, 216.8] |
| bge-small-k20 | lcc-placed | 0.0840 [0.0620, 0.1080] | 0.9973 [0.9934, 0.9994] | 0.0027 [0.0006, 0.0066] | 0.0041 [0.0009, 0.0101] | 1.0483 | 63.0 [36.3, 56.7] | 1.29 | 10.21 [10.11, 10.31] | 236.8 [237.2, 237.4] |
| bge-small-k20 | bk-placed | 0.0440 [0.0260, 0.0640] | 0.9976 [0.9941, 0.9997] | 0.0024 [0.0003, 0.0059] | 0.0036 [0.0005, 0.0091] | 1.0483 | 30.8 [17.5, 32.6] | 1.23 | 10.14 [10.05, 10.25] | 242.3 [242.6, 242.9] |
| bge-small-k20 | fdc-placed | 0.0520 [0.0340, 0.0720] | 0.9984 [0.9955, 0.9999] | 0.0016 [0.0001, 0.0045] | 0.0024 [0.0001, 0.0068] | 1.0489 | 31.1 [16.1, 29.1] | 1.14 | 10.10 [10.03, 10.17] | 242.7 [243.1, 243.3] |
| bge-small-k50 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.5061 [0.4836, 0.5315] | 0.4939 [0.4685, 0.5164] | 0.8452 [0.8019, 0.8865] | 0.6151 | 13.5 [12.8, 14.1] | 7.93 | 21.81 [21.44, 22.16] | 89.7 [85.9, 94.3] |
| bge-small-k50 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.5069 [0.4806, 0.5378] | 0.4931 [0.4622, 0.5194] | 0.8438 [0.7980, 0.8872] | 0.6043 | 15.4 [14.9, 15.8] | 9.11 | 19.49 [19.26, 19.68] | 65.7 [63.9, 68.4] |
| bge-small-k50 | learned-own | 1.0000 [1.0000, 1.0000] | 0.5363 [0.5134, 0.5610] | 0.4637 [0.4390, 0.4866] | 0.7935 [0.7532, 0.8337] | 0.6347 | 15.1 [14.3, 15.6] | 8.79 | 19.69 [19.40, 19.96] | 73.1 [70.6, 76.6] |
| bge-small-k50 | lcc-full-gold | 0.9220 [0.8960, 0.9440] | 0.7237 [0.7029, 0.7445] | 0.2763 [0.2555, 0.2971] | 0.4729 [0.4395, 0.5072] | 0.8222 | 85.2 [65.8, 84.8] | 5.19 | 19.71 [19.17, 20.24] | 55.6 [53.0, 59.7] |
| bge-small-k50 | lcc-gold | 0.9220 [0.8960, 0.9440] | 0.7252 [0.7050, 0.7455] | 0.2748 [0.2545, 0.2950] | 0.4702 [0.4369, 0.5041] | 0.8182 | 35.0 [28.4, 37.3] | 4.20 | 18.59 [18.13, 19.04] | 69.4 [65.5, 74.7] |
| bge-small-k50 | lcc-full-placed | 0.5660 [0.5220, 0.6080] | 0.9389 [0.9220, 0.9545] | 0.0611 [0.0455, 0.0780] | 0.1045 [0.0786, 0.1337] | 1.0088 | 340.8 [243.6, 291.0] | 4.14 | 13.98 [13.52, 14.42] | 124.9 [124.9, 125.9] |
| bge-small-k50 | ilc2-placed | 0.4560 [0.4120, 0.4980] | 0.9736 [0.9638, 0.9815] | 0.0264 [0.0185, 0.0362] | 0.0452 [0.0316, 0.0615] | 1.0359 | 188.1 [134.7, 173.2] | 2.94 | 12.46 [12.10, 12.81] | 155.5 [155.2, 156.8] |
| bge-small-k50 | learned-placed | 0.0860 [0.0620, 0.1120] | 0.9920 [0.9852, 0.9966] | 0.0080 [0.0034, 0.0148] | 0.0138 [0.0060, 0.0256] | 1.0448 | 26.1 [19.5, 25.7] | 1.34 | 10.57 [10.37, 10.79] | 231.6 [231.9, 232.4] |
| bge-small-k50 | unesco-thesaurus-placed | 0.1580 [0.1280, 0.1920] | 0.9923 [0.9876, 0.9960] | 0.0077 [0.0040, 0.0124] | 0.0131 [0.0068, 0.0212] | 1.0464 | 93.8 [63.5, 81.8] | 1.65 | 10.82 [10.60, 11.06] | 216.0 [216.3, 216.8] |
| bge-small-k50 | lcc-placed | 0.0840 [0.0620, 0.1080] | 0.9982 [0.9960, 0.9995] | 0.0018 [0.0005, 0.0040] | 0.0031 [0.0009, 0.0067] | 1.0492 | 63.0 [36.3, 56.7] | 1.29 | 10.21 [10.11, 10.31] | 236.8 [237.2, 237.4] |
| bge-small-k50 | bk-placed | 0.0440 [0.0260, 0.0640] | 0.9986 [0.9966, 0.9998] | 0.0014 [0.0002, 0.0034] | 0.0025 [0.0004, 0.0059] | 1.0493 | 30.8 [17.5, 32.6] | 1.23 | 10.14 [10.05, 10.25] | 242.3 [242.6, 242.9] |
| bge-small-k50 | fdc-placed | 0.0520 [0.0340, 0.0720] | 0.9989 [0.9971, 0.9999] | 0.0011 [0.0001, 0.0029] | 0.0019 [0.0002, 0.0049] | 1.0494 | 31.1 [16.1, 29.1] | 1.14 | 10.10 [10.03, 10.17] | 242.7 [243.1, 243.3] |
| nomic-k10 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.3771 [0.3424, 0.4195] | 0.6229 [0.5805, 0.6576] | 0.8199 [0.7748, 0.8584] | 0.4745 | 15.4 [14.9, 15.8] | 9.11 | 19.49 [19.26, 19.68] | 65.7 [63.9, 68.4] |
| nomic-k10 | learned-own | 1.0000 [1.0000, 1.0000] | 0.4828 [0.4462, 0.5254] | 0.5172 [0.4746, 0.5538] | 0.6808 [0.6279, 0.7280] | 0.5812 | 15.1 [14.3, 15.6] | 8.79 | 19.69 [19.40, 19.96] | 73.1 [70.6, 76.6] |
| nomic-k10 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.5259 [0.4934, 0.5664] | 0.4741 [0.4336, 0.5066] | 0.6239 [0.5764, 0.6649] | 0.6350 | 13.5 [12.8, 14.1] | 7.93 | 21.81 [21.44, 22.16] | 89.7 [85.9, 94.3] |
| nomic-k10 | lcc-full-gold | 0.9220 [0.8960, 0.9440] | 0.6958 [0.6675, 0.7247] | 0.3042 [0.2753, 0.3325] | 0.4004 [0.3611, 0.4404] | 0.7944 | 85.2 [65.8, 84.8] | 5.19 | 19.71 [19.17, 20.24] | 55.6 [53.0, 59.7] |
| nomic-k10 | lcc-gold | 0.9220 [0.8960, 0.9440] | 0.6979 [0.6710, 0.7269] | 0.3021 [0.2731, 0.3290] | 0.3976 [0.3586, 0.4384] | 0.7909 | 35.0 [28.4, 37.3] | 4.20 | 18.59 [18.13, 19.04] | 69.4 [65.5, 74.7] |
| nomic-k10 | lcc-full-placed | 0.5660 [0.5220, 0.6080] | 0.9127 [0.8825, 0.9367] | 0.0873 [0.0633, 0.1175] | 0.1149 [0.0835, 0.1545] | 0.9826 | 340.8 [243.6, 291.0] | 4.14 | 13.98 [13.52, 14.42] | 124.9 [124.9, 125.9] |
| nomic-k10 | ilc2-placed | 0.4560 [0.4120, 0.4980] | 0.9471 [0.9254, 0.9660] | 0.0529 [0.0340, 0.0746] | 0.0697 [0.0451, 0.0978] | 1.0094 | 188.1 [134.7, 173.2] | 2.94 | 12.46 [12.10, 12.81] | 155.5 [155.2, 156.8] |
| nomic-k10 | unesco-thesaurus-placed | 0.1580 [0.1280, 0.1920] | 0.9871 [0.9772, 0.9943] | 0.0129 [0.0057, 0.0228] | 0.0169 [0.0075, 0.0297] | 1.0413 | 93.8 [63.5, 81.8] | 1.65 | 10.82 [10.60, 11.06] | 216.0 [216.3, 216.8] |
| nomic-k10 | learned-placed | 0.0860 [0.0620, 0.1120] | 0.9902 [0.9791, 0.9967] | 0.0098 [0.0033, 0.0209] | 0.0129 [0.0043, 0.0275] | 1.0430 | 26.1 [19.5, 25.7] | 1.34 | 10.57 [10.37, 10.79] | 231.6 [231.9, 232.4] |
| nomic-k10 | lcc-placed | 0.0840 [0.0620, 0.1080] | 0.9953 [0.9894, 0.9990] | 0.0047 [0.0010, 0.0106] | 0.0061 [0.0013, 0.0139] | 1.0464 | 63.0 [36.3, 56.7] | 1.29 | 10.21 [10.11, 10.31] | 236.8 [237.2, 237.4] |
| nomic-k10 | bk-placed | 0.0440 [0.0260, 0.0640] | 0.9970 [0.9924, 0.9996] | 0.0030 [0.0004, 0.0076] | 0.0039 [0.0005, 0.0101] | 1.0477 | 30.8 [17.5, 32.6] | 1.23 | 10.14 [10.05, 10.25] | 242.3 [242.6, 242.9] |
| nomic-k10 | fdc-placed | 0.0520 [0.0340, 0.0720] | 0.9971 [0.9922, 0.9998] | 0.0029 [0.0002, 0.0078] | 0.0038 [0.0003, 0.0106] | 1.0476 | 31.1 [16.1, 29.1] | 1.14 | 10.10 [10.03, 10.17] | 242.7 [243.1, 243.3] |
| nomic-k20 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.3973 [0.3652, 0.4355] | 0.6027 [0.5645, 0.6348] | 0.8373 [0.7994, 0.8707] | 0.4947 | 15.4 [14.9, 15.8] | 9.11 | 19.49 [19.26, 19.68] | 65.7 [63.9, 68.4] |
| nomic-k20 | learned-own | 1.0000 [1.0000, 1.0000] | 0.4980 [0.4629, 0.5369] | 0.5020 [0.4631, 0.5371] | 0.6974 [0.6485, 0.7456] | 0.5965 | 15.1 [14.3, 15.6] | 8.79 | 19.69 [19.40, 19.96] | 73.1 [70.6, 76.6] |
| nomic-k20 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.5381 [0.5083, 0.5726] | 0.4619 [0.4274, 0.4917] | 0.6418 [0.6006, 0.6797] | 0.6471 | 13.5 [12.8, 14.1] | 7.93 | 21.81 [21.44, 22.16] | 89.7 [85.9, 94.3] |
| nomic-k20 | lcc-full-gold | 0.9220 [0.8960, 0.9440] | 0.7054 [0.6820, 0.7315] | 0.2946 [0.2685, 0.3180] | 0.4093 [0.3723, 0.4457] | 0.8040 | 85.2 [65.8, 84.8] | 5.19 | 19.71 [19.17, 20.24] | 55.6 [53.0, 59.7] |
| nomic-k20 | lcc-gold | 0.9220 [0.8960, 0.9440] | 0.7072 [0.6839, 0.7332] | 0.2928 [0.2668, 0.3161] | 0.4068 [0.3701, 0.4435] | 0.8002 | 35.0 [28.4, 37.3] | 4.20 | 18.59 [18.13, 19.04] | 69.4 [65.5, 74.7] |
| nomic-k20 | lcc-full-placed | 0.5660 [0.5220, 0.6080] | 0.9220 [0.8977, 0.9421] | 0.0780 [0.0579, 0.1023] | 0.1083 [0.0814, 0.1428] | 0.9920 | 340.8 [243.6, 291.0] | 4.14 | 13.98 [13.52, 14.42] | 124.9 [124.9, 125.9] |
| nomic-k20 | ilc2-placed | 0.4560 [0.4120, 0.4980] | 0.9586 [0.9435, 0.9720] | 0.0414 [0.0280, 0.0565] | 0.0575 [0.0391, 0.0784] | 1.0209 | 188.1 [134.7, 173.2] | 2.94 | 12.46 [12.10, 12.81] | 155.5 [155.2, 156.8] |
| nomic-k20 | unesco-thesaurus-placed | 0.1580 [0.1280, 0.1920] | 0.9898 [0.9825, 0.9949] | 0.0102 [0.0051, 0.0175] | 0.0141 [0.0071, 0.0243] | 1.0440 | 93.8 [63.5, 81.8] | 1.65 | 10.82 [10.60, 11.06] | 216.0 [216.3, 216.8] |
| nomic-k20 | learned-placed | 0.0860 [0.0620, 0.1120] | 0.9920 [0.9836, 0.9971] | 0.0080 [0.0029, 0.0164] | 0.0111 [0.0039, 0.0231] | 1.0448 | 26.1 [19.5, 25.7] | 1.34 | 10.57 [10.37, 10.79] | 231.6 [231.9, 232.4] |
| nomic-k20 | lcc-placed | 0.0840 [0.0620, 0.1080] | 0.9968 [0.9931, 0.9991] | 0.0032 [0.0009, 0.0069] | 0.0044 [0.0012, 0.0099] | 1.0478 | 63.0 [36.3, 56.7] | 1.29 | 10.21 [10.11, 10.31] | 236.8 [237.2, 237.4] |
| nomic-k20 | bk-placed | 0.0440 [0.0260, 0.0640] | 0.9980 [0.9951, 0.9997] | 0.0020 [0.0003, 0.0049] | 0.0028 [0.0004, 0.0068] | 1.0487 | 30.8 [17.5, 32.6] | 1.23 | 10.14 [10.05, 10.25] | 242.3 [242.6, 242.9] |
| nomic-k20 | fdc-placed | 0.0520 [0.0340, 0.0720] | 0.9980 [0.9948, 0.9998] | 0.0020 [0.0002, 0.0052] | 0.0027 [0.0003, 0.0074] | 1.0485 | 31.1 [16.1, 29.1] | 1.14 | 10.10 [10.03, 10.17] | 242.7 [243.1, 243.3] |
| nomic-k50 | learned-nomic-own | 1.0000 [1.0000, 1.0000] | 0.4394 [0.4112, 0.4709] | 0.5606 [0.5291, 0.5888] | 0.8800 [0.8463, 0.9130] | 0.5369 | 15.4 [14.9, 15.8] | 9.11 | 19.49 [19.26, 19.68] | 65.7 [63.9, 68.4] |
| nomic-k50 | learned-own | 1.0000 [1.0000, 1.0000] | 0.5285 [0.4989, 0.5611] | 0.4715 [0.4389, 0.5011] | 0.7402 [0.6912, 0.7868] | 0.6269 | 15.1 [14.3, 15.6] | 8.79 | 19.69 [19.40, 19.96] | 73.1 [70.6, 76.6] |
| nomic-k50 | learned-bge-small-own | 1.0000 [1.0000, 1.0000] | 0.5614 [0.5350, 0.5907] | 0.4386 [0.4093, 0.4650] | 0.6884 [0.6458, 0.7307] | 0.6705 | 13.5 [12.8, 14.1] | 7.93 | 21.81 [21.44, 22.16] | 89.7 [85.9, 94.3] |
| nomic-k50 | lcc-full-gold | 0.9220 [0.8960, 0.9440] | 0.7157 [0.6956, 0.7376] | 0.2843 [0.2624, 0.3044] | 0.4462 [0.4104, 0.4815] | 0.8143 | 85.2 [65.8, 84.8] | 5.19 | 19.71 [19.17, 20.24] | 55.6 [53.0, 59.7] |
| nomic-k50 | lcc-gold | 0.9220 [0.8960, 0.9440] | 0.7172 [0.6971, 0.7395] | 0.2828 [0.2605, 0.3029] | 0.4439 [0.4078, 0.4798] | 0.8101 | 35.0 [28.4, 37.3] | 4.20 | 18.59 [18.13, 19.04] | 69.4 [65.5, 74.7] |
| nomic-k50 | lcc-full-placed | 0.5660 [0.5220, 0.6080] | 0.9300 [0.9114, 0.9465] | 0.0700 [0.0535, 0.0886] | 0.1099 [0.0845, 0.1407] | 0.9999 | 340.8 [243.6, 291.0] | 4.14 | 13.98 [13.52, 14.42] | 124.9 [124.9, 125.9] |
| nomic-k50 | ilc2-placed | 0.4560 [0.4120, 0.4980] | 0.9684 [0.9585, 0.9785] | 0.0316 [0.0215, 0.0415] | 0.0497 [0.0342, 0.0654] | 1.0307 | 188.1 [134.7, 173.2] | 2.94 | 12.46 [12.10, 12.81] | 155.5 [155.2, 156.8] |
| nomic-k50 | unesco-thesaurus-placed | 0.1580 [0.1280, 0.1920] | 0.9922 [0.9873, 0.9958] | 0.0078 [0.0042, 0.0127] | 0.0123 [0.0066, 0.0200] | 1.0463 | 93.8 [63.5, 81.8] | 1.65 | 10.82 [10.60, 11.06] | 216.0 [216.3, 216.8] |
| nomic-k50 | learned-placed | 0.0860 [0.0620, 0.1120] | 0.9930 [0.9872, 0.9969] | 0.0070 [0.0031, 0.0128] | 0.0110 [0.0048, 0.0205] | 1.0458 | 26.1 [19.5, 25.7] | 1.34 | 10.57 [10.37, 10.79] | 231.6 [231.9, 232.4] |
| nomic-k50 | lcc-placed | 0.0840 [0.0620, 0.1080] | 0.9980 [0.9959, 0.9995] | 0.0020 [0.0005, 0.0041] | 0.0032 [0.0008, 0.0067] | 1.0490 | 63.0 [36.3, 56.7] | 1.29 | 10.21 [10.11, 10.31] | 236.8 [237.2, 237.4] |
| nomic-k50 | bk-placed | 0.0440 [0.0260, 0.0640] | 0.9986 [0.9967, 0.9997] | 0.0014 [0.0003, 0.0033] | 0.0023 [0.0005, 0.0053] | 1.0493 | 30.8 [17.5, 32.6] | 1.23 | 10.14 [10.05, 10.25] | 242.3 [242.6, 242.9] |
| nomic-k50 | fdc-placed | 0.0520 [0.0340, 0.0720] | 0.9987 [0.9966, 0.9997] | 0.0013 [0.0003, 0.0034] | 0.0021 [0.0004, 0.0054] | 1.0491 | 31.1 [16.1, 29.1] | 1.14 | 10.10 [10.03, 10.17] | 242.7 [243.1, 243.3] |

### split

| condition | row | coverage [95% CI] | cost_norm [95% CI] | MW revenue_norm [95% CI] | skill [95% CI] | J | eff. classes [95% CI] | mean depth | nav log a=1 [95% CI] | nav scan [95% CI] |
| --- | --- | --- | --- | --- | --- | ---: | --- | ---: | --- | --- |
| minilm-k10 | learned-own | 1.0000 [1.0000, 1.0000] | 0.1509 [0.1347, 0.1709] | 0.8491 [0.8291, 0.8653] | 0.8917 [0.8713, 0.9098] | 0.2317 | 39.2 [38.5, 39.4] | 4.98 | 16.15 [16.11, 16.18] | 40.1 [39.6, 41.6] |
| minilm-k10 | learned-bge-small-heldout | 1.0000 [1.0000, 1.0000] | 0.1586 [0.1420, 0.1786] | 0.8414 [0.8214, 0.8580] | 0.8836 [0.8634, 0.9032] | 0.2390 | 36.3 [35.6, 36.5] | 5.00 | 16.08 [16.05, 16.10] | 44.0 [43.5, 45.4] |
| minilm-k10 | learned-heldout | 1.0000 [1.0000, 1.0000] | 0.1597 [0.1408, 0.1784] | 0.8403 [0.8216, 0.8592] | 0.8825 [0.8632, 0.9031] | 0.2402 | 33.9 [33.5, 34.0] | 5.00 | 16.10 [16.07, 16.13] | 43.8 [43.5, 45.3] |
| minilm-k10 | learned-nomic-heldout | 1.0000 [1.0000, 1.0000] | 0.1761 [0.1587, 0.1971] | 0.8239 [0.8029, 0.8413] | 0.8652 [0.8424, 0.8858] | 0.2570 | 39.8 [39.0, 40.0] | 4.97 | 16.19 [16.15, 16.22] | 40.8 [40.3, 42.5] |
| minilm-k10 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3272 [0.2986, 0.3593] | 0.6728 [0.6407, 0.7014] | 0.7065 [0.6731, 0.7382] | 0.4069 | 35.7 [33.9, 36.6] | 3.90 | 15.92 [15.82, 16.02] | 75.1 [71.8, 79.4] |
| minilm-k10 | openalex-topics-placed | 0.9750 [0.9680, 0.9820] | 0.6450 [0.6159, 0.6745] | 0.3550 [0.3255, 0.3841] | 0.3728 [0.3426, 0.4031] | 0.7511 | 70.7 [64.8, 70.4] | 4.43 | 21.22 [21.04, 21.38] | 76.8 [75.0, 79.7] |
| minilm-k10 | unesco-thesaurus-placed | 0.9740 [0.9670, 0.9810] | 0.7030 [0.6769, 0.7323] | 0.2970 [0.2677, 0.3231] | 0.3119 [0.2820, 0.3392] | 0.8074 | 134.4 [123.3, 135.0] | 4.89 | 20.89 [20.71, 21.06] | 76.1 [74.3, 78.9] |
| minilm-k10 | ipc-placed | 0.9815 [0.9750, 0.9870] | 0.7578 [0.7333, 0.7833] | 0.2422 [0.2167, 0.2667] | 0.2544 [0.2279, 0.2809] | 0.8610 | 302.9 [271.0, 299.2] | 6.33 | 20.65 [20.46, 20.83] | 76.2 [73.2, 80.1] |
| minilm-k10 | ilc2-placed | 1.0000 [1.0000, 1.0000] | 0.8111 [0.7827, 0.8361] | 0.1889 [0.1639, 0.2173] | 0.1984 [0.1722, 0.2287] | 0.9054 | 262.6 [235.9, 258.0] | 5.05 | 18.87 [18.69, 19.06] | 52.7 [52.0, 54.6] |
| minilm-k10 | fdc-placed | 0.7730 [0.7545, 0.7925] | 0.8194 [0.7897, 0.8443] | 0.1806 [0.1557, 0.2103] | 0.1896 [0.1635, 0.2202] | 0.9192 | 52.9 [49.5, 54.1] | 3.20 | 19.95 [19.68, 20.23] | 292.2 [287.9, 297.2] |
| minilm-k10 | lcc-full-placed | 0.9995 [0.9985, 1.0000] | 0.8204 [0.7955, 0.8442] | 0.1796 [0.1558, 0.2045] | 0.1886 [0.1640, 0.2145] | 0.9142 | 566.6 [487.1, 527.8] | 6.09 | 18.77 [18.58, 18.94] | 30.5 [30.6, 31.5] |
| minilm-k10 | acm-ccs-placed | 0.7460 [0.7265, 0.7640] | 0.8280 [0.8018, 0.8538] | 0.1720 [0.1462, 0.1982] | 0.1806 [0.1532, 0.2079] | 0.9217 | 84.1 [76.1, 86.2] | 3.60 | 18.74 [18.49, 18.98] | 301.5 [298.9, 305.1] |
| minilm-k10 | eurovoc-placed | 0.9775 [0.9710, 0.9835] | 0.8369 [0.8107, 0.8624] | 0.1631 [0.1376, 0.1893] | 0.1712 [0.1448, 0.1991] | 0.9317 | 250.6 [227.4, 248.6] | 5.00 | 18.94 [18.78, 19.11] | 65.0 [64.1, 67.1] |
| minilm-k10 | lcc-placed | 0.8755 [0.8615, 0.8905] | 0.8445 [0.8194, 0.8674] | 0.1555 [0.1326, 0.1806] | 0.1633 [0.1397, 0.1902] | 0.9441 | 120.1 [110.4, 122.1] | 4.04 | 19.90 [19.66, 20.14] | 183.9 [181.5, 187.4] |
| minilm-k10 | arxiv-placed | 0.4185 [0.3970, 0.4405] | 0.8923 [0.8690, 0.9139] | 0.1077 [0.0861, 0.1310] | 0.1131 [0.0909, 0.1378] | 0.9744 | 13.7 [12.7, 14.4] | 2.19 | 16.41 [16.12, 16.69] | 661.1 [653.4, 670.0] |
| minilm-k10 | bk-placed | 0.4825 [0.4605, 0.5050] | 0.8931 [0.8669, 0.9175] | 0.1069 [0.0825, 0.1331] | 0.1123 [0.0868, 0.1394] | 0.9777 | 71.4 [65.2, 73.5] | 3.17 | 16.93 [16.65, 17.23] | 551.9 [549.6, 555.6] |
| minilm-k10 | jel-placed | 0.4780 [0.4540, 0.4990] | 0.9020 [0.8768, 0.9241] | 0.0980 [0.0759, 0.1232] | 0.1029 [0.0798, 0.1293] | 0.9807 | 28.5 [26.4, 29.8] | 2.28 | 15.74 [15.49, 15.96] | 584.8 [580.5, 590.1] |
| minilm-k10 | msc-placed | 0.3870 [0.3665, 0.4075] | 0.9502 [0.9337, 0.9654] | 0.0498 [0.0346, 0.0663] | 0.0523 [0.0363, 0.0699] | 1.0211 | 112.9 [95.0, 113.1] | 1.99 | 14.18 [14.01, 14.34] | 652.1 [651.2, 654.2] |
| minilm-k10 | msc-gold | 0.1370 [0.1225, 0.1525] | 0.9799 [0.9671, 0.9892] | 0.0201 [0.0108, 0.0329] | 0.0211 [0.0114, 0.0347] | 1.0425 | 268.5 [182.8, 216.9] | 1.41 | 12.52 [12.43, 12.62] | 888.7 [889.1, 889.3] |
| minilm-k10 | jel-gold | 0.0000 [0.0000, 0.0000] | 1.0000 [1.0000, 1.0000] | 0.0000 [0.0000, 0.0000] | 0.0000 [0.0000, 0.0000] | 1.0598 | 1.0 [1.0, 1.0] | 1.00 | 11.97 [11.97, 11.97] | 1001.5 [1002.0, 1002.0] |
| minilm-k20 | learned-heldout | 1.0000 [1.0000, 1.0000] | 0.1722 [0.1565, 0.1889] | 0.8278 [0.8111, 0.8435] | 0.9222 [0.9045, 0.9404] | 0.2528 | 33.9 [33.5, 34.0] | 5.00 | 16.10 [16.07, 16.13] | 43.8 [43.5, 45.3] |
| minilm-k20 | learned-own | 1.0000 [1.0000, 1.0000] | 0.1793 [0.1647, 0.1967] | 0.8207 [0.8033, 0.8353] | 0.9144 [0.8974, 0.9300] | 0.2600 | 39.2 [38.5, 39.4] | 4.98 | 16.15 [16.11, 16.18] | 40.1 [39.6, 41.6] |
| minilm-k20 | learned-bge-small-heldout | 1.0000 [1.0000, 1.0000] | 0.1802 [0.1652, 0.1974] | 0.8198 [0.8026, 0.8348] | 0.9134 [0.8956, 0.9311] | 0.2606 | 36.3 [35.6, 36.5] | 5.00 | 16.08 [16.05, 16.10] | 44.0 [43.5, 45.4] |
| minilm-k20 | learned-nomic-heldout | 1.0000 [1.0000, 1.0000] | 0.1933 [0.1764, 0.2131] | 0.8067 [0.7869, 0.8236] | 0.8987 [0.8781, 0.9187] | 0.2743 | 39.8 [39.0, 40.0] | 4.97 | 16.19 [16.15, 16.22] | 40.8 [40.3, 42.5] |
| minilm-k20 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3424 [0.3181, 0.3698] | 0.6576 [0.6302, 0.6819] | 0.7327 [0.7028, 0.7601] | 0.4220 | 35.7 [33.9, 36.6] | 3.90 | 15.92 [15.82, 16.02] | 75.1 [71.8, 79.4] |
| minilm-k20 | openalex-topics-placed | 0.9750 [0.9680, 0.9820] | 0.6519 [0.6301, 0.6746] | 0.3481 [0.3254, 0.3699] | 0.3878 [0.3633, 0.4121] | 0.7580 | 70.7 [64.8, 70.4] | 4.43 | 21.22 [21.04, 21.38] | 76.8 [75.0, 79.7] |
| minilm-k20 | unesco-thesaurus-placed | 0.9740 [0.9670, 0.9810] | 0.7192 [0.6998, 0.7404] | 0.2808 [0.2596, 0.3002] | 0.3129 [0.2886, 0.3353] | 0.8236 | 134.4 [123.3, 135.0] | 4.89 | 20.89 [20.71, 21.06] | 76.1 [74.3, 78.9] |
| minilm-k20 | ipc-placed | 0.9815 [0.9750, 0.9870] | 0.7657 [0.7466, 0.7851] | 0.2343 [0.2149, 0.2534] | 0.2611 [0.2393, 0.2826] | 0.8689 | 302.9 [271.0, 299.2] | 6.33 | 20.65 [20.46, 20.83] | 76.2 [73.2, 80.1] |
| minilm-k20 | ilc2-placed | 1.0000 [1.0000, 1.0000] | 0.8196 [0.7972, 0.8393] | 0.1804 [0.1607, 0.2028] | 0.2010 [0.1792, 0.2258] | 0.9139 | 262.6 [235.9, 258.0] | 5.05 | 18.87 [18.69, 19.06] | 52.7 [52.0, 54.6] |
| minilm-k20 | lcc-full-placed | 0.9995 [0.9985, 1.0000] | 0.8325 [0.8138, 0.8506] | 0.1675 [0.1494, 0.1862] | 0.1866 [0.1664, 0.2077] | 0.9264 | 566.6 [487.1, 527.8] | 6.09 | 18.77 [18.58, 18.94] | 30.5 [30.6, 31.5] |
| minilm-k20 | fdc-placed | 0.7730 [0.7545, 0.7925] | 0.8328 [0.8126, 0.8526] | 0.1672 [0.1474, 0.1874] | 0.1863 [0.1647, 0.2092] | 0.9326 | 52.9 [49.5, 54.1] | 3.20 | 19.95 [19.68, 20.23] | 292.2 [287.9, 297.2] |
| minilm-k20 | acm-ccs-placed | 0.7460 [0.7265, 0.7640] | 0.8346 [0.8146, 0.8546] | 0.1654 [0.1454, 0.1854] | 0.1843 [0.1618, 0.2073] | 0.9283 | 84.1 [76.1, 86.2] | 3.60 | 18.74 [18.49, 18.98] | 301.5 [298.9, 305.1] |
| minilm-k20 | eurovoc-placed | 0.9775 [0.9710, 0.9835] | 0.8411 [0.8214, 0.8594] | 0.1589 [0.1406, 0.1786] | 0.1770 [0.1570, 0.1999] | 0.9359 | 250.6 [227.4, 248.6] | 5.00 | 18.94 [18.78, 19.11] | 65.0 [64.1, 67.1] |
| minilm-k20 | lcc-placed | 0.8755 [0.8615, 0.8905] | 0.8547 [0.8361, 0.8715] | 0.1453 [0.1285, 0.1639] | 0.1619 [0.1432, 0.1832] | 0.9542 | 120.1 [110.4, 122.1] | 4.04 | 19.90 [19.66, 20.14] | 183.9 [181.5, 187.4] |
| minilm-k20 | arxiv-placed | 0.4185 [0.3970, 0.4405] | 0.8955 [0.8768, 0.9139] | 0.1045 [0.0861, 0.1232] | 0.1164 [0.0961, 0.1375] | 0.9776 | 13.7 [12.7, 14.4] | 2.19 | 16.41 [16.12, 16.69] | 661.1 [653.4, 670.0] |
| minilm-k20 | bk-placed | 0.4825 [0.4605, 0.5050] | 0.8959 [0.8732, 0.9155] | 0.1041 [0.0845, 0.1268] | 0.1159 [0.0943, 0.1407] | 0.9806 | 71.4 [65.2, 73.5] | 3.17 | 16.93 [16.65, 17.23] | 551.9 [549.6, 555.6] |
| minilm-k20 | jel-placed | 0.4780 [0.4540, 0.4990] | 0.9029 [0.8814, 0.9221] | 0.0971 [0.0779, 0.1186] | 0.1082 [0.0865, 0.1319] | 0.9815 | 28.5 [26.4, 29.8] | 2.28 | 15.74 [15.49, 15.96] | 584.8 [580.5, 590.1] |
| minilm-k20 | msc-placed | 0.3870 [0.3665, 0.4075] | 0.9552 [0.9420, 0.9675] | 0.0448 [0.0325, 0.0580] | 0.0499 [0.0365, 0.0647] | 1.0261 | 112.9 [95.0, 113.1] | 1.99 | 14.18 [14.01, 14.34] | 652.1 [651.2, 654.2] |
| minilm-k20 | msc-gold | 0.1370 [0.1225, 0.1525] | 0.9805 [0.9713, 0.9878] | 0.0195 [0.0122, 0.0287] | 0.0217 [0.0137, 0.0321] | 1.0431 | 268.5 [182.8, 216.9] | 1.41 | 12.52 [12.43, 12.62] | 888.7 [889.1, 889.3] |
| minilm-k20 | jel-gold | 0.0000 [0.0000, 0.0000] | 1.0000 [1.0000, 1.0000] | -0.0000 [-0.0000, -0.0000] | -0.0000 [-0.0000, -0.0000] | 1.0598 | 1.0 [1.0, 1.0] | 1.00 | 11.97 [11.97, 11.97] | 1001.5 [1002.0, 1002.0] |
| minilm-k50 | learned-heldout | 1.0000 [1.0000, 1.0000] | 0.1927 [0.1798, 0.2067] | 0.8073 [0.7933, 0.8202] | 0.9557 [0.9405, 0.9720] | 0.2733 | 33.9 [33.5, 34.0] | 5.00 | 16.10 [16.07, 16.13] | 43.8 [43.5, 45.3] |
| minilm-k50 | learned-bge-small-heldout | 1.0000 [1.0000, 1.0000] | 0.2052 [0.1913, 0.2203] | 0.7948 [0.7797, 0.8087] | 0.9410 [0.9258, 0.9575] | 0.2855 | 36.3 [35.6, 36.5] | 5.00 | 16.08 [16.05, 16.10] | 44.0 [43.5, 45.4] |
| minilm-k50 | learned-own | 1.0000 [1.0000, 1.0000] | 0.2138 [0.2002, 0.2297] | 0.7862 [0.7703, 0.7998] | 0.9307 [0.9156, 0.9468] | 0.2946 | 39.2 [38.5, 39.4] | 4.98 | 16.15 [16.11, 16.18] | 40.1 [39.6, 41.6] |
| minilm-k50 | learned-nomic-heldout | 1.0000 [1.0000, 1.0000] | 0.2142 [0.1998, 0.2310] | 0.7858 [0.7690, 0.8002] | 0.9303 [0.9148, 0.9464] | 0.2951 | 39.8 [39.0, 40.0] | 4.97 | 16.19 [16.15, 16.22] | 40.8 [40.3, 42.5] |
| minilm-k50 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3612 [0.3405, 0.3845] | 0.6388 [0.6155, 0.6595] | 0.7562 [0.7330, 0.7780] | 0.4408 | 35.7 [33.9, 36.6] | 3.90 | 15.92 [15.82, 16.02] | 75.1 [71.8, 79.4] |
| minilm-k50 | openalex-topics-placed | 0.9750 [0.9680, 0.9820] | 0.6607 [0.6441, 0.6770] | 0.3393 [0.3230, 0.3559] | 0.4017 [0.3832, 0.4213] | 0.7668 | 70.7 [64.8, 70.4] | 4.43 | 21.22 [21.04, 21.38] | 76.8 [75.0, 79.7] |
| minilm-k50 | unesco-thesaurus-placed | 0.9740 [0.9670, 0.9810] | 0.7373 [0.7242, 0.7522] | 0.2627 [0.2478, 0.2758] | 0.3109 [0.2938, 0.3282] | 0.8418 | 134.4 [123.3, 135.0] | 4.89 | 20.89 [20.71, 21.06] | 76.1 [74.3, 78.9] |
| minilm-k50 | ipc-placed | 0.9815 [0.9750, 0.9870] | 0.7762 [0.7614, 0.7899] | 0.2238 [0.2101, 0.2386] | 0.2649 [0.2482, 0.2837] | 0.8795 | 302.9 [271.0, 299.2] | 6.33 | 20.65 [20.46, 20.83] | 76.2 [73.2, 80.1] |
| minilm-k50 | ilc2-placed | 1.0000 [1.0000, 1.0000] | 0.8283 [0.8113, 0.8429] | 0.1717 [0.1571, 0.1887] | 0.2033 [0.1863, 0.2245] | 0.9226 | 262.6 [235.9, 258.0] | 5.05 | 18.87 [18.69, 19.06] | 52.7 [52.0, 54.6] |
| minilm-k50 | fdc-placed | 0.7730 [0.7545, 0.7925] | 0.8412 [0.8257, 0.8552] | 0.1588 [0.1448, 0.1743] | 0.1880 [0.1716, 0.2074] | 0.9409 | 52.9 [49.5, 54.1] | 3.20 | 19.95 [19.68, 20.23] | 292.2 [287.9, 297.2] |
| minilm-k50 | acm-ccs-placed | 0.7460 [0.7265, 0.7640] | 0.8437 [0.8288, 0.8582] | 0.1563 [0.1418, 0.1712] | 0.1850 [0.1676, 0.2039] | 0.9374 | 84.1 [76.1, 86.2] | 3.60 | 18.74 [18.49, 18.98] | 301.5 [298.9, 305.1] |
| minilm-k50 | lcc-full-placed | 0.9995 [0.9985, 1.0000] | 0.8446 [0.8305, 0.8576] | 0.1554 [0.1424, 0.1695] | 0.1840 [0.1690, 0.2010] | 0.9384 | 566.6 [487.1, 527.8] | 6.09 | 18.77 [18.58, 18.94] | 30.5 [30.6, 31.5] |
| minilm-k50 | eurovoc-placed | 0.9775 [0.9710, 0.9835] | 0.8551 [0.8422, 0.8684] | 0.1449 [0.1316, 0.1578] | 0.1715 [0.1558, 0.1879] | 0.9498 | 250.6 [227.4, 248.6] | 5.00 | 18.94 [18.78, 19.11] | 65.0 [64.1, 67.1] |
| minilm-k50 | lcc-placed | 0.8755 [0.8615, 0.8905] | 0.8647 [0.8508, 0.8773] | 0.1353 [0.1227, 0.1492] | 0.1602 [0.1455, 0.1767] | 0.9642 | 120.1 [110.4, 122.1] | 4.04 | 19.90 [19.66, 20.14] | 183.9 [181.5, 187.4] |
| minilm-k50 | arxiv-placed | 0.4185 [0.3970, 0.4405] | 0.8931 [0.8767, 0.9103] | 0.1069 [0.0897, 0.1233] | 0.1266 [0.1059, 0.1467] | 0.9751 | 13.7 [12.7, 14.4] | 2.19 | 16.41 [16.12, 16.69] | 661.1 [653.4, 670.0] |
| minilm-k50 | bk-placed | 0.4825 [0.4605, 0.5050] | 0.9035 [0.8872, 0.9188] | 0.0965 [0.0812, 0.1128] | 0.1143 [0.0961, 0.1334] | 0.9881 | 71.4 [65.2, 73.5] | 3.17 | 16.93 [16.65, 17.23] | 551.9 [549.6, 555.6] |
| minilm-k50 | jel-placed | 0.4780 [0.4540, 0.4990] | 0.9105 [0.8954, 0.9242] | 0.0895 [0.0758, 0.1046] | 0.1059 [0.0895, 0.1242] | 0.9892 | 28.5 [26.4, 29.8] | 2.28 | 15.74 [15.49, 15.96] | 584.8 [580.5, 590.1] |
| minilm-k50 | msc-placed | 0.3870 [0.3665, 0.4075] | 0.9621 [0.9524, 0.9706] | 0.0379 [0.0294, 0.0476] | 0.0449 [0.0350, 0.0566] | 1.0330 | 112.9 [95.0, 113.1] | 1.99 | 14.18 [14.01, 14.34] | 652.1 [651.2, 654.2] |
| minilm-k50 | msc-gold | 0.1370 [0.1225, 0.1525] | 0.9826 [0.9747, 0.9886] | 0.0174 [0.0114, 0.0253] | 0.0206 [0.0136, 0.0298] | 1.0451 | 268.5 [182.8, 216.9] | 1.41 | 12.52 [12.43, 12.62] | 888.7 [889.1, 889.3] |
| minilm-k50 | jel-gold | 0.0000 [0.0000, 0.0000] | 1.0000 [1.0000, 1.0000] | 0.0000 [0.0000, 0.0000] | 0.0000 [0.0000, 0.0000] | 1.0598 | 1.0 [1.0, 1.0] | 1.00 | 11.97 [11.97, 11.97] | 1001.5 [1002.0, 1002.0] |
| bge-small-k10 | learned-bge-small-heldout | 1.0000 [1.0000, 1.0000] | 0.1351 [0.1191, 0.1554] | 0.8649 [0.8446, 0.8809] | 0.8985 [0.8790, 0.9164] | 0.2155 | 36.3 [35.6, 36.5] | 5.00 | 16.08 [16.05, 16.10] | 44.0 [43.5, 45.4] |
| bge-small-k10 | learned-nomic-heldout | 1.0000 [1.0000, 1.0000] | 0.1663 [0.1465, 0.1896] | 0.8337 [0.8104, 0.8535] | 0.8661 [0.8433, 0.8875] | 0.2473 | 39.8 [39.0, 40.0] | 4.97 | 16.19 [16.15, 16.22] | 40.8 [40.3, 42.5] |
| bge-small-k10 | learned-heldout | 1.0000 [1.0000, 1.0000] | 0.1736 [0.1538, 0.1959] | 0.8264 [0.8041, 0.8462] | 0.8585 [0.8361, 0.8813] | 0.2541 | 33.9 [33.5, 34.0] | 5.00 | 16.10 [16.07, 16.13] | 43.8 [43.5, 45.3] |
| bge-small-k10 | learned-own | 1.0000 [1.0000, 1.0000] | 0.1783 [0.1593, 0.2037] | 0.8217 [0.7963, 0.8407] | 0.8536 [0.8280, 0.8749] | 0.2591 | 39.2 [38.5, 39.4] | 4.98 | 16.15 [16.11, 16.18] | 40.1 [39.6, 41.6] |
| bge-small-k10 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3225 [0.2935, 0.3567] | 0.6775 [0.6433, 0.7065] | 0.7038 [0.6713, 0.7337] | 0.4021 | 35.7 [33.9, 36.6] | 3.90 | 15.92 [15.82, 16.02] | 75.1 [71.8, 79.4] |
| bge-small-k10 | openalex-topics-placed | 0.9750 [0.9680, 0.9820] | 0.6458 [0.6189, 0.6725] | 0.3542 [0.3275, 0.3811] | 0.3680 [0.3408, 0.3954] | 0.7519 | 70.7 [64.8, 70.4] | 4.43 | 21.22 [21.04, 21.38] | 76.8 [75.0, 79.7] |
| bge-small-k10 | unesco-thesaurus-placed | 0.9740 [0.9670, 0.9810] | 0.7131 [0.6877, 0.7428] | 0.2869 [0.2572, 0.3123] | 0.2980 [0.2680, 0.3248] | 0.8176 | 134.4 [123.3, 135.0] | 4.89 | 20.89 [20.71, 21.06] | 76.1 [74.3, 78.9] |
| bge-small-k10 | ipc-placed | 0.9815 [0.9750, 0.9870] | 0.7717 [0.7468, 0.7974] | 0.2283 [0.2026, 0.2532] | 0.2372 [0.2110, 0.2639] | 0.8749 | 302.9 [271.0, 299.2] | 6.33 | 20.65 [20.46, 20.83] | 76.2 [73.2, 80.1] |
| bge-small-k10 | lcc-full-placed | 0.9995 [0.9985, 1.0000] | 0.8072 [0.7832, 0.8312] | 0.1928 [0.1688, 0.2168] | 0.2003 [0.1754, 0.2250] | 0.9011 | 566.6 [487.1, 527.8] | 6.09 | 18.77 [18.58, 18.94] | 30.5 [30.6, 31.5] |
| bge-small-k10 | ilc2-placed | 1.0000 [1.0000, 1.0000] | 0.8086 [0.7783, 0.8352] | 0.1914 [0.1648, 0.2217] | 0.1988 [0.1720, 0.2309] | 0.9030 | 262.6 [235.9, 258.0] | 5.05 | 18.87 [18.69, 19.06] | 52.7 [52.0, 54.6] |
| bge-small-k10 | fdc-placed | 0.7730 [0.7545, 0.7925] | 0.8247 [0.7984, 0.8511] | 0.1753 [0.1489, 0.2016] | 0.1821 [0.1549, 0.2090] | 0.9245 | 52.9 [49.5, 54.1] | 3.20 | 19.95 [19.68, 20.23] | 292.2 [287.9, 297.2] |
| bge-small-k10 | eurovoc-placed | 0.9775 [0.9710, 0.9835] | 0.8371 [0.8129, 0.8610] | 0.1629 [0.1390, 0.1871] | 0.1693 [0.1443, 0.1952] | 0.9318 | 250.6 [227.4, 248.6] | 5.00 | 18.94 [18.78, 19.11] | 65.0 [64.1, 67.1] |
| bge-small-k10 | acm-ccs-placed | 0.7460 [0.7265, 0.7640] | 0.8419 [0.8151, 0.8679] | 0.1581 [0.1321, 0.1849] | 0.1642 [0.1377, 0.1924] | 0.9356 | 84.1 [76.1, 86.2] | 3.60 | 18.74 [18.49, 18.98] | 301.5 [298.9, 305.1] |
| bge-small-k10 | lcc-placed | 0.8755 [0.8615, 0.8905] | 0.8483 [0.8221, 0.8717] | 0.1517 [0.1283, 0.1779] | 0.1576 [0.1334, 0.1850] | 0.9478 | 120.1 [110.4, 122.1] | 4.04 | 19.90 [19.66, 20.14] | 183.9 [181.5, 187.4] |
| bge-small-k10 | bk-placed | 0.4825 [0.4605, 0.5050] | 0.8976 [0.8732, 0.9210] | 0.1024 [0.0790, 0.1268] | 0.1064 [0.0822, 0.1315] | 0.9822 | 71.4 [65.2, 73.5] | 3.17 | 16.93 [16.65, 17.23] | 551.9 [549.6, 555.6] |
| bge-small-k10 | arxiv-placed | 0.4185 [0.3970, 0.4405] | 0.9069 [0.8847, 0.9268] | 0.0931 [0.0732, 0.1153] | 0.0967 [0.0758, 0.1199] | 0.9890 | 13.7 [12.7, 14.4] | 2.19 | 16.41 [16.12, 16.69] | 661.1 [653.4, 670.0] |
| bge-small-k10 | jel-placed | 0.4780 [0.4540, 0.4990] | 0.9086 [0.8869, 0.9290] | 0.0914 [0.0710, 0.1131] | 0.0949 [0.0736, 0.1171] | 0.9873 | 28.5 [26.4, 29.8] | 2.28 | 15.74 [15.49, 15.96] | 584.8 [580.5, 590.1] |
| bge-small-k10 | msc-placed | 0.3870 [0.3665, 0.4075] | 0.9412 [0.9201, 0.9582] | 0.0588 [0.0418, 0.0799] | 0.0611 [0.0435, 0.0830] | 1.0121 | 112.9 [95.0, 113.1] | 1.99 | 14.18 [14.01, 14.34] | 652.1 [651.2, 654.2] |
| bge-small-k10 | msc-gold | 0.1370 [0.1225, 0.1525] | 0.9725 [0.9574, 0.9842] | 0.0275 [0.0158, 0.0426] | 0.0286 [0.0166, 0.0446] | 1.0351 | 268.5 [182.8, 216.9] | 1.41 | 12.52 [12.43, 12.62] | 888.7 [889.1, 889.3] |
| bge-small-k10 | jel-gold | 0.0000 [0.0000, 0.0000] | 1.0000 [1.0000, 1.0000] | 0.0000 [0.0000, 0.0000] | 0.0000 [0.0000, 0.0000] | 1.0598 | 1.0 [1.0, 1.0] | 1.00 | 11.97 [11.97, 11.97] | 1001.5 [1002.0, 1002.0] |
| bge-small-k20 | learned-bge-small-heldout | 1.0000 [1.0000, 1.0000] | 0.1520 [0.1385, 0.1685] | 0.8480 [0.8315, 0.8615] | 0.9379 [0.9209, 0.9549] | 0.2324 | 36.3 [35.6, 36.5] | 5.00 | 16.08 [16.05, 16.10] | 44.0 [43.5, 45.4] |
| bge-small-k20 | learned-nomic-heldout | 1.0000 [1.0000, 1.0000] | 0.1757 [0.1582, 0.1951] | 0.8243 [0.8049, 0.8418] | 0.9117 [0.8928, 0.9313] | 0.2566 | 39.8 [39.0, 40.0] | 4.97 | 16.19 [16.15, 16.22] | 40.8 [40.3, 42.5] |
| bge-small-k20 | learned-heldout | 1.0000 [1.0000, 1.0000] | 0.1854 [0.1686, 0.2034] | 0.8146 [0.7966, 0.8314] | 0.9010 [0.8815, 0.9206] | 0.2659 | 33.9 [33.5, 34.0] | 5.00 | 16.10 [16.07, 16.13] | 43.8 [43.5, 45.3] |
| bge-small-k20 | learned-own | 1.0000 [1.0000, 1.0000] | 0.1982 [0.1816, 0.2208] | 0.8018 [0.7792, 0.8184] | 0.8868 [0.8646, 0.9058] | 0.2790 | 39.2 [38.5, 39.4] | 4.98 | 16.15 [16.11, 16.18] | 40.1 [39.6, 41.6] |
| bge-small-k20 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3352 [0.3108, 0.3642] | 0.6648 [0.6358, 0.6892] | 0.7353 [0.7066, 0.7620] | 0.4148 | 35.7 [33.9, 36.6] | 3.90 | 15.92 [15.82, 16.02] | 75.1 [71.8, 79.4] |
| bge-small-k20 | openalex-topics-placed | 0.9750 [0.9680, 0.9820] | 0.6495 [0.6283, 0.6699] | 0.3505 [0.3301, 0.3717] | 0.3877 [0.3645, 0.4121] | 0.7555 | 70.7 [64.8, 70.4] | 4.43 | 21.22 [21.04, 21.38] | 76.8 [75.0, 79.7] |
| bge-small-k20 | unesco-thesaurus-placed | 0.9740 [0.9670, 0.9810] | 0.7265 [0.7055, 0.7481] | 0.2735 [0.2519, 0.2945] | 0.3025 [0.2782, 0.3263] | 0.8309 | 134.4 [123.3, 135.0] | 4.89 | 20.89 [20.71, 21.06] | 76.1 [74.3, 78.9] |
| bge-small-k20 | ipc-placed | 0.9815 [0.9750, 0.9870] | 0.7760 [0.7571, 0.7962] | 0.2240 [0.2038, 0.2429] | 0.2477 [0.2262, 0.2694] | 0.8793 | 302.9 [271.0, 299.2] | 6.33 | 20.65 [20.46, 20.83] | 76.2 [73.2, 80.1] |
| bge-small-k20 | ilc2-placed | 1.0000 [1.0000, 1.0000] | 0.8173 [0.7946, 0.8383] | 0.1827 [0.1617, 0.2054] | 0.2020 [0.1795, 0.2275] | 0.9117 | 262.6 [235.9, 258.0] | 5.05 | 18.87 [18.69, 19.06] | 52.7 [52.0, 54.6] |
| bge-small-k20 | lcc-full-placed | 0.9995 [0.9985, 1.0000] | 0.8245 [0.8076, 0.8423] | 0.1755 [0.1577, 0.1924] | 0.1941 [0.1743, 0.2127] | 0.9183 | 566.6 [487.1, 527.8] | 6.09 | 18.77 [18.58, 18.94] | 30.5 [30.6, 31.5] |
| bge-small-k20 | fdc-placed | 0.7730 [0.7545, 0.7925] | 0.8365 [0.8152, 0.8569] | 0.1635 [0.1431, 0.1848] | 0.1808 [0.1587, 0.2042] | 0.9363 | 52.9 [49.5, 54.1] | 3.20 | 19.95 [19.68, 20.23] | 292.2 [287.9, 297.2] |
| bge-small-k20 | eurovoc-placed | 0.9775 [0.9710, 0.9835] | 0.8458 [0.8266, 0.8639] | 0.1542 [0.1361, 0.1734] | 0.1705 [0.1505, 0.1922] | 0.9406 | 250.6 [227.4, 248.6] | 5.00 | 18.94 [18.78, 19.11] | 65.0 [64.1, 67.1] |
| bge-small-k20 | acm-ccs-placed | 0.7460 [0.7265, 0.7640] | 0.8488 [0.8296, 0.8684] | 0.1512 [0.1316, 0.1704] | 0.1672 [0.1466, 0.1894] | 0.9426 | 84.1 [76.1, 86.2] | 3.60 | 18.74 [18.49, 18.98] | 301.5 [298.9, 305.1] |
| bge-small-k20 | lcc-placed | 0.8755 [0.8615, 0.8905] | 0.8551 [0.8357, 0.8719] | 0.1449 [0.1281, 0.1643] | 0.1603 [0.1415, 0.1814] | 0.9546 | 120.1 [110.4, 122.1] | 4.04 | 19.90 [19.66, 20.14] | 183.9 [181.5, 187.4] |
| bge-small-k20 | bk-placed | 0.4825 [0.4605, 0.5050] | 0.9010 [0.8826, 0.9188] | 0.0990 [0.0812, 0.1174] | 0.1095 [0.0902, 0.1296] | 0.9856 | 71.4 [65.2, 73.5] | 3.17 | 16.93 [16.65, 17.23] | 551.9 [549.6, 555.6] |
| bge-small-k20 | arxiv-placed | 0.4185 [0.3970, 0.4405] | 0.9084 [0.8920, 0.9241] | 0.0916 [0.0759, 0.1080] | 0.1013 [0.0835, 0.1197] | 0.9904 | 13.7 [12.7, 14.4] | 2.19 | 16.41 [16.12, 16.69] | 661.1 [653.4, 670.0] |
| bge-small-k20 | jel-placed | 0.4780 [0.4540, 0.4990] | 0.9141 [0.8973, 0.9306] | 0.0859 [0.0694, 0.1027] | 0.0950 [0.0773, 0.1140] | 0.9928 | 28.5 [26.4, 29.8] | 2.28 | 15.74 [15.49, 15.96] | 584.8 [580.5, 590.1] |
| bge-small-k20 | msc-placed | 0.3870 [0.3665, 0.4075] | 0.9504 [0.9354, 0.9633] | 0.0496 [0.0367, 0.0646] | 0.0549 [0.0406, 0.0716] | 1.0213 | 112.9 [95.0, 113.1] | 1.99 | 14.18 [14.01, 14.34] | 652.1 [651.2, 654.2] |
| bge-small-k20 | msc-gold | 0.1370 [0.1225, 0.1525] | 0.9747 [0.9632, 0.9841] | 0.0253 [0.0159, 0.0368] | 0.0280 [0.0176, 0.0405] | 1.0372 | 268.5 [182.8, 216.9] | 1.41 | 12.52 [12.43, 12.62] | 888.7 [889.1, 889.3] |
| bge-small-k20 | jel-gold | 0.0000 [0.0000, 0.0000] | 1.0000 [1.0000, 1.0000] | 0.0000 [0.0000, 0.0000] | 0.0000 [0.0000, 0.0000] | 1.0598 | 1.0 [1.0, 1.0] | 1.00 | 11.97 [11.97, 11.97] | 1001.5 [1002.0, 1002.0] |
| bge-small-k50 | learned-bge-small-heldout | 1.0000 [1.0000, 1.0000] | 0.1757 [0.1635, 0.1896] | 0.8243 [0.8104, 0.8365] | 0.9641 [0.9501, 0.9788] | 0.2561 | 36.3 [35.6, 36.5] | 5.00 | 16.08 [16.05, 16.10] | 44.0 [43.5, 45.4] |
| bge-small-k50 | learned-nomic-heldout | 1.0000 [1.0000, 1.0000] | 0.1938 [0.1804, 0.2095] | 0.8062 [0.7905, 0.8196] | 0.9428 [0.9284, 0.9572] | 0.2748 | 39.8 [39.0, 40.0] | 4.97 | 16.19 [16.15, 16.22] | 40.8 [40.3, 42.5] |
| bge-small-k50 | learned-heldout | 1.0000 [1.0000, 1.0000] | 0.2030 [0.1887, 0.2180] | 0.7970 [0.7820, 0.8113] | 0.9321 [0.9167, 0.9470] | 0.2835 | 33.9 [33.5, 34.0] | 5.00 | 16.10 [16.07, 16.13] | 43.8 [43.5, 45.3] |
| bge-small-k50 | learned-own | 1.0000 [1.0000, 1.0000] | 0.2255 [0.2108, 0.2449] | 0.7745 [0.7551, 0.7892] | 0.9058 [0.8887, 0.9222] | 0.3063 | 39.2 [38.5, 39.4] | 4.98 | 16.15 [16.11, 16.18] | 40.1 [39.6, 41.6] |
| bge-small-k50 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3531 [0.3316, 0.3762] | 0.6469 [0.6238, 0.6684] | 0.7566 [0.7353, 0.7790] | 0.4327 | 35.7 [33.9, 36.6] | 3.90 | 15.92 [15.82, 16.02] | 75.1 [71.8, 79.4] |
| bge-small-k50 | openalex-topics-placed | 0.9750 [0.9680, 0.9820] | 0.6642 [0.6486, 0.6793] | 0.3358 [0.3207, 0.3514] | 0.3928 [0.3747, 0.4115] | 0.7702 | 70.7 [64.8, 70.4] | 4.43 | 21.22 [21.04, 21.38] | 76.8 [75.0, 79.7] |
| bge-small-k50 | unesco-thesaurus-placed | 0.9740 [0.9670, 0.9810] | 0.7458 [0.7318, 0.7621] | 0.2542 [0.2379, 0.2682] | 0.2973 [0.2781, 0.3149] | 0.8503 | 134.4 [123.3, 135.0] | 4.89 | 20.89 [20.71, 21.06] | 76.1 [74.3, 78.9] |
| bge-small-k50 | ipc-placed | 0.9815 [0.9750, 0.9870] | 0.7841 [0.7698, 0.7968] | 0.2159 [0.2032, 0.2302] | 0.2525 [0.2373, 0.2706] | 0.8874 | 302.9 [271.0, 299.2] | 6.33 | 20.65 [20.46, 20.83] | 76.2 [73.2, 80.1] |
| bge-small-k50 | ilc2-placed | 1.0000 [1.0000, 1.0000] | 0.8298 [0.8128, 0.8445] | 0.1702 [0.1555, 0.1872] | 0.1990 [0.1822, 0.2191] | 0.9242 | 262.6 [235.9, 258.0] | 5.05 | 18.87 [18.69, 19.06] | 52.7 [52.0, 54.6] |
| bge-small-k50 | lcc-full-placed | 0.9995 [0.9985, 1.0000] | 0.8372 [0.8242, 0.8503] | 0.1628 [0.1497, 0.1758] | 0.1904 [0.1756, 0.2058] | 0.9310 | 566.6 [487.1, 527.8] | 6.09 | 18.77 [18.58, 18.94] | 30.5 [30.6, 31.5] |
| bge-small-k50 | fdc-placed | 0.7730 [0.7545, 0.7925] | 0.8456 [0.8297, 0.8609] | 0.1544 [0.1391, 0.1703] | 0.1806 [0.1630, 0.1994] | 0.9454 | 52.9 [49.5, 54.1] | 3.20 | 19.95 [19.68, 20.23] | 292.2 [287.9, 297.2] |
| bge-small-k50 | acm-ccs-placed | 0.7460 [0.7265, 0.7640] | 0.8561 [0.8420, 0.8703] | 0.1439 [0.1297, 0.1580] | 0.1683 [0.1510, 0.1856] | 0.9498 | 84.1 [76.1, 86.2] | 3.60 | 18.74 [18.49, 18.98] | 301.5 [298.9, 305.1] |
| bge-small-k50 | eurovoc-placed | 0.9775 [0.9710, 0.9835] | 0.8615 [0.8476, 0.8745] | 0.1385 [0.1255, 0.1524] | 0.1620 [0.1472, 0.1792] | 0.9562 | 250.6 [227.4, 248.6] | 5.00 | 18.94 [18.78, 19.11] | 65.0 [64.1, 67.1] |
| bge-small-k50 | lcc-placed | 0.8755 [0.8615, 0.8905] | 0.8678 [0.8544, 0.8800] | 0.1322 [0.1200, 0.1456] | 0.1546 [0.1400, 0.1700] | 0.9673 | 120.1 [110.4, 122.1] | 4.04 | 19.90 [19.66, 20.14] | 183.9 [181.5, 187.4] |
| bge-small-k50 | arxiv-placed | 0.4185 [0.3970, 0.4405] | 0.9076 [0.8941, 0.9216] | 0.0924 [0.0784, 0.1059] | 0.1081 [0.0916, 0.1249] | 0.9896 | 13.7 [12.7, 14.4] | 2.19 | 16.41 [16.12, 16.69] | 661.1 [653.4, 670.0] |
| bge-small-k50 | bk-placed | 0.4825 [0.4605, 0.5050] | 0.9105 [0.8963, 0.9249] | 0.0895 [0.0751, 0.1037] | 0.1047 [0.0880, 0.1221] | 0.9951 | 71.4 [65.2, 73.5] | 3.17 | 16.93 [16.65, 17.23] | 551.9 [549.6, 555.6] |
| bge-small-k50 | jel-placed | 0.4780 [0.4540, 0.4990] | 0.9191 [0.9052, 0.9307] | 0.0809 [0.0693, 0.0948] | 0.0946 [0.0810, 0.1112] | 0.9978 | 28.5 [26.4, 29.8] | 2.28 | 15.74 [15.49, 15.96] | 584.8 [580.5, 590.1] |
| bge-small-k50 | msc-placed | 0.3870 [0.3665, 0.4075] | 0.9579 [0.9473, 0.9673] | 0.0421 [0.0327, 0.0527] | 0.0493 [0.0384, 0.0620] | 1.0287 | 112.9 [95.0, 113.1] | 1.99 | 14.18 [14.01, 14.34] | 652.1 [651.2, 654.2] |
| bge-small-k50 | msc-gold | 0.1370 [0.1225, 0.1525] | 0.9812 [0.9731, 0.9877] | 0.0188 [0.0123, 0.0269] | 0.0220 [0.0144, 0.0314] | 1.0438 | 268.5 [182.8, 216.9] | 1.41 | 12.52 [12.43, 12.62] | 888.7 [889.1, 889.3] |
| bge-small-k50 | jel-gold | 0.0000 [0.0000, 0.0000] | 1.0000 [1.0000, 1.0000] | -0.0000 [-0.0000, -0.0000] | -0.0000 [-0.0000, -0.0000] | 1.0598 | 1.0 [1.0, 1.0] | 1.00 | 11.97 [11.97, 11.97] | 1001.5 [1002.0, 1002.0] |
| nomic-k10 | learned-bge-small-heldout | 1.0000 [1.0000, 1.0000] | 0.1654 [0.1473, 0.1858] | 0.8346 [0.8142, 0.8527] | 0.8786 [0.8563, 0.8993] | 0.2458 | 36.3 [35.6, 36.5] | 5.00 | 16.08 [16.05, 16.10] | 44.0 [43.5, 45.4] |
| nomic-k10 | learned-nomic-heldout | 1.0000 [1.0000, 1.0000] | 0.1677 [0.1478, 0.1895] | 0.8323 [0.8105, 0.8522] | 0.8762 [0.8528, 0.8987] | 0.2486 | 39.8 [39.0, 40.0] | 4.97 | 16.19 [16.15, 16.22] | 40.8 [40.3, 42.5] |
| nomic-k10 | learned-own | 1.0000 [1.0000, 1.0000] | 0.1905 [0.1700, 0.2147] | 0.8095 [0.7853, 0.8300] | 0.8522 [0.8270, 0.8752] | 0.2712 | 39.2 [38.5, 39.4] | 4.98 | 16.15 [16.11, 16.18] | 40.1 [39.6, 41.6] |
| nomic-k10 | learned-heldout | 1.0000 [1.0000, 1.0000] | 0.1913 [0.1691, 0.2150] | 0.8087 [0.7850, 0.8309] | 0.8513 [0.8273, 0.8753] | 0.2719 | 33.9 [33.5, 34.0] | 5.00 | 16.10 [16.07, 16.13] | 43.8 [43.5, 45.3] |
| nomic-k10 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3458 [0.3181, 0.3786] | 0.6542 [0.6214, 0.6819] | 0.6886 [0.6550, 0.7179] | 0.4254 | 35.7 [33.9, 36.6] | 3.90 | 15.92 [15.82, 16.02] | 75.1 [71.8, 79.4] |
| nomic-k10 | openalex-topics-placed | 0.9750 [0.9680, 0.9820] | 0.6390 [0.6122, 0.6643] | 0.3610 [0.3357, 0.3878] | 0.3800 [0.3541, 0.4089] | 0.7451 | 70.7 [64.8, 70.4] | 4.43 | 21.22 [21.04, 21.38] | 76.8 [75.0, 79.7] |
| nomic-k10 | unesco-thesaurus-placed | 0.9740 [0.9670, 0.9810] | 0.7029 [0.6740, 0.7326] | 0.2971 [0.2674, 0.3260] | 0.3128 [0.2819, 0.3432] | 0.8073 | 134.4 [123.3, 135.0] | 4.89 | 20.89 [20.71, 21.06] | 76.1 [74.3, 78.9] |
| nomic-k10 | ipc-placed | 0.9815 [0.9750, 0.9870] | 0.7632 [0.7380, 0.7862] | 0.2368 [0.2138, 0.2620] | 0.2493 [0.2241, 0.2775] | 0.8664 | 302.9 [271.0, 299.2] | 6.33 | 20.65 [20.46, 20.83] | 76.2 [73.2, 80.1] |
| nomic-k10 | ilc2-placed | 1.0000 [1.0000, 1.0000] | 0.8131 [0.7848, 0.8396] | 0.1869 [0.1604, 0.2152] | 0.1968 [0.1687, 0.2264] | 0.9074 | 262.6 [235.9, 258.0] | 5.05 | 18.87 [18.69, 19.06] | 52.7 [52.0, 54.6] |
| nomic-k10 | lcc-full-placed | 0.9995 [0.9985, 1.0000] | 0.8144 [0.7903, 0.8387] | 0.1856 [0.1613, 0.2097] | 0.1953 [0.1701, 0.2212] | 0.9083 | 566.6 [487.1, 527.8] | 6.09 | 18.77 [18.58, 18.94] | 30.5 [30.6, 31.5] |
| nomic-k10 | eurovoc-placed | 0.9775 [0.9710, 0.9835] | 0.8302 [0.8053, 0.8566] | 0.1698 [0.1434, 0.1947] | 0.1787 [0.1511, 0.2052] | 0.9250 | 250.6 [227.4, 248.6] | 5.00 | 18.94 [18.78, 19.11] | 65.0 [64.1, 67.1] |
| nomic-k10 | fdc-placed | 0.7730 [0.7545, 0.7925] | 0.8316 [0.8048, 0.8563] | 0.1684 [0.1437, 0.1952] | 0.1773 [0.1514, 0.2062] | 0.9314 | 52.9 [49.5, 54.1] | 3.20 | 19.95 [19.68, 20.23] | 292.2 [287.9, 297.2] |
| nomic-k10 | lcc-placed | 0.8755 [0.8615, 0.8905] | 0.8391 [0.8132, 0.8625] | 0.1609 [0.1375, 0.1868] | 0.1694 [0.1443, 0.1969] | 0.9386 | 120.1 [110.4, 122.1] | 4.04 | 19.90 [19.66, 20.14] | 183.9 [181.5, 187.4] |
| nomic-k10 | acm-ccs-placed | 0.7460 [0.7265, 0.7640] | 0.8462 [0.8203, 0.8686] | 0.1538 [0.1314, 0.1797] | 0.1619 [0.1383, 0.1896] | 0.9399 | 84.1 [76.1, 86.2] | 3.60 | 18.74 [18.49, 18.98] | 301.5 [298.9, 305.1] |
| nomic-k10 | bk-placed | 0.4825 [0.4605, 0.5050] | 0.8844 [0.8567, 0.9101] | 0.1156 [0.0899, 0.1433] | 0.1217 [0.0948, 0.1508] | 0.9691 | 71.4 [65.2, 73.5] | 3.17 | 16.93 [16.65, 17.23] | 551.9 [549.6, 555.6] |
| nomic-k10 | arxiv-placed | 0.4185 [0.3970, 0.4405] | 0.8943 [0.8718, 0.9164] | 0.1057 [0.0836, 0.1282] | 0.1113 [0.0884, 0.1354] | 0.9764 | 13.7 [12.7, 14.4] | 2.19 | 16.41 [16.12, 16.69] | 661.1 [653.4, 670.0] |
| nomic-k10 | jel-placed | 0.4780 [0.4540, 0.4990] | 0.9001 [0.8762, 0.9222] | 0.0999 [0.0778, 0.1238] | 0.1052 [0.0818, 0.1311] | 0.9788 | 28.5 [26.4, 29.8] | 2.28 | 15.74 [15.49, 15.96] | 584.8 [580.5, 590.1] |
| nomic-k10 | msc-placed | 0.3870 [0.3665, 0.4075] | 0.9440 [0.9227, 0.9598] | 0.0560 [0.0402, 0.0773] | 0.0590 [0.0421, 0.0807] | 1.0149 | 112.9 [95.0, 113.1] | 1.99 | 14.18 [14.01, 14.34] | 652.1 [651.2, 654.2] |
| nomic-k10 | msc-gold | 0.1370 [0.1225, 0.1525] | 0.9850 [0.9740, 0.9927] | 0.0150 [0.0073, 0.0260] | 0.0158 [0.0077, 0.0275] | 1.0476 | 268.5 [182.8, 216.9] | 1.41 | 12.52 [12.43, 12.62] | 888.7 [889.1, 889.3] |
| nomic-k10 | jel-gold | 0.0000 [0.0000, 0.0000] | 1.0000 [1.0000, 1.0000] | -0.0000 [-0.0000, -0.0000] | -0.0000 [-0.0000, -0.0000] | 1.0598 | 1.0 [1.0, 1.0] | 1.00 | 11.97 [11.97, 11.97] | 1001.5 [1002.0, 1002.0] |
| nomic-k20 | learned-nomic-heldout | 1.0000 [1.0000, 1.0000] | 0.1793 [0.1642, 0.1977] | 0.8207 [0.8023, 0.8358] | 0.9178 [0.8991, 0.9366] | 0.2602 | 39.8 [39.0, 40.0] | 4.97 | 16.19 [16.15, 16.22] | 40.8 [40.3, 42.5] |
| nomic-k20 | learned-bge-small-heldout | 1.0000 [1.0000, 1.0000] | 0.1870 [0.1709, 0.2043] | 0.8130 [0.7957, 0.8291] | 0.9092 [0.8892, 0.9282] | 0.2674 | 36.3 [35.6, 36.5] | 5.00 | 16.08 [16.05, 16.10] | 44.0 [43.5, 45.4] |
| nomic-k20 | learned-heldout | 1.0000 [1.0000, 1.0000] | 0.2028 [0.1853, 0.2210] | 0.7972 [0.7790, 0.8147] | 0.8915 [0.8710, 0.9110] | 0.2833 | 33.9 [33.5, 34.0] | 5.00 | 16.10 [16.07, 16.13] | 43.8 [43.5, 45.3] |
| nomic-k20 | learned-own | 1.0000 [1.0000, 1.0000] | 0.2138 [0.1956, 0.2346] | 0.7862 [0.7654, 0.8044] | 0.8792 [0.8585, 0.8989] | 0.2946 | 39.2 [38.5, 39.4] | 4.98 | 16.15 [16.11, 16.18] | 40.1 [39.6, 41.6] |
| nomic-k20 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3525 [0.3292, 0.3784] | 0.6475 [0.6216, 0.6708] | 0.7241 [0.6947, 0.7483] | 0.4321 | 35.7 [33.9, 36.6] | 3.90 | 15.92 [15.82, 16.02] | 75.1 [71.8, 79.4] |
| nomic-k20 | openalex-topics-placed | 0.9750 [0.9680, 0.9820] | 0.6525 [0.6317, 0.6732] | 0.3475 [0.3268, 0.3683] | 0.3886 [0.3657, 0.4132] | 0.7586 | 70.7 [64.8, 70.4] | 4.43 | 21.22 [21.04, 21.38] | 76.8 [75.0, 79.7] |
| nomic-k20 | unesco-thesaurus-placed | 0.9740 [0.9670, 0.9810] | 0.7160 [0.6952, 0.7389] | 0.2840 [0.2611, 0.3048] | 0.3176 [0.2927, 0.3421] | 0.8205 | 134.4 [123.3, 135.0] | 4.89 | 20.89 [20.71, 21.06] | 76.1 [74.3, 78.9] |
| nomic-k20 | ipc-placed | 0.9815 [0.9750, 0.9870] | 0.7748 [0.7555, 0.7935] | 0.2252 [0.2065, 0.2445] | 0.2519 [0.2307, 0.2735] | 0.8780 | 302.9 [271.0, 299.2] | 6.33 | 20.65 [20.46, 20.83] | 76.2 [73.2, 80.1] |
| nomic-k20 | ilc2-placed | 1.0000 [1.0000, 1.0000] | 0.8212 [0.7997, 0.8412] | 0.1788 [0.1588, 0.2003] | 0.2000 [0.1772, 0.2238] | 0.9155 | 262.6 [235.9, 258.0] | 5.05 | 18.87 [18.69, 19.06] | 52.7 [52.0, 54.6] |
| nomic-k20 | lcc-full-placed | 0.9995 [0.9985, 1.0000] | 0.8266 [0.8088, 0.8438] | 0.1734 [0.1562, 0.1912] | 0.1939 [0.1747, 0.2144] | 0.9205 | 566.6 [487.1, 527.8] | 6.09 | 18.77 [18.58, 18.94] | 30.5 [30.6, 31.5] |
| nomic-k20 | fdc-placed | 0.7730 [0.7545, 0.7925] | 0.8347 [0.8138, 0.8539] | 0.1653 [0.1461, 0.1862] | 0.1849 [0.1634, 0.2083] | 0.9345 | 52.9 [49.5, 54.1] | 3.20 | 19.95 [19.68, 20.23] | 292.2 [287.9, 297.2] |
| nomic-k20 | eurovoc-placed | 0.9775 [0.9710, 0.9835] | 0.8427 [0.8243, 0.8627] | 0.1573 [0.1373, 0.1757] | 0.1759 [0.1542, 0.1968] | 0.9374 | 250.6 [227.4, 248.6] | 5.00 | 18.94 [18.78, 19.11] | 65.0 [64.1, 67.1] |
| nomic-k20 | lcc-placed | 0.8755 [0.8615, 0.8905] | 0.8486 [0.8275, 0.8674] | 0.1514 [0.1326, 0.1725] | 0.1693 [0.1480, 0.1935] | 0.9481 | 120.1 [110.4, 122.1] | 4.04 | 19.90 [19.66, 20.14] | 183.9 [181.5, 187.4] |
| nomic-k20 | acm-ccs-placed | 0.7460 [0.7265, 0.7640] | 0.8502 [0.8314, 0.8684] | 0.1498 [0.1316, 0.1686] | 0.1675 [0.1476, 0.1892] | 0.9439 | 84.1 [76.1, 86.2] | 3.60 | 18.74 [18.49, 18.98] | 301.5 [298.9, 305.1] |
| nomic-k20 | bk-placed | 0.4825 [0.4605, 0.5050] | 0.8972 [0.8768, 0.9173] | 0.1028 [0.0827, 0.1232] | 0.1150 [0.0926, 0.1380] | 0.9818 | 71.4 [65.2, 73.5] | 3.17 | 16.93 [16.65, 17.23] | 551.9 [549.6, 555.6] |
| nomic-k20 | arxiv-placed | 0.4185 [0.3970, 0.4405] | 0.8978 [0.8806, 0.9153] | 0.1022 [0.0847, 0.1194] | 0.1143 [0.0948, 0.1339] | 0.9798 | 13.7 [12.7, 14.4] | 2.19 | 16.41 [16.12, 16.69] | 661.1 [653.4, 670.0] |
| nomic-k20 | jel-placed | 0.4780 [0.4540, 0.4990] | 0.9077 [0.8888, 0.9249] | 0.0923 [0.0751, 0.1112] | 0.1032 [0.0845, 0.1244] | 0.9864 | 28.5 [26.4, 29.8] | 2.28 | 15.74 [15.49, 15.96] | 584.8 [580.5, 590.1] |
| nomic-k20 | msc-placed | 0.3870 [0.3665, 0.4075] | 0.9517 [0.9361, 0.9646] | 0.0483 [0.0354, 0.0639] | 0.0540 [0.0397, 0.0717] | 1.0226 | 112.9 [95.0, 113.1] | 1.99 | 14.18 [14.01, 14.34] | 652.1 [651.2, 654.2] |
| nomic-k20 | msc-gold | 0.1370 [0.1225, 0.1525] | 0.9846 [0.9759, 0.9910] | 0.0154 [0.0090, 0.0241] | 0.0172 [0.0101, 0.0271] | 1.0472 | 268.5 [182.8, 216.9] | 1.41 | 12.52 [12.43, 12.62] | 888.7 [889.1, 889.3] |
| nomic-k20 | jel-gold | 0.0000 [0.0000, 0.0000] | 1.0000 [1.0000, 1.0000] | 0.0000 [-0.0000, -0.0000] | 0.0000 [-0.0000, -0.0000] | 1.0598 | 1.0 [1.0, 1.0] | 1.00 | 11.97 [11.97, 11.97] | 1001.5 [1002.0, 1002.0] |
| nomic-k50 | learned-nomic-heldout | 1.0000 [1.0000, 1.0000] | 0.1999 [0.1865, 0.2155] | 0.8001 [0.7845, 0.8135] | 0.9531 [0.9388, 0.9680] | 0.2808 | 39.8 [39.0, 40.0] | 4.97 | 16.19 [16.15, 16.22] | 40.8 [40.3, 42.5] |
| nomic-k50 | learned-bge-small-heldout | 1.0000 [1.0000, 1.0000] | 0.2134 [0.1989, 0.2289] | 0.7866 [0.7711, 0.8011] | 0.9370 [0.9223, 0.9531] | 0.2938 | 36.3 [35.6, 36.5] | 5.00 | 16.08 [16.05, 16.10] | 44.0 [43.5, 45.4] |
| nomic-k50 | learned-heldout | 1.0000 [1.0000, 1.0000] | 0.2228 [0.2090, 0.2384] | 0.7772 [0.7616, 0.7910] | 0.9258 [0.9103, 0.9416] | 0.3034 | 33.9 [33.5, 34.0] | 5.00 | 16.10 [16.07, 16.13] | 43.8 [43.5, 45.3] |
| nomic-k50 | learned-own | 1.0000 [1.0000, 1.0000] | 0.2428 [0.2268, 0.2604] | 0.7572 [0.7396, 0.7732] | 0.9021 [0.8836, 0.9184] | 0.3235 | 39.2 [38.5, 39.4] | 4.98 | 16.15 [16.11, 16.18] | 40.1 [39.6, 41.6] |
| nomic-k50 | arxiv-gold | 1.0000 [1.0000, 1.0000] | 0.3730 [0.3528, 0.3961] | 0.6270 [0.6039, 0.6472] | 0.7469 [0.7229, 0.7690] | 0.4527 | 35.7 [33.9, 36.6] | 3.90 | 15.92 [15.82, 16.02] | 75.1 [71.8, 79.4] |
| nomic-k50 | openalex-topics-placed | 0.9750 [0.9680, 0.9820] | 0.6607 [0.6452, 0.6754] | 0.3393 [0.3246, 0.3548] | 0.4042 [0.3866, 0.4238] | 0.7668 | 70.7 [64.8, 70.4] | 4.43 | 21.22 [21.04, 21.38] | 76.8 [75.0, 79.7] |
| nomic-k50 | unesco-thesaurus-placed | 0.9740 [0.9670, 0.9810] | 0.7316 [0.7166, 0.7469] | 0.2684 [0.2531, 0.2834] | 0.3197 [0.3015, 0.3383] | 0.8360 | 134.4 [123.3, 135.0] | 4.89 | 20.89 [20.71, 21.06] | 76.1 [74.3, 78.9] |
| nomic-k50 | ipc-placed | 0.9815 [0.9750, 0.9870] | 0.7818 [0.7673, 0.7944] | 0.2182 [0.2056, 0.2327] | 0.2599 [0.2444, 0.2783] | 0.8851 | 302.9 [271.0, 299.2] | 6.33 | 20.65 [20.46, 20.83] | 76.2 [73.2, 80.1] |
| nomic-k50 | ilc2-placed | 1.0000 [1.0000, 1.0000] | 0.8329 [0.8170, 0.8461] | 0.1671 [0.1539, 0.1830] | 0.1990 [0.1830, 0.2181] | 0.9273 | 262.6 [235.9, 258.0] | 5.05 | 18.87 [18.69, 19.06] | 52.7 [52.0, 54.6] |
| nomic-k50 | lcc-full-placed | 0.9995 [0.9985, 1.0000] | 0.8411 [0.8284, 0.8544] | 0.1589 [0.1456, 0.1716] | 0.1893 [0.1733, 0.2047] | 0.9349 | 566.6 [487.1, 527.8] | 6.09 | 18.77 [18.58, 18.94] | 30.5 [30.6, 31.5] |
| nomic-k50 | fdc-placed | 0.7730 [0.7545, 0.7925] | 0.8441 [0.8271, 0.8590] | 0.1559 [0.1410, 0.1729] | 0.1857 [0.1682, 0.2064] | 0.9439 | 52.9 [49.5, 54.1] | 3.20 | 19.95 [19.68, 20.23] | 292.2 [287.9, 297.2] |
| nomic-k50 | acm-ccs-placed | 0.7460 [0.7265, 0.7640] | 0.8552 [0.8413, 0.8702] | 0.1448 [0.1298, 0.1587] | 0.1724 [0.1543, 0.1896] | 0.9489 | 84.1 [76.1, 86.2] | 3.60 | 18.74 [18.49, 18.98] | 301.5 [298.9, 305.1] |
| nomic-k50 | eurovoc-placed | 0.9775 [0.9710, 0.9835] | 0.8582 [0.8452, 0.8712] | 0.1418 [0.1288, 0.1548] | 0.1690 [0.1539, 0.1845] | 0.9529 | 250.6 [227.4, 248.6] | 5.00 | 18.94 [18.78, 19.11] | 65.0 [64.1, 67.1] |
| nomic-k50 | lcc-placed | 0.8755 [0.8615, 0.8905] | 0.8604 [0.8459, 0.8734] | 0.1396 [0.1266, 0.1541] | 0.1663 [0.1503, 0.1838] | 0.9599 | 120.1 [110.4, 122.1] | 4.04 | 19.90 [19.66, 20.14] | 183.9 [181.5, 187.4] |
| nomic-k50 | arxiv-placed | 0.4185 [0.3970, 0.4405] | 0.8979 [0.8818, 0.9137] | 0.1021 [0.0863, 0.1182] | 0.1216 [0.1022, 0.1406] | 0.9800 | 13.7 [12.7, 14.4] | 2.19 | 16.41 [16.12, 16.69] | 661.1 [653.4, 670.0] |
| nomic-k50 | bk-placed | 0.4825 [0.4605, 0.5050] | 0.9050 [0.8892, 0.9207] | 0.0950 [0.0793, 0.1108] | 0.1132 [0.0950, 0.1312] | 0.9896 | 71.4 [65.2, 73.5] | 3.17 | 16.93 [16.65, 17.23] | 551.9 [549.6, 555.6] |
| nomic-k50 | jel-placed | 0.4780 [0.4540, 0.4990] | 0.9114 [0.8952, 0.9251] | 0.0886 [0.0749, 0.1048] | 0.1056 [0.0890, 0.1248] | 0.9900 | 28.5 [26.4, 29.8] | 2.28 | 15.74 [15.49, 15.96] | 584.8 [580.5, 590.1] |
| nomic-k50 | msc-placed | 0.3870 [0.3665, 0.4075] | 0.9612 [0.9503, 0.9701] | 0.0388 [0.0299, 0.0497] | 0.0463 [0.0356, 0.0590] | 1.0320 | 112.9 [95.0, 113.1] | 1.99 | 14.18 [14.01, 14.34] | 652.1 [651.2, 654.2] |
| nomic-k50 | msc-gold | 0.1370 [0.1225, 0.1525] | 0.9867 [0.9809, 0.9915] | 0.0133 [0.0085, 0.0191] | 0.0158 [0.0101, 0.0228] | 1.0493 | 268.5 [182.8, 216.9] | 1.41 | 12.52 [12.43, 12.62] | 888.7 [889.1, 889.3] |
| nomic-k50 | jel-gold | 0.0000 [0.0000, 0.0000] | 1.0000 [1.0000, 1.0000] | 0.0000 [-0.0000, 0.0000] | 0.0000 [-0.0000, 0.0000] | 1.0598 | 1.0 [1.0, 1.0] | 1.00 | 11.97 [11.97, 11.97] | 1001.5 [1002.0, 1002.0] |

## Check against the published all-MiniLM-L6-v2 k = 20 results

45 of 46 rescored runs equal the published cost_norm exactly.

| run | published | source | rescored |
| --- | ---: | --- | ---: |
| split/msc-gold | 0.980498 | 2026-10-10-learned | 0.980538 |

