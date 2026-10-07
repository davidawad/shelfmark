//! Stable error codes (R21). Every error carries a code that never
//! changes meaning, a plain explanation, and the next command that would
//! fix it. `proto/shelfmark/v1/shelfmark.proto` lists the same codes; the
//! `proto_sync` test keeps them aligned.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A stable error code. Serialized as its snake_case string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Request shape is wrong (unknown verb, bad or missing argument).
    InvalidRequest,
    /// A string is not a valid typed ID.
    InvalidId,
    /// A well-formed ID that resolves to nothing.
    NotFound,
    /// The scheme file does not validate against the scheme schema.
    SchemeInvalid,
    /// The scheme is known but has not been compiled (AE7).
    SchemeNotCompiled,
    /// No profile with that ID.
    ProfileNotFound,
    /// The profile needs a tier this build or machine cannot run.
    TierUnavailable,
    /// A class code fails the scheme's grammar.
    InvalidCode,
    /// `expected_revision` no longer matches (preview/commit).
    RevisionConflict,
    /// A mutation in commit mode without `expected_revision`.
    RevisionRequired,
    /// The store could not be read or written.
    StoreError,
    /// Input file could not be read or its text extracted.
    InputUnreadable,
    /// A bug: an invariant failed.
    Internal,
}

impl ErrorCode {
    pub const ALL: [ErrorCode; 13] = [
        ErrorCode::InvalidRequest,
        ErrorCode::InvalidId,
        ErrorCode::NotFound,
        ErrorCode::SchemeInvalid,
        ErrorCode::SchemeNotCompiled,
        ErrorCode::ProfileNotFound,
        ErrorCode::TierUnavailable,
        ErrorCode::InvalidCode,
        ErrorCode::RevisionConflict,
        ErrorCode::RevisionRequired,
        ErrorCode::StoreError,
        ErrorCode::InputUnreadable,
        ErrorCode::Internal,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::InvalidRequest => "invalid_request",
            ErrorCode::InvalidId => "invalid_id",
            ErrorCode::NotFound => "not_found",
            ErrorCode::SchemeInvalid => "scheme_invalid",
            ErrorCode::SchemeNotCompiled => "scheme_not_compiled",
            ErrorCode::ProfileNotFound => "profile_not_found",
            ErrorCode::TierUnavailable => "tier_unavailable",
            ErrorCode::InvalidCode => "invalid_code",
            ErrorCode::RevisionConflict => "revision_conflict",
            ErrorCode::RevisionRequired => "revision_required",
            ErrorCode::StoreError => "store_error",
            ErrorCode::InputUnreadable => "input_unreadable",
            ErrorCode::Internal => "internal",
        }
    }

    /// CLI exit code. 0 is success; 1 stays free for "ran, with flagged
    /// results" so scripts can tell a low-confidence run from a failure.
    pub fn exit_code(self) -> i32 {
        match self {
            ErrorCode::InvalidRequest | ErrorCode::InvalidId | ErrorCode::InvalidCode => 2,
            ErrorCode::NotFound | ErrorCode::ProfileNotFound => 3,
            ErrorCode::SchemeInvalid | ErrorCode::SchemeNotCompiled => 4,
            ErrorCode::TierUnavailable => 5,
            ErrorCode::RevisionConflict | ErrorCode::RevisionRequired => 6,
            ErrorCode::StoreError | ErrorCode::InputUnreadable => 7,
            ErrorCode::Internal => 70,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The error body every surface returns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
#[error("{code}: {message}")]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    /// The command that would fix this, runnable as given (R21).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl Error {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Error {
            code,
            message: message.into(),
            next_command: None,
            details: None,
        }
    }

    pub fn next(mut self, command: impl Into<String>) -> Self {
        self.next_command = Some(command.into());
        self
    }

    pub fn details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// AE7: a scheme that is known but not compiled names the compile
    /// command.
    pub fn scheme_not_compiled(scheme: &crate::id::SchemeId, source: Option<&str>) -> Self {
        let cmd = match source {
            Some(path) => format!("shelfmark scheme compile {path}"),
            None => format!("shelfmark scheme compile <file defining {scheme}>"),
        };
        Error::new(
            ErrorCode::SchemeNotCompiled,
            format!("{scheme} is not compiled, so nothing can be classified into it"),
        )
        .next(cmd)
        .details(serde_json::json!({ "scheme": scheme.to_string() }))
    }

    pub fn invalid_id(err: &crate::id::IdError) -> Self {
        Error::new(ErrorCode::InvalidId, err.to_string()).next("shelfmark describe ids")
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::Internal, message)
    }
}

impl From<crate::id::IdError> for Error {
    fn from(e: crate::id::IdError) -> Self {
        Error::invalid_id(&e)
    }
}
