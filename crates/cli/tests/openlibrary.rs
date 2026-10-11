//! The `openlibrary` profile through the service. Nothing here reaches
//! the network: without feature `openlibrary` the tier is absent, and
//! with it the store's response cache is seeded first.

mod common;

use common::{call, data, temp_dir};
use serde_json::{json, Value};
use shelfmark_core::Verb;

const SICP: &str = "Structure and Interpretation of Computer Programs";

fn store() -> std::path::PathBuf {
    let store = temp_dir("openlibrary").join("store");
    std::fs::create_dir_all(&store).unwrap();
    store
}

fn tier_states(p: &Value) -> Vec<(String, String)> {
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
fn the_default_profile_never_runs_openlibrary() {
    let s = shelfmark_cli::wiring::open(&store());
    let out = data(call(
        &s,
        Verb::Classify,
        json!({"title": SICP, "authors": ["Harold Abelson"]}),
    ));
    let p = &out["placement"];
    assert_ne!(p["profile"], "profile:openlibrary@2");
    assert!(tier_states(p).iter().all(|(t, _)| t != "openlibrary"));
}

#[test]
fn authors_are_part_of_a_title_only_work() {
    let s = shelfmark_cli::wiring::open(&store());
    let work = |args: Value| {
        data(call(&s, Verb::Classify, args))["placement"]["work"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let a = work(json!({"title": "Dune", "authors": ["Frank Herbert"]}));
    let b = work(json!({"title": "Dune", "authors": ["Jane Elsewhere"]}));
    let c = work(json!({"title": "Dune"}));
    assert_ne!(a, b);
    assert_ne!(a, c);
}

#[cfg(not(feature = "openlibrary"))]
#[test]
fn without_the_feature_the_tier_is_unavailable() {
    let s = shelfmark_cli::wiring::open(&store());
    let out = data(call(
        &s,
        Verb::Classify,
        json!({"title": SICP, "authors": ["Harold Abelson"], "profile": "profile:openlibrary@2"}),
    ));
    let states = tier_states(&out["placement"]);
    assert_eq!(states[1], ("openlibrary".into(), "unavailable".into()));
    assert_eq!(states[2].1, "ran", "the lexical tiers still run");
}

#[cfg(feature = "openlibrary")]
#[test]
fn a_cached_search_decides_with_provenance() {
    use shelfmark_tier_openlibrary::{Client, DiskCache, FixtureTransport, Query};

    let store = store();
    let query = Query {
        title: SICP.into(),
        authors: vec!["Harold Abelson".into()],
    };
    let url = Client::new(Box::new(FixtureTransport::new()), None).url(&query);
    let page = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tier-openlibrary/tests/fixtures/sicp.json"
    ))
    .unwrap();
    DiskCache::new(store.join("cache").join("openlibrary")).put(&url, &page);

    let s = shelfmark_cli::wiring::open(&store);
    let out = data(call(
        &s,
        Verb::Classify,
        json!({"title": SICP, "authors": ["Harold Abelson"], "profile": "profile:openlibrary@2"}),
    ));
    let p = &out["placement"];
    assert_eq!(p["class"], "class:lcc@2024:QA75.5-76.95");
    assert_eq!(p["status"], "machine");
    assert_eq!(p["trace"]["deciding_tier"], "openlibrary");
    let ev = &p["trace"]["candidates"][0]["evidence"][0];
    assert_eq!(ev["kind"], "catalog");
    assert_eq!(ev["record"], "/works/OL3267304W");
    assert_eq!(
        tier_states(p)[2..],
        [
            ("vocab".to_string(), "skipped".to_string()),
            ("rules".to_string(), "skipped".to_string())
        ]
    );
}
