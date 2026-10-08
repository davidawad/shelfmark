//! Fetch the ACM Computing Classification System 2012 and build it as a
//! shelfmark scheme (`scheme:acm-ccs@2012`). See the README for why the
//! data is fetched rather than bundled.

pub mod build;
pub mod skos;
pub mod sources;

use std::path::{Path, PathBuf};

use shelfmark_core::scheme::SchemeDoc;

/// This crate's directory: the lock lives here, and the default cache and
/// output directories (both git-ignored) sit under it.
pub fn home() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn lock_path() -> PathBuf {
    home().join("sources.lock.json")
}

/// Build the scheme and its JSON text from the SKOS file's bytes, which
/// must match `lock`.
pub fn build_bytes(
    bytes: &[u8],
    lock: &sources::Lock,
) -> Result<(SchemeDoc, String, build::Stats), String> {
    lock.verify(bytes, &lock.file)?;
    let skos = skos::parse(bytes).map_err(|e| format!("{}: {e}", lock.file))?;
    let (doc, stats) = build::build(&skos, lock)?;
    let json = build::scheme_json(&doc)?;
    Ok((doc, json, stats))
}

/// The cached SKOS file, downloading it first unless `offline`.
pub fn source_bytes(cache: &Path, lock: &sources::Lock, offline: bool) -> Result<Vec<u8>, String> {
    if let Some(bytes) = sources::read_cached(cache, lock)? {
        return Ok(bytes);
    }
    if offline {
        return Err(format!(
            "{} is not cached; run `acm-ccs fetch --cache {}`",
            lock.file,
            cache.display()
        ));
    }
    sources::fetch(cache, lock, false)?;
    sources::read_cached(cache, lock)?.ok_or_else(|| format!("{} vanished after fetch", lock.file))
}
