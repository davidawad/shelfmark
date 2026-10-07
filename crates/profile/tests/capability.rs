//! Capability detection and recommendation for fixed fake machines.

use std::collections::BTreeSet;

use shelfmark_profile::builtins;
use shelfmark_profile::capability::{assess, detect, Capability};

fn machine(memory_mb: u64, cpus: u32, avx2: bool, neon: bool) -> Capability {
    Capability {
        total_memory_mb: memory_mb,
        logical_cpus: cpus,
        avx2,
        neon,
        os: "linux".into(),
        arch: if neon { "aarch64" } else { "x86_64" }.into(),
    }
}

fn all_tiers() -> BTreeSet<String> {
    ["identifier", "vocab", "rules", "shortlist", "rerank"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn lexical_only() -> BTreeSet<String> {
    ["identifier", "vocab", "rules"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn recommended(cap: &Capability, runnable: &BTreeSet<String>) -> Option<String> {
    assess(cap, &builtins(), runnable)
        .recommended
        .map(|p| p.to_string())
}

#[test]
fn big_simd_machine_gets_rerank() {
    let cap = machine(16_384, 8, true, false);
    assert_eq!(
        recommended(&cap, &all_tiers()).as_deref(),
        Some("profile:rerank@1")
    );
    let cap = machine(16_384, 8, false, true);
    assert_eq!(
        recommended(&cap, &all_tiers()).as_deref(),
        Some("profile:rerank@1")
    );
}

#[test]
fn mid_memory_machine_gets_shortlist() {
    // Budget 1024 MB: shortlist (964) fits, rerank (1864) does not.
    let cap = machine(2_048, 4, true, false);
    let r = assess(&cap, &builtins(), &all_tiers());
    assert_eq!(r.recommended.unwrap().to_string(), "profile:shortlist@1");
    let rerank = &r.profiles[2];
    assert!(!rerank.fits);
    assert!(
        rerank.reasons[0].contains("needs 1864 MB"),
        "{:?}",
        rerank.reasons
    );
}

#[test]
fn no_simd_or_one_cpu_falls_back_to_lexical() {
    let cap = machine(16_384, 8, false, false);
    let r = assess(&cap, &builtins(), &all_tiers());
    assert_eq!(r.recommended.unwrap().to_string(), "profile:lexical@1");
    assert!(r.profiles[1]
        .reasons
        .iter()
        .any(|m| m.contains("AVX2 or NEON")));

    let cap = machine(16_384, 1, true, false);
    assert_eq!(
        recommended(&cap, &all_tiers()).as_deref(),
        Some("profile:lexical@1")
    );
}

#[test]
fn unbuilt_semantic_tiers_fall_back_to_lexical() {
    let cap = machine(16_384, 8, true, false);
    let r = assess(&cap, &builtins(), &lexical_only());
    assert_eq!(r.recommended.unwrap().to_string(), "profile:lexical@1");
    assert!(r.profiles[1].reasons[0].contains("shortlist"));
}

#[test]
fn nothing_fits_a_tiny_machine() {
    let cap = machine(64, 1, false, false);
    let r = assess(&cap, &builtins(), &all_tiers());
    assert_eq!(r.recommended, None);
    assert!(r.profiles.iter().all(|a| !a.fits));
}

#[test]
fn assessment_serializes_with_capability() {
    let cap = machine(8_192, 4, true, false);
    let v = serde_json::to_value(assess(&cap, &builtins(), &all_tiers())).unwrap();
    assert_eq!(v["capability"]["total_memory_mb"], 8192);
    assert_eq!(v["profiles"].as_array().unwrap().len(), 3);
}

#[test]
fn detect_reports_this_machine() {
    let cap = detect();
    assert!(cap.total_memory_mb > 0);
    assert!(cap.logical_cpus >= 1);
    assert_eq!(cap.os, std::env::consts::OS);
    assert!(!(cap.avx2 && cap.neon));
}
