#!/usr/bin/env bash
# Re-download the PDFs indexed in README.md. Idempotent: existing files are
# skipped. Run from anywhere: bash docs/research/fetch.sh [open|restricted]
# Throttle: 3 s between requests to the same host (arXiv policy: <= 1 req / 3 s).
# A download is kept only if it starts with %PDF; HTML error pages are refused.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
only="${1:-all}"
ua='shelfmark-docs-research/1.0 (personal reading; contact via repo)'
declare -A last
ok=0; skip=0; fail=0

fetch() { # dir key url
  local dir=$1 key=$2 url=$3 out="$here/pdf/$1/$2.pdf" host tmp now
  [ "$only" = all ] || [ "$only" = "$dir" ] || return 0
  if [ -s "$out" ]; then skip=$((skip+1)); return 0; fi
  host=${url#*://}; host=${host%%/*}
  now=$(date +%s); [ -n "${last[$host]:-}" ] && [ $((now-last[$host])) -lt 3 ] && sleep $((3-(now-last[$host])))
  mkdir -p "$here/pdf/$dir"
  tmp=$(mktemp)
  curl -sSL --max-time 120 -A "$ua" -o "$tmp" "$url"; last[$host]=$(date +%s)
  if [ "$(head -c 4 "$tmp")" = "%PDF" ]; then
    mv -f "$tmp" "$out"; ok=$((ok+1)); echo "ok    $dir/$key.pdf"
  else
    rm -f "$tmp"; fail=$((fail+1)); echo "FAIL  $dir/$key.pdf (not a PDF) $url" >&2
  fi
}

while IFS=$'\t' read -r dir key url; do
  [ -z "$dir" ] && continue
  fetch "$dir" "$key" "$url"
done <<'MANIFEST'
restricted	arxiv-0710.0845	https://arxiv.org/pdf/0710.0845
restricted	waltman2012	https://arxiv.org/pdf/1203.0532
restricted	dasgupta2016	https://arxiv.org/pdf/1510.05043
restricted	charikar2016	https://arxiv.org/pdf/1609.09548
restricted	roy2016	https://arxiv.org/pdf/1610.09269
restricted	cohenaddad2017	https://arxiv.org/pdf/1704.02147
restricted	chatziafratis2018	https://arxiv.org/pdf/1805.09476
restricted	arxiv-1805.12216	https://arxiv.org/pdf/1805.12216
restricted	arxiv-1811.01727	https://arxiv.org/pdf/1811.01727
restricted	zhang2018taxogen	https://arxiv.org/pdf/1812.09551
restricted	arxiv-1904.08249	https://arxiv.org/pdf/1904.08249
restricted	arxiv-1905.02331	https://arxiv.org/pdf/1905.02331
open	reimers2019	https://arxiv.org/pdf/1908.10084
restricted	shen2020taxoexpan	https://arxiv.org/pdf/2001.09522
restricted	wang2020minilm	https://arxiv.org/pdf/2002.10957
restricted	chami2020	https://arxiv.org/pdf/2010.00402
restricted	grootendorst2022	https://arxiv.org/pdf/2203.05794
open	priem2022	https://arxiv.org/pdf/2205.01833
restricted	zeng2024chainoflayer	https://arxiv.org/pdf/2402.07386
restricted	unterkalmsteiner2023	https://arxiv.org/pdf/2403.00111
restricted	wan2024tntllm	https://arxiv.org/pdf/2403.12173
open	arxiv-2502.15745	https://arxiv.org/pdf/2502.15745
open	semeval2025task5	https://arxiv.org/pdf/2504.07199
open	annifsemeval2025	https://arxiv.org/pdf/2504.19675
open	arxiv-2504.21589	https://arxiv.org/pdf/2504.21589
open	wullschleger2025	https://arxiv.org/pdf/2505.11470
open	kargupta2025taxoadapt	https://arxiv.org/pdf/2506.10737
open	arxiv-2509.19125	https://arxiv.org/pdf/2509.19125
restricted	watts2002	https://arxiv.org/pdf/cond-mat/0205383
restricted	arxiv-physics-0004057	https://arxiv.org/pdf/physics/0004057
open	traag2019	https://www.nature.com/articles/s41598-019-41695-z.pdf
open	moseley2023	https://jmlr.org/papers/volume24/18-080/18-080.pdf
restricted	moseley2017	https://proceedings.neurips.cc/paper_files/paper/2017/file/d8d31bd778da8bdd536187c36e48892b-Paper.pdf
restricted	kleinberg2002	https://proceedings.neurips.cc/paper_files/paper/2002/file/43e4e6a6f341e00671e123714de019a8-Paper.pdf
restricted	carlsson2010	https://jmlr.org/papers/volume11/carlsson10a/carlsson10a.pdf
open	nasko2025	https://journals.lib.washington.edu/index.php/nasko/article/download/16375/14081
open	llm4ddc2025	https://madoc.bib.uni-mannheim.de/71151/1/heiBOOKS-1652-978-3-911056-51-9-CH40.pdf
restricted	lcecd2025	https://repository.ifla.org/bitstreams/ca2b5c84-46b1-4f2e-b087-2e5f5a86e628/download
restricted	rpt-hearst1992	https://aclanthology.org/C92-2082.pdf
restricted	rpt-snow2006	https://aclanthology.org/P06-1101.pdf
restricted	rpt-icmbl2024-annif	https://www.atlantis-press.com/article/126011042.pdf
restricted	rpt-jgu-annif-chapter	https://pure.jgu.edu.in/id/eprint/9881/1/9_Chapter-Meghna_Biswas.pdf
restricted	rpt-uh-course-mapping	https://scholarspace.manoa.hawaii.edu/bitstreams/cd4a8726-bb94-4674-b49e-96c4c77a6ad3/download
restricted	rpt-upriss-iccs2000	https://www.upriss.org.uk/papers/iccs00.pdf
restricted	rpt-upriss-isko2000	https://www.upriss.org.uk/papers/isko00.pdf
open	rpt-jlis-437	https://jlis.it/index.php/jlis/article/download/437/430/445
restricted	rpt-oclc-ddc-summaries	https://www.oclc.org/content/dam/oclc/dewey/resources/summaries/deweysummaries.pdf
restricted	rpt-imls-lg-256666	https://imls.gov/sites/default/files/project-proposals/lg-256666-ols-24-preliminary-proposal.pdf
restricted	rpt-acm-ccs-howto	https://www.acm.org/binaries/content/assets/publications/article-templates/ccs-howto-v6-12jan2015.pdf
MANIFEST

echo "downloaded=$ok skipped=$skip failed=$fail"
[ "$fail" -eq 0 ]
