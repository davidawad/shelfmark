# Open Library tier: 50-book measurement

`books-50.json` pins 50 well-known books across literature, science,
computing, economics, history and philosophy, each with its title, first
author and an expected LC class number taken from Open Library itself
(CC0, <https://openlibrary.org/developers/licensing>), retrieved
2026-10-10 by `build_list.py`:

- one free-text search per book, `search.json?q=<title> <author>&limit=1`;
- `expected_lc` is the most frequent class number among the top work's
  `lcc` call numbers whose letters are an LCC subclass (the outline in
  `data/schemes/lcc/lcc.json`), and `expected_share` its share of them.

The tier searches differently (fielded `title=` and `author=` with up to
10 results, its own matching and threshold), so the expected label does
not come from the tier's own query. It is still Open Library's own call
number, so this measures whether title+author finds the right work and
maps its number into the scheme. It does not check Open Library's
cataloging.

Run it (it sends at most 50 requests, one a second, and caches them in
the store; a re-run sends none):

```sh
cargo run -p shelfmark-cli --features openlibrary --example openlibrary_measure -- \
    crates/tier-openlibrary/eval/books-50.json /tmp/ol-store
```

Each book is classified from title and authors alone into the bundled LCC
outline with `profile:lexical` and `profile:openlibrary` (the result
below was measured at version 1, before `vocab-2` stopped matching
stopword-only labels). A hit is the class the outline gives the
expected number; a subclass hit only needs the same letters.

## Result (2026-10-10, version 1)

| Profile | Class hit | Subclass hit | Decided (machine) | Correct when decided |
|---|---|---|---|---|
| `lexical` | 0/50 (0%) | 3/50 (6%) | 0 | - |
| `openlibrary` | 45/50 (90%) | 45/50 (90%) | 43 | 43/43 |

The tier matched a work for 47 books. The five misses:

- *Thus Spoke Zarathustra*, *The Second Sex*, *Don Quixote*: Open Library
  files the canonical work under its original title (Also sprach
  Zarathustra, Le Deuxième Sexe, Don Quijote de la Mancha), and a
  fielded `title=` search does not reach it. The English-titled works it
  returns have no call numbers, so nothing matched and the local tiers
  guessed.
- *The Republic*: matched, but the call numbers split between the Greek
  text (PA) and political theory (JC); PA led, flagged at 0.73.
- *Democracy in America*: matched, placed in JC131-273 (political theory)
  against the expected JK216, flagged at 0.75.

Every miss was flagged rather than decided.
