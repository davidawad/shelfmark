//! Predictions: one NDJSON line per (item, profile) — the shape the
//! classifier's `classify` emits in batch mode, reduced to what eval needs.

use crate::error::{ndjson_lines, read_file, EvalError, Result};
use crate::gold::GoldSet;
use crate::lcc::Class;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Candidate {
    /// LCC code, range, or typed class ID (`class:lcc@2024:QA76`).
    pub class: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PredictionLine {
    /// Gold `id` or L0 `item:` ID.
    pub item: String,
    #[serde(default = "default_profile")]
    pub profile: String,
    /// The tier that decided (`identifier`, `vocabulary`, `rules`, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    /// Ranked best first; empty means the classifier abstained.
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    /// Confidence in the top candidate; defaults to its score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub flagged: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fast: Vec<String>,
}

fn default_profile() -> String {
    "default".into()
}

/// A prediction joined to its gold item.
#[derive(Debug, Clone)]
pub struct Prediction {
    pub gold: usize,
    pub profile: String,
    pub tier: String,
    pub classes: Vec<Class>,
    pub confidence: f64,
    pub flagged: bool,
    pub latency_ms: Option<f64>,
    pub fast: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Loaded {
    pub predictions: Vec<Prediction>,
    /// Item keys that matched no gold item (with their line numbers).
    pub unknown: Vec<(usize, String)>,
}

fn bad(code: &'static str, at: &str, ln: usize, msg: impl std::fmt::Display) -> EvalError {
    EvalError::new(
        code,
        format!("{at} line {ln}: {msg}"),
        "fix the line to match schemas/prediction.schema.json, then `mc-eval validate`",
    )
}

pub fn parse(text: &str, gold: &GoldSet) -> Result<Loaded> {
    let mut out = Loaded::default();
    parse_into(text, "predictions", gold, &mut out, &mut HashSet::new())?;
    Ok(out)
}

fn parse_into(
    text: &str,
    at: &str,
    gold: &GoldSet,
    out: &mut Loaded,
    seen: &mut HashSet<(usize, String)>,
) -> Result<()> {
    for (ln, line) in ndjson_lines(text) {
        let p: PredictionLine =
            serde_json::from_str(line).map_err(|e| bad("E_PRED_PARSE", at, ln, e))?;
        let Some(g) = gold.lookup(&p.item) else {
            out.unknown.push((ln, p.item));
            continue;
        };
        if !seen.insert((g, p.profile.clone())) {
            let msg = format!(
                "second prediction for `{}` under profile `{}`",
                p.item, p.profile
            );
            return Err(bad("E_PRED_DUP", at, ln, msg));
        }
        let mut classes = Vec::with_capacity(p.candidates.len());
        for c in &p.candidates {
            classes.push(Class::parse_id(&c.class).map_err(|e| bad("E_PRED_CODE", at, ln, e))?);
            if let Some(s) = c.score {
                if !s.is_finite() {
                    return Err(bad("E_PRED_SCORE", at, ln, "candidate score is not finite"));
                }
            }
        }
        let confidence = p
            .confidence
            .or_else(|| p.candidates.first().and_then(|c| c.score))
            .unwrap_or(0.0);
        if !(0.0..=1.0).contains(&confidence) {
            let msg = format!("confidence {confidence} is outside [0, 1]");
            return Err(bad("E_PRED_CONFIDENCE", at, ln, msg));
        }
        if let Some(l) = p.latency_ms {
            if !(l.is_finite() && l >= 0.0) {
                return Err(bad(
                    "E_PRED_LATENCY",
                    at,
                    ln,
                    "latency_ms must be a non-negative number",
                ));
            }
        }
        out.predictions.push(Prediction {
            gold: g,
            profile: p.profile,
            tier: p.tier.unwrap_or_else(|| "unknown".into()),
            classes,
            confidence,
            flagged: p.flagged,
            latency_ms: p.latency_ms,
            fast: p.fast,
        });
    }
    Ok(())
}

/// Load and concatenate several prediction files; duplicates across files
/// are errors like duplicates within one.
pub fn load(paths: &[String], gold: &GoldSet) -> Result<Loaded> {
    let mut out = Loaded::default();
    let mut seen = HashSet::new();
    for p in paths {
        let text = read_file(
            p,
            "pass existing prediction NDJSON files with --predictions",
        )?;
        parse_into(&text, p, gold, &mut out, &mut seen)?;
    }
    Ok(out)
}
