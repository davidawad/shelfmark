//! The committed `unesco-thesaurus.json` is valid, and rebuilds byte for
//! byte from the locked export when it is cached.

mod common;

use shelfmark_core::scheme::CompiledIndex;
use unesco_thesaurus::{build, generate, read_lock, LOCK, SCHEME};

#[test]
fn committed_scheme_validates_and_compiles() {
    let json = std::fs::read_to_string(common::crate_dir().join(SCHEME)).unwrap();
    let index = common::validate_and_compile(&json);
    assert_eq!(index.len(), 4595);
    let doc: serde_json::Value = serde_json::from_str(&json).unwrap();
    let desc = doc["source"]["description"].as_str().unwrap();
    let lock = read_lock(&common::crate_dir().join(LOCK)).unwrap();
    assert!(
        desc.contains(&lock.sha256) && desc.contains(&lock.digest()),
        "{desc}"
    );
}

#[test]
fn rebuilds_byte_for_byte_from_the_locked_export() {
    let Ok(cache) = std::env::var("UNESCO_THESAURUS_CACHE") else {
        assert!(
            std::env::var("UNESCO_THESAURUS_REQUIRE_CACHE").is_err(),
            "UNESCO_THESAURUS_CACHE is not set"
        );
        eprintln!("skipped: UNESCO_THESAURUS_CACHE is not set");
        return;
    };
    let lock = read_lock(&common::crate_dir().join(LOCK)).unwrap();
    let (doc, stats) = generate(std::path::Path::new(&cache), &lock).unwrap();
    let have = std::fs::read_to_string(common::crate_dir().join(SCHEME)).unwrap();
    assert_eq!(
        have,
        build::scheme_json(&doc).unwrap(),
        "run `unesco-thesaurus build`"
    );
    assert_eq!(
        (stats.domains, stats.microthesauri, stats.concepts),
        (7, 88, 4500)
    );
}
