//! The scheme built from the real AEA source. The source may not ship
//! with the workspace, so these run only when `JEL_CACHE` names a
//! directory holding the locked `classificationTree.xml` (or one to
//! download it into, with `JEL_FETCH=1`); otherwise each test records a
//! skip in `$TEST_SKIP_LOG`, and `JEL_REQUIRE_SOURCE=1` makes that a
//! failure.

mod common;

use common::{assert_valid, build, committed_lock, skip};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};

fn source() -> Option<Vec<u8>> {
    let Some(cache) = std::env::var_os("JEL_CACHE").map(std::path::PathBuf::from) else {
        skip("JEL_CACHE is unset, so the AEA source is not available");
        return None;
    };
    let lock = committed_lock();
    if !cache.join(&lock.source.file).exists() && std::env::var("JEL_FETCH").as_deref() != Ok("1") {
        skip("JEL_CACHE has no classificationTree.xml and JEL_FETCH is not 1");
        return None;
    }
    Some(jel_scheme::source::cached(&cache, &lock).unwrap())
}

fn index(xml: &[u8]) -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(build(xml, &committed_lock()).as_bytes(), "jel.json").unwrap()
}

#[test]
fn the_locked_source_builds_a_valid_scheme() {
    let Some(xml) = source() else { return };
    let text = build(&xml, &committed_lock());
    let doc = assert_valid(&text);
    assert_eq!(doc.id().to_string(), "scheme:jel@2026");
    assert!(doc.classes.len() > 900, "{} classes", doc.classes.len());
    assert_eq!(text, build(&xml, &committed_lock()));
}

#[test]
fn has_the_twenty_categories_and_their_subcategories() {
    let Some(xml) = source() else { return };
    let idx = index(&xml);
    let top: Vec<String> = idx.children(None).into_iter().map(|c| c.id.code).collect();
    assert_eq!(
        top,
        "A B C D E F G H I J K L M N O P Q R Y Z"
            .split(' ')
            .collect::<Vec<_>>()
    );
    for (code, label) in [
        ("A", "General Economics and Teaching"),
        ("E", "Macroeconomics and Monetary Economics"),
        (
            "E5",
            "Monetary Policy, Central Banking, and the Supply of Money and Credit",
        ),
        ("O32", "Management of Technological Innovation and R&D"),
    ] {
        assert_eq!(
            idx.get(code).unwrap_or_else(|| panic!("no {code}")).label,
            label
        );
    }
    let ancestors: Vec<String> = idx
        .ancestors("E52")
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    assert_eq!(ancestors, ["E52", "E5", "E"]);
}

#[test]
fn searches_and_checks_real_codes() {
    let Some(xml) = source() else { return };
    let idx = index(&xml);
    let hits = idx.search("monetary policy", 10);
    assert!(hits.iter().any(|h| h.class.code == "E52"), "{hits:?}");
    assert!(matches!(idx.check_code("E52"), CodeCheck::Exact { .. }));
    assert!(matches!(idx.check_code("E5X"), CodeCheck::Invalid { .. }));
}
