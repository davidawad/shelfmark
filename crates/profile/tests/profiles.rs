//! The built-in profiles and profile hash stability.

use shelfmark_core::error::ErrorCode;
use shelfmark_core::tier::TierInfo;
use shelfmark_profile::builtin::{find, lexical, rerank, tier_spec};
use shelfmark_profile::{builtins, profile_hash};

#[test]
fn builtins_are_declared_cheapest_first() {
    let ps = builtins();
    let ids: Vec<String> = ps.iter().map(|p| p.id.to_string()).collect();
    assert_eq!(
        ids,
        [
            "profile:lexical@1",
            "profile:shortlist@1",
            "profile:rerank@1"
        ]
    );
    assert_eq!(ps[0].tiers, ["identifier", "vocab", "rules"]);
    assert_eq!(ps[1].tiers.last().unwrap(), "shortlist");
    assert_eq!(ps[2].tiers[3..], ["shortlist", "rerank"]);
    for w in ps.windows(2) {
        assert!(w[0].expected_memory_mb < w[1].expected_memory_mb);
        assert!(w[0].expected_micros_per_item < w[1].expected_micros_per_item);
    }
}

#[test]
fn profile_costs_sum_their_tiers() {
    for p in builtins() {
        let mem: u32 = p
            .tiers
            .iter()
            .map(|t| tier_spec(t).unwrap().expected_memory_mb)
            .sum();
        assert_eq!(p.expected_memory_mb, mem, "{}", p.id);
    }
    assert!(lexical()
        .tiers
        .iter()
        .all(|t| !tier_spec(t).unwrap().needs_model));
}

#[test]
fn find_by_name_or_id() {
    assert_eq!(find("rerank").unwrap(), rerank());
    assert_eq!(find("profile:lexical@1").unwrap(), lexical());
    let err = find("llm").unwrap_err();
    assert_eq!(err.code, ErrorCode::ProfileNotFound);
    assert!(err.next_command.is_some());
}

fn info(name: &str, version: &str) -> TierInfo {
    TierInfo {
        name: name.into(),
        version: version.into(),
        needs_model: false,
        expected_memory_mb: 1,
        expected_micros_per_item: 1,
    }
}

fn lexical_infos() -> Vec<TierInfo> {
    vec![
        info("identifier", "1"),
        info("vocab", "1"),
        info("rules", "1"),
    ]
}

#[test]
fn profile_hash_is_pinned() {
    // Changing this value stales every stored lexical placement; do it
    // only on purpose.
    assert_eq!(
        profile_hash(&lexical(), &lexical_infos()),
        "s256:48c1405cae031d1f582d65531c1ec33e5dbddf3d50a657d0164691b04d7821fb"
    );
}

#[test]
fn profile_hash_ignores_costs_and_description() {
    let base = profile_hash(&lexical(), &lexical_infos());
    let mut p = lexical();
    p.description = "reworded".into();
    p.expected_memory_mb += 100;
    p.expected_micros_per_item += 100;
    let mut infos = lexical_infos();
    infos[0].expected_memory_mb = 999;
    assert_eq!(profile_hash(&p, &infos), base);
}

#[test]
fn profile_hash_tracks_decision_inputs() {
    let base = profile_hash(&lexical(), &lexical_infos());
    let mut infos = lexical_infos();
    infos[1].version = "2".into();
    assert_ne!(profile_hash(&lexical(), &infos), base);

    let mut p = lexical();
    p.default_target = 0.7;
    assert_ne!(profile_hash(&p, &lexical_infos()), base);

    let mut p = lexical();
    p.tiers.swap(0, 1);
    assert_ne!(profile_hash(&p, &lexical_infos()), base);
}
