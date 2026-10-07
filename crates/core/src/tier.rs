//! L2 tiers and profiles (R7-R10).
//!
//! A tier scores candidate classes for one input against one compiled
//! scheme. A profile names an ordered list of tiers plus what running it
//! costs. The cascade (`crate::cascade`) runs a profile's tiers and applies
//! the stop rule.

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::id::{ClassId, ProfileId, WorkId};
use crate::scheme::CompiledIndex;

/// An exact identifier found in or supplied with a Work (R7).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Identifier {
    /// ISBN-13 digits (ISBN-10 normalized to 13).
    Isbn(String),
    Lccn(String),
    /// Lowercased DOI without the `https://doi.org/` prefix.
    Doi(String),
    Arxiv(String),
    /// Registrable domain, lowercased, without `www.`.
    UrlDomain(String),
}

impl Identifier {
    /// Table key form, e.g. `isbn:9780262033848` (plan PD5).
    pub fn key(&self) -> String {
        let (t, v) = match self {
            Identifier::Isbn(v) => ("isbn", v),
            Identifier::Lccn(v) => ("lccn", v),
            Identifier::Doi(v) => ("doi", v),
            Identifier::Arxiv(v) => ("arxiv", v),
            Identifier::UrlDomain(v) => ("domain", v),
        };
        format!("{t}:{v}")
    }
}

/// What a tier classifies: the Work plus whatever was extracted from it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ClassifyInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work: Option<WorkId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Extracted text (may be truncated by the caller).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identifiers: Vec<Identifier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Subject headings supplied with the Work (e.g. from a catalog record).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headings: Vec<String>,
}

/// Why a tier scored a class: what it matched.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Evidence {
    Identifier {
        identifier: Identifier,
        table: String,
    },
    Vocab {
        term: String,
        source: String,
        count: u32,
    },
    Rule {
        rule: String,
    },
    Similarity {
        model: String,
    },
}

/// One scored class from one tier. `score` is the tier's raw score in
/// [0, 1] (finite; anything else is an `internal` error); the cascade
/// applies the fitted calibration, if any, before comparing it with the
/// target. A tier may list a class more than once; the cascade merges
/// them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub class: ClassId,
    pub score: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
}

/// Cost actually spent by one tier run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Cost {
    /// Wall time in microseconds. Excluded from placement hashing, so
    /// timing never breaks determinism.
    pub micros: u64,
    /// Model calls made (0 for lexical tiers).
    pub model_calls: u32,
}

impl std::ops::Add for Cost {
    type Output = Cost;

    fn add(self, other: Cost) -> Cost {
        Cost {
            micros: self.micros + other.micros,
            model_calls: self.model_calls + other.model_calls,
        }
    }
}

/// What a tier declares about itself before it runs (R9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TierInfo {
    /// Stable tier name, e.g. `identifier`, `vocab`, `rules`, `shortlist`.
    pub name: String,
    /// Version of the tier's logic or model; part of the profile hash.
    pub version: String,
    /// Whether this tier needs a model download.
    pub needs_model: bool,
    pub expected_memory_mb: u32,
    pub expected_micros_per_item: u64,
}

/// One tier's output for one input.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TierOutput {
    /// Sorted best first (the cascade re-sorts defensively).
    pub candidates: Vec<Candidate>,
    pub cost: Cost,
}

/// A step in the cascade.
pub trait Tier: Send + Sync {
    fn info(&self) -> TierInfo;
    /// `prior` holds the candidates of earlier tiers, so shortlist and
    /// rerank tiers can narrow rather than search the whole scheme.
    fn run(
        &self,
        input: &ClassifyInput,
        index: &dyn CompiledIndex,
        prior: &[Candidate],
    ) -> Result<TierOutput, Error>;
}

/// A named, versioned cascade (R8, R9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub id: ProfileId,
    pub description: String,
    /// Tier names in run order.
    pub tiers: Vec<String>,
    /// Default target confidence when the caller gives none (R10).
    pub default_target: f64,
    /// Cascade margin below which a decision is flagged even if it
    /// reached the target.
    pub min_margin: f64,
    pub expected_memory_mb: u32,
    pub expected_micros_per_item: u64,
}
