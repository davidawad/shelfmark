# Covered-subset scores, coverage analysis and dense-vs-sparse cost, 2026-10-10

This run rescores the 28 runs of `../2026-10-10-scores/` and adds four
analyses:

1. **Covered-subset gold scoring.** Each gold run's labelled items form a
   subset: arXiv (all 10,000 papers), MSC (1,322 papers, 13%), LCC and
   LCC-full (461 of 500 books). JEL labels 2 papers and is reported by
   size only. Every run of the corpus (gold, shelfmark-placed and learned)
   is rescored on each subset, keeping only the subset's items and the
   graph edges between them. Each gold run is also scored against its
   placed counterpart on the items *both* place, so neither run leaves an
   item at the root.
2. **Every score on all items and on each run's own covered subset**, and
   how much of each score's ranking coverage explains (Spearman ρ with
   coverage, ρ², Kendall τ between the all-items and covered rankings, and
   partial correlations on coverage).
3. **Dense against kNN-sparse Dasgupta cost** (DESIGN.md section 1) on
   five random subsets each of 1,000, 2,000 and 5,000 papers and on the
   whole book holdout.
4. **Bootstrap intervals for all of the above.** There are two bootstraps:
   an edge bootstrap and a new vertex (item) bootstrap. Paired-bootstrap
   differences between runs on the same items.

`REPRODUCE.md` gives the commands. `results.json` holds every score for
every run, on all items and on its covered subset. It also holds every
gold-subset run, each analysis and the dense check.

**Inputs are the published ones.** The placements were regenerated with
the r2 commands. All 13 scheme files hashed as published, and all 28
harness results reproduced the published tree SHA-256 and cost_norm.
Both embedding vector files have the published SHA-256. On all items,
every point score matches `../2026-10-10-scores/results.json` exactly:
2,648 scores, plus every null-model and noise value. Latency is wall-clock
and differs. The bootstrap now draws edges, items and kNN-probe items from
separate streams. Two runs on the same graph and items therefore share
every resample, which is what makes the paired differences valid. The
intervals differ slightly from the scores run as a result.

**Caveats.**
- Restricting the graph removes edges. The MSC subset keeps 7,527 of
  148,111 edges, about 11 per item against 30 on all papers. The LCC
  subset keeps 6,344 of 7,297. The dense check (section 5) shows that the
  graph restricted to a random subset ranks schemes as well as a kNN
  graph rebuilt on that subset.
- A run's *own* covered subset differs from run to run. Its covered scores
  remove the root penalty but compare different items. The gold-subset
  tables (section 1) compare runs on the same items and are paired.
- The book-holdout placed runs place few books: BK 22, FDC 26, LCC 42 and
  the learned scheme 43. On the LCC subset most placed runs still have
  coverage of 0.05–0.16. The LCC gold-vs-placed comparison on shared
  items has only 37 books and 64 edges. Its intervals span ±0.25 and
  nothing in it is significant.
- The 95% intervals are percentile intervals from 200 replicates. Scores
  with a resampling bias (TSD, dendrogram purity, partial correlations)
  can have an interval that does not contain the point value. With 200
  replicates the smallest reportable p is 0.005. No multiplicity
  correction is applied.
- The gold group has only 6 runs and mixes the two corpora, so its
  correlations are indicative only.

## Conclusions

**Gold scores mostly measured coverage.** On all papers MSC gold scores
*worse* than shelfmark's MSC placements: cost_norm 0.984 against 0.950.
Restricted to the 1,322 MSC-labelled papers, MSC gold scores 0.679 and
msc-placed 0.936 (Δ −0.26, vertex 95% CI [−0.29, −0.22]). That subset
still charges msc-placed for the 54% of those papers it leaves at the
root. On the 606 papers both place, the gap shrinks to Δ −0.056
[−0.117, −0.003] (p = 0.04): TSD +0.15 (p < 0.005), triplet +0.025 (not
significant). Gold MSC is a better hierarchy for those papers, but only
slightly. The same holds for arXiv: −0.56 on all papers, −0.23 on the
4,260 papers both place. LCC-full gold beats LCC-full placements on 257
shared books by −0.11 [−0.18, −0.04]. LCC on 37 shared books is
undecided. On the MSC subset, arXiv gold beats MSC gold by a wide margin
(cost 0.507 against 0.679; Δ +0.17 [+0.15, +0.20]). The learned scheme
leads every subset, but it is in-sample. JEL cannot be scored (2 papers).

**How much of each ranking is coverage** (14 placed runs on the papers):
- Coverage explains 59% of the rank variance of Dasgupta cost_norm
  (ρ = −0.77), 54% of TSD's and 38% of triplet accuracy's. The ranking by
  cost on all items and by cost on each run's own covered items barely
  agree (τ = +0.10). The same holds for triplet (τ = +0.01) and TSD
  (+0.16). Low-coverage schemes (MSC, arXiv, BK and JEL placements, about
  40–48% coverage) move from the bottom to the middle once their unplaced
  papers are dropped. MSC-placed goes from 0.950 to 0.726.
- V-measure (ρ² = 0.80) and the warrant share (0.83) are the most
  coverage-bound. AMI (0.31) and hierarchical F (0.25) are less so.
- Shelf locality, its Kendall τ, the kNN and centroid probes,
  Davies–Bouldin, author collocation and the notation and hospitality
  scores keep their ranking (τ(all, covered) = 1.00). Silhouette and
  scheme bits come close (0.91). These scores read only placed items, so
  coverage cannot enter them. MDL (0.82), cophenetic correlation (0.67)
  and gap bits (0.67) move somewhat.
- The graph-fit block survives the control. Partial on coverage,
  cost_norm still agrees with triplet (raw ρ −0.85, partial −0.74
  [−0.75, −0.65]), AMI (−0.77 → −0.64), dendrogram purity (−0.53 → −0.73)
  and TSD (−0.79 → −0.53). Part of the block is real nesting quality, not
  only coverage. The kNN probe, close to independent of cost in raw terms
  (−0.23), agrees with it once coverage is held fixed (−0.80).
- On the book holdout, coverage *is* the cost ranking: ρ(cost_norm,
  coverage) = −1.00 over the 7 placed runs, so the partial correlations
  are undefined. TSD, triplet, dendrogram purity and V-measure follow
  coverage with ρ ≥ 0.89. No book-holdout ranking of placed schemes by a
  graph score says anything beyond coverage.

**Bootstrap intervals.** Edges that share an item are not independent.
The vertex bootstrap (resample items) gives intervals 3.8 times wider
than the edge bootstrap on the papers and 3.1 times wider on the books,
for cost; triplet, TSD and locality widen similarly. Most scheme pairs
still differ significantly on the papers: 86 of 91 placed pairs on cost
(vertex), 80 on triplet and 77 on locality. On the 500-book holdout,
16 of 21 placed pairs differ on cost (vertex; 19 with the edge
bootstrap). The ones that do not: UNESCO vs learned, learned vs LCC,
LCC vs FDC, LCC vs BK and FDC vs BK, all placing under 16% of the books.
No pair differs on locality (0 of 21, interval width 0.54), and 9 of 21
differ on triplet. On the holdout only the cost ranking is resolved, and
mostly through coverage.

**Dense and sparse cost rank schemes alike.** Over the 14 placed runs,
dense and kNN rankings agree with Spearman ρ = 0.95–0.97 and Kendall
τ = 0.82–0.91 at n = 1,000, 2,000 and 5,000 for k = 10, 20 and 50 (all
18 runs: ρ ≥ 0.96). Across the five random subsets, ρ varies by at most
0.06. More neighbours help a little at n = 1,000 (k = 10: ρ 0.956,
k = 50: 0.974) and make no difference at 5,000. The published k = 20
graph restricted to a subset agrees as well as a rebuilt kNN graph
(n = 5,000: ρ 0.965). At n = 1,000 it has under 1.5 edges per item and
falls to ρ 0.91. The swaps are local and repeat across sizes:
IPC/UNESCO, and the middle block of ACM CCS, ILC2, LCC-full and FDC. The
dense sum adds the many weakly similar pairs that meet near the root, so
it shifts and compresses every cost (arXiv gold 0.33 sparse, 0.57 dense)
and reorders schemes whose fit differs only at the broadest level. On the
books (n = 500) only BK and FDC swap (ρ 0.964 [0.89, 1.0]). The kNN
sparsification behind every published cost does not change the scheme
ranking beyond such adjacent swaps. Check of the construction: k = 20 on
all 10,000 papers rebuilds the published graph's 148,111 edges, and
cost_norm is within 6·10⁻⁶ of the harness.

## 1. Gold scores on the covered subset

Each gold run defines a subset: the items its source labels place in the scheme. Every run on that corpus (gold, shelfmark-placed and learned) is rescored restricted to the subset: items outside it are dropped, and so are the graph edges that touch them. Inside the subset a gold run covers every item, so its scores no longer measure coverage, and all runs share one graph and one item set: their bootstrap replicates are paired. "coverage" below is the share of subset items the run places.

### arxiv-gold subset (10000 of 10000 items)

Graph restricted to the subset: 148111 edges.

| run | coverage | cost_norm [edge CI] | [vertex CI] | TSD [vertex CI] | triplet [vertex CI] | locality [vertex CI] | kNN d1 | AMI fine | DP fine [CI] |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-own | 1.000 | 0.172 [0.171, 0.173] | [+0.166, +0.177] | 1.802 [1.767, 1.846] | 0.873 [0.869, 0.879] | 0.793 [0.786, 0.803] | 0.968 | 0.451 | 0.225 [0.215, 0.238] |
| arxiv-gold | 1.000 | 0.333 [0.331, 0.335] | [+0.323, +0.342] | 1.095 [1.061, 1.131] | 0.798 [0.793, 0.805] | 0.606 [0.593, 0.622] | 0.858 | 1.000 | 1.000 [1.000, 1.000] |
| learned-placed | 0.999 | 0.379 [0.378, 0.381] | [+0.372, +0.386] | 0.677 [0.653, 0.709] | 0.750 [0.745, 0.756] | 0.566 [0.557, 0.576] | 0.905 | 0.306 | 0.132 [0.128, 0.140] |
| openalex-topics-placed | 0.973 | 0.641 [0.640, 0.643] | [+0.633, +0.649] | 0.255 [0.243, 0.278] | 0.576 [0.569, 0.583] | 0.204 [0.190, 0.219] | 0.761 | 0.188 | 0.073 [0.071, 0.079] |
| unesco-thesaurus-placed | 0.973 | 0.721 [0.719, 0.723] | [+0.715, +0.727] | 0.179 [0.170, 0.197] | 0.559 [0.554, 0.563] | 0.116 [0.104, 0.129] | 0.551 | 0.157 | 0.064 [0.062, 0.068] |
| ipc-placed | 0.980 | 0.769 [0.767, 0.771] | [+0.763, +0.775] | 0.082 [0.077, 0.095] | 0.533 [0.529, 0.537] | 0.061 [0.047, 0.073] | 0.415 | 0.118 | 0.059 [0.058, 0.063] |
| ilc2-placed | 1.000 | 0.819 [0.817, 0.821] | [+0.813, +0.825] | 0.108 [0.102, 0.123] | 0.550 [0.547, 0.553] | 0.163 [0.147, 0.178] | 0.324 | 0.102 | 0.059 [0.059, 0.064] |
| lcc-full-placed | 1.000 | 0.823 [0.821, 0.825] | [+0.816, +0.829] | 0.114 [0.109, 0.131] | 0.533 [0.530, 0.537] | 0.079 [0.067, 0.093] | 0.339 | 0.108 | 0.058 [0.057, 0.063] |
| acm-ccs-placed | 0.745 | 0.831 [0.829, 0.832] | [+0.824, +0.838] | 0.091 [0.085, 0.102] | 0.545 [0.542, 0.549] | 0.096 [0.080, 0.110] | 0.437 | 0.127 | 0.066 [0.064, 0.071] |
| fdc-placed | 0.770 | 0.834 [0.832, 0.835] | [+0.827, +0.842] | 0.092 [0.082, 0.104] | 0.535 [0.532, 0.539] | 0.117 [0.097, 0.134] | 0.432 | 0.114 | 0.061 [0.059, 0.066] |
| eurovoc-placed | 0.974 | 0.848 [0.846, 0.849] | [+0.843, +0.854] | 0.125 [0.117, 0.141] | 0.539 [0.536, 0.542] | 0.106 [0.093, 0.118] | 0.296 | 0.142 | 0.060 [0.059, 0.065] |
| lcc-placed | 0.876 | 0.849 [0.847, 0.850] | [+0.842, +0.854] | 0.098 [0.090, 0.110] | 0.538 [0.535, 0.541] | 0.080 [0.067, 0.095] | 0.345 | 0.129 | 0.060 [0.058, 0.064] |
| arxiv-placed | 0.426 | 0.891 [0.890, 0.892] | [+0.885, +0.898] | 0.035 [0.031, 0.041] | 0.527 [0.523, 0.529] | 0.166 [0.142, 0.190] | 0.623 | 0.076 | 0.061 [0.058, 0.065] |
| bk-placed | 0.481 | 0.902 [0.900, 0.904] | [+0.895, +0.908] | 0.061 [0.055, 0.072] | 0.526 [0.523, 0.529] | 0.175 [0.154, 0.199] | 0.467 | 0.086 | 0.057 [0.055, 0.060] |
| jel-placed | 0.479 | 0.903 [0.901, 0.904] | [+0.896, +0.908] | 0.049 [0.044, 0.056] | 0.532 [0.529, 0.535] | 0.213 [0.193, 0.233] | 0.488 | 0.081 | 0.061 [0.059, 0.065] |
| msc-placed | 0.392 | 0.950 [0.949, 0.951] | [+0.945, +0.954] | 0.075 [0.068, 0.086] | 0.521 [0.519, 0.523] | 0.271 [0.243, 0.303] | 0.425 | 0.077 | 0.055 [0.053, 0.058] |
| msc-gold | 0.132 | 0.984 [0.984, 0.985] | [+0.981, +0.986] | 0.043 [0.038, 0.052] | 0.508 [0.507, 0.509] | 0.499 [0.458, 0.540] | 0.507 | 0.053 | 0.064 [0.062, 0.068] |
| jel-gold | 0.000 | 1.000 [1.000, 1.000] | [+1.000, +1.000] | 0.000 [0.000, 0.000] | 0.500 [0.500, 0.500] | 0.000 [0.000, 0.000] | 0.000 | 0.000 | 0.052 [0.050, 0.055] |

Paired-bootstrap differences (first − second; 95% interval; two-sided bootstrap p, * = p < 0.05). Lower cost is better; higher TSD and triplet are better.

| comparison | cost_norm (vertex) | cost_norm (edge) | TSD (vertex) | triplet (vertex) |
| --- | ---: | ---: | ---: | ---: |
| arxiv-gold − arxiv-placed | -0.5579 [-0.5714, -0.5454] p<0.005* | -0.5579 [-0.5600, -0.5555] p<0.005* | +1.060 [+1.025, +1.099] p<0.005* | +0.272 [+0.266, +0.280] p<0.005* |
| arxiv-gold − msc-gold | -0.6510 [-0.6613, -0.6423] p<0.005* | -0.6510 [-0.6531, -0.6492] p<0.005* | +1.052 [+1.015, +1.088] p<0.005* | +0.290 [+0.285, +0.297] p<0.005* |
| arxiv-gold − jel-gold | -0.6667 [-0.6767, -0.6581] p<0.005* | -0.6667 [-0.6686, -0.6649] p<0.005* | +1.095 [+1.061, +1.131] p<0.005* | +0.298 [+0.293, +0.305] p<0.005* |

### msc-gold subset (1322 of 10000 items)

Graph restricted to the subset: 7527 edges.

| run | coverage | cost_norm [edge CI] | [vertex CI] | TSD [vertex CI] | triplet [vertex CI] | locality [vertex CI] | kNN d1 | AMI fine | DP fine [CI] |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-own | 1.000 | 0.283 [0.276, 0.289] | [+0.268, +0.302] | 0.989 [0.905, 1.072] | 0.813 [0.795, 0.826] | 0.705 [0.674, 0.725] | 0.903 | 0.368 | 0.256 [0.232, 0.292] |
| arxiv-gold | 1.000 | 0.507 [0.496, 0.516] | [+0.484, +0.532] | 0.584 [0.520, 0.643] | 0.712 [0.694, 0.727] | 0.439 [0.394, 0.486] | 0.834 | 1.000 | 1.000 [1.000, 1.000] |
| learned-placed | 0.997 | 0.515 [0.508, 0.522] | [+0.489, +0.538] | 0.287 [0.239, 0.354] | 0.651 [0.627, 0.675] | 0.300 [0.252, 0.345] | 0.852 | 0.222 | 0.201 [0.180, 0.237] |
| openalex-topics-placed | 0.936 | 0.659 [0.652, 0.668] | [+0.635, +0.691] | 0.177 [0.150, 0.231] | 0.561 [0.538, 0.582] | 0.146 [0.090, 0.190] | 0.731 | 0.168 | 0.134 [0.126, 0.160] |
| msc-gold | 1.000 | 0.679 [0.669, 0.689] | [+0.653, +0.708] | 0.523 [0.470, 0.610] | 0.641 [0.624, 0.654] | 0.499 [0.464, 0.535] | 0.507 | 0.156 | 0.409 [0.376, 0.462] |
| unesco-thesaurus-placed | 0.906 | 0.743 [0.735, 0.751] | [+0.712, +0.767] | 0.132 [0.109, 0.189] | 0.551 [0.533, 0.568] | 0.157 [0.109, 0.210] | 0.519 | 0.144 | 0.140 [0.132, 0.168] |
| ilc2-placed | 0.999 | 0.756 [0.747, 0.764] | [+0.735, +0.777] | 0.060 [0.053, 0.092] | 0.556 [0.541, 0.569] | 0.124 [0.087, 0.162] | 0.346 | 0.060 | 0.119 [0.111, 0.143] |
| lcc-full-placed | 0.999 | 0.820 [0.811, 0.829] | [+0.800, +0.837] | 0.076 [0.070, 0.120] | 0.520 [0.507, 0.534] | 0.072 [0.026, 0.122] | 0.277 | 0.075 | 0.121 [0.117, 0.149] |
| ipc-placed | 0.937 | 0.831 [0.824, 0.839] | [+0.812, +0.849] | 0.066 [0.058, 0.104] | 0.525 [0.513, 0.536] | 0.070 [0.023, 0.111] | 0.328 | 0.086 | 0.117 [0.109, 0.142] |
| fdc-placed | 0.619 | 0.881 [0.874, 0.889] | [+0.860, +0.902] | 0.056 [0.043, 0.087] | 0.522 [0.510, 0.534] | 0.185 [0.119, 0.253] | 0.390 | 0.100 | 0.115 [0.103, 0.138] |
| lcc-placed | 0.721 | 0.892 [0.886, 0.899] | [+0.871, +0.911] | 0.076 [0.058, 0.108] | 0.526 [0.516, 0.537] | 0.108 [0.051, 0.159] | 0.272 | 0.114 | 0.117 [0.106, 0.140] |
| eurovoc-placed | 0.905 | 0.895 [0.888, 0.902] | [+0.877, +0.910] | 0.050 [0.042, 0.086] | 0.520 [0.511, 0.529] | 0.043 [-0.005, 0.084] | 0.199 | 0.080 | 0.115 [0.108, 0.139] |
| acm-ccs-placed | 0.504 | 0.918 [0.912, 0.924] | [+0.898, +0.935] | 0.055 [0.047, 0.087] | 0.520 [0.512, 0.530] | 0.184 [0.106, 0.261] | 0.401 | 0.128 | 0.146 [0.126, 0.179] |
| msc-placed | 0.458 | 0.936 [0.931, 0.943] | [+0.921, +0.952] | 0.082 [0.067, 0.119] | 0.526 [0.518, 0.535] | 0.355 [0.297, 0.414] | 0.338 | 0.043 | 0.116 [0.105, 0.136] |
| arxiv-placed | 0.286 | 0.967 [0.963, 0.970] | [+0.955, +0.976] | 0.028 [0.017, 0.051] | 0.509 [0.504, 0.515] | 0.114 [-0.039, 0.242] | 0.389 | 0.081 | 0.113 [0.101, 0.132] |
| jel-placed | 0.318 | 0.969 [0.964, 0.972] | [+0.960, +0.978] | 0.017 [0.010, 0.030] | 0.507 [0.502, 0.512] | 0.128 [0.001, 0.240] | 0.352 | 0.062 | 0.111 [0.098, 0.130] |
| bk-placed | 0.275 | 0.977 [0.974, 0.980] | [+0.969, +0.984] | 0.011 [0.007, 0.031] | 0.502 [0.497, 0.506] | 0.055 [-0.115, 0.174] | 0.299 | 0.060 | 0.110 [0.098, 0.130] |
| jel-gold | 0.000 | 1.000 [1.000, 1.000] | [+1.000, +1.000] | 0.001 [0.002, 0.002] | 0.500 [0.500, 0.500] | 0.000 [0.000, 0.000] | 0.000 | 0.000 | 0.108 [0.094, 0.126] |

Paired-bootstrap differences (first − second; 95% interval; two-sided bootstrap p, * = p < 0.05). Lower cost is better; higher TSD and triplet are better.

| comparison | cost_norm (vertex) | cost_norm (edge) | TSD (vertex) | triplet (vertex) |
| --- | ---: | ---: | ---: | ---: |
| msc-gold − msc-placed | -0.2569 [-0.2867, -0.2217] p<0.005* | -0.2569 [-0.2682, -0.2459] p<0.005* | +0.441 [+0.376, +0.510] p<0.005* | +0.115 [+0.099, +0.130] p<0.005* |
| msc-gold − arxiv-gold | +0.1725 [+0.1500, +0.1951] p<0.005* | +0.1725 [+0.1628, +0.1833] p<0.005* | -0.060 [-0.098, +0.014] p=0.170 | -0.071 [-0.084, -0.057] p<0.005* |
| msc-gold − jel-gold | -0.3207 [-0.3469, -0.2920] p<0.005* | -0.3207 [-0.3313, -0.3112] p<0.005* | +0.522 [+0.468, +0.608] p<0.005* | +0.141 [+0.124, +0.154] p<0.005* |

### jel-gold subset (2 of 10000 items)

Not scored: 2 labelled items, below the minimum of 30.

### lcc-gold subset (461 of 500 items)

Graph restricted to the subset: 6344 edges.

| run | coverage | cost_norm [edge CI] | [vertex CI] | TSD [vertex CI] | triplet [vertex CI] | locality [vertex CI] | kNN d1 | AMI fine | DP fine [CI] |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-own-holdout | 1.000 | 0.446 [0.438, 0.454] | [+0.412, +0.479] | 0.386 [0.306, 0.472] | 0.716 [0.695, 0.739] | 0.340 [0.248, 0.426] | 0.974 | 0.062 | 0.266 [0.245, 0.309] |
| lcc-full-gold | 1.000 | 0.706 [0.698, 0.715] | [+0.679, +0.733] | 0.037 [0.031, 0.073] | 0.537 [0.512, 0.559] | 0.121 [0.065, 0.169] | 0.839 | 0.368 | 1.000 [1.000, 1.000] |
| lcc-gold | 1.000 | 0.709 [0.700, 0.717] | [+0.683, +0.735] | 0.027 [0.021, 0.057] | 0.537 [0.512, 0.559] | 0.103 [0.043, 0.160] | 0.839 | 0.738 | 1.000 [1.000, 1.000] |
| lcc-full-placed-holdout | 0.557 | 0.924 [0.917, 0.931] | [+0.898, +0.945] | 0.046 [0.036, 0.078] | 0.515 [0.506, 0.525] | 0.123 [0.028, 0.212] | 0.436 | 0.033 | 0.241 [0.229, 0.279] |
| ilc2-placed-holdout | 0.445 | 0.956 [0.950, 0.962] | [+0.939, +0.970] | 0.055 [0.040, 0.088] | 0.511 [0.505, 0.518] | 0.182 [0.085, 0.284] | 0.366 | 0.053 | 0.241 [0.226, 0.276] |
| unesco-thesaurus-placed-holdout | 0.156 | 0.989 [0.986, 0.991] | [+0.980, +0.995] | 0.018 [0.013, 0.035] | 0.504 [0.501, 0.508] | 0.232 [-0.028, 0.425] | 0.597 | 0.096 | 0.239 [0.221, 0.274] |
| learned-placed-holdout | 0.087 | 0.991 [0.989, 0.993] | [+0.982, +0.996] | 0.018 [0.011, 0.036] | 0.503 [0.500, 0.507] | 0.312 [-0.042, 0.612] | 0.775 | 0.023 | 0.240 [0.222, 0.274] |
| lcc-placed-holdout | 0.080 | 0.997 [0.995, 0.998] | [+0.993, +0.999] | 0.010 [0.007, 0.022] | 0.501 [0.500, 0.503] | 0.217 [-0.265, 0.542] | 0.486 | 0.071 | 0.239 [0.220, 0.272] |
| bk-placed-holdout | 0.046 | 0.998 [0.997, 0.999] | [+0.996, +1.000] | 0.007 [0.006, 0.016] | 0.501 [0.499, 0.502] | 0.352 [-0.164, 0.643] | 0.524 | 0.056 | 0.239 [0.220, 0.271] |
| fdc-placed-holdout | 0.052 | 0.998 [0.997, 0.999] | [+0.996, +1.000] | 0.007 [0.006, 0.016] | 0.501 [0.500, 0.502] | 0.287 [-0.141, 0.702] | 0.500 | 0.067 | 0.239 [0.220, 0.271] |

Paired-bootstrap differences (first − second; 95% interval; two-sided bootstrap p, * = p < 0.05). Lower cost is better; higher TSD and triplet are better.

| comparison | cost_norm (vertex) | cost_norm (edge) | TSD (vertex) | triplet (vertex) |
| --- | ---: | ---: | ---: | ---: |
| lcc-gold − lcc-placed-holdout | -0.2879 [-0.3136, -0.2608] p<0.005* | -0.2879 [-0.2963, -0.2793] p<0.005* | +0.017 [+0.005, +0.044] p<0.005* | +0.035 [+0.010, +0.057] p<0.005* |
| lcc-gold − lcc-full-gold | +0.0023 [+0.0014, +0.0037] p<0.005* | +0.0023 [+0.0020, +0.0027] p<0.005* | -0.010 [-0.021, -0.005] p<0.005* | -0.001 [-0.002, +0.001] p=0.560 |

### lcc-full-gold subset (461 of 500 items)

Graph restricted to the subset: 6344 edges.

| run | coverage | cost_norm [edge CI] | [vertex CI] | TSD [vertex CI] | triplet [vertex CI] | locality [vertex CI] | kNN d1 | AMI fine | DP fine [CI] |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-own-holdout | 1.000 | 0.446 [0.438, 0.454] | [+0.412, +0.479] | 0.386 [0.306, 0.472] | 0.716 [0.695, 0.739] | 0.340 [0.248, 0.426] | 0.974 | 0.062 | 0.266 [0.245, 0.309] |
| lcc-full-gold | 1.000 | 0.706 [0.698, 0.715] | [+0.679, +0.733] | 0.037 [0.031, 0.073] | 0.537 [0.512, 0.559] | 0.121 [0.065, 0.169] | 0.839 | 0.368 | 1.000 [1.000, 1.000] |
| lcc-gold | 1.000 | 0.709 [0.700, 0.717] | [+0.683, +0.735] | 0.027 [0.021, 0.057] | 0.537 [0.512, 0.559] | 0.103 [0.043, 0.160] | 0.839 | 0.738 | 1.000 [1.000, 1.000] |
| lcc-full-placed-holdout | 0.557 | 0.924 [0.917, 0.931] | [+0.898, +0.945] | 0.046 [0.036, 0.078] | 0.515 [0.506, 0.525] | 0.123 [0.028, 0.212] | 0.436 | 0.033 | 0.241 [0.229, 0.279] |
| ilc2-placed-holdout | 0.445 | 0.956 [0.950, 0.962] | [+0.939, +0.970] | 0.055 [0.040, 0.088] | 0.511 [0.505, 0.518] | 0.182 [0.085, 0.284] | 0.366 | 0.053 | 0.241 [0.226, 0.276] |
| unesco-thesaurus-placed-holdout | 0.156 | 0.989 [0.986, 0.991] | [+0.980, +0.995] | 0.018 [0.013, 0.035] | 0.504 [0.501, 0.508] | 0.232 [-0.028, 0.425] | 0.597 | 0.096 | 0.239 [0.221, 0.274] |
| learned-placed-holdout | 0.087 | 0.991 [0.989, 0.993] | [+0.982, +0.996] | 0.018 [0.011, 0.036] | 0.503 [0.500, 0.507] | 0.312 [-0.042, 0.612] | 0.775 | 0.023 | 0.240 [0.222, 0.274] |
| lcc-placed-holdout | 0.080 | 0.997 [0.995, 0.998] | [+0.993, +0.999] | 0.010 [0.007, 0.022] | 0.501 [0.500, 0.503] | 0.217 [-0.265, 0.542] | 0.486 | 0.071 | 0.239 [0.220, 0.272] |
| bk-placed-holdout | 0.046 | 0.998 [0.997, 0.999] | [+0.996, +1.000] | 0.007 [0.006, 0.016] | 0.501 [0.499, 0.502] | 0.352 [-0.164, 0.643] | 0.524 | 0.056 | 0.239 [0.220, 0.271] |
| fdc-placed-holdout | 0.052 | 0.998 [0.997, 0.999] | [+0.996, +1.000] | 0.007 [0.006, 0.016] | 0.501 [0.500, 0.502] | 0.287 [-0.141, 0.702] | 0.500 | 0.067 | 0.239 [0.220, 0.271] |

Paired-bootstrap differences (first − second; 95% interval; two-sided bootstrap p, * = p < 0.05). Lower cost is better; higher TSD and triplet are better.

| comparison | cost_norm (vertex) | cost_norm (edge) | TSD (vertex) | triplet (vertex) |
| --- | ---: | ---: | ---: | ---: |
| lcc-full-gold − lcc-full-placed-holdout | -0.2179 [-0.2573, -0.1773] p<0.005* | -0.2179 [-0.2291, -0.2072] p<0.005* | -0.008 [-0.034, +0.021] p=0.710 | +0.022 [-0.003, +0.044] p=0.140 |
| lcc-full-gold − lcc-gold | -0.0023 [-0.0037, -0.0014] p<0.005* | -0.0023 [-0.0027, -0.0020] p<0.005* | +0.010 [+0.005, +0.021] p<0.005* | +0.001 [-0.001, +0.002] p=0.560 |

### Gold against placed on the items both place

The strictest equal footing: each gold run and its shelfmark-placed counterpart restricted to the items that both place (graph edges among them), so neither run leaves an item at the root. Differences are gold − placed, paired bootstrap; * = p < 0.05.

| gold | placed | items | edges | cost gold | cost placed | Δ cost (vertex) | Δ TSD (vertex) | Δ triplet (vertex) | Δ locality (vertex) | AMI fine gold / placed |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| arxiv-gold | arxiv-placed | 4260 | 34036 | 0.419 [0.404, 0.435] | 0.646 [0.635, 0.658] | -0.2274 [-0.2448, -0.2106] p<0.005* | +0.643 [+0.594, +0.697] p<0.005* | +0.155 [+0.141, +0.169] p<0.005* | +0.341 [+0.311, +0.372] p<0.005* | 1.000 / 0.139 |
| msc-gold | msc-placed | 606 | 1912 | 0.704 [0.653, 0.750] | 0.760 [0.725, 0.803] | -0.0562 [-0.1165, -0.0034] p=0.040* | +0.148 [+0.045, +0.279] p<0.005* | +0.025 [-0.005, +0.059] p=0.160 | +0.112 [+0.032, +0.198] p=0.030* | 0.099 / 0.115 |
| lcc-gold | lcc-placed-holdout | 37 | 64 | 0.708 [0.504, 0.879] | 0.667 [0.409, 0.858] | +0.0411 [-0.1995, +0.3042] p=0.700 | -0.199 [-0.477, +0.140] p=0.440 | -0.023 [-0.239, +0.266] p=0.750 | -0.023 [-0.541, +0.502] p=0.960 | 0.758 / 0.020 |
| lcc-full-gold | lcc-full-placed-holdout | 257 | 2150 | 0.695 [0.649, 0.735] | 0.807 [0.746, 0.853] | -0.1115 [-0.1760, -0.0425] p=0.010* | -0.058 [-0.127, +0.008] p=0.090 | +0.015 [-0.027, +0.060] p=0.600 | +0.025 [-0.101, +0.141] p=0.680 | 0.371 / 0.017 |

## 2. Every score on all items and on the covered subset

For every run, each catalogue score on all items and on the items the run places (its own covered subset; graph edges restricted to it). Runs that place every item have one value. Covered subsets under 30 items are not scored ("-"). The key scores are below; the appendix and `results.json` hold every score.

### gold labels (source classifiers; learned: own placements)

Each cell: all items → covered subset.

| run | coverage | covered n | cost_norm | TSD | triplet | locality | kNN d1 | AMI fine | DP fine | silhouette d1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-own | 1.000 | 10000 | 0.172 → 0.172 | 1.802 → 1.802 | 0.873 → 0.873 | 0.793 → 0.793 | 0.968 → 0.968 | 0.451 → 0.451 | 0.225 → 0.225 | 0.087 → 0.087 |
| arxiv-gold | 1.000 | 10000 | 0.333 → 0.333 | 1.095 → 1.095 | 0.798 → 0.798 | 0.606 → 0.606 | 0.858 → 0.858 | 1.000 → 1.000 | 1.000 → 1.000 | 0.041 → 0.041 |
| learned-own-holdout | 1.000 | 500 | 0.443 → 0.443 | 0.389 → 0.389 | 0.704 → 0.704 | 0.346 → 0.346 | 0.976 → 0.976 | 0.062 → 0.062 | 0.266 → 0.266 | 0.040 → 0.040 |
| lcc-full-gold | 0.922 | 461 | 0.706 → 0.706 | 0.032 → 0.037 | 0.534 → 0.537 | 0.121 → 0.121 | 0.839 → 0.839 | 0.368 → 0.368 | 1.000 → 1.000 | -0.149 → -0.149 |
| lcc-gold | 0.922 | 461 | 0.708 → 0.709 | 0.024 → 0.027 | 0.533 → 0.537 | 0.103 → 0.103 | 0.839 → 0.839 | 0.738 → 0.738 | 1.000 → 1.000 | -0.149 → -0.149 |
| msc-gold | 0.132 | 1322 | 0.984 → 0.679 | 0.043 → 0.523 | 0.508 → 0.641 | 0.499 → 0.499 | 0.507 → 0.507 | 0.053 → 0.156 | 0.064 → 0.409 | -0.115 → -0.067 |
| jel-gold | 0.000 | 2 | 1.000 → - | 0.000 → - | 0.500 → - | 0.000 → - | 0.000 → - | 0.000 → - | 0.052 → - | 0.000 → - |

### shelfmark-placed, arXiv corpus

Each cell: all items → covered subset.

| run | coverage | covered n | cost_norm | TSD | triplet | locality | kNN d1 | AMI fine | DP fine | silhouette d1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-placed | 0.999 | 9993 | 0.379 → 0.379 | 0.677 → 0.678 | 0.750 → 0.751 | 0.566 → 0.566 | 0.905 → 0.905 | 0.306 → 0.307 | 0.132 → 0.132 | 0.080 → 0.078 |
| openalex-topics-placed | 0.973 | 9732 | 0.641 → 0.633 | 0.255 → 0.267 | 0.576 → 0.580 | 0.204 → 0.204 | 0.761 → 0.761 | 0.188 → 0.191 | 0.073 → 0.075 | -0.033 → -0.022 |
| unesco-thesaurus-placed | 0.973 | 9731 | 0.721 → 0.712 | 0.179 → 0.186 | 0.559 → 0.559 | 0.116 → 0.116 | 0.551 → 0.551 | 0.157 → 0.155 | 0.064 → 0.066 | -0.045 → -0.040 |
| ipc-placed | 0.980 | 9799 | 0.769 → 0.764 | 0.082 → 0.084 | 0.533 → 0.533 | 0.061 → 0.061 | 0.415 → 0.415 | 0.118 → 0.116 | 0.059 → 0.060 | -0.018 → -0.021 |
| ilc2-placed | 1.000 | 9999 | 0.819 → 0.819 | 0.108 → 0.108 | 0.550 → 0.550 | 0.163 → 0.163 | 0.324 → 0.324 | 0.102 → 0.102 | 0.059 → 0.059 | -0.189 → -0.160 |
| lcc-full-placed | 1.000 | 9999 | 0.823 → 0.823 | 0.114 → 0.114 | 0.533 → 0.533 | 0.079 → 0.079 | 0.339 → 0.339 | 0.108 → 0.108 | 0.058 → 0.058 | -0.068 → -0.086 |
| acm-ccs-placed | 0.745 | 7445 | 0.831 → 0.754 | 0.091 → 0.131 | 0.545 → 0.560 | 0.096 → 0.096 | 0.437 → 0.437 | 0.127 → 0.131 | 0.066 → 0.086 | -0.036 → -0.037 |
| fdc-placed | 0.770 | 7696 | 0.834 → 0.744 | 0.092 → 0.143 | 0.535 → 0.557 | 0.117 → 0.117 | 0.432 → 0.432 | 0.114 → 0.135 | 0.061 → 0.072 | -0.060 → -0.047 |
| eurovoc-placed | 0.974 | 9737 | 0.848 → 0.842 | 0.125 → 0.129 | 0.539 → 0.541 | 0.106 → 0.106 | 0.296 → 0.296 | 0.142 → 0.140 | 0.060 → 0.062 | -0.066 → -0.065 |
| lcc-placed | 0.876 | 8765 | 0.849 → 0.814 | 0.098 → 0.120 | 0.538 → 0.545 | 0.080 → 0.080 | 0.345 → 0.345 | 0.129 → 0.134 | 0.060 → 0.065 | -0.078 → -0.094 |
| arxiv-placed | 0.426 | 4260 | 0.891 → 0.646 | 0.035 → 0.123 | 0.527 → 0.590 | 0.166 → 0.166 | 0.623 → 0.623 | 0.076 → 0.139 | 0.061 → 0.102 | -0.037 → -0.047 |
| bk-placed | 0.481 | 4806 | 0.902 → 0.679 | 0.061 → 0.190 | 0.526 → 0.585 | 0.175 → 0.175 | 0.467 → 0.467 | 0.086 → 0.149 | 0.057 → 0.078 | -0.014 → -0.011 |
| jel-placed | 0.479 | 4786 | 0.903 → 0.694 | 0.049 → 0.145 | 0.532 → 0.602 | 0.213 → 0.213 | 0.488 → 0.488 | 0.081 → 0.142 | 0.061 → 0.101 | -0.039 → -0.068 |
| msc-placed | 0.392 | 3916 | 0.950 → 0.726 | 0.075 → 0.378 | 0.521 → 0.614 | 0.271 → 0.271 | 0.425 → 0.425 | 0.077 → 0.166 | 0.055 → 0.072 | -0.129 → -0.137 |

### shelfmark-placed, book holdout

Each cell: all items → covered subset.

| run | coverage | covered n | cost_norm | TSD | triplet | locality | kNN d1 | AMI fine | DP fine | silhouette d1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| lcc-full-placed-holdout | 0.566 | 283 | 0.925 → 0.811 | 0.045 → 0.116 | 0.512 → 0.528 | 0.118 → 0.118 | 0.435 → 0.435 | 0.033 → 0.017 | 0.241 → 0.232 | -0.116 → -0.116 |
| ilc2-placed-holdout | 0.456 | 228 | 0.954 → 0.797 | 0.056 → 0.220 | 0.514 → 0.552 | 0.175 → 0.175 | 0.364 → 0.364 | 0.053 → 0.028 | 0.241 → 0.231 | -0.143 → -0.143 |
| unesco-thesaurus-placed-holdout | 0.158 | 79 | 0.989 → 0.680 | 0.017 → 0.374 | 0.505 → 0.593 | 0.247 → 0.247 | 0.582 → 0.582 | 0.096 → 0.062 | 0.239 → 0.201 | -0.010 → -0.010 |
| learned-placed-holdout | 0.086 | 43 | 0.992 → 0.453 | 0.016 → 0.590 | 0.503 → 0.641 | 0.273 → 0.273 | 0.791 → 0.791 | 0.023 → 0.081 | 0.240 → 0.433 | 0.028 → 0.028 |
| lcc-placed-holdout | 0.084 | 42 | 0.996 → 0.635 | 0.010 → 0.421 | 0.502 → 0.611 | 0.251 → 0.251 | 0.500 → 0.500 | 0.071 → 0.020 | 0.239 → 0.176 | -0.053 → -0.053 |
| fdc-placed-holdout | 0.052 | 26 | 0.998 → - | 0.007 → - | 0.501 → - | 0.354 → - | 0.500 → - | 0.067 → - | 0.239 → - | -0.018 → - |
| bk-placed-holdout | 0.044 | 22 | 0.999 → - | 0.006 → - | 0.501 → - | 0.382 → - | 0.500 → - | 0.056 → - | 0.239 → - | 0.009 → - |

## 3. How much of each ranking is coverage

Across the runs of a group: ρ(cov) is Spearman's ρ between the score and coverage, and ρ² the share of the score's rank variance that coverage explains. τ(all, covered) is Kendall's τ between the run ranking on all items and on each run's covered subset (runs whose covered subset was scored). The last two columns are the score's ρ with Dasgupta cost_norm, raw and partial on coverage. Intervals come from the bootstrap replicates of the scores (coverage is fixed), so scores without replicates have none.

### gold labels (source classifiers; learned: own placements) (6 runs)

| score | better | ρ(cov) [CI] | ρ² | τ(all, covered) [CI] | runs | ρ with cost | partial ρ with cost [CI] |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| warrant.useful_depth_share | up | +1.00 [+1.00, +1.00] | 1.00 | +0.64 - | 6 | -0.93 | - (undefined) |
| probe.knn_d1 | up | +0.94 [+0.63, +0.94] | 0.88 | +1.00 [+1.00, +1.00] | 6 | -0.81 | +0.45 [-0.45, +0.45] |
| probe.centroid_d1 | up | +0.94 [+0.94, +0.94] | 0.88 | +1.00 - | 6 | -0.93 | -0.45 [-0.45, -0.45] |
| dasgupta.cost_norm | down | -0.93 [-0.93, -0.93] | 0.86 | +0.73 [+0.73, +0.73] | 6 | +1.00 | - |
| dasgupta.binarized_cost_norm | down | -0.93 [-0.93, -0.93] | 0.86 | +0.60 [+0.60, +0.60] | 6 | +1.00 | +1.00 [+1.00, +1.00] |
| triplet.accuracy | up | +0.93 [+0.93, +0.93] | 0.86 | +0.73 [+0.60, +0.73] | 6 | -1.00 | -1.00 [-1.00, -0.60] |
| embed.cophenetic | up | +0.93 [+0.93, +0.93] | 0.86 | +0.60 - | 6 | -0.89 | -0.20 [-0.20, -0.20] |
| insertion.splits_per_1000 | down | +0.93 [+0.93, +0.93] | 0.86 | +1.00 - | 6 | -0.94 | -0.60 [-0.60, -0.60] |
| embed.silhouette_d1 | up | +0.75 [+0.75, +0.75] | 0.56 | +1.00 - | 6 | -0.81 | -0.46 [-0.46, -0.46] |
| embed.davies_bouldin_d1 | down | +0.75 [+0.75, +0.75] | 0.56 | +1.00 - | 6 | -0.58 | +0.46 [+0.46, +0.46] |
| hospitality.array | up | -0.75 [-0.75, -0.75] | 0.56 | +1.00 - | 6 | +0.72 | +0.12 [+0.12, +0.12] |
| tsd.nats | up | +0.74 [+0.74, +0.83] | 0.55 | +0.87 [+0.73, +0.87] | 6 | -0.83 | -0.56 [-0.82, -0.56] |
| mdl.total_bits_per_item | down | -0.74 [-0.74, -0.74] | 0.55 | +1.00 - | 6 | +0.54 | -0.56 [-0.56, -0.56] |
| granularity.empty_class_share | down | -0.74 [-0.74, -0.74] | 0.55 | +1.00 - | 6 | +0.77 | +0.34 [+0.34, +0.34] |
| dasgupta.level1_cost_norm | down | -0.63 [-0.63, -0.63] | 0.39 | +0.57 - | 6 | +0.75 | +0.59 [+0.59, +0.59] |
| notation.length | down | +0.62 [+0.62, +0.62] | 0.38 | +1.00 - | 6 | -0.49 | +0.29 [+0.29, +0.29] |
| structure.gini | down | +0.62 [+0.62, +0.62] | 0.38 | +1.00 - | 3 | -0.31 | +0.86 [+0.86, +0.86] |
| notation.regularity | up | +0.50 [+0.50, +0.50] | 0.25 | +1.00 - | 6 | -0.43 | +0.09 [+0.09, +0.09] |
| order.locality | up | +0.49 [+0.49, +0.49] | 0.24 | +1.00 [+1.00, +1.00] | 6 | -0.66 | -0.61 [-0.61, -0.61] |
| order.kendall_tau | up | +0.49 [+0.49, +0.49] | 0.24 | +1.00 - | 6 | -0.66 | -0.61 [-0.61, -0.61] |
| gold.ami_fine | up | +0.46 [+0.46, +0.46] | 0.21 | +0.87 - | 6 | -0.49 | -0.17 [-0.17, -0.17] |
| search.gap_bits | down | -0.43 [-0.43, -0.43] | 0.19 | +0.60 - | 6 | +0.54 | +0.42 [+0.42, +0.42] |
| gold.v_measure_fine | up | +0.31 [+0.06, +0.31] | 0.10 | +0.60 [+0.60, +0.73] | 6 | -0.31 | -0.08 [-0.23, -0.08] |
| gold.dendrogram_purity_fine | up | +0.16 [+0.16, +0.31] | 0.03 | +0.67 [+0.62, +0.71] | 6 | -0.09 | +0.16 [+0.16, +0.16] |
| gold.dendrogram_purity_coarse | up | +0.16 [+0.16, +0.16] | 0.03 | +0.83 [+0.83, +0.83] | 6 | -0.09 | +0.16 [+0.16, +0.16] |
| gold.hierarchical_f_fine | up | +0.00 [+0.00, +0.31] | 0.00 | +0.71 [+0.43, +0.71] | 6 | +0.06 | +0.15 [+0.15, +0.16] |
| search.scheme_bits | down | +0.00 [+0.00, +0.00] | 0.00 | +0.47 - | 6 | +0.03 | +0.08 [+0.08, +0.08] |
| structure.largest_class_share | down | +0.00 [+0.00, +0.00] | 0.00 | +1.00 - | 3 | -0.03 | -0.08 [-0.08, -0.08] |

### shelfmark-placed, arXiv corpus (14 runs)

| score | better | ρ(cov) [CI] | ρ² | τ(all, covered) [CI] | runs | ρ with cost | partial ρ with cost [CI] |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| warrant.useful_depth_share | up | +0.91 [+0.91, +0.91] | 0.83 | +0.22 - | 14 | -0.73 | -0.10 [-0.13, -0.10] |
| gold.v_measure_fine | up | +0.89 [+0.89, +0.91] | 0.80 | +0.76 [+0.69, +0.76] | 14 | -0.80 | -0.40 [-0.40, -0.32] |
| search.scheme_bits | down | +0.82 [+0.82, +0.82] | 0.68 | +0.91 - | 14 | -0.65 | -0.05 [-0.07, -0.05] |
| dasgupta.cost_norm | down | -0.77 [-0.77, -0.75] | 0.59 | +0.10 [+0.08, +0.12] | 14 | +1.00 | - |
| dasgupta.level1_cost_norm | down | -0.76 [-0.76, -0.76] | 0.58 | -0.08 - | 14 | +0.99 | +0.98 [+0.97, +0.99] |
| tsd.nats | up | +0.73 [+0.73, +0.73] | 0.54 | +0.16 [+0.14, +0.23] | 14 | -0.79 | -0.53 [-0.54, -0.51] |
| mdl.total_bits_per_item | down | +0.73 [+0.73, +0.73] | 0.54 | +0.82 - | 14 | -0.41 | +0.35 [+0.33, +0.35] |
| collocation.author_same_class | up | -0.73 [-0.73, -0.73] | 0.53 | +1.00 - | 14 | +0.40 | -0.37 [-0.39, -0.36] |
| triplet.accuracy | up | +0.62 [+0.57, +0.64] | 0.38 | +0.01 [-0.03, +0.05] | 14 | -0.85 | -0.74 [-0.75, -0.65] |
| insertion.splits_per_1000 | down | -0.61 [-0.61, -0.61] | 0.37 | +1.00 - | 14 | +0.37 | -0.20 [-0.26, -0.20] |
| gold.ami_fine | up | +0.55 [+0.55, +0.55] | 0.31 | +0.27 - | 14 | -0.77 | -0.64 [-0.64, -0.62] |
| gold.hierarchical_f_fine | up | +0.50 [+0.59, +0.76] | 0.25 | +0.14 [-0.01, +0.19] | 14 | -0.71 | -0.60 [-0.68, -0.52] |
| order.kendall_tau | up | -0.48 [-0.48, -0.48] | 0.23 | +1.00 - | 14 | +0.16 | -0.36 [-0.38, -0.35] |
| dasgupta.binarized_cost_norm | down | -0.46 [-0.57, -0.36] | 0.21 | +0.25 [+0.21, +0.34] | 14 | +0.68 | +0.57 [+0.51, +0.62] |
| probe.centroid_d1 | up | -0.40 [-0.40, -0.40] | 0.16 | +1.00 - | 14 | -0.08 | -0.67 [-0.67, -0.63] |
| order.locality | up | -0.38 [-0.38, -0.29] | 0.14 | +1.00 [+0.93, +1.00] | 14 | +0.17 | -0.21 [-0.24, -0.17] |
| search.gap_bits | down | +0.37 [+0.37, +0.37] | 0.14 | +0.67 - | 14 | -0.18 | +0.18 [+0.14, +0.18] |
| probe.knn_d1 | up | -0.33 [-0.37, -0.26] | 0.11 | +1.00 [+1.00, +1.00] | 14 | -0.23 | -0.80 [-0.81, -0.76] |
| embed.davies_bouldin_d1 | down | +0.33 [+0.33, +0.33] | 0.11 | +1.00 - | 14 | -0.25 | -0.00 [-0.00, +0.06] |
| structure.gini | down | -0.22 [-0.22, -0.22] | 0.05 | - - | 0 | -0.16 | -0.54 [-0.54, -0.49] |
| hospitality.array | up | +0.21 [+0.21, +0.21] | 0.04 | +1.00 - | 14 | +0.09 | +0.40 [+0.32, +0.41] |
| notation.length | down | +0.19 [+0.19, +0.19] | 0.04 | +1.00 - | 14 | -0.32 | -0.29 [-0.32, -0.25] |
| granularity.empty_class_share | down | +0.18 [+0.18, +0.18] | 0.03 | +1.00 - | 14 | +0.27 | +0.65 [+0.59, +0.65] |
| embed.cophenetic | up | -0.18 [-0.18, -0.18] | 0.03 | +0.67 - | 14 | -0.06 | -0.32 [-0.34, -0.28] |
| gold.dendrogram_purity_coarse | up | +0.17 [+0.06, +0.33] | 0.03 | +0.47 [+0.32, +0.56] | 14 | -0.56 | -0.68 [-0.75, -0.57] |
| structure.largest_class_share | down | -0.10 [-0.10, -0.10] | 0.01 | - - | 0 | -0.45 | -0.83 [-0.84, -0.83] |
| gold.dendrogram_purity_fine | up | +0.08 [+0.11, +0.41] | 0.01 | +0.43 [+0.14, +0.41] | 14 | -0.53 | -0.73 [-0.80, -0.61] |
| notation.regularity | up | +0.06 [+0.06, +0.06] | 0.00 | +1.00 - | 14 | -0.44 | -0.62 [-0.64, -0.58] |
| embed.silhouette_d1 | up | -0.05 [-0.05, -0.05] | 0.00 | +0.91 - | 14 | -0.32 | -0.57 [-0.57, -0.50] |

### shelfmark-placed, book holdout (7 runs)

| score | better | ρ(cov) [CI] | ρ² | τ(all, covered) [CI] | runs | ρ with cost | partial ρ with cost [CI] |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| dasgupta.cost_norm | down | -1.00 [-1.00, -0.96] | 1.00 | -0.80 [-0.80, -0.40] | 5 | +1.00 | - |
| dasgupta.level1_cost_norm | down | -1.00 [-1.00, -1.00] | 1.00 | -1.00 - | 5 | +1.00 | - (undefined) |
| warrant.useful_depth_share | up | +1.00 [+1.00, +1.00] | 1.00 | +0.36 - | 5 | -1.00 | - (undefined) |
| tsd.nats | up | +0.96 [+0.89, +0.96] | 0.93 | -0.60 [-0.80, -0.20] | 5 | -0.96 | - (undefined) |
| order.locality | up | -0.96 [-0.96, -0.07] | 0.93 | +1.00 [-0.20, +1.00] | 5 | +0.96 | - (undefined) |
| triplet.accuracy | up | +0.93 [+0.89, +0.99] | 0.86 | -0.60 [-1.00, -0.20] | 5 | -0.93 | - (undefined) |
| gold.dendrogram_purity_fine | up | +0.93 [+0.86, +1.00] | 0.86 | +0.40 [-1.00, -0.20] | 5 | -0.93 | - (undefined) |
| gold.v_measure_fine | up | +0.89 [+0.75, +1.00] | 0.80 | -0.40 [-0.80, +0.00] | 5 | -0.89 | - (undefined) |
| embed.cophenetic | up | +0.82 [+0.82, +0.82] | 0.67 | -0.20 - | 5 | -0.82 | - (undefined) |
| gold.hierarchical_f_fine | up | +0.78 [+0.64, +1.00] | 0.61 | -0.20 [-1.00, -0.40] | 5 | -0.78 | - (undefined) |
| search.scheme_bits | down | +0.71 [+0.71, +0.71] | 0.51 | +0.80 - | 5 | -0.71 | - (undefined) |
| structure.gini | down | +0.71 [+0.71, +0.71] | 0.51 | - - | 0 | -0.71 | - (undefined) |
| probe.centroid_d1 | up | -0.68 [-0.68, -0.68] | 0.46 | +0.80 - | 5 | +0.68 | - (undefined) |
| mdl.total_bits_per_item | down | +0.68 [+0.68, +0.68] | 0.46 | +0.60 - | 5 | -0.68 | - (undefined) |
| embed.davies_bouldin_d1 | down | +0.64 [+0.64, +0.64] | 0.41 | +1.00 - | 5 | -0.64 | - (undefined) |
| embed.silhouette_d1 | up | -0.57 [-0.57, -0.57] | 0.33 | +1.00 - | 5 | +0.57 | - (undefined) |
| structure.largest_class_share | down | -0.54 [-0.54, -0.54] | 0.29 | - - | 0 | +0.54 | - (undefined) |
| dasgupta.binarized_cost_norm | down | -0.50 [-0.93, -0.29] | 0.25 | -0.40 [-0.80, +0.00] | 5 | +0.50 | - (undefined) |
| probe.knn_d1 | up | -0.37 [-0.68, +0.18] | 0.14 | +1.00 [+1.00, +1.00] | 5 | +0.37 | - (undefined) |
| hospitality.array | up | +0.36 [+0.36, +0.36] | 0.13 | +1.00 - | 5 | -0.36 | - (undefined) |
| gold.ami_fine | up | -0.32 [-0.32, -0.32] | 0.10 | +0.00 - | 5 | +0.32 | - (undefined) |
| notation.regularity | up | -0.20 [-0.20, -0.20] | 0.04 | +1.00 - | 5 | +0.20 | - (undefined) |
| order.kendall_tau | up | -0.11 [-0.11, -0.11] | 0.01 | +1.00 - | 5 | +0.11 | - (undefined) |
| granularity.empty_class_share | down | +0.07 [+0.07, +0.07] | 0.01 | +1.00 - | 5 | -0.07 | - (undefined) |
| notation.length | down | +0.07 [+0.07, +0.07] | 0.01 | +1.00 - | 5 | -0.07 | - (undefined) |
| search.gap_bits | down | -0.04 [-0.04, -0.04] | 0.00 | +0.20 - | 5 | +0.04 | - (undefined) |
| gold.dendrogram_purity_coarse | up | +0.00 [-0.11, +0.96] | 0.00 | +0.00 [-0.40, +0.80] | 5 | +0.00 | - (undefined) |
| insertion.splits_per_1000 | down | +0.00 [+0.00, +0.00] | 0.00 | +0.00 - | 5 | +0.00 | - (undefined) |

## 4. Bootstrap intervals and paired significance

Two bootstraps for the graph scores. The edge bootstrap resamples graph edges; the vertex bootstrap resamples items and counts each edge `m_i · m_j` times. Edges that share an item are dependent, so the vertex intervals are the honest ones, above all on the 500-book holdout. Every run on one graph and item set draws the same resamples in every replicate, so a difference between two runs is bootstrapped replicate by replicate (paired).

### Interval widths (mean over runs, all items)

| group | runs | cost edge | cost vertex | ratio | triplet edge | triplet vertex | TSD edge | TSD vertex | locality edge | locality vertex |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| gold | 7 | 0.0081 | 0.0303 | 3.8 | 0.0078 | 0.0214 | 0.015 | 0.055 | 0.0196 | 0.0743 |
| placed-corpus | 14 | 0.0033 | 0.0127 | 3.8 | 0.0025 | 0.0072 | 0.006 | 0.023 | 0.0097 | 0.0334 |
| placed-holdout | 7 | 0.0054 | 0.0167 | 3.1 | 0.0030 | 0.0077 | 0.014 | 0.024 | 0.2880 | 0.5364 |

### Significant pairwise differences

Pairs of runs whose paired 95% interval excludes 0 (p < 0.05), out of all pairs. No multiplicity correction; with this many pairs a few significant pairs are expected by chance at p close to 0.05.

| set | dasgupta.cost_norm | vertex.dasgupta.cost_norm | tsd.nats | vertex.tsd.nats | triplet.accuracy | vertex.triplet.accuracy | order.locality | vertex.order.locality | probe.knn_d1 | gold.dendrogram_purity_fine | gold.ari_fine |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| placed-corpus | 89/91 | 86/91 | 90/91 | 82/91 | 86/91 | 80/91 | 86/91 | 77/91 | 86/91 | 67/91 | 80/91 |
| placed-holdout | 19/21 | 16/21 | 16/21 | 13/21 | 16/21 | 9/21 | 2/21 | 0/21 | 8/21 | 15/21 | 5/21 |
| arxiv-gold subset | 151/153 | 148/153 | 152/153 | 143/153 | 148/153 | 142/153 | 148/153 | 139/153 | 147/153 | 125/153 | 139/153 |
| msc-gold subset | 149/153 | 140/153 | 136/153 | 122/153 | 128/153 | 121/153 | 120/153 | 95/153 | 130/153 | 133/153 | 124/153 |
| lcc-gold subset | 43/45 | 40/45 | 38/45 | 31/45 | 39/45 | 32/45 | 13/45 | 4/45 | 27/45 | 38/45 | 23/45 |
| lcc-full-gold subset | 43/45 | 40/45 | 38/45 | 31/45 | 39/45 | 32/45 | 13/45 | 4/45 | 27/45 | 38/45 | 23/45 |

### Book holdout: pairwise cost_norm differences (vertex bootstrap)

Row − column, runs in cost order (best first); * = p < 0.05. Positive means the row run costs more.

| | lcc-full-placed-holdout | ilc2-placed-holdout | unesco-thesaurus-placed-holdout | learned-placed-holdout | lcc-placed-holdout | fdc-placed-holdout | bk-placed-holdout |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| lcc-full-placed-holdout |  | -0.0293* | -0.0638* | -0.0665* | -0.0714* | -0.0731* | -0.0736* |
| ilc2-placed-holdout | +0.0293* |  | -0.0345* | -0.0372* | -0.0421* | -0.0438* | -0.0442* |
| unesco-thesaurus-placed-holdout | +0.0638* | +0.0345* |  | -0.0027 | -0.0076* | -0.0093* | -0.0097* |
| learned-placed-holdout | +0.0665* | +0.0372* | +0.0027 |  | -0.0049 | -0.0066* | -0.0070* |
| lcc-placed-holdout | +0.0714* | +0.0421* | +0.0076* | +0.0049 |  | -0.0017 | -0.0021 |
| fdc-placed-holdout | +0.0731* | +0.0438* | +0.0093* | +0.0066* | +0.0017 |  | -0.0004 |
| bk-placed-holdout | +0.0736* | +0.0442* | +0.0097* | +0.0070* | +0.0021 | +0.0004 |  |

The same matrix from the edge bootstrap:

| | lcc-full-placed-holdout | ilc2-placed-holdout | unesco-thesaurus-placed-holdout | learned-placed-holdout | lcc-placed-holdout | fdc-placed-holdout | bk-placed-holdout |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| lcc-full-placed-holdout |  | -0.0293* | -0.0638* | -0.0665* | -0.0714* | -0.0731* | -0.0736* |
| ilc2-placed-holdout | +0.0293* |  | -0.0345* | -0.0372* | -0.0421* | -0.0438* | -0.0442* |
| unesco-thesaurus-placed-holdout | +0.0638* | +0.0345* |  | -0.0027 | -0.0076* | -0.0093* | -0.0097* |
| learned-placed-holdout | +0.0665* | +0.0372* | +0.0027 |  | -0.0049* | -0.0066* | -0.0070* |
| lcc-placed-holdout | +0.0714* | +0.0421* | +0.0076* | +0.0049* |  | -0.0017* | -0.0021* |
| fdc-placed-holdout | +0.0731* | +0.0438* | +0.0093* | +0.0066* | +0.0017* |  | -0.0004 |
| bk-placed-holdout | +0.0736* | +0.0442* | +0.0097* | +0.0070* | +0.0021* | +0.0004 |  |

## 5. Dense against kNN-sparse Dasgupta cost

On deterministic random subsets of the papers (SplitMix64, seed 20261010) every pair gets the benchmark weight `max(0, cos − τ)` with the corpus graph's τ. The dense cost sums all pairs; the kNN graphs keep each item's k most similar subset items (symmetrised), and "graph-restricted" is the published k = 20 corpus graph restricted to the subset (the construction of the covered-subset runs). Each scheme's tree is induced on the subset. Agreement is between the dense and the sparse run ranking: over the shelfmark-placed runs, and over all runs (plus gold and learned-own). Intervals: 200 Poisson pair-bootstrap replicates on the first subset (dense and sparse share the pair weights in a replicate). "subsets" gives the min–max of the point Spearman ρ over the independent subsets.

### Papers

| n | sparsifier | edges (dense pairs w>0) | placed: Spearman [CI] | Kendall [CI] | subsets ρ | all: Spearman [CI] | Kendall [CI] |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1000 | graph-restricted | 1471 (217084) | +0.912 [+0.89, +0.96] | +0.780 [+0.71, +0.87] | 0.89–0.94 | +0.957 [+0.95, +0.98] | +0.856 [+0.82, +0.91] |
| 1000 | knn10 | 7153 (217084) | +0.956 [+0.94, +0.98] | +0.868 [+0.82, +0.91] | 0.93–0.96 | +0.977 [+0.97, +0.99] | +0.908 [+0.88, +0.93] |
| 1000 | knn20 | 14007 (217084) | +0.974 [+0.96, +0.99] | +0.912 [+0.87, +0.93] | 0.93–0.99 | +0.986 [+0.98, +0.99] | +0.935 [+0.91, +0.95] |
| 1000 | knn50 | 34023 (217084) | +0.974 [+0.97, +0.99] | +0.912 [+0.89, +0.96] | 0.96–0.99 | +0.988 [+0.99, +1.00] | +0.948 [+0.93, +0.97] |
| 2000 | graph-restricted | 5919 (869585) | +0.960 [+0.94, +0.96] | +0.846 [+0.80, +0.87] | 0.91–0.97 | +0.979 [+0.97, +0.98] | +0.895 [+0.87, +0.91] |
| 2000 | knn10 | 14698 (869585) | +0.947 [+0.94, +0.96] | +0.824 [+0.80, +0.87] | 0.95–0.97 | +0.973 [+0.97, +0.98] | +0.882 [+0.87, +0.91] |
| 2000 | knn20 | 28717 (869585) | +0.960 [+0.94, +0.97] | +0.846 [+0.80, +0.89] | 0.95–0.97 | +0.979 [+0.97, +0.98] | +0.895 [+0.87, +0.92] |
| 2000 | knn50 | 69939 (869585) | +0.965 [+0.96, +0.97] | +0.868 [+0.85, +0.89] | 0.95–0.98 | +0.981 [+0.98, +0.98] | +0.908 [+0.90, +0.92] |
| 5000 | graph-restricted | 37319 (5404658) | +0.965 [+0.95, +0.96] | +0.890 [+0.82, +0.89] | 0.95–0.96 | +0.981 [+0.97, +0.98] | +0.922 [+0.88, +0.92] |
| 5000 | knn10 | 37390 (5404658) | +0.960 [+0.94, +0.96] | +0.868 [+0.82, +0.89] | 0.94–0.96 | +0.979 [+0.97, +0.98] | +0.908 [+0.88, +0.92] |
| 5000 | knn20 | 73389 (5404658) | +0.956 [+0.95, +0.96] | +0.868 [+0.82, +0.89] | 0.94–0.96 | +0.977 [+0.97, +0.98] | +0.908 [+0.88, +0.92] |
| 5000 | knn50 | 178666 (5404658) | +0.965 [+0.96, +0.97] | +0.890 [+0.87, +0.91] | 0.95–0.97 | +0.981 [+0.98, +0.99] | +0.922 [+0.91, +0.93] |

Costs on the first subset of n = 5000, runs in dense-cost order:

| run | coverage | dense [CI] | graph-restricted | knn10 | knn20 | knn50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-own | 1.000 | 0.4541 [+0.4538, +0.4545] | 0.1761 | 0.1771 | 0.2035 | 0.2409 |
| learned-placed | 1.000 | 0.5279 [+0.5276, +0.5282] | 0.3795 | 0.3782 | 0.3870 | 0.4027 |
| arxiv-gold | 1.000 | 0.5677 [+0.5674, +0.5681] | 0.3333 | 0.3338 | 0.3500 | 0.3769 |
| openalex-topics-placed | 0.976 | 0.7229 [+0.7225, +0.7233] | 0.6442 | 0.6441 | 0.6539 | 0.6680 |
| ipc-placed | 0.979 | 0.7995 [+0.7992, +0.7999] | 0.7684 | 0.7684 | 0.7743 | 0.7827 |
| unesco-thesaurus-placed | 0.973 | 0.8013 [+0.8010, +0.8018] | 0.7233 | 0.7233 | 0.7353 | 0.7503 |
| acm-ccs-placed | 0.741 | 0.8665 [+0.8661, +0.8668] | 0.8367 | 0.8371 | 0.8429 | 0.8489 |
| lcc-full-placed | 1.000 | 0.8671 [+0.8668, +0.8674] | 0.8204 | 0.8220 | 0.8285 | 0.8360 |
| fdc-placed | 0.775 | 0.8806 [+0.8802, +0.8809] | 0.8382 | 0.8378 | 0.8422 | 0.8507 |
| ilc2-placed | 1.000 | 0.8818 [+0.8814, +0.8821] | 0.8138 | 0.8153 | 0.8240 | 0.8357 |
| lcc-placed | 0.878 | 0.9026 [+0.9022, +0.9029] | 0.8478 | 0.8477 | 0.8564 | 0.8692 |
| eurovoc-placed | 0.974 | 0.9028 [+0.9025, +0.9031] | 0.8499 | 0.8487 | 0.8591 | 0.8719 |
| arxiv-placed | 0.421 | 0.9031 [+0.9028, +0.9033] | 0.8950 | 0.8952 | 0.8973 | 0.9005 |
| jel-placed | 0.483 | 0.9270 [+0.9268, +0.9273] | 0.8951 | 0.8946 | 0.8979 | 0.9039 |
| bk-placed | 0.488 | 0.9313 [+0.9311, +0.9316] | 0.9019 | 0.9013 | 0.9053 | 0.9110 |
| msc-placed | 0.389 | 0.9845 [+0.9844, +0.9847] | 0.9509 | 0.9511 | 0.9562 | 0.9645 |
| msc-gold | 0.129 | 0.9973 [+0.9972, +0.9974] | 0.9832 | 0.9832 | 0.9856 | 0.9884 |
| jel-gold | 0.000 | 1.0000 [+1.0000, +1.0000] | 1.0000 | 1.0000 | 1.0000 | 1.0000 |

### Book holdout (all 500 books)

| n | sparsifier | edges (dense pairs w>0) | placed: Spearman [CI] | Kendall [CI] | subsets ρ | all: Spearman [CI] | Kendall [CI] |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 500 | graph-restricted | 7297 (57607) | +0.964 [+0.89, +1.00] | +0.905 [+0.81, +1.00] | - | +0.988 [+0.96, +1.00] | +0.956 [+0.91, +1.00] |
| 500 | knn10 | 3718 (57607) | +0.964 [+0.89, +1.00] | +0.905 [+0.81, +1.00] | - | +0.988 [+0.96, +1.00] | +0.956 [+0.91, +1.00] |
| 500 | knn20 | 7297 (57607) | +0.964 [+0.89, +1.00] | +0.905 [+0.81, +1.00] | - | +0.988 [+0.96, +1.00] | +0.956 [+0.91, +1.00] |
| 500 | knn50 | 17468 (57607) | +0.964 [+0.93, +1.00] | +0.905 [+0.81, +1.00] | - | +0.988 [+0.98, +1.00] | +0.956 [+0.91, +1.00] |

Costs on the first subset of n = 500, runs in dense-cost order:

| run | coverage | dense [CI] | graph-restricted | knn10 | knn20 | knn50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| learned-own-holdout | 1.000 | 0.5222 [+0.5188, +0.5261] | 0.4432 | 0.4355 | 0.4432 | 0.4699 |
| lcc-full-gold | 0.922 | 0.7194 [+0.7160, +0.7224] | 0.7057 | 0.7010 | 0.7057 | 0.7134 |
| lcc-gold | 0.922 | 0.7208 [+0.7177, +0.7238] | 0.7076 | 0.7030 | 0.7076 | 0.7150 |
| lcc-full-placed-holdout | 0.566 | 0.9396 [+0.9373, +0.9425] | 0.9250 | 0.9137 | 0.9250 | 0.9347 |
| ilc2-placed-holdout | 0.456 | 0.9764 [+0.9744, +0.9785] | 0.9543 | 0.9423 | 0.9543 | 0.9678 |
| unesco-thesaurus-placed-holdout | 0.158 | 0.9936 [+0.9927, +0.9944] | 0.9888 | 0.9876 | 0.9888 | 0.9917 |
| learned-placed-holdout | 0.086 | 0.9950 [+0.9942, +0.9956] | 0.9915 | 0.9890 | 0.9915 | 0.9936 |
| lcc-placed-holdout | 0.084 | 0.9987 [+0.9982, +0.9991] | 0.9964 | 0.9947 | 0.9964 | 0.9979 |
| bk-placed-holdout | 0.044 | 0.9991 [+0.9987, +0.9994] | 0.9985 | 0.9985 | 0.9985 | 0.9990 |
| fdc-placed-holdout | 0.052 | 0.9993 [+0.9989, +0.9996] | 0.9981 | 0.9972 | 0.9981 | 0.9988 |

## Appendix: every score, all items / covered subset

### gold labels (source classifiers; learned: own placements)

| score | learned-own | arxiv-gold | learned-own-holdout | lcc-full-gold | lcc-gold | msc-gold | jel-gold |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| canon.exclusiveness | 1.000 / 1.000 | 0.453 / 0.453 | 1.000 / 1.000 | 0.935 / 0.935 | 0.967 / 0.967 | 0.124 / 0.124 | 0.000 / - |
| collocation.author_locality | 0.274 / 0.274 | 0.265 / 0.265 | - / - | - / - | - / - | 0.519 / 0.519 | 0.000 / - |
| collocation.author_pairs | 6990.000 / 6990.000 | 6990.000 / 6990.000 | - / - | - / - | - / - | 84.000 / 84.000 | 0.000 / - |
| collocation.author_same_class | 0.125 / 0.125 | 0.186 / 0.186 | - / - | - / - | - / - | 0.190 / 0.190 | 0.000 / - |
| collocation.author_same_class_chance | 0.015 / 0.015 | 0.027 / 0.027 | - / - | - / - | - / - | 0.002 / 0.002 | 0.500 / - |
| coverage | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 0.922 / 1.000 | 0.922 / 1.000 | 0.132 / 1.000 | 0.000 / - |
| dasgupta.binarized_cost_norm | 0.157 / 0.157 | 0.252 / 0.252 | 0.421 / 0.421 | 0.604 / 0.606 | 0.607 / 0.609 | 0.624 / 0.394 | 0.637 / - |
| dasgupta.cost | 103672286.530 / 103672286.530 | 200711464.181 / 200711464.181 | 323431.270 / 323431.270 | 515063.285 / 420385.460 | 516449.472 / 421771.647 | 592764874.565 / 2543117.650 | 602201197.389 / - |
| dasgupta.cost_norm | 0.172 / 0.172 | 0.333 / 0.333 | 0.443 / 0.443 | 0.706 / 0.706 | 0.708 / 0.709 | 0.984 / 0.679 | 1.000 / - |
| dasgupta.level1_cost_norm | 0.533 / 0.533 | 0.447 / 0.447 | 0.976 / 0.976 | 0.825 / 0.853 | 0.825 / 0.853 | 0.984 / 0.685 | 1.000 / - |
| dasgupta.level2_cost_norm | 0.305 / 0.305 | 0.414 / 0.414 | 0.926 / 0.926 | 0.717 / 0.721 | 0.717 / 0.721 | 0.984 / 0.680 | 1.000 / - |
| dasgupta.level3_cost_norm | 0.204 / 0.204 | 0.333 / 0.333 | 0.917 / 0.917 | 0.710 / 0.711 | 0.710 / 0.711 | 0.984 / 0.679 | 1.000 / - |
| dasgupta.skill | 0.976 / 0.976 | 0.786 / 0.786 | 0.797 / 0.797 | 0.421 / - | 0.419 / - | 0.018 / - | 0.000 / - |
| embed.cophenetic | 0.407 / 0.407 | 0.459 / 0.459 | 0.213 / 0.213 | 0.129 / 0.126 | 0.132 / 0.130 | 0.037 / 0.235 | 0.000 / - |
| embed.davies_bouldin_d1 | 4.541 / 4.541 | 5.017 / 5.017 | 5.354 / 5.354 | 3.086 / 3.086 | 3.086 / 3.086 | 3.501 / 3.501 | 0.000 / - |
| embed.davies_bouldin_d2 | 5.377 / 5.377 | 4.870 / 4.870 | 5.074 / 5.074 | 2.923 / 2.923 | 2.923 / 2.923 | 2.167 / 2.167 | 0.000 / - |
| embed.davies_bouldin_d3 | 4.701 / 4.701 | 4.283 / 4.283 | 2.707 / 2.707 | 2.595 / 2.595 | 2.789 / 2.789 | 1.198 / 1.198 | 0.000 / - |
| embed.silhouette_d1 | 0.087 / 0.087 | 0.041 / 0.041 | 0.040 / 0.040 | -0.149 / -0.149 | -0.149 / -0.149 | -0.115 / -0.067 | 0.000 / - |
| embed.silhouette_d2 | 0.043 / 0.043 | -0.000 / -0.000 | -0.032 / -0.032 | -0.190 / -0.190 | -0.190 / -0.190 | -0.123 / -0.157 | 0.000 / - |
| embed.silhouette_d3 | 0.027 / 0.027 | -0.066 / -0.066 | -0.142 / -0.142 | -0.203 / -0.203 | -0.177 / -0.177 | -0.048 / -0.120 | 0.000 / - |
| gold.ami_coarse | 0.347 / 0.347 | 1.000 / 1.000 | -0.003 / -0.003 | 1.000 / 1.000 | 1.000 / 1.000 | 0.152 / 0.167 | 0.001 / - |
| gold.ami_fine | 0.451 / 0.451 | 1.000 / 1.000 | 0.062 / 0.062 | 0.368 / 0.368 | 0.738 / 0.738 | 0.053 / 0.156 | 0.000 / - |
| gold.ari_coarse | 0.371 / 0.371 | 1.000 / 1.000 | 0.019 / 0.019 | 1.000 / 1.000 | 1.000 / 1.000 | 0.078 / 0.028 | 0.000 / - |
| gold.ari_fine | 0.155 / 0.155 | 1.000 / 1.000 | 0.026 / 0.026 | 0.228 / 0.228 | 0.557 / 0.557 | 0.008 / 0.020 | 0.000 / - |
| gold.completeness_coarse | 0.553 / 0.553 | 1.000 / 1.000 | 0.045 / 0.045 | 1.000 / 1.000 | 1.000 / 1.000 | 0.229 / 0.132 | 0.391 / - |
| gold.completeness_fine | 0.504 / 0.504 | 1.000 / 1.000 | 0.225 / 0.225 | 0.477 / 0.477 | 0.687 / 0.687 | 0.471 / 0.503 | 0.494 / - |
| gold.dendrogram_purity_coarse | 0.683 / 0.683 | 1.000 / 1.000 | 0.844 / 0.844 | 1.000 / 1.000 | 1.000 / 1.000 | 0.329 / 0.787 | 0.327 / - |
| gold.dendrogram_purity_fine | 0.225 / 0.225 | 1.000 / 1.000 | 0.266 / 0.266 | 1.000 / 1.000 | 1.000 / 1.000 | 0.064 / 0.409 | 0.052 / - |
| gold.hierarchical_f_coarse | 0.749 / 0.749 | 1.000 / 1.000 | 0.818 / 0.818 | 1.000 / 1.000 | 1.000 / 1.000 | 0.420 / 0.806 | 0.413 / - |
| gold.hierarchical_f_fine | 0.376 / 0.376 | 1.000 / 1.000 | 0.415 / 0.415 | 1.000 / 1.000 | 1.000 / 1.000 | 0.156 / 0.629 | 0.070 / - |
| gold.homogeneity_coarse | 0.252 / 0.252 | 1.000 / 1.000 | 0.007 / 0.007 | 1.000 / 1.000 | 1.000 / 1.000 | 0.129 / 0.531 | 0.001 / - |
| gold.homogeneity_fine | 0.524 / 0.524 | 1.000 / 1.000 | 0.222 / 0.222 | 1.000 / 1.000 | 1.000 / 1.000 | 0.142 / 0.924 | 0.000 / - |
| gold.labelled_coarse | 10000.000 / 10000.000 | 10000.000 / 10000.000 | 460.000 / 460.000 | 460.000 / 460.000 | 460.000 / 460.000 | 10000.000 / 1322.000 | 10000.000 / - |
| gold.labelled_fine | 10000.000 / 10000.000 | 10000.000 / 10000.000 | 460.000 / 460.000 | 460.000 / 460.000 | 460.000 / 460.000 | 10000.000 / 1322.000 | 10000.000 / - |
| gold.purity_coarse | 0.606 / 0.606 | 1.000 / 1.000 | 0.830 / 0.830 | 1.000 / 1.000 | 1.000 / 1.000 | 0.502 / 0.874 | 0.397 / - |
| gold.purity_fine | 0.406 / 0.406 | 1.000 / 1.000 | 0.367 / 0.367 | 1.000 / 1.000 | 1.000 / 1.000 | 0.190 / 0.852 | 0.080 / - |
| gold.v_measure_coarse | 0.347 / 0.347 | 1.000 / 1.000 | 0.012 / 0.012 | 1.000 / 1.000 | 1.000 / 1.000 | 0.165 / 0.211 | 0.001 / - |
| gold.v_measure_fine | 0.514 / 0.514 | 1.000 / 1.000 | 0.223 / 0.223 | 0.646 / 0.646 | 0.815 / 0.815 | 0.218 / 0.651 | 0.000 / - |
| granularity.classes_per_1000 | 8.800 / 8.800 | 14.800 / 14.800 | 60.000 / 60.000 | 557.484 / 557.484 | 216.920 / 216.920 | 637.670 / 637.670 | 1000.000 / - |
| granularity.empty_class_share | 0.000 / 0.000 | 0.130 / 0.130 | 0.540 / 0.540 | 0.999 / 0.999 | 0.973 / 0.973 | 0.821 / 0.821 | 0.994 / - |
| granularity.empty_leaf_share | 0.000 / 0.000 | 0.145 / 0.145 | 0.688 / 0.688 | 1.000 / 1.000 | 0.987 / 0.987 | 0.861 / 0.861 | 0.998 / - |
| granularity.works_per_leaf_mean | 113.636 / 113.636 | 67.568 / 67.568 | 16.667 / 16.667 | 1.683 / 1.683 | 4.879 / 4.879 | 1.572 / 1.572 | 1.000 / - |
| granularity.works_per_leaf_median | 98.000 / 98.000 | 29.000 / 29.000 | 2.000 / 2.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / - |
| hospitality.array | 0.000 / 0.000 | 0.384 / 0.384 | 0.000 / 0.000 | 1.000 / 1.000 | 1.000 / 1.000 | 0.740 / 0.740 | 0.126 / - |
| hospitality.ordered_siblings | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 0.910 / 0.910 | 0.993 / 0.993 | 1.000 / 1.000 | 0.999 / - |
| insertion.churn_per_insert | 1.400 / 1.400 | 2.639 / 2.639 | 0.612 / 0.612 | 0.111 / 0.111 | 0.221 / 0.221 | 0.000 / 0.000 | 0.000 / - |
| insertion.splits_per_1000 | 15.200 / 15.200 | 14.200 / 14.200 | 12.000 / 12.000 | 2.169 / 2.169 | 4.338 / 4.338 | 0.000 / 0.000 | 0.000 / - |
| mdl.data_bits_per_item | 6.265 / 6.265 | 6.029 / 6.029 | 3.325 / 3.325 | 6.922 / 7.079 | 4.933 / 4.922 | 1.811 / 9.438 | 0.003 / - |
| mdl.model_bits | 747.386 / 747.386 | 1457.752 / 1457.752 | 880.574 / 880.574 | 7768890.947 / 7768890.947 | 95001.572 / 95001.572 | 83786.288 / 83786.288 | 10138.515 / - |
| mdl.total_bits_per_item | 6.340 / 6.340 | 6.174 / 6.174 | 5.086 / 5.086 | 15544.704 / 16859.337 | 194.936 / 210.999 | 10.190 / 72.816 | 1.017 / - |
| notation.expressiveness | 1.000 / 1.000 | 0.793 / 0.793 | 1.000 / 1.000 | 0.154 / 0.154 | 0.323 / 0.323 | 0.000 / 0.000 | 1.000 / - |
| notation.length | 10.935 / 10.935 | 7.325 / 7.325 | 22.370 / 22.370 | 8.983 / 8.983 | 9.278 / 9.278 | 5.000 / 5.000 | 3.000 / - |
| notation.length_depth_spearman | 1.000 / 1.000 | 0.066 / 0.066 | 1.000 / 1.000 | 0.224 / 0.224 | 0.323 / 0.323 | 0.000 / 0.000 | 1.000 / - |
| notation.lengthening | 1.000 / 1.000 | 0.810 / 0.810 | 1.000 / 1.000 | 0.285 / 0.285 | 0.827 / 0.827 | 1.000 / 1.000 | 1.000 / - |
| notation.regularity | 1.000 / 1.000 | 0.667 / 0.667 | 1.000 / 1.000 | 0.432 / 0.432 | 0.655 / 0.655 | 0.924 / 0.924 | 1.000 / - |
| order.kendall_tau | 0.468 / 0.468 | 0.343 / 0.343 | 0.200 / 0.200 | 0.058 / 0.058 | 0.052 / 0.052 | 0.294 / 0.294 | 0.000 / - |
| order.locality | 0.793 / 0.793 | 0.606 / 0.606 | 0.346 / 0.346 | 0.121 / 0.121 | 0.103 / 0.103 | 0.499 / 0.499 | 0.000 / - |
| probe.centroid_d1 | 0.909 / 0.909 | 0.772 / 0.772 | 0.902 / 0.902 | 0.651 / 0.683 | 0.651 / 0.683 | 0.507 / 0.495 | 0.000 / - |
| probe.centroid_d2 | 0.782 / 0.782 | 0.694 / 0.694 | 0.738 / 0.738 | 0.249 / 0.255 | 0.249 / 0.255 | 0.293 / 0.273 | 0.000 / - |
| probe.centroid_d3 | 0.721 / 0.721 | 0.529 / 0.529 | 0.726 / 0.726 | 0.171 / 0.166 | 0.183 / 0.179 | 0.085 / 0.083 | 0.000 / - |
| probe.knn_d1 | 0.968 / 0.968 | 0.858 / 0.858 | 0.976 / 0.976 | 0.839 / 0.839 | 0.839 / 0.839 | 0.507 / 0.507 | 0.000 / - |
| probe.knn_d2 | 0.916 / 0.916 | 0.807 / 0.807 | 0.922 / 0.922 | 0.277 / 0.277 | 0.277 / 0.277 | 0.261 / 0.261 | 0.000 / - |
| probe.knn_d3 | 0.871 / 0.871 | 0.590 / 0.590 | 0.912 / 0.912 | 0.249 / 0.249 | 0.258 / 0.258 | 0.093 / 0.093 | 0.000 / - |
| probe.labelled_d1 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 0.922 / 1.000 | 0.922 / 1.000 | 0.132 / 1.000 | 0.000 / - |
| probe.labelled_d2 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 0.916 / 0.993 | 0.916 / 0.993 | 0.132 / 0.998 | 0.000 / - |
| probe.labelled_d3 | 1.000 / 1.000 | 0.898 / 0.898 | 1.000 / 1.000 | 0.890 / 0.965 | 0.862 / 0.935 | 0.130 / 0.985 | 0.000 / - |
| probe.majority_d1 | 0.530 / 0.530 | 0.397 / 0.397 | 0.976 / 0.976 | 0.829 / 0.829 | 0.829 / 0.829 | 0.117 / 0.117 | 0.500 / - |
| probe.majority_d2 | 0.256 / 0.256 | 0.397 / 0.397 | 0.922 / 0.922 | 0.288 / 0.288 | 0.288 / 0.288 | 0.039 / 0.039 | 0.500 / - |
| probe.majority_d3 | 0.122 / 0.122 | 0.089 / 0.089 | 0.912 / 0.912 | 0.222 / 0.222 | 0.230 / 0.230 | 0.008 / 0.008 | 0.500 / - |
| scheme.classes | 110.000 / 110.000 | 192.000 / 192.000 | 126.000 / 126.000 | 416186.000 / 416186.000 | 7392.000 / 7392.000 | 6603.000 / 6603.000 | 1015.000 / - |
| search.entropy_bits | 6.265 / 6.265 | 6.029 / 6.029 | 3.325 / 3.325 | 6.922 / 7.079 | 4.933 / 4.922 | 1.811 / 9.438 | 0.003 / - |
| search.gap_bits | 0.376 / 0.376 | 1.837 / 1.837 | 4.454 / 4.454 | 4.610 / 5.095 | 4.024 / 4.466 | 4.424 / 1.619 | 1.000 / - |
| search.hu_tucker_bits | 6.316 / 6.316 | 6.162 / 6.162 | 3.438 / 3.438 | 6.994 / 7.150 | 5.072 / 5.059 | 2.257 / 9.511 | 1.000 / - |
| search.huffman_bits | 6.294 / 6.294 | 6.067 / 6.067 | 3.364 / 3.364 | 6.940 / 7.104 | 4.972 / 4.950 | 2.256 / 9.502 | 1.000 / - |
| search.scheme_bits | 6.692 / 6.692 | 7.999 / 7.999 | 7.892 / 7.892 | 11.604 / 12.245 | 9.096 / 9.525 | 6.681 / 11.130 | 2.000 / - |
| search.worst_bits | 8.000 / 8.000 | 12.000 / 12.000 | 9.000 / 9.000 | 17.000 / 17.000 | 12.000 / 12.000 | 14.000 / 14.000 | 2.000 / - |
| structure.effective_classes | 39.725 / 39.725 | 36.482 / 36.482 | 15.062 / 15.062 | 85.198 / - | 34.986 / - | 461.501 / - | 6.000 / - |
| structure.gini | 0.678 / 0.678 | 0.795 / 0.795 | 0.802 / 0.802 | 0.718 / - | 0.791 / - | 0.599 / - | 0.000 / - |
| structure.largest_class_share | 0.133 / 0.133 | 0.137 / 0.137 | 0.125 / 0.125 | 0.182 / - | 0.239 / - | 0.039 / - | 0.167 / - |
| structure.max_branching | 345.000 / 345.000 | 797.000 / 797.000 | 93.000 / 93.000 | 68.000 / - | 89.000 / - | 25.000 / - | 1.000 / - |
| structure.max_depth | 5.000 / 5.000 | 4.000 / 4.000 | 10.000 / 10.000 | 10.000 / - | 9.000 / - | 4.000 / - | 4.000 / - |
| structure.mean_branching | 91.891 / 91.891 | 60.832 / 60.832 | 9.586 / 9.586 | 1.935 / - | 3.213 / - | 2.065 / - | 1.000 / - |
| structure.mean_depth | 4.978 / 4.978 | 3.898 / 3.898 | 8.790 / 8.790 | 5.194 / - | 4.196 / - | 1.394 / - | 1.001 / - |
| structure.nav_log_a0 | 13.470 / 13.470 | 15.172 / 15.172 | 12.764 / 12.764 | 15.351 / - | 14.755 / - | 13.893 / - | 13.288 / - |
| structure.nav_log_a1 | 18.449 / 18.449 | 18.332 / 18.332 | 19.690 / 19.690 | 19.709 / - | 18.587 / - | 15.212 / - | 14.288 / - |
| structure.nav_log_a2 | 23.427 / 23.427 | 21.493 / 21.493 | 26.616 / 26.616 | 24.067 / - | 22.419 / - | 16.532 / - | 15.288 / - |
| structure.nav_scan | 156.533 / 156.533 | 291.255 / 291.255 | 73.069 / 73.069 | 55.639 / - | 69.398 / - | 4371.911 / - | 5001.500 / - |
| structure.nodes_over_50_children | 77.000 / 77.000 | 52.000 / 52.000 | 6.000 / 6.000 | 1.000 / - | 2.000 / - | 0.000 / - | 0.000 / - |
| structure.normalized_entropy | 0.783 / 0.783 | 0.703 / 0.703 | 0.668 / 0.668 | 0.720 / - | 0.670 / - | 0.867 / - | 1.000 / - |
| structure.p50_depth | 5.000 / 5.000 | 4.000 / 4.000 | 9.000 / 9.000 | 5.000 / - | 4.000 / - | 1.000 / - | 1.000 / - |
| structure.p95_branching | 226.000 / 226.000 | 200.000 / 200.000 | 61.000 / 61.000 | 5.000 / - | 10.000 / - | 6.000 / - | 1.000 / - |
| structure.p95_depth | 5.000 / 5.000 | 4.000 / 4.000 | 10.000 / 10.000 | 7.000 / - | 6.000 / - | 4.000 / - | 1.000 / - |
| structure.root_fanout | 2.000 / 2.000 | 8.000 / 8.000 | 2.000 / 2.000 | 53.000 / - | 53.000 / - | 8738.000 / - | 10000.000 / - |
| triplet.accuracy | 0.873 / 0.873 | 0.798 / 0.798 | 0.704 / 0.704 | 0.534 / 0.537 | 0.533 / 0.537 | 0.508 / 0.641 | 0.500 / - |
| tsd.nats | 1.802 / 1.802 | 1.095 / 1.095 | 0.389 / 0.389 | 0.032 / 0.037 | 0.024 / 0.027 | 0.043 / 0.523 | 0.000 / - |
| warrant.useful_depth_share | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 0.916 / 0.993 | 0.916 / 0.993 | 0.132 / 0.998 | 0.000 / - |

### shelfmark-placed, arXiv corpus

| score | learned-placed | openalex-topics-placed | unesco-thesaurus-placed | ipc-placed | ilc2-placed | lcc-full-placed | acm-ccs-placed | fdc-placed | eurovoc-placed | lcc-placed | arxiv-placed | bk-placed | jel-placed | msc-placed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| canon.exclusiveness | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 |
| collocation.author_locality | 0.192 / 0.192 | 0.065 / 0.065 | 0.014 / 0.014 | 0.012 / 0.012 | 0.084 / 0.084 | 0.003 / 0.003 | -0.006 / -0.006 | 0.019 / 0.019 | 0.076 / 0.076 | -0.002 / -0.002 | 0.066 / 0.067 | 0.052 / 0.052 | 0.130 / 0.130 | 0.094 / 0.095 |
| collocation.author_pairs | 6990.000 / 6990.000 | 6757.000 / 6757.000 | 6900.000 / 6900.000 | 6790.000 / 6790.000 | 6990.000 / 6990.000 | 6990.000 / 6990.000 | 4892.000 / 4900.000 | 4498.000 / 4498.000 | 6766.000 / 6766.000 | 5963.000 / 5963.000 | 1676.000 / 1680.000 | 1993.000 / 1997.000 | 1969.000 / 1971.000 | 889.000 / 893.000 |
| collocation.author_same_class | 0.120 / 0.120 | 0.041 / 0.041 | 0.038 / 0.038 | 0.041 / 0.041 | 0.017 / 0.017 | 0.012 / 0.012 | 0.059 / 0.059 | 0.059 / 0.059 | 0.032 / 0.032 | 0.040 / 0.040 | 0.314 / 0.314 | 0.094 / 0.094 | 0.177 / 0.177 | 0.103 / 0.103 |
| collocation.author_same_class_chance | 0.026 / 0.026 | 0.013 / 0.013 | 0.015 / 0.015 | 0.020 / 0.020 | 0.008 / 0.008 | 0.002 / 0.002 | 0.032 / 0.032 | 0.036 / 0.036 | 0.011 / 0.011 | 0.022 / 0.022 | 0.184 / 0.184 | 0.057 / 0.057 | 0.109 / 0.109 | 0.034 / 0.034 |
| coverage | 0.999 / 1.000 | 0.973 / 1.000 | 0.973 / 1.000 | 0.980 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 0.745 / 1.000 | 0.770 / 1.000 | 0.974 / 1.000 | 0.876 / 1.000 | 0.426 / 1.000 | 0.481 / 1.000 | 0.479 / 1.000 | 0.392 / 1.000 |
| dasgupta.binarized_cost_norm | 0.305 / 0.304 | 0.556 / 0.552 | 0.584 / 0.581 | 0.618 / 0.620 | 0.553 / 0.553 | 0.606 / 0.606 | 0.588 / 0.595 | 0.605 / 0.579 | 0.607 / 0.606 | 0.615 / 0.609 | 0.608 / 0.540 | 0.610 / 0.556 | 0.606 / 0.547 | 0.621 / 0.494 |
| dasgupta.cost | 228448851.043 / 227977096.754 | 386256916.109 / 355203058.222 | 434194609.983 / 400704192.482 | 463087338.908 / 437845260.761 | 493141496.720 / 493027342.039 | 495515151.023 / 495394662.886 | 500211058.913 / 220610755.794 | 502056615.019 / 216971883.526 | 510576910.457 / 474815796.483 | 511109706.110 / 346726221.238 | 536697893.270 / 39153693.792 | 543115379.075 / 55712355.456 | 543651253.409 / 58048841.047 | 571969235.843 / 30450317.688 |
| dasgupta.cost_norm | 0.379 / 0.379 | 0.641 / 0.633 | 0.721 / 0.712 | 0.769 / 0.764 | 0.819 / 0.819 | 0.823 / 0.823 | 0.831 / 0.754 | 0.834 / 0.744 | 0.848 / 0.842 | 0.849 / 0.814 | 0.891 / 0.646 | 0.902 / 0.679 | 0.903 / 0.694 | 0.950 / 0.726 |
| dasgupta.level1_cost_norm | 0.565 / 0.565 | 0.791 / 0.794 | 0.762 / 0.756 | 0.794 / 0.790 | 0.827 / 0.827 | 0.837 / 0.837 | 0.841 / 0.775 | 0.847 / 0.772 | 0.856 / 0.850 | 0.857 / 0.826 | 0.896 / 0.697 | 0.906 / 0.712 | 0.904 / 0.703 | 0.950 / 0.730 |
| dasgupta.level2_cost_norm | 0.426 / 0.426 | 0.659 / 0.652 | 0.725 / 0.717 | 0.772 / 0.766 | 0.820 / 0.820 | 0.825 / 0.825 | 0.839 / 0.770 | 0.835 / 0.747 | 0.849 / 0.843 | 0.850 / 0.817 | 0.896 / 0.696 | 0.903 / 0.686 | 0.903 / 0.696 | 0.950 / 0.727 |
| dasgupta.level3_cost_norm | 0.388 / 0.388 | 0.643 / 0.635 | 0.722 / 0.713 | 0.769 / 0.764 | 0.819 / 0.819 | 0.823 / 0.823 | 0.831 / 0.754 | 0.834 / 0.745 | 0.848 / 0.843 | 0.849 / 0.815 | 0.891 / 0.646 | 0.903 / 0.686 | 0.903 / 0.694 | 0.950 / 0.726 |
| dasgupta.skill | 0.732 / - | 0.423 / - | 0.329 / - | 0.272 / - | 0.214 / - | 0.209 / - | 0.200 / - | 0.196 / - | 0.179 / - | 0.178 / - | 0.128 / - | 0.116 / - | 0.115 / - | 0.059 / - |
| embed.cophenetic | 0.291 / 0.283 | 0.072 / 0.062 | 0.070 / 0.051 | 0.077 / 0.084 | 0.068 / 0.071 | 0.039 / 0.030 | 0.134 / 0.103 | 0.049 / 0.059 | 0.074 / 0.079 | 0.063 / 0.055 | 0.142 / 0.213 | 0.075 / 0.115 | 0.151 / 0.216 | 0.061 / 0.143 |
| embed.davies_bouldin_d1 | 4.727 / 4.727 | 11.414 / 11.414 | 14.132 / 14.132 | 18.541 / 18.541 | 5.736 / 5.736 | 14.303 / 14.303 | 9.825 / 9.825 | 11.726 / 11.726 | 11.815 / 11.815 | 10.523 / 10.523 | 7.270 / 7.270 | 12.031 / 12.031 | 6.929 / 6.929 | 4.688 / 4.688 |
| embed.davies_bouldin_d2 | 6.098 / 6.098 | 8.874 / 8.874 | 6.574 / 6.574 | 7.097 / 7.097 | 4.903 / 4.903 | 5.999 / 5.999 | 5.256 / 5.256 | 5.636 / 5.636 | 6.382 / 6.382 | 4.945 / 4.945 | 5.307 / 5.307 | 6.003 / 6.003 | 4.844 / 4.844 | 2.890 / 2.890 |
| embed.davies_bouldin_d3 | 5.272 / 5.272 | 4.500 / 4.500 | 3.860 / 3.860 | 4.442 / 4.442 | 3.526 / 3.526 | 3.372 / 3.372 | 3.499 / 3.499 | 3.979 / 3.979 | 4.062 / 4.062 | 3.612 / 3.612 | 3.764 / 3.764 | 6.003 / 6.003 | 4.019 / 4.019 | 2.395 / 2.395 |
| embed.silhouette_d1 | 0.080 / 0.078 | -0.033 / -0.022 | -0.045 / -0.040 | -0.018 / -0.021 | -0.189 / -0.160 | -0.068 / -0.086 | -0.036 / -0.037 | -0.060 / -0.047 | -0.066 / -0.065 | -0.078 / -0.094 | -0.037 / -0.047 | -0.014 / -0.011 | -0.039 / -0.068 | -0.129 / -0.137 |
| embed.silhouette_d2 | 0.029 / 0.029 | -0.111 / -0.082 | -0.144 / -0.174 | -0.187 / -0.216 | -0.247 / -0.252 | -0.249 / -0.263 | -0.193 / -0.169 | -0.164 / -0.152 | -0.229 / -0.200 | -0.223 / -0.234 | -0.069 / -0.084 | -0.148 / -0.117 | -0.096 / -0.116 | -0.186 / -0.176 |
| embed.silhouette_d3 | -0.001 / -0.001 | -0.193 / -0.211 | -0.236 / -0.232 | -0.269 / -0.280 | -0.272 / -0.270 | -0.315 / -0.314 | -0.228 / -0.242 | -0.207 / -0.212 | -0.260 / -0.262 | -0.243 / -0.250 | -0.179 / -0.181 | -0.148 / -0.117 | -0.120 / -0.140 | -0.191 / -0.189 |
| gold.ami_coarse | 0.284 / 0.285 | 0.039 / 0.034 | 0.049 / 0.036 | 0.033 / 0.024 | 0.075 / 0.075 | 0.033 / 0.033 | 0.099 / 0.064 | 0.052 / 0.052 | 0.057 / 0.048 | 0.064 / 0.054 | 0.067 / 0.109 | 0.058 / 0.079 | 0.084 / 0.131 | 0.070 / 0.126 |
| gold.ami_fine | 0.306 / 0.307 | 0.188 / 0.191 | 0.157 / 0.155 | 0.118 / 0.116 | 0.102 / 0.102 | 0.108 / 0.108 | 0.127 / 0.131 | 0.114 / 0.135 | 0.142 / 0.140 | 0.129 / 0.134 | 0.076 / 0.139 | 0.086 / 0.149 | 0.081 / 0.142 | 0.077 / 0.166 |
| gold.ari_coarse | 0.325 / 0.325 | 0.020 / 0.015 | 0.008 / 0.001 | 0.034 / 0.031 | 0.033 / 0.033 | 0.024 / 0.024 | 0.085 / 0.057 | 0.024 / 0.023 | 0.035 / 0.032 | 0.030 / 0.020 | 0.033 / 0.178 | 0.008 / 0.075 | 0.020 / 0.133 | 0.018 / 0.041 |
| gold.ari_fine | 0.106 / 0.106 | 0.030 / 0.030 | 0.033 / 0.033 | 0.031 / 0.031 | 0.012 / 0.012 | 0.012 / 0.012 | 0.007 / 0.033 | 0.011 / 0.029 | 0.032 / 0.032 | 0.023 / 0.031 | -0.000 / 0.058 | -0.001 / 0.031 | 0.001 / 0.067 | 0.002 / 0.045 |
| gold.completeness_coarse | 0.451 / 0.455 | 0.056 / 0.053 | 0.050 / 0.037 | 0.032 / 0.024 | 0.065 / 0.065 | 0.030 / 0.030 | 0.087 / 0.057 | 0.048 / 0.051 | 0.047 / 0.040 | 0.054 / 0.047 | 0.077 / 0.121 | 0.061 / 0.077 | 0.083 / 0.117 | 0.071 / 0.104 |
| gold.completeness_fine | 0.388 / 0.388 | 0.352 / 0.357 | 0.306 / 0.307 | 0.297 / 0.297 | 0.304 / 0.304 | 0.366 / 0.366 | 0.273 / 0.277 | 0.239 / 0.269 | 0.305 / 0.306 | 0.261 / 0.271 | 0.221 / 0.304 | 0.247 / 0.315 | 0.201 / 0.278 | 0.277 / 0.374 |
| gold.dendrogram_purity_coarse | 0.590 / 0.590 | 0.382 / 0.389 | 0.347 / 0.356 | 0.344 / 0.350 | 0.345 / 0.345 | 0.343 / 0.343 | 0.353 / 0.447 | 0.342 / 0.380 | 0.345 / 0.353 | 0.341 / 0.369 | 0.351 / 0.499 | 0.344 / 0.453 | 0.346 / 0.460 | 0.330 / 0.321 |
| gold.dendrogram_purity_fine | 0.132 / 0.132 | 0.073 / 0.075 | 0.064 / 0.066 | 0.059 / 0.060 | 0.059 / 0.059 | 0.058 / 0.058 | 0.066 / 0.086 | 0.061 / 0.072 | 0.060 / 0.062 | 0.060 / 0.065 | 0.061 / 0.102 | 0.057 / 0.078 | 0.061 / 0.101 | 0.055 / 0.072 |
| gold.hierarchical_f_coarse | 0.687 / 0.687 | 0.469 / 0.477 | 0.430 / 0.432 | 0.420 / 0.422 | 0.427 / 0.427 | 0.425 / 0.425 | 0.422 / 0.486 | 0.426 / 0.450 | 0.421 / 0.423 | 0.424 / 0.433 | 0.420 / 0.503 | 0.416 / 0.502 | 0.418 / 0.493 | 0.422 / 0.430 |
| gold.hierarchical_f_fine | 0.276 / 0.276 | 0.206 / 0.210 | 0.168 / 0.170 | 0.131 / 0.133 | 0.135 / 0.135 | 0.123 / 0.124 | 0.141 / 0.170 | 0.132 / 0.153 | 0.148 / 0.150 | 0.142 / 0.154 | 0.118 / 0.190 | 0.122 / 0.194 | 0.120 / 0.177 | 0.129 / 0.223 |
| gold.homogeneity_coarse | 0.208 / 0.208 | 0.031 / 0.026 | 0.052 / 0.037 | 0.038 / 0.027 | 0.117 / 0.116 | 0.046 / 0.046 | 0.121 / 0.080 | 0.061 / 0.060 | 0.085 / 0.072 | 0.088 / 0.074 | 0.062 / 0.105 | 0.058 / 0.084 | 0.092 / 0.165 | 0.090 / 0.224 |
| gold.homogeneity_fine | 0.392 / 0.391 | 0.459 / 0.468 | 0.367 / 0.370 | 0.367 / 0.369 | 0.420 / 0.420 | 0.606 / 0.606 | 0.247 / 0.300 | 0.210 / 0.263 | 0.387 / 0.390 | 0.268 / 0.293 | 0.092 / 0.192 | 0.146 / 0.287 | 0.094 / 0.182 | 0.155 / 0.384 |
| gold.labelled_coarse | 10000.000 / 9993.000 | 10000.000 / 9732.000 | 10000.000 / 9731.000 | 10000.000 / 9799.000 | 10000.000 / 9999.000 | 10000.000 / 9999.000 | 10000.000 / 7445.000 | 10000.000 / 7696.000 | 10000.000 / 9737.000 | 10000.000 / 8765.000 | 10000.000 / 4260.000 | 10000.000 / 4806.000 | 10000.000 / 4786.000 | 10000.000 / 3916.000 |
| gold.labelled_fine | 10000.000 / 9993.000 | 10000.000 / 9732.000 | 10000.000 / 9731.000 | 10000.000 / 9799.000 | 10000.000 / 9999.000 | 10000.000 / 9999.000 | 10000.000 / 7445.000 | 10000.000 / 7696.000 | 10000.000 / 9737.000 | 10000.000 / 8765.000 | 10000.000 / 4260.000 | 10000.000 / 4806.000 | 10000.000 / 4786.000 | 10000.000 / 3916.000 |
| gold.purity_coarse | 0.583 / 0.583 | 0.406 / 0.402 | 0.430 / 0.420 | 0.433 / 0.426 | 0.482 / 0.482 | 0.454 / 0.454 | 0.516 / 0.546 | 0.437 / 0.464 | 0.481 / 0.472 | 0.494 / 0.493 | 0.421 / 0.550 | 0.440 / 0.563 | 0.446 / 0.570 | 0.476 / 0.555 |
| gold.purity_fine | 0.317 / 0.317 | 0.343 / 0.349 | 0.257 / 0.261 | 0.238 / 0.241 | 0.268 / 0.267 | 0.409 / 0.409 | 0.195 / 0.242 | 0.165 / 0.192 | 0.258 / 0.261 | 0.196 / 0.210 | 0.114 / 0.186 | 0.143 / 0.215 | 0.108 / 0.159 | 0.161 / 0.283 |
| gold.v_measure_coarse | 0.285 / 0.285 | 0.040 / 0.035 | 0.051 / 0.037 | 0.035 / 0.025 | 0.083 / 0.083 | 0.036 / 0.036 | 0.101 / 0.067 | 0.054 / 0.055 | 0.060 / 0.052 | 0.067 / 0.057 | 0.069 / 0.113 | 0.059 / 0.081 | 0.087 / 0.137 | 0.079 / 0.142 |
| gold.v_measure_fine | 0.390 / 0.390 | 0.399 / 0.405 | 0.334 / 0.335 | 0.328 / 0.329 | 0.353 / 0.353 | 0.456 / 0.456 | 0.259 / 0.288 | 0.224 / 0.266 | 0.341 / 0.343 | 0.264 / 0.281 | 0.130 / 0.236 | 0.183 / 0.300 | 0.128 / 0.220 | 0.199 / 0.379 |
| granularity.classes_per_1000 | 10.908 / 10.908 | 104.809 / 104.809 | 69.366 / 69.366 | 87.050 / 87.050 | 110.311 / 110.311 | 242.124 / 242.124 | 54.399 / 54.399 | 30.016 / 30.016 | 82.366 / 82.366 | 43.240 / 43.240 | 16.901 / 16.901 | 44.320 / 44.320 | 12.954 / 12.954 | 82.737 / 82.737 |
| granularity.empty_class_share | 0.000 / 0.000 | 0.762 / 0.762 | 0.772 / 0.772 | 0.972 / 0.972 | 0.842 / 0.842 | 0.990 / 0.990 | 0.728 / 0.728 | 0.750 / 0.750 | 0.828 / 0.828 | 0.910 / 0.910 | 0.589 / 0.589 | 0.817 / 0.817 | 0.890 / 0.890 | 0.928 / 0.928 |
| granularity.empty_leaf_share | 0.000 / 0.000 | 0.795 / 0.795 | 0.875 / 0.875 | 0.991 / 0.991 | 0.895 / 0.895 | 0.994 / 0.994 | 0.815 / 0.815 | 0.816 / 0.816 | 0.908 / 0.908 | 0.953 / 0.953 | 0.647 / 0.647 | 0.887 / 0.887 | 0.936 / 0.936 | 0.958 / 0.958 |
| granularity.works_per_leaf_mean | 84.784 / 84.784 | 6.632 / 6.632 | 13.384 / 13.384 | 10.517 / 10.517 | 8.499 / 8.499 | 4.152 / 4.152 | 19.157 / 19.157 | 32.116 / 32.116 | 10.442 / 10.442 | 23.406 / 23.406 | 58.841 / 58.841 | 22.091 / 22.091 | 57.255 / 57.255 | 10.761 / 10.761 |
| granularity.works_per_leaf_median | 72.000 / 72.000 | 2.000 / 2.000 | 2.000 / 2.000 | 2.000 / 2.000 | 2.000 / 2.000 | 1.000 / 1.000 | 2.000 / 2.000 | 4.000 / 4.000 | 2.000 / 2.000 | 3.000 / 3.000 | 7.000 / 7.000 | 3.000 / 3.000 | 5.000 / 5.000 | 2.000 / 2.000 |
| hospitality.array | 0.000 / 0.000 | 0.008 / 0.008 | 0.463 / 0.463 | 0.867 / 0.867 | 1.000 / 1.000 | 1.000 / 1.000 | 0.401 / 0.401 | 0.857 / 0.857 | 0.152 / 0.152 | 1.000 / 1.000 | 0.384 / 0.384 | 0.294 / 0.294 | 0.126 / 0.126 | 0.740 / 0.740 |
| hospitality.ordered_siblings | 1.000 / 1.000 | 1.000 / 1.000 | 0.753 / 0.753 | 0.968 / 0.968 | 0.998 / 0.998 | 0.910 / 0.910 | 1.000 / 1.000 | 0.998 / 0.998 | 0.568 / 0.568 | 0.993 / 0.993 | 1.000 / 1.000 | 1.000 / 1.000 | 0.999 / 0.999 | 1.000 / 1.000 |
| insertion.churn_per_insert | 2.602 / 2.602 | 1.097 / 1.097 | 1.316 / 1.316 | 1.779 / 1.779 | 0.661 / 0.661 | 0.137 / 0.137 | 2.296 / 2.296 | 2.892 / 2.892 | 0.923 / 0.923 | 1.878 / 1.878 | 7.844 / 7.844 | 2.644 / 2.644 | 5.157 / 5.157 | 1.210 / 1.210 |
| insertion.splits_per_1000 | 14.810 / 14.810 | 7.912 / 7.912 | 10.482 / 10.482 | 8.776 / 8.776 | 6.401 / 6.401 | 2.300 / 2.300 | 12.357 / 12.357 | 14.163 / 14.163 | 9.243 / 9.243 | 13.006 / 13.006 | 15.493 / 15.493 | 12.276 / 12.276 | 17.133 / 17.133 | 9.448 / 9.448 |
| latency.ms_per_item | 4.210 / 4.210 | 8.320 / 8.320 | 5.600 / 5.600 | 9.960 / 9.960 | 10.030 / 10.030 | 93.630 / 93.630 | 6.930 / 6.930 | 5.860 / 5.860 | 8.170 / 8.170 | 8.660 / 8.660 | 7.060 / 7.060 | 4.920 / 4.920 | 5.340 / 5.340 | 5.330 / 5.330 |
| mdl.data_bits_per_item | 6.082 / 6.078 | 7.860 / 7.893 | 7.232 / 7.248 | 7.444 / 7.452 | 8.349 / 8.348 | 9.997 / 9.996 | 5.443 / 6.210 | 5.285 / 5.856 | 7.642 / 7.668 | 6.194 / 6.451 | 2.522 / 3.611 | 3.562 / 5.333 | 2.819 / 3.802 | 3.377 / 6.157 |
| mdl.model_bits | 747.386 / 747.386 | 58672.430 / 58672.430 | 55903.519 / 55903.519 | 1305589.532 / 1305589.532 | 134603.008 / 134603.008 | 7768890.947 / 7768890.947 | 23339.690 / 23339.690 | 11080.427 / 11080.427 | 98882.447 / 98882.447 | 95001.572 / 95001.572 | 1457.752 / 1457.752 | 23677.108 / 23677.108 | 10138.515 / 10138.515 | 83786.288 / 83786.288 |
| mdl.total_bits_per_item | 6.157 / 6.153 | 13.727 / 13.922 | 12.822 / 12.993 | 138.003 / 140.689 | 21.809 / 21.810 | 786.886 / 786.963 | 7.777 / 9.345 | 6.393 / 7.295 | 17.530 / 17.823 | 15.694 / 17.290 | 2.668 / 3.953 | 5.930 / 10.260 | 3.832 / 5.921 | 11.755 / 27.552 |
| notation.expressiveness | 1.000 / 1.000 | 0.000 / 0.000 | 0.021 / 0.021 | 0.169 / 0.169 | 1.000 / 1.000 | 0.154 / 0.154 | 1.000 / 1.000 | 0.155 / 0.155 | 0.017 / 0.017 | 0.323 / 0.323 | 0.793 / 0.793 | 0.029 / 0.029 | 1.000 / 1.000 | 0.000 / 0.000 |
| notation.length | 9.156 / 9.156 | 5.478 / 5.478 | 4.770 / 4.770 | 8.624 / 8.624 | 4.199 / 4.199 | 8.338 / 8.338 | 30.580 / 30.580 | 3.288 / 3.288 | 4.905 / 4.905 | 8.369 / 8.369 | 6.051 / 6.051 | 5.000 / 5.000 | 2.658 / 2.658 | 5.000 / 5.000 |
| notation.length_depth_spearman | 1.000 / 1.000 | 1.000 / 1.000 | 0.520 / 0.520 | 0.253 / 0.253 | 0.849 / 0.849 | 0.224 / 0.224 | 1.000 / 1.000 | 0.632 / 0.632 | 0.174 / 0.174 | 0.323 / 0.323 | 0.066 / 0.066 | 0.291 / 0.291 | 1.000 / 1.000 | 0.000 / 0.000 |
| notation.lengthening | 1.000 / 1.000 | 1.000 / 1.000 | 0.944 / 0.944 | 0.988 / 0.988 | 1.000 / 1.000 | 0.285 / 0.285 | 1.000 / 1.000 | 0.934 / 0.934 | 0.909 / 0.909 | 0.827 / 0.827 | 0.810 / 0.810 | 0.984 / 0.984 | 1.000 / 1.000 | 1.000 / 1.000 |
| notation.regularity | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 0.939 / 0.939 | 0.432 / 0.432 | 1.000 / 1.000 | 0.910 / 0.910 | 0.938 / 0.938 | 0.655 / 0.655 | 0.667 / 0.667 | 0.999 / 0.999 | 1.000 / 1.000 | 0.924 / 0.924 |
| order.kendall_tau | 0.315 / 0.315 | 0.118 / 0.118 | 0.092 / 0.092 | 0.037 / 0.037 | 0.091 / 0.091 | 0.043 / 0.043 | 0.069 / 0.069 | 0.061 / 0.061 | 0.049 / 0.049 | 0.050 / 0.050 | 0.120 / 0.120 | 0.095 / 0.095 | 0.135 / 0.135 | 0.163 / 0.163 |
| order.locality | 0.566 / 0.566 | 0.204 / 0.204 | 0.116 / 0.116 | 0.061 / 0.061 | 0.163 / 0.163 | 0.079 / 0.079 | 0.096 / 0.096 | 0.117 / 0.117 | 0.106 / 0.106 | 0.080 / 0.080 | 0.166 / 0.166 | 0.175 / 0.175 | 0.213 / 0.213 | 0.271 / 0.271 |
| probe.centroid_d1 | 0.883 / 0.883 | 0.523 / 0.522 | 0.365 / 0.361 | 0.250 / 0.249 | 0.171 / 0.170 | 0.182 / 0.181 | 0.276 / 0.278 | 0.238 / 0.239 | 0.199 / 0.199 | 0.191 / 0.191 | 0.508 / 0.505 | 0.453 / 0.453 | 0.409 / 0.405 | 0.337 / 0.334 |
| probe.centroid_d2 | 0.721 / 0.721 | 0.280 / 0.279 | 0.184 / 0.185 | 0.117 / 0.114 | 0.122 / 0.122 | 0.099 / 0.101 | 0.197 / 0.199 | 0.216 / 0.211 | 0.155 / 0.155 | 0.161 / 0.155 | 0.454 / 0.451 | 0.253 / 0.251 | 0.371 / 0.367 | 0.348 / 0.340 |
| probe.centroid_d3 | 0.602 / 0.606 | 0.224 / 0.225 | 0.188 / 0.187 | 0.117 / 0.113 | 0.116 / 0.117 | 0.094 / 0.093 | 0.183 / 0.185 | 0.230 / 0.233 | 0.153 / 0.154 | 0.177 / 0.180 | 0.297 / 0.300 | 0.253 / 0.251 | 0.383 / 0.374 | 0.325 / 0.332 |
| probe.knn_d1 | 0.905 / 0.905 | 0.761 / 0.761 | 0.551 / 0.551 | 0.415 / 0.415 | 0.324 / 0.324 | 0.339 / 0.339 | 0.437 / 0.437 | 0.432 / 0.432 | 0.296 / 0.296 | 0.345 / 0.345 | 0.623 / 0.623 | 0.467 / 0.467 | 0.488 / 0.488 | 0.425 / 0.425 |
| probe.knn_d2 | 0.768 / 0.768 | 0.433 / 0.433 | 0.318 / 0.318 | 0.238 / 0.238 | 0.200 / 0.200 | 0.212 / 0.212 | 0.423 / 0.423 | 0.324 / 0.324 | 0.237 / 0.237 | 0.268 / 0.268 | 0.614 / 0.614 | 0.372 / 0.372 | 0.466 / 0.466 | 0.425 / 0.425 |
| probe.knn_d3 | 0.654 / 0.654 | 0.320 / 0.320 | 0.284 / 0.284 | 0.208 / 0.208 | 0.172 / 0.172 | 0.157 / 0.157 | 0.267 / 0.267 | 0.364 / 0.364 | 0.228 / 0.228 | 0.265 / 0.265 | 0.531 / 0.531 | 0.372 / 0.372 | 0.499 / 0.499 | 0.415 / 0.415 |
| probe.labelled_d1 | 0.999 / 1.000 | 0.973 / 1.000 | 0.973 / 1.000 | 0.980 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 0.745 / 1.000 | 0.770 / 1.000 | 0.974 / 1.000 | 0.876 / 1.000 | 0.426 / 1.000 | 0.481 / 1.000 | 0.479 / 1.000 | 0.392 / 1.000 |
| probe.labelled_d2 | 0.851 / 0.851 | 0.973 / 1.000 | 0.973 / 0.999 | 0.977 / 0.997 | 0.976 / 0.976 | 1.000 / 1.000 | 0.724 / 0.972 | 0.752 / 0.977 | 0.929 / 0.955 | 0.861 / 0.982 | 0.424 / 0.996 | 0.481 / 1.000 | 0.479 / 1.000 | 0.327 / 0.834 |
| probe.labelled_d3 | 0.792 / 0.793 | 0.912 / 0.937 | 0.966 / 0.993 | 0.971 / 0.991 | 0.886 / 0.886 | 0.998 / 0.998 | 0.690 / 0.927 | 0.552 / 0.718 | 0.918 / 0.943 | 0.778 / 0.888 | 0.365 / 0.857 | 0.481 / 1.000 | 0.315 / 0.658 | 0.281 / 0.717 |
| probe.majority_d1 | 0.525 / 0.525 | 0.740 / 0.740 | 0.476 / 0.476 | 0.329 / 0.329 | 0.182 / 0.182 | 0.266 / 0.266 | 0.350 / 0.350 | 0.289 / 0.289 | 0.195 / 0.195 | 0.210 / 0.210 | 0.521 / 0.521 | 0.243 / 0.243 | 0.223 / 0.223 | 0.097 / 0.097 |
| probe.majority_d2 | 0.302 / 0.302 | 0.271 / 0.271 | 0.126 / 0.126 | 0.133 / 0.133 | 0.075 / 0.075 | 0.127 / 0.127 | 0.360 / 0.360 | 0.148 / 0.148 | 0.059 / 0.059 | 0.121 / 0.121 | 0.523 / 0.523 | 0.187 / 0.187 | 0.203 / 0.203 | 0.105 / 0.105 |
| probe.majority_d3 | 0.173 / 0.173 | 0.096 / 0.096 | 0.090 / 0.090 | 0.088 / 0.088 | 0.063 / 0.063 | 0.038 / 0.038 | 0.127 / 0.127 | 0.200 / 0.200 | 0.050 / 0.050 | 0.085 / 0.085 | 0.456 / 0.456 | 0.187 / 0.187 | 0.309 / 0.309 | 0.123 / 0.123 |
| scheme.classes | 110.000 / 110.000 | 4798.000 / 4798.000 | 4595.000 / 4595.000 | 80145.000 / 80145.000 | 10117.000 / 10117.000 | 416186.000 / 416186.000 | 2113.000 / 2113.000 | 1097.000 / 1097.000 | 7663.000 / 7663.000 | 7392.000 / 7392.000 | 192.000 / 192.000 | 2140.000 / 2140.000 | 1015.000 / 1015.000 | 6603.000 / 6603.000 |
| search.entropy_bits | 6.082 / 6.078 | 7.860 / 7.893 | 7.232 / 7.248 | 7.444 / 7.452 | 8.349 / 8.348 | 9.997 / 9.996 | 5.443 / 6.210 | 5.285 / 5.856 | 7.642 / 7.668 | 6.194 / 6.451 | 2.522 / 3.611 | 3.562 / 5.333 | 2.819 / 3.802 | 3.377 / 6.157 |
| search.gap_bits | 0.992 / 0.587 | 3.239 / 3.392 | 3.233 / 3.412 | 4.107 / 4.168 | 4.963 / 4.862 | 4.418 / 4.300 | 2.321 / 2.757 | 2.605 / 3.144 | 2.691 / 2.804 | 2.786 / 3.173 | 3.064 / 3.629 | 1.934 / 2.769 | 2.468 / 1.896 | 3.559 / 2.310 |
| search.hu_tucker_bits | 6.278 / 6.161 | 8.013 / 8.052 | 7.452 / 7.482 | 7.611 / 7.624 | 8.537 / 8.537 | 10.110 / 10.109 | 5.598 / 6.409 | 5.445 / 6.076 | 7.831 / 7.860 | 6.394 / 6.689 | 2.670 / 3.919 | 3.644 / 5.502 | 3.076 / 4.338 | 3.498 / 6.380 |
| search.huffman_bits | 6.104 / 6.100 | 7.888 / 7.917 | 7.256 / 7.274 | 7.473 / 7.480 | 8.378 / 8.378 | 10.028 / 10.028 | 5.466 / 6.237 | 5.310 / 5.882 | 7.669 / 7.695 | 6.216 / 6.480 | 2.561 / 3.664 | 3.581 / 5.371 | 2.832 / 3.827 | 3.426 / 6.194 |
| search.scheme_bits | 7.270 / 6.748 | 11.252 / 11.443 | 10.684 / 10.894 | 11.718 / 11.792 | 13.501 / 13.398 | 14.527 / 14.409 | 7.918 / 9.166 | 8.050 / 9.220 | 10.523 / 10.665 | 9.180 / 9.862 | 5.734 / 7.549 | 5.578 / 8.271 | 5.544 / 6.234 | 7.057 / 8.690 |
| search.worst_bits | 10.000 / 9.000 | 16.000 / 16.000 | 16.000 / 16.000 | 18.000 / 18.000 | 21.000 / 20.000 | 24.000 / 23.000 | 15.000 / 15.000 | 15.000 / 15.000 | 17.000 / 17.000 | 14.000 / 14.000 | 9.000 / 9.000 | 13.000 / 12.000 | 7.000 / 7.000 | 12.000 / 12.000 |
| structure.effective_classes | 33.471 / - | 77.589 / - | 146.663 / - | 317.783 / - | 295.063 / - | 657.196 / - | 84.856 / - | 54.544 / - | 269.348 / - | 129.041 / - | 14.526 / - | 74.993 / - | 28.947 / - | 114.369 / - |
| structure.gini | 0.723 / - | 0.892 / - | 0.852 / - | 0.838 / - | 0.824 / - | 0.803 / - | 0.861 / - | 0.836 / - | 0.821 / - | 0.837 / - | 0.856 / - | 0.841 / - | 0.806 / - | 0.800 / - |
| structure.largest_class_share | 0.155 / - | 0.209 / - | 0.119 / - | 0.061 / - | 0.044 / - | 0.053 / - | 0.100 / - | 0.100 / - | 0.048 / - | 0.060 / - | 0.183 / - | 0.054 / - | 0.084 / - | 0.038 / - |
| structure.max_branching | 1183.000 / - | 554.000 / - | 535.000 / - | 794.000 / - | 417.000 / - | 121.000 / - | 877.000 / - | 1103.000 / - | 350.000 / - | 661.000 / - | 1665.000 / - | 788.000 / - | 972.000 / - | 380.000 / - |
| structure.max_depth | 5.000 / - | 5.000 / - | 9.000 / - | 10.000 / - | 13.000 / - | 11.000 / - | 7.000 / - | 7.000 / - | 8.000 / - | 10.000 / - | 4.000 / - | 8.000 / - | 4.000 / - | 4.000 / - |
| structure.mean_branching | 91.827 / - | 9.511 / - | 10.270 / - | 5.440 / - | 7.219 / - | 3.332 / - | 13.925 / - | 29.051 / - | 8.389 / - | 14.170 / - | 54.823 / - | 13.279 / - | 43.580 / - | 9.073 / - |
| structure.mean_depth | 4.383 / - | 4.446 / - | 4.898 / - | 6.314 / - | 5.090 / - | 6.059 / - | 3.612 / - | 3.213 / - | 4.976 / - | 4.076 / - | 2.215 / - | 3.152 / - | 2.272 / - | 1.999 / - |
| structure.nav_log_a0 | 26.692 / - | 22.450 / - | 21.980 / - | 22.335 / - | 18.859 / - | 17.485 / - | 20.508 / - | 22.385 / - | 20.163 / - | 21.678 / - | 18.451 / - | 18.327 / - | 17.373 / - | 15.974 / - |
| structure.nav_log_a1 | 31.075 / - | 26.850 / - | 26.682 / - | 27.315 / - | 23.384 / - | 22.864 / - | 23.733 / - | 25.564 / - | 24.799 / - | 25.356 / - | 20.341 / - | 20.936 / - | 19.410 / - | 17.824 / - |
| structure.nav_log_a2 | 35.458 / - | 31.251 / - | 31.384 / - | 32.294 / - | 27.909 / - | 28.243 / - | 26.958 / - | 28.743 / - | 29.435 / - | 29.033 / - | 22.231 / - | 23.544 / - | 21.447 / - | 19.673 / - |
| structure.nav_scan | 1036.677 / - | 311.148 / - | 304.687 / - | 322.147 / - | 135.601 / - | 52.267 / - | 1476.215 / - | 1413.473 / - | 268.704 / - | 824.857 / - | 3231.367 / - | 2736.407 / - | 2875.853 / - | 3128.469 / - |
| structure.nodes_over_50_children | 61.000 / - | 36.000 / - | 46.000 / - | 33.000 / - | 35.000 / - | 20.000 / - | 33.000 / - | 38.000 / - | 50.000 / - | 45.000 / - | 11.000 / - | 18.000 / - | 15.000 / - | 16.000 / - |
| structure.normalized_entropy | 0.747 / - | 0.618 / - | 0.717 / - | 0.748 / - | 0.771 / - | 0.776 / - | 0.699 / - | 0.712 / - | 0.779 / - | 0.748 / - | 0.612 / - | 0.723 / - | 0.713 / - | 0.768 / - |
| structure.p50_depth | 5.000 / - | 5.000 / - | 5.000 / - | 6.000 / - | 5.000 / - | 6.000 / - | 4.000 / - | 4.000 / - | 5.000 / - | 4.000 / - | 1.000 / - | 1.000 / - | 1.000 / - | 1.000 / - |
| structure.p95_branching | 233.000 / - | 35.000 / - | 41.000 / - | 18.000 / - | 28.000 / - | 12.000 / - | 59.000 / - | 153.000 / - | 33.000 / - | 72.000 / - | 185.000 / - | 46.000 / - | 218.000 / - | 32.000 / - |
| structure.p95_depth | 5.000 / - | 5.000 / - | 7.000 / - | 9.000 / - | 7.000 / - | 8.000 / - | 6.000 / - | 5.000 / - | 6.000 / - | 6.000 / - | 4.000 / - | 6.000 / - | 4.000 / - | 4.000 / - |
| structure.root_fanout | 9.000 / - | 272.000 / - | 276.000 / - | 209.000 / - | 68.000 / - | 22.000 / - | 2568.000 / - | 2314.000 / - | 284.000 / - | 1255.000 / - | 5748.000 / - | 5199.000 / - | 5231.000 / - | 6141.000 / - |
| triplet.accuracy | 0.750 / 0.751 | 0.576 / 0.580 | 0.559 / 0.559 | 0.533 / 0.533 | 0.550 / 0.550 | 0.533 / 0.533 | 0.545 / 0.560 | 0.535 / 0.557 | 0.539 / 0.541 | 0.538 / 0.545 | 0.527 / 0.590 | 0.526 / 0.585 | 0.532 / 0.602 | 0.521 / 0.614 |
| tsd.nats | 0.677 / 0.678 | 0.255 / 0.267 | 0.179 / 0.186 | 0.082 / 0.084 | 0.108 / 0.108 | 0.114 / 0.114 | 0.091 / 0.131 | 0.092 / 0.143 | 0.125 / 0.129 | 0.098 / 0.120 | 0.035 / 0.123 | 0.061 / 0.190 | 0.049 / 0.145 | 0.075 / 0.378 |
| warrant.useful_depth_share | 0.851 / 0.851 | 0.973 / 1.000 | 0.973 / 0.999 | 0.977 / 0.997 | 0.976 / 0.976 | 1.000 / 1.000 | 0.724 / 0.972 | 0.752 / 0.977 | 0.929 / 0.955 | 0.861 / 0.982 | 0.424 / 0.996 | 0.481 / 1.000 | 0.479 / 1.000 | 0.327 / 0.834 |

### shelfmark-placed, book holdout

| score | lcc-full-placed-holdout | ilc2-placed-holdout | unesco-thesaurus-placed-holdout | learned-placed-holdout | lcc-placed-holdout | fdc-placed-holdout | bk-placed-holdout |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| canon.exclusiveness | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / - | 1.000 / - |
| coverage | 0.566 / 1.000 | 0.456 / 1.000 | 0.158 / 1.000 | 0.086 / 1.000 | 0.084 / 1.000 | 0.052 / - | 0.044 / - |
| dasgupta.binarized_cost_norm | 0.611 / 0.624 | 0.613 / 0.568 | 0.646 / 0.552 | 0.642 / 0.436 | 0.646 / 0.466 | 0.645 / - | 0.644 / - |
| dasgupta.cost | 675054.541 / 117464.027 | 696443.974 / 57603.418 | 721638.430 / 2469.430 | 723622.712 / 336.342 | 727167.369 / 342.394 | 728437.063 / - | 728734.830 / - |
| dasgupta.cost_norm | 0.925 / 0.811 | 0.954 / 0.797 | 0.989 / 0.680 | 0.992 / 0.453 | 0.996 / 0.635 | 0.998 / - | 0.999 / - |
| dasgupta.level1_cost_norm | 0.929 / 0.831 | 0.955 / 0.808 | 0.989 / 0.713 | 0.992 / 0.710 | 0.996 / 0.649 | 0.998 / - | 0.999 / - |
| dasgupta.level2_cost_norm | 0.925 / 0.813 | 0.955 / 0.800 | 0.989 / 0.686 | 0.992 / 0.517 | 0.996 / 0.636 | 0.998 / - | 0.999 / - |
| dasgupta.level3_cost_norm | 0.925 / 0.812 | 0.954 / 0.797 | 0.989 / 0.681 | 0.992 / 0.460 | 0.996 / 0.636 | 0.998 / - | 0.999 / - |
| dasgupta.skill | 0.107 / - | 0.065 / - | 0.016 / - | 0.012 / - | 0.005 / - | 0.003 / - | 0.002 / - |
| embed.cophenetic | 0.042 / 0.065 | 0.056 / 0.146 | 0.024 / 0.177 | 0.025 / 0.277 | 0.009 / 0.165 | 0.015 / - | 0.013 / - |
| embed.davies_bouldin_d1 | 4.467 / 4.467 | 3.051 / 3.051 | 4.432 / 4.432 | 4.533 / 4.533 | 2.079 / 2.079 | 2.033 / - | 2.989 / - |
| embed.davies_bouldin_d2 | 2.066 / 2.066 | 2.027 / 2.027 | 1.700 / 1.700 | 3.062 / 3.062 | 1.095 / 1.095 | 1.114 / - | 1.305 / - |
| embed.davies_bouldin_d3 | 1.586 / 1.586 | 1.485 / 1.485 | 1.185 / 1.185 | 1.717 / 1.717 | 0.998 / 0.998 | 0.753 / - | 1.305 / - |
| embed.silhouette_d1 | -0.116 / -0.116 | -0.143 / -0.143 | -0.010 / -0.010 | 0.028 / 0.028 | -0.053 / -0.053 | -0.018 / - | 0.009 / - |
| embed.silhouette_d2 | -0.193 / -0.193 | -0.133 / -0.133 | -0.069 / -0.069 | 0.028 / 0.028 | 0.003 / 0.003 | 0.013 / - | 0.057 / - |
| embed.silhouette_d3 | -0.156 / -0.156 | -0.104 / -0.104 | -0.031 / -0.031 | -0.002 / -0.002 | 0.019 / 0.019 | 0.068 / - | 0.057 / - |
| gold.ami_coarse | -0.003 / 0.007 | 0.044 / 0.073 | 0.036 / 0.005 | 0.033 / 0.144 | 0.070 / 0.011 | 0.093 / - | 0.068 / - |
| gold.ami_fine | 0.033 / 0.017 | 0.053 / 0.028 | 0.096 / 0.062 | 0.023 / 0.081 | 0.071 / 0.020 | 0.067 / - | 0.056 / - |
| gold.ari_coarse | 0.008 / -0.024 | 0.058 / 0.031 | 0.190 / 0.003 | 0.116 / -0.042 | 0.170 / -0.025 | 0.192 / - | 0.164 / - |
| gold.ari_fine | 0.027 / 0.002 | 0.033 / 0.010 | 0.040 / 0.024 | 0.003 / 0.045 | 0.025 / 0.012 | 0.019 / - | 0.022 / - |
| gold.completeness_coarse | 0.052 / 0.069 | 0.090 / 0.125 | 0.092 / 0.104 | 0.099 / 0.275 | 0.167 / 0.233 | 0.234 / - | 0.207 / - |
| gold.completeness_fine | 0.349 / 0.407 | 0.344 / 0.425 | 0.441 / 0.627 | 0.339 / 0.502 | 0.481 / 0.719 | 0.550 / - | 0.526 / - |
| gold.dendrogram_purity_coarse | 0.822 / 0.802 | 0.824 / 0.788 | 0.823 / 0.629 | 0.823 / 0.689 | 0.823 / 0.531 | 0.823 / - | 0.823 / - |
| gold.dendrogram_purity_fine | 0.241 / 0.232 | 0.241 / 0.231 | 0.239 / 0.201 | 0.240 / 0.433 | 0.239 / 0.176 | 0.239 / - | 0.239 / - |
| gold.hierarchical_f_coarse | 0.816 / 0.807 | 0.813 / 0.792 | 0.809 / 0.668 | 0.816 / 0.780 | 0.805 / 0.630 | 0.803 / - | 0.804 / - |
| gold.hierarchical_f_fine | 0.397 / 0.429 | 0.404 / 0.453 | 0.395 / 0.545 | 0.400 / 0.658 | 0.390 / 0.643 | 0.390 / - | 0.390 / - |
| gold.homogeneity_coarse | 0.128 / 0.189 | 0.229 / 0.418 | 0.083 / 0.152 | 0.046 / 0.169 | 0.100 / 0.415 | 0.095 / - | 0.062 / - |
| gold.homogeneity_fine | 0.538 / 0.904 | 0.404 / 0.810 | 0.196 / 0.893 | 0.079 / 0.737 | 0.111 / 0.882 | 0.083 / - | 0.064 / - |
| gold.labelled_coarse | 460.000 / 257.000 | 460.000 / 205.000 | 460.000 / 72.000 | 460.000 / 40.000 | 460.000 / 37.000 | 460.000 / - | 460.000 / - |
| gold.labelled_fine | 460.000 / 257.000 | 460.000 / 205.000 | 460.000 / 72.000 | 460.000 / 40.000 | 460.000 / 37.000 | 460.000 / - | 460.000 / - |
| gold.purity_coarse | 0.833 / 0.817 | 0.848 / 0.829 | 0.830 / 0.639 | 0.830 / 0.650 | 0.837 / 0.649 | 0.839 / - | 0.835 / - |
| gold.purity_fine | 0.646 / 0.875 | 0.520 / 0.756 | 0.387 / 0.819 | 0.313 / 0.700 | 0.339 / 0.784 | 0.320 / - | 0.309 / - |
| gold.v_measure_coarse | 0.074 / 0.101 | 0.130 / 0.193 | 0.087 / 0.123 | 0.062 / 0.209 | 0.125 / 0.299 | 0.135 / - | 0.096 / - |
| gold.v_measure_fine | 0.424 / 0.561 | 0.372 / 0.558 | 0.272 / 0.737 | 0.128 / 0.597 | 0.181 / 0.792 | 0.145 / - | 0.115 / - |
| granularity.classes_per_1000 | 802.120 / 802.120 | 662.281 / 662.281 | 759.494 / 759.494 | 534.884 / 534.884 | 738.095 / 738.095 | 769.231 / - | 590.909 / - |
| granularity.empty_class_share | 0.998 / 0.998 | 0.965 / 0.965 | 0.968 / 0.968 | 0.600 / 0.600 | 0.989 / 0.989 | 0.966 / - | 0.979 / - |
| granularity.empty_leaf_share | 0.999 / 0.999 | 0.986 / 0.986 | 0.991 / 0.991 | 0.739 / 0.739 | 0.997 / 0.997 | 0.988 / - | 0.993 / - |
| granularity.works_per_leaf_mean | 1.248 / 1.248 | 1.529 / 1.529 | 1.333 / 1.333 | 1.870 / 1.870 | 1.379 / 1.379 | 1.312 / - | 1.692 / - |
| granularity.works_per_leaf_median | 1.000 / 1.000 | 1.000 / 1.000 | 1.000 / 1.000 | 2.000 / 2.000 | 1.000 / 1.000 | 1.000 / - | 1.000 / - |
| hospitality.array | 1.000 / 1.000 | 1.000 / 1.000 | 0.463 / 0.463 | 0.000 / 0.000 | 1.000 / 1.000 | 0.857 / - | 0.294 / - |
| hospitality.ordered_siblings | 0.910 / 0.910 | 0.998 / 0.998 | 0.753 / 0.753 | 1.000 / 1.000 | 0.993 / 0.993 | 0.998 / - | 1.000 / - |
| insertion.churn_per_insert | 0.000 / 0.000 | 0.000 / 0.000 | 0.000 / 0.000 | 0.000 / 0.000 | 0.000 / 0.000 | 0.000 / - | 0.000 / - |
| insertion.splits_per_1000 | 0.000 / 0.000 | 0.000 / 0.000 | 0.000 / 0.000 | 0.000 / 0.000 | 0.000 / 0.000 | 0.000 / - | 0.000 / - |
| latency.ms_per_item | 63.400 / 63.400 | 15.000 / 15.000 | 7.200 / 7.200 | 6.000 / 6.000 | 11.400 / 11.400 | 7.000 / - | 6.600 / - |
| mdl.data_bits_per_item | 5.327 / 7.666 | 4.081 / 6.770 | 1.541 / 5.769 | 0.792 / 4.293 | 0.822 / 4.833 | 0.512 / - | 0.405 / - |
| mdl.model_bits | 7768890.947 / 7768890.947 | 134603.008 / 134603.008 | 55903.519 / 55903.519 | 747.386 / 747.386 | 95001.572 / 95001.572 | 11080.427 / - | 23677.108 / - |
| mdl.total_bits_per_item | 15543.108 / 27459.578 | 273.287 / 597.134 | 113.348 / 713.408 | 2.287 / 21.674 | 190.825 / 2266.775 | 22.673 / - | 47.759 / - |
| notation.expressiveness | 0.154 / 0.154 | 1.000 / 1.000 | 0.021 / 0.021 | 1.000 / 1.000 | 0.323 / 0.323 | 0.155 / - | 0.029 / - |
| notation.length | 8.498 / 8.498 | 4.404 / 4.404 | 4.937 / 4.937 | 10.930 / 10.930 | 8.786 / 8.786 | 3.385 / - | 5.000 / - |
| notation.length_depth_spearman | 0.224 / 0.224 | 0.849 / 0.849 | 0.520 / 0.520 | 1.000 / 1.000 | 0.323 / 0.323 | 0.632 / - | 0.291 / - |
| notation.lengthening | 0.285 / 0.285 | 1.000 / 1.000 | 0.944 / 0.944 | 1.000 / 1.000 | 0.827 / 0.827 | 0.934 / - | 0.984 / - |
| notation.regularity | 0.432 / 0.432 | 0.939 / 0.939 | 1.000 / 1.000 | 1.000 / 1.000 | 0.655 / 0.655 | 0.910 / - | 0.999 / - |
| order.kendall_tau | 0.058 / 0.058 | 0.120 / 0.120 | 0.167 / 0.167 | 0.166 / 0.166 | 0.126 / 0.126 | 0.216 / - | -0.019 / - |
| order.locality | 0.118 / 0.118 | 0.175 / 0.175 | 0.247 / 0.247 | 0.273 / 0.273 | 0.251 / 0.251 | 0.354 / - | 0.382 / - |
| probe.centroid_d1 | 0.276 / 0.265 | 0.268 / 0.268 | 0.443 / 0.329 | 0.884 / 0.837 | 0.357 / 0.310 | 0.500 / - | 0.727 / - |
| probe.centroid_d2 | 0.163 / 0.180 | 0.301 / 0.301 | 0.304 / 0.304 | 0.651 / 0.581 | 0.263 / 0.132 | 0.435 / - | 0.682 / - |
| probe.centroid_d3 | 0.103 / 0.117 | 0.284 / 0.303 | 0.289 / 0.276 | 0.488 / 0.488 | 0.286 / 0.171 | 0.312 / - | 0.682 / - |
| probe.knn_d1 | 0.435 / 0.435 | 0.364 / 0.364 | 0.582 / 0.582 | 0.791 / 0.791 | 0.500 / 0.500 | 0.500 / - | 0.500 / - |
| probe.knn_d2 | 0.180 / 0.180 | 0.310 / 0.310 | 0.367 / 0.367 | 0.628 / 0.628 | 0.289 / 0.289 | 0.435 / - | 0.500 / - |
| probe.knn_d3 | 0.135 / 0.135 | 0.308 / 0.308 | 0.303 / 0.303 | 0.605 / 0.605 | 0.314 / 0.314 | 0.312 / - | 0.500 / - |
| probe.labelled_d1 | 0.566 / 1.000 | 0.456 / 1.000 | 0.158 / 1.000 | 0.086 / 1.000 | 0.084 / 1.000 | 0.052 / - | 0.044 / - |
| probe.labelled_d2 | 0.566 / 1.000 | 0.432 / 0.947 | 0.158 / 1.000 | 0.086 / 1.000 | 0.076 / 0.905 | 0.046 / - | 0.044 / - |
| probe.labelled_d3 | 0.564 / 0.996 | 0.402 / 0.882 | 0.152 / 0.962 | 0.086 / 1.000 | 0.070 / 0.833 | 0.032 / - | 0.044 / - |
| probe.majority_d1 | 0.392 / 0.392 | 0.180 / 0.180 | 0.241 / 0.241 | 0.698 / 0.698 | 0.214 / 0.214 | 0.192 / - | 0.591 / - |
| probe.majority_d2 | 0.110 / 0.110 | 0.125 / 0.125 | 0.114 / 0.114 | 0.279 / 0.279 | 0.105 / 0.105 | 0.174 / - | 0.318 / - |
| probe.majority_d3 | 0.064 / 0.064 | 0.124 / 0.124 | 0.066 / 0.066 | 0.140 / 0.140 | 0.086 / 0.086 | 0.188 / - | 0.318 / - |
| scheme.classes | 416186.000 / 416186.000 | 10117.000 / 10117.000 | 4595.000 / 4595.000 | 110.000 / 110.000 | 7392.000 / 7392.000 | 1097.000 / - | 2140.000 / - |
| search.entropy_bits | 5.327 / 7.666 | 4.081 / 6.770 | 1.541 / 5.769 | 0.792 / 4.293 | 0.822 / 4.833 | 0.512 / - | 0.405 / - |
| search.gap_bits | 2.322 / 2.449 | 2.472 / 1.610 | 1.684 / 0.785 | 0.932 / 0.512 | 2.730 / 0.714 | 2.806 / - | 0.964 / - |
| search.hu_tucker_bits | 5.366 / 7.714 | 4.108 / 6.816 | 1.922 / 5.835 | 1.378 / 4.395 | 1.414 / 4.929 | 1.220 / - | 1.148 / - |
| search.huffman_bits | 5.360 / 7.703 | 4.100 / 6.798 | 1.922 / 5.835 | 1.374 / 4.349 | 1.412 / 4.905 | 1.220 / - | 1.148 / - |
| search.scheme_bits | 7.688 / 10.163 | 6.580 / 8.425 | 3.606 / 6.620 | 2.310 / 4.907 | 4.144 / 5.643 | 4.026 / - | 2.112 / - |
| search.worst_bits | 15.000 / 16.000 | 13.000 / 13.000 | 9.000 / 9.000 | 7.000 / 6.000 | 8.000 / 8.000 | 5.000 / - | 5.000 / - |
| structure.effective_classes | 340.762 / - | 188.146 / - | 93.818 / - | 26.076 / - | 63.041 / - | 31.051 / - | 30.788 / - |
| structure.gini | 0.492 / - | 0.527 / - | 0.439 / - | 0.525 / - | 0.329 / - | 0.314 / - | 0.439 / - |
| structure.largest_class_share | 0.071 / - | 0.042 / - | 0.058 / - | 0.175 / - | 0.063 / - | 0.072 / - | 0.115 / - |
| structure.max_branching | 13.000 / - | 25.000 / - | 8.000 / - | 5.000 / - | 6.000 / - | 4.000 / - | 7.000 / - |
| structure.max_depth | 11.000 / - | 11.000 / - | 7.000 / - | 5.000 / - | 8.000 / - | 7.000 / - | 7.000 / - |
| structure.mean_branching | 1.384 / - | 1.559 / - | 1.500 / - | 1.932 / - | 1.380 / - | 1.486 / - | 1.432 / - |
| structure.mean_depth | 4.140 / - | 2.944 / - | 1.652 / - | 1.342 / - | 1.286 / - | 1.138 / - | 1.226 / - |
| structure.nav_log_a0 | 11.261 / - | 10.338 / - | 9.427 / - | 9.275 / - | 9.077 / - | 9.015 / - | 9.061 / - |
| structure.nav_log_a1 | 13.983 / - | 12.464 / - | 10.825 / - | 10.567 / - | 10.207 / - | 10.097 / - | 10.143 / - |
| structure.nav_log_a2 | 16.705 / - | 14.590 / - | 12.223 / - | 11.859 / - | 11.337 / - | 11.179 / - | 11.225 / - |
| structure.nav_scan | 124.879 / - | 155.489 / - | 216.031 / - | 231.640 / - | 236.806 / - | 242.666 / - | 242.253 / - |
| structure.nodes_over_50_children | 0.000 / - | 0.000 / - | 0.000 / - | 0.000 / - | 0.000 / - | 0.000 / - | 0.000 / - |
| structure.normalized_entropy | 0.892 / - | 0.891 / - | 0.911 / - | 0.862 / - | 0.948 / - | 0.951 / - | 0.906 / - |
| structure.p50_depth | 5.000 / - | 1.000 / - | 1.000 / - | 1.000 / - | 1.000 / - | 1.000 / - | 1.000 / - |
| structure.p95_branching | 3.000 / - | 4.000 / - | 3.000 / - | 4.000 / - | 3.000 / - | 3.000 / - | 3.000 / - |
| structure.p95_depth | 9.000 / - | 7.000 / - | 6.000 / - | 5.000 / - | 4.000 / - | 2.000 / - | 1.000 / - |
| structure.root_fanout | 236.000 / - | 300.000 / - | 427.000 / - | 459.000 / - | 470.000 / - | 482.000 / - | 481.000 / - |
| triplet.accuracy | 0.512 / 0.528 | 0.514 / 0.552 | 0.505 / 0.593 | 0.503 / 0.641 | 0.502 / 0.611 | 0.501 / - | 0.501 / - |
| tsd.nats | 0.045 / 0.116 | 0.056 / 0.220 | 0.017 / 0.374 | 0.016 / 0.590 | 0.010 / 0.421 | 0.007 / - | 0.006 / - |
| warrant.useful_depth_share | 0.566 / 1.000 | 0.432 / 0.947 | 0.158 / 1.000 | 0.086 / 1.000 | 0.076 / 0.905 | 0.046 / - | 0.044 / - |

