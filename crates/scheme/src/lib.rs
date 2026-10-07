//! The shelfmark scheme compiler (plan U1; R4-R6, AE7).
//!
//! A scheme is one JSON file. [`validate_bytes`] checks it against the
//! published schema and then the rules a schema cannot state (code
//! grammar, parents, ranges), reporting every problem with its JSON
//! pointer. [`compile_bytes`] turns a valid file into a [`SchemeIndex`],
//! the crate's [`CompiledIndex`](shelfmark_core::scheme::CompiledIndex).
//! [`CompileCache`] keeps compiled indexes on disk keyed by the source
//! file's hash; the on-disk form is private (PD7).

mod cache;
mod code;
mod index;
mod search;
mod validate;

use std::path::Path;

use shelfmark_core::id::sha256_hex;
use shelfmark_core::scheme::SchemeDoc;
use shelfmark_core::{Error, ErrorCode};

pub use cache::{CacheStatus, CompileCache, Compiled};
pub use index::SchemeIndex;
pub use validate::{validate_bytes, validate_value, Diagnostic, SCHEME_SCHEMA};

/// `s256:<hex>` of a scheme file's bytes: its `content_hash`.
pub fn content_hash(bytes: &[u8]) -> String {
    format!("s256:{}", sha256_hex(bytes))
}

/// The `scheme_invalid` error for a file's problems, with every problem in
/// `details.errors` as `{pointer, message}`.
pub fn scheme_invalid(source: &str, problems: &[Diagnostic]) -> Error {
    let first = problems
        .first()
        .map(|d| format!("; first at `{}`: {}", d.pointer, d.message))
        .unwrap_or_default();
    Error::new(
        ErrorCode::SchemeInvalid,
        format!("{source} has {} problem(s){first}", problems.len()),
    )
    .next(format!("shelfmark scheme validate {source}"))
    .details(serde_json::json!({ "source": source, "errors": problems }))
}

/// Read a scheme file, or `input_unreadable`.
pub fn read(path: &Path) -> Result<Vec<u8>, Error> {
    std::fs::read(path).map_err(|e| {
        Error::new(
            ErrorCode::InputUnreadable,
            format!("cannot read scheme file {}: {e}", path.display()),
        )
    })
}

/// Validate a scheme file; `source` names it in errors.
pub fn validate_source(bytes: &[u8], source: &str) -> Result<SchemeDoc, Error> {
    validate_bytes(bytes).map_err(|p| scheme_invalid(source, &p))
}

/// Validate and compile scheme bytes; `source` names the file in errors.
pub fn compile_bytes(bytes: &[u8], source: &str) -> Result<SchemeIndex, Error> {
    let doc = validate_source(bytes, source)?;
    SchemeIndex::from_stored(index::Stored::build(&doc, content_hash(bytes)))
}

/// Validate and compile a scheme file, without the cache.
pub fn compile_file(path: &Path) -> Result<SchemeIndex, Error> {
    compile_bytes(&read(path)?, &path.display().to_string())
}
