# Scheme benchmark results, 2026-10-10-r3

Every scheme is scored as a hierarchy over a fixed similarity graph: Dasgupta cost and Moseley-Wang revenue (both normalized to [0, 1]; lower cost and higher revenue are better), skill relative to the average-linkage baseline built on the same graph (`(1 - cost_norm) / (1 - ref_cost_norm)`; 1.0 matches the baseline), placement coverage, and structure and navigation statistics. Items without a placement attach to the root, so low coverage pushes cost toward 1.

- Corpus: 10,000 arXiv papers, `corpus.jsonl` sha256 `1027bf9d34928f470ed8b0d7b5ddc5c983fde49bc0cd33eaa162bbd1f830fea5`.
- Holdout: 500 Open Library editions, `holdout.jsonl` sha256 `3e057eb4a257488ca1668a1ed370306e74b5b21a79cf7b4fdbd2f5b21873c833`.
- Model: `sentence-transformers/all-MiniLM-L6-v2` (fastembed 0.9.0 / ONNX Runtime CPU), weights `Qdrant/all-MiniLM-L6-v2-onnx` revision `d13954661f83248295ba75c1ed411eef3b7b936e`, `model.onnx` sha256 `bbd7b466f6d58e646fdc2bd5fd67b2f5e93c0b687011bd4548c420f7bd46f0c5`.
- Graph: k = 20 mean-offset kNN cosine. Average-linkage baseline cost_norm: corpus 0.1518, holdout 0.3014.
- Harness: `shelfmark-bench` at commit `092494850adc3755f09e225bfd10d76870f10df9`.

## Corpus fixes since 2026-10-10

This run rescores the gold placements for `acm-ccs` and `msc` after two fixes
to `corpus.jsonl`. Neither fix touched the row order, `id`, `title` or
`abstract_text`, which are the only inputs to the similarity graph, so the
2026-10-10 graphs and average-linkage baselines still apply.

1. **ACM CCS** (`7685685d…` → `80c4adb1…`). The `acm_ccs` field now comes from
   arXiv's `arXivOld` records: 690 ACM CCS 1998 codes on 317 papers. Before, it
   held 433 values on 132 papers copied from the `arXivRaw` `acm-class`
   element, which repeats `msc-class`.
2. **MSC** (`80c4adb1…` → `1027bf9d…`). The `msc` field keeps only tokens
   shaped like MSC 2020 codes (`05C10`, `05Cxx`, `05-XX`, `05-01`). Before, it
   also held markers and words from the `msc-class` text, such as `(Primary)`
   and `Secondary:`. It went from 4,885 values on 1,348 papers to 4,298 values on
   1,325 papers. 316 rows changed, and no other byte of the file changed.

The gold MSC normalizer (`prepare_placements.py normalize-msc`) now accepts
every code shape in the scheme's grammar (`05C10`, `05Cxx`, `05-XX`, `05-01`).
Before, it accepted only five-character codes such as `05C10`. That change
alone causes the MSC difference below. With the old normalizer, the gold rows
from the fixed corpus are the same as before, and they reproduce the
2026-10-10 `msc-gold.json` exactly: same tree hash `6310d784…` and the same
scores. The new normalizer places 8 more papers, 1,322 in total, whose only
valid codes are section or form codes. It also changes the first label, which
is the one the harness scores, for 12 papers whose first source code is such a
code, for example `65-02` or `37Axx`.

ACM CCS gold is still skipped, for a different reason than in 2026-10-10. The
labels are now real ACM classes, but they are CCS 1998 codes such as `I.2.6`.
The scheme is CCS 2012, and its codes are paths of 8-digit concept IDs. No
CCS 1998 to 2012 crosswalk is used.

Only gold placements were scored: the source labels, normalized to scheme codes. Items without a label attach to the root.

Column notes: `classes used` is the effective number of classes (exp of placement entropy); `nav log a=1` is the expected menu cost `sum(1 + log2 fanout)` from root to item; `nav scan` adds a linear menu scan and a scan of the final class shelf.

## Gold placements

arXiv corpus:

| scheme | version | coverage | cost_norm | revenue_norm | skill vs avg-link | classes used | mean depth | max depth | max branching | gini | nav log a=1 | nav scan |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| msc | 2020 | 0.1322 | 0.9843 | 0.0157 | 0.0185 | 461.5009 | 1.3944 | 4 | 25 | 0.5986 | 15.2124 | 4371.9113 |

## Before and after

Against the gold results in `bench/results/2026-10-10/results.json` (corpus.jsonl sha256 `7685685d0de25d6e8a614859783dcb5376d81e4fe336a65f22b550f4c7527b8d`), on the same graphs and average-linkage baselines with the same scheme files.

| scheme | corpus | coverage | cost_norm | skill |
| --- | ---: | ---: | ---: | ---: |
| acm-ccs | corpus | skipped → skipped | skipped → skipped | skipped → skipped |
| msc | corpus | 0.1314 → 0.1322 | 0.9844 → 0.9843 | 0.0183 → 0.0185 |

## Skipped

- `acm-ccs-gold`: no acm_ccs label in corpus is a code of the scheme (10000 rows checked); the corpus acm_ccs field (690 values on 317 papers) holds ACM CCS 1998 codes such as `I.2.6`, as submitted to arXiv; the scheme is CCS 2012, whose codes are paths of 8-digit concept IDs, and no CCS 1998 to 2012 crosswalk is used

## Runtime

- schemes (build_schemes.sh acm-ccs msc): not timed; fetch-and-build of two schemes
- gold normalization and scoring (run_benchmark.py --gold-only): 1.4 s
