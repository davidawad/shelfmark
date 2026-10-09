//! The builder over a small synthetic export in UNESCO's Turtle format:
//! the scheme validates and compiles, the hierarchy and the
//! multiple-broader rule hold, and the failure cases fail.

mod common;

use shelfmark_core::scheme::CompiledIndex;
use unesco_thesaurus::build;
use unesco_thesaurus::sources::Lock;

fn lock() -> Lock {
    Lock {
        file: "mini.ttl".into(),
        url: "https://example.org/mini.ttl".into(),
        bytes: 0,
        sha256: "0".repeat(64),
        fetched: "2026-10-09".into(),
    }
}

fn text() -> String {
    std::fs::read_to_string(common::crate_dir().join("tests/fixtures/mini.ttl")).unwrap()
}

fn built(t: &str) -> (String, build::Stats) {
    let (doc, stats) = build::build(&lock(), t).unwrap();
    (build::scheme_json(&doc).unwrap(), stats)
}

fn fails(t: &str) -> String {
    build::build(&lock(), t).unwrap_err()
}

fn codes(records: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    records.into_iter().map(|c| c.id.code).collect()
}

#[test]
fn validates_compiles_and_counts() {
    let (json, stats) = built(&text());
    let index = common::validate_and_compile(&json);
    assert_eq!(
        index.scheme().to_string(),
        "scheme:unesco-thesaurus@2026-08-31"
    );
    assert_eq!(index.len(), 8);
    assert_eq!(
        (stats.domains, stats.microthesauri, stats.concepts),
        (1, 2, 5)
    );
    assert_eq!(stats.multi_broader, 1);
    assert_eq!(stats.demoted_links, 2);
    assert_eq!(stats.cross_only, 1);
    assert_eq!(stats.depth, 4);
    let doc: serde_json::Value = serde_json::from_str(&json).unwrap();
    let license = doc["source"]["license"].as_str().unwrap();
    assert!(license.contains("CC BY-SA 3.0 IGO"));
    assert_eq!(built(&text()).0, json, "deterministic");
}

#[test]
fn hierarchy_follows_domain_microthesaurus_then_broader() {
    let index = common::validate_and_compile(&built(&text()).0);
    // Tutoring has broaders c2 and c1; the lowest id in its microthesaurus wins.
    assert_eq!(codes(index.ancestors("c3")), ["c3", "c1", "1.05", "1"]);
    // Classroom's only broader is in another microthesaurus: it hangs under its own.
    assert_eq!(codes(index.ancestors("c4")), ["c4", "1.05", "1"]);
    assert_eq!(codes(index.ancestors("c2")), ["c2", "c1", "1.05", "1"]);
    let mut kids = codes(index.children(Some("c1")));
    kids.sort();
    assert_eq!(kids, ["c2", "c3"]);
}

#[test]
fn keeps_labels_other_broaders_and_notes() {
    let (json, _) = built(&text());
    let doc: serde_json::Value = serde_json::from_str(&json).unwrap();
    let get = |code: &str| {
        doc["classes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["code"] == code)
            .unwrap()
            .clone()
    };
    let c1 = get("c1");
    assert_eq!(c1["label"], "Teaching");
    assert_eq!(c1["note"], "Imparting \"knowledge\".");
    let terms: Vec<(&str, &str)> = c1["vocab"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| (v["term"].as_str().unwrap(), v["source"].as_str().unwrap()))
        .collect();
    assert_eq!(
        terms,
        [
            ("Pupil teaching", "unesco-alt"),
            ("Teachin", "unesco-hidden")
        ]
    );
    let c3 = get("c3");
    assert_eq!(c3["parent"], "c1");
    assert_eq!(c3["vocab"][0]["term"], "Lecturing");
    assert_eq!(c3["vocab"][0]["source"], "unesco-broader");
    assert_eq!(c3["vocab"][0]["source_id"], "concept2");
    assert_eq!(c3["note"], "Also narrower than: Lecturing.");
    let c4 = get("c4");
    assert_eq!(c4["parent"], "1.05");
    assert_eq!(c4["vocab"][0]["term"], "Buildings");
}

#[test]
fn searches_labels_and_alt_labels() {
    let index = common::validate_and_compile(&built(&text()).0);
    assert_eq!(index.search("tutoring", 3)[0].class.code, "c3");
    assert_eq!(index.search("pupil teaching", 3)[0].class.code, "c1");
}

#[test]
fn rejects_bad_exports() {
    let t = text();
    let err = fails(&t.replace(
        "skos:member  :concept9;",
        "skos:member  :concept9 , :concept1;",
    ));
    assert!(err.contains("is in microthesauri"), "{err}");
    let err = fails(&t.replace(
        "skos:member  :concept3 , :concept1 , :concept2 , :concept4;",
        "skos:member  :concept3 , :concept1 , :concept2;",
    ));
    assert!(err.contains("is in no microthesaurus"), "{err}");
    let err = fails(&t.replace("skos:broader  :concept9 .", "skos:broader  :concept77 ."));
    assert!(err.contains("not a concept"), "{err}");
    let cyclic = t.replace(
        ":concept1  rdf:type  skos:Concept;",
        ":concept1  rdf:type  skos:Concept;\n        skos:broader  :concept2;",
    );
    assert!(fails(&cyclic).contains("cycle"));
    let err = fails(&t.replace("\"2026-08-31\"^^xsd:date", "\"soon\""));
    assert!(err.contains("not a date"), "{err}");
}
