//! Paths and loaders for the committed benchmark data the tests read.

#![allow(dead_code)]

use learned_scheme::graph::{load_similarity, read_maybe_gz};
use shelfmark_bench::{CorpusRow, SimilarityGraph};
use shelfmark_core::scheme::SchemeDoc;
use std::path::PathBuf;

pub fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn root() -> PathBuf {
    here().join("../..")
}

pub fn rows(name: &str) -> Vec<CorpusRow> {
    std::fs::read_to_string(root().join("bench/corpus").join(name))
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

pub fn graph_path() -> PathBuf {
    root().join("bench/results/2026-10-10/graphs/corpus-k20.json.gz")
}

pub fn graph() -> SimilarityGraph {
    load_similarity(&graph_path()).unwrap()
}

pub fn graph_sha() -> String {
    shelfmark_bench::sha256_hex(&read_maybe_gz(&graph_path()).unwrap())
}

pub fn committed_text() -> String {
    std::fs::read_to_string(here().join("learned.json")).unwrap()
}

pub fn committed() -> SchemeDoc {
    serde_json::from_str(&committed_text()).unwrap()
}

/// Passes the published JSON Schema, round-trips through `SchemeDoc`, and
/// passes the scheme compiler's checks.
pub fn assert_valid(text: &str) -> SchemeDoc {
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("schemas/scheme.schema.json")).unwrap(),
    )
    .unwrap();
    let v = jsonschema::validator_for(&schema).unwrap();
    let doc: serde_json::Value = serde_json::from_str(text).unwrap();
    let errors: Vec<String> = v
        .iter_errors(&doc)
        .take(10)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    let model: SchemeDoc = serde_json::from_value(doc.clone()).unwrap();
    assert_eq!(serde_json::to_value(&model).unwrap(), doc);
    if let Err(p) = shelfmark_scheme::validate_bytes(text.as_bytes()) {
        panic!("{} problems, first: {:?}", p.len(), &p[..p.len().min(10)]);
    }
    model
}
