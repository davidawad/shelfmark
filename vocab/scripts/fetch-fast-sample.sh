#!/bin/sh
# Fetch a small sample of FAST authority records as per-record RDF/XML.
#
# The FAST bulk downloads (researchworks.oclc.org) sit behind a Cloudflare
# browser challenge that non-interactive clients cannot pass, so this pulls
# individual records from id.worldcat.org instead: for each search term, the
# top authorised headings from the FAST suggest API, then <id>.rdf.xml.
#
# usage: fetch-fast-sample.sh OUT_DIR [TERM...]
# FAST is ODC-By 1.0 (OCLC); keep attribution with any copy of the records.
set -eu
out=$1
shift
mkdir -p "$out"
[ $# -gt 0 ] || set -- compilers "programming languages" mathematics "organic chemistry" \
    "stochastic processes" botany "jazz" "philosophy" "economics" "agriculture" \
    "music theory" "painting" "law" "medicine" "naval art and science" "education" \
    "religion" "geography" "library science" "photography"
for term in "$@"; do
    q=$(printf %s "$term" | sed 's/ /%20/g')
    curl -sS -m 30 "https://fast.oclc.org/searchfast/fastsuggest?query=$q&queryIndex=suggestall&queryReturn=idroot,auth,type&suggest=autoSubject&rows=20" |
        grep -o '"idroot":\["fst[0-9]*"\],"type":"auth"' | grep -o 'fst[0-9]*' | head -2
done | sort -u | while read -r fst; do
    n=$(printf %s "$fst" | sed 's/^fst0*//')
    [ -s "$out/$fst.rdf.xml" ] && continue
    curl -sS -m 30 -o "$out/$fst.rdf.xml" "https://id.worldcat.org/fast/$n.rdf.xml"
    sleep 0.3
done
ls "$out" | wc -l
