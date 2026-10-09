//! The builder on a synthetic source in the shape of the GBV's JSKOS
//! export: the output is a valid scheme, deterministic, browsable and
//! searchable in both languages, and the lock guards it. Needs no network
//! and no BK data.

mod common;

use bk_scheme::source::Lock;
use common::{assert_valid, build, committed_lock, here};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};

fn src() -> Vec<u8> {
    std::fs::read(here().join("fixtures/bk.ndjson")).unwrap()
}

/// The committed lock re-pinned to the fixture.
fn lock() -> Lock {
    committed_lock().relocked(&src(), "2026-10-09".into())
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(build(&src(), &lock()).as_bytes(), "bk.json").unwrap()
}

fn codes(v: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    v.into_iter().map(|c| c.id.code).collect()
}

#[test]
fn builds_a_valid_scheme_with_provenance() {
    let doc = assert_valid(&build(&src(), &lock()));
    assert_eq!(doc.id().to_string(), "scheme:bk@2026-10-03");
    assert_eq!(doc.classes.len(), 8);
    let s = doc.source.unwrap();
    let sha = shelfmark_core::id::sha256_hex(&src());
    assert!(s.description.contains(&sha), "{}", s.description);
    assert!(s.description.contains(bk_scheme::source::EXPORT_URL));
    assert_eq!(s.url.as_deref(), Some(bk_scheme::source::PAGE));
    let license = s.license.unwrap();
    assert!(
        license.contains("CC0") && license.contains("GBV"),
        "{license}"
    );
}

#[test]
fn rebuilds_byte_for_byte() {
    assert_eq!(build(&src(), &lock()), build(&src(), &lock()));
}

#[test]
fn lists_classes_in_tree_order_with_labels_vocab_and_notes() {
    let doc = assert_valid(&build(&src(), &lock()));
    let order: Vec<&str> = doc.classes.iter().map(|c| c.code.as_str()).collect();
    assert_eq!(
        order,
        ["1-2", "11", "11.10", "11.20", "11.30X", "9", "93", "93.10"]
    );
    let by = |code: &str| doc.classes.iter().find(|c| c.code == code).unwrap();

    // German label; English label and alternative labels as vocabulary,
    // never repeating the label itself.
    let canal = by("11.10");
    assert_eq!(canal.label, "Kanalgebühren");
    let terms: Vec<_> = canal
        .vocab
        .iter()
        .map(|v| (v.term.as_str(), v.source.as_str()))
        .collect();
    assert_eq!(
        terms,
        [("Canal tolls", "bk-en"), ("Schleusengeld", "bk-alt")]
    );
    assert_eq!(
        canal.note.as_deref(),
        Some("Definition: Abgaben für die Kanalnutzung")
    );

    // An empty English label adds nothing.
    assert!(by("11.30X").vocab.is_empty());
    // Scope note and cross-reference note are kept, in that order.
    assert_eq!(
        by("11.20").note.as_deref(),
        Some("Scope: Seezeichen | Note: Siehe auch: 11.10 <Kanalgebühren>")
    );
    // Headings are marked.
    assert!(by("11")
        .note
        .as_deref()
        .unwrap()
        .starts_with("Heading only"));
    assert!(by("11.10").parent.as_deref() == Some("11") && by("1-2").parent.is_none());
}

#[test]
fn browses_the_hierarchy() {
    let idx = index();
    assert_eq!(codes(idx.children(None)), ["1-2", "9"]);
    assert_eq!(
        codes(idx.children(Some("11"))),
        ["11.10", "11.20", "11.30X"]
    );
    assert_eq!(codes(idx.ancestors("11.20")), ["11.20", "11", "1-2"]);
    let rec = idx.get("93.10").unwrap();
    assert_eq!(rec.label, "Wetterfahnen");
    assert_eq!(rec.level, 2, "levels count from 0 at the roots");
    assert_eq!(rec.id.to_string(), "class:bk@2026-10-03:93.10");
}

#[test]
fn searches_german_labels_and_english_and_alternative_terms() {
    let idx = index();
    for (query, want) in [
        ("kanalgebühren", "11.10"),
        ("canal tolls", "11.10"),
        ("schleusengeld", "11.10"),
        ("weather vanes", "93.10"),
        ("leuchttürme", "11.20"),
    ] {
        let hits = idx.search(query, 3);
        assert_eq!(
            hits.first().map(|h| h.class.code.as_str()),
            Some(want),
            "{query}: {hits:?}"
        );
    }
}

#[test]
fn checks_codes_against_the_grammar() {
    let idx = index();
    for ok in ["11.10", "1-2", "11.30X", "93"] {
        assert!(
            matches!(idx.check_code(ok), CodeCheck::Exact { class } if class.code == ok),
            "{ok}"
        );
    }
    for bad in ["111", "11.1", "11.100", "a1", "11.10Y", "1-", "11.10x"] {
        assert!(
            matches!(idx.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
    assert!(!matches!(
        idx.check_code("11.99"),
        CodeCheck::Exact { .. } | CodeCheck::Invalid { .. }
    ));
}

#[test]
fn the_lock_refuses_a_changed_source() {
    let mut changed = src();
    changed.extend_from_slice(b"\n");
    let err = lock().verify(&changed).unwrap_err();
    assert!(err.contains("--relock"), "{err}");
    lock().verify(&src()).unwrap();
}

#[test]
fn build_fetches_nothing_when_the_cache_matches() {
    let dir = std::env::temp_dir().join(format!("bk-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(&lock().source.file);
    std::fs::write(&file, src()).unwrap();
    assert_eq!(bk_scheme::source::cached(&dir, &lock()).unwrap(), src());
    std::fs::write(&file, b"{}").unwrap();
    assert!(bk_scheme::source::cached(&dir, &lock()).is_err());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_committed_lock_records_the_license_check() {
    let lock = committed_lock();
    assert!(lock.license.redistributable);
    assert!(lock.license.terms.contains("CC0"));
    assert!(lock
        .license
        .terms_url
        .starts_with("https://wiki.k10plus.de/"));
    assert_eq!(lock.source.url, bk_scheme::source::EXPORT_URL);
    assert_eq!(lock.source.sha256.len(), 64);
    assert!(lock.source.sha256.chars().all(|c| c.is_ascii_hexdigit()));
}
