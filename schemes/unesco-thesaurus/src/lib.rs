//! Builds shelfmark's UNESCO Thesaurus scheme from UNESCO's published
//! SKOS Turtle export (CC BY-SA 3.0 IGO). See README.md.
//!
//! [`generate`] is the whole build: it reads the cached export, refuses
//! it if its sha256 differs from `sources.lock.json`, and renders the
//! scheme. The same lock and export always give the same bytes.

pub mod build;
pub mod sources;
pub mod ttl;

use std::path::Path;

use shelfmark_core::scheme::SchemeDoc;

use sources::Lock;

/// The lock file's name, next to this crate's manifest.
pub const LOCK: &str = "sources.lock.json";
/// The committed scheme file's name.
pub const SCHEME: &str = "unesco-thesaurus.json";

/// Read a lock file.
pub fn read_lock(path: &Path) -> Result<Lock, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Build the scheme from the locked export in `cache`, offline.
pub fn generate(cache: &Path, lock: &Lock) -> Result<(SchemeDoc, build::Stats), String> {
    build::build(lock, &sources::read(cache, lock)?)
}
