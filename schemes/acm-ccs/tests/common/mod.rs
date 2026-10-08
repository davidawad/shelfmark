//! Shared test helpers: the synthetic fixture, its lock, the published
//! schema, and skips logged to `TEST_SKIP_LOG`.

#![allow(dead_code)]

use std::io::Write;
use std::path::PathBuf;

use acm_ccs::sources::Lock;
use serde_json::Value;
use shelfmark_core::id::sha256_hex;

pub fn fixture() -> Vec<u8> {
    std::fs::read(acm_ccs::home().join("tests/fixtures/mini-ccs.xml")).unwrap()
}

/// The committed lock, re-pointed at `bytes`.
pub fn lock_for(bytes: &[u8]) -> Lock {
    Lock {
        bytes: bytes.len() as u64,
        sha256: sha256_hex(bytes),
        ..Lock::read(&acm_ccs::lock_path()).unwrap()
    }
}

pub fn schema() -> Value {
    let path = acm_ccs::home().join("../../schemas/scheme.schema.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Validate scheme JSON against the published schema; return the errors.
pub fn schema_errors(json: &str) -> Vec<String> {
    let schema = schema();
    let v = jsonschema::validator_for(&schema).unwrap();
    let doc: Value = serde_json::from_str(json).unwrap();
    v.iter_errors(&doc)
        .take(10)
        .map(|e| e.to_string())
        .collect()
}

/// Record a skipped test in `TEST_SKIP_LOG` (one line each). With
/// `ACM_CCS_REQUIRE=1` a skip fails instead.
pub fn skip(test: &str, reason: &str) {
    if std::env::var("ACM_CCS_REQUIRE").as_deref() == Ok("1") {
        panic!("{test}: {reason} (ACM_CCS_REQUIRE=1)");
    }
    eprintln!("SKIP {test}: {reason}");
    if let Ok(path) = std::env::var("TEST_SKIP_LOG") {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(PathBuf::from(path))
            .unwrap();
        // One write per line, so parallel tests do not interleave.
        f.write_all(format!("acm-ccs {test}: {reason}\n").as_bytes())
            .unwrap();
    }
}
