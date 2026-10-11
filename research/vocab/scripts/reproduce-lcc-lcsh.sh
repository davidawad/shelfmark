#!/usr/bin/env bash
# Reproduce every LCC and LCSH scheme file from public Library of Congress
# sources, then measure candidate "full" bundled schemes. Heavy (a 230 MB
# dump, 44 schedule PDFs, several release builds): run it on a box, never
# a laptop.
#
#   research/vocab/scripts/reproduce-lcc-lcsh.sh OUT [CACHE]
#   REPRODUCE_ONLY=1 research/vocab/scripts/reproduce-lcc-lcsh.sh OUT   # steps 1-3 only
#
# Writes a report (OUT/report.json plus logs) and, for review, the fresh
# outline lock and id.loc.gov classification records (OUT/outline-data,
# OUT/outline-records). It never edits committed files. Steps that fail are
# recorded in the report and the script goes on.
#
# 1. tools/lcc-outline fetch (relocks into a scratch data dir) + build there;
#    compares every source sha256 and the built lcc.json with the committed ones.
# 2. schemes/lcc-full fetch (checks the 44 Text PDFs against the lock) + check,
#    against the outline file the lock pins.
# 3. build-lcc-lcsh.sh check: the bundled lcc-lcsh.json.gz from the locked dump.
# 4. Variants: the LCSH table folded into lcc.json and into lcc-full, with and
#    without broader-heading inheritance: sizes, coverage, and gold-set
#    accuracy (research/eval starter-v1, profile:lexical@2) when each replaces
#    the bundled lcc@2024 for a run.
# 5. For the lcc-full variants: a shelfmark binary with the variant bundled:
#    binary size, cold and warm classify wall time and peak RSS.
set -uo pipefail
out=$(mkdir -p "${1:?usage: reproduce-lcc-lcsh.sh OUT [CACHE]}" && cd "$1" && pwd)
cache=${2:-$HOME/lcc-cache}
root=$(cd "$(dirname "$0")/../../.." && pwd)
mkdir -p "$cache" "$out/logs"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$cache/target}
report=$out/report.ndjson
: > "$report"
note() {
  jq -nc --arg step "$1" --arg status "$2" --argjson data "${3:-null}" '{step:$step,status:$status,data:$data}' >> "$report"
  echo "== $1: $2" >&2
}
run() {
  local name=$1
  shift
  if "$@" > "$out/logs/$name.log" 2>&1; then note "$name" ok; else
    note "$name" "failed ($?)"
    return 1
  fi
}
sha256() { sha256sum "$1" | cut -d' ' -f1; }
timed() { # timed NAME CMD... -> {rc, wall_s, max_rss_kb}; stdout+stderr to logs/NAME.log
  local name=$1; shift
  python3 -I -c '
import json, resource, subprocess, sys, time
log = open(sys.argv[1], "wb")
t = time.monotonic()
rc = subprocess.call(sys.argv[2:], stdout=log, stderr=subprocess.STDOUT)
wall = time.monotonic() - t
rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
print(json.dumps({"rc": rc, "wall_s": round(wall, 3), "max_rss_kb": rss}))
' "$out/logs/$name.log" "$@"
}

# 1. outline
mkdir -p "$out/outline-data"
cp "$root/data/schemes/lcc/sources.lock.json" "$out/outline-data/committed.lock.json"
(cd "$root/tools/lcc-outline" && cargo build --release) > "$out/logs/outline-build.log" 2>&1
lo=$CARGO_TARGET_DIR/release/lcc-outline
if run outline-fetch "$lo" fetch --cache "$cache/outline" --data "$out/outline-data" &&
  run outline-build "$lo" build --cache "$cache/outline" --data "$out/outline-data"; then
  diff=$(jq -n --slurpfile a "$out/outline-data/committed.lock.json" --slurpfile b "$out/outline-data/sources.lock.json" '
    ($a[0].sources | map({(.file): .sha256}) | add) as $old
    | ($b[0].sources | map({(.file): .sha256}) | add) as $new
    | {same: [$old | keys[] | select($new[.] == $old[.])] | length,
       changed: [$old | keys[] | select($new[.] != null and $new[.] != $old[.])],
       gone: [$old | keys[] | select($new[.] == null)],
       added: [$new | keys[] | select($old[.] == null)]}')
  same_scheme=$(cmp -s "$out/outline-data/lcc.json" "$root/data/schemes/lcc/lcc.json" && echo true || echo false)
  note outline-compare ok "$(jq -c --argjson s "$same_scheme" '. + {lcc_json_identical: $s}' <<< "$diff")"
  mkdir -p "$out/outline-records" && cp -r "$cache/outline/id-classification" "$out/outline-records/"
fi
# The committed lock and records against the PDFs and dump the fetch cached.
run outline-check "$lo" check --cache "$cache/outline"

# 2. lcc-full
(cd "$root/schemes/lcc-full" && cargo build --release) > "$out/logs/full-build.log" 2>&1
lf=$CARGO_TARGET_DIR/release/lcc-full
run full-fetch "$lf" fetch --cache "$cache/text"
# lcc-full was built on the outline as committed before the path in its
# source.description changed (the lock's outline_sha256, b21e4c66...); the
# classes are the same. That file is today's lcc.json with the old path.
sed 's|Built by tools/lcc-outline;|Built by src/shelfmark/tools/lcc-outline;|' \
  "$root/data/schemes/lcc/lcc.json" > "$cache/lcc-outline-for-full.json"
run full-check "$lf" check --cache "$cache/text" --outline "$cache/lcc-outline-for-full.json"

# 3. bundled lcc-lcsh (the dump is the one the outline fetch cached)
run lcsh-check "$root/research/vocab/scripts/build-lcc-lcsh.sh" "$cache/outline" check
cp "$cache/outline/stats/lcsh-stats.json" "$out/lcsh-stats.json" 2> /dev/null
note lcsh-dump ok "$(jq -nc --arg s "$(sha256 "$cache/outline/subjects.madsrdf.nt.gz")" --argjson b "$(stat -c %s "$cache/outline/subjects.madsrdf.nt.gz")" '{sha256:$s, bytes:$b}')"

if [ "${REPRODUCE_ONLY:-0}" = 1 ]; then
  jq -s . "$report" > "$out/report.json"
  echo "reproduce-lcc-lcsh: report in $out/report.json (REPRODUCE_ONLY: no variants)" >&2
  exit 0
fi

# 4. variants
vi=$CARGO_TARGET_DIR/release/vocab-ingest
table=$cache/outline/lcsh.tsv.gz
gzip -dc "$root/schemes/lcc-full/lcc-full.json.gz" > "$cache/lcc-full.json"
(cd "$root/research/eval" && cargo build --release) > "$out/logs/eval-build.log" 2>&1
(cd "$root" && cargo build --release -p shelfmark-cli) > "$out/logs/cli-build.log" 2>&1
cp "$CARGO_TARGET_DIR/release/shelfmark" "$cache/shelfmark-baseline"
gold=$root/research/eval/gold/starter-v1.ndjson
score() { # score NAME [SCHEME_FILE]
  local name=$1 scheme=${2:-}
  local t
  t=$(SHELFMARK="$cache/shelfmark-baseline" SHELFMARK_SCHEME="$scheme" timed "pred-$name" \
    "$root/research/eval/scripts/shelfmark-predictions.sh" "$gold" "$out/pred-$name.ndjson" profile:lexical@2)
  "$CARGO_TARGET_DIR/release/mc-eval" run --gold "$gold" --predictions "$out/pred-$name.ndjson" > "$out/eval-$name.json" 2> "$out/logs/eval-$name.log"
  note "eval-$name" ok "$(jq -c --argjson t "$t" '{time:$t, levels:[.profiles[0].levels[]? | {level, n, top1:(.top1.value // .top1), topk:(.topk.value // .topk)}]}' "$out/eval-$name.json" 2> /dev/null || echo "$t")"
}
score baseline
for v in "outline-inherit3 lcc.json 3" "full-direct lcc-full 0" "full-inherit3 lcc-full 3"; do
  set -- $v
  name=$1 base=$2 inherit=$3
  src=$root/data/schemes/lcc/lcc.json
  [ "$base" = lcc-full ] && src=$cache/lcc-full.json
  if run "enrich-$name" "$vi" enrich-scheme --scheme "$src" --table "$table" --inherit "$inherit" \
    --canonical true --out "$cache/$name.json" --stats "$out/enrich-$name.json"; then
    # As a candidate for the bundled scheme: the bundled id, lcc@2024.
    jq -c '.name = "lcc" | .version = "2024"' "$cache/$name.json" | head -c -1 > "$cache/$name.as-lcc.json"
    gzip -9n -c "$cache/$name.as-lcc.json" > "$cache/$name.as-lcc.json.gz"
    note "size-$name" ok "$(jq -nc --argjson raw "$(stat -c %s "$cache/$name.as-lcc.json")" --argjson gz "$(stat -c %s "$cache/$name.as-lcc.json.gz")" --slurpfile s "$out/enrich-$name.json" '{raw:$raw, gz:$gz, stats:$s[0]}')"
    score "$name" "$cache/$name.as-lcc.json"
  fi
done

# 5. bundle each lcc-full variant into the binary and time it
cp "$root/data/schemes/lcc/lcc-lcsh.json.gz" "$cache/lcc-lcsh.committed.json.gz"
probe() { # probe NAME BIN
  local name=$1 bin=$2 s=$cache/probe-$name
  rm -rf "$s"
  local size cold first warm
  size=$(stat -c %s "$bin")
  cold=$(timed "probe-$name-nostore" "$bin" --store "$s.absent" --json classify --title "Programming in R" --text "Statistical computing with the R language")
  mkdir -p "$s"
  first=$(timed "probe-$name-first" "$bin" --store "$s" --json classify --title "Programming in R" --text "Statistical computing with the R language")
  warm=$(timed "probe-$name-warm" "$bin" --store "$s" --json classify --title "Organic chemistry" --text "Heterocyclic compounds and their synthesis")
  note "probe-$name" ok "$(jq -nc --argjson size "$size" --argjson cold "$cold" --argjson first "$first" --argjson warm "$warm" '{binary_bytes:$size, no_cache:$cold, first_with_cache:$first, warm:$warm}')"
}
probe baseline "$cache/shelfmark-baseline"
for name in full-direct full-inherit3 outline-inherit3; do
  [ -f "$cache/$name.as-lcc.json.gz" ] || continue
  cp "$cache/$name.as-lcc.json.gz" "$root/data/schemes/lcc/lcc-lcsh.json.gz"
  if (cd "$root" && cargo build --release -p shelfmark-cli) > "$out/logs/cli-build-$name.log" 2>&1; then
    cp "$CARGO_TARGET_DIR/release/shelfmark" "$cache/shelfmark-$name"
    probe "$name" "$cache/shelfmark-$name"
  else
    note "probe-$name" "build failed"
  fi
done
cp "$cache/lcc-lcsh.committed.json.gz" "$root/data/schemes/lcc/lcc-lcsh.json.gz"
mkdir -p "$out/variants" && cp "$cache"/*.as-lcc.json.gz "$out/variants/" 2> /dev/null
jq -s . "$report" > "$out/report.json"
echo "reproduce-lcc-lcsh: report in $out/report.json" >&2
