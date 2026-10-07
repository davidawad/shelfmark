//! NDJSON batches (R19, PD8): one [`BatchLine`] per input, keyed by the
//! input's content hash, so a rerun with `resume` skips every input that
//! already has an `ok` line in the output.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde_json::{json, Map, Value};
use shelfmark_core::envelope::{completed_inputs, BatchLine};
use shelfmark_core::id::sha256_hex;
use shelfmark_core::placement::canonical_json;
use shelfmark_core::{Envelope, Error, ErrorCode, ItemId, Request, Service, Verb};

/// One unit of work: its key and the request it becomes.
pub struct Input {
    pub key: String,
    pub request: Result<Request, Error>,
}

/// A file to classify, keyed by its `item:` ID. `base` carries the
/// shared classify args (scheme, profile, target).
pub fn file_input(path: &str, base: &Map<String, Value>) -> Input {
    match fs::read(path) {
        Ok(bytes) => {
            let mut args = base.clone();
            args.insert("file".into(), json!(path));
            Input {
                key: ItemId::from_bytes(&bytes).to_string(),
                request: Ok(Request::new(Verb::Classify, Value::Object(args))),
            }
        }
        // No content, no hash: key by path so the line is still traceable;
        // an error line never counts as done.
        Err(e) => Input {
            key: format!("path:{path}"),
            request: Err(Error::new(
                ErrorCode::InputUnreadable,
                format!("cannot read {path}: {e}"),
            )
            .next("shelfmark describe classify")),
        },
    }
}

/// One NDJSON request line, keyed by the hash of its canonical JSON.
pub fn request_input(line: &str) -> Input {
    let parsed: Result<Value, _> = serde_json::from_str(line);
    let key = match &parsed {
        Ok(v) => sha256_hex(canonical_json(v).as_bytes()),
        Err(_) => sha256_hex(line.as_bytes()),
    };
    Input {
        key: format!("request:s256:{key}"),
        request: parsed
            .and_then(serde_json::from_value::<Request>)
            .map_err(|e| {
                Error::new(ErrorCode::InvalidRequest, format!("not a request: {e}"))
                    .next("shelfmark describe")
            }),
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Report {
    pub total: usize,
    pub skipped: usize,
    pub ok: usize,
    pub failed: usize,
    /// Exit code of the first failure, 0 when none failed.
    pub exit_code: i32,
}

/// Run `inputs` through `service`, writing one line each to `out`.
/// `done` holds keys to skip (the resume set); duplicates within the batch
/// run once.
pub fn run(
    service: &dyn Service,
    inputs: impl IntoIterator<Item = Input>,
    mut done: BTreeSet<String>,
    out: &mut dyn Write,
) -> Result<Report, Error> {
    let mut report = Report::default();
    for input in inputs {
        report.total += 1;
        if !done.insert(input.key.clone()) {
            report.skipped += 1;
            continue;
        }
        let envelope = match input.request {
            Ok(req) => service.call(req),
            Err(e) => Envelope::err(e),
        };
        if envelope.is_ok() {
            report.ok += 1;
        } else {
            report.failed += 1;
            if report.exit_code == 0 {
                report.exit_code = envelope.exit_code();
            }
        }
        let line = BatchLine {
            input: input.key,
            envelope,
        };
        let text = serde_json::to_string(&line).map_err(|e| Error::internal(e.to_string()))?;
        writeln!(out, "{text}").map_err(|e| {
            Error::new(
                ErrorCode::StoreError,
                format!("cannot write batch output: {e}"),
            )
            .next("shelfmark describe")
        })?;
    }
    Ok(report)
}

/// The resume set from an earlier output file (missing file: empty).
pub fn resume_set(path: &PathBuf) -> Result<BTreeSet<String>, Error> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(completed_inputs(&text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeSet::new()),
        Err(e) => Err(Error::new(
            ErrorCode::InputUnreadable,
            format!("cannot read {}: {e}", path.display()),
        )
        .next("shelfmark describe")),
    }
}
