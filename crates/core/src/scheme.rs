//! L1 scheme data model (R4) and the compiled-index interface (R5, R6).
//!
//! `SchemeDoc` mirrors `schemas/scheme.schema.json` field for field; the
//! `schema_sync` test keeps them aligned. A scheme compiles into some
//! implementation of [`CompiledIndex`]; the compiled on-disk form is the
//! scheme crate's business, never a contract.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::id::{ClassId, SchemeId};

/// The published schema's `$id`; scheme files name it in `$schema`.
pub const SCHEME_SCHEMA_ID: &str = "https://shelfmark.dev/schemas/scheme/v1.json";

/// One scheme data file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemeDoc {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Short scheme name, e.g. `lcc`.
    pub name: String,
    /// Scheme version, e.g. `2024` (part of every class ID).
    pub version: String,
    pub title: String,
    /// Where the data came from and under what terms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// Regex (Rust `regex` syntax, anchored by the validator) every class
    /// code must match.
    pub code_grammar: String,
    pub classes: Vec<ClassDef>,
}

impl SchemeDoc {
    pub fn id(&self) -> SchemeId {
        SchemeId::new(&self.name, &self.version)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// One class: a code, a label, an optional parent and range, and the
/// vocabulary that links words and headings to it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassDef {
    pub code: String,
    pub label: String,
    /// Parent class code; absent for a top-level class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// The span of codes this class covers, e.g. QA76-QA76.95.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<CodeRange>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vocab: Vec<VocabLink>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodeRange {
    pub start: String,
    pub end: String,
}

/// A term or heading that points at a class (e.g. the LCSH heading
/// "Compilers (Computer programs)" carries QA76.76.C65).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VocabLink {
    pub term: String,
    /// Vocabulary the term came from, e.g. `lcsh`, `label`, `user`.
    pub source: String,
    /// Identifier of the term in its vocabulary, e.g. `sh86007588`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    /// Relative strength in (0, 1]; default 1.
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub weight: f64,
}

fn one() -> f64 {
    1.0
}

fn is_one(w: &f64) -> bool {
    (*w - 1.0).abs() < f64::EPSILON
}

/// A class as the index returns it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassRecord {
    pub id: ClassId,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<ClassId>,
    /// Depth from a top-level class (top level is 0).
    pub level: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<CodeRange>,
}

/// A text-search hit over labels and vocabulary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchHit {
    pub class: ClassId,
    pub score: f64,
    /// The label or vocabulary term that matched.
    pub matched: String,
}

/// Result of validating a code against the scheme (R6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CodeCheck {
    /// The code is itself a class.
    Exact { class: ClassId },
    /// The code is grammatical and falls inside this class's range or
    /// under it as a deeper code.
    Within { class: ClassId },
    /// Grammatical but not covered by any class.
    Unplaced,
    /// Does not match the scheme's code grammar.
    Invalid { reason: String },
}

/// The runtime form of a compiled scheme. Implemented by the scheme crate;
/// tiers, the cascade and every surface only see this trait.
pub trait CompiledIndex: Send + Sync {
    fn scheme(&self) -> &SchemeId;
    /// Hash of the source scheme file; part of every placement's inputs.
    fn content_hash(&self) -> &str;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get(&self, code: &str) -> Option<ClassRecord>;
    fn children(&self, code: Option<&str>) -> Vec<ClassRecord>;
    /// From the class itself up to its top-level ancestor.
    fn ancestors(&self, code: &str) -> Vec<ClassRecord>;
    fn search(&self, text: &str, limit: usize) -> Vec<SearchHit>;
    fn check_code(&self, code: &str) -> CodeCheck;
    /// Every vocabulary link per class code, for lexical tiers.
    fn vocab(&self) -> &BTreeMap<String, Vec<VocabLink>>;
}
