//! The cascade stop rule (R7, R10, plan PD9; AE2 and AE3 shapes).

use std::collections::BTreeMap;

use shelfmark_core::cascade::{self, Decision};
use shelfmark_core::placement::{Status, TierRunState};
use shelfmark_core::testkit::{FixedTier, MemIndex};
use shelfmark_core::tier::{ClassifyInput, Profile, Tier};
use shelfmark_core::{ErrorCode, ProfileId};

fn profile(tiers: &[&str]) -> Profile {
    Profile {
        id: ProfileId::new("test", "1"),
        description: "test".into(),
        tiers: tiers.iter().map(|t| (*t).into()).collect(),
        default_target: 0.8,
        min_margin: 0.1,
        expected_memory_mb: 0,
        expected_micros_per_item: 0,
    }
}

fn run(
    p: &Profile,
    tiers: &[&FixedTier],
    target: Option<f64>,
) -> Result<Decision, shelfmark_core::Error> {
    let map: BTreeMap<String, &dyn Tier> = tiers
        .iter()
        .map(|t| (t.name.clone(), *t as &dyn Tier))
        .collect();
    cascade::run(
        p,
        &map,
        &MemIndex::sample(),
        &ClassifyInput::default(),
        target,
    )
}

#[test]
fn ae3_first_tier_meeting_target_decides_and_models_never_run() {
    let lexical = FixedTier::new("vocab", &[("QA76", 0.9), ("QA", 0.3)]);
    let mut rerank = FixedTier::new("rerank", &[("H", 0.99)]);
    rerank.model_calls = 1;
    let d = run(
        &profile(&["vocab", "rerank"]),
        &[&lexical, &rerank],
        Some(0.85),
    )
    .unwrap();
    assert_eq!(d.class.code, "QA76");
    assert_eq!(d.status, Status::Machine);
    assert_eq!(d.trace.deciding_tier.as_deref(), Some("vocab"));
    assert_eq!(d.trace.tiers[1].state, TierRunState::Skipped);
    assert_eq!(d.trace.cost.model_calls, 0);
}

#[test]
fn ae2_close_margin_takes_top_class_flagged_with_candidates() {
    let lexical = FixedTier::new("vocab", &[("K", 0.52), ("H", 0.55)]);
    let d = run(&profile(&["vocab"]), &[&lexical], None).unwrap();
    assert_eq!(d.class.code, "H");
    assert_eq!(d.status, Status::Flagged);
    assert_eq!(d.trace.deciding_tier, None);
    let codes: Vec<&str> = d
        .trace
        .candidates
        .iter()
        .map(|c| c.class.code.as_str())
        .collect();
    assert_eq!(codes, ["H", "K"]);
    assert!((d.trace.margin - 0.03).abs() < 1e-9);
    assert!(d.trace.flag_reason.unwrap().contains("target"));
}

#[test]
fn reaching_target_without_margin_falls_through_then_flags() {
    let lexical = FixedTier::new("vocab", &[("H", 0.90), ("K", 0.85)]);
    let d = run(&profile(&["vocab"]), &[&lexical], None).unwrap();
    assert_eq!((d.class.code.as_str(), d.status), ("H", Status::Flagged));
    assert!(d.trace.flag_reason.unwrap().contains("margin"));
}

#[test]
fn later_tier_can_decide_and_trace_keeps_every_tier_score() {
    let lexical = FixedTier::new("vocab", &[("H", 0.5), ("K", 0.45)]);
    let rerank = FixedTier::new("rerank", &[("K", 0.95), ("H", 0.2)]);
    let d = run(&profile(&["vocab", "rerank"]), &[&lexical, &rerank], None).unwrap();
    assert_eq!(d.class.code, "K");
    assert_eq!(d.trace.deciding_tier.as_deref(), Some("rerank"));
    let k = &d.trace.candidates[0];
    assert_eq!(k.per_tier.len(), 2);
    assert_eq!(k.score, 0.95);
}

#[test]
fn ties_break_by_code_so_runs_are_identical() {
    let a = FixedTier::new("vocab", &[("K", 0.5), ("H", 0.5)]);
    let first = run(&profile(&["vocab"]), &[&a], None).unwrap();
    for _ in 0..5 {
        assert_eq!(run(&profile(&["vocab"]), &[&a], None).unwrap(), first);
    }
    assert_eq!(first.class.code, "H");
}

#[test]
fn missing_or_unavailable_tiers_are_reported_not_fatal() {
    let lexical = FixedTier::new("vocab", &[("H", 0.4)]);
    let off = FixedTier::unavailable("shortlist");
    let d = run(
        &profile(&["vocab", "shortlist", "rerank"]),
        &[&lexical, &off],
        None,
    )
    .unwrap();
    let states: Vec<_> = d.trace.tiers.iter().map(|t| t.state).collect();
    assert_eq!(
        states,
        [
            TierRunState::Ran,
            TierRunState::Unavailable,
            TierRunState::Unavailable
        ]
    );
    assert_eq!(d.status, Status::Flagged);
}

#[test]
fn no_candidates_is_not_found_with_a_next_command() {
    let empty = FixedTier::new("vocab", &[]);
    let e = run(&profile(&["vocab"]), &[&empty], None).unwrap_err();
    assert_eq!(e.code, ErrorCode::NotFound);
    assert!(e
        .next_command
        .unwrap()
        .starts_with("shelfmark scheme search scheme:test@1"));
}
