# Learned scheme benchmark results, 2026-10-10-learned

The learned scheme (`schemes/learned`, `scheme:learned@2026-10-10`) is scored
here against every Paper 1 scheme on the same corpus, holdout, graphs and
harness (`shelfmark-bench`) as `../2026-10-10/` and `../2026-10-10-r2/`.
Unlike the Paper 1 schemes, it was fitted to this corpus, by minimizing
the quantities the harness reports.

**Read the learned rows carefully.** Every learned run is gold-free: there
are no source labels for it, so it is scored on placements it made itself.
Its own placements of the 10,000-paper corpus are **in-sample** (the corpus
is what it was fitted to). The fair tests are the **held-out 80/20 split**
(fitted on 80%, the other 20% filed as new works) and the **holdout**
(500 Open Library books filed as new works). The shelfmark-placed learned
rows go through the same lexical classifier as the Paper 1 placed rows, but
their vocabulary (c-TF-IDF terms) was mined from the full corpus.

- Corpus: 10,000 arXiv papers, `corpus.jsonl` sha256 `7685685d...27b8d`; holdout: 500 Open Library editions, `holdout.jsonl` sha256 `3e057eb4...c873c833`.
- Graphs: the published 2026-10-10 k = 20 all-MiniLM-L6-v2 graphs. Average-linkage baselines: corpus 0.1518, holdout 0.3014, and 0.1024 for the induced graph of the 20% test split.
- Paper 1 placements were regenerated with `profile:lexical@2`. All 13 scheme files hash as in 2026-10-10, and all 19 placed and 5 gold scores equal the published ones (`results.json`, `baselines.paper1_rerun_matches_published`).
- Commands: `REPRODUCE.md`.

## Objective

`J = cost_norm + 0.005 * nav_log_a1`: normalized Dasgupta cost with works
under their class (the harness's `cost_norm`), plus 0.005 times the expected
navigation cost in bits, `sum (1 + log2 fan-out)` from the root to a work
(the harness's `nav_log_a1`; a leaf class's fan-out is its number of works).
The learned scheme minimizes J under 2-12 child classes, at most 4 class
levels, leaf classes of at least 5 works, and classes that are contiguous
runs of a fixed shelf order. The tables report J for every scheme, computed
from the harness's own two numbers. Lower is better throughout. `skill` is
`(1 - cost_norm) / (1 - ref)` against the average-linkage tree of the same
graph. J is not defined for the binary average-linkage tree, which is not a
usable classification: every work is its own class.

Fit (`build-report.json`): average linkage reaches cost 0.1518. Rotation
local search makes no move, because average linkage is reducible. The exact
DP collapse gives J 0.26473 with 129 classes. Order-preserving local search
(88 boundary shifts, 20 structure moves) gives J 0.26440: cost_norm 0.1722,
nav 18.45, 110 classes (2 / 5 / 16 / 87 by level) and 88 leaf classes of
38-345 works. The fit takes about 2.5 s.

## Corpus (n = 10,000)

| scheme | placements | coverage | cost_norm | skill | J | nav log a=1 | classes used | mean depth | max branching |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned | own, in-sample (fitted on this corpus) | 1.0000 | 0.1722 | 0.9760 | 0.2644 | 18.45 | 39.7 | 4.98 | 345 |
| arxiv | gold (source labels) | 1.0000 | 0.3333 | 0.7861 | 0.4250 | 18.33 | 36.5 | 3.90 | 797 |
| learned | shelfmark lexical@2 | 0.9993 | 0.3794 | 0.7317 | 0.5347 | 31.08 | 33.5 | 4.38 | 1183 |
| openalex-topics | shelfmark lexical@2 | 0.9732 | 0.6414 | 0.4228 | 0.7757 | 26.85 | 77.6 | 4.45 | 554 |
| unesco-thesaurus | shelfmark lexical@2 | 0.9731 | 0.7210 | 0.3289 | 0.8544 | 26.68 | 146.7 | 4.90 | 535 |
| ipc | shelfmark lexical@2 | 0.9799 | 0.7690 | 0.2724 | 0.9056 | 27.31 | 317.8 | 6.31 | 794 |
| ilc2 | shelfmark lexical@2 | 0.9999 | 0.8189 | 0.2135 | 0.9358 | 23.38 | 295.1 | 5.09 | 417 |
| lcc-full | shelfmark lexical@2 | 0.9999 | 0.8228 | 0.2089 | 0.9372 | 22.86 | 657.2 | 6.06 | 121 |
| acm-ccs | shelfmark lexical@2 | 0.7445 | 0.8306 | 0.1997 | 0.9493 | 23.73 | 84.9 | 3.61 | 877 |
| fdc | shelfmark lexical@2 | 0.7696 | 0.8337 | 0.1961 | 0.9615 | 25.56 | 54.5 | 3.21 | 1103 |
| eurovoc | shelfmark lexical@2 | 0.9737 | 0.8479 | 0.1794 | 0.9718 | 24.80 | 269.3 | 4.98 | 350 |
| lcc | shelfmark lexical@2 | 0.8765 | 0.8487 | 0.1783 | 0.9755 | 25.36 | 129.0 | 4.08 | 661 |
| arxiv | shelfmark lexical@2 | 0.4260 | 0.8912 | 0.1282 | 0.9929 | 20.34 | 14.5 | 2.21 | 1665 |
| jel | shelfmark lexical@2 | 0.4786 | 0.9028 | 0.1146 | 0.9998 | 19.41 | 28.9 | 2.27 | 972 |
| bk | shelfmark lexical@2 | 0.4806 | 0.9019 | 0.1157 | 1.0066 | 20.94 | 75.0 | 3.15 | 788 |
| msc | shelfmark lexical@2 | 0.3916 | 0.9498 | 0.0592 | 1.0389 | 17.82 | 114.4 | 2.00 | 380 |
| msc | gold (source labels) | 0.1314 | 0.9844 | 0.0183 | 1.0605 | 15.20 | 461.3 | 1.39 | 25 |
| jel | gold (source labels) | 0.0002 | 1.0000 | 0.0000 | 1.0714 | 14.29 | 6.0 | 1.00 | 1 |
| average linkage (binary reference, not a classification) | - | 1.0000 | 0.1518 | 1.0000 | - | - | - | - | 2 |

The learned scheme's own placements (in-sample) reach 97.6% of the
average-linkage tree's revenue with 110 named classes and four levels.
Reached through shelfmark's lexical classifier, it scores cost 0.379; the
best Paper 1 placed scheme (OpenAlex Topics) scores 0.641. The best gold run,
arXiv's own author-chosen categories, scores 0.333. Two effects explain why
the lexical classifier does so well here. The classes are cut along the
graph's own clusters, and their vocabulary is the corpus's own most
distinctive terms. The second effect is in-sample, so this row is not a
held-out result.

## Held-out split (fitted on 80%, test on 20%, n = 2,000)

The split ranks ids by SHA-256 with seed `learned-split-2026-10-10`; the test
ids are in `split/test-ids.txt`. The 80% fit uses the same parameters with
automatic labels (`split/learned-split80.json`: 86 classes). The 20% are then
inserted as new works by nearest leaf centroid. No leaf doubled and no new
work formed a distinct group, so no split happened and the 86 classes stayed
as fitted. Everything is scored on the graph induced on the 2,000 test works.
Every Paper 1 scheme is restricted to the same works.

| scheme | placements | coverage | cost_norm | skill | J | nav log a=1 | classes used | mean depth | max branching |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned (fit on the other 80%) | own insertion, held out | 1.0000 | 0.1722 | 0.9222 | 0.2528 | 16.10 | 33.9 | 5.00 | 69 |
| learned (full fit) | own, in-sample | 1.0000 | 0.1793 | 0.9144 | 0.2600 | 16.15 | 39.2 | 4.98 | 71 |
| arxiv | gold (source labels) | 1.0000 | 0.3424 | 0.7327 | 0.4220 | 15.92 | 35.7 | 3.90 | 167 |
| learned (full fit) | shelfmark lexical@2 | 0.9995 | 0.3798 | 0.6910 | 0.4946 | 22.98 | 33.0 | 4.39 | 238 |
| openalex-topics | shelfmark lexical@2 | 0.9750 | 0.6519 | 0.3878 | 0.7580 | 21.22 | 70.7 | 4.43 | 119 |
| unesco-thesaurus | shelfmark lexical@2 | 0.9740 | 0.7192 | 0.3129 | 0.8236 | 20.89 | 134.4 | 4.89 | 127 |
| ipc | shelfmark lexical@2 | 0.9815 | 0.7657 | 0.2611 | 0.8689 | 20.65 | 302.9 | 6.33 | 152 |
| ilc2 | shelfmark lexical@2 | 1.0000 | 0.8196 | 0.2010 | 0.9139 | 18.87 | 262.6 | 5.05 | 76 |
| lcc-full | shelfmark lexical@2 | 0.9995 | 0.8325 | 0.1866 | 0.9264 | 18.77 | 566.6 | 6.09 | 30 |
| acm-ccs | shelfmark lexical@2 | 0.7460 | 0.8346 | 0.1843 | 0.9283 | 18.74 | 84.1 | 3.60 | 164 |
| fdc | shelfmark lexical@2 | 0.7730 | 0.8328 | 0.1863 | 0.9326 | 19.95 | 52.9 | 3.20 | 234 |
| eurovoc | shelfmark lexical@2 | 0.9775 | 0.8411 | 0.1770 | 0.9359 | 18.94 | 250.6 | 5.00 | 75 |
| lcc | shelfmark lexical@2 | 0.8755 | 0.8547 | 0.1619 | 0.9542 | 19.90 | 120.1 | 4.04 | 156 |
| arxiv | shelfmark lexical@2 | 0.4185 | 0.8955 | 0.1164 | 0.9776 | 16.41 | 13.7 | 2.19 | 340 |
| bk | shelfmark lexical@2 | 0.4825 | 0.8959 | 0.1159 | 0.9806 | 16.93 | 71.4 | 3.17 | 158 |
| jel | shelfmark lexical@2 | 0.4780 | 0.9029 | 0.1082 | 0.9815 | 15.74 | 28.5 | 2.28 | 207 |
| msc | shelfmark lexical@2 | 0.3870 | 0.9552 | 0.0499 | 1.0261 | 14.18 | 112.9 | 1.99 | 79 |
| msc | gold (source labels) | 0.1360 | 0.9805 | 0.0217 | 1.0431 | 12.51 | 269.7 | 1.41 | 11 |
| jel | gold (source labels) | 0.0000 | 1.0000 | -0.0000 | 1.0598 | 11.97 | 1.0 | 1.00 | 0 |

| scheme | placements | coverage | cost_norm | skill | J | nav log a=1 | classes used | mean depth | max branching |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned (fit on 80%) | own, in-sample, 80% graph | 1.0000 | 0.1665 | 0.9777 | 0.2570 | 18.11 | 33.7 | 5.00 | 259 |
| learned (fit on 80%) | 80% fitted + 20% inserted, whole corpus graph | 1.0000 | 0.1720 | 0.9763 | 0.2641 | 18.42 | 33.8 | 5.00 | 328 |
| learned (full fit) | own, in-sample, whole corpus graph | 1.0000 | 0.1722 | 0.9760 | 0.2644 | 18.45 | 39.7 | 4.98 | 345 |

Held out, the learned scheme scores cost 0.1722 on the test works. The full
fit, which saw them, scores 0.1793 on the same works, so there is no sign of
overfitting. On the whole corpus graph, the 80% fit with the 20% inserted
(cost 0.1720, J 0.2641) is as good as the full fit (0.1722, J 0.2644).
Caveat: insertion uses the same embedding model as the graph (nearest
centroid in all-MiniLM-L6-v2 space), so new works are filed by the geometry
the graph is built from. The Paper 1 placed rows use shelfmark's lexical
classifier and have no such advantage. The arXiv gold row has none either:
it scores 0.342 against the learned scheme's 0.172.

## Holdout (n = 500, Open Library books, titles only)

| scheme | placements | coverage | cost_norm | skill | J | nav log a=1 | classes used | mean depth | max branching |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned | own insertion (embedding nearest centroid) | 1.0000 | 0.4432 | 0.7971 | 0.5416 | 19.69 | 15.1 | 8.79 | 93 |
| lcc | gold (source labels) | 0.9220 | 0.7076 | 0.4185 | 0.8006 | 18.59 | 35.0 | 4.20 | 89 |
| lcc-full | gold (source labels) | 0.9220 | 0.7057 | 0.4212 | 0.8043 | 19.71 | 85.2 | 5.19 | 68 |
| lcc-full | shelfmark lexical@2 | 0.5660 | 0.9250 | 0.1074 | 0.9949 | 13.98 | 340.8 | 4.14 | 13 |
| ilc2 | shelfmark lexical@2 | 0.4560 | 0.9543 | 0.0655 | 1.0166 | 12.46 | 188.1 | 2.94 | 25 |
| unesco-thesaurus | shelfmark lexical@2 | 0.1580 | 0.9888 | 0.0161 | 1.0429 | 10.82 | 93.8 | 1.65 | 8 |
| learned | shelfmark lexical@2 | 0.0860 | 0.9915 | 0.0122 | 1.0443 | 10.57 | 26.1 | 1.34 | 5 |
| lcc | shelfmark lexical@2 | 0.0840 | 0.9964 | 0.0052 | 1.0474 | 10.21 | 63.0 | 1.29 | 6 |
| fdc | shelfmark lexical@2 | 0.0520 | 0.9981 | 0.0027 | 1.0486 | 10.10 | 31.1 | 1.14 | 4 |
| bk | shelfmark lexical@2 | 0.0440 | 0.9985 | 0.0021 | 1.0492 | 10.14 | 30.8 | 1.23 | 7 |
| average linkage (binary reference, not a classification) | - | 1.0000 | 0.3014 | 1.0000 | - | - | - | - | 2 |

The 500 books were inserted as new works (`learned-holdout.json`,
`insert-report.json`): 110 classes became 126 through 8 leaf splits, and
every existing code, label and parent link was unchanged (checked on the run
and by `schemes/learned/tests/insertion.rs`). The scheme was fitted on arXiv
only, so most books first land in one fitted leaf, 01.02.01.04 "Image
generation and editing", the nearest centroid to short book titles. Two
splits for distinct groups then separate the books from that class's
papers. Five capacity splits (principal-direction bisection once a class
created by insertion reaches 98 works, the median fitted leaf) cut them into
book classes of 48-93 titles. The labels of inserted classes come from title
words ("Harry / potter / adventures", "Edition / holmes / sherlock") and are
weak. On the holdout graph, insertion scores cost 0.443 (skill 0.797), better
than the librarians' LC call numbers (gold lcc 0.708, lcc-full 0.706). The
same embedding-circularity caveat applies, and more strongly: the holdout
graph and the insertion both see only the title embeddings. Through
shelfmark's lexical classifier the learned scheme places only 43 of the 500
books (8.6%; vocabulary mined from arXiv abstracts rarely matches book
titles) and scores 0.992, like most Paper 1 schemes on this holdout.

## Sensitivity (per-branch cost a = 1)

Fits of the full corpus with automatic labels (`results.json`, `sweep`, which
also has a = 2). The committed scheme is depth bound 4, lambda 0.005.

| depth bound | lambda | J | cost_norm | nav (a=1) | classes | leaf classes | root children | mean class depth | median leaf |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 3 | 0.001 | 0.2049 | 0.1875 | 17.46 | 59 | 50 | 2 | 3.00 | 174 |
| 3 | 0.002 | 0.2224 | 0.1875 | 17.46 | 59 | 50 | 2 | 3.00 | 174 |
| 3 | 0.005 | 0.2747 | 0.1877 | 17.41 | 54 | 45 | 2 | 3.00 | 226 |
| 3 | 0.01 | 0.3619 | 0.1879 | 17.40 | 51 | 42 | 2 | 3.00 | 247 |
| 3 | 0.02 | 0.5361 | 0.1881 | 17.40 | 49 | 40 | 2 | 3.00 | 262 |
| 3 | 0.05 | 1.0513 | 0.2075 | 16.88 | 33 | 28 | 3 | 2.37 | 307 |
| 4 | 0.001 | 0.1902 | 0.1717 | 18.50 | 125 | 100 | 2 | 4.00 | 81 |
| 4 | 0.002 | 0.2087 | 0.1717 | 18.49 | 120 | 95 | 2 | 4.00 | 87 |
| 4 | 0.005 | 0.2644 | 0.1722 | 18.45 | 110 | 88 | 2 | 3.98 | 98 |
| 4 | 0.01 | 0.3565 | 0.1731 | 18.33 | 92 | 73 | 2 | 3.88 | 121 |
| 4 | 0.02 | 0.5359 | 0.1860 | 17.50 | 52 | 42 | 2 | 3.08 | 243 |
| 4 | 0.05 | 1.0513 | 0.2075 | 16.88 | 33 | 28 | 3 | 2.37 | 307 |
| 6 | 0.001 | 0.1783 | 0.1577 | 20.68 | 330 | 248 | 2 | 5.92 | 31 |
| 6 | 0.002 | 0.1990 | 0.1581 | 20.44 | 271 | 204 | 2 | 5.72 | 45 |
| 6 | 0.005 | 0.2594 | 0.1600 | 19.90 | 207 | 161 | 2 | 5.26 | 56 |
| 6 | 0.01 | 0.3550 | 0.1675 | 18.76 | 101 | 77 | 2 | 4.37 | 119 |
| 6 | 0.02 | 0.5359 | 0.1860 | 17.50 | 52 | 42 | 2 | 3.08 | 243 |
| 6 | 0.05 | 1.0513 | 0.2075 | 16.88 | 33 | 28 | 3 | 2.37 | 307 |

The root has two children in all but the flattest settings: Dasgupta cost
prefers a binary split near the top, where most cross-cluster weight sits,
and the navigation term only charges per level. Raising lambda trades cost
for fewer levels and classes. The depth bound binds from lambda 0.01 down.

## Caveats

- In-sample: the learned scheme's own corpus placements and its
  shelfmark-placed corpus and 20% rows use a tree or vocabulary fitted on
  those works. Only the held-out split and the holdout file new works.
- Embedding circularity: insertion and the graphs use the same embedding
  model (see DESIGN.md finding 6 on circularity). The lexical-classifier rows
  are the comparison without it.
- Scope: one corpus of arXiv papers. The scheme has no humanities until
  insertion makes some, and its top level splits computing from mathematics
  and physical sciences.
- Labels: the 110 fitted classes were labelled by hand from c-TF-IDF terms
  and central titles (`schemes/learned/labels.tsv`). Classes created by
  insertion get automatic labels.

## Runtime

- Fit: about 2.5 s (average linkage 0.9 s, local search 1.3 s); 36-fit sweep about 4 min.
- Paper 1 placement rerun: about 25 min on 8 CPUs (lcc-full corpus about 16 min of it).
- Learned runs: about 3 min, most of it the shelfmark lexical placement of the corpus.
