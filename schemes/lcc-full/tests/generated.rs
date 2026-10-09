//! The committed `lcc-full.json.gz`: valid, a superset of the bundled
//! outline's codes (so ids agree), and compilable. Needs no network.

use std::collections::BTreeSet;
use std::io::Read;
use std::path::PathBuf;

use shelfmark_core::scheme::SchemeDoc;

fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn gunzip(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(bytes)
        .read_to_end(&mut out)
        .unwrap();
    out
}

fn committed() -> Vec<u8> {
    gunzip(&std::fs::read(here().join("lcc-full.json.gz")).unwrap())
}

#[test]
fn committed_scheme_is_valid_and_complete() {
    let bytes = committed();
    let doc = shelfmark_scheme::validate_bytes(&bytes)
        .unwrap_or_else(|e| panic!("{:?}", &e[..e.len().min(5)]));
    assert_eq!(doc.id().to_string(), "scheme:lcc-full@2024");
    assert!(doc.classes.len() > 400_000, "{}", doc.classes.len());
    assert!(doc.source.as_ref().unwrap().license.as_ref().unwrap().contains("17 U.S.C. 105"));
}

#[test]
fn keeps_every_outline_code() {
    let full: SchemeDoc = serde_json::from_slice(&committed()).unwrap();
    let outline: SchemeDoc =
        serde_json::from_slice(&std::fs::read(here().join("../../data/schemes/lcc/lcc.json")).unwrap())
            .unwrap();
    let have: BTreeSet<&str> = full.classes.iter().map(|c| c.code.as_str()).collect();
    let missing: Vec<&str> = outline
        .classes
        .iter()
        .map(|c| c.code.as_str())
        .filter(|c| !have.contains(c))
        .take(5)
        .collect();
    assert!(missing.is_empty(), "outline codes missing: {missing:?}");
    assert_eq!(full.code_grammar, outline.code_grammar);
}

#[test]
fn spot_checks_deep_entries() {
    let full: SchemeDoc = serde_json::from_slice(&committed()).unwrap();
    let get = |code: &str| full.classes.iter().find(|c| c.code == code);
    let c = get("QA76.73.R3").expect("QA76.73.R3");
    assert_eq!(c.label, "R (Computer program language)");
    assert_eq!(c.parent.as_deref(), Some("QA76.73.A-76.73.Z"));
    let k = get("KF8925.A9").expect("KF8925.A9");
    assert_eq!(k.parent.as_deref(), Some("KF8925.A-8925.Z"));
    assert_eq!(get("Z678.2").unwrap().parent.as_deref(), Some("Z678"));
}

#[test]
fn compiles() {
    let idx = shelfmark_scheme::compile_bytes(&committed(), "lcc-full.json").unwrap();
    let _ = idx;
}
