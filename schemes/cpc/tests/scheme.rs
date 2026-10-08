//! The built CPC scheme is a valid shelfmark scheme: it passes the
//! published JSON Schema, round-trips through `SchemeDoc`, passes the
//! scheme compiler's checks, and browses and searches as CPC should.
//!
//! Most tests build from `fixtures/titles-excerpt.txt`, 42 lines of the
//! 2026.08 title list. `full_release_*` builds the locked release itself
//! when `CPC_SOURCES` names a cache holding it (`cpc-scheme fetch`), and
//! otherwise logs a skip to `$TEST_SKIP_LOG`
//! (`CPC_REQUIRE_SOURCES=1` turns the skip into a failure).

use std::path::{Path, PathBuf};

use cpc_scheme::build::{self, Depth, Provenance};
use cpc_scheme::sources::{self, Lock};
use cpc_scheme::{generate, render, titles};
use serde_json::Value;
use shelfmark_core::id::sha256_hex;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn excerpt(depth: Depth) -> SchemeDoc {
    let text = std::fs::read_to_string(here().join("tests/fixtures/titles-excerpt.txt")).unwrap();
    let prov = Provenance {
        release: "2026.08".into(),
        file: "titles-excerpt.txt".into(),
        url: "tests/fixtures/titles-excerpt.txt".into(),
        sha256: sha256_hex(text.as_bytes()),
    };
    build::build(&titles::parse(&text).unwrap(), &prov, depth).unwrap()
}

fn json_of(doc: &SchemeDoc) -> String {
    render::scheme_json(doc).unwrap()
}

fn index(doc: &SchemeDoc) -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(json_of(doc).as_bytes(), "cpc.json").unwrap()
}

fn codes(recs: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    recs.into_iter().map(|c| c.id.code).collect()
}

#[test]
fn validates_against_the_published_schema() {
    let schema: Value = serde_json::from_str(
        &std::fs::read_to_string(here().join("../../schemas/scheme.schema.json")).unwrap(),
    )
    .unwrap();
    let v = jsonschema::validator_for(&schema).unwrap();
    let doc: Value = serde_json::from_str(&json_of(&excerpt(Depth::All))).unwrap();
    let errors: Vec<String> = v.iter_errors(&doc).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn round_trips_through_the_model() {
    let doc = excerpt(Depth::All);
    let back: SchemeDoc = serde_json::from_str(&json_of(&doc)).unwrap();
    assert_eq!(back, doc);
    assert_eq!(back.id().to_string(), "scheme:cpc@2026.08");
    let source = back.source.unwrap();
    assert!(source.description.contains(&sha256_hex(
        std::fs::read(here().join("tests/fixtures/titles-excerpt.txt"))
            .unwrap()
            .as_slice()
    )));
    assert!(source.license.unwrap().contains("European Patent Office"));
}

#[test]
fn passes_the_scheme_compilers_checks() {
    for depth in [Depth::All, Depth::MainGroup, Depth::Section] {
        if let Err(problems) = shelfmark_scheme::validate_bytes(json_of(&excerpt(depth)).as_bytes())
        {
            panic!("{depth:?}: {problems:?}");
        }
    }
}

#[test]
fn browses_the_hierarchy() {
    let idx = index(&excerpt(Depth::All));
    assert_eq!(idx.len(), 42);
    assert_eq!(codes(idx.children(None)), ["A", "G", "H", "Y"]);
    assert_eq!(codes(idx.children(Some("A01"))), ["A01B", "A01C"]);
    assert_eq!(
        codes(idx.children(Some("A01B1/02"))),
        [
            "A01B1/022",
            "A01B1/024",
            "A01B1/026",
            "A01B1/028",
            "A01B1/04"
        ]
    );
    assert_eq!(
        codes(idx.ancestors("G06F16/337")),
        [
            "G06F16/337",
            "G06F16/335",
            "G06F16/33",
            "G06F16/30",
            "G06F16/00",
            "G06F",
            "G06",
            "G"
        ]
    );
    // A 2000-series code nests inside main group 1/00's run.
    assert_eq!(
        idx.get("A01C2001/048").unwrap().parent.unwrap().code,
        "A01C1/04"
    );
    assert_eq!(
        idx.get("Y02E10/50").unwrap().label,
        "Photovoltaic [PV] energy"
    );
}

#[test]
fn searches_labels() {
    let idx = index(&excerpt(Depth::All));
    let top = |q: &str| {
        idx.search(q, 3)
            .into_iter()
            .map(|h| h.class.code)
            .collect::<Vec<_>>()
    };
    assert!(top("spades shovels").contains(&"A01B1/02".to_string()));
    assert!(top("photovoltaic").contains(&"Y02E10/50".to_string()));
    assert!(top("stream cipher").contains(&"H04L9/065".to_string()));
}

#[test]
fn checks_codes() {
    let idx = index(&excerpt(Depth::All));
    match idx.check_code("H04L9/065") {
        CodeCheck::Exact { class } => assert_eq!(class.code, "H04L9/065"),
        other => panic!("{other:?}"),
    }
    // No ranges, so a subgroup the scheme lacks lands on its subclass.
    match idx.check_code("H04L9/0643") {
        CodeCheck::Within { class } => assert_eq!(class.code, "H04L"),
        other => panic!("{other:?}"),
    }
    // Known limit (README): the shelf key drops trailing zeros, so an
    // absent main group 90 keys like main group 9. If this starts failing,
    // the key learned CPC; update the README's Limits section.
    match idx.check_code("H04L90/00") {
        CodeCheck::Within { class } => assert_eq!(class.code, "H04L9/00"),
        other => panic!("{other:?}"),
    }
    for bad in ["h04l9/06", "H04L9", "Q01A1/00"] {
        assert!(
            matches!(idx.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
}

#[test]
fn depth_cuts_below_main_groups() {
    let doc = excerpt(Depth::MainGroup);
    assert!(doc
        .classes
        .iter()
        .all(|c| !c.code.contains('/') || c.code.ends_with("/00")));
    assert_eq!(doc.classes.len(), 18);
}

/// Record a skip the runner can count; panics when `CPC_REQUIRE_SOURCES=1`.
fn skip(why: &str) {
    assert!(
        std::env::var("CPC_REQUIRE_SOURCES").as_deref() != Ok("1"),
        "{why}, and CPC_REQUIRE_SOURCES=1 makes that a failure"
    );
    let line = format!(
        "SKIP: {why} [{}]\n",
        std::thread::current().name().unwrap_or("?")
    );
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

#[test]
fn full_release_builds_to_the_locked_hashes_and_compiles() {
    let Some(cache) = std::env::var_os("CPC_SOURCES") else {
        skip("CPC_SOURCES is unset, so the locked CPC title list is not available");
        return;
    };
    let lock: Lock =
        serde_json::from_str(&std::fs::read_to_string(here().join("sources.lock.json")).unwrap())
            .unwrap();
    let bytes = sources::read(Path::new(&cache), &lock.source).unwrap();
    for out in &lock.outputs {
        let depth = Depth::parse(&out.depth).unwrap();
        let (doc, text) = generate(&bytes, &lock.source, &lock.release, depth).unwrap();
        assert_eq!(sha256_hex(text.as_bytes()), out.sha256, "{}", out.file);
        assert_eq!(doc.classes.len(), out.classes);
        let idx = shelfmark_scheme::compile_bytes(text.as_bytes(), &out.file).unwrap();
        assert_eq!(idx.len(), out.classes);
        assert_eq!(
            codes(idx.children(None)),
            ["A", "B", "C", "D", "E", "F", "G", "H", "Y"]
        );
        if depth == Depth::All {
            assert_eq!(
                idx.get("H04L9/0861").unwrap().parent.unwrap().code,
                "H04L9/08"
            );
            let hits = idx.search("photovoltaic energy", 10);
            assert!(hits.iter().any(|h| h.class.code == "Y02E10/50"), "{hits:?}");
        }
    }
}
