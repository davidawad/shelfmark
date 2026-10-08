//! The bundled LCSH-enriched scheme and the store's cache of derived data
//! (compiled schemes, vocab matchers).

mod common;

use common::{call, data, seed_mini_lcc, temp_dir};
use serde_json::json;
use shelfmark_core::scheme::SchemeDoc;
use shelfmark_core::Verb;

fn outline() -> SchemeDoc {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/schemes/lcc/lcc.json"
    );
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out: Vec<_> = std::fs::read_dir(dir)
        .map(|d| d.map(|e| e.unwrap().path()).collect())
        .unwrap_or_default();
    out.sort();
    out
}

#[test]
fn the_bundled_scheme_is_canonical_and_valid() {
    let bytes = shelfmark_cli::wiring::bundled_lcc_bytes();
    let doc: SchemeDoc = serde_json::from_slice(&bytes).unwrap();
    // Canonical bytes hash the same as the same document in a store.
    assert!(serde_json::to_vec(&doc).unwrap() == bytes);
    shelfmark_scheme::validate_bytes(&bytes).expect("bundled scheme validates");
    let index = shelfmark_cli::wiring::bundled_lcc();
    assert_eq!(index.content_hash(), shelfmark_scheme::content_hash(&bytes));
}

#[test]
fn the_bundled_scheme_is_the_outline_plus_lcsh_vocabulary() {
    let outline = outline();
    let bundled: SchemeDoc =
        serde_json::from_slice(&shelfmark_cli::wiring::bundled_lcc_bytes()).unwrap();
    assert_eq!(bundled.id(), outline.id());
    assert_eq!(bundled.code_grammar, outline.code_grammar);
    assert_eq!(bundled.classes.len(), outline.classes.len());
    let mut lcsh = 0;
    for (b, o) in bundled.classes.iter().zip(&outline.classes) {
        assert_eq!(
            (&b.code, &b.label, &b.parent, &b.range, &b.note),
            (&o.code, &o.label, &o.parent, &o.range, &o.note)
        );
        for link in &o.vocab {
            assert!(
                b.vocab.iter().any(|v| v.term == link.term
                    && v.source == link.source
                    && v.source_id == link.source_id),
                "{}: outline term {:?} is kept",
                o.code,
                link.term
            );
        }
        lcsh += b.vocab.iter().filter(|v| v.source == "lcsh").count();
    }
    assert!(lcsh > 90_000, "{lcsh} LCSH links");
}

#[test]
fn a_store_caches_compiled_schemes_and_vocab_matchers() {
    let store = temp_dir("cache").join("store");
    seed_mini_lcc(&store);
    let args = json!({"text": "A book about compilers and the law"});
    let first = data(call(
        &shelfmark_cli::wiring::open(&store),
        Verb::Classify,
        args.clone(),
    ));
    let objects = files(&store.join("cache/schemes/objects"));
    let matchers = files(&store.join("cache/vocab"));
    assert!(!objects.is_empty(), "compiled scheme saved");
    assert_eq!(matchers.len(), 1, "{matchers:?}");

    // A new service (a new process) reads them and decides the same.
    let second = data(call(
        &shelfmark_cli::wiring::open(&store),
        Verb::Classify,
        args.clone(),
    ));
    assert_eq!(first["placement"]["class"], second["placement"]["class"]);
    assert_eq!(
        first["placement"]["confidence"],
        second["placement"]["confidence"]
    );

    // A damaged cache is rebuilt, never an error.
    for f in objects.iter().chain(&matchers) {
        std::fs::write(f, b"garbage").unwrap();
    }
    let third = data(call(
        &shelfmark_cli::wiring::open(&store),
        Verb::Classify,
        args,
    ));
    assert_eq!(first["placement"]["class"], third["placement"]["class"]);
}

#[test]
fn an_absent_store_gets_no_cache() {
    let store = temp_dir("nocache").join("store");
    let s = shelfmark_cli::wiring::open(&store);
    data(call(&s, Verb::Status, json!({})));
    assert!(!store.exists());
}
