//! The committed `bk.json`: valid, consistent with the lock, and (with the
//! GBV export cached) reproduced byte for byte. The committed-file tests
//! need no network. The rebuild test runs only when `BK_CACHE` names a
//! directory holding the locked export (or one to download it into, with
//! `BK_FETCH=1`); otherwise it records a skip in `$TEST_SKIP_LOG`, and
//! `BK_REQUIRE_SOURCE=1` makes that a failure.

mod common;

use common::{assert_valid, build, committed_lock, here, skip};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};

fn committed() -> String {
    std::fs::read_to_string(here().join("bk.json")).unwrap()
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(committed().as_bytes(), "bk.json").unwrap()
}

#[test]
fn the_committed_scheme_is_valid_and_records_its_source() {
    let text = committed();
    let doc = assert_valid(&text);
    let lock = committed_lock();
    assert_eq!(doc.id().to_string(), format!("scheme:bk@{}", lock.version));
    assert_eq!(doc.classes.len(), 2140);
    let s = doc.source.unwrap();
    assert!(s.description.contains(&lock.source.sha256));
    assert!(s.license.unwrap().contains("CC0"));
}

#[test]
fn has_the_five_top_classes_and_real_branches() {
    let idx = index();
    let top: Vec<String> = idx.children(None).into_iter().map(|c| c.id.code).collect();
    assert_eq!(top, ["0", "1-2", "3-4", "5", "7-8"]);
    for (code, label) in [
        ("0", "Allgemeine Werke und Philosophie"),
        ("01.10", "Bibliographie: Allgemeines"),
        ("42.82", "Amphibia, Reptilia"),
    ] {
        assert_eq!(
            idx.get(code).unwrap_or_else(|| panic!("no {code}")).label,
            label
        );
    }
    let up: Vec<String> = idx
        .ancestors("42.82")
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    assert_eq!(up[0], "42.82");
    assert_eq!(up[1], "42.80");
    assert_eq!(up.last().unwrap(), "3-4");
}

#[test]
fn searches_in_german_and_english_and_checks_real_codes() {
    let idx = index();
    for query in ["Bibliographie", "bibliography general"] {
        let hits = idx.search(query, 10);
        assert!(
            hits.iter().any(|h| h.class.code == "01.10"),
            "{query}: {hits:?}"
        );
    }
    let hits = idx.search("Amphibien", 10);
    assert!(hits.iter().any(|h| h.class.code == "42.82"), "{hits:?}");
    assert!(matches!(idx.check_code("42.82"), CodeCheck::Exact { .. }));
    assert!(matches!(idx.check_code("42.8"), CodeCheck::Invalid { .. }));
}

#[test]
fn rebuilds_byte_for_byte_from_the_locked_export() {
    let Some(cache) = std::env::var_os("BK_CACHE").map(std::path::PathBuf::from) else {
        skip("BK_CACHE is unset, so the GBV export is not available");
        return;
    };
    let lock = committed_lock();
    if !cache.join(&lock.source.file).exists() && std::env::var("BK_FETCH").as_deref() != Ok("1") {
        skip("BK_CACHE has no export file and BK_FETCH is not 1");
        return;
    }
    let src = bk_scheme::source::cached(&cache, &lock).unwrap();
    assert_eq!(committed(), build(&src, &lock), "run `bk-scheme build`");
}
