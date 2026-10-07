//! The per-index phrase automaton behind the `vocab` tier.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use aho_corasick::{AhoCorasick, MatchKind};
use shelfmark_core::error::{Error, ErrorCode};
use shelfmark_core::id::{normalize_text_v1, SchemeId};
use shelfmark_core::scheme::CompiledIndex;

/// Source name recorded for matches against a class's own label.
pub const LABEL_SOURCE: &str = "label";

/// Weight factor for a vocabulary term with its parenthetical qualifier
/// dropped (`Compilers (Computer programs)` also matches `compilers`).
pub const QUALIFIER_DISCOUNT: f64 = 0.8;

/// One class a phrase points at, from one vocabulary source.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Link {
    pub class: String,
    pub source: String,
    pub weight: f64,
}

/// One normalized phrase and every class it points at.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Phrase {
    pub text: String,
    /// Inverse class frequency over the scheme, scaled to (0, 1]: 1 for a
    /// phrase naming one class, falling as more classes share it.
    pub idf: f64,
    /// One per class, sorted by class code.
    pub links: Vec<Link>,
}

/// Labels and vocabulary links of one compiled index, as an automaton
/// over normalized (v1) text. Built once per index content hash.
pub struct VocabMatcher {
    scheme: SchemeId,
    content_hash: String,
    automaton: AhoCorasick,
    pub(crate) phrases: Vec<Phrase>,
    /// Strict ancestors of every class, nearest first.
    pub(crate) ancestors: BTreeMap<String, Vec<String>>,
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

impl VocabMatcher {
    /// Build the automaton for `index`. Fails with `scheme_invalid` when a
    /// vocabulary weight is outside (0, 1].
    pub fn build(index: &dyn CompiledIndex) -> Result<Self, Error> {
        let codes = class_codes(index);
        // phrase -> (class, source) -> weight (max over duplicates)
        let mut table: BTreeMap<String, BTreeMap<(String, String), f64>> = BTreeMap::new();
        let mut add = |phrase: String, code: &str, source: &str, weight: f64| {
            if phrase.is_empty() {
                return;
            }
            let w = table
                .entry(phrase)
                .or_default()
                .entry((code.to_string(), source.to_string()))
                .or_insert(0.0);
            *w = w.max(weight);
        };
        let mut ancestors = BTreeMap::new();
        for code in &codes {
            if let Some(rec) = index.get(code) {
                add(normalize_text_v1(&rec.label), code, LABEL_SOURCE, 1.0);
            }
            for link in index.vocab().get(code).into_iter().flatten() {
                if !(link.weight.is_finite() && link.weight > 0.0 && link.weight <= 1.0) {
                    return Err(bad_weight(code, &link.term, link.weight));
                }
                let full = normalize_text_v1(&link.term);
                let bare = normalize_text_v1(&strip_qualifiers(&link.term));
                if bare != full {
                    add(bare, code, &link.source, link.weight * QUALIFIER_DISCOUNT);
                }
                add(full, code, &link.source, link.weight);
            }
            let up: Vec<String> = index
                .ancestors(code)
                .into_iter()
                .map(|r| r.id.code)
                .filter(|c| c != code)
                .collect();
            ancestors.insert(code.clone(), up);
        }

        let n = codes.len().max(1) as f64;
        let phrases: Vec<Phrase> = table
            .into_iter()
            .map(|(text, by_source)| {
                // One link per class: the strongest source, ties to the
                // first source name, so a label that is also a vocabulary
                // term counts once.
                let mut links: Vec<Link> = Vec::new();
                for ((class, source), weight) in by_source {
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
        let automaton = AhoCorasick::builder()
            .match_kind(MatchKind::Standard)
            .build(phrases.iter().map(|p| p.text.as_str()))
            .map_err(|e| Error::internal(format!("vocab automaton: {e}")))?;
        Ok(VocabMatcher {
            scheme: index.scheme().clone(),
            content_hash: index.content_hash().to_string(),
            automaton,
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
            let bytes = field.as_bytes();
            let mut hits: Vec<(usize, usize, usize)> = self
                .automaton
                .find_overlapping_iter(field.as_str())
                .filter(|m| {
                    (m.start() == 0 || bytes[m.start() - 1] == b' ')
                        && (m.end() == bytes.len() || bytes[m.end()] == b' ')
                })
                .map(|m| (m.start(), m.end(), m.pattern().as_usize()))
                .collect();
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
