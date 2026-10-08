//! Builds a shelfmark scheme file from the UDC Summary.
//!
//! The UDC Summary is licensed CC BY-NC 4.0, so shelfmark cannot ship it
//! inside an MIT/Apache binary. This crate fetches the summary's tree
//! pages from udcsummary.info, checks them against `sources.lock.json`
//! (URLs and sha256 only) and writes a scheme file the user loads with
//! `shelfmark scheme compile`. See README.md.

pub mod build;
pub mod page;
pub mod sources;

use std::path::Path;

use shelfmark_core::scheme::SchemeDoc;

use sources::Lock;

/// Read a lock file.
pub fn read_lock(path: &Path) -> Result<Lock, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Build the scheme from the locked pages in `cache`, offline.
pub fn generate(cache: &Path, lock: &Lock) -> Result<(SchemeDoc, build::Stats), String> {
    let pages = lock
        .sources
        .iter()
        .map(|l| sources::read(cache, l))
        .collect::<Result<Vec<_>, _>>()?;
    build::build(lock, &pages)
}
