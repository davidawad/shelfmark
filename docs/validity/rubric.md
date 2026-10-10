# Judging rubric: would a careful librarian put these two works together?

This rubric is fixed before judging and used unchanged for every pair and both
passes. It is written for a judge who sees, for each pair, only:

- the name of the classification scheme;
- for each of the two works: its title and abstract (abstracts are cut at
  about 900 characters);
- for each work: the class path the scheme assigned it, from the top class
  down to the assigned class, as `code — label` steps.

The judge does not see how the pair was selected, how similar the two texts
are to any model, or what the benchmark metric thinks of the pair. Judge
from the subject matter of the works and the meaning of the classes only.

## The judge's stance

Act as a careful subject cataloguer in a large research library that holds
every discipline. You classify by *what a work is about* (its subject,
problem and method), not by shared vocabulary. Two works that both use
"neural networks" or "entropy" may belong far apart; two works with no words
in common may belong side by side. The question behind every label is: *would
a reader who wants one of these works be well served by finding the other
next to it?*

## Labels

Give every pair all six fields. Each field is forced: pick the best answer,
even when unsure.

### 1. `relation` — how close are the two works in subject?

- `same-topic`: they address the same specific subject: the same problem,
  object or phenomenon at the level of a research specialty (e.g. both on
  elliptic curves over finite fields; both on dark-matter halo profiles; both
  on graph neural networks for recommendation). A specialist in one would
  normally read the other.
- `same-field`: same discipline or broad field (both pure mathematics, both
  astrophysics, both machine learning, both economics), but different
  specialties.
- `different-field`: different disciplines (e.g. a number-theory paper and a
  condensed-matter paper), even if they share a method or vocabulary.

When a work is interdisciplinary, judge by its main contribution (what its
title and abstract say it delivers), not by its side applications.

### 2. `together` — would a careful librarian put them together?

- `together`: a careful librarian classifying both works with *any* good
  general or subject scheme would place them in the same specific subdivision
  of the same discipline, so that they sit close together on the shelf or in
  a browse list. This is normally the case for `same-topic`, and for
  `same-field` pairs only when the two specialties are closely allied (a
  scheme would plausibly put them under one narrow heading).
- `apart`: otherwise. This is normally the case for `different-field`, and
  for `same-field` pairs whose specialties a scheme would separate below the
  discipline level.

### 3. `scheme_decision` — is this scheme's decision correct?

Compare the two class paths. Find where they diverge: the longest shared
prefix is where the scheme brings the two works together. If they share no
step, the scheme separates them at its very top. Then judge whether *the
depth and meaning of the shared part* matches the real subject relation:

- `correct`: the shared part of the paths is about as specific as the
  works' actual subject overlap, *within what this scheme can express*.
  Examples: two `different-field` works that share no step or only a very
  broad top class; two `same-topic` works in the same specific class or
  adjacent specific classes; two `same-field` works that meet at the
  discipline class and split below it.
- `incorrect`: the scheme's decision is clearly wrong in either direction:
  - *false split*: the works belong together (or in the same field) but the
    scheme separates them at a level far broader than their overlap, e.g.
    two papers on the same problem under different top classes;
  - *false join*: the works do not belong together but the scheme puts them
    in the same specific class or a narrow common branch, e.g. a
    number-theory paper and a fluid-dynamics paper in one narrow class.

Judge the scheme's *decision for this pair as placed*, including the effect
of misplacements: if one work is in a wrong class and that makes the pair
falsely joined or falsely split, the decision is `incorrect`. If a work is
misplaced but the resulting depth of meeting is still right for the pair
(e.g. two unrelated works that happen to be separated anyway), the decision is
`correct`.

Tie-breaks. Schemes differ in granularity; do not penalize a coarse scheme for
being coarse, nor a fine scheme for being fine, as long as the meeting point
is sensible for that scheme. Meeting one level too high or too low is still
`correct`; only a clear mismatch (a whole discipline too broad, or a narrow
class joining unrelated subjects) is `incorrect`. Odd but defensible
placements are `correct`.

### 4–5. `class_1_ok`, `class_2_ok` — is each assigned class defensible?

- `yes`: a cataloguer could defend the assigned class (the last step of the
  path) for that work, even if it is not the best one.
- `no`: the class is about a different subject from the work (a
  misclassification), e.g. a paper on prime gaps placed under "Animal
  husbandry", or a class chosen only for a word that appears in the title.

### 6. `reason`

At most 25 words: the deciding consideration, e.g. "same topic (Ramsey
numbers); scheme splits them at the top: false split".

## Output format

One JSON object per pair, one per line:

```json
{"uid": "…", "relation": "same-field", "together": "apart", "scheme_decision": "correct", "class_1_ok": "yes", "class_2_ok": "no", "reason": "…"}
```

`class_1_ok` refers to `class_path_1`/`work_1`, `class_2_ok` to
`class_path_2`/`work_2`.
