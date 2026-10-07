//! The shelfmark placement store (plan U6, PD6; R11, R12, R14, R15).
//!
//! A store is a directory:
//!
//! - `placements.jsonl`: the append-only claim log behind
//!   [`PlacementLog`]. Claims are never rewritten: a correction or re-run
//!   appends a claim whose `supersedes` names the old one.
//! - `gold.jsonl`: the append-only gold set ([`GoldExample`]s).
//! - `memo.jsonl`: the memo index keyed by `InputHashes::memo_key`,
//!   derived from the log and rebuildable at any time.
//! - `.lock`: every append holds an exclusive advisory lock on it; reads
//!   hold a shared one, so they never see half an append.
//!
//! A torn final line (an append that did not finish) is an error on every
//! read and append until [`Store::repair`] keeps or cuts it.
//!
//! [`GoldExample`]: shelfmark_core::calibration::GoldExample

mod gold;
mod jsonl;
mod memo;
mod stale;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use fs2::FileExt;
use serde::Serialize;
use shelfmark_core::calibration::GoldExample;
use shelfmark_core::error::{Error, ErrorCode};
use shelfmark_core::placement::{self, InputHashes, Placement, PlacementLog};
use shelfmark_core::{SchemeId, WorkId};

pub use jsonl::TailRepair;
pub use memo::MemoMap;
pub use stale::{stale, StaleClaim, StaleQuery, StaleReason};

pub const PLACEMENTS: &str = "placements.jsonl";
pub const GOLD: &str = "gold.jsonl";
pub const MEMO: &str = "memo.jsonl";
pub const LOCK: &str = ".lock";
/// Store directory when no `--store` is given.
pub const ENV_VAR: &str = "SHELFMARK_STORE";
/// Store directory when neither `--store` nor [`ENV_VAR`] is set.
pub const DEFAULT_DIR: &str = ".shelfmark";

/// Where a store lives: `--store`, else `$SHELFMARK_STORE`, else
/// `./.shelfmark`. An empty value counts as unset.
pub fn resolve_path(flag: Option<&Path>, env: Option<OsString>) -> PathBuf {
    flag.filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .or_else(|| env.filter(|v| !v.is_empty()).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DIR))
}

/// A placement store directory.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

/// An advisory lock on `<store>/.lock`, released on drop.
#[derive(Debug)]
pub struct LockGuard(File);

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

/// What [`Store::repair`] did to each log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Repair {
    pub placements: TailRepair,
    pub gold: TailRepair,
}

impl Store {
    /// Open (creating if needed) the store directory at `root`.
    pub fn open(root: impl Into<PathBuf>) -> Result<Store, Error> {
        let root = root.into();
        std::fs::create_dir_all(&root).map_err(|e| jsonl::io_error(&root, "create", e))?;
        Ok(Store { root })
    }

    /// Open the store named by `--store` or the environment (see
    /// [`resolve_path`]).
    pub fn open_default(flag: Option<&Path>) -> Result<Store, Error> {
        Store::open(resolve_path(flag, std::env::var_os(ENV_VAR)))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn lock_file(&self) -> Result<File, Error> {
        let path = self.path(LOCK);
        OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|e| jsonl::io_error(&path, "open", e))
    }

    /// Block until this process holds the store's exclusive writer lock.
    /// Every append takes it, so do not append while holding the guard.
    pub fn lock_exclusive(&self) -> Result<LockGuard, Error> {
        let f = self.lock_file()?;
        f.lock_exclusive()
            .map_err(|e| jsonl::io_error(&self.path(LOCK), "lock", e))?;
        Ok(LockGuard(f))
    }

    pub(crate) fn lock_shared(&self) -> Result<LockGuard, Error> {
        let f = self.lock_file()?;
        FileExt::lock_shared(&f).map_err(|e| jsonl::io_error(&self.path(LOCK), "lock", e))?;
        Ok(LockGuard(f))
    }

    fn read_claims(&self) -> Result<Vec<Placement>, Error> {
        jsonl::read(&self.path(PLACEMENTS))
    }

    /// The current placement of `work` in `scheme` (R12): the latest live
    /// confirmed claim, else the latest live machine claim.
    pub fn current(&self, work: &WorkId, scheme: &SchemeId) -> Result<Option<Placement>, Error> {
        Ok(placement::current(&self.claims_for(work, scheme)?).cloned())
    }

    /// The current placement of every (work, scheme) with a live claim, in
    /// order of each pair's first claim.
    pub fn current_all(&self) -> Result<Vec<Placement>, Error> {
        let mut groups: BTreeMap<(WorkId, SchemeId), (usize, Vec<Placement>)> = BTreeMap::new();
        for (i, p) in self.claims()?.into_iter().enumerate() {
            groups
                .entry((p.work.clone(), p.scheme.clone()))
                .or_insert_with(|| (i, Vec::new()))
                .1
                .push(p);
        }
        let mut out: Vec<(usize, Placement)> = groups
            .into_values()
            .filter_map(|(first, claims)| placement::current(&claims).cloned().map(|p| (first, p)))
            .collect();
        out.sort_by_key(|(first, _)| *first);
        Ok(out.into_iter().map(|(_, p)| p).collect())
    }

    /// Live claims made stale by a new scheme or profile (AE6).
    pub fn stale(&self, query: &StaleQuery) -> Result<Vec<StaleClaim>, Error> {
        Ok(stale::stale(&self.claims()?, query))
    }

    /// The memo index as currently on disk, or rebuilt in memory from the
    /// log when the file is missing or behind.
    pub fn memo_index(&self) -> Result<MemoMap, Error> {
        let _lock = self.lock_shared()?;
        let claims = self.read_claims()?;
        Ok(memo::load(&self.path(MEMO), claims.len())
            .unwrap_or_else(|| memo::fold(memo::rebuild(&claims))))
    }

    /// Rewrite `memo.jsonl` from the log.
    pub fn rebuild_memo(&self) -> Result<(), Error> {
        let _lock = self.lock_exclusive()?;
        memo::write_rebuilt(&self.path(MEMO), &self.read_claims()?)
    }

    /// Fix a torn final line in the claim and gold logs: a tail that is a
    /// complete record only missing its newline is kept, anything else is
    /// cut and returned so the caller can report it. The memo index is
    /// rebuilt afterwards.
    pub fn repair(&self) -> Result<Repair, Error> {
        let _lock = self.lock_exclusive()?;
        let repair = Repair {
            placements: jsonl::repair_tail::<Placement>(&self.path(PLACEMENTS))?,
            gold: jsonl::repair_tail::<GoldExample>(&self.path(GOLD))?,
        };
        memo::write_rebuilt(&self.path(MEMO), &self.read_claims()?)?;
        Ok(repair)
    }

    fn check_supersedes(claims: &[Placement], p: &Placement) -> Result<(), Error> {
        let Some(old) = &p.supersedes else {
            return Ok(());
        };
        if !claims.iter().any(|c| &c.id == old) {
            return Err(Error::new(
                ErrorCode::NotFound,
                format!("{} supersedes {old}, which is not in this store", p.id),
            )
            .details(serde_json::json!({ "supersedes": old.to_string() })));
        }
        if let Some(by) = claims.iter().find(|c| c.supersedes.as_ref() == Some(old)) {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                format!(
                    "{old} is already superseded by {}; supersede the current claim instead",
                    by.id
                ),
            )
            .details(serde_json::json!({
                "supersedes": old.to_string(),
                "superseded_by": by.id.to_string(),
            })));
        }
        Ok(())
    }
}

impl PlacementLog for Store {
    fn append(&mut self, p: &Placement) -> Result<bool, Error> {
        if p.compute_id() != p.id {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                format!("{} is not sealed: its id is not its content hash", p.id),
            )
            .details(serde_json::json!({ "expected_id": p.compute_id().to_string() })));
        }
        let _lock = self.lock_exclusive()?;
        let mut claims = self.read_claims()?;
        if claims.iter().any(|c| c.id == p.id) {
            return Ok(false);
        }
        Self::check_supersedes(&claims, p)?;
        let memo_path = self.path(MEMO);
        let memo_current = memo::load(&memo_path, claims.len()).is_some();
        jsonl::append(&self.path(PLACEMENTS), std::slice::from_ref(p))?;
        claims.push(p.clone());
        if memo_current {
            jsonl::append(&memo_path, &memo::entries_after_append(&claims))?;
        } else {
            memo::write_rebuilt(&memo_path, &claims)?;
        }
        Ok(true)
    }

    fn claims(&self) -> Result<Vec<Placement>, Error> {
        let _lock = self.lock_shared()?;
        self.read_claims()
    }

    /// Served from `memo.jsonl` when it is current, else from the log.
    fn memo(&self, inputs: &InputHashes) -> Result<Option<Placement>, Error> {
        let _lock = self.lock_shared()?;
        let log = jsonl::read_bytes(&self.path(PLACEMENTS))?;
        let len = jsonl::lines(&self.path(PLACEMENTS), &log)?.len();
        let key = inputs.memo_key();
        if let Some(map) = memo::load(&self.path(MEMO), len) {
            return Ok(map.get(&key).cloned());
        }
        Ok(memo::lookup(&self.read_claims()?, &key))
    }

    /// `log:<claims>:<last claim id>`: the count grows on every append.
    fn revision(&self) -> Result<String, Error> {
        let _lock = self.lock_shared()?;
        let path = self.path(PLACEMENTS);
        let bytes = jsonl::read_bytes(&path)?;
        let lines = jsonl::lines(&path, &bytes)?;
        match lines.last() {
            None => Ok("log:0".into()),
            Some(last) => {
                let p: Placement = jsonl::parse(&path, lines.len(), last)?;
                Ok(format!("log:{}:{}", lines.len(), p.id))
            }
        }
    }
}
