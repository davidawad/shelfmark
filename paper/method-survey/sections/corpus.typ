#import "../template.typ": *
#import "../generated.typ": v

= Corpus <sec:corpus>

*arXiv corpus.* The arXiv OAI-PMH endpoint (`arXivRaw`) was harvested for
records with datestamps from 2026-01-01 to 2026-10-09 in eight top-level
groups. A seeded SHA-256 ranking then sampled #v.c_n_corpus records,
proportional to group size with a floor of 100 per group: #v.c_groups. arXiv
applies CC0 1.0 to metadata @arxivlicense. Datestamps follow record updates,
so older papers appear: #v.c_share_2026_ids of identifiers carry a 2026
prefix. Computer science, mathematics and physics make up
#v.c_share_physical_math_cs of the sample. Gold labels in other schemes are
sparse: #v.c_n_msc_rows papers carry an MSC 2020-shaped code, #v.c_n_acm_rows
an ACM CCS 1998 code and #v.c_n_jel_rows a JEL code (after the correction of
the label fields described in @sec:results, which leaves the text, order and
identifiers unchanged).

*Open Library holdout.* #v.c_n_holdout Open Library edition records with a
non-empty `lc_classifications` array were drawn through the Open Library
Search and editions APIs @openlibrarylicensing. They hold titles only, with no
description: #v.c_holdout_with_text rows have text beyond the title, and
titles average #v.c_holdout_title_words words. The holdout is skewed:
#v.c_holdout_p records (#v.c_holdout_p_share) have a first call number in class
P (language and literature). The other #v.c_holdout_letter_count letters seen
are #v.c_holdout_other_top. #v.c_holdout_empty records hold only an empty
string in the array. Open Library asks contributors to share under CC0 1.0
@openlibraryfaq. Both files are pinned by SHA-256 and by ID lists.

*Book sample with Dewey numbers.* A second book file holds #v.x_books_n
editions that carry an LC call number, a Dewey number and subjects, with equal
quotas over the 21 LCC main classes (@sec:books). It shares no work with the
500-book holdout.

*Schemes.* @tab:survey lists the 13 builds with their class counts and
licence status. Five are general or universal in intent: LCC (outline and
complete schedules), FDC, BK and ILC2. The rest are thesauri or vocabularies
(UNESCO Thesaurus, EuroVoc, OpenAlex Topics) or domain schemes (arXiv, MSC,
JEL, ACM CCS, IPC). Each scheme file's SHA-256 is recorded, and the rerun
under the corrected classifier reused byte-identical files.
