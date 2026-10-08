//! The in-memory index over `placements.jsonl` that keeps every store
//! operation linear in the claims it touches, not in the size of the log.
//!
//! The index holds what lookups need (each claim's ID, what it supersedes,
//! its memo key, its (work, scheme) pair and where its line sits in the
//! file), never the claims themselves: a claim carries its full trace, so
//! the few that are needed are read back by offset. Each operation first
//! [`LogIndex::refresh`]es under the store lock: the log is append-only,
//! so only the bytes past the last parsed offset are read. A log that got
//! shorter (a repair cut a torn line) is parsed again from the start.

use std::collections::HashMap;
use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use shelfmark_core::error::{Error, ErrorCode};
use shelfmark_core::placement::{AuthorKind, Placement};
use shelfmark_core::{PlacementId, SchemeId, WorkId};

use crate::jsonl;

/// One claim's line in the log.
#[derive(Debug, Clone)]
pub(crate) struct Meta {
    pub id: PlacementId,
    pub human: bool,
    /// `InputHashes::memo_key` of its inputs.
    pub key: String,
    pub offset: u64,
    /// Length of the line without its newline.
    pub len: usize,
}

#[derive(Debug, Default)]
pub(crate) struct LogIndex {
    /// Bytes of the log parsed so far.
    pub bytes: u64,
    pub metas: Vec<Meta>,
    by_id: HashMap<PlacementId, usize>,
    /// Superseded claim ID to the first claim that superseded it.
    superseded_by: HashMap<PlacementId, usize>,
    by_pair: HashMap<(WorkId, SchemeId), Vec<usize>>,
    by_key: HashMap<String, Vec<usize>>,
    /// `memo.jsonl` as last seen: its fingerprint and highest `seq`.
    pub memo: Option<(Fingerprint, usize)>,
}

/// Enough of a file's metadata to tell an append from a replacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Fingerprint {
    pub len: u64,
    /// Inode on Unix (a rename-replaced file gets a new one), else 0.
    pub ino: u64,
}

impl Fingerprint {
    /// `None` when the file does not exist.
    pub fn of(path: &Path) -> Result<Option<Fingerprint>, Error> {
        match std::fs::metadata(path) {
            Ok(m) => Ok(Some(Fingerprint {
                len: m.len(),
                ino: ino(&m),
            })),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(jsonl::io_error(path, "stat", e)),
        }
    }
}

#[cfg(unix)]
fn ino(m: &std::fs::Metadata) -> u64 {
    std::os::unix::fs::MetadataExt::ino(m)
}

#[cfg(not(unix))]
fn ino(_: &std::fs::Metadata) -> u64 {
    0
}

/// `len` bytes of `path` from `offset`.
pub(crate) fn read_range(path: &Path, offset: u64, len: usize) -> Result<Vec<u8>, Error> {
    let mut f = File::open(path).map_err(|e| jsonl::io_error(path, "open", e))?;
    let mut buf = vec![0; len];
    f.seek(SeekFrom::Start(offset))
        .and_then(|_| f.read_exact(&mut buf))
        .map_err(|e| jsonl::io_error(path, "read", e))?;
    Ok(buf)
}

impl LogIndex {
    /// Catch up with the log at `path`. A torn final line is an error and
    /// is not consumed, so a later repair is picked up.
    pub fn refresh(&mut self, path: &Path) -> Result<(), Error> {
        let len = Fingerprint::of(path)?.map_or(0, |f| f.len);
        if len == self.bytes {
            return Ok(());
        }
        // Resume only where the parsed part still ends in a newline;
        // otherwise the file was rewritten, so start over.
        let resume =
            len > self.bytes && (self.bytes == 0 || read_range(path, self.bytes - 1, 1)? == b"\n");
        if !resume {
            self.reset();
        }
        let tail = read_range(path, self.bytes, (len - self.bytes) as usize)?;
        let complete = tail.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        if complete != tail.len() {
            return Err(jsonl::torn(
                path,
                self.bytes as usize + complete,
                tail.len() - complete,
            ));
        }
        if let Err(e) = self.absorb(path, &tail[..complete]) {
            // Half a tail is not an index: start over next time.
            self.reset();
            return Err(e);
        }
        Ok(())
    }

    fn reset(&mut self) {
        let memo = self.memo.take();
        *self = LogIndex::default();
        self.memo = memo;
    }

    /// Index whole lines that start at `self.bytes`.
    fn absorb(&mut self, path: &Path, lines: &[u8]) -> Result<(), Error> {
        for line in lines.split_inclusive(|&b| b == b'\n') {
            let body = &line[..line.len() - 1];
            if !body.is_empty() {
                let p: Placement = jsonl::parse(path, self.metas.len() + 1, body)?;
                self.push(&p, self.bytes, body.len());
            }
            self.bytes += line.len() as u64;
        }
        Ok(())
    }

    /// Record a claim whose line starts at `offset`. The caller advances
    /// `bytes` past it.
    pub fn push(&mut self, p: &Placement, offset: u64, len: usize) {
        let i = self.metas.len();
        self.by_id.entry(p.id.clone()).or_insert(i);
        if let Some(old) = &p.supersedes {
            self.superseded_by.entry(old.clone()).or_insert(i);
        }
        self.by_pair
            .entry((p.work.clone(), p.scheme.clone()))
            .or_default()
            .push(i);
        let key = p.inputs.memo_key();
        self.by_key.entry(key.clone()).or_default().push(i);
        self.metas.push(Meta {
            id: p.id.clone(),
            human: p.provenance.author_kind == AuthorKind::Human,
            key,
            offset,
            len,
        });
    }

    pub fn position(&self, id: &PlacementId) -> Option<usize> {
        self.by_id.get(id).copied()
    }

    /// Claims for one (work, scheme), in append order.
    pub fn pair(&self, work: &WorkId, scheme: &SchemeId) -> &[usize] {
        self.by_pair
            .get(&(work.clone(), scheme.clone()))
            .map_or(&[], Vec::as_slice)
    }

    /// Every memo key in the log.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.by_key.keys()
    }

    /// The memo answer for `key`: the latest live machine claim with
    /// those inputs (same rule as `PlacementLog::memo`).
    pub fn memo_answer(&self, key: &str) -> Option<usize> {
        self.by_key.get(key)?.iter().rev().copied().find(|&i| {
            let m = &self.metas[i];
            !m.human && !self.superseded_by.contains_key(&m.id)
        })
    }

    /// Claim `i`, read back from the log.
    pub fn read(&self, path: &Path, i: usize) -> Result<Placement, Error> {
        let m = &self.metas[i];
        jsonl::parse(path, i + 1, &read_range(path, m.offset, m.len)?)
    }

    /// A claim may only supersede a claim in this store that nothing has
    /// superseded yet.
    pub fn check_supersedes(&self, p: &Placement) -> Result<(), Error> {
        let Some(old) = &p.supersedes else {
            return Ok(());
        };
        if !self.by_id.contains_key(old) {
            return Err(Error::new(
                ErrorCode::NotFound,
                format!("{} supersedes {old}, which is not in this store", p.id),
            )
            .details(serde_json::json!({ "supersedes": old.to_string() })));
        }
        if let Some(&by) = self.superseded_by.get(old) {
            let by = &self.metas[by].id;
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                format!("{old} is already superseded by {by}; supersede the current claim instead"),
            )
            .details(serde_json::json!({
                "supersedes": old.to_string(),
                "superseded_by": by.to_string(),
            })));
        }
        Ok(())
    }
}
