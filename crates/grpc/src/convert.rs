//! Lossless mapping between the core shapes and the protobuf messages.
//!
//! JSON values travel as strings. serde_json (without `preserve_order`,
//! with `float_roundtrip`) re-serializes a parsed value byte-for-byte, so
//! an envelope that crosses the wire prints exactly as it did on the
//! server.

use shelfmark_core::envelope::{Mode, Request, Verb};
use shelfmark_core::{Envelope, Error, ErrorCode};

use crate::pb;

pub fn verb_to_pb(v: Verb) -> pb::Verb {
    match v {
        Verb::Describe => pb::Verb::Describe,
        Verb::Status => pb::Verb::Status,
        Verb::Resolve => pb::Verb::Resolve,
        Verb::Classify => pb::Verb::Classify,
        Verb::Explain => pb::Verb::Explain,
        Verb::ListFlagged => pb::Verb::ListFlagged,
        Verb::Confirm => pb::Verb::Confirm,
        Verb::Correct => pb::Verb::Correct,
        Verb::Stale => pb::Verb::Stale,
        Verb::SchemeCompile => pb::Verb::SchemeCompile,
        Verb::SchemeBrowse => pb::Verb::SchemeBrowse,
        Verb::SchemeSearch => pb::Verb::SchemeSearch,
        Verb::SchemeValidate => pb::Verb::SchemeValidate,
        Verb::Eval => pb::Verb::Eval,
        Verb::Link => pb::Verb::Link,
        Verb::Name => pb::Verb::Name,
    }
}

/// `None` for `VERB_UNSPECIFIED` and values this build does not know.
pub fn verb_from_pb(raw: i32) -> Option<Verb> {
    Some(match pb::Verb::try_from(raw).ok()? {
        pb::Verb::Unspecified => return None,
        pb::Verb::Describe => Verb::Describe,
        pb::Verb::Status => Verb::Status,
        pb::Verb::Resolve => Verb::Resolve,
        pb::Verb::Classify => Verb::Classify,
        pb::Verb::Explain => Verb::Explain,
        pb::Verb::ListFlagged => Verb::ListFlagged,
        pb::Verb::Confirm => Verb::Confirm,
        pb::Verb::Correct => Verb::Correct,
        pb::Verb::Stale => Verb::Stale,
        pb::Verb::SchemeCompile => Verb::SchemeCompile,
        pb::Verb::SchemeBrowse => Verb::SchemeBrowse,
        pb::Verb::SchemeSearch => Verb::SchemeSearch,
        pb::Verb::SchemeValidate => Verb::SchemeValidate,
        pb::Verb::Eval => Verb::Eval,
        pb::Verb::Link => Verb::Link,
        pb::Verb::Name => Verb::Name,
    })
}

pub fn code_to_pb(c: ErrorCode) -> pb::ErrorCode {
    match c {
        ErrorCode::InvalidRequest => pb::ErrorCode::InvalidRequest,
        ErrorCode::InvalidId => pb::ErrorCode::InvalidId,
        ErrorCode::NotFound => pb::ErrorCode::NotFound,
        ErrorCode::SchemeInvalid => pb::ErrorCode::SchemeInvalid,
        ErrorCode::SchemeNotCompiled => pb::ErrorCode::SchemeNotCompiled,
        ErrorCode::ProfileNotFound => pb::ErrorCode::ProfileNotFound,
        ErrorCode::TierUnavailable => pb::ErrorCode::TierUnavailable,
        ErrorCode::InvalidCode => pb::ErrorCode::InvalidCode,
        ErrorCode::RevisionConflict => pb::ErrorCode::RevisionConflict,
        ErrorCode::RevisionRequired => pb::ErrorCode::RevisionRequired,
        ErrorCode::StoreError => pb::ErrorCode::StoreError,
        ErrorCode::InputUnreadable => pb::ErrorCode::InputUnreadable,
        ErrorCode::Internal => pb::ErrorCode::Internal,
    }
}

/// `None` for `ERROR_CODE_UNSPECIFIED` and unknown values.
pub fn code_from_pb(raw: i32) -> Option<ErrorCode> {
    Some(match pb::ErrorCode::try_from(raw).ok()? {
        pb::ErrorCode::Unspecified => return None,
        pb::ErrorCode::InvalidRequest => ErrorCode::InvalidRequest,
        pb::ErrorCode::InvalidId => ErrorCode::InvalidId,
        pb::ErrorCode::NotFound => ErrorCode::NotFound,
        pb::ErrorCode::SchemeInvalid => ErrorCode::SchemeInvalid,
        pb::ErrorCode::SchemeNotCompiled => ErrorCode::SchemeNotCompiled,
        pb::ErrorCode::ProfileNotFound => ErrorCode::ProfileNotFound,
        pb::ErrorCode::TierUnavailable => ErrorCode::TierUnavailable,
        pb::ErrorCode::InvalidCode => ErrorCode::InvalidCode,
        pb::ErrorCode::RevisionConflict => ErrorCode::RevisionConflict,
        pb::ErrorCode::RevisionRequired => ErrorCode::RevisionRequired,
        pb::ErrorCode::StoreError => ErrorCode::StoreError,
        pb::ErrorCode::InputUnreadable => ErrorCode::InputUnreadable,
        pb::ErrorCode::Internal => ErrorCode::Internal,
    })
}

pub fn request_to_pb(r: &Request) -> pb::Request {
    pb::Request {
        verb: verb_to_pb(r.verb) as i32,
        args_json: r.args.to_string(),
        mode: match r.mode {
            Mode::Preview => pb::Mode::Preview,
            Mode::Commit => pb::Mode::Commit,
        } as i32,
        expected_revision: r.expected_revision.clone(),
    }
}

/// A malformed request becomes an `invalid_request` error, which the
/// server returns as an envelope, the same way the CLI would.
pub fn request_from_pb(r: pb::Request) -> Result<Request, Error> {
    let verb = verb_from_pb(r.verb).ok_or_else(|| {
        Error::new(
            ErrorCode::InvalidRequest,
            format!("unknown or unspecified verb {}", r.verb),
        )
        .next("shelfmark describe")
    })?;
    let mode = match pb::Mode::try_from(r.mode) {
        Ok(pb::Mode::Preview) => Mode::Preview,
        Ok(pb::Mode::Commit) => Mode::Commit,
        Err(_) => {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                format!("unknown mode {}", r.mode),
            ))
        }
    };
    // An empty string is proto3's default: no arguments.
    let args = if r.args_json.is_empty() {
        serde_json::Value::Object(Default::default())
    } else {
        serde_json::from_str(&r.args_json).map_err(|e| {
            Error::new(
                ErrorCode::InvalidRequest,
                format!("args_json is not valid JSON: {e}"),
            )
            .next(format!("shelfmark describe {}", verb.as_str()))
        })?
    };
    Ok(Request {
        verb,
        args,
        mode,
        expected_revision: r.expected_revision,
    })
}

pub fn envelope_to_pb(e: &Envelope) -> pb::Envelope {
    match e {
        Envelope::Ok { data, .. } => pb::Envelope {
            ok: true,
            data_json: data.to_string(),
            error: None,
        },
        Envelope::Err { error, .. } => pb::Envelope {
            ok: false,
            data_json: String::new(),
            error: Some(pb::Error {
                code: code_to_pb(error.code) as i32,
                message: error.message.clone(),
                next_command: error.next_command.clone(),
                details_json: error.details.as_ref().map(|d| d.to_string()),
            }),
        },
    }
}

/// The inverse of [`envelope_to_pb`]. A message no server of ours would
/// send (bad JSON, missing error, unknown code) decodes to an `internal`
/// error envelope rather than failing the call.
pub fn envelope_from_pb(e: pb::Envelope) -> Envelope {
    match decode_envelope(e) {
        Ok(env) => env,
        Err(msg) => Envelope::err(Error::internal(format!("malformed gRPC envelope: {msg}"))),
    }
}

fn decode_envelope(e: pb::Envelope) -> Result<Envelope, String> {
    if e.ok {
        let data = serde_json::from_str(&e.data_json).map_err(|err| format!("data_json: {err}"))?;
        return Ok(Envelope::Ok {
            ok: shelfmark_core::envelope::True,
            data,
        });
    }
    let pe = e.error.ok_or("ok is false but error is missing")?;
    let code = code_from_pb(pe.code).ok_or_else(|| format!("unknown error code {}", pe.code))?;
    let details = pe
        .details_json
        .map(|d| serde_json::from_str(&d))
        .transpose()
        .map_err(|err| format!("details_json: {err}"))?;
    Ok(Envelope::err(Error {
        code,
        message: pe.message,
        next_command: pe.next_command,
        details,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_verb_and_code_round_trips() {
        for v in Verb::ALL {
            assert_eq!(verb_from_pb(verb_to_pb(v) as i32), Some(v));
        }
        for c in ErrorCode::ALL {
            assert_eq!(code_from_pb(code_to_pb(c) as i32), Some(c));
        }
        assert_eq!(verb_from_pb(0), None);
        assert_eq!(code_from_pb(0), None);
        assert_eq!(verb_from_pb(999), None);
    }

    #[test]
    fn empty_args_json_is_an_empty_object() {
        let r = request_from_pb(pb::Request {
            verb: pb::Verb::Status as i32,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(r.args, serde_json::json!({}));
    }

    #[test]
    fn malformed_envelope_is_internal() {
        let env = envelope_from_pb(pb::Envelope {
            ok: false,
            data_json: String::new(),
            error: None,
        });
        match env {
            Envelope::Err { error, .. } => assert_eq!(error.code, ErrorCode::Internal),
            Envelope::Ok { .. } => panic!("expected an error"),
        }
    }
}
