# Pre-publication audit

The audit covered tracked source, documentation, and research artifacts. The
following findings were resolved in this cleanup:

| Finding | Resolution |
|---|---|
| Internal tracker references and private planning-document links | Removed from public research text, comments, and package descriptions. |
| References to private knowledge-base locations and specific developer machines | Reworded as general, reproducible workflow guidance. |
| Personal-name test strings | Replaced with `alice` or `the user`. |
| Paths that assumed a nested source checkout | Rewritten to repository-root paths, including the merged ILC2 registry commands. |
| Package repository metadata | Set to the public repository URL in the root manifest. |
| CPC title sample | Replaced with invented labels, marked as synthetic in the fixture, and accepted by the fixture parser and tests. |
| Research crate workspace boundaries | Kept as independent workspaces and excluded explicitly from the root workspace. |
| Wikipedia evaluation items | The evaluation README records CC BY-SA 4.0 and the attribution/share-alike requirement. |
| Local paths and personal references in research artifacts | Removed from the imported reports, benchmark notes, and this audit. |

Licensing notes: Wikipedia extracts in `research/eval/gold` are CC BY-SA 4.0;
arXiv metadata and Open Library records are CC0; LC and Census data are US
government works; IPC is CC BY 4.0; FAST sample data is ODC-By with attribution
in its source block; the arXiv taxonomy is MIT licensed. UDC, JEL, ACM CCS, MSC,
and CPC data are fetch-only and are not committed.
