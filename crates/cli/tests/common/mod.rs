//! Shared test helpers: a fresh store directory and request shorthands.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;
use shelfmark_cli::Shelfmark;
use shelfmark_core::envelope::Mode;
use shelfmark_core::{Envelope, Error, Request, Service, Verb};

/// A new, empty directory under the system temp dir.
pub fn temp_dir(tag: &str) -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "shelfmark-cli-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Compile the five-class mini-LCC fixture into `store`, so it is the
/// default scheme there (store schemes list before the bundled LCC
/// outline). These tests exercise the service's mechanics, and a scheme
/// this small keeps every expected class and score readable; the
/// acceptance suite runs against the bundled outline.
pub fn seed_mini_lcc(store: &Path) {
    let schemes = store.join("schemes");
    std::fs::create_dir_all(&schemes).unwrap();
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/schemes/mini-lcc.json"
        ),
        schemes.join("mini-lcc@2024.json"),
    )
    .unwrap();
}

/// A service over a fresh store seeded with the fixture identifier table
/// and the mini-LCC scheme.
pub fn service(tag: &str) -> Shelfmark {
    let store = temp_dir(tag).join("store");
    seed_mini_lcc(&store);
    let tables = store.join(shelfmark_cli::wiring::TABLES_DIR);
    std::fs::create_dir_all(&tables).unwrap();
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/tables/identifiers.jsonl"
        ),
        tables.join("identifiers.jsonl"),
    )
    .unwrap();
    shelfmark_cli::wiring::open(&store)
}

pub fn call(s: &Shelfmark, verb: Verb, args: Value) -> Envelope {
    s.call(Request::new(verb, args))
}

pub fn commit(s: &Shelfmark, verb: Verb, args: Value, revision: Option<&str>) -> Envelope {
    s.call(Request {
        verb,
        args,
        mode: Mode::Commit,
        expected_revision: revision.map(str::to_string),
    })
}

pub fn data(e: Envelope) -> Value {
    match e {
        Envelope::Ok { data, .. } => data,
        Envelope::Err { error, .. } => panic!("expected ok, got {error:?}"),
    }
}

pub fn error(e: Envelope) -> Error {
    match e {
        Envelope::Err { error, .. } => error,
        Envelope::Ok { data, .. } => panic!("expected an error, got {data}"),
    }
}
