//! The builder on a synthetic SKOS file shaped like ACM's: the scheme it
//! writes validates, compiles, browses, searches and checks codes; the
//! output is deterministic; and a malformed or tampered source is refused.

mod common;

use acm_ccs::build::{self, Stats};
use acm_ccs::skos;
use common::{fixture, lock_for, schema_errors};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

fn built() -> (SchemeDoc, String, Stats) {
    let bytes = fixture();
    acm_ccs::build_bytes(&bytes, &lock_for(&bytes)).unwrap()
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(built().1.as_bytes(), "acm-ccs.json").unwrap()
}

fn codes(records: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    records.into_iter().map(|c| c.id.code).collect()
}

#[test]
fn validates_against_the_published_schema_and_round_trips() {
    let (doc, json, _) = built();
    let errors = schema_errors(&json);
    assert!(errors.is_empty(), "{errors:?}");
    let back: SchemeDoc = serde_json::from_str(&json).unwrap();
    assert_eq!(back, doc);
    assert_eq!(doc.id().to_string(), "scheme:acm-ccs@2012");
}

#[test]
fn passes_the_scheme_compilers_checks() {
    let (_, json, _) = built();
    if let Err(problems) = shelfmark_scheme::validate_bytes(json.as_bytes()) {
        panic!("{problems:?}");
    }
}

#[test]
fn classes_are_pre_order_with_siblings_in_code_order() {
    let (doc, _, stats) = built();
    let order: Vec<&str> = doc.classes.iter().map(|c| c.code.as_str()).collect();
    assert_eq!(
        order,
        [
            "10000001",
            "10000001.10000010",
            "10000001.10000010.10000100",
            "10000001.10000010.10000101",
            "10000001.10000011",
            "10000002",
            "10000002.10000020",
            "10000002.10000020.10000101",
        ]
    );
    assert_eq!(
        stats,
        Stats {
            classes: 8,
            top: 2,
            depth: 3,
            poly: 1
        }
    );
}

#[test]
fn a_concept_at_two_paths_notes_the_other() {
    let (doc, _, _) = built();
    let note = |code: &str| {
        doc.classes
            .iter()
            .find(|c| c.code == code)
            .unwrap()
            .note
            .clone()
    };
    assert_eq!(
        note("10000001.10000010.10000101").as_deref(),
        Some("ACM concept 10000101 is also filed at: 10000002.10000020.10000101")
    );
    assert_eq!(
        note("10000002.10000020.10000101").as_deref(),
        Some("ACM concept 10000101 is also filed at: 10000001.10000010.10000101")
    );
    assert_eq!(note("10000001.10000010.10000100"), None);
}

#[test]
fn records_provenance_and_license() {
    let bytes = fixture();
    let lock = lock_for(&bytes);
    let (doc, _, _) = built();
    let source = doc.source.unwrap();
    assert_eq!(source.url.as_deref(), Some(lock.url.as_str()));
    assert!(source.description.contains(&lock.sha256));
    assert!(source.description.contains("https://dl.acm.org/ccs"));
    let license = source.license.unwrap();
    assert!(license.contains("ACM") && license.contains("does not bundle"));
    assert!(doc.classes.iter().all(|c| c.vocab.is_empty()));
}

#[test]
fn the_same_source_builds_the_same_bytes_in_any_file_order() {
    let (_, first, _) = built();
    assert_eq!(built().1, first);
    // Reverse the concepts and top list: the output must not change.
    let bytes = fixture();
    let mut skos = skos::parse(&bytes).unwrap();
    skos.concepts.reverse();
    skos.top.reverse();
    let (doc, _) = build::build(&skos, &lock_for(&bytes)).unwrap();
    assert_eq!(build::scheme_json(&doc).unwrap(), first);
}

#[test]
fn browses_the_hierarchy() {
    let idx = index();
    assert_eq!(codes(idx.children(None)), ["10000001", "10000002"]);
    assert_eq!(
        codes(idx.children(Some("10000001.10000010"))),
        ["10000001.10000010.10000100", "10000001.10000010.10000101"]
    );
    assert_eq!(
        codes(idx.ancestors("10000001.10000010.10000100")),
        [
            "10000001.10000010.10000100",
            "10000001.10000010",
            "10000001"
        ]
    );
    let rec = idx.get("10000001.10000011").unwrap();
    assert_eq!(rec.label, "Widget testing & inspection");
    assert_eq!(rec.level, 1);
}

#[test]
fn searches_labels_and_finds_every_path_of_a_concept() {
    let idx = index();
    let hits = idx.search("sprocket", 5);
    assert_eq!(hits[0].class.code, "10000001.10000010.10000100");
    let gears: Vec<String> = idx
        .search("gear trains", 5)
        .into_iter()
        .filter(|h| h.matched == "Gear trains")
        .map(|h| h.class.code)
        .collect();
    assert_eq!(gears.len(), 2, "{gears:?}");
}

#[test]
fn validates_codes() {
    let idx = index();
    assert!(matches!(
        idx.check_code("10000002.10000020"),
        CodeCheck::Exact { .. }
    ));
    match idx.check_code("10000002.10000020.19999999") {
        CodeCheck::Within { class } => assert_eq!(class.code, "10000002.10000020"),
        other => panic!("{other:?}"),
    }
    assert_eq!(idx.check_code("19999999"), CodeCheck::Unplaced);
    for bad in ["I.2.6", "1000000", "10000002.", "10000002-10000020"] {
        assert!(
            matches!(idx.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
}

#[test]
fn refuses_a_tampered_source() {
    let bytes = fixture();
    let lock = lock_for(&bytes);
    let mut tampered = bytes.clone();
    tampered.extend_from_slice(b"\n");
    let err = acm_ccs::build_bytes(&tampered, &lock).unwrap_err();
    assert!(err.contains("sha256") && err.contains("--relock"), "{err}");
}

#[test]
fn refuses_a_tree_that_disagrees_with_its_paths() {
    let text = String::from_utf8(fixture()).unwrap();
    for (from, to, want) in [
        (
            r#"<skos:broader rdf:resource="10000001" />"#,
            r#"<skos:broader rdf:resource="10000002" />"#,
            "its path says",
        ),
        (
            r#"rdf:about="10000001.10000011""#,
            r#"rdf:about="10000001.1000011""#,
            "8-digit",
        ),
        (
            r#"<skos:hasTopConcept rdf:resource="10000001" />"#,
            "",
            "hasTopConcept",
        ),
        (
            r#"<skos:narrower rdf:resource="10000001.10000011" />"#,
            "",
            "narrower and broader disagree",
        ),
    ] {
        let broken = text.replacen(from, to, 1);
        assert_ne!(broken, text, "{from}");
        let bytes = broken.into_bytes();
        let err = acm_ccs::build_bytes(&bytes, &lock_for(&bytes)).unwrap_err();
        assert!(err.contains(want), "{want}: {err}");
    }
}
