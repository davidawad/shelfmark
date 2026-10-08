//! The compile cache (hit/miss on file change), AE7 and determinism.

use std::path::{Path, PathBuf};

use shelfmark_core::id::SchemeId;
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::ErrorCode;
use shelfmark_scheme::{compile_file, CacheStatus, CompileCache};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/schemes/mini-lcc.json")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("shelfmark-scheme-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn copy_fixture(dir: &Path) -> PathBuf {
    let path = dir.join("mini-lcc.json");
    std::fs::copy(fixture(), &path).unwrap();
    path
}

#[test]
fn unchanged_file_hits_changed_file_misses() {
    let dir = scratch("hitmiss");
    let src = copy_fixture(&dir);
    let cache = CompileCache::new(dir.join("cache"));

    let first = cache.compile_file(&src).unwrap();
    assert_eq!(first.status, CacheStatus::Miss);
    let again = cache.compile_file(&src).unwrap();
    assert_eq!(again.status, CacheStatus::Hit);
    assert_eq!(again.index.content_hash(), first.index.content_hash());

    let text = std::fs::read_to_string(&src).unwrap();
    std::fs::write(&src, text.replace("\"Law\"", "\"Law (general)\"")).unwrap();
    let changed = cache.compile_file(&src).unwrap();
    assert_eq!(changed.status, CacheStatus::Miss);
    assert_ne!(changed.index.content_hash(), first.index.content_hash());
    assert_eq!(changed.index.get("K").unwrap().label, "Law (general)");

    // Changing it back finds the first compile again.
    std::fs::write(&src, text).unwrap();
    assert_eq!(cache.compile_file(&src).unwrap().status, CacheStatus::Hit);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_corrupt_entry_recompiles() {
    let dir = scratch("corrupt");
    let src = copy_fixture(&dir);
    let cache = CompileCache::new(dir.join("cache"));
    cache.compile_file(&src).unwrap();
    for entry in std::fs::read_dir(dir.join("cache/objects")).unwrap() {
        std::fs::write(entry.unwrap().path(), b"garbage").unwrap();
    }
    assert_eq!(cache.compile_file(&src).unwrap().status, CacheStatus::Miss);
    assert_eq!(cache.compile_file(&src).unwrap().status, CacheStatus::Hit);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn invalid_files_are_not_cached() {
    let dir = scratch("invalid");
    let src = dir.join("bad.json");
    std::fs::write(&src, br#"{"name":"x"}"#).unwrap();
    let cache = CompileCache::new(dir.join("cache"));
    let err = cache.compile_file(&src).unwrap_err();
    assert_eq!(err.code, ErrorCode::SchemeInvalid);
    assert!(!dir.join("cache/objects").exists());
    let err = cache.compile_file(&dir.join("missing.json")).unwrap_err();
    assert_eq!(err.code, ErrorCode::InputUnreadable);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn open_returns_the_latest_compile() {
    let dir = scratch("open");
    let src = copy_fixture(&dir);
    let cache = CompileCache::new(dir.join("cache"));
    let compiled = cache.compile_file(&src).unwrap().index;
    let opened = cache
        .open(&SchemeId::new("mini-lcc", "2024"), None)
        .unwrap();
    assert_eq!(opened.content_hash(), compiled.content_hash());
    assert_eq!(
        opened.search("compilers", 3),
        compiled.search("compilers", 3)
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn ae7_uncompiled_scheme_names_the_compile_command() {
    let dir = scratch("ae7");
    let cache = CompileCache::new(dir.join("cache"));
    let scheme = SchemeId::new("lcc", "2024");

    let err = cache
        .open(&scheme, Some("data/schemes/lcc/lcc.json"))
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::SchemeNotCompiled);
    assert_eq!(
        err.next_command.as_deref(),
        Some("shelfmark scheme compile data/schemes/lcc/lcc.json")
    );
    assert_eq!(err.details.unwrap()["scheme"], "scheme:lcc@2024");

    let err = cache.open(&scheme, None).unwrap_err();
    assert_eq!(err.code, ErrorCode::SchemeNotCompiled);
    assert!(err
        .next_command
        .unwrap()
        .starts_with("shelfmark scheme compile "));

    // A pointer whose compiled object vanished still names the known source.
    let src = copy_fixture(&dir);
    cache.compile_file(&src).unwrap();
    std::fs::remove_dir_all(dir.join("cache/objects")).unwrap();
    let err = cache
        .open(&SchemeId::new("mini-lcc", "2024"), None)
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::SchemeNotCompiled);
    assert_eq!(
        err.next_command.unwrap(),
        format!("shelfmark scheme compile {}", src.display())
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn compiles_are_deterministic() {
    let dir = scratch("determinism");
    let src = copy_fixture(&dir);
    let (a, b) = (compile_file(&src).unwrap(), compile_file(&src).unwrap());
    let cache_a = CompileCache::new(dir.join("a"));
    let cache_b = CompileCache::new(dir.join("b"));
    cache_a.compile_file(&src).unwrap();
    cache_b.compile_file(&src).unwrap();
    let objects = |d: &str| {
        let mut out = Vec::new();
        for e in std::fs::read_dir(dir.join(d).join("objects")).unwrap() {
            out.push(std::fs::read(e.unwrap().path()).unwrap());
        }
        out
    };
    assert_eq!(objects("a"), objects("b"), "byte-identical compiled form");

    for q in [
        "computer",
        "law social",
        "compilers computer programs",
        "science",
    ] {
        assert_eq!(a.search(q, 10), b.search(q, 10), "{q}");
    }
    for code in ["QA76.76", "QB5", "Z1", "bad", "QA76"] {
        assert_eq!(a.check_code(code), b.check_code(code), "{code}");
    }
    assert_eq!(a.children(None), b.children(None));
    assert_eq!(a.vocab(), b.vocab());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn compile_object_hits_without_writing_a_pointer() {
    let dir = scratch("object");
    let bytes = std::fs::read(fixture()).unwrap();
    let cache = CompileCache::new(dir.join("cache"));

    let first = cache.compile_object(&bytes, "mini").unwrap();
    assert_eq!(first.status, CacheStatus::Miss);
    let again = cache.compile_object(&bytes, "mini").unwrap();
    assert_eq!(again.status, CacheStatus::Hit);
    assert_eq!(again.index.content_hash(), first.index.content_hash());
    assert!(!dir.join("cache/schemes").exists());
    let err = cache
        .open(first.index.scheme(), None)
        .expect_err("no pointer was written");
    assert_eq!(err.code, ErrorCode::SchemeNotCompiled);
    let _ = std::fs::remove_dir_all(dir);
}
