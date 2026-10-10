//! The committed `schemes/ipc/ipc.json` is a valid scheme: it passes the
//! published JSON Schema, round-trips through `SchemeDoc`, passes the
//! scheme compiler's own checks and answers validate, browse and search
//! the way the classifier uses them.
//!
//! `ipc.json` is not committed (see the README); build it with the builder's
//! `fetch` and `build`. Without it every test skips, says so on stderr and
//! appends one line to `$TEST_SKIP_LOG`; `IPC_REQUIRE_BUILD=1` makes the skip
//! a failure.

use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::Value;
use shelfmark_core::id::ClassId;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// Whether `schemes/ipc/ipc.json` is built; logs a skip when it is not.
fn built() -> bool {
    if root().join("schemes/ipc/ipc.json").exists() {
        return true;
    }
    let why = "schemes/ipc/ipc.json is not built (run the builder's fetch and build)";
    assert!(
        std::env::var("IPC_REQUIRE_BUILD").as_deref() != Ok("1"),
        "{why}, and IPC_REQUIRE_BUILD=1 makes that a failure"
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
    false
}

fn bytes() -> &'static [u8] {
    static B: OnceLock<Vec<u8>> = OnceLock::new();
    B.get_or_init(|| std::fs::read(root().join("schemes/ipc/ipc.json")).unwrap())
}

fn json(rel: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(rel)).unwrap()).unwrap()
}

fn index() -> &'static shelfmark_scheme::SchemeIndex {
    static I: OnceLock<shelfmark_scheme::SchemeIndex> = OnceLock::new();
    I.get_or_init(|| shelfmark_scheme::compile_bytes(bytes(), "ipc.json").unwrap())
}

fn codes(records: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    records.into_iter().map(|c| c.id.code).collect()
}

#[test]
fn validates_against_the_published_schema() {
    if !built() {
        return;
    }
    let schema = json("schemas/scheme.schema.json");
    let v = jsonschema::validator_for(&schema).unwrap();
    let doc = json("schemes/ipc/ipc.json");
    let errors: Vec<String> = v
        .iter_errors(&doc)
        .take(10)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn round_trips_through_the_model() {
    if !built() {
        return;
    }
    let doc = json("schemes/ipc/ipc.json");
    let model: SchemeDoc = serde_json::from_value(doc.clone()).unwrap();
    assert_eq!(model.id().to_string(), "scheme:ipc@2026.01");
    assert_eq!(serde_json::to_value(&model).unwrap(), doc);
}

#[test]
fn passes_the_scheme_compilers_checks() {
    if !built() {
        return;
    }
    if let Err(problems) = shelfmark_scheme::validate_bytes(bytes()) {
        panic!(
            "{} problems, first: {:?}",
            problems.len(),
            &problems[..problems.len().min(10)]
        );
    }
}

#[test]
fn records_provenance_and_license() {
    if !built() {
        return;
    }
    let doc: SchemeDoc = serde_json::from_slice(bytes()).unwrap();
    let source = doc.source.unwrap();
    assert!(source.license.unwrap().starts_with("CC BY 4.0"));
    let lock = json("schemes/ipc/sources.lock.json");
    for s in lock["sources"].as_array().unwrap() {
        for key in ["url", "sha256", "member_sha256"] {
            let want = s[key].as_str().unwrap();
            assert!(
                source.description.contains(want),
                "description lacks {key} {want}"
            );
        }
    }
}

#[test]
fn browses_sections_classes_and_subclasses() {
    if !built() {
        return;
    }
    let idx = index();
    assert_eq!(
        codes(idx.children(None)),
        ["A", "B", "C", "D", "E", "F", "G", "H"]
    );
    assert_eq!(idx.get("H").unwrap().label, "ELECTRICITY");
    assert_eq!(
        codes(idx.children(Some("H"))),
        ["H01", "H02", "H03", "H04", "H05", "H10", "H99"]
    );
    let h01b = idx.get("H01B").unwrap();
    assert!(
        h01b.label.starts_with("CABLES; CONDUCTORS; INSULATORS"),
        "{}",
        h01b.label
    );
    assert_eq!(h01b.level, 2);
    let subclasses: usize = idx
        .children(None)
        .iter()
        .flat_map(|s| idx.children(Some(&s.id.code)))
        .map(|c| idx.children(Some(&c.id.code)).len())
        .sum();
    assert_eq!(subclasses, 655);
    assert_eq!(idx.len(), 80_145);
}

#[test]
fn nests_groups_by_dot_level() {
    if !built() {
        return;
    }
    let idx = index();
    assert_eq!(
        codes(idx.ancestors("H01B1/02")),
        ["H01B1/02", "H01B1/00", "H01B", "H01", "H"]
    );
    assert_eq!(
        idx.get("H01B1/02").unwrap().label,
        "mainly consisting of metals or alloys"
    );
    // A deep subgroup: three dots under its main group.
    let deep = idx.ancestors("H01B1/08");
    assert_eq!(codes(deep.clone())[1..3], ["H01B1/06", "H01B1/00"]);
    assert_eq!(deep[0].level, 5);
}

#[test]
fn checks_codes_against_the_grammar_and_the_hierarchy() {
    if !built() {
        return;
    }
    let idx = index();
    match idx.check_code("H01B1/02") {
        CodeCheck::Exact { class } => assert_eq!(class.to_string(), "class:ipc@2026.01:H01B1/02"),
        other => panic!("{other:?}"),
    }
    match idx.check_code("H01B99/00") {
        CodeCheck::Within { class } => assert_eq!(class.code, "H01B"),
        other => panic!("{other:?}"),
    }
    for bad in ["H01B 1/02", "I01", "H01B1/2", "lorem"] {
        assert!(
            matches!(idx.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
}

#[test]
fn class_ids_round_trip() {
    if !built() {
        return;
    }
    let id: ClassId = "class:ipc@2026.01:A01B3/421".parse().unwrap();
    assert_eq!(id.code, "A01B3/421");
    assert_eq!(id.to_string(), "class:ipc@2026.01:A01B3/421");
    assert!(index().get(&id.code).is_some());
}

#[test]
fn searches_labels_and_catchwords() {
    if !built() {
        return;
    }
    let idx = index();
    let hits = idx.search("abacuses", 5);
    assert!(hits.iter().any(|h| h.class.code == "G06C1/00"), "{hits:?}");
    let g06c = &idx.vocab()["G06C1/00"];
    assert!(
        g06c.iter().any(|v| v.term == "ABACUSES"
            && v.source == "ipc-catchword"
            && v.source_id.as_deref() == Some("CW259852291")),
        "{g06c:?}"
    );
    let hits = idx.search("superconductive cables", 10);
    assert!(
        hits.iter().any(|h| h.class.code.starts_with("H01B12")),
        "{hits:?}"
    );
    // Guidance headings reach every main group under them.
    let ploughs = idx.search("ploughs", 50);
    assert!(
        ploughs.iter().any(|h| h.class.code == "A01B3/00"),
        "{ploughs:?}"
    );
}

#[test]
fn every_class_has_a_clean_label_and_no_lcc_or_lcsh_vocab() {
    if !built() {
        return;
    }
    let doc: SchemeDoc = serde_json::from_slice(bytes()).unwrap();
    for c in &doc.classes {
        assert!(!c.label.trim().is_empty(), "{} has no label", c.code);
        assert_eq!(
            c.label,
            c.label.split_whitespace().collect::<Vec<_>>().join(" "),
            "{}",
            c.code
        );
        for v in &c.vocab {
            assert!(
                ["ipc-catchword", "ipc-heading"].contains(&v.source.as_str()),
                "{v:?}"
            );
            // An indication read twice ("AGRICULTUREAGRICULTURE").
            let (a, b) = v.term.split_at(v.term.len() / 2);
            assert!(
                a != b || v.term.len() < 4,
                "{} has doubled term {:?}",
                c.code,
                v.term
            );
        }
    }
}
