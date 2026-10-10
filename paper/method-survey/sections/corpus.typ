#import "../template.typ": *
#import "../generated.typ": v

= Corpus <sec:corpus>

*arXiv corpus.* The arXiv OAI-PMH endpoint (`arXivRaw`) was harvested for
records with datestamps from 2026-01-01 to 2026-10-09 in eight top-level
groups. A seeded SHA-256 ranking then sampled 10,000 records, proportional to
group size with a floor of 100 per group: cs 3,968 (39.7%), physics 2,630
(26.3%), math 2,042 (20.4%), eess 519 (5.2%), stat 345 (3.5%), econ 185
(1.8%), q-bio 172 (1.7%) and q-fin 139 (1.4%). arXiv applies CC0 1.0
to metadata @arxivlicense. Datestamps follow record updates, so older papers
appear: 69.7% of identifiers carry a 2026 prefix. Computer science,
mathematics and physics make up 86.4% of the sample. Gold labels in other
schemes are sparse. In the corpus file used for the original runs, 1,348
papers carry an MSC code, 132 an ACM class field and 2 a JEL code. A later
correction of the label fields (@sec:results) leaves the text, order and
identifiers unchanged; it gives 1,325 papers with MSC 2020-shaped codes and
317 papers with 690 ACM CCS 1998 codes.

*Open Library holdout.* 500 Open Library edition records with a non-empty
`lc_classifications` array were drawn through the Open Library Search and
editions APIs @openlibrarylicensing. They hold titles only, with no
description: no row has text beyond the title, and titles average 3.5
words. The holdout is skewed: 382 records (76.4%) have a first call number in
class P (language and literature). The other 13 letters seen are B 32, H 19,
Q 8, D 5, M 4, R 3, U 2, N 2, C 2, J 1, G 1, E 1, L 1. 37 records hold only
an empty string in the array. Open Library asks contributors to share under
CC0 1.0 @openlibraryfaq. Both files are pinned by SHA-256 and by ID lists.

*Schemes.* @tab:survey lists the 13 builds with their class counts and
licence status. Five are general or universal in intent: LCC (outline and
complete schedules), FDC, BK and ILC2. The rest are thesauri or vocabularies
(UNESCO Thesaurus, EuroVoc, OpenAlex Topics) or domain schemes (arXiv, MSC,
JEL, ACM CCS, IPC). Each scheme file's SHA-256 is recorded, and the rerun
under the corrected classifier reused byte-identical files.
