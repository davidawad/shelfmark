//! The memo index (R14, PD6): `memo.jsonl`, derived from the claim log
//! and rebuildable from it at any time.
//!
//! Each line is the memo answer for one key as of a log length (`seq`,
//! the number of claims in the log when it was written); the last line per
//! key wins. An index whose highest `seq` is not the log's length is
//! behind (a crash between the two appends, or a deleted file) and is
//! never trusted: lookups fall back to the log and the next append
//! rebuilds it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use shelfmark_core::error::Error;
use shelfmark_core::placement::{superseded_ids, AuthorKind, Placement};
use shelfmark_core::PlacementId;

use crate::jsonl;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Entry {
    pub seq: usize,
    pub key: String,
    /// The memo hit for `key`, or `None` once no live machine claim has it.
    pub placement: Option<Placement>,
}

/// The memo answer for `key`: the latest live machine claim with those
/// inputs (same rule as `PlacementLog::memo`).
fn answer<'a>(
    claims: &'a [Placement],
    superseded: &BTreeSet<PlacementId>,
    key: &str,
) -> Option<&'a Placement> {
    claims.iter().rev().find(|p| {
        p.provenance.author_kind != AuthorKind::Human
            && !superseded.contains(&p.id)
            && p.inputs.memo_key() == key
    })
}

/// The memo answer for `key` computed from the log alone.
pub(crate) fn lookup(claims: &[Placement], key: &str) -> Option<Placement> {
    answer(claims, &superseded_ids(claims), key).cloned()
}

/// Entries for the keys an append of `claims.last()` can change: its own
/// key and the key of the claim it supersedes.
pub(crate) fn entries_after_append(claims: &[Placement]) -> Vec<Entry> {
    let Some(new) = claims.last() else {
        return Vec::new();
    };
    let mut keys = vec![new.inputs.memo_key()];
    if let Some(old) = &new.supersedes {
        if let Some(old) = claims.iter().find(|p| &p.id == old) {
            keys.push(old.inputs.memo_key());
        }
    }
    keys.dedup();
    entries(claims, keys)
}

fn entries(claims: &[Placement], keys: impl IntoIterator<Item = String>) -> Vec<Entry> {
    let superseded = superseded_ids(claims);
    keys.into_iter()
        .map(|key| Entry {
            seq: claims.len(),
            placement: answer(claims, &superseded, &key).cloned(),
            key,
        })
        .collect()
}

/// The full index rebuilt from the log: a marker recording the log length
/// (so an index with no hits still reads as current), then one entry per
/// key with a hit.
pub(crate) fn rebuild(claims: &[Placement]) -> Vec<Entry> {
    let keys: BTreeSet<String> = claims.iter().map(|p| p.inputs.memo_key()).collect();
    let marker = Entry {
        seq: claims.len(),
        key: String::new(),
        placement: None,
    };
    std::iter::once(marker)
        .chain(
            entries(claims, keys)
                .into_iter()
                .filter(|e| e.placement.is_some()),
        )
        .collect()
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
    let seq = entries.iter().map(|e| e.seq).max().unwrap_or(0);
    (seq == log_len).then(|| fold(entries))
}

pub(crate) fn write_rebuilt(path: &Path, claims: &[Placement]) -> Result<(), Error> {
    jsonl::replace(path, &rebuild(claims))
}
