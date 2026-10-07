//! The cascade stop rule (R7, R10, plan PD9).
//!
//! Tiers run in profile order. The first tier whose top candidate reaches
//! the target confidence with at least the profile's minimum margin over
//! its own runner-up decides, and later tiers are skipped. Otherwise the
//! top-ranked candidate across every tier that ran is taken, flagged.
//! Ranking is by score, then class code in byte order, so identical inputs
//! always give an identical decision.

use std::cmp::Ordering;
use std::collections::BTreeMap;

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

/// Run `profile` over `input`. `tiers` maps tier names to implementations;
/// a name with no implementation is reported `unavailable`, as is a tier
/// that returns `tier_unavailable`. Any other tier error aborts.
pub fn run(
    profile: &Profile,
    tiers: &BTreeMap<String, &dyn Tier>,
    index: &dyn CompiledIndex,
    input: &ClassifyInput,
    target: Option<f64>,
) -> Result<Decision, Error> {
    let target = target.unwrap_or(profile.default_target);
    let mut pool: BTreeMap<String, TraceCandidate> = BTreeMap::new();
    let mut runs = Vec::with_capacity(profile.tiers.len());
    let mut total = Cost::default();
    let mut decided: Option<(String, Candidate)> = None;

    for name in &profile.tiers {
        if decided.is_some() {
            runs.push(TierRun {
                tier: name.clone(),
                state: TierRunState::Skipped,
                cost: Cost::default(),
            });
            continue;
        }
        let Some(tier) = tiers.get(name) else {
            runs.push(TierRun {
                tier: name.clone(),
                state: TierRunState::Unavailable,
                cost: Cost::default(),
            });
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
        let mut out = match tier.run(input, index, &prior) {
            Ok(out) => out,
            Err(e) if e.code == ErrorCode::TierUnavailable => {
                runs.push(TierRun {
                    tier: name.clone(),
                    state: TierRunState::Unavailable,
                    cost: Cost::default(),
                });
                continue;
            }
            Err(e) => return Err(e),
        };
        out.candidates.sort_by(rank);
        total = total + out.cost;
        runs.push(TierRun {
            tier: name.clone(),
            state: TierRunState::Ran,
            cost: out.cost,
        });

        let scores: Vec<f64> = out.candidates.iter().map(|c| c.score).collect();
        if let Some(top) = out.candidates.first() {
            if top.score >= target && margin_of(&scores) >= profile.min_margin {
                decided = Some((name.clone(), top.clone()));
            }
        }
        for c in out.candidates {
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
    let scores: Vec<f64> = candidates.iter().map(|c| c.score).collect();
    let margin = margin_of(&scores);

    let (class, confidence, status, deciding_tier, flag_reason) = match decided {
        Some((tier, top)) => (top.class, top.score, Status::Machine, Some(tier), None),
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
            let reason = if top.score >= target {
                format!(
                    "margin {margin:.3} below the profile minimum {:.3}",
                    profile.min_margin
                )
            } else {
                format!("no tier reached target confidence {target:.3}")
            };
            (
                top.class.clone(),
                top.score,
                Status::Flagged,
                None,
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
