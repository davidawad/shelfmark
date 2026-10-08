//! The PD3 calibration fit (R17): per tier, gold-set accuracy binned by
//! raw score into [`BINS`] equal-width bins, then made monotone by
//! pool-adjacent-violators weighted by each bin's count.
//!
//! Each bin's accuracy is smoothed to `(hits + 1) / (n + 2)` before the
//! fit, so a bin with one lucky example does not claim certainty. A bin
//! with no examples takes the value of the nearest fitted bin below it
//! (the first fitted bin for leading empties), which keeps the map
//! monotone.

use std::collections::BTreeMap;

use shelfmark_core::calibration::{Calibration, TierCalibration, BINS};
use shelfmark_core::SchemeId;

use crate::metrics::bin;

/// A tier's top raw score on one gold example and whether that class
/// was acceptable.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub tier: String,
    pub raw: f64,
    pub correct: bool,
}

/// One tier's fit; `None` without samples.
pub fn fit_tier(samples: &[(f64, bool)]) -> Option<TierCalibration> {
    if samples.is_empty() {
        return None;
    }
    let mut counts = [(0usize, 0usize); BINS];
    for &(raw, correct) in samples {
        let c = &mut counts[bin(raw)];
        c.0 += 1;
        c.1 += usize::from(correct);
    }
    // Pool adjacent violators over the non-empty bins: (value, weight,
    // bins pooled).
    let mut blocks: Vec<(f64, f64, Vec<usize>)> = Vec::new();
    for (i, &(n, hits)) in counts.iter().enumerate().filter(|(_, c)| c.0 > 0) {
        let value = (hits as f64 + 1.0) / (n as f64 + 2.0);
        blocks.push((value, n as f64, vec![i]));
        while blocks.len() > 1 && blocks[blocks.len() - 2].0 > blocks[blocks.len() - 1].0 {
            let (v2, w2, b2) = blocks.pop().expect("two blocks");
            let last = blocks.last_mut().expect("two blocks");
            last.0 = (last.0 * last.1 + v2 * w2) / (last.1 + w2);
            last.1 += w2;
            last.2.extend(b2);
        }
    }
    let mut fitted: [Option<f64>; BINS] = [None; BINS];
    for (value, _, members) in &blocks {
        for &i in members {
            fitted[i] = Some(*value);
        }
    }
    let first = blocks[0].0;
    let mut prev = first;
    let bins = fitted
        .iter()
        .map(|v| {
            prev = v.unwrap_or(prev);
            prev
        })
        .collect();
    Some(TierCalibration {
        bins,
        support: samples.len(),
    })
}

/// A [`Calibration`] for every tier with samples.
pub fn fit(
    scheme: SchemeId,
    gold_hash: String,
    gold_count: usize,
    fitted_at: String,
    samples: &[Sample],
) -> Calibration {
    let mut per_tier: BTreeMap<&str, Vec<(f64, bool)>> = BTreeMap::new();
    for s in samples {
        per_tier
            .entry(&s.tier)
            .or_default()
            .push((s.raw, s.correct));
    }
    let tiers = per_tier
        .into_iter()
        .filter_map(|(t, s)| fit_tier(&s).map(|c| (t.to_string(), c)))
        .collect();
    Calibration {
        scheme,
        gold_hash,
        gold_count,
        fitted_at,
        tiers,
    }
}

/// The lowest raw score whose calibrated confidence reaches `target`
/// (the lower edge of the first bin that does), or `None` if no bin does.
pub fn thresholds(calibration: &TierCalibration, target: f64) -> Option<f64> {
    let n = calibration.bins.len();
    calibration
        .bins
        .iter()
        .position(|&v| v >= target)
        .map(|i| i as f64 / n as f64)
}
