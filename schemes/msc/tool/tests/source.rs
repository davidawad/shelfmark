//! The real MSC2020 CSV, when a cache holding it is named by
//! `MSC_SOURCE_CACHE` (as `msc-scheme build` leaves it). Without one each
//! test records a skip in `$TEST_SKIP_LOG`; `MSC_REQUIRE_SOURCE=1` turns
//! that skip into a failure. The tests never download.

use std::path::PathBuf;

use msc_scheme::sources::{self, Lock};
use msc_scheme::{csv, scheme};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

/// Record a skip the runner can count (cargo hides a passing test's output).
fn skip(why: &str) {
    assert!(
        std::env::var("MSC_REQUIRE_SOURCE").as_deref() != Ok("1"),
        "{why}, and MSC_REQUIRE_SOURCE=1 makes that a failure"
    );
    let test = std::thread::current().name().unwrap_or("?").to_string();
    let line = format!("SKIP: {why} [{test}]\n");
    eprint!("{line}");
    if let Some(log) = std::env::var_os("TEST_SKIP_LOG") {
        use std::io::Write;
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log)
            .and_then(|mut f| f.write_all(line.as_bytes()));
    }
}

fn real() -> Option<(SchemeDoc, String)> {
    let Some(cache) = std::env::var_os("MSC_SOURCE_CACHE") else {
        skip("MSC_SOURCE_CACHE is not set, so the real MSC2020 CSV is unavailable");
        return None;
    };
    let lock = Lock::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sources.lock.json"))
        .unwrap();
    let bytes = sources::obtain(&PathBuf::from(cache), &lock, true).unwrap();
    let doc = scheme::build(&csv::parse(&bytes).unwrap(), &lock).unwrap();
    let json = scheme::to_json(&doc).unwrap();
    Some((doc, json))
}

#[test]
fn the_real_csv_builds_a_valid_scheme_of_every_class() {
    let Some((doc, json)) = real() else { return };
    shelfmark_scheme::validate_bytes(json.as_bytes()).unwrap();
    assert_eq!(doc.classes.len(), 6603);
    assert_eq!(
        doc.classes.iter().filter(|c| c.parent.is_none()).count(),
        63
    );
    assert!(
        doc.classes.iter().all(|c| c.vocab.is_empty()),
        "the source has no LCC/LCSH links"
    );
}

#[test]
fn the_real_scheme_browses_searches_and_checks_codes() {
    let Some((_, json)) = real() else { return };
    let ix = shelfmark_scheme::compile_bytes(json.as_bytes(), "msc2020").unwrap();
    assert_eq!(ix.get("68P27").unwrap().label, "Privacy of data");
    let up: Vec<String> = ix
        .ancestors("14Q25")
        .iter()
        .map(|r| r.id.code.clone())
        .collect();
    assert_eq!(up, ["14Q25", "14Qxx", "14-XX"]);
    assert_eq!(
        ix.get("03F40").unwrap().label,
        "G\u{f6}del numberings and issues of incompleteness"
    );
    assert_eq!(ix.search("Turán theory", 1)[0].class.code, "11N30");
    assert_eq!(
        ix.get("05C99").unwrap().label,
        "None of the above, but in this section"
    );
    assert!(
        matches!(ix.check_code("05C98"), CodeCheck::Within { ref class } if class.code == "05Cxx")
    );
}
