#!/usr/bin/env bash
# Run shelfmark over a gold set and write mc-eval predictions (one line per
# item). Needs jq and a built shelfmark binary.
#
#   scripts/shelfmark-predictions.sh GOLD OUT [PROFILE] [TARGET]
#
# SHELFMARK names the binary (default: shelfmark on PATH). SHELFMARK_SCHEME
# names a scheme file to compile into the store first; with the bundled
# scheme's id it replaces that scheme for the run. Each run uses a fresh
# store, so by default no identifier table is loaded: the Open Library gold
# labels come from the same records an identifier table is built from, and
# scoring the identifier tier on them would score the table against itself.
# SHELFMARK_TABLE names an identifier table (shelfmark JSONL) to load
# anyway; build it from a dump without the gold records to score the tier
# honestly. Title, text and typed identifiers (isbn, lccn, doi, arxiv) go in;
# the record's source URL does not, because it is where the label came from.
# A zero-candidate best guess is predicted as its placed class.
# Latency is the placement's own cascade cost, so a memoized rerun and
# process start-up do not count.
set -euo pipefail
gold=${1:?gold ndjson}
out=${2:?output ndjson}
profile=${3:-profile:lexical@1}
target=${4:-}
bin=${SHELFMARK:-shelfmark}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

jq -c --arg profile "$profile" --arg target "$target" '
  def ids: .identifiers as $i
    | [ ($i.isbn // empty | "isbn:" + .), ($i.isbn10 // empty | "isbn:" + .),
        ($i.lccn // empty | "lccn:" + .), ($i.doi // empty | "doi:" + .),
        ($i.arxiv // empty | "arxiv:" + .) ];
  {verb: "classify",
   args: ({text: (.text // ""), title: .title, identifiers: ids, profile: $profile}
          + (if $target == "" then {} else {target: ($target | tonumber)} end)
          | with_entries(select(.value != null and .value != [])))}
' "$gold" > "$work/requests.ndjson"
jq -r .id "$gold" > "$work/ids.txt"

if [ -n "${SHELFMARK_TABLE:-}" ]; then
  mkdir -p "$work/store/tables"
  cp "$SHELFMARK_TABLE" "$work/store/tables/"
fi
if [ -n "${SHELFMARK_SCHEME:-}" ]; then
  rev=$("$bin" --store "$work/store" scheme compile "$SHELFMARK_SCHEME" | jq -r .data.revision)
  "$bin" --store "$work/store" scheme compile "$SHELFMARK_SCHEME" --commit --expected-revision "$rev" > /dev/null
fi
"$bin" --store "$work/store" batch --requests "$work/requests.ndjson" --out "$work/lines.ndjson" > "$work/summary.json" || true
want=$(wc -l < "$work/ids.txt")
got=$(wc -l < "$work/lines.ndjson")
if [ "$want" != "$got" ]; then
  echo "shelfmark-predictions: $want gold items but $got batch lines" >&2
  exit 1
fi

paste -d '\t' "$work/ids.txt" "$work/lines.ndjson" | while IFS=$'\t' read -r id line; do
  jq -c --arg id "$id" --arg profile "$profile" '
    if .ok then .data.placement as $p
      | {item: $id, profile: $profile,
         tier: ($p.trace.deciding_tier // "none"),
         candidates: ([$p.trace.candidates[] | {class, score}]
                      | if length == 0 then [{class: $p.class, score: $p.confidence}] else . end),
         confidence: $p.confidence,
         flagged: ($p.status == "flagged"),
         latency_ms: (($p.trace.cost.micros // 0) / 1000)}
    else {item: $id, profile: $profile, tier: "none", candidates: [], confidence: 0,
          flagged: true, latency_ms: 0}
    end' <<<"$line"
done > "$out"
jq -c . "$work/summary.json" >&2
