//! Per-item scoring: how deep each ranked candidate agrees with the gold
//! classes. Everything downstream (accuracy, calibration, risk-coverage)
//! aggregates these records.

use crate::gold::GoldSet;
use crate::lcc::Class;
use crate::pred::Prediction;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub gold: usize,
    /// Deepest level any acceptable class reaches.
    pub gold_depth: usize,
    /// `reach[r]`: deepest level at which one of the top `r + 1` candidates
    /// agrees with an acceptable class (cumulative max over ranks).
    pub reach: Vec<usize>,
    /// Hierarchical precision and recall of the top candidate against its
    /// best-matching acceptable class (by F1).
    pub h_precision: f64,
    pub h_recall: f64,
    pub confidence: f64,
    pub flagged: bool,
    pub latency_ms: Option<f64>,
    /// No prediction existed for this gold item.
    pub missing: bool,
    /// FAST tag counts (true positives, false positives, false negatives);
    /// `None` when the gold item has no FAST tags.
    pub fast: Option<(usize, usize, usize)>,
}

impl Outcome {
    /// Is the top-`k` list correct at `level`? `None` when the gold item does
    /// not reach that level (it is then outside that level's denominator).
    pub fn correct(&self, level: usize, k: usize) -> Option<bool> {
        if level > self.gold_depth {
            return None;
        }
        let r = self
            .reach
            .get(k.saturating_sub(1))
            .or(self.reach.last())
            .copied()
            .unwrap_or(0);
        Some(r >= level)
    }

    pub fn h_f1(&self) -> f64 {
        let (p, r) = (self.h_precision, self.h_recall);
        if p + r == 0.0 {
            0.0
        } else {
            2.0 * p * r / (p + r)
        }
    }
}

fn best_depth(c: &Class, gold: &[Class]) -> usize {
    gold.iter().map(|g| c.matched_depth(g)).max().unwrap_or(0)
}

fn fast_counts(gold: &[String], pred: &[String]) -> Option<(usize, usize, usize)> {
    if gold.is_empty() {
        return None;
    }
    let norm = |v: &[String]| {
        v.iter()
            .map(|s| s.trim().to_lowercase())
            .collect::<BTreeSet<_>>()
    };
    let (g, p) = (norm(gold), norm(pred));
    let tp = g.intersection(&p).count();
    Some((tp, p.len() - tp, g.len() - tp))
}

pub fn score(gold: &GoldSet, p: &Prediction) -> Outcome {
    let acceptable = &gold.classes[p.gold];
    let mut reach = Vec::with_capacity(p.classes.len());
    let mut best = 0;
    for c in &p.classes {
        best = best.max(best_depth(c, acceptable));
        reach.push(best);
    }
    let (mut hp, mut hr, mut hf) = (0.0, 0.0, -1.0);
    if let Some(top) = p.classes.first() {
        for g in acceptable {
            let m = top.matched_depth(g) as f64;
            let (cp, cr) = (m / top.depth() as f64, m / g.depth() as f64);
            let f = if cp + cr == 0.0 {
                0.0
            } else {
                2.0 * cp * cr / (cp + cr)
            };
            if f > hf {
                (hp, hr, hf) = (cp, cr, f);
            }
        }
    }
    Outcome {
        gold: p.gold,
        gold_depth: gold.depth(p.gold),
        reach,
        h_precision: hp,
        h_recall: hr,
        confidence: p.confidence,
        flagged: p.flagged,
        latency_ms: p.latency_ms,
        missing: false,
        fast: fast_counts(&gold.items[p.gold].fast, &p.fast),
    }
}

/// The outcome recorded for a gold item that received no prediction:
/// wrong at every level, zero confidence.
pub fn missing(gold: &GoldSet, i: usize) -> Outcome {
    Outcome {
        gold: i,
        gold_depth: gold.depth(i),
        reach: Vec::new(),
        h_precision: 0.0,
        h_recall: 0.0,
        confidence: 0.0,
        flagged: false,
        latency_ms: None,
        missing: true,
        fast: fast_counts(&gold.items[i].fast, &[]),
    }
}
