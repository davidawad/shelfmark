//! AE3 (R10): with a target the lexical tiers already meet, the rerank
//! profile runs no model and reports a lexical tier as deciding.

use crate::common::Bench;

fn tier_states(p: &serde_json::Value) -> Vec<(String, String)> {
    p["trace"]["tiers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            (
                t["tier"].as_str().unwrap().to_string(),
                t["state"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn rerank_profile_stops_at_the_lexical_tier_that_meets_the_target() {
    let b = Bench::new("ae3");
    let out = b.ok(&[
        "classify",
        "--text",
        "Lecture notes on algorithms",
        "--identifier",
        "isbn:9780262033848",
        "--profile",
        "profile:rerank@1",
        "--target",
        "0.9",
    ]);
    let p = &out["placement"];
    assert_eq!(p["profile"], "profile:rerank@1");
    assert_eq!(p["status"], "machine");
    assert_eq!(p["trace"]["deciding_tier"], "identifier");
    assert_eq!(p["trace"]["cost"]["model_calls"], 0);
    let states = tier_states(p);
    let names: Vec<&str> = states.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        ["identifier", "vocab", "rules", "shortlist", "rerank"]
    );
    assert_eq!(states[0].1, "ran");
    for (tier, state) in &states[1..] {
        assert_eq!(state, "skipped", "{tier} should not run");
    }
}

#[test]
fn vocabulary_tier_can_decide_for_the_rerank_profile_too() {
    let b = Bench::new("ae3-vocab");
    let out = b.ok(&[
        "classify",
        "--text",
        "Computer software",
        "--profile",
        "profile:rerank@1",
        "--target",
        "0.5",
    ]);
    let p = &out["placement"];
    assert_eq!(p["trace"]["deciding_tier"], "vocab");
    assert_eq!(p["class"], "class:lcc@2024:QA76.75-76.765");
    assert_eq!(p["trace"]["cost"]["model_calls"], 0);
    let states = tier_states(p);
    assert!(
        states[3..].iter().all(|(_, s)| s == "skipped"),
        "{states:?}"
    );
}
