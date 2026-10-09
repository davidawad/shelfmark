# Pre-publication audit

Scope: full history of the repository (38 commits, 662 tracked files) scanned
with gitleaks 8.30.1 and a regex sweep of every revision for absolute paths,
personal names, employer or client names, e-mail addresses and tokens. No
secrets, no restricted data, no model weights, no binaries over 1.8 MB.

All findings were low severity and are resolved on the cleanup branch:

| Finding | Resolution |
|---|---|
| Internal task identifiers and references to private planning documents in research READMEs, bench script and `crates/profile` | Reworded to "spike N" or dropped |
| References to a private knowledge-base path and to the author's machines | Reworded generically ("a dedicated benchmark machine") |
| Personal-name test strings in CLI and core tests | Replaced with "alice" / "the user" |
| Paths assuming the repo was a subdirectory (`src/shelfmark/...`) in READMEs, docs, `registry.json`, generated-report templates, research path deps | Rewritten to resolve from the repository root |
| `Cargo.toml` repository URL | Set to https://gitlab.com/davidawad/shelfmark |
| CPC title fixture | Synthetic labels only; README states they are invented |
| Commit messages carry internal task ids | Left in history; harmless |

Licensing notes: the 30 Wikipedia extracts in `research/eval/gold` are
CC BY-SA 4.0 (recorded per item); arXiv metadata and Open Library records are
CC0; LC and Census data are US government works; IPC is CC BY 4.0; FAST sample
is ODC-By (attribution in its `source` block); the arXiv taxonomy is MIT
(LICENSE shipped). UDC, JEL, ACM CCS, MSC and CPC data are fetch-only and not
committed.
