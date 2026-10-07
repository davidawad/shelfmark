//! The compiled index: a [`CompiledIndex`] over one validated scheme.

use std::collections::{BTreeMap, HashMap};

use regex::Regex;
use serde::{Deserialize, Serialize};
use shelfmark_core::id::{ClassId, SchemeId};
use shelfmark_core::scheme::{
    ClassRecord, CodeCheck, CodeRange, CompiledIndex, SchemeDoc, SearchHit, VocabLink,
};
use shelfmark_core::Error;

use crate::code::{Key, Span};
use crate::search::{self, Postings, Text};
use crate::validate::grammar;

/// Bumped whenever [`Stored`] changes shape, so old cache entries miss.
pub(crate) const FORMAT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Node {
    pub code: String,
    pub label: String,
    pub parent: Option<u32>,
    pub level: u32,
    pub range: Option<CodeRange>,
    pub children: Vec<u32>,
}

/// The serialized compiled form (private to this crate, plan PD7).
/// Classes are in shelf order, so every listing is too.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Stored {
    pub format: u32,
    pub scheme: SchemeId,
    pub content_hash: String,
    pub code_grammar: String,
    pub classes: Vec<Node>,
    pub roots: Vec<u32>,
    pub vocab: BTreeMap<String, Vec<VocabLink>>,
    pub texts: Vec<Text>,
    pub postings: Postings,
}

impl Stored {
    /// Compile a validated document.
    pub(crate) fn build(doc: &SchemeDoc, content_hash: String) -> Stored {
        let mut order: Vec<usize> = (0..doc.classes.len()).collect();
        let keys: Vec<Key> = doc.classes.iter().map(|c| Key::parse(&c.code)).collect();
        order.sort_by(|&a, &b| {
            keys[a]
                .cmp(&keys[b])
                .then_with(|| doc.classes[a].code.cmp(&doc.classes[b].code))
        });
        let pos: HashMap<&str, u32> = order
            .iter()
            .enumerate()
            .map(|(p, &i)| (doc.classes[i].code.as_str(), p as u32))
            .collect();
        let mut classes: Vec<Node> = order
            .iter()
            .map(|&i| {
                let c = &doc.classes[i];
                Node {
                    code: c.code.clone(),
                    label: c.label.clone(),
                    parent: c.parent.as_deref().map(|p| pos[p]),
                    level: 0,
                    range: c.range.clone(),
                    children: Vec::new(),
                }
            })
            .collect();
        let mut roots = Vec::new();
        for p in 0..classes.len() {
            let (mut level, mut up) = (0, classes[p].parent);
            while let Some(u) = up {
                level += 1;
                up = classes[u as usize].parent;
            }
            classes[p].level = level;
            match classes[p].parent {
                Some(u) => classes[u as usize].children.push(p as u32),
                None => roots.push(p as u32),
            }
        }
        let (mut texts, mut postings) = (Vec::new(), Postings::new());
        let mut vocab = BTreeMap::new();
        for (p, &i) in order.iter().enumerate() {
            let c = &doc.classes[i];
            search::add(&mut texts, &mut postings, p as u32, &c.label, 1.0);
            for v in &c.vocab {
                search::add(&mut texts, &mut postings, p as u32, &v.term, v.weight);
            }
            vocab.insert(c.code.clone(), c.vocab.clone());
        }
        Stored {
            format: FORMAT,
            scheme: doc.id(),
            content_hash,
            code_grammar: doc.code_grammar.clone(),
            classes,
            roots,
            vocab,
            texts,
            postings,
        }
    }
}

/// A compiled scheme, ready for lookup, browsing and search.
#[derive(Debug, Clone)]
pub struct SchemeIndex {
    pub(crate) stored: Stored,
    grammar: Regex,
    by_code: HashMap<String, u32>,
    keys: Vec<Key>,
    spans: Vec<Option<Span>>,
}

impl SchemeIndex {
    pub(crate) fn from_stored(stored: Stored) -> Result<SchemeIndex, Error> {
        let grammar = grammar(&stored.code_grammar)
            .map_err(|e| Error::internal(format!("compiled code_grammar does not compile: {e}")))?;
        let by_code = stored
            .classes
            .iter()
            .enumerate()
            .map(|(p, n)| (n.code.clone(), p as u32))
            .collect();
        let keys = stored.classes.iter().map(|n| Key::parse(&n.code)).collect();
        let spans = stored
            .classes
            .iter()
            .map(|n| n.range.as_ref().map(Span::of))
            .collect();
        Ok(SchemeIndex {
            stored,
            grammar,
            by_code,
            keys,
            spans,
        })
    }

    fn class(&self, p: u32) -> ClassId {
        ClassId::new(
            self.stored.scheme.clone(),
            &self.stored.classes[p as usize].code,
        )
    }

    fn record(&self, p: u32) -> ClassRecord {
        let n = &self.stored.classes[p as usize];
        ClassRecord {
            id: self.class(p),
            label: n.label.clone(),
            parent: n.parent.map(|u| self.class(u)),
            level: n.level,
            range: n.range.clone(),
        }
    }

    /// The deepest class whose code or range covers `key`; ties go to the
    /// earliest in shelf order.
    fn covering(&self, key: &Key) -> Option<u32> {
        let mut best: Option<u32> = None;
        for (p, n) in self.stored.classes.iter().enumerate() {
            let covers =
                key.under(&self.keys[p]) || self.spans[p].as_ref().is_some_and(|s| s.contains(key));
            if covers && best.is_none_or(|b| n.level > self.stored.classes[b as usize].level) {
                best = Some(p as u32);
            }
        }
        best
    }
}

impl CompiledIndex for SchemeIndex {
    fn scheme(&self) -> &SchemeId {
        &self.stored.scheme
    }

    fn content_hash(&self) -> &str {
        &self.stored.content_hash
    }

    fn len(&self) -> usize {
        self.stored.classes.len()
    }

    fn get(&self, code: &str) -> Option<ClassRecord> {
        self.by_code.get(code).map(|&p| self.record(p))
    }

    fn children(&self, code: Option<&str>) -> Vec<ClassRecord> {
        let kids = match code {
            None => &self.stored.roots,
            Some(c) => match self.by_code.get(c) {
                Some(&p) => &self.stored.classes[p as usize].children,
                None => return Vec::new(),
            },
        };
        kids.iter().map(|&p| self.record(p)).collect()
    }

    fn ancestors(&self, code: &str) -> Vec<ClassRecord> {
        let mut out = Vec::new();
        let mut cur = self.by_code.get(code).copied();
        while let Some(p) = cur {
            out.push(self.record(p));
            cur = self.stored.classes[p as usize].parent;
        }
        out
    }

    fn search(&self, text: &str, limit: usize) -> Vec<SearchHit> {
        let classes = &self.stored.classes;
        search::rank(
            &self.stored.texts,
            &self.stored.postings,
            text,
            limit,
            |p| classes[p as usize].code.as_str(),
        )
        .into_iter()
        .map(|(p, score, t)| SearchHit {
            class: self.class(p),
            score,
            matched: self.stored.texts[t as usize].text.clone(),
        })
        .collect()
    }

    fn check_code(&self, code: &str) -> CodeCheck {
        if !self.grammar.is_match(code) {
            return CodeCheck::Invalid {
                reason: format!(
                    "`{code}` does not match {} code_grammar `{}`",
                    self.stored.scheme, self.stored.code_grammar
                ),
            };
        }
        if let Some(&p) = self.by_code.get(code) {
            return CodeCheck::Exact {
                class: self.class(p),
            };
        }
        match self.covering(&Key::parse(code)) {
            Some(p) => CodeCheck::Within {
                class: self.class(p),
            },
            None => CodeCheck::Unplaced,
        }
    }

    fn vocab(&self) -> &BTreeMap<String, Vec<VocabLink>> {
        &self.stored.vocab
    }
}
