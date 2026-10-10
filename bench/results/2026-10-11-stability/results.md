# Rank stability report

Threshold for a robust ranking claim: Spearman ρ ≥ 0.8 for every pair of controlled settings within each objective.

## Design

The run scores all 13 schemes on the 10,000-item corpus for three embedding models, k ∈ {10, 20, 50}, three graph thresholds (the source graph, then the 25th and 50th percentile positive-edge offsets), and lexical@2 / shortlist@2 placements. Each score row supplies Dasgupta cost, Moseley–Wang revenue, skill, triplet accuracy, and shelf-order locality. Skill uses the unthresholded average-linkage reference for the same embedding and k, held fixed through the threshold sweep.

The rank matrix and every pairwise setting comparison within each objective are in `results.json`. Confidence intervals resample the same 13 schemes as paired ranking units (200 draws, seed 20261011). Rank flips list each scheme whose position changes from the MiniLM k=20 q00 lexical baseline and every pair whose ordering reverses.

## Ranking checks and report 9 audit

The first five rows are paper-corpus stability diagnostics for the five requested objectives. The remaining rows map report 9's book-specific rank claims to this matrix and mark them not cleared where its book schemes, cut variants, or classifier outcomes are absent.

| Claim | Objective | Minimum Spearman ρ across setting pairs | Threshold | Result |
| --- | --- | ---: | ---: | --- |
| Paper corpus: FDC versus LCC scheme-rank stability by Dasgupta cost | `dasgupta_cost_norm` | 0.038 | 0.800 | NOT CLEARED |
| Paper corpus: Moseley–Wang revenue rank stability | `moseley_wang_revenue` | 0.038 | 0.800 | NOT CLEARED |
| Paper corpus: skill rank stability | `skill` | 0.038 | 0.800 | NOT CLEARED |
| Paper corpus: triplet-based scheme-rank stability | `triplet_accuracy` | -0.280 | 0.800 | NOT CLEARED |
| Paper corpus: shelf-order locality-rank stability | `shelf_order_locality` | 0.549 | 0.800 | NOT CLEARED |
| Report 9 §§5.1 and 6.1: book FDC versus LCC scheme-choice claim | `dasgupta_cost_norm` | n/a | 0.800 | NOT CLEARED |
| Report 9 §4.3: book triplet and locality rankings | `triplet_accuracy / shelf_order_locality` | n/a | 0.800 | NOT CLEARED |
| Sections 5.1 and 6.1: Dewey-section versus FDC-depth-three ranking | `dasgupta_cost_norm` | n/a | 0.800 | NOT CLEARED |
| Section 6.1: full-depth Dewey versus section ranking | `dasgupta_cost_norm` | n/a | 0.800 | NOT CLEARED |
| Sections 4.4 and 6.1: learned scheme and hybrid rankings | `dasgupta_cost_norm` | n/a | 0.800 | NOT CLEARED |
| Sections 6.2 and 6.3: cut-depth and classifier-pipeline rankings | `skill` | n/a | 0.800 | NOT CLEARED |
| Section 6.1: locality-weighted composite J ranking | `composite_J` | n/a | 0.800 | NOT CLEARED |
| Section 8: semantic-classifier book threshold | `coverage_and_agreement` | n/a | 0.800 | NOT CLEARED |

The report-9 checks cover its scheme, cut-depth, secondary-score, classifier-pipeline, and learned/hybrid ranking claims. `Not in this matrix` means the claim's book-only candidates or composite objective are absent from the 13-scheme paper-corpus factorial; it is not evidence for the claim. Individual pair reversals appear in `rank_flips`.

## Reproduction

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-bench
python3 bench/results/2026-10-11-stability/generate_stability.py --bench-bin target/release/shelfmark-bench --jobs 8 --bootstrap 200
```

The script reads the published graph files, schemes, and tier placements. Temporary thresholded graphs are written under `/work/artifacts` and removed when scoring completes.
