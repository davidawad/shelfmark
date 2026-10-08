//! The scheme built from ACM's real SKOS file. The file is not committed
//! (see the README), so these tests read it from `ACM_CCS_CACHE` (default:
//! this crate's `cache/`, filled by `acm-ccs fetch`) and skip, logging to
//! `TEST_SKIP_LOG`, when it is absent. `ACM_CCS_REQUIRE=1` turns a skip
//! into a failure.

mod common;

use std::path::PathBuf;

use acm_ccs::build::Stats;
use acm_ccs::sources::{read_cached, Lock};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

/// The built scheme, or `None` after logging a skip.
fn official(test: &str) -> Option<(SchemeDoc, String, Stats)> {
    let cache = std::env::var_os("ACM_CCS_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| acm_ccs::home().join("cache"));
    let lock = Lock::read(&acm_ccs::lock_path()).unwrap();
    let Some(bytes) = read_cached(&cache, &lock).unwrap() else {
        common::skip(
            test,
            &format!("{} not cached in {}", lock.file, cache.display()),
        );
        return None;
    };
    Some(acm_ccs::build_bytes(&bytes, &lock).unwrap())
}

#[test]
fn official_scheme_has_the_2012_shape() {
    let Some((doc, json, stats)) = official("official_scheme_has_the_2012_shape") else {
        return;
    };
    assert_eq!(
        stats,
        Stats {
            classes: 2113,
            top: 13,
            depth: 6,
            poly: 159
        }
    );
    assert_eq!(doc.id().to_string(), "scheme:acm-ccs@2012");
    let errors = common::schema_errors(&json);
    assert!(errors.is_empty(), "{errors:?}");
    if let Err(problems) = shelfmark_scheme::validate_bytes(json.as_bytes()) {
        panic!(
            "{} problems: {:?}",
            problems.len(),
            &problems[..problems.len().min(10)]
        );
    }
}

#[test]
fn official_scheme_browses_searches_and_checks_codes() {
    let Some((doc, json, _)) = official("official_scheme_browses_searches_and_checks_codes") else {
        return;
    };
    let idx = shelfmark_scheme::compile_bytes(json.as_bytes(), "acm-ccs.json").unwrap();
    assert_eq!(idx.len(), 2113);
    assert_eq!(idx.children(None).len(), 13);
    // The deepest class: its ancestors run back to a top-level class.
    let deepest = doc
        .classes
        .iter()
        .max_by_key(|c| (c.code.split('.').count(), std::cmp::Reverse(&c.code)))
        .unwrap();
    let up = idx.ancestors(&deepest.code);
    assert_eq!(up.len(), 6);
    assert_eq!(up.last().unwrap().level, 0);
    // Every top-level label finds its own class first.
    for top in idx.children(None) {
        let hits = idx.search(&top.label, 3);
        assert_eq!(hits[0].class, top.id, "{}", top.label);
    }
    let top = &idx.children(None)[0].id.code;
    assert!(matches!(idx.check_code(top), CodeCheck::Exact { .. }));
    assert!(matches!(
        idx.check_code(&format!("{top}.99999999")),
        CodeCheck::Within { .. }
    ));
    // A 1998 CCS code is not a 2012 code.
    assert!(matches!(idx.check_code("I.2.6"), CodeCheck::Invalid { .. }));
}
