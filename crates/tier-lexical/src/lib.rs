//! Lexical tiers for shelfmark (plan U4; R7, AE2).
//!
//! - [`VocabTier`] (`vocab`) matches normalized input text against every
//!   class label and vocabulary link of the scheme with one Aho-Corasick
//!   automaton built once per compiled index, weights matches by inverse
//!   class frequency over the scheme, and turns them into margin-aware
//!   scores in [0, 1].
//! - [`RulesTier`] (`rules`) applies configured rules
//!   `{when: {domain|term|regex}, class, weight}` from a JSON file.
//!
//! Both are deterministic: candidates are ranked by score, then class code
//! in byte order, and evidence is listed in a fixed order.

use std::cmp::Ordering;

use shelfmark_core::id::normalize_text_v1;
use shelfmark_core::tier::{Candidate, ClassifyInput};

mod rules;
mod vocab;

pub use rules::{Rule, RuleWhen, RulesConfig, RulesTier};
pub use vocab::{VocabMatcher, VocabTier, LABEL_SOURCE, QUALIFIER_DISCOUNT};

/// Rank best first: score, then class code in byte order (plan PD9).
pub(crate) fn rank(a: &Candidate, b: &Candidate) -> Ordering {
    b.score
        .total_cmp(&a.score)
        .then_with(|| a.class.code.as_bytes().cmp(b.class.code.as_bytes()))
}

/// The input's text fields, each normalized (v1) and non-empty: title,
/// text, then headings. Matching never spans two fields.
pub(crate) fn normalized_fields(input: &ClassifyInput) -> Vec<String> {
    input
        .title
        .iter()
        .chain(std::iter::once(&input.text))
        .chain(input.headings.iter())
        .map(|s| normalize_text_v1(s))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Microseconds since `start`, saturating.
pub(crate) fn micros_since(start: std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}
