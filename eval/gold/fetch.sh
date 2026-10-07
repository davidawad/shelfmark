#!/usr/bin/env bash
# Download the raw source snapshots named in manifest.json into raw/.
# Existing snapshots are kept unless --refresh is given, so a rebuild of the
# gold set (`mc-eval gold build`) stays offline and reproducible.
# Requires curl and jq. Usage: gold/fetch.sh [--refresh]
set -euo pipefail

here=$(cd -- "$(dirname -- "$0")" && pwd)
refresh=0
[[ ${1:-} == --refresh ]] && refresh=1
ua='mc-eval-goldset/0.1 (materials classifier eval harness)'
stamps="$here/raw/retrieved.tsv"
mkdir -p "$here/raw/arxiv" "$here/raw/openlibrary" "$here/raw/wikipedia"
touch "$stamps"

# fetch REL URL [DELAY]: download URL to raw/REL and record the UTC time.
fetch() {
	local rel=$1 url=$2 delay=${3:-1} dest="$here/$1" tmp
	if [[ -s $dest && $refresh -eq 0 ]]; then
		return 0
	fi
	tmp=$(mktemp)
	curl -sSfL --retry 4 --retry-delay 15 -m 60 -A "$ua" "$url" -o "$tmp"
	mv "$tmp" "$dest"
	grep -v -F -- "$rel"$'\t' "$stamps" >"$stamps.tmp" || true
	printf '%s\t%s\n' "$rel" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >>"$stamps.tmp"
	sort "$stamps.tmp" >"$stamps"
	rm -f "$stamps.tmp"
	sleep "$delay"
}

jq -r '.sources[] | "\(.kind)\t\(.record)"' "$here/manifest.json" |
	while IFS=$'\t' read -r kind record; do
		case $kind in
		arxiv)
			fetch "raw/arxiv/$record.xml" "https://export.arxiv.org/api/query?id_list=$record" 3
			;;
		openlibrary)
			fetch "raw/openlibrary/$record.edition.json" "https://openlibrary.org/books/$record.json"
			work=$(jq -r '.works[0].key' "$here/raw/openlibrary/$record.edition.json")
			fetch "raw/openlibrary/$record.work.json" "https://openlibrary.org$work.json"
			;;
		wikipedia)
			fetch "raw/wikipedia/$record.claims.json" \
				"https://www.wikidata.org/w/api.php?action=wbgetclaims&format=json&property=P1149&entity=$record" 2
			if [[ ! -s $here/raw/wikipedia/$record.summary.json || $refresh -eq 1 ]]; then
				title=$(curl -sSf --retry 4 --retry-delay 15 -m 60 -A "$ua" \
					"https://www.wikidata.org/w/api.php?action=wbgetentities&format=json&props=sitelinks&sitefilter=enwiki&ids=$record" |
					jq -r --arg q "$record" '.entities[$q].sitelinks.enwiki.title | gsub(" "; "_") | @uri')
				fetch "raw/wikipedia/$record.summary.json" "https://en.wikipedia.org/api/rest_v1/page/summary/$title"
			fi
			;;
		*)
			echo "unknown source kind: $kind" >&2
			exit 3
			;;
		esac
	done
echo "snapshots in $here/raw" >&2
