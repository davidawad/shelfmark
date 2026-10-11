#!/usr/bin/env bash
# Rebuild the bundled scheme data/schemes/lcc/lcc-lcsh.json.gz from public
# Library of Congress sources, or check that the committed file still is
# what they build.
#
#   research/vocab/scripts/build-lcc-lcsh.sh CACHE [build|check]
#
# CACHE holds the LCSH bulk dump (subjects.madsrdf.nt.gz, 230 MB, never
# committed); it is downloaded when absent, the same file tools/lcc-outline
# caches, so `lcc-outline fetch --cache CACHE` fills it too. The dump must
# match the sha256 that data/schemes/lcc/sources.lock.json records; when LC
# has issued a newer one, relock with `lcc-outline fetch` first.
#
# Inputs: that dump and data/schemes/lcc/lcc.json (built by tools/lcc-outline).
# build: writes lcc-lcsh.json.gz and research/vocab/stats/{lcsh,enrich}-stats.json.
# check: rebuilds into CACHE and fails unless the decompressed bytes equal
#        the committed file's.
# Needs cargo, jq, gzip and sha256sum (or shasum).
set -euo pipefail
cache=${1:?usage: build-lcc-lcsh.sh CACHE [build|check]}
mode=${2:-build}
root=$(cd "$(dirname "$0")/../../.." && pwd)
lock=$root/data/schemes/lcc/sources.lock.json
bundled=$root/data/schemes/lcc/lcc-lcsh.json.gz
mkdir -p "$cache"

sha256() { if command -v sha256sum > /dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d' ' -f1; }

entry=$(jq -c '.sources[] | select(.file == "subjects.madsrdf.nt.gz")' "$lock")
url=$(jq -r .url <<< "$entry")
want=$(jq -r .sha256 <<< "$entry")
dump=$cache/subjects.madsrdf.nt.gz
if [ ! -f "$dump" ]; then
  curl -fsSL --retry 3 -o "$dump.part" "$url"
  mv "$dump.part" "$dump"
fi
got=$(sha256 "$dump")
if [ "$got" != "$want" ]; then
  echo "build-lcc-lcsh: $dump has sha256 $got, the lock wants $want (LC issued a new dump? relock with lcc-outline fetch)" >&2
  exit 1
fi

export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$root/research/vocab/target}
cargo build --release --manifest-path "$root/research/vocab/Cargo.toml"
bin=$CARGO_TARGET_DIR/release/vocab-ingest

if [ "$mode" = build ]; then stats=$root/research/vocab/stats; else
  stats=$cache/stats
  mkdir -p "$stats"
fi
"$bin" lcsh --input "$dump" --out "$cache/lcsh.tsv.gz" --stats "$stats/lcsh-stats.json"

# Title and description are part of the content hash every placement
# records: change them only together with a scheme content change.
title="Library of Congress Classification outline with LCSH vocabulary"
description='Main classes and subclasses from the LC Classification Outline; numbered spans and captions from the 2024 FreeLCC outline PDFs (src/shelfmark/tools/lcc-outline, data/schemes/lcc/SOURCES.md). Vocabulary: every active id.loc.gov LCSH heading that carries an LCC number, as a link on the class that holds that number (the class itself or the deepest span covering it); see data/schemes/lcc/LCSH.md. A span'"'"'s code is `<start>-<end>` with the end'"'"'s letters dropped when they repeat the start'"'"'s.'
"$bin" enrich-scheme --scheme "$root/data/schemes/lcc/lcc.json" --table "$cache/lcsh.tsv.gz" \
  --title "$title" --description "$description" --canonical true \
  --out "$cache/lcc-lcsh.json" --stats "$stats/enrich-stats.json"

case "$mode" in
  build)
    gzip -9n -c "$cache/lcc-lcsh.json" > "$bundled"
    echo "build-lcc-lcsh: wrote $bundled ($(wc -c < "$bundled") bytes)" >&2
    ;;
  check)
    if gzip -dc "$bundled" | cmp -s - "$cache/lcc-lcsh.json"; then
      echo "build-lcc-lcsh: $bundled is what the locked sources build" >&2
    else
      echo "build-lcc-lcsh: $bundled differs from the rebuild in $cache/lcc-lcsh.json" >&2
      exit 1
    fi
    ;;
  *)
    echo "build-lcc-lcsh: unknown mode $mode" >&2
    exit 2
    ;;
esac
