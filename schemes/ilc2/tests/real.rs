//! The scheme built from the real, locked ILC2 file. Needs a cache filled
//! by `ilc2 build --cache DIR --fetch`, named by `ILC2_CACHE`; without one
//! the test is skipped (logged to `TEST_SKIP_LOG`), or fails when
//! `ILC2_REQUIRE_CACHE=1`.

mod common;

use std::path::PathBuf;

use ilc2::{build, generate, read_lock};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};

fn cache(test: &str) -> Option<PathBuf> {
    match std::env::var_os("ILC2_CACHE") {
        Some(dir) => Some(PathBuf::from(dir)),
        None if std::env::var("ILC2_REQUIRE_CACHE").as_deref() == Ok("1") => {
            panic!("ILC2_REQUIRE_CACHE=1 but ILC2_CACHE is unset")
        }
        None => {
            skip(test, "ILC2_CACHE unset (no fetched ILC2 file)");
            None
        }
    }
}

#[test]
fn builds_the_locked_file() {
    let Some(cache) = cache("builds_the_locked_file") else {
        return;
    };
    let lock = read_lock(&common::crate_dir().join("sources.lock.json")).unwrap();
    let (doc, stats) = generate(&cache, &lock).unwrap();
    let json = build::scheme_json(&doc).unwrap();
    assert_eq!(
        json,
        build::scheme_json(&generate(&cache, &lock).unwrap().0).unwrap()
    );
    let index = common::validate_and_compile(&json);
    assert_eq!(index.scheme().to_string(), "scheme:ilc2@2019-10-17");
    // The publisher counts 10,845 classes and facets; the SKOS file holds
    // 9,174 concepts and 943 facet indicators.
    assert_eq!(stats.concepts.classes, 9174);
    assert_eq!(stats.facets.classes, 943);
    assert_eq!(index.len(), 10117);
    assert_eq!(stats.concepts.roots, 65);

    assert_eq!(index.get("mq").unwrap().label, "animals");
    let hits = index.search("animals", 5);
    assert!(hits.iter().any(|h| h.class.code == "mq"), "{hits:?}");
    // A facet indicator is a class with domain and range as data.
    let facet = doc.classes.iter().find(|c| c.code == "mq94").unwrap();
    assert!(facet
        .note
        .as_deref()
        .unwrap()
        .starts_with("facet: domain=mq range=mqad"));

    for c in &doc.classes {
        assert!(
            matches!(index.check_code(&c.code), CodeCheck::Exact { .. }),
            "{}",
            c.code
        );
    }
}

/// Record a skipped test in `$TEST_SKIP_LOG` (and on stderr).
pub fn skip(test: &str, reason: &str) {
    let line = format!("SKIP: {reason} [ilc2 {test}]");
    eprintln!("{line}");
    if let Ok(path) = std::env::var("TEST_SKIP_LOG") {
        use std::io::Write as _;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        writeln!(f, "{line}").unwrap();
    }
}
