#!/usr/bin/env bash
# Build or copy every benchmark scheme into $OUT (default bench/corpus/cache/schemes).
# Fetch-and-build schemes are downloaded from upstream and checked against
# their committed source locks; outputs stay in the ignored cache and are
# never committed. One line per scheme is appended to $OUT/status.tsv:
# name<TAB>ok|skipped<TAB>detail. Scheme names given as arguments limit the
# build to those schemes (default: all).
set -uo pipefail
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
OUT=${OUT:-$ROOT/bench/corpus/cache/schemes}
SRC=${SRC:-$ROOT/bench/corpus/cache/sources}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/bench/corpus/cache/target-schemes}
mkdir -p "$OUT" "$SRC" "$OUT/logs"
: > "$OUT/status.tsv"

ONLY=" $* "
want() { [ "$ONLY" = "  " ] || [[ $ONLY == *" $1 "* ]]; }

record() { printf '%s\t%s\t%s\n' "$1" "$2" "$3" >> "$OUT/status.tsv"; }

# run NAME CMD...: run a builder, log it, and record the outcome.
run() {
    local name=$1; shift
    want "$name" || return 0
    if "$@" > "$OUT/logs/$name.log" 2>&1 && [ -s "$OUT/$name.json" ]; then
        record "$name" ok "$(sha256sum "$OUT/$name.json" | cut -d' ' -f1)"
    else
        record "$name" skipped "build failed: $(grep -v '^\s*\(Compiling\|Finished\|Running\|Downloaded\|Downloading\|Updating\|Locking\)' "$OUT/logs/$name.log" | tail -1)"
    fi
}

copy() {
    local name=$1 file=$2
    want "$name" || return 0
    cp "$ROOT/$file" "$OUT/$name.json" && record "$name" ok "$(sha256sum "$OUT/$name.json" | cut -d' ' -f1)"
}

# Committed (bundled) scheme files.
copy lcc data/schemes/lcc/lcc.json
copy openalex-topics schemes/openalex-topics/openalex-topics.json
copy fdc schemes/fdc/fdc.json
copy unesco-thesaurus schemes/unesco-thesaurus/unesco-thesaurus.json
copy bk schemes/bk/bk.json
copy arxiv schemes/arxiv/arxiv.json
want lcc-full && gzip -dc "$ROOT/schemes/lcc-full/lcc-full.json.gz" > "$OUT/lcc-full.json" \
    && record lcc-full ok "$(sha256sum "$OUT/lcc-full.json" | cut -d' ' -f1)"

# IPC: bundle-eligible but too large to commit; fetch into a scratch data
# directory so the committed lock is never rewritten, then build from it.
ipc() {
    mkdir -p "$SRC/ipc-data"
    cp "$ROOT/schemes/ipc/sources.lock.json" "$SRC/ipc-data/sources.lock.json"
    local m=$ROOT/schemes/ipc/builder/Cargo.toml
    # The refetched lock may differ only in its "fetched" dates.
    cargo run -q --release --manifest-path "$m" -- fetch --cache "$SRC/ipc" --data "$SRC/ipc-data" &&
        diff <(grep -v '"fetched"' "$SRC/ipc-data/sources.lock.json") \
            <(grep -v '"fetched"' "$ROOT/schemes/ipc/sources.lock.json") &&
        cp "$ROOT/schemes/ipc/sources.lock.json" "$SRC/ipc-data/sources.lock.json" &&
        cargo run -q --release --manifest-path "$m" -- build --cache "$SRC/ipc" --data "$SRC/ipc-data" &&
        cp "$SRC/ipc-data/ipc.json" "$OUT/ipc.json"
}
run ipc ipc

# Fetch-and-build schemes (not redistributable; local use only).
run acm-ccs cargo run -q --release --manifest-path "$ROOT/schemes/acm-ccs/Cargo.toml" -- \
    build --cache "$SRC/acm-ccs" --out "$OUT/acm-ccs.json"
run eurovoc cargo run -q --release --manifest-path "$ROOT/schemes/eurovoc/Cargo.toml" -- \
    build --cache "$SRC/eurovoc" --out "$OUT/eurovoc.json"
run ilc2 cargo run -q --release --manifest-path "$ROOT/schemes/ilc2/Cargo.toml" -- \
    build --cache "$SRC/ilc2" --fetch --out "$OUT/ilc2.json"
run msc cargo run -q --release --manifest-path "$ROOT/schemes/msc/tool/Cargo.toml" -- \
    build --cache "$SRC/msc" --out "$OUT/msc.json"
run jel cargo run -q --release --manifest-path "$ROOT/schemes/jel/Cargo.toml" -- \
    build --cache "$SRC/jel" --out "$OUT/jel.json"
cat "$OUT/status.tsv"
