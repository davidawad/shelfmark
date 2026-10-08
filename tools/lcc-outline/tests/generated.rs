//! The committed `data/schemes/lcc/lcc.json` is a valid scheme: it passes
//! the published JSON Schema, round-trips through `SchemeDoc`, passes the
//! scheme compiler's own checks (grammar, parents, range nesting) and
//! answers the lookups the classifier relies on.

use std::path::PathBuf;

use serde_json::Value;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn bytes() -> Vec<u8> {
    std::fs::read(root().join("data/schemes/lcc/lcc.json")).unwrap()
}

fn json(rel: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(rel)).unwrap()).unwrap()
}

#[test]
fn validates_against_the_published_schema() {
    let schema = json("schemas/scheme.schema.json");
    let v = jsonschema::validator_for(&schema).unwrap();
    let doc = json("data/schemes/lcc/lcc.json");
    let errors: Vec<String> = v
        .iter_errors(&doc)
        .take(10)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn round_trips_through_the_model() {
    let doc = json("data/schemes/lcc/lcc.json");
    let model: SchemeDoc = serde_json::from_value(doc.clone()).unwrap();
    assert_eq!(model.id().to_string(), "scheme:lcc@2024");
    assert_eq!(serde_json::to_value(&model).unwrap(), doc);
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

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(&bytes(), "lcc.json").unwrap()
}

#[test]
fn has_the_21_main_classes_and_every_two_letter_subclass() {
    let idx = index();
    let top: Vec<String> = idx.children(None).into_iter().map(|c| c.id.code).collect();
    assert_eq!(
        top,
        "A B C D E F G H J K L M N P Q R S T U V Z"
            .split(' ')
            .collect::<Vec<_>>()
    );
    assert_eq!(idx.get("Q").unwrap().label, "Science");
    // A sample across classes, from the LC Classification Outline.
    for (code, label) in [
        ("AC", "Collections. Series. Collected works"),
        ("BF", "Psychology"),
        ("DAW", "Central Europe"),
        ("HD", "Industries. Land use. Labor"),
        ("KF", "United States"),
        ("PS", "American literature"),
        ("QA", "Mathematics"),
        ("ZA", "Information resources (General)"),
    ] {
        let rec = idx
            .get(code)
            .unwrap_or_else(|| panic!("no subclass {code}"));
        assert_eq!(rec.label, label, "{code}");
    }
    let subclasses = top
        .iter()
        .flat_map(|c| idx.children(Some(c)))
        .filter(|c| c.id.code.chars().all(|ch| ch.is_ascii_uppercase()))
        .count();
    assert!(subclasses >= 200, "{subclasses} subclasses");
    assert!(idx.len() > 6_000, "{} classes", idx.len());
}

#[test]
fn nests_spans_and_places_deep_codes() {
    let idx = index();
    let ancestors: Vec<String> = idx
        .ancestors("QA76.75-76.765")
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    assert_eq!(
        ancestors,
        [
            "QA76.75-76.765",
            "QA75.5-76.95",
            "QA75-76.95",
            "QA71-90",
            "QA",
            "Q"
        ]
    );
    let rec = idx.get("QA76.75-76.765").unwrap();
    assert_eq!(rec.label, "Computer software");
    let r = rec.range.unwrap();
    assert_eq!((r.start.as_str(), r.end.as_str()), ("QA76.75", "QA76.765"));
    match idx.check_code("QA76.76.C65") {
        CodeCheck::Within { class } => assert_eq!(class.code, "QA76.75-76.765"),
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        idx.check_code("not a code"),
        CodeCheck::Invalid { .. }
    ));
}

#[test]
fn links_lcsh_headings_to_outline_classes() {
    let idx = index();
    let qa = &idx.vocab()["QA"];
    assert!(
        qa.iter().any(|v| v.term == "Mathematics"
            && v.source == "lcsh"
            && v.source_id.as_deref() == Some("sh85082139")),
        "{qa:?}"
    );
    let hits = idx.search("mathematics", 5);
    assert!(hits.iter().any(|h| h.class.code == "QA"), "{hits:?}");
}

#[test]
fn every_class_has_a_label_and_spans_have_both_ends() {
    let doc: SchemeDoc = serde_json::from_slice(&bytes()).unwrap();
    for c in &doc.classes {
        assert!(!c.label.trim().is_empty(), "{} has no label", c.code);
        assert_eq!(c.label, c.label.trim(), "{} label has edge spaces", c.code);
        if c.code.contains('-') {
            assert!(c.range.is_some(), "{} has no range", c.code);
        }
    }
}
