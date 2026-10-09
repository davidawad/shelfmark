//! Builds a shelfmark scheme file from the Integrative Levels
//! Classification, edition 2 (ILC2).
//!
//! The licence of the ILC2 data is unsettled (the publisher's SKOS file
//! says CC BY 3.0, BARTOC says CC BY-SA 3.0, no iskoi.org page says
//! anything), so shelfmark does not ship it. This crate fetches the SKOS
//! file from iskoi.org, checks it against `sources.lock.json` (URL and
//! sha256 only) and writes a scheme file the user loads with
//! `shelfmark scheme compile`. See README.md.

pub mod build;
pub mod nt;
pub mod sources;

use std::path::Path;

use shelfmark_core::scheme::SchemeDoc;

use sources::Lock;

/// Read a lock file.
pub fn read_lock(path: &Path) -> Result<Lock, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Build the scheme from the locked file in `cache`, offline.
pub fn generate(cache: &Path, lock: &Lock) -> Result<(SchemeDoc, build::Stats), String> {
    let text = sources::read(cache, &lock.source)?;
    build::build(lock, &text)
}
