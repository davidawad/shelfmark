# Reproducing the rank-stability sweep

Run from the repository root on Linux. The published graph files, scheme
definitions, and lexical / shortlist placement files are inputs; no model
download is needed for the scoring sweep.

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-bench
python3 bench/results/2026-10-11-stability/generate_stability.py \
  --bench-bin target/release/shelfmark-bench --jobs 8 --bootstrap 200
```

The generator evaluates all 13 schemes for each of 3 embedding models, 3 k
values, 3 graph thresholds, and 2 classifier tiers (702 score rows). For each
graph, it takes the source kNN graph and subtracts the 25th or 50th percentile
of positive edge weights, dropping weights that become zero. This defines the
threshold sweep without rebuilding the nearest-neighbor graph. It uses the
same unthresholded average-linkage reference cost for skill throughout each
threshold sweep.

Every objective's rankings are compared for all 1,431 pairs of settings.
Intervals use 200 paired bootstrap resamples of the 13 schemes, seed
20261011. The claim threshold is Spearman ρ ≥ 0.8. The resulting rank matrix,
all 7,155 objective-specific pair comparisons, 95% intervals, baseline rank
flips, and report 9 audit are in `results.json` and summarized in `results.md`.

Temporary thresholded graphs go under `/work/artifacts` and are removed when
the generator exits. Use `--reuse-results` to regenerate the Markdown and
report 9 claim audit from the existing `results.json` without rescoring.
