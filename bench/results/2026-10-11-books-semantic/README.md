# Semantic shortlist against librarians on the 3,000 books (2026-10-11)

Does the semantic shortlist tier place books the way librarians do?
`research/reports/9-kb-structure-proposal.md` §5.3 and §8 left this open.
The test is the 3,000-book DDC holdout (`bench/corpus/holdout-books.jsonl`).
Each book is placed into FDC, the LCC outline (`lcc.json`) and the bundled
LCC outline plus LCSH (`scheme:lcc@2024`), once with `profile:lexical@2` and
once with `profile:shortlist@2`. The placement is then compared with the
librarian's number: `fdc-ddc` for FDC and `lcc-gold` for both LCC runs, from
`../2026-10-10-ddc/placements/books/`.

`table.md` and `results.json` hold every number. Wilson intervals are 95%.

## Result

- The shortlist places every book (coverage 1.000, up from 0.839, 0.915 and
  0.985), but main-class agreement does not rise. For FDC it is 0.514
  [0.496, 0.532] on 3,000 books, against lexical's 0.544 [0.525, 0.564] on
  2,518. For LCC + LCSH it is 0.558 [0.540, 0.575], against 0.553. For the LCC
  outline it is 0.438, against 0.372. At depth 3 every scheme gets worse
  (FDC 0.179 against 0.210; LCC + LCSH 0.287 against 0.357).
- `profile:shortlist@2` is a cascade: the vocabulary tier still decides
  first (`deciding_tiers` in each `.stats.json`). For FDC the shortlist
  decides only the 485 books vocab misses, and it keeps 2,516 of lexical's
  2,518 placements. On the books lexical leaves unplaced, the shortlist
  agrees with the librarian on the main class for 0.349 [0.307, 0.392] (FDC,
  482 books), 0.414 (LCC outline, 256) and 0.295 (LCC + LCSH, 44).
- For the LCC schemes the shortlist also re-ranks low-confidence vocab
  matches: 1,053 of 2,744 (outline) and 938 of 2,956 (LCC + LCSH) lexical
  placements change class.

Report 9 §8 sets the bar for re-opening classifier-only placement: at least
0.85 main-class agreement at coverage of at least 0.95. The shortlist reaches
the coverage but stays near 0.5 agreement. It does not meet the bar, and the
recommendation and the default-scheme record stand. The `rerank` tier is
still a stub (see `../2026-10-10-semantic/`), so it was not run.

## Reproduce

On a box (this run: 16 GiB limit, 16 CPUs; about 1 minute to build and
6 minutes to place):

```sh
cargo build --release -p shelfmark-cli --features semantic-shortlist
BIN=${CARGO_TARGET_DIR:-target}/release
python3 crates/bench/scripts/books_semantic.py --shelfmark $BIN/shelfmark \
  --profile profile:lexical@2 --profile profile:shortlist@2 \
  --out bench/results/2026-10-11-books-semantic --check-lexical
```

`--check-lexical` requires the lexical placements to equal the committed
ones byte for byte (`../2026-10-10-ddc/placements/books/fdc-placed.jsonl`,
`lcc-placed.jsonl` and
`../2026-10-11-book-stability/placements/books/lcc-lcsh-placed.jsonl`). In
this run all three were equal. To recompute the tables from the committed
placements without shelfmark, add `--tables-only`. The shortlist placements
depend on the shelfmark build and the BGE-small-en-v1.5 model; with the same
build and model they reproduce exactly.
