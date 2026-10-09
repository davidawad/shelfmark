//! Shared by the fixture and source tests: paths, the committed lock, and
//! the checks every built BK scheme must pass.

#![allow(dead_code)]

use std::path::PathBuf;

use bk_scheme::source::Lock;
use serde_json::Value;
use shelfmark_core::scheme::SchemeDoc;

pub fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn committed_lock() -> Lock {
    Lock::read(&here().join("sources.lock.json")).unwrap()
}

/// `bk.json` text for `ndjson` under `lock`.
pub fn build(ndjson: &[u8], lock: &Lock) -> String {
    bk_scheme::scheme_json(&bk_scheme::scheme(ndjson, lock).unwrap()).unwrap()
}

/// The text passes the published JSON Schema, round-trips through
/// `SchemeDoc` unchanged, and passes the scheme compiler's checks.
pub fn assert_valid(text: &str) -> SchemeDoc {
    let schema: Value = serde_json::from_str(
        &std::fs::read_to_string(here().join("../../schemas/scheme.schema.json")).unwrap(),
    )
    .unwrap();
    let v = jsonschema::validator_for(&schema).unwrap();
    let doc: Value = serde_json::from_str(text).unwrap();
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

/// Record a skip in `$TEST_SKIP_LOG` (cargo hides a passing test's
/// output), or fail when `BK_REQUIRE_SOURCE=1`.
pub fn skip(why: &str) {
    assert!(
        std::env::var("BK_REQUIRE_SOURCE").as_deref() != Ok("1"),
        "{why}, and BK_REQUIRE_SOURCE=1 makes that a failure"
    );
    let test = std::thread::current().name().unwrap_or("?").to_string();
    let line = format!("SKIP: {why} [{test}]\n");
    eprint!("{line}");
    if let Some(path) = std::env::var_os("TEST_SKIP_LOG") {
        use std::io::Write;
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut f| f.write_all(line.as_bytes()));
    }
}
