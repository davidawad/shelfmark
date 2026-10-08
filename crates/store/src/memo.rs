//! The memo index (R14, PD6): `memo.jsonl`, derived from the claim log
//! and rebuildable from it at any time.
//!
//! Each line is the memo answer for one key as of a log length (`seq`,
//! the number of claims in the log when it was written); the last line per
//! key wins. An index whose highest `seq` is not the log's length is
//! behind (a crash between the two appends, or a deleted file) and is
//! never trusted: the next append rebuilds it. Lookups are answered from
//! the in-memory index over the log ([`crate::index`]), so a behind file
//! never answers one.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use shelfmark_core::error::Error;
use shelfmark_core::placement::Placement;

use crate::index::{read_range, Fingerprint, LogIndex};
use crate::jsonl;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Entry {
    pub seq: usize,
    pub key: String,
    /// The memo hit for `key`, or `None` once no live machine claim has it.
    pub placement: Option<Placement>,
}

/// Entries for the keys an append of `new` (already in `ix`) can change:
/// its own key and the key of the claim it supersedes.
pub(crate) fn entries_after_append(
    ix: &LogIndex,
    log: &Path,
    new: &Placement,
) -> Result<Vec<Entry>, Error> {
    let last = ix.metas.len() - 1;
    let mut keys = vec![ix.metas[last].key.clone()];
    if let Some(old) = new.supersedes.as_ref().and_then(|o| ix.position(o)) {
        keys.push(ix.metas[old].key.clone());
    }
    keys.dedup();
    keys.into_iter()
        .map(|key| {
            let placement = match ix.memo_answer(&key) {
                Some(i) if i == last => Some(new.clone()),
                Some(i) => Some(ix.read(log, i)?),
                None => None,
            };
            Ok(Entry {
                seq: ix.metas.len(),
                key,
                placement,
            })
        })
        .collect()
}

/// The full index rebuilt from the log: a marker recording the log length
/// (so an index with no hits still reads as current), then one entry per
/// key with a hit.
pub(crate) fn rebuild(ix: &LogIndex, log: &Path) -> Result<Vec<Entry>, Error> {
    let bytes = jsonl::read_bytes(log)?;
    let keys: BTreeSet<&String> = ix.keys().collect();
    let seq = ix.metas.len();
    let mut out = vec![Entry {
        seq,
        key: String::new(),
        placement: None,
    }];
    for key in keys {
        let Some(i) = ix.memo_answer(key) else {
            continue;
        };
        let m = &ix.metas[i];
        let start = m.offset as usize;
        let line = bytes.get(start..start + m.len).ok_or_else(|| {
            Error::internal(format!("{} is shorter than its index", log.display()))
        })?;
        out.push(Entry {
            seq,
            key: key.clone(),
            placement: Some(jsonl::parse(log, i + 1, line)?),
        });
    }
    Ok(out)
}

/// Key to memo hit, as the index on disk says.
pub type MemoMap = BTreeMap<String, Placement>;

/// Fold entries into the map they describe.
pub(crate) fn fold(entries: Vec<Entry>) -> MemoMap {
    let mut map = MemoMap::new();
    for e in entries {
        match e.placement {
            Some(p) => map.insert(e.key, p),
            None => map.remove(&e.key),
        };
    }
    map
}

/// The index on disk if it is current for a log of `log_len` claims;
/// `None` when it is missing, behind or unreadable (it is derived, so the
/// log stays authoritative).
pub(crate) fn load(path: &Path, log_len: usize) -> Option<MemoMap> {
    let entries: Vec<Entry> = jsonl::read(path).ok()?;
    (max_seq(&entries) == log_len).then(|| fold(entries))
}

fn max_seq(entries: &[Entry]) -> usize {
    entries.iter().map(|e| e.seq).max().unwrap_or(0)
}

/// The file's fingerprint and highest `seq`, or `None` when it is missing
/// or unreadable. When `seen` is an earlier answer for the same file, only
/// the lines appended since are read.
pub(crate) fn seq(
    path: &Path,
    seen: Option<(Fingerprint, usize)>,
) -> Result<Option<(Fingerprint, usize)>, Error> {
    let Some(now) = Fingerprint::of(path)? else {
        return Ok(None);
    };
    if let Some((was, seq)) = seen.filter(|(was, _)| was.ino == now.ino && was.len <= now.len) {
        if was.len == now.len {
            return Ok(Some((now, seq)));
        }
        if let Some(tail) = appended(path, was.len, now.len) {
            return Ok(Some((now, seq.max(max_seq(&tail)))));
        }
    }
    Ok(jsonl::read::<Entry>(path)
        .ok()
        .map(|entries| (now, max_seq(&entries))))
}

/// The whole entries between `from` (just past a newline) and `to`.
fn appended(path: &Path, from: u64, to: u64) -> Option<Vec<Entry>> {
    let start = from.checked_sub(1)?;
    let bytes = read_range(path, start, (to - start) as usize).ok()?;
    let (first, rest) = bytes.split_first()?;
    if *first != b'\n' || rest.last() != Some(&b'\n') {
        return None;
    }
    rest[..rest.len() - 1]
        .split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_slice(l).ok())
        .collect()
}

pub(crate) fn write_rebuilt(path: &Path, ix: &LogIndex, log: &Path) -> Result<(), Error> {
    jsonl::replace(path, &rebuild(ix, log)?)
}
