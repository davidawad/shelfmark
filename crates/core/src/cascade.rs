//! The cascade stop rule (R7, R10, plan PD9).
//!
//! Tiers run in profile order. Each tier's raw scores are calibrated
//! (plan PD3) and merged per class. The first tier whose top candidate
//! reaches the target confidence with at least the profile's minimum
//! margin over its own runner-up decides, and later tiers are skipped.
//! Otherwise the top-ranked candidate across every tier that ran is taken,
//! flagged. Ranking is by score, then class code in byte order, so
//! identical inputs always give an identical decision.

use std::cmp::Ordering;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use crate::calibration::Calibration;
use crate::error::{Error, ErrorCode};
use crate::id::ClassId;
use crate::placement::{Status, TierRun, TierRunState, TierScore, Trace, TraceCandidate};
use crate::scheme::CompiledIndex;
use crate::tier::{Candidate, ClassifyInput, Cost, Profile, Tier};

/// What the cascade decided, before it becomes a placement claim.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub class: ClassId,
    pub confidence: f64,
    /// `Machine` when a tier decided, `Flagged` otherwise.
    pub status: Status,
    pub trace: Trace,
}

fn rank(a: &Candidate, b: &Candidate) -> Ordering {
    b.score
        .total_cmp(&a.score)
        .then_with(|| a.class.code.as_bytes().cmp(b.class.code.as_bytes()))
}

fn trace_rank(a: &TraceCandidate, b: &TraceCandidate) -> Ordering {
    b.score
        .total_cmp(&a.score)
        .then_with(|| a.class.code.as_bytes().cmp(b.class.code.as_bytes()))
}

fn margin_of(scores: &[f64]) -> f64 {
    match scores {
        [] => 0.0,
        [_] => 1.0,
        [a, b, ..] => a - b,
    }
}

fn run_state(tier: &str, state: TierRunState) -> TierRun {
    TierRun {
        tier: tier.to_string(),
        state,
        cost: Cost::default(),
    }
}

/// Check, calibrate and merge one tier's output: one candidate per class
/// (max score, evidence combined), ranked. Two identifiers pointing at the
/// same class must not read as a zero margin.
fn normalize(
    tier: &str,
    raw: Vec<Candidate>,
    calibration: Option<&Calibration>,
) -> Result<Vec<Candidate>, Error> {
    let mut merged: BTreeMap<String, Candidate> = BTreeMap::new();
    for mut c in raw {
        if !c.score.is_finite() || !(0.0..=1.0).contains(&c.score) {
            return Err(Error::internal(format!(
                "tier {tier} returned score {} for {}; scores must be finite and in [0, 1]",
                c.score, c.class
            )));
        }
        if let Some(cal) = calibration {
            c.score = cal.apply(tier, c.score);
        }
        match merged.entry(c.class.to_string()) {
            Entry::Vacant(v) => {
                v.insert(c);
            }
            Entry::Occupied(mut o) => {
                let m = o.get_mut();
                m.score = m.score.max(c.score);
                m.evidence.extend(c.evidence);
            }
        }
    }
    let mut out: Vec<Candidate> = merged.into_values().collect();
    out.sort_by(rank);
    Ok(out)
}

fn invalid_target(target: f64) -> Error {
    Error::new(
        ErrorCode::InvalidRequest,
        format!("target confidence {target} must be a number in [0, 1]"),
    )
    .next("shelfmark describe classify")
}

/// Run `profile` over `input`. `tiers` maps tier names to implementations;
/// a name with no implementation is reported `unavailable`, as is a tier
/// that returns `tier_unavailable`. Any other tier error aborts. Without a
/// calibration, raw tier scores are used as confidences.
// Frozen U0 contract: splitting it is a core change, so the size lint is
// waived here rather than refactored during integration.
#[allow(clippy::too_many_lines)]
pub fn run(
    profile: &Profile,
    tiers: &BTreeMap<String, &dyn Tier>,
    index: &dyn CompiledIndex,
    input: &ClassifyInput,
    target: Option<f64>,
    calibration: Option<&Calibration>,
) -> Result<Decision, Error> {
    let target = target.unwrap_or(profile.default_target);
    if !(target.is_finite() && (0.0..=1.0).contains(&target)) {
        return Err(invalid_target(target));
    }
    let mut pool: BTreeMap<String, TraceCandidate> = BTreeMap::new();
    let mut runs = Vec::with_capacity(profile.tiers.len());
    let mut total = Cost::default();
    let mut decided: Option<(String, Candidate, f64)> = None;

    for name in &profile.tiers {
        if decided.is_some() {
            runs.push(run_state(name, TierRunState::Skipped));
            continue;
        }
        let Some(tier) = tiers.get(name) else {
            runs.push(run_state(name, TierRunState::Unavailable));
            continue;
        };
        let mut prior: Vec<Candidate> = pool
            .values()
            .map(|c| Candidate {
                class: c.class.clone(),
                score: c.score,
                evidence: Vec::new(),
            })
            .collect();
        prior.sort_by(rank);
        let out = match tier.run(input, index, &prior) {
            Ok(out) => out,
            Err(e) if e.code == ErrorCode::TierUnavailable => {
                runs.push(run_state(name, TierRunState::Unavailable));
                continue;
            }
            Err(e) => return Err(e),
        };
        let candidates = normalize(name, out.candidates, calibration)?;
        total = total + out.cost;
        runs.push(TierRun {
            tier: name.clone(),
            state: TierRunState::Ran,
            cost: out.cost,
        });

        let scores: Vec<f64> = candidates.iter().map(|c| c.score).collect();
        let tier_margin = margin_of(&scores);
        if let Some(top) = candidates.first() {
            if top.score >= target && tier_margin >= profile.min_margin {
                decided = Some((name.clone(), top.clone(), tier_margin));
            }
        }
        for c in candidates {
            let entry = pool
                .entry(c.class.to_string())
                .or_insert_with(|| TraceCandidate {
                    class: c.class.clone(),
                    score: c.score,
                    per_tier: Vec::new(),
                    evidence: Vec::new(),
                });
            entry.score = entry.score.max(c.score);
            entry.per_tier.push(TierScore {
                tier: name.clone(),
                score: c.score,
            });
            entry.evidence.extend(c.evidence);
        }
    }

    let mut candidates: Vec<TraceCandidate> = pool.into_values().collect();
    candidates.sort_by(trace_rank);

    let (class, confidence, status, deciding_tier, margin, flag_reason) = match decided {
        Some((tier, top, m)) => (top.class, top.score, Status::Machine, Some(tier), m, None),
        None => {
            let top = candidates.first().ok_or_else(|| {
                Error::new(
                    ErrorCode::NotFound,
                    "no tier produced a candidate class for this input",
                )
                .next(format!(
                    "shelfmark scheme search {} <terms>",
                    index.scheme()
                ))
            })?;
            let scores: Vec<f64> = candidates.iter().map(|c| c.score).collect();
            let margin = margin_of(&scores);
            let reason = if top.score >= target {
                format!(
                    "margin {margin:.3} below the profile minimum {:.3}",
                    profile.min_margin
                )
            } else {
                format!("no tier reached target confidence {target:.3}")
            };
            let class = top.class.clone();
            (
                class,
                top.score,
                Status::Flagged,
                None,
                margin,
                Some(reason),
            )
        }
    };

    Ok(Decision {
        class,
        confidence,
        status,
        trace: Trace {
            target,
            candidates,
            margin,
            deciding_tier,
            tiers: runs,
            cost: total,
            flag_reason,
        },
    })
}
