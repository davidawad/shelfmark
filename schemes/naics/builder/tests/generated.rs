//! The committed `schemes/naics/naics.json` is a valid scheme: it passes
//! the published JSON Schema, round-trips through `SchemeDoc`, passes the
//! scheme compiler's own checks (grammar, parents, range nesting), and
//! browses and searches the way the `scheme browse` and `scheme search`
//! verbs serve it.

use std::path::PathBuf;

use serde_json::Value;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn bytes() -> Vec<u8> {
    std::fs::read(root().join("schemes/naics/naics.json")).unwrap()
}

fn json(rel: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(rel)).unwrap()).unwrap()
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(&bytes(), "naics.json").unwrap()
}

fn codes(records: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    records.into_iter().map(|c| c.id.code).collect()
}

#[test]
fn validates_against_the_published_schema() {
    let schema = json("schemas/scheme.schema.json");
    let v = jsonschema::validator_for(&schema).unwrap();
    let doc = json("schemes/naics/naics.json");
    let errors: Vec<String> = v
        .iter_errors(&doc)
        .take(10)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn round_trips_through_the_model() {
    let doc = json("schemes/naics/naics.json");
    let model: SchemeDoc = serde_json::from_value(doc.clone()).unwrap();
    assert_eq!(model.id().to_string(), "scheme:naics@2022");
    assert_eq!(serde_json::to_value(&model).unwrap(), doc);
    let source = model.source.unwrap();
    assert_eq!(source.url.as_deref(), Some("https://www.census.gov/naics/"));
    assert!(source.license.unwrap().contains("17 U.S.C. 105"));
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
fn browses_the_twenty_sectors_and_their_subsectors() {
    let idx = index();
    assert_eq!(idx.len(), 2125);
    assert_eq!(
        codes(idx.children(None)),
        "11 21 22 23 31-33 42 44-45 48-49 51 52 53 54 55 56 61 62 71 72 81 92"
            .split(' ')
            .collect::<Vec<_>>()
    );
    assert_eq!(idx.get("31-33").unwrap().label, "Manufacturing");
    let mfg = codes(idx.children(Some("31-33")));
    assert_eq!(mfg.len(), 21, "{mfg:?}");
    assert_eq!((mfg[0].as_str(), mfg[20].as_str()), ("311", "339"));
    assert_eq!(
        codes(idx.ancestors("541511")),
        ["541511", "54151", "5415", "541", "54"]
    );
    assert_eq!(
        idx.get("541511").unwrap().label,
        "Custom Computer Programming Services"
    );
    assert_eq!(codes(idx.ancestors("445110"))[4], "44-45");
}

#[test]
fn checks_codes_against_the_grammar_and_the_sector_spans() {
    let idx = index();
    assert!(matches!(idx.check_code("513210"), CodeCheck::Exact { .. }));
    match idx.check_code("32") {
        CodeCheck::Within { class } => assert_eq!(class.code, "31-33"),
        other => panic!("{other:?}"),
    }
    for bad in ["5", "5415111", "54-1511", "A54"] {
        assert!(
            matches!(idx.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
}

#[test]
fn searches_titles_and_index_terms() {
    let idx = index();
    let hit = |text: &str, code: &str| {
        let hits = idx.search(text, 10);
        assert!(
            hits.iter().any(|h| h.class.code == code),
            "{text}: {:?}",
            hits.iter().map(|h| &h.class.code).collect::<Vec<_>>()
        );
    };
    hit("software publishers", "513210");
    // Index-only terms: neither word is in the industry's title.
    hit("canola", "111120");
    hit("rapeseed", "111120");
    let canola = &idx.vocab()["111120"];
    assert!(canola.iter().any(
        |v| v.term == "Canola farming, field and seed production" && v.source == "naics-index"
    ));
}

#[test]
fn the_lock_and_the_notes_agree() {
    let lock = json("schemes/naics/sources.lock.json");
    let notes = std::fs::read_to_string(root().join("schemes/naics/SOURCES.md")).unwrap();
    let sources = lock["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 2);
    for s in sources {
        let url = s["url"].as_str().unwrap();
        assert!(
            url.starts_with("https://www.census.gov/naics/2022NAICS/"),
            "{url}"
        );
        assert!(notes.contains(s["sha256"].as_str().unwrap()), "{url}");
    }
}

#[test]
fn every_class_has_a_trimmed_label_and_only_spans_have_ranges() {
    let doc: SchemeDoc = serde_json::from_slice(&bytes()).unwrap();
    for c in &doc.classes {
        assert!(!c.label.trim().is_empty(), "{} has no label", c.code);
        assert_eq!(c.label, c.label.trim(), "{} label has edge spaces", c.code);
        assert_eq!(c.range.is_some(), c.code.contains('-'), "{}", c.code);
        if c.code.len() < 6 {
            assert!(c.vocab.is_empty(), "{} carries index terms", c.code);
        }
    }
}
