# Identifier table findings

The table builder converts edition records into deterministic identifier-to-
classification rows. Inputs are normalized, duplicate observations are
collapsed, and each result records the source call number and confidence.

## Limits

- A record may carry conflicting classifications. The builder preserves the
  observations so downstream users can inspect rather than silently discard
  disagreement.
- Coverage depends on the source collection and on whether records include
  standard identifiers. A missing row is not evidence that an item has no
  classification.
- Building from a complete public catalogue export can require substantial
  memory and storage. The committed sample is deliberately small.
- Classification ranges are reduced to their deepest unambiguous shared
  class. Broad ranges therefore provide less detail than a specific call
  number.

The committed sample and tests demonstrate normalization, deduplication,
deterministic output, and deepest-covering-class selection. Full catalogue
measurements are not included.
