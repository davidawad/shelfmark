//! The committed `schemes/openalex-topics/openalex-topics.json` is a valid
//! scheme with the OpenAlex topic hierarchy, is tied to the pinned pages
//! in the lock, and answers the browse, search and code lookups the
//! classifier relies on. (A rebuild needs the fetched pages; see the
//! README's `check` command.)

use std::path::PathBuf;

use serde_json::Value;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn bytes() -> Vec<u8> {
    std::fs::read(dir().join("openalex-topics.json")).unwrap()
}

fn json(path: PathBuf) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(&bytes(), "openalex-topics.json").unwrap()
}

fn codes(records: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    records.into_iter().map(|c| c.id.code).collect()
}

#[test]
fn validates_against_the_published_schema() {
    let schema = json(dir().join("../../schemas/scheme.schema.json"));
    let v = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = v
        .iter_errors(&json(dir().join("openalex-topics.json")))
        .take(10)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn round_trips_through_the_model_and_passes_the_compilers_checks() {
    let doc = json(dir().join("openalex-topics.json"));
    let model: SchemeDoc = serde_json::from_value(doc.clone()).unwrap();
    assert!(model
        .id()
        .to_string()
        .starts_with("scheme:openalex-topics@"));
    assert_eq!(serde_json::to_value(&model).unwrap(), doc);
    if let Err(problems) = shelfmark_scheme::validate_bytes(&bytes()) {
        panic!(
            "{} problems: {:?}",
            problems.len(),
            &problems[..problems.len().min(10)]
        );
    }
}

#[test]
fn has_the_documented_shape() {
    let model: SchemeDoc = serde_json::from_slice(&bytes()).unwrap();
    let st = openalex_topics_scheme::build::stats(&model);
    assert_eq!(st.per_level, [4, 26, 252, 4516]);
    assert_eq!(st.classes, 4798);
    assert_eq!(st.depth, 4);
    assert!(st.vocab > 90_000, "{st:?}");
}

#[test]
fn records_license_and_pins_every_page() {
    let doc = json(dir().join("openalex-topics.json"));
    let source = &doc["source"];
    assert!(source["license"].as_str().unwrap().starts_with("CC0 1.0"));
    let desc = source["description"].as_str().unwrap();
    let lock = json(dir().join("sources.lock.json"));
    let pinned: Vec<&Value> = lock["sources"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["role"] != "license")
        .collect();
    assert!(pinned.len() > 25);
    for s in pinned {
        assert!(desc.contains(s["sha256"].as_str().unwrap()), "{s}");
    }
}

#[test]
fn browses_domain_field_subfield_topic() {
    let idx = index();
    assert_eq!(codes(idx.children(None)), ["D1", "D2", "D3", "D4"]);
    assert_eq!(idx.get("D3").unwrap().label, "Physical Sciences");
    assert_eq!(idx.get("F31").unwrap().label, "Physics and Astronomy");
    assert_eq!(
        codes(idx.ancestors("T10346")),
        ["T10346", "S3106", "F31", "D3"]
    );
    assert_eq!(idx.get("T10346").unwrap().level, 3);
    assert!(codes(idx.children(Some("S3106"))).contains(&"T10346".to_string()));
}

#[test]
fn searches_labels_and_keywords() {
    let idx = index();
    let hits = idx.search("magnetic confinement fusion", 3);
    assert_eq!(hits[0].class.code, "T10346");
    // "Doppler reflectometry" is only a keyword of that topic.
    let hits = idx.search("doppler reflectometry", 3);
    assert!(hits.iter().any(|h| h.class.code == "T10346"), "{hits:?}");
    assert!(idx.search("zzzzqqqq", 5).is_empty());
}

#[test]
fn checks_codes_against_the_grammar() {
    let idx = index();
    assert!(matches!(idx.check_code("T10346"), CodeCheck::Exact { .. }));
    for bad in ["t10346", "T", "T10346.1", "X1", "T 1"] {
        assert!(
            matches!(idx.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
}
