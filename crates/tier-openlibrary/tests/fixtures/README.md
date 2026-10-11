# Fixtures

Search pages served by `FixtureTransport`; no test opens a socket.

- `hobbit.json`, `sicp.json`, `kant.json` (`limit=3`) and
  `odyssey.json` (`limit=2`): recorded from
  `https://openlibrary.org/search.json` on 2026-10-10 with the tier's
  title, author and field parameters. Open Library data is CC0
  (<https://openlibrary.org/developers/licensing>).
- `synthetic.json`: hand-written records exercising author choice, raw
  `lc_classifications`, an original-language work title matched through
  its edition, Dewey-only and junk call numbers.
