# Dewey numbers without the Dewey schedules: the benchmark's licensing position

Written 2026-10-10 for `bench/results/2026-10-10-ddc/`. Every source below
was fetched on that date; quotes are verbatim. This is an engineering
position, not legal advice, and it is written for the United States and the
European Union only.

## What the benchmark uses, and what it does not

The DDC comparison scores the Dewey numbers that librarians gave to books
against the benchmark's similarity graph. It needs three things from Dewey:
each book's class number, the hierarchy between numbers, and nothing else.

- **Numbers come from Open Library edition records**, field
  `dewey_decimal_class`, the same records the LCC holdout came from. They are
  stored in `bench/corpus/holdout-ddc.jsonl` (the 500-book holdout) and
  `bench/corpus/holdout-books.jsonl` (the 3,000-book holdout) exactly as the
  records give them (`823/.912`, `016.3092 s`).
- **The hierarchy is the notation.** `crates/bench/scripts/ddc_numbers.py`
  builds the tree by dropping the last digit: `823.912` sits under
  `823.91`, then `823.9`, then the section `823`, the division `82` and the
  main class `8`. Any decimal notation implies this; the script reads no
  Dewey schedule to build it.
- **Labels are the notation itself** (`8xx`, `82x`, `823`, `823.912`). The
  scheme file contains no caption, note, table, Manual entry, relative-index
  term or summary from any edition of the Dewey Decimal Classification, not
  even the ten main-class names.
- **Only numbers that occur in the sample are in the tree**: the numbers on
  the 3,110 sampled books that carry one, and their notational prefixes.
  That is 6,778 numbers (see `results.json`, `schemes.ddc`). Most are
  built numbers specific to one book, and the tree has no number the
  sample does not use. DDC 23 enumerates tens of thousands of numbers and
  allows many more to be built.
- **Nothing is taken from OCLC.** No WebDewey, Dewey linked data, dewey.info,
  DDC Summaries, Classify or WorldCat data or service was used.
- **The tree cannot classify anything.** With no captions or vocabulary, a
  classifier cannot place a new book in it. It is used only to score
  librarian-assigned numbers. The Dewey-derived scheme that shelfmark can
  place books in is FDC, which is open in its own right (below).

## Why this is defensible

### 1. The numbers are published catalog data

The Open Library FAQ says: "Our catalog was created by the generous
donations of libraries who dedicated their data into the public domain."
It also asks that contributions be "given under the following Creative
Commons license: CC0 1.0 Universal license" [OL-FAQ]. The licensing page
adds: "The Internet Archive does not assert any new copyright or other
proprietary rights over any of the material in the Open Library database,"
and warns that "There may be existing rights issues on some contributions
and in some jurisdictions" [OL-LIC]. `dewey_decimal_class` is a standard
field of the edition record schema [OL-SCHEMA].

Much of this catalog data comes from library MARC records, whose 082 field
holds the Dewey number. The Library of Congress gives its own MARC records
away: "we now offer an 'open-access' provision, MDSConnect, at no cost to
users... nearly 25 million MARC records... made available primarily for
research and development usage" [LC-MDS].

Before any copyright argument, the plain fact is that these numbers are
printed in public catalog records whose publisher dedicates the catalog to
the public domain. The benchmark copies them from those records and from
nowhere else.

### 2. A single class number is not copyrightable

The US Copyright Office's practice manual is explicit: "Individual numbers
are not copyrightable and cannot be registered with the U.S. Copyright
Office," and "An individual number in and of itself never comprises
sufficient authorship to be copyrightable" (Compendium III §707.1)
[COMP-707]. Section 313.4(C) says the same of "individual numbers,
letters, sounds, and short phrases consisting of such elements" [COMP-313].
The regulation lists "Words and short phrases such as names, titles, and
slogans" among works not subject to copyright (37 CFR 202.1(a)) [CFR].

The closest appellate case is about numbers inside a classification:
*Southco, Inc. v. Kanebridge Corp.*, 390 F.3d 276 (3d Cir. 2004) (en banc).
The court held that "the Southco part numbers are not copyrightable"
because "each number is rigidly dictated by the rules of the Southco
system," and that they "are also excluded from copyright protection because
they are analogous to short phrases or the titles of works" [SOUTHCO]. A
Dewey number on a book is the same kind of object: a short string that
DDC's rules dictate once a cataloger has decided what the book is about.
Assigning that number to a particular book records a fact about the book,
and "That there can be no valid copyright in facts is universally
understood" (*Feist Publications v. Rural Telephone Service*, 499 U.S. 340,
1991) [FEIST].

### 3. The hierarchy is the notation's system, not OCLC's text

17 U.S.C. §102(b): "In no case does copyright protection for an original
work of authorship extend to any idea, procedure, process, system, method
of operation, concept, principle, or discovery, regardless of the form"
[USC102]. The rule that `823.9` is inside `823` is the decimal system's
method of operation. Applying it needs no expression from the schedules,
and the benchmark's tree is produced by that rule from numbers in public
records. *Baker v. Selden*, 101 U.S. 99 (1879), is the classic statement
that a copyright in a book describing a system does not give "the exclusive
right to the art or manufacture described therein" [BAKER].

### 4. The case against, and why the line still holds

The strongest contrary authority is *American Dental Ass'n v. Delta Dental
Plans Ass'n*, 126 F.3d 977 (7th Cir. 1997). Judge Easterbrook held that
"Classification is a creative endeavor," and that "all three elements of
the Code--numbers, short descriptions, and long descriptions, are
copyrightable subject matter" [ADA]. If a court followed *ADA* rather than
*Southco*, the Dewey schedules as a whole, captions included, would be
protected. Nobody disputes that anyway: OCLC states that "All copyright
rights in the Dewey Decimal Classification system are owned by OCLC, Inc."
[OCLC].

Even *ADA* draws the line where this benchmark stands. The same opinion
says: "Section 102(b) permits Delta to disseminate forms inviting dentists
to use the ADA's Code when submitting bills to insurers," and "Section
102(b) precludes the ADA from suing, for copyright infringement, a dentist
whose office files record treatments using the Code's nomenclature" [ADA].
The benchmark is the dentist's file: per-book records that carry the code
a professional assigned. It does not reproduce the descriptions (captions)
that *ADA* found expressive, nor the code book's selection and arrangement.
It holds only the numbers its sample happens to use, in the order the
notation dictates.

### 5. OCLC's own licence draws the same line

OCLC's WebDewey licence (Schedule 8, §3.1) lets subscribers put Dewey
numbers in catalog records, with one condition: "Such bibliographic records
and metadata may display DDC numbers, but shall not display DDC captions"
[SCHED8]. That schedule binds WebDewey subscribers, and this project is not
one. It is cited for what it shows about OCLC's position: OCLC itself
treats numbers in records as distributable and captions as the protected
part. The benchmark publishes numbers and no captions.

### 6. EU database right

The sui generis right lets a database maker "prevent extraction and/or
re-utilization of the whole or of a substantial part, evaluated
qualitatively and/or quantitatively, of the contents of that database"
(Directive 96/9/EC, art. 7(1)) [DIR]. Nothing here was extracted from an
OCLC database. The numbers come from Open Library's catalog, one record per
sampled book, and the 6,778 notation nodes are the numbers those books
happen to carry, with no schedule text. Recital 45 adds that the right "does not in any way constitute
an extension of copyright protection to mere facts or data" [DIR].

### 7. Trademarks

"Dewey, Dewey Decimal Classification, OCLC and WebDewey are registered
trademarks of OCLC, Inc." [OCLC]. The benchmark uses these names only to
say which classification the numbers belong to. The scheme file is titled
"Dewey Decimal Classification numbers (notation only, no schedule
captions)" and its source note states that the project is not affiliated
with or endorsed by OCLC. It is not offered as a Dewey product.

## FDC, for contrast

Free Decimal Correspondence (`schemes/fdc`) is a Dewey-shaped scheme with
captions. It is CC0, and its author wrote it from the 1922 11th edition of
Dewey, which is out of copyright (see `schemes/fdc/README.md`). The
benchmark places books in FDC two ways: by mapping each gold Dewey number to
the deepest FDC class whose number is a leading part of it or whose range
contains it, and by shelfmark's lexical classifier. The mapping uses only
the book's number and FDC's own CC0 file.

## Limits and residual risk

- This is not legal advice, and no court has ruled on a dataset of Dewey
  numbers drawn from catalog records. *ADA* and *Southco* disagree on
  numbers in classifications. The position rests on four points: numbers
  only, taken from public records, hierarchy from the notation, and no text.
- The Open Library licensing page itself notes possible "existing rights
  issues on some contributions." The benchmark commits only fields of
  records that Open Library publishes under its public-domain terms.
- Other jurisdictions are not covered.
- Anyone who extends this work must not add captions, summaries, Manual
  notes or index terms from any DDC edition after the 11th (1922). They must
  not fetch numbers or structure from OCLC services: report 4 records that
  OCLC's Dewey linked data terms forbid "scraping, retrieving, or indexing."

## Sources

- [OL-FAQ] Open Library, "Using Open Library data" FAQ. <https://openlibrary.org/help/faq/using>
- [OL-LIC] Open Library, "Licensing". <https://openlibrary.org/developers/licensing>
- [OL-SCHEMA] Open Library client, edition JSON schema (`dewey_decimal_class`). <https://raw.githubusercontent.com/internetarchive/openlibrary-client/master/olclient/schemata/edition.schema.json>
- [LC-MDS] Library of Congress, MARC Distribution Services. <https://www.loc.gov/cds/products/marcDist.php>
- [COMP-707] U.S. Copyright Office, Compendium III, ch. 700, §707.1 "Numbers". <https://www.copyright.gov/comp3/chap700/ch700-literary-works.pdf>
- [COMP-313] U.S. Copyright Office, Compendium III, ch. 300, §313.4(C). <https://www.copyright.gov/comp3/chap300/ch300-copyrightable-authorship.pdf>
- [CFR] 37 CFR 202.1. <https://www.law.cornell.edu/cfr/text/37/202.1>
- [USC102] 17 U.S.C. §102. <https://www.law.cornell.edu/uscode/text/17/102>
- [SOUTHCO] *Southco, Inc. v. Kanebridge Corp.*, 390 F.3d 276 (3d Cir. 2004) (en banc). <https://law.resource.org/pub/us/case/reporter/F3/390/390.F3d.276.02-1243.html>
- [FEIST] *Feist Publications, Inc. v. Rural Telephone Service Co.*, 499 U.S. 340 (1991). <https://www.law.cornell.edu/supremecourt/text/499/340>
- [BAKER] *Baker v. Selden*, 101 U.S. 99 (1879). <https://www.law.cornell.edu/supremecourt/text/101/99>
- [ADA] *American Dental Ass'n v. Delta Dental Plans Ass'n*, 126 F.3d 977 (7th Cir. 1997). <https://law.resource.org/pub/us/case/reporter/F3/126/126.F3d.977.96-4140.html>
- [OCLC] OCLC, "Dewey Services: resources". <https://www.oclc.org/en/dewey/resources.html>
- [SCHED8] OCLC, Schedule 8, WebDewey (August 2024), §3.1. <https://www.oclc.org/content/dam/legal/schedules/en_us/Schedule-08-WebDewey-EN-US.pdf>
- [DIR] Directive 96/9/EC on the legal protection of databases, art. 7 and recital 45. <https://eur-lex.europa.eu/legal-content/EN/TXT/HTML/?uri=CELEX:31996L0009>
- Report 4 of this directory (`4-licensing-crosswalks.md`) for the Dewey linked data terms and the DDC Summaries notice.
