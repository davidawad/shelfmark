//! The one request/response shape every surface shares (R19, plan PD8).
//!
//! The CLI prints an [`Envelope`]; gRPC carries the same JSON; the `kb`
//! core resource wraps it. Batches are NDJSON: one [`BatchLine`] per input,
//! keyed by the input's content hash so a rerun can resume.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::error::Error;

/// The agent verbs. `describe` with no arguments lists them all (R20).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verb {
    Describe,
    Status,
    Resolve,
    Classify,
    Explain,
    ListFlagged,
    Confirm,
    Correct,
    Stale,
    SchemeCompile,
    SchemeBrowse,
    SchemeSearch,
    SchemeValidate,
    Eval,
    Link,
    Name,
}

impl Verb {
    pub const ALL: [Verb; 16] = [
        Verb::Describe,
        Verb::Status,
        Verb::Resolve,
        Verb::Classify,
        Verb::Explain,
        Verb::ListFlagged,
        Verb::Confirm,
        Verb::Correct,
        Verb::Stale,
        Verb::SchemeCompile,
        Verb::SchemeBrowse,
        Verb::SchemeSearch,
        Verb::SchemeValidate,
        Verb::Eval,
        Verb::Link,
        Verb::Name,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Verb::Describe => "describe",
            Verb::Status => "status",
            Verb::Resolve => "resolve",
            Verb::Classify => "classify",
            Verb::Explain => "explain",
            Verb::ListFlagged => "list-flagged",
            Verb::Confirm => "confirm",
            Verb::Correct => "correct",
            Verb::Stale => "stale",
            Verb::SchemeCompile => "scheme-compile",
            Verb::SchemeBrowse => "scheme-browse",
            Verb::SchemeSearch => "scheme-search",
            Verb::SchemeValidate => "scheme-validate",
            Verb::Eval => "eval",
            Verb::Link => "link",
            Verb::Name => "name",
        }
    }

    /// Mutating verbs preview by default and commit only with
    /// `mode: commit` plus `expected_revision` (`PlacementLog::revision`).
    /// `classify` is not one: it appends machine claims as an idempotent,
    /// memoized cache and never moves files or changes a confirmed claim.
    /// `eval` writes only its own summary and calibration. `name` previews
    /// a path; moving files is the kb resource's job.
    pub fn mutates(self) -> bool {
        matches!(
            self,
            Verb::Confirm | Verb::Correct | Verb::Link | Verb::SchemeCompile
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Preview,
    Commit,
}

/// One request to the service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub verb: Verb,
    /// Verb-specific arguments as a JSON object.
    #[serde(default = "empty_object")]
    pub args: serde_json::Value,
    #[serde(default)]
    pub mode: Mode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_revision: Option<String>,
}

fn empty_object() -> serde_json::Value {
    serde_json::Value::Object(Default::default())
}

impl Request {
    pub fn new(verb: Verb, args: serde_json::Value) -> Self {
        Request {
            verb,
            args,
            mode: Mode::Preview,
            expected_revision: None,
        }
    }
}

/// `{"ok": true, "data": ...}` or `{"ok": false, "error": {...}}`, the
/// same envelope as the core-resource contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Envelope {
    Ok { ok: True, data: serde_json::Value },
    Err { ok: False, error: Error },
}

/// Serde helper for the literal `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct True;
/// Serde helper for the literal `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct False;

macro_rules! bool_literal {
    ($ty:ident, $v:literal) => {
        impl Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_bool($v)
            }
        }
        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                if bool::deserialize(d)? == $v {
                    Ok($ty)
                } else {
                    Err(serde::de::Error::custom(concat!(
                        "expected ",
                        stringify!($v)
                    )))
                }
            }
        }
    };
}

bool_literal!(True, true);
bool_literal!(False, false);

impl Envelope {
    pub fn ok(data: impl Serialize) -> Self {
        match serde_json::to_value(data) {
            Ok(data) => Envelope::Ok { ok: True, data },
            Err(e) => Envelope::err(Error::internal(format!("response did not serialize: {e}"))),
        }
    }

    pub fn err(error: Error) -> Self {
        Envelope::Err { ok: False, error }
    }

    pub fn from_result<T: Serialize>(r: Result<T, Error>) -> Self {
        match r {
            Ok(v) => Envelope::ok(v),
            Err(e) => Envelope::err(e),
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Envelope::Ok { .. })
    }

    /// Process exit code for the CLI: 0, or the error code's exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            Envelope::Ok { .. } => 0,
            Envelope::Err { error, .. } => error.code.exit_code(),
        }
    }
}

/// The operations surface: CLI, gRPC and the kb resource all call this.
pub trait Service: Send + Sync {
    fn call(&self, request: Request) -> Envelope;
}

/// One NDJSON batch line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchLine {
    /// Content hash of the input (an `item:` ID for files).
    pub input: String,
    #[serde(flatten)]
    pub envelope: Envelope,
}

/// Inputs already done in an earlier run's output: the resume set.
/// Lines that fail to parse or carry an error are not done.
pub fn completed_inputs(ndjson: &str) -> BTreeSet<String> {
    ndjson
        .lines()
        .filter_map(|l| serde_json::from_str::<BatchLine>(l).ok())
        .filter(|b| b.envelope.is_ok())
        .map(|b| b.input)
        .collect()
}
