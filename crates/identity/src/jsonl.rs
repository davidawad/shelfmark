//! An append-only JSONL file with a chained revision. Lines are never
//! rewritten. Appends happen only while the caller holds the store lock
//! ([`crate::lock::with_store_lock`]), and each append first reads any
//! lines another writer added since this handle last looked.

use std::fs::{File, OpenOptions};
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;
use shelfmark_core::id::sha256_hex;
use shelfmark_core::{Error, ErrorCode};

use crate::lock::store_error;

/// The revision of an empty log.
const EMPTY_REVISION: &str = "s256:empty";

#[derive(Debug)]
pub(crate) struct JsonlLog<T> {
    path: PathBuf,
    /// Bytes of the file already parsed.
    len: u64,
    /// Lines parsed so far, which is the line number of the next one.
    lines: usize,
    revision: String,
    _record: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> JsonlLog<T> {
    /// Open `path` (absent means empty) and return every record in order.
    pub(crate) fn open(path: PathBuf) -> Result<(Self, Vec<T>), Error> {
        let mut log = JsonlLog {
            path,
            len: 0,
            lines: 0,
            revision: EMPTY_REVISION.to_string(),
            _record: PhantomData,
        };
        let records = log.catch_up()?;
        Ok((log, records))
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Opaque revision: changes on every append. Each line extends a hash
    /// chain, so equal revisions mean equal logs.
    pub(crate) fn revision(&self) -> &str {
        &self.revision
    }

    /// Parse records appended since the last look.
    pub(crate) fn catch_up(&mut self) -> Result<Vec<T>, Error> {
        let mut file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && self.len == 0 => {
                return Ok(Vec::new())
            }
            Err(e) => return Err(store_error("open", &self.path, &e)),
        };
        let mut buf = String::new();
        file.seek(SeekFrom::Start(self.len))
            .and_then(|_| file.read_to_string(&mut buf))
            .map_err(|e| store_error("read", &self.path, &e))?;
        // Only whole lines count: a line still being written by a crashed
        // writer is not consumed.
        let whole = buf.rfind('\n').map_or(0, |i| i + 1);
        let mut out = Vec::new();
        for line in buf[..whole].split_terminator('\n') {
            self.lines += 1;
            if line.trim().is_empty() {
                continue;
            }
            let record = serde_json::from_str(line).map_err(|e| {
                Error::new(
                    ErrorCode::StoreError,
                    format!("{} line {}: {e}", self.path.display(), self.lines),
                )
                .details(serde_json::json!({
                    "path": self.path.display().to_string(),
                    "line": self.lines,
                }))
            })?;
            self.chain(line);
            out.push(record);
        }
        self.len += whole as u64;
        Ok(out)
    }

    fn chain(&mut self, line: &str) {
        let input = format!("{}\n{line}", self.revision);
        self.revision = format!("s256:{}", sha256_hex(input.as_bytes()));
    }

    /// Append one record. Call only while holding the store lock, after
    /// [`JsonlLog::catch_up`].
    pub(crate) fn append_locked(&mut self, record: &T) -> Result<(), Error> {
        let line = serde_json::to_string(record)
            .map_err(|e| Error::internal(format!("serialize record: {e}")))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| store_error("open", &self.path, &e))?;
        let end = file
            .seek(SeekFrom::End(0))
            .map_err(|e| store_error("seek", &self.path, &e))?;
        if end > self.len {
            // A torn last line from a crashed writer. Appending after it
            // would make the file unreadable, so stop and say where.
            return Err(Error::new(
                ErrorCode::StoreError,
                format!(
                    "{} ends with an incomplete line at byte {}; remove it and retry",
                    self.path.display(),
                    self.len
                ),
            )
            .details(serde_json::json!({
                "path": self.path.display().to_string(),
                "offset": self.len,
            })));
        }
        file.write_all(format!("{line}\n").as_bytes())
            .and_then(|_| file.sync_data())
            .map_err(|e| store_error("append to", &self.path, &e))?;
        self.lines += 1;
        self.len = end + line.len() as u64 + 1;
        self.chain(&line);
        Ok(())
    }
}
