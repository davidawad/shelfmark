//! The builder on a synthetic source in the AEA's XML shape: the output
//! is a valid scheme, deterministic, browsable and searchable, and the
//! lock guards it. Needs no network and no AEA data.

mod common;

use common::{assert_valid, build, committed_lock, here};
use jel_scheme::source::Lock;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};

fn xml() -> Vec<u8> {
    std::fs::read(here().join("fixtures/classificationTree.xml")).unwrap()
}

/// The committed lock re-pinned to the fixture.
fn lock() -> Lock {
    committed_lock().relocked(&xml(), "2026-10-08".into())
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(build(&xml(), &lock()).as_bytes(), "jel.json").unwrap()
}

#[test]
fn builds_a_valid_scheme_with_provenance() {
    let doc = assert_valid(&build(&xml(), &lock()));
    assert_eq!(doc.id().to_string(), "scheme:jel@2026");
    assert_eq!(doc.classes.len(), 9);
    let src = doc.source.unwrap();
    let sha = shelfmark_core::id::sha256_hex(&xml());
    assert!(src.description.contains(&sha), "{}", src.description);
    assert!(src.description.contains(jel_scheme::source::TREE_URL));
    assert_eq!(src.url.as_deref(), Some(jel_scheme::source::PAGE));
    let license = src.license.unwrap();
    assert!(
        license.contains("American Economic Association")
            && license.contains("Not redistributable")
    );
    assert!(doc
        .classes
        .iter()
        .all(|c| c.vocab.is_empty() && c.range.is_none()));
}

#[test]
fn rebuilds_byte_for_byte() {
    assert_eq!(build(&xml(), &lock()), build(&xml(), &lock()));
}

#[test]
fn browses_the_hierarchy() {
    let idx = index();
    let codes = |v: Vec<shelfmark_core::scheme::ClassRecord>| -> Vec<String> {
        v.into_iter().map(|c| c.id.code).collect()
    };
    assert_eq!(codes(idx.children(None)), ["A", "Y"]);
    assert_eq!(codes(idx.children(Some("A"))), ["A1", "A2"]);
    assert_eq!(codes(idx.children(Some("A1"))), ["A10", "A11", "A12"]);
    assert_eq!(codes(idx.ancestors("A11")), ["A11", "A1", "A"]);
    let rec = idx.get("A11").unwrap();
    assert_eq!(rec.label, "Harbour Tolls; Canal Pricing");
    assert_eq!(rec.level, 2);
    assert_eq!(rec.id.to_string(), "class:jel@2026:A11");
    assert_eq!(idx.get("A12").unwrap().label, "Lighthouse R&D, 1800–");
}

#[test]
fn searches_labels() {
    let idx = index();
    let hits = idx.search("canal pricing", 3);
    assert_eq!(
        hits.first().map(|h| h.class.code.as_str()),
        Some("A11"),
        "{hits:?}"
    );
    let hits = idx.search("weather vanes", 3);
    assert_eq!(
        hits.first().map(|h| h.class.code.as_str()),
        Some("Y91"),
        "{hits:?}"
    );
}

#[test]
fn checks_codes_against_the_grammar() {
    let idx = index();
    assert!(matches!(idx.check_code("A11"), CodeCheck::Exact { class } if class.code == "A11"));
    for bad in ["a11", "A111", "AB", "11", "A1.1"] {
        assert!(
            matches!(idx.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
    assert!(!matches!(
        idx.check_code("A13"),
        CodeCheck::Exact { .. } | CodeCheck::Invalid { .. }
    ));
}

#[test]
fn the_lock_refuses_a_changed_source() {
    let mut changed = xml();
    changed.extend_from_slice(b"\n");
    let err = lock().verify(&changed).unwrap_err();
    assert!(err.contains("--relock"), "{err}");
    lock().verify(&xml()).unwrap();
}

#[test]
fn build_fetches_nothing_when_the_cache_matches() {
    let dir = std::env::temp_dir().join(format!("jel-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("classificationTree.xml"), xml()).unwrap();
    let got = jel_scheme::source::cached(&dir, &lock()).unwrap();
    assert_eq!(got, xml());
    std::fs::write(dir.join("classificationTree.xml"), b"<data/>").unwrap();
    assert!(jel_scheme::source::cached(&dir, &lock()).is_err());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_committed_lock_records_the_license_check() {
    let lock = committed_lock();
    assert!(!lock.license.redistributable);
    assert!(lock
        .license
        .holder
        .contains("American Economic Association"));
    assert_eq!(lock.source.url, jel_scheme::source::TREE_URL);
    assert_eq!(lock.source.sha256.len(), 64);
    assert!(lock.source.sha256.chars().all(|c| c.is_ascii_hexdigit()));
    // The generated file stays out of git: the data may not be redistributed.
    let ignore = std::fs::read_to_string(here().join(".gitignore")).unwrap();
    assert!(ignore.lines().any(|l| l == "/jel.json"));
}
