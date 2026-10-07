//! Test doubles for crates that build against the contracts before the
//! real scheme compiler and tiers land (feature `testkit`).
//!
//! [`MemIndex`] is a naive [`CompiledIndex`] over a [`SchemeDoc`]: exact
//! codes, parent links, substring search. [`FixedTier`] returns scores
//! it was given. Neither is fit for production use.

use std::collections::BTreeMap;

use crate::error::{Error, ErrorCode};
use crate::id::{sha256_hex, ClassId, SchemeId};
use crate::scheme::{
    ClassDef, ClassRecord, CodeCheck, CompiledIndex, SchemeDoc, SearchHit, VocabLink,
};
use crate::tier::{Candidate, ClassifyInput, Cost, Tier, TierInfo, TierOutput};

pub struct MemIndex {
    id: SchemeId,
    hash: String,
    classes: BTreeMap<String, ClassDef>,
    vocab: BTreeMap<String, Vec<VocabLink>>,
}

impl MemIndex {
    pub fn new(doc: &SchemeDoc) -> Self {
        let json = serde_json::to_string(doc).expect("scheme serializes");
        MemIndex {
            id: doc.id(),
            hash: format!("s256:{}", sha256_hex(json.as_bytes())),
            classes: doc
                .classes
                .iter()
                .map(|c| (c.code.clone(), c.clone()))
                .collect(),
            vocab: doc
                .classes
                .iter()
                .map(|c| (c.code.clone(), c.vocab.clone()))
                .collect(),
        }
    }

    /// A tiny scheme: H (social sciences) and K (law) at the top, with
    /// QA76 under QA.
    pub fn sample() -> Self {
        let class = |code: &str, label: &str, parent: Option<&str>, terms: &[&str]| ClassDef {
            code: code.into(),
            label: label.into(),
            parent: parent.map(Into::into),
            range: None,
            vocab: terms
                .iter()
                .map(|t| VocabLink {
                    term: (*t).into(),
                    source: "test".into(),
                    source_id: None,
                    weight: 1.0,
                })
                .collect(),
            note: None,
        };
        MemIndex::new(&SchemeDoc {
            schema: None,
            name: "test".into(),
            version: "1".into(),
            title: "Test scheme".into(),
            source: None,
            code_grammar: "[A-Z]{1,3}[0-9.]*".into(),
            classes: vec![
                class("H", "Social sciences", None, &["economics", "society"]),
                class("K", "Law", None, &["law", "statute"]),
                class("QA", "Mathematics", None, &["mathematics"]),
                class(
                    "QA76",
                    "Computer software",
                    Some("QA"),
                    &["software", "compilers"],
                ),
            ],
        })
    }

    pub fn class(&self, code: &str) -> ClassId {
        ClassId::new(self.id.clone(), code)
    }

    fn record(&self, code: &str) -> Option<ClassRecord> {
        let def = self.classes.get(code)?;
        let mut level = 0;
        let mut up = def.parent.clone();
        while let Some(p) = up {
            level += 1;
            up = self.classes.get(&p).and_then(|d| d.parent.clone());
        }
        Some(ClassRecord {
            id: self.class(code),
            label: def.label.clone(),
            parent: def.parent.as_deref().map(|p| self.class(p)),
            level,
            range: def.range.clone(),
        })
    }
}

impl CompiledIndex for MemIndex {
    fn scheme(&self) -> &SchemeId {
        &self.id
    }
    fn content_hash(&self) -> &str {
        &self.hash
    }
    fn len(&self) -> usize {
        self.classes.len()
    }
    fn get(&self, code: &str) -> Option<ClassRecord> {
        self.record(code)
    }
    fn children(&self, code: Option<&str>) -> Vec<ClassRecord> {
        self.classes
            .values()
            .filter(|c| c.parent.as_deref() == code)
            .filter_map(|c| self.record(&c.code))
            .collect()
    }
    fn ancestors(&self, code: &str) -> Vec<ClassRecord> {
        let mut out = Vec::new();
        let mut cur = Some(code.to_string());
        while let Some(c) = cur {
            let Some(rec) = self.record(&c) else { break };
            cur = rec.parent.as_ref().map(|p| p.code.clone());
            out.push(rec);
        }
        out
    }
    fn search(&self, text: &str, limit: usize) -> Vec<SearchHit> {
        let q = text.to_lowercase();
        self.classes
            .values()
            .filter(|c| c.label.to_lowercase().contains(&q))
            .take(limit)
            .map(|c| SearchHit {
                class: self.class(&c.code),
                score: 1.0,
                matched: c.label.clone(),
            })
            .collect()
    }
    fn check_code(&self, code: &str) -> CodeCheck {
        if self.classes.contains_key(code) {
            return CodeCheck::Exact {
                class: self.class(code),
            };
        }
        let within = self
            .classes
            .keys()
            .filter(|k| code.starts_with(k.as_str()))
            .max_by_key(|k| k.len());
        match within {
            Some(k) => CodeCheck::Within {
                class: self.class(k),
            },
            None => CodeCheck::Unplaced,
        }
    }
    fn vocab(&self) -> &BTreeMap<String, Vec<VocabLink>> {
        &self.vocab
    }
}

/// A tier that returns fixed `(code, score)` pairs, or `tier_unavailable`
/// when built with [`FixedTier::unavailable`].
pub struct FixedTier {
    pub name: String,
    pub scores: Vec<(String, f64)>,
    pub available: bool,
    pub model_calls: u32,
}

impl FixedTier {
    pub fn new(name: &str, scores: &[(&str, f64)]) -> Self {
        FixedTier {
            name: name.into(),
            scores: scores.iter().map(|(c, s)| ((*c).into(), *s)).collect(),
            available: true,
            model_calls: 0,
        }
    }

    pub fn unavailable(name: &str) -> Self {
        FixedTier {
            available: false,
            ..FixedTier::new(name, &[])
        }
    }
}

impl Tier for FixedTier {
    fn info(&self) -> TierInfo {
        TierInfo {
            name: self.name.clone(),
            version: "fixed-1".into(),
            needs_model: self.model_calls > 0,
            expected_memory_mb: 0,
            expected_micros_per_item: 0,
        }
    }

    fn run(
        &self,
        _: &ClassifyInput,
        index: &dyn CompiledIndex,
        _: &[Candidate],
    ) -> Result<TierOutput, Error> {
        if !self.available {
            return Err(Error::new(
                ErrorCode::TierUnavailable,
                format!("{} is not built in", self.name),
            ));
        }
        Ok(TierOutput {
            candidates: self
                .scores
                .iter()
                .map(|(code, score)| Candidate {
                    class: ClassId::new(index.scheme().clone(), code),
                    score: *score,
                    evidence: Vec::new(),
                })
                .collect(),
            cost: Cost {
                micros: 0,
                model_calls: self.model_calls,
            },
        })
    }
}
