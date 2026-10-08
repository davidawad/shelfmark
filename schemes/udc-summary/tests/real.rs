//! The scheme built from the real, locked udcsummary.info pages. Needs a
//! cache filled by `udc-summary build --cache DIR --fetch`, named by
//! `UDC_SUMMARY_CACHE`; without one the test is skipped (logged to
//! `TEST_SKIP_LOG`), or fails when `UDC_SUMMARY_REQUIRE_CACHE=1`.

mod common;

use std::path::PathBuf;

use shelfmark_core::scheme::{CodeCheck, CompiledIndex};
use udc_summary::{build, generate, read_lock};

fn cache(test: &str) -> Option<PathBuf> {
    match std::env::var_os("UDC_SUMMARY_CACHE") {
        Some(dir) => Some(PathBuf::from(dir)),
        None if std::env::var("UDC_SUMMARY_REQUIRE_CACHE").as_deref() == Ok("1") => {
            panic!("UDC_SUMMARY_REQUIRE_CACHE=1 but UDC_SUMMARY_CACHE is unset")
        }
        None => {
            skip(
                test,
                "UDC_SUMMARY_CACHE unset (no fetched UDC Summary pages)",
            );
            None
        }
    }
}

#[test]
fn builds_the_locked_summary() {
    let Some(cache) = cache("builds_the_locked_summary") else {
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
    assert_eq!(
        index.scheme().to_string(),
        format!("scheme:udc-summary@{}", lock.edition)
    );
    // The site says "around 2,600 classes".
    assert!((2500..3000).contains(&index.len()), "{}", index.len());
    assert_eq!(stats.headers, vec!["---".to_string()]);
    assert!(stats.per_page.iter().all(|(_, n)| *n > 0), "{stats:?}");

    let roots: Vec<String> = index
        .children(None)
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    for top in ["0", "1", "2", "3", "5", "6", "7", "8", "9", "=...", "(1/9)"] {
        assert!(roots.contains(&top.to_string()), "{top} in {roots:?}");
    }
    assert!(!roots.contains(&"4".to_string()), "class 4 is vacant");

    let up: Vec<String> = index
        .ancestors("004.738")
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    assert_eq!(up.last().map(String::as_str), Some("0"), "{up:?}");
    assert!(up.contains(&"004".to_string()), "{up:?}");
    assert!(index
        .get("004.738")
        .unwrap()
        .label
        .contains("Internetworking"));

    let hits = index.search("internetworking", 5);
    assert_eq!(hits[0].class.code, "004.738", "{hits:?}");
    let hits = index.search("biotechnology", 5);
    assert!(hits.iter().any(|h| h.class.code == "60"), "{hits:?}");

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
    let line = format!("SKIP: {reason} [udc-summary {test}]");
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
