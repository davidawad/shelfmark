# Reproducing the 2026-10-10-ddc results

This run compares gold Dewey Decimal Classification numbers with gold LCC,
FDC and the shelfmark-placed general schemes on two book holdouts:

- `holdout`: the 500 Open Library editions of Paper 1
  (`bench/corpus/holdout.jsonl`, titles only), on the published 2026-10-10
  graph, with Dewey numbers re-read from the same edition records
  (`bench/corpus/holdout-ddc.jsonl`);
- `holdout-ddc`: the editions of `holdout` that carry a Dewey number, on the
  graph induced on them;
- `books`: 3,000 new editions with both an LC call number and a Dewey
  number, plus a description or subjects (`bench/corpus/holdout-books.jsonl`),
  on a new graph built with the same model and k.

No text of the Dewey schedules is used anywhere; see
`research/reports/7-ddc-position.md`.

All commands run from the repository root on Linux. Intermediate files go
under `bench/corpus/cache/`, which git ignores. Network access is needed for
the Open Library harvest, the `ilc2` scheme build and the embedding model.

## 1. Tools

```sh
export PATH=/usr/local/cargo/bin:$PATH
cargo build --release -p shelfmark-cli -p shelfmark-bench
cargo build --release --manifest-path schemes/learned/Cargo.toml
BIN=${CARGO_TARGET_DIR:-target}/release    # learned-scheme may be in schemes/learned/target/release
uv venv bench/corpus/cache/venv
VIRTUAL_ENV=bench/corpus/cache/venv uv pip install 'fastembed==0.9.0'
```

The published run used fastembed 0.9.0, numpy 2.5.3 and onnxruntime 1.31.0.

## 2. Open Library harvest

```sh
python3 bench/corpus/builder/harvest_openlibrary_ddc.py all
```

`holdout` re-reads the 500 edition records (`/books/<id>.json`) and writes
`holdout-ddc.jsonl`. `books` samples the new holdout. The LCC main classes
`ABCDEFGHJKLMNPQRSTUVZ` each get an equal quota (143 for the first 18,
142 for the last 3; 3,000 in all). For each class the Search API query
`lcc:<class>* AND ddc:*` (`sort=lcc_sort`) is read at 20 evenly spaced
offsets, 30 works each. The back 10 pages are read from `sort=lcc_sort desc`
at the mirrored offset, because the API answers HTTP 500 to very deep
offsets (class H has about 660,000 works). The candidate works are ranked by SHA-256 of
`openlibrary-ddc-holdout-v1-2026|<class>|<work>`. In that order, each work's
first editions page is read. An edition qualifies when its first LC call
number is in the class (`MLCS` shelf numbers excluded) and a Dewey value
holds a number; the one with the lowest seeded hash is taken. Then the work
record supplies the description and subjects, one of which must be
non-empty. The works of the 500-book holdout are skipped. The text the graph
and classifier read (`abstract_text`) is the description (edition, else
work; at most 4,000 characters), then `Subjects: ` and up to 30 subjects
(edition first, then work).

Requests go one at a time, at least 1.05 s apart, and every response is
cached under `bench/corpus/cache/openlibrary-ddc/http/`, so a rerun replays
from the cache. Open Library records change, so a fresh harvest can differ.
The pinned files are the reference: `holdout-books-ids.txt`,
`holdout-books.jsonl` and `holdout-ddc.jsonl`, with the SHA-256 values in
`bench/corpus/README.md` and `results.json`. Per-class counts are in
`bench/corpus/holdout-books.sampling.json`.

## 3. Schemes

```sh
crates/bench/scripts/build_schemes.sh lcc lcc-full fdc bk ilc2 unesco-thesaurus
```

All six files must hash as in `../2026-10-10/results.json`; in this run they
did. `learned` is `schemes/learned/learned.json`. The Dewey tree is built
from the numbers by `crates/bench/scripts/ddc_numbers.py scheme`: main class,
division, section, then one level per decimal digit, with the notation as
the label. The published copy is `schemes/ddc.json`.

## 4. Placements, graphs and scoring

```sh
python3 crates/bench/scripts/ddc_benchmark.py --shelfmark $BIN/shelfmark \
  --harness $BIN/shelfmark-bench --learned $BIN/learned-scheme --resamples 1000
```

It works in `bench/corpus/cache/ddc-bench/`, skips every step whose output
exists, and, per corpus:

1. Graphs. `holdout` uses the published
   `../2026-10-10/graphs/holdout-k20.json.gz`. `books` is built by
   `build_graph_fastembed.py --k 20 --vectors` (all-MiniLM-L6-v2, pinned
   revision `d13954661f83248295ba75c1ed411eef3b7b936e`, exact mean-pair tau,
   max-symmetrized kNN). `holdout-ddc` is the `holdout` graph induced on the
   editions with a Dewey number (`learned-scheme subset`). Each graph's
   manifest is checked against the graph and corpus hashes.
2. Gold placements: `ddc-gold`, the first Dewey value that holds a number,
   at its own class, and `ddc-section`, the same number cut to its
   three-digit section. `lcc-gold` and `lcc-full-gold`, the deepest class
   covering the LC call number, resolved with `scheme-browse` exactly as in
   Paper 1, first label scored. `fdc-ddc`, the Dewey number mapped to the
   deepest FDC class whose number is a leading part of it or whose range
   contains it.
3. shelfmark placements into `lcc`, `lcc-full`, `fdc`, `bk`, `ilc2`,
   `unesco-thesaurus` and `learned`, with `profile:lexical@2`, one isolated
   store per scheme with empty rules, low-confidence placements kept and
   fallbacks unplaced, as in `../2026-10-10-r2/`.
4. `learned-own`: the corpus inserted into the learned scheme as new works
   (nearest leaf centroid). For `holdout` this uses the committed
   `schemes/learned/data/holdout-vectors.json.gz`; for `books` the vectors
   written with the graph, quantized by `learned-scheme quantize`.
5. `shelfmark-bench run` for every run, with `--reference-cost` set to the
   graph's average-linkage cost, then one `shelfmark-bench bootstrap` call
   per corpus over all runs: 1,000 paired draws, seed 20261010, 95%
   percentile intervals.

The ten `holdout` runs shared with the published 2026-10-10, 2026-10-10-r2
and 2026-10-10-learned results score exactly as published, with the same
graph and tree hashes. The learned scheme-file hash differs only because
its source note now pins the corrected corpus.

## 5. Publish

```sh
python3 crates/bench/scripts/publish_ddc.py --harness $BIN/shelfmark-bench \
  --shelfmark $BIN/shelfmark --learned $BIN/learned-scheme
```

This writes `results.json`, `runs/`, `bootstrap/`, `placements/`
(every placement file with its stats; all are under 20 MB, so none is
gzipped), `graphs/` (the gzipped `books` graph, the `holdout-ddc` ids, and
manifests) and `schemes/` (`ddc.json`, notation only, and the books-inserted
learned scheme `learned-books.json`). The tables of `results.md` are
written to `bench/corpus/cache/ddc-bench/tables.md`; the prose is by hand.

To rescore without rerunning the classifier, decompress the graphs and run
`shelfmark-bench run` or `bootstrap` on the committed placements with the
scheme files from step 3. The fetch-only `ilc2` file is not committed.
