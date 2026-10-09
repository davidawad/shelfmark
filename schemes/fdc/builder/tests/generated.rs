//! The committed `schemes/fdc/fdc.json` is a valid scheme: it passes the
//! published JSON Schema, round-trips through `SchemeDoc`, passes the
//! scheme compiler's own checks (grammar, parents, range nesting), and
//! browses, checks codes and searches the way the `scheme` verbs serve it.

use std::path::PathBuf;

use serde_json::Value;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn bytes() -> Vec<u8> {
    std::fs::read(root().join("schemes/fdc/fdc.json")).unwrap()
}

fn json(rel: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(rel)).unwrap()).unwrap()
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(&bytes(), "fdc.json").unwrap()
}

fn codes(records: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    records.into_iter().map(|c| c.id.code).collect()
}

#[test]
fn validates_against_the_published_schema() {
    let schema = json("schemas/scheme.schema.json");
    let v = jsonschema::validator_for(&schema).unwrap();
    let doc = json("schemes/fdc/fdc.json");
    let errors: Vec<String> = v
        .iter_errors(&doc)
        .take(10)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn round_trips_through_the_model() {
    let doc = json("schemes/fdc/fdc.json");
    let model: SchemeDoc = serde_json::from_value(doc.clone()).unwrap();
    assert_eq!(model.id().to_string(), "scheme:fdc@0.06");
    assert_eq!(serde_json::to_value(&model).unwrap(), doc);
    let source = model.source.unwrap();
    assert_eq!(
        source.url.as_deref(),
        Some("https://github.com/JohnMarkOckerbloom/fdc")
    );
    assert!(source.license.unwrap().starts_with("CC0 1.0"));
}

#[test]
fn passes_the_scheme_compilers_checks() {
    if let Err(problems) = shelfmark_scheme::validate_bytes(&bytes()) {
        panic!(
            "{} problems, first: {:?}",
            problems.len(),
            &problems[..problems.len().min(10)]
        );
    }
}

#[test]
fn browses_the_ten_main_classes_down_to_the_decimals() {
    let idx = index();
    assert_eq!(idx.len(), 1097);
    assert_eq!(
        codes(idx.children(None)),
        "000 100 200 300 400 500 600 700 800 900"
            .split(' ')
            .collect::<Vec<_>>()
    );
    assert_eq!(idx.get("510").unwrap().label, "Mathematics");
    assert_eq!(
        codes(idx.ancestors("971.23")),
        ["971.23", "971.2", "971", "970", "940-990", "900"]
    );
    // A range groups the numbers inside it, under the tens class.
    assert_eq!(
        codes(idx.ancestors("018")),
        ["018", "017-019", "010", "000"]
    );
    assert_eq!(
        codes(idx.ancestors("975.1")),
        ["975.1", "975", "973-979", "970", "940-990", "900"]
    );
    assert_eq!(idx.get("976").unwrap().label, "Gulf States; Southwest, Old");
    assert_eq!(
        idx.get("865").unwrap().label,
        "Speeches, addresses, etc., Spanish"
    );
}

#[test]
fn checks_codes_against_the_grammar_and_the_spans() {
    let idx = index();
    assert!(matches!(idx.check_code("510"), CodeCheck::Exact { .. }));
    // 013 is unassigned in FDC; it falls inside the 010s.
    match idx.check_code("013") {
        CodeCheck::Within { class } => assert_eq!(class.code, "010"),
        other => panic!("{other:?}"),
    }
    for bad in ["51", "5100", "A51", "510."] {
        assert!(
            matches!(idx.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
}

#[test]
fn searches_labels_and_alternative_terms() {
    let idx = index();
    let hit = |text: &str, code: &str| {
        let hits = idx.search(text, 10);
        assert!(
            hits.iter().any(|h| h.class.code == code),
            "{text}: {:?}",
            hits.iter().map(|h| &h.class.code).collect::<Vec<_>>()
        );
    };
    hit("computer programming", "005");
    // Alternative terms are not in the label.
    hit("history of the book", "002");
    hit("incunabula", "093");
    let vocab = &idx.vocab()["093"];
    assert!(vocab
        .iter()
        .any(|v| v.term == "Incunabula" && v.source == "fdc-lcsh"));
}

#[test]
fn the_lock_and_the_notes_agree() {
    let lock = json("schemes/fdc/sources.lock.json");
    let notes = std::fs::read_to_string(root().join("schemes/fdc/SOURCES.md")).unwrap();
    let sources = lock["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 1);
    let s = &sources[0];
    let url = s["url"].as_str().unwrap();
    assert!(
        url.starts_with("https://raw.githubusercontent.com/JohnMarkOckerbloom/fdc/"),
        "{url}"
    );
    assert!(url.contains(s["commit"].as_str().unwrap()));
    assert!(notes.contains(s["sha256"].as_str().unwrap()));
}

#[test]
fn every_class_has_a_trimmed_label_and_ranges_match_codes() {
    let doc: SchemeDoc = serde_json::from_slice(&bytes()).unwrap();
    for c in &doc.classes {
        assert!(!c.label.trim().is_empty(), "{} has no label", c.code);
        assert_eq!(c.label, c.label.trim(), "{} label has edge spaces", c.code);
        if c.code.contains('-') {
            assert!(c.range.is_some(), "{} is a range without one", c.code);
        }
        for v in &c.vocab {
            assert_ne!(v.term, c.label, "{} repeats its label", c.code);
        }
    }
}
