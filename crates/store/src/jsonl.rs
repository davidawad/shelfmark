//! Line-oriented JSON files: strict reads that report a torn tail, and
//! single-write appends.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;
use shelfmark_core::error::{Error, ErrorCode};

pub(crate) fn io_error(path: &Path, what: &str, e: std::io::Error) -> Error {
    Error::new(
        ErrorCode::StoreError,
        format!("could not {what} {}: {e}", path.display()),
    )
    .details(serde_json::json!({ "file": path.display().to_string() }))
}

/// The whole file, or nothing when it does not exist yet.
pub(crate) fn read_bytes(path: &Path) -> Result<Vec<u8>, Error> {
    match std::fs::read(path) {
        Ok(b) => Ok(b),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(io_error(path, "read", e)),
    }
}

/// Every complete line, without its newline. A file that does not end in
/// a newline holds a torn final line (an append that did not finish):
/// that is an error, never silently dropped.
pub(crate) fn lines<'a>(path: &Path, bytes: &'a [u8]) -> Result<Vec<&'a [u8]>, Error> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let complete = match bytes.iter().rposition(|&b| b == b'\n') {
        Some(i) => i + 1,
        None => 0,
    };
    if complete != bytes.len() {
        return Err(torn(path, complete, bytes.len() - complete));
    }
    Ok(bytes[..complete - 1]
        .split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
        .collect())
}

pub(crate) fn torn(path: &Path, offset: usize, len: usize) -> Error {
    Error::new(
        ErrorCode::StoreError,
        format!(
            "{} ends in a torn line ({len} bytes at offset {offset} with no newline): \
             an append did not finish; repair the store to keep or cut it",
            path.display()
        ),
    )
    .details(serde_json::json!({
        "file": path.display().to_string(),
        "reason": "torn_final_line",
        "offset": offset,
        "torn_bytes": len,
    }))
}

/// Parse every line; a line that is not a valid record is an error naming
/// its line number.
pub(crate) fn read<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>, Error> {
    let bytes = read_bytes(path)?;
    lines(path, &bytes)?
        .into_iter()
        .enumerate()
        .map(|(i, line)| parse(path, i + 1, line))
        .collect()
}

pub(crate) fn parse<T: DeserializeOwned>(path: &Path, n: usize, line: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(line).map_err(|e| {
        Error::new(
            ErrorCode::StoreError,
            format!("{} line {n} is not a valid record: {e}", path.display()),
        )
        .details(serde_json::json!({
            "file": path.display().to_string(),
            "reason": "invalid_line",
            "line": n,
        }))
    })
}

pub(crate) fn encode<T: Serialize>(record: &T) -> Result<Vec<u8>, Error> {
    let mut line = serde_json::to_vec(record)
        .map_err(|e| Error::internal(format!("record does not serialize: {e}")))?;
    line.push(b'\n');
    Ok(line)
}

/// Append records in one write and flush them to disk. Returns the
/// number of bytes written.
pub(crate) fn append<T: Serialize>(path: &Path, records: &[T]) -> Result<u64, Error> {
    let mut buf = Vec::new();
    for r in records {
        buf.extend(encode(r)?);
    }
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| io_error(path, "open", e))?;
    f.write_all(&buf)
        .and_then(|()| f.sync_data())
        .map_err(|e| io_error(path, "append to", e))?;
    Ok(buf.len() as u64)
}

/// Replace a whole file atomically (write a sibling, then rename).
pub(crate) fn replace<T: Serialize>(path: &Path, records: &[T]) -> Result<(), Error> {
    let tmp = path.with_extension("jsonl.tmp");
    let mut buf = Vec::new();
    for r in records {
        buf.extend(encode(r)?);
    }
    let mut f = File::create(&tmp).map_err(|e| io_error(&tmp, "create", e))?;
    f.write_all(&buf)
        .and_then(|()| f.sync_data())
        .map_err(|e| io_error(&tmp, "write", e))?;
    std::fs::rename(&tmp, path).map_err(|e| io_error(path, "replace", e))
}

/// What [`repair_tail`] did to a file's torn final line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum TailRepair {
    /// The file was clean.
    Clean,
    /// The tail was a complete record missing only its newline; kept.
    Terminated,
    /// The tail was not a complete record; these bytes were cut.
    Truncated { offset: usize, bytes: String },
}

/// Fix a torn final line: keep it when it parses as `T`, cut it otherwise.
pub(crate) fn repair_tail<T: DeserializeOwned>(path: &Path) -> Result<TailRepair, Error> {
    let bytes = read_bytes(path)?;
    let complete = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
    if complete == bytes.len() {
        return Ok(TailRepair::Clean);
    }
    let tail = &bytes[complete..];
    if serde_json::from_slice::<T>(tail).is_ok() {
        let mut f = OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| io_error(path, "open", e))?;
        f.write_all(b"\n")
            .and_then(|()| f.sync_data())
            .map_err(|e| io_error(path, "terminate", e))?;
        return Ok(TailRepair::Terminated);
    }
    let f = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| io_error(path, "open", e))?;
    f.set_len(complete as u64)
        .and_then(|()| f.sync_data())
        .map_err(|e| io_error(path, "truncate", e))?;
    Ok(TailRepair::Truncated {
        offset: complete,
        bytes: String::from_utf8_lossy(tail).into_owned(),
    })
}
