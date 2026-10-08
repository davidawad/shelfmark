//! Building a cascade from a profile, and AE3: an unavailable tier is
//! recorded, never fatal.

use shelfmark_core::error::ErrorCode;
use shelfmark_core::placement::{Status, TierRunState};
use shelfmark_core::testkit::{FixedTier, MemIndex};
use shelfmark_core::tier::{ClassifyInput, Tier};
use shelfmark_profile::builtin::{lexical, rerank};
use shelfmark_profile::{build, runnable, Registry};

fn registry(tiers: Vec<FixedTier>) -> Registry {
    tiers
        .into_iter()
        .map(|t| (t.name.clone(), Box::new(t) as Box<dyn Tier>))
        .collect()
}

fn states(d: &shelfmark_core::cascade::Decision) -> Vec<(String, TierRunState)> {
    d.trace
        .tiers
        .iter()
        .map(|r| (r.tier.clone(), r.state))
        .collect()
}

fn lexical_tiers(identifier: &[(&str, f64)], vocab: &[(&str, f64)]) -> Vec<FixedTier> {
    vec![
        FixedTier::new("identifier", identifier),
        FixedTier::new("vocab", vocab),
        FixedTier::new("rules", &[]),
    ]
}

#[test]
fn tiers_come_out_in_profile_order() {
    let reg = registry(lexical_tiers(&[], &[]));
    let c = build(rerank(), &reg).unwrap();
    let names: Vec<&str> = c.tiers().map(|(n, _)| n).collect();
    assert_eq!(
        names,
        ["identifier", "vocab", "rules", "shortlist", "rerank"]
    );
    assert_eq!(c.tier_map().len(), 5);
}

#[test]
fn ae3_lexical_tier_decides_and_no_model_runs() {
    let reg = registry(lexical_tiers(&[("QA76", 0.95)], &[("QA", 0.4)]));
    let d = build(rerank(), &reg)
        .unwrap()
        .run(
            &MemIndex::sample(),
            &ClassifyInput::default(),
            Some(0.9),
            None,
        )
        .unwrap();
    assert_eq!(d.status, Status::Machine);
    assert_eq!(d.class.code, "QA76");
    assert_eq!(d.trace.deciding_tier.as_deref(), Some("identifier"));
    assert_eq!(d.trace.cost.model_calls, 0);
    for (tier, state) in &states(&d)[1..] {
        assert_eq!(*state, TierRunState::Skipped, "{tier}");
    }
}

#[test]
fn ae3_unavailable_semantic_tiers_are_recorded_and_the_result_flagged() {
    let reg = registry(lexical_tiers(&[], &[("H", 0.5), ("K", 0.45)]));
    let c = build(rerank(), &reg).unwrap();
    let d = c
        .run(&MemIndex::sample(), &ClassifyInput::default(), None, None)
        .unwrap();
    assert_eq!(d.status, Status::Flagged);
    assert_eq!(d.class.code, "H");
    assert!(d.trace.flag_reason.is_some());
    let s = states(&d);
    assert_eq!(s[3], ("shortlist".into(), TierRunState::Unavailable));
    assert_eq!(s[4], ("rerank".into(), TierRunState::Unavailable));
    assert_eq!(d.trace.cost.model_calls, 0);
}

#[test]
fn missing_semantic_tier_reports_feature_and_model() {
    let reg = registry(lexical_tiers(&[], &[]));
    let c = build(rerank(), &reg).unwrap();
    let (_, shortlist) = c.tiers().find(|(n, _)| *n == "shortlist").unwrap();
    let err = shortlist
        .run(&ClassifyInput::default(), &MemIndex::sample(), &[])
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::TierUnavailable);
    let details = err.details.unwrap();
    assert_eq!(details["feature"], "semantic-shortlist");
    assert_eq!(details["model"], "BGE-small-en-v1.5 (int8 ONNX)");
    assert_eq!(details["feature_enabled"], false);
    assert!(err.next_command.unwrap().contains("semantic-shortlist"));
}

#[test]
fn registered_tier_reporting_unavailable_is_marked_unavailable() {
    let mut tiers = lexical_tiers(&[], &[("K", 0.6)]);
    tiers[0] = FixedTier::unavailable("identifier");
    let reg = registry(tiers);
    let d = build(lexical(), &reg)
        .unwrap()
        .run(&MemIndex::sample(), &ClassifyInput::default(), None, None)
        .unwrap();
    assert_eq!(states(&d)[0].1, TierRunState::Unavailable);
    assert_eq!(d.status, Status::Flagged);
}

#[test]
fn unknown_tier_name_is_an_error() {
    let reg = registry(vec![FixedTier::new("identifier", &[])]);
    let err = build(lexical(), &reg).err().unwrap();
    assert_eq!(err.code, ErrorCode::TierUnavailable);
    assert_eq!(err.details.unwrap()["tier"], "vocab");
    assert!(err.next_command.is_some());
}

#[test]
fn duplicate_and_misnamed_tiers_are_rejected() {
    let reg = registry(lexical_tiers(&[], &[]));
    let mut p = lexical();
    p.tiers.push("vocab".into());
    assert_eq!(
        build(p, &reg).err().unwrap().code,
        ErrorCode::InvalidRequest
    );

    let mut reg = registry(lexical_tiers(&[], &[]));
    reg.insert("rules".into(), Box::new(FixedTier::new("other", &[])));
    assert_eq!(
        build(lexical(), &reg).err().unwrap().code,
        ErrorCode::InvalidRequest
    );
}

#[test]
fn runnable_excludes_stubs() {
    let reg = registry(lexical_tiers(&[], &[]));
    let names: Vec<String> = runnable(&reg).into_iter().collect();
    assert_eq!(names, ["identifier", "rules", "vocab"]);
}
