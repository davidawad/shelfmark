//! `docs/schemes/lcc-lcsh-sources.md` lists every file the LCC and LCSH
//! locks pin, with its URL and sha256, and every committed id.loc.gov
//! record still has its locked hash.

use std::path::{Path, PathBuf};

use serde_json::Value;
use shelfmark_core::id::sha256_hex;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn lock(rel: &str) -> Vec<Value> {
    let text = std::fs::read_to_string(root().join(rel)).unwrap();
    let doc: Value = serde_json::from_str(&text).unwrap();
    doc["sources"].as_array().unwrap().clone()
}

#[test]
fn every_locked_source_is_documented() {
    let doc = std::fs::read_to_string(root().join("docs/schemes/lcc-lcsh-sources.md")).unwrap();
    let mut missing = Vec::new();
    for rel in [
        "data/schemes/lcc/sources.lock.json",
        "schemes/lcc-full/sources.lock.json",
    ] {
        for s in lock(rel) {
            let (url, sha) = (s["url"].as_str().unwrap(), s["sha256"].as_str().unwrap());
            let row = doc
                .lines()
                .find(|l| l.contains(&format!("<{url}>")))
                .unwrap_or_default();
            if !row.contains(sha) {
                missing.push(format!("{rel}: {url} {sha}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "not in the sources doc:\n{}",
        missing.join("\n")
    );
}

#[test]
fn committed_records_match_the_lock() {
    let dir = root().join("data/schemes/lcc/sources");
    for s in lock("data/schemes/lcc/sources.lock.json")
        .iter()
        .filter(|s| s["role"] == "id_record")
    {
        let file = s["file"].as_str().unwrap();
        let bytes = std::fs::read(dir.join(file)).unwrap();
        assert_eq!(sha256_hex(&bytes), s["sha256"].as_str().unwrap(), "{file}");
    }
}
