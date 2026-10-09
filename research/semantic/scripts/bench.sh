#!/usr/bin/env bash
# Run every candidate profile in a fresh process (so RSS/peak are per-profile), after a
# prefetch pass that keeps download time out of load timings. Controlled benchmark environment only; never
# run in a controlled benchmark environment.
#
#   scripts/bench.sh [filter-regex]        # from research/semantic/
set -euo pipefail
cd "$(dirname "$0")/.."
BIN=${SEMSPIKE_BIN:-${CARGO_TARGET_DIR:-target}/release/semspike}
export FASTEMBED_CACHE_DIR=${FASTEMBED_CACHE_DIR:-$HOME/.cache/fastembed}
OUT=results
mkdir -p "$OUT"
FILTER=${1:-.}

# Labels are embedded as instructed queries and items as passages (--flip): on this task it
# lifted Qwen3-Embedding sub@1 from 20% to 47% and helped every model (see README).
# fe-qwen3:bf16 is omitted: candle's CPU backend has no BF16 matmul.
# The benchmark host exposes 12 vCPUs but has a 4-CPU cgroup quota (cpu.max 400000/100000), so
# the 4core:* rows (taskset 0-3, 4 threads) are the representative numbers.
# name | embedder | reranker (- for none) | cpu list (- for all)
PROFILES=(
  "lite-minilm-l6-q|fe:all-minilm-l6-v2-q|-|-"
  "lite-bge-small|fe:bge-small-en-v1.5|-|-"
  "lite-bge-small-q|fe:bge-small-en-v1.5-q|-|-"
  "lite-arctic-xs-q|fe:snowflake-arctic-xs-q|-|-"
  "lite-e5-small-multi|fe:multilingual-e5-small|-|-"
  "lite-gemma300m-q|fe:embeddinggemma-300m-q|-|-"
  "qwen-fe-f32|fe-qwen3:f32|-|-"
  "qwen-f32|qwen3:f32|-|-"
  "qwen-q8|qwen3:q8_0|-|-"
  "qwen-q6k|qwen3:q6k|-|-"
  "qwen-q4k|qwen3:q4k|-|-"
  "qwen-q8+rr-qwen-f32|qwen3:q8_0|qwen3:f32|-"
  "qwen-q8+rr-qwen-q8|qwen3:q8_0|qwen3:q8_0|-"
  "qwen-q8+rr-qwen-q4k|qwen3:q8_0|qwen3:q4k|-"
  "qwen-q8+rr-jina-turbo|qwen3:q8_0|fe:jina-reranker-v1-turbo-en|-"
  "qwen-q8+rr-bge-base|qwen3:q8_0|fe:bge-reranker-base|-"
  "lite-bge-small-q+rr-jina-turbo|fe:bge-small-en-v1.5-q|fe:jina-reranker-v1-turbo-en|-"
  "lite-bge-small-q+rr-qwen-q8|fe:bge-small-en-v1.5-q|qwen3:q8_0|-"
  "4core:lite-bge-small-q|fe:bge-small-en-v1.5-q|-|0-3"
  "4core:qwen-q8|qwen3:q8_0|-|0-3"
  "4core:qwen-q8+rr-qwen-q8|qwen3:q8_0|qwen3:q8_0|0-3"
  "4core:lite-minilm-l6-q|fe:all-minilm-l6-v2-q|-|0-3"
  "4core:lite-gemma300m-q|fe:embeddinggemma-300m-q|-|0-3"
  "4core:qwen-q4k|qwen3:q4k|-|0-3"
  "4core:qwen-q8+rr-jina-turbo|qwen3:q8_0|fe:jina-reranker-v1-turbo-en|0-3"
)

for row in "${PROFILES[@]}"; do
  IFS='|' read -r name emb rr cpus <<<"$row"
  [[ "$name" =~ $FILTER ]] || continue
  file="$OUT/${name//[:+]/_}.json"
  pin=(); [[ "$cpus" != - ]] && pin=(taskset -c "$cpus")
  rrargs=(); [[ "$rr" != - ]] && rrargs=(--reranker "$rr")
  echo ">> $name" >&2
  if ! "$BIN" load --embedder "$emb" "${rrargs[@]}" >/dev/null; then
    echo "FAILED load $name" >&2; continue
  fi
  RAYON_NUM_THREADS=$([[ "$cpus" != - ]] && echo 4 || nproc) \
    "${pin[@]}" "$BIN" run --profile "$name" --embedder "$emb" "${rrargs[@]}" --flip --out "$file" \
    || echo "FAILED run $name" >&2
done
"$BIN" report "$OUT"/*.json > "$OUT/table.md"
echo "wrote $OUT/table.md" >&2
