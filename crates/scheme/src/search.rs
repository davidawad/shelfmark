//! Deterministic token search over class labels and vocabulary terms.
//!
//! Text is tokenized with Work-ID normalization v1 (NFKC, lowercase,
//! punctuation as space). A text scores `weight * 2m / (|q| + |t|)`, the
//! Dice overlap of its `m` shared tokens with the query, scaled by the
//! vocabulary weight (labels weigh 1). A class keeps its best text; hits
//! sort by score, then class code in byte order.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::normalize_text_v1;

/// One searchable text: a label or a vocabulary term.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Text {
    pub class: u32,
    pub text: String,
    pub weight: f64,
    /// Distinct tokens in the text.
    pub tokens: u32,
}

pub(crate) fn tokens(text: &str) -> BTreeSet<String> {
    normalize_text_v1(text)
        .split(' ')
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

/// Token -> ascending text indices.
pub(crate) type Postings = BTreeMap<String, Vec<u32>>;

pub(crate) fn add(
    texts: &mut Vec<Text>,
    postings: &mut Postings,
    class: u32,
    text: &str,
    weight: f64,
) {
    let toks = tokens(text);
    if toks.is_empty() {
        return;
    }
    let idx = texts.len() as u32;
    for t in &toks {
        postings.entry(t.clone()).or_default().push(idx);
    }
    texts.push(Text {
        class,
        text: text.to_string(),
        weight,
        tokens: toks.len() as u32,
    });
}

/// Best `(score, text index)` per class, sorted best first; `code` names
/// a class for the tie-break.
pub(crate) fn rank<'a>(
    texts: &[Text],
    postings: &Postings,
    query: &str,
    limit: usize,
    code: impl Fn(u32) -> &'a str,
) -> Vec<(u32, f64, u32)> {
    let q = tokens(query);
    if q.is_empty() || limit == 0 {
        return Vec::new();
    }
    let mut shared: BTreeMap<u32, u32> = BTreeMap::new();
    for t in &q {
        for &i in postings.get(t).map(Vec::as_slice).unwrap_or_default() {
            *shared.entry(i).or_default() += 1;
        }
    }
    let mut best: BTreeMap<u32, (f64, u32)> = BTreeMap::new();
    for (i, m) in shared {
        let t = &texts[i as usize];
        let score = t.weight * 2.0 * f64::from(m) / (q.len() as f64 + f64::from(t.tokens));
        let slot = best.entry(t.class).or_insert((score, i));
        if score > slot.0 {
            *slot = (score, i);
        }
    }
    let mut hits: Vec<(u32, f64, u32)> = best.into_iter().map(|(c, (s, i))| (c, s, i)).collect();
    hits.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| code(a.0).as_bytes().cmp(code(b.0).as_bytes()))
    });
    hits.truncate(limit);
    hits
}
