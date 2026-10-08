//! The full build from the locked source: needs the cached
//! `FASTTopical.nt.zip` in `$FAST_SOURCES` (`fast-scheme fetch --cache
//! DIR`). Without it the test skips, says so on stderr and appends one
//! line to `$TEST_SKIP_LOG`; `FAST_REQUIRE_SOURCES=1` makes the skip a
//! failure.

use std::io::Write;
use std::path::{Path, PathBuf};

use fast_scheme::{generate, sources, Outputs};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};

fn here(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn skip(why: &str) {
    assert!(
        std::env::var("FAST_REQUIRE_SOURCES").as_deref() != Ok("1"),
        "{why}, and FAST_REQUIRE_SOURCES=1 makes that a failure"
    );
    let test = std::thread::current().name().unwrap_or("?").to_string();
    let line = format!("SKIP: {why} [{test}]");
    eprintln!("{line}");
    if let Some(log) = std::env::var_os("TEST_SKIP_LOG") {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log)
            .and_then(|mut f| f.write_all(format!("{line}\n").as_bytes()));
    }
}

#[test]
fn the_locked_source_rebuilds_the_committed_files_and_a_valid_scheme() {
    let lock = sources::read_lock(&here("sources.lock.json")).unwrap();
    let Some(cache) = std::env::var_os("FAST_SOURCES") else {
        return skip("FAST_SOURCES is not set (no cached FAST source)");
    };
    let cache = Path::new(&cache);
    if lock
        .sources
        .iter()
        .any(|l| !sources::cached(cache, l).exists())
    {
        return skip("FAST_SOURCES lacks the locked files; run `fast-scheme fetch`");
    }
    let out = generate(cache, &lock).unwrap();
    for (file, want) in [(Outputs::SAMPLE, &out.sample), (Outputs::NOTES, &out.notes)] {
        let have = std::fs::read_to_string(here(file)).unwrap();
        assert!(&have == want, "{file} is stale; run `fast-scheme build`");
    }
    let idx = shelfmark_scheme::compile_bytes(out.scheme.as_bytes(), "fast.json").unwrap();
    assert_eq!(idx.scheme().to_string(), "scheme:fast@2024");
    assert_eq!(idx.len(), 440_612);
    assert_eq!(idx.get("fst00997341").unwrap().label, "Libraries");
    assert!(matches!(
        idx.check_code("fst01004795"),
        CodeCheck::Exact { .. }
    ));
    let hits = idx.search("artists studios", 3);
    assert_eq!(hits[0].class.code, "fst00817698", "{hits:?}");
}
