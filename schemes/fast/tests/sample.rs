//! The committed `fast-sample.json` is a valid scheme: it passes the
//! published JSON Schema, round-trips through `SchemeDoc`, passes the
//! scheme compiler's checks, and browses, searches and validates codes the
//! way the full scheme does. SOURCES.md records its hash.

use std::path::PathBuf;

use serde_json::Value;
use shelfmark_core::id::sha256_hex;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

fn here(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn bytes() -> Vec<u8> {
    std::fs::read(here("fast-sample.json")).unwrap()
}

fn json(path: PathBuf) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(&bytes(), "fast-sample.json").unwrap()
}

fn codes(records: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    records.into_iter().map(|r| r.id.code).collect()
}

#[test]
fn validates_against_the_published_schema() {
    let schema = json(here("../../schemas/scheme.schema.json"));
    let v = jsonschema::validator_for(&schema).unwrap();
    let doc = json(here("fast-sample.json"));
    let errors: Vec<String> = v
        .iter_errors(&doc)
        .take(10)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn round_trips_through_the_model_and_names_its_source() {
    let doc = json(here("fast-sample.json"));
    let model: SchemeDoc = serde_json::from_value(doc.clone()).unwrap();
    assert_eq!(model.id().to_string(), "scheme:fast@2024-sample");
    assert_eq!(serde_json::to_value(&model).unwrap(), doc);
    let source = model.source.unwrap();
    assert_eq!(source.license.as_deref(), Some("ODC-By-1.0"));
    assert!(source
        .description
        .contains("OCLC Online Computer Library Center"));
    assert!(source.description.contains("sha256 217826c9"));
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
fn browses_down_and_up_the_hierarchy() {
    let idx = index();
    // Academic libraries is under Libraries, which follows LCSH's broader
    // chain up to "Auxiliary sciences of history", a top-level class.
    assert_eq!(idx.get("fst00997341").unwrap().label, "Libraries");
    assert!(codes(idx.children(Some("fst00997341"))).contains(&"fst00794997".to_string()));
    assert_eq!(idx.get("fst00794997").unwrap().label, "Academic libraries");
    let up = codes(idx.ancestors("fst00794998"));
    assert_eq!(
        up[..4],
        ["fst00794998", "fst00794997", "fst00997341", "fst00896108"]
    );
    let root = up.last().unwrap();
    assert_eq!(
        idx.get(root).unwrap().label,
        "Auxiliary sciences of history"
    );
    assert_eq!(idx.get(root).unwrap().level, 0);
    assert!(codes(idx.children(None)).contains(root));
    // Machine learning sits under a broader heading, not at the top.
    let ml = idx.ancestors("fst01004795");
    assert_eq!(ml[0].label, "Machine learning");
    assert!(ml.len() > 1, "{ml:?}");
    assert!(!codes(idx.children(None)).contains(&"fst01004795".to_string()));
}

#[test]
fn searches_labels_and_vocabulary() {
    let idx = index();
    let hits = idx.search("machine learning", 3);
    assert_eq!(hits[0].class.code, "fst01004795", "{hits:?}");
    // An alternate label finds its class.
    let hits = idx.search("university libraries", 5);
    assert!(
        hits.iter().any(|h| h.class.code == "fst00794997"),
        "{hits:?}"
    );
    // So does the LCSH link, with its LCSH ID.
    let v = &idx.vocab()["fst01004795"];
    assert!(
        v.iter()
            .any(|l| l.source == "lcsh" && l.source_id.as_deref() == Some("sh85079324")),
        "{v:?}"
    );
}

#[test]
fn validates_codes_against_the_grammar() {
    let idx = index();
    assert!(matches!(
        idx.check_code("fst00997341"),
        CodeCheck::Exact { .. }
    ));
    assert!(matches!(idx.check_code("fst99999999"), CodeCheck::Unplaced));
    assert!(matches!(
        idx.check_code("997341"),
        CodeCheck::Invalid { .. }
    ));
    assert!(matches!(
        idx.check_code("fst0997341"),
        CodeCheck::Invalid { .. }
    ));
}

#[test]
fn sources_md_records_the_committed_sample() {
    let notes = std::fs::read_to_string(here("SOURCES.md")).unwrap();
    let hash = format!("`{}`", sha256_hex(&bytes()));
    assert!(
        notes
            .lines()
            .any(|l| l.starts_with("| fast-sample.json ") && l.contains(&hash)),
        "SOURCES.md does not record {hash}; run `fast-scheme build`"
    );
}
