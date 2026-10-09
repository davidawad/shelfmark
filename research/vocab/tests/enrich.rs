//! `enrich-scheme`: headings land on the shelfmark class holding their
//! LCC number, split across holders, never repeating a term.

use serde_json::Value;
use vocab_ingest::enrich::enrich;
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
    let stats = enrich(&mut doc, &headings, &index, false).unwrap();
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
