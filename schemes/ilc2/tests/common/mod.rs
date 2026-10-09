//! Checks every built ILC2 scheme must pass, whatever the pages.

use std::path::PathBuf;

use serde_json::Value;
use shelfmark_core::scheme::SchemeDoc;
use shelfmark_scheme::SchemeIndex;

pub fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn published_schema() -> Value {
    let path = crate_dir().join("../../schemas/scheme.schema.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The scheme JSON passes the published JSON Schema, round-trips through
/// `SchemeDoc` and the scheme compiler's own checks, and compiles.
pub fn validate_and_compile(json: &str) -> SchemeIndex {
    let value: Value = serde_json::from_str(json).unwrap();
    let schema = published_schema();
    let v = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = v
        .iter_errors(&value)
        .take(10)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    let model: SchemeDoc = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&model).unwrap(), value);
    if let Err(problems) = shelfmark_scheme::validate_bytes(json.as_bytes()) {
        panic!(
            "{} problems, first: {:?}",
            problems.len(),
            &problems[..problems.len().min(10)]
        );
    }
    shelfmark_scheme::compile_bytes(json.as_bytes(), "ilc2.json").unwrap()
}
