//! The per-index phrase table behind the `vocab` tier.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use shelfmark_core::error::{Error, ErrorCode};
use shelfmark_core::id::{normalize_text_v1, SchemeId};
use shelfmark_core::scheme::CompiledIndex;

use super::stopwords::is_uninformative;
use super::trie::TokenTrie;

/// Source name recorded for matches against a class's own label.
pub const LABEL_SOURCE: &str = "label";

/// Weight factor for a vocabulary term with its parenthetical qualifier
/// dropped (`Compilers (Computer programs)` also matches `compilers`).
pub const QUALIFIER_DISCOUNT: f64 = 0.8;

/// One class a phrase points at, from one vocabulary source. `class` and
/// `source` index [`VocabMatcher::codes`] and [`VocabMatcher::sources`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Link {
    pub class: u32,
    pub source: u32,
    pub weight: f64,
}

/// One normalized phrase and every class it points at.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Phrase {
    pub text: String,
    /// Inverse class frequency over the scheme, scaled to (0, 1]: 1 for a
    /// phrase naming one class, falling as more classes share it.
    pub idf: f64,
    /// One per class, sorted by class code.
    pub links: Vec<Link>,
}

/// Labels and vocabulary links of one compiled index, as a token trie
/// over normalized (v1) text. Built once per index content hash.
#[derive(PartialEq, Serialize, Deserialize)]
pub struct VocabMatcher {
    pub(crate) scheme: SchemeId,
    pub(crate) content_hash: String,
    pub(crate) trie: TokenTrie,
    /// Every class code, in byte order; a code's index is its number.
    pub(crate) codes: Vec<String>,
    /// Vocabulary source names, in byte order.
    pub(crate) sources: Vec<String>,
    /// Sorted by text; a phrase's index is its number.
    pub(crate) phrases: Vec<Phrase>,
    /// Strict ancestors of every class, nearest first, by class number.
    pub(crate) ancestors: Vec<Vec<u32>>,
}

impl std::fmt::Debug for VocabMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VocabMatcher")
            .field("scheme", &self.scheme)
            .field("content_hash", &self.content_hash)
            .field("phrases", &self.phrases.len())
            .finish()
    }
}

/// Every class code the index knows: the hierarchy from the top plus any
/// code that carries vocabulary.
fn class_codes(index: &dyn CompiledIndex) -> BTreeSet<String> {
    let mut codes: BTreeSet<String> = index.vocab().keys().cloned().collect();
    let mut queue: VecDeque<String> = index
        .children(None)
        .into_iter()
        .map(|r| r.id.code)
        .collect();
    let mut seen = BTreeSet::new();
    while let Some(code) = queue.pop_front() {
        if seen.insert(code.clone()) {
            queue.extend(index.children(Some(&code)).into_iter().map(|r| r.id.code));
            codes.insert(code);
        }
    }
    codes
}

/// The term without parenthetical qualifiers, if that differs.
fn strip_qualifiers(term: &str) -> String {
    let mut depth = 0usize;
    let mut out = String::with_capacity(term.len());
    for ch in term.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

fn bad_weight(code: &str, term: &str, weight: f64) -> Error {
    Error::new(
        ErrorCode::SchemeInvalid,
        format!("vocabulary term {term:?} on class {code} has weight {weight}; weights must be in (0, 1]"),
    )
    .next("shelfmark scheme validate <scheme file>")
}

/// Phrase text to its `(class, source, weight)` entries, in insertion order.
#[derive(Default)]
struct Table {
    entries: HashMap<String, Vec<(u32, u32, f64)>>,
}

impl Table {
    /// Phrases made only of stopwords are dropped (see `stopwords`).
    fn add(&mut self, phrase: String, class: u32, source: u32, weight: f64) {
        if !is_uninformative(&phrase) {
            self.entries
                .entry(phrase)
                .or_default()
                .push((class, source, weight));
        }
    }

    /// Phrases sorted by text, each with one link per class: the strongest
    /// source, ties to the first source name, so a label that is also a
    /// vocabulary term counts once. `n` is the class count for the idf.
    fn into_phrases(self, n: usize) -> Vec<Phrase> {
        let n = n.max(1) as f64;
        let mut phrases: Vec<Phrase> = self
            .entries
            .into_iter()
            .map(|(text, mut entries)| {
                // Sources are numbered in name order, so this is (class,
                // source name) order.
                entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
                let mut links: Vec<Link> = Vec::new();
                for (class, source, weight) in entries {
                    match links.last_mut() {
                        Some(l) if l.class == class => {
                            if weight > l.weight {
                                *l = Link {
                                    class,
                                    source,
                                    weight,
                                };
                            }
                        }
                        _ => links.push(Link {
                            class,
                            source,
                            weight,
                        }),
                    }
                }
                let df = links.len() as f64;
                Phrase {
                    text,
                    idf: (1.0 + n / df).ln() / (1.0 + n).ln(),
                    links,
                }
            })
            .collect();
        phrases.sort_unstable_by(|a, b| a.text.cmp(&b.text));
        phrases
    }
}

impl VocabMatcher {
    /// Build the phrase table for `index`. Fails with `scheme_invalid` when
    /// a vocabulary weight is outside (0, 1].
    pub fn build(index: &dyn CompiledIndex) -> Result<Self, Error> {
        let codes: Vec<String> = class_codes(index).into_iter().collect();
        let number: HashMap<&str, u32> = codes
            .iter()
            .enumerate()
            .map(|(i, c)| (c.as_str(), i as u32))
            .collect();
        let sources: Vec<String> = std::iter::once(LABEL_SOURCE.to_string())
            .chain(index.vocab().values().flatten().map(|l| l.source.clone()))
            .collect::<BTreeSet<String>>()
            .into_iter()
            .collect();
        let source_number = |s: &str| sources.binary_search_by(|x| x.as_str().cmp(s)).ok();
        let label = source_number(LABEL_SOURCE).expect("label source is listed") as u32;

        let mut table = Table::default();
        let mut ancestors = Vec::with_capacity(codes.len());
        for (class, code) in codes.iter().enumerate() {
            let class = class as u32;
            if let Some(rec) = index.get(code) {
                table.add(normalize_text_v1(&rec.label), class, label, 1.0);
            }
            for link in index.vocab().get(code).into_iter().flatten() {
                if !(link.weight.is_finite() && link.weight > 0.0 && link.weight <= 1.0) {
                    return Err(bad_weight(code, &link.term, link.weight));
                }
                let source = source_number(&link.source).expect("source is listed") as u32;
                let full = normalize_text_v1(&link.term);
                if link.term.contains(['(', ')']) {
                    let bare = normalize_text_v1(&strip_qualifiers(&link.term));
                    if bare != full {
                        table.add(bare, class, source, link.weight * QUALIFIER_DISCOUNT);
                    }
                }
                table.add(full, class, source, link.weight);
            }
            let up: Vec<u32> = index
                .ancestors(code)
                .into_iter()
                .filter(|r| &r.id.code != code)
                .filter_map(|r| number.get(r.id.code.as_str()).copied())
                .collect();
            ancestors.push(up);
        }

        let phrases = table.into_phrases(codes.len());
        let trie = TokenTrie::build(phrases.iter().map(|p| p.text.as_str()));
        Ok(VocabMatcher {
            scheme: index.scheme().clone(),
            content_hash: index.content_hash().to_string(),
            trie,
            codes,
            sources,
            phrases,
            ancestors,
        })
    }

    pub fn scheme(&self) -> &SchemeId {
        &self.scheme
    }

    /// Content hash of the index this matcher was built from.
    pub fn content_hash(&self) -> &str {
        &self.content_hash
    }

    /// Whether this matcher was built from `index` as it is now.
    pub fn is_for(&self, index: &dyn CompiledIndex) -> bool {
        &self.scheme == index.scheme() && self.content_hash == index.content_hash()
    }

    /// Number of distinct normalized phrases.
    pub fn phrase_count(&self) -> usize {
        self.phrases.len()
    }

    /// Whole-word phrase occurrences in normalized `fields`, by phrase
    /// index. Overlapping hits resolve leftmost-longest, so a phrase inside
    /// a longer matched phrase is not counted again.
    pub(crate) fn count(&self, fields: &[String]) -> BTreeMap<usize, u32> {
        let mut counts = BTreeMap::new();
        for field in fields {
            let mut hits = self.trie.hits(field);
            hits.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
            let mut end = 0;
            for (s, e, pid) in hits {
                if s >= end {
                    *counts.entry(pid).or_insert(0) += 1;
                    end = e;
                }
            }
        }
        counts
    }
}
