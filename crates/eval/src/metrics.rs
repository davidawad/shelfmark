//! R16's metrics over cascade outcomes, sliced per profile, per deciding
//! tier and per hierarchy level.

use std::collections::BTreeMap;

use shelfmark_core::calibration::{Metrics, BINS};

/// Ranks counted by `topk` unless the caller says otherwise.
pub const DEFAULT_K: usize = 5;

/// One profile's result on one gold example. Every class is its path from
/// the scheme root down to itself (`["Q", "QA", "QA76"]`), so hierarchy
/// levels compare by prefix.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub profile: String,
    /// The tier that decided; `None` when the claim was flagged or no
    /// tier produced a candidate.
    pub tier: Option<String>,
    /// Candidate classes, best first; empty when no tier produced one.
    pub ranked: Vec<Vec<String>>,
    /// Every class the gold set accepts for the example.
    pub acceptable: Vec<Vec<String>>,
    pub confidence: f64,
    pub flagged: bool,
    pub micros: u64,
}

impl Outcome {
    /// Whether one of the first `k` candidates is acceptable: exactly
    /// (`level` `None`) or down to `level` (1 is the root's children).
    fn hit(&self, k: usize, level: Option<usize>) -> bool {
        self.ranked
            .iter()
            .take(k)
            .any(|p| self.acceptable.iter().any(|g| matches(p, g, level)))
    }

    /// Whether some acceptable class is at least `level` deep.
    fn reaches(&self, level: usize) -> bool {
        self.acceptable.iter().any(|g| g.len() >= level)
    }
}

fn matches(pred: &[String], gold: &[String], level: Option<usize>) -> bool {
    match level {
        None => pred == gold,
        Some(l) => gold.len() >= l && pred.len() >= l && pred[..l] == gold[..l],
    }
}

/// [`summarize`]'s three slicings. Tier and level keys are prefixed with
/// the profile (`profile:lexical@1/vocab`, `profile:lexical@1/2`); an
/// outcome no tier decided counts under the tier `none`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Slices {
    pub per_profile: BTreeMap<String, Metrics>,
    pub per_tier: BTreeMap<String, Metrics>,
    pub per_level: BTreeMap<String, Metrics>,
}

pub fn summarize(outcomes: &[Outcome], k: usize) -> Slices {
    let mut by_profile: BTreeMap<&str, Vec<&Outcome>> = BTreeMap::new();
    for o in outcomes {
        by_profile.entry(&o.profile).or_default().push(o);
    }
    let mut out = Slices::default();
    for (profile, items) in by_profile {
        out.per_profile
            .insert(profile.to_string(), metrics(&items, k, None));
        let mut by_tier: BTreeMap<&str, Vec<&Outcome>> = BTreeMap::new();
        for o in &items {
            by_tier
                .entry(o.tier.as_deref().unwrap_or("none"))
                .or_default()
                .push(o);
        }
        for (tier, slice) in by_tier {
            out.per_tier
                .insert(format!("{profile}/{tier}"), metrics(&slice, k, None));
        }
        let depth = items
            .iter()
            .flat_map(|o| o.acceptable.iter().map(Vec::len))
            .max()
            .unwrap_or(0);
        for level in 1..=depth {
            let slice: Vec<&Outcome> = items.iter().copied().filter(|o| o.reaches(level)).collect();
            out.per_level.insert(
                format!("{profile}/{level}"),
                metrics(&slice, k, Some(level)),
            );
        }
    }
    out
}

fn metrics(items: &[&Outcome], k: usize, level: Option<usize>) -> Metrics {
    let n = items.len();
    let rate = |f: &dyn Fn(&Outcome) -> bool| {
        if n == 0 {
            0.0
        } else {
            items.iter().filter(|o| f(o)).count() as f64 / n as f64
        }
    };
    let scored: Vec<(f64, bool)> = items
        .iter()
        .map(|o| (o.confidence, o.hit(1, level)))
        .collect();
    let mut micros: Vec<u64> = items.iter().map(|o| o.micros).collect();
    micros.sort_unstable();
    Metrics {
        n,
        top1: rate(&|o| o.hit(1, level)),
        topk: rate(&|o| o.hit(k, level)),
        k,
        ece: ece(&scored),
        flagged_rate: rate(&|o| o.flagged),
        p50_micros: percentile(&micros, 0.50),
        p95_micros: percentile(&micros, 0.95),
    }
}

/// Expected calibration error over [`BINS`] equal-width confidence bins.
fn ece(scored: &[(f64, bool)]) -> f64 {
    if scored.is_empty() {
        return 0.0;
    }
    let mut bins = [(0usize, 0.0f64, 0usize); BINS];
    for &(conf, correct) in scored {
        let b = &mut bins[bin(conf)];
        b.0 += 1;
        b.1 += conf;
        b.2 += usize::from(correct);
    }
    let total = scored.len() as f64;
    bins.iter()
        .filter(|(n, _, _)| *n > 0)
        .map(|&(n, conf, hits)| {
            let n = n as f64;
            (conf / n - hits as f64 / n).abs() * n / total
        })
        .sum()
}

/// The equal-width bin of a score in [0, 1], as `TierCalibration::apply`
/// indexes it.
pub(crate) fn bin(score: f64) -> usize {
    ((score.clamp(0.0, 1.0) * BINS as f64) as usize).min(BINS - 1)
}

/// Nearest-rank percentile of sorted values; 0 when empty.
fn percentile(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (p * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}
