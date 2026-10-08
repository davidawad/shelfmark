#!/usr/bin/env bash
# measure-sizes.sh <build-out-dir>
# Sizes of the full table and of the projections a runtime would embed.
# Writes projections next to the build output and prints a TSV report.
set -euo pipefail
out=$1
cd "$out"
# decision.tsv: one row per (key, scheme) -- the best-supported full number
# (ties: lexically first), i.e. what the identifier tier would answer.
awk -F'\t' 'NR > 1 {
  k = $1 FS $2
  if (k != cur) { if (cur != "") print best; cur = k; bs = -1 }
  if ($5 > bs) { bs = $5; best = $1 FS $2 FS $4 }
} END { if (cur != "") print best }' lookup.tsv > decision.tsv
grep -P '^isbn:[0-9]+\tlcc\t' decision.tsv > decision-isbn-lcc.tsv || true
# Prefix-only form (the decision row's class prefix): enough to place a Work.
awk -F'\t' 'NR > 1 { k = $1 FS $2; if (k != cur) { cur = k; bs = -1 } if ($5 > bs) { bs = $5; p[k] = $3 } }
  END { for (k in p) print k FS p[k] }' lookup.tsv | LC_ALL=C sort > decision-prefix.tsv
printf 'file\trows\traw_bytes\tgzip9_bytes\txz9_bytes\n'
for f in lookup.tsv conflicts.tsv decision.tsv decision-isbn-lcc.tsv decision-prefix.tsv; do
  rows=$(wc -l < "$f")
  raw=$(wc -c < "$f")
  gz=$(gzip -9 -c "$f" | wc -c)
  zs=$(xz -9 -T0 -c "$f" | wc -c)
  printf '%s\t%s\t%s\t%s\t%s\n' "$f" "$rows" "$raw" "$gz" "$zs"
done
