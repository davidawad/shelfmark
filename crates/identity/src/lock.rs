//! The store write lock (plan PD6): every writer to a store directory
//! holds an exclusive advisory lock on `<store>/.lock` for each append.
//! This crate opens the lock file itself rather than depending on the
//! store crate, so the rule is shared and the code is not.

use std::fs::{self, File, OpenOptions};
use std::path::Path;

use shelfmark_core::{Error, ErrorCode};

/// Name of the lock file inside a store directory.
pub const LOCK_FILE: &str = ".lock";

pub(crate) fn store_error(what: &str, path: &Path, err: &std::io::Error) -> Error {
    Error::new(
        ErrorCode::StoreError,
        format!("could not {what} {}: {err}", path.display()),
    )
    .details(serde_json::json!({ "path": path.display().to_string() }))
}

/// Run `f` while holding the exclusive lock on `<dir>/.lock`, creating the
/// directory and lock file if needed. The lock is released when `f`
/// returns, including on error.
pub fn with_store_lock<T>(dir: &Path, f: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
    fs::create_dir_all(dir).map_err(|e| store_error("create store directory", dir, &e))?;
    let path = dir.join(LOCK_FILE);
    let file: File = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| store_error("open lock file", &path, &e))?;
    // Fully qualified: newer std has an inherent `File::lock` above the
    // workspace MSRV.
    fs4::FileExt::lock(&file).map_err(|e| store_error("lock", &path, &e))?;
    let out = f();
    // Closing the file releases the lock too; unlock explicitly so an
    // unlock failure is not silently lost behind a successful append.
    let unlocked = fs4::FileExt::unlock(&file).map_err(|e| store_error("unlock", &path, &e));
    let value = out?;
    unlocked?;
    Ok(value)
}
