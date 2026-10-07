//! Accretion contracts (R15-R17, plan PD3): gold examples, the fitted
//! calibration the cascade applies, and the eval summary `status` shows.
//!
//! The store (U6) writes [`GoldExample`]s, the eval (U10) fits a
//! [`Calibration`] and writes an [`EvalSummary`], and the CLI (U9) reads
//! both without depending on either crate.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::id::{sha256_hex, ClassId, ItemId, PlacementId, SchemeId, WorkId};
use crate::placement::{canonical_json, Provenance};
use crate::tier::ClassifyInput;

/// Number of equal-width raw-score bins (plan PD3).
pub const BINS: usize = 10;

/// A labelled example: a person confirmed or corrected a Work's class in
/// one scheme version (R15). It carries the classify input so the eval can
/// re-run every profile on it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoldExample {
    pub work: WorkId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<ItemId>,
    pub scheme: SchemeId,
    pub class: ClassId,
    pub input: ClassifyInput,
    /// The machine claim this label confirmed or corrected, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_placement: Option<PlacementId>,
    pub provenance: Provenance,
}

/// Hash identifying a gold set: count plus a hash over every example's
/// canonical JSON in order. Calibration is stale when the current gold
/// set's hash differs from the one it was fitted on (R17).
pub fn gold_set_hash(examples: &[GoldExample]) -> String {
    let mut all = String::new();
    for e in examples {
        let v = serde_json::to_value(e).expect("gold example serializes");
        all.push_str(&canonical_json(&v));
        all.push('\n');
    }
    format!("s256:{}", sha256_hex(all.as_bytes()))
}

/// Raw-score-to-confidence map for one tier: the calibrated value of each
/// of the [`BINS`] equal-width bins, monotone non-decreasing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TierCalibration {
    pub bins: Vec<f64>,
    /// Gold examples the fit saw for this tier.
    pub support: usize,
}

impl TierCalibration {
    pub fn apply(&self, raw: f64) -> f64 {
        if self.bins.is_empty() {
            return raw;
        }
        let i = ((raw * self.bins.len() as f64) as usize).min(self.bins.len() - 1);
        self.bins[i]
    }
}

/// A fitted calibration (written by the eval, applied by the cascade).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    pub scheme: SchemeId,
    /// [`gold_set_hash`] of the examples it was fitted on.
    pub gold_hash: String,
    pub gold_count: usize,
    /// RFC 3339.
    pub fitted_at: String,
    /// Per tier name; a tier with no entry keeps its raw scores.
    pub tiers: BTreeMap<String, TierCalibration>,
}

impl Calibration {
    pub fn apply(&self, tier: &str, raw: f64) -> f64 {
        self.tiers.get(tier).map_or(raw, |t| t.apply(raw))
    }

    pub fn is_stale(&self, current_gold_hash: &str) -> bool {
        self.gold_hash != current_gold_hash
    }
}

/// Metrics for one slice (a profile, a tier, or a hierarchy level).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metrics {
    pub n: usize,
    pub top1: f64,
    pub topk: f64,
    pub k: usize,
    /// Expected calibration error over [`BINS`] bins.
    pub ece: f64,
    pub flagged_rate: f64,
    pub p50_micros: u64,
    pub p95_micros: u64,
}

/// The latest eval's results (R16), as `status` reports them (R23).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalSummary {
    pub scheme: SchemeId,
    pub gold_hash: String,
    pub gold_count: usize,
    /// RFC 3339.
    pub at: String,
    pub per_profile: BTreeMap<String, Metrics>,
    pub per_tier: BTreeMap<String, Metrics>,
    /// Keyed by hierarchy level (`"0"`, `"1"`, ...).
    pub per_level: BTreeMap<String, Metrics>,
}

/// `InputHashes.config`: hash of the caller's effective config (rules,
/// tables, target, calibration) as canonical JSON.
pub fn config_hash(config: &serde_json::Value) -> String {
    format!("s256:{}", sha256_hex(canonical_json(config).as_bytes()))
}
