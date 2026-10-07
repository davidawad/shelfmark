//! Shared test helpers: a fresh store directory and request shorthands.

#![allow(dead_code)]

use std::path::PathBuf;
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

pub fn service(tag: &str) -> Shelfmark {
    shelfmark_cli::wiring::open(&temp_dir(tag).join("store"))
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
