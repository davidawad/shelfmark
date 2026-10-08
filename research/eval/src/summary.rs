//! Aggregate metrics over a set of outcomes. Every scalar is a [`Stat`] in a
//! fixed position so bootstrap resamples line up field by field.

use crate::lcc::{LEVEL_NAMES, MAX_LEVEL};
use crate::outcome::Outcome;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Stat {
    pub value: f64,
    /// Percentile bootstrap interval; absent when bootstrap is off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci: Option<[f64; 2]>,
}

impl Stat {
    pub fn of(value: f64) -> Stat {
        Stat { value, ci: None }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Config {
    /// Ranks reported as top-k (1 is always reported as top-1).
    pub ks: Vec<usize>,
    /// Level used for calibration bins, risk-coverage and thresholds.
    pub primary_level: usize,
    pub bins: usize,
    pub target_precisions: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Level {
    pub level: usize,
    pub name: &'static str,
    /// Items whose gold reaches this level (the denominator).
    pub n: usize,
    pub top1: Stat,
    pub topk: BTreeMap<String, Stat>,
    pub ece: Stat,
    pub brier: Stat,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bin {
    pub lo: f64,
    pub hi: f64,
    pub n: usize,
    pub mean_confidence: f64,
    pub accuracy: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskPoint {
    pub coverage: f64,
    pub risk: f64,
    /// Lowest confidence still accepted at this coverage.
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Threshold {
    pub target_precision: f64,
    /// Accept a placement when confidence >= this; `null` if unreachable.
    pub threshold: Option<f64>,
    pub coverage: f64,
    pub precision: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub n: usize,
    pub missing: usize,
    pub levels: Vec<Level>,
    pub hierarchical: BTreeMap<&'static str, Stat>,
    pub calibration_level: usize,
    pub reliability: Vec<Bin>,
    pub aurc: Stat,
    pub risk_coverage: Vec<RiskPoint>,
    pub thresholds: Vec<Threshold>,
    pub flagged_rate: Stat,
    /// Primary-level top-1 accuracy among flagged and unflagged items.
    pub accuracy_flagged: Stat,
    pub accuracy_unflagged: Stat,
    pub latency_ms: BTreeMap<&'static str, Stat>,
    /// Micro-averaged FAST precision/recall/F1; empty when gold has no FAST.
    pub fast: BTreeMap<&'static str, Stat>,
}

fn mean(xs: impl Iterator<Item = f64>) -> f64 {
    let (s, n) = xs.fold((0.0, 0usize), |(s, n), x| (s + x, n + 1));
    if n == 0 {
        f64::NAN
    } else {
        s / n as f64
    }
}

/// Nearest-rank percentile of sorted values.
pub fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let rank = (q * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

fn ece_brier(pairs: &[(f64, bool)], bins: usize) -> (f64, f64, Vec<Bin>) {
    if pairs.is_empty() {
        return (f64::NAN, f64::NAN, Vec::new());
    }
    let mut acc = vec![(0usize, 0.0f64, 0usize); bins];
    for &(c, ok) in pairs {
        let b = ((c * bins as f64) as usize).min(bins - 1);
        acc[b].0 += 1;
        acc[b].1 += c;
        acc[b].2 += usize::from(ok);
    }
    let n = pairs.len() as f64;
    let mut ece = 0.0;
    let mut out = Vec::new();
    for (i, &(k, csum, hits)) in acc.iter().enumerate() {
        if k == 0 {
            continue;
        }
        let (mc, a) = (csum / k as f64, hits as f64 / k as f64);
        ece += k as f64 / n * (a - mc).abs();
        out.push(Bin {
            lo: i as f64 / bins as f64,
            hi: (i + 1) as f64 / bins as f64,
            n: k,
            mean_confidence: mc,
            accuracy: a,
        });
    }
    let brier = mean(
        pairs
            .iter()
            .map(|&(c, ok)| (c - f64::from(u8::from(ok))).powi(2)),
    );
    (ece, brier, out)
}

/// Sort by confidence, highest first; ties keep gold order so results are
/// deterministic.
fn ranked(pairs: &[(f64, bool)]) -> Vec<(f64, bool)> {
    let mut v = pairs.to_vec();
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    v
}

fn risk_coverage(pairs: &[(f64, bool)], targets: &[f64]) -> (f64, Vec<RiskPoint>, Vec<Threshold>) {
    let n = pairs.len();
    if n == 0 {
        let t = targets.iter().map(|&p| Threshold {
            target_precision: p,
            threshold: None,
            coverage: 0.0,
            precision: f64::NAN,
        });
        return (f64::NAN, Vec::new(), t.collect());
    }
    let v = ranked(pairs);
    let mut errors = 0usize;
    let mut risks = Vec::with_capacity(n);
    for (_, ok) in &v {
        errors += usize::from(!ok);
        risks.push(errors as f64 / (risks.len() + 1) as f64);
    }
    let aurc = mean(risks.iter().copied());
    // Cut points only between distinct confidences: a threshold cannot split ties.
    let cut = |m: usize| m == n || v[m].0 < v[m - 1].0;
    let mut points = Vec::new();
    for d in 1..=10 {
        let m = ((d as f64 / 10.0) * n as f64).ceil() as usize;
        let m = (m..=n).find(|&m| cut(m)).unwrap_or(n);
        if points
            .last()
            .is_some_and(|p: &RiskPoint| p.coverage == m as f64 / n as f64)
        {
            continue;
        }
        points.push(RiskPoint {
            coverage: m as f64 / n as f64,
            risk: risks[m - 1],
            threshold: v[m - 1].0,
        });
    }
    let thresholds = targets
        .iter()
        .map(|&t| {
            let best = (1..=n).rev().find(|&m| cut(m) && 1.0 - risks[m - 1] >= t);
            match best {
                Some(m) => Threshold {
                    target_precision: t,
                    threshold: Some(v[m - 1].0),
                    coverage: m as f64 / n as f64,
                    precision: 1.0 - risks[m - 1],
                },
                None => Threshold {
                    target_precision: t,
                    threshold: None,
                    coverage: 0.0,
                    precision: f64::NAN,
                },
            }
        })
        .collect();
    (aurc, points, thresholds)
}

pub fn summarize(outs: &[&Outcome], cfg: &Config) -> Summary {
    let pairs_at = |level: usize| -> Vec<(f64, bool)> {
        outs.iter()
            .filter_map(|o| o.correct(level, 1).map(|ok| (o.confidence, ok)))
            .collect()
    };
    let mut levels = Vec::with_capacity(MAX_LEVEL);
    for level in 1..=MAX_LEVEL {
        let pairs = pairs_at(level);
        let rate = |k: usize| {
            mean(
                outs.iter()
                    .filter_map(|o| o.correct(level, k))
                    .map(|b| f64::from(u8::from(b))),
            )
        };
        let (ece, brier, _) = ece_brier(&pairs, cfg.bins);
        levels.push(Level {
            level,
            name: LEVEL_NAMES[level - 1],
            n: pairs.len(),
            top1: Stat::of(rate(1)),
            topk: cfg
                .ks
                .iter()
                .map(|&k| (k.to_string(), Stat::of(rate(k))))
                .collect(),
            ece: Stat::of(ece),
            brier: Stat::of(brier),
        });
    }
    let primary = pairs_at(cfg.primary_level);
    let (_, _, reliability) = ece_brier(&primary, cfg.bins);
    let (aurc, risk_coverage, thresholds) = risk_coverage(&primary, &cfg.target_precisions);
    let acc_where = |flag: bool| {
        mean(
            outs.iter()
                .filter(|o| o.flagged == flag)
                .filter_map(|o| o.correct(cfg.primary_level, 1))
                .map(|b| f64::from(u8::from(b))),
        )
    };
    let mut lat: Vec<f64> = outs.iter().filter_map(|o| o.latency_ms).collect();
    lat.sort_by(f64::total_cmp);
    let hierarchical = BTreeMap::from([
        ("accuracy", Stat::of(mean(outs.iter().map(|o| o.h_recall)))),
        (
            "precision",
            Stat::of(mean(outs.iter().map(|o| o.h_precision))),
        ),
        ("f1", Stat::of(mean(outs.iter().map(|o| o.h_f1())))),
    ]);
    let latency_ms = BTreeMap::from([
        ("mean", Stat::of(mean(lat.iter().copied()))),
        ("p50", Stat::of(percentile(&lat, 0.5))),
        ("p95", Stat::of(percentile(&lat, 0.95))),
        ("max", Stat::of(lat.last().copied().unwrap_or(f64::NAN))),
    ]);
    let mut fast = BTreeMap::new();
    let counts: Vec<_> = outs.iter().filter_map(|o| o.fast).collect();
    if !counts.is_empty() {
        let (tp, fp, fn_) = counts
            .iter()
            .fold((0, 0, 0), |a, c| (a.0 + c.0, a.1 + c.1, a.2 + c.2));
        let p = if tp + fp == 0 {
            0.0
        } else {
            tp as f64 / (tp + fp) as f64
        };
        let r = if tp + fn_ == 0 {
            0.0
        } else {
            tp as f64 / (tp + fn_) as f64
        };
        let f = if p + r == 0.0 {
            0.0
        } else {
            2.0 * p * r / (p + r)
        };
        fast = BTreeMap::from([
            ("precision", Stat::of(p)),
            ("recall", Stat::of(r)),
            ("f1", Stat::of(f)),
        ]);
    }
    Summary {
        n: outs.len(),
        missing: outs.iter().filter(|o| o.missing).count(),
        levels,
        hierarchical,
        calibration_level: cfg.primary_level,
        reliability,
        aurc: Stat::of(aurc),
        risk_coverage,
        thresholds,
        flagged_rate: Stat::of(mean(outs.iter().map(|o| f64::from(u8::from(o.flagged))))),
        accuracy_flagged: Stat::of(acc_where(true)),
        accuracy_unflagged: Stat::of(acc_where(false)),
        latency_ms,
        fast,
    }
}

impl Summary {
    /// Every bootstrapped scalar, in a fixed order.
    pub fn stats_mut(&mut self) -> Vec<&mut Stat> {
        let mut v: Vec<&mut Stat> = Vec::new();
        for l in &mut self.levels {
            v.push(&mut l.top1);
            v.extend(l.topk.values_mut());
            v.push(&mut l.ece);
            v.push(&mut l.brier);
        }
        v.extend(self.hierarchical.values_mut());
        v.push(&mut self.aurc);
        v.push(&mut self.flagged_rate);
        v.push(&mut self.accuracy_flagged);
        v.push(&mut self.accuracy_unflagged);
        v.extend(self.latency_ms.values_mut());
        v.extend(self.fast.values_mut());
        v
    }
}
