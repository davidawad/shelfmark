//! `enrich-scheme`: headings land on the shelfmark class holding their
//! LCC number, split across holders, never repeating a term.

use serde_json::Value;
use vocab_ingest::enrich::{enrich, Options};
use vocab_ingest::table::Heading;

const MINI: &str = "../../fixtures/schemes/mini-lcc.json";

fn heading(id: &str, label: &str, lcc: &[&str]) -> Heading {
    Heading {
        id: id.into(),
        vocab: "lcsh".into(),
        kind: "Topic".into(),
        pref_label: label.into(),
        lcc: lcc.iter().map(|s| s.to_string()).collect(),
        ..Heading::default()
    }
}

fn vocab_of<'a>(doc: &'a Value, code: &str) -> Vec<&'a Value> {
    doc["classes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == code)
        .and_then(|c| c["vocab"].as_array())
        .map(|v| v.iter().collect())
        .unwrap_or_default()
}

#[test]
fn headings_join_the_class_that_holds_their_number() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(MINI);
    let bytes = std::fs::read(&path).unwrap();
    let index = shelfmark_scheme::compile_bytes(&bytes, "mini").unwrap();
    let mut doc: Value = serde_json::from_slice(&bytes).unwrap();
    let mut deprecated = heading("sh3", "Old heading", &["QA76.5"]);
    deprecated.deprecated = true;
    let headings = vec![
        heading("sh1", "Compilers (Computer programs)", &["QA76.76.C65"]),
        heading("sh2", "Programming, Functional", &["QA76.62", "K100"]),
        deprecated,
        heading("sh4", "Psychology of learning", &["BF318"]),
    ];
    let stats = enrich(&mut doc, &headings, &index, &Options::default()).unwrap();
    assert_eq!(stats.headings_with_lcc, 3);
    assert_eq!(stats.headings_placed, 2);
    // Already a QA76 term: kept once. Two holders: half weight on each.
    let qa76 = vocab_of(&doc, "QA76");
    let terms: Vec<&str> = qa76.iter().map(|v| v["term"].as_str().unwrap()).collect();
    assert_eq!(
        terms,
        [
            "Computer programs",
            "Compilers (Computer programs)",
            "Programming, Functional"
        ]
    );
    assert_eq!(qa76[2]["weight"], 0.5);
    assert_eq!(vocab_of(&doc, "K")[0]["source_id"], "sh2");
    assert_eq!(stats.terms_added, 2);
}

#[test]
fn headings_without_a_number_borrow_from_the_nearest_broader_one() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(MINI);
    let bytes = std::fs::read(&path).unwrap();
    let index = shelfmark_scheme::compile_bytes(&bytes, "mini").unwrap();
    let mut child = heading("sh2", "Lambda calculus in teaching", &[]);
    child.broader = vec!["sh1".into()];
    let mut grandchild = heading("sh3", "Typed lambda calculus in teaching", &[]);
    grandchild.broader = vec!["sh2".into()];
    let orphan = heading("sh4", "Nothing above", &[]);
    let headings = vec![
        heading("sh1", "Programming, Functional", &["QA76.62"]),
        child,
        grandchild,
        orphan,
    ];
    let mut off: Value = serde_json::from_slice(&bytes).unwrap();
    let stats = enrich(&mut off, &headings, &index, &Options::default()).unwrap();
    assert_eq!((stats.headings_active, stats.headings_inherited), (4, 0));
    let mut doc: Value = serde_json::from_slice(&bytes).unwrap();
    let opts = Options {
        inherit: 1,
        ..Options::default()
    };
    let stats = enrich(&mut doc, &headings, &index, &opts).unwrap();
    // One level: the child borrows at half weight, the grandchild is out of reach.
    assert_eq!(stats.headings_inherited, 1);
    let qa76 = vocab_of(&doc, "QA76");
    let child = qa76.iter().find(|v| v["source_id"] == "sh2").unwrap();
    assert_eq!(child["weight"], 0.5);
    assert!(qa76.iter().all(|v| v["source_id"] != "sh3"));
    let mut doc: Value = serde_json::from_slice(&bytes).unwrap();
    let opts = Options {
        inherit: 2,
        ..Options::default()
    };
    let stats = enrich(&mut doc, &headings, &index, &opts).unwrap();
    // sh2 has no number of its own, so sh3 finds sh1 two levels up, at a quarter.
    assert_eq!(stats.headings_inherited, 2);
    let qa76 = vocab_of(&doc, "QA76");
    let grandchild = qa76.iter().find(|v| v["source_id"] == "sh3").unwrap();
    assert_eq!(grandchild["weight"], 0.25);
}

#[test]
fn inheritance_skips_deprecated_and_missing_broader_headings_and_stops_on_cycles() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(MINI);
    let bytes = std::fs::read(&path).unwrap();
    let index = shelfmark_scheme::compile_bytes(&bytes, "mini").unwrap();
    let mut old = heading("sh1", "Old functional programming", &["QA76.62"]);
    old.deprecated = true;
    let mut via_old = heading("sh2", "Child of a deprecated heading", &[]);
    via_old.broader = vec!["sh1".into()];
    let mut missing = heading("sh3", "Child of an unknown heading", &[]);
    missing.broader = vec!["sh999".into()];
    let mut a = heading("sh4", "Cycle A", &[]);
    a.broader = vec!["sh5".into()];
    let mut b = heading("sh5", "Cycle B", &[]);
    b.broader = vec!["sh4".into()];
    let headings = vec![old, via_old, missing, a, b];
    let mut doc: Value = serde_json::from_slice(&bytes).unwrap();
    let opts = Options {
        inherit: 10,
        ..Options::default()
    };
    let stats = enrich(&mut doc, &headings, &index, &opts).unwrap();
    assert_eq!(stats.headings_active, 4);
    assert_eq!(stats.headings_inherited, 0);
    assert_eq!(stats.terms_added, 0);
}
