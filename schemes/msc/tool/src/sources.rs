//! The source lock and the fetch that honours it.
//!
//! `sources.lock.json` pins the CSV's URL, size and sha256 along with its
//! license and attribution. A build reads the cached CSV, or downloads it
//! when absent, and refuses any bytes whose hash differs from the lock, so
//! the same lock always builds the same scheme file.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

/// `schemes/msc/sources.lock.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    pub url: String,
    /// File name in the cache directory.
    pub file: String,
    pub bytes: u64,
    pub sha256: String,
    /// Day the hash was last confirmed against upstream (YYYY-MM-DD).
    pub checked: String,
    pub license: String,
    pub license_url: String,
    /// Credit the license requires.
    pub attribution: String,
    /// May the data ship inside shelfmark's MIT/Apache binary?
    pub bundle: bool,
}

impl Lock {
    pub fn load(path: &Path) -> Result<Lock, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map(|s| s + "\n")
            .map_err(|e| e.to_string())
    }

    fn verify(&self, bytes: &[u8], what: &str) -> Result<(), String> {
        let got = sha256_hex(bytes);
        if got != self.sha256 {
            return Err(format!(
                "{what} has sha256 {got}, the lock pins {}; if upstream changed on purpose, \
                 review it and run `msc-scheme relock`",
                self.sha256
            ));
        }
        Ok(())
    }
}

/// The locked CSV: from `cache` when present and intact, else downloaded
/// (unless `offline`), checked and saved there.
pub fn obtain(cache: &Path, lock: &Lock, offline: bool) -> Result<Vec<u8>, String> {
    let path = cache.join(&lock.file);
    if let Ok(bytes) = std::fs::read(&path) {
        lock.verify(&bytes, &path.display().to_string())?;
        return Ok(bytes);
    }
    if offline {
        return Err(format!(
            "{} is not cached and --offline forbids fetching {}",
            path.display(),
            lock.url
        ));
    }
    let bytes = download(&lock.url)?;
    lock.verify(&bytes, &lock.url)?;
    save(&path, &bytes)?;
    Ok(bytes)
}

/// Download the current upstream file and return a lock pinning it,
/// keeping the license fields of `old`.
pub fn relock(cache: &Path, old: &Lock, today: &str) -> Result<Lock, String> {
    let bytes = download(&old.url)?;
    save(&cache.join(&old.file), &bytes)?;
    Ok(Lock {
        bytes: bytes.len() as u64,
        sha256: sha256_hex(&bytes),
        checked: today.to_string(),
        ..old.clone()
    })
}

/// Write via a temporary file so a failed write never leaves a partial CSV.
fn save(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let tmp = PathBuf::from(format!("{}.part", path.display()));
    std::fs::write(&tmp, bytes).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let mut last = String::new();
    for _ in 0..3 {
        match ureq::get(url).call() {
            Ok(mut resp) => match resp.body_mut().with_config().limit(64 << 20).read_to_vec() {
                Ok(bytes) => return Ok(bytes),
                Err(e) => last = e.to_string(),
            },
            Err(e) => last = e.to_string(),
        }
    }
    Err(format!("fetch {url}: {last}"))
}

/// Today's UTC date as YYYY-MM-DD.
pub fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    civil(secs / 86_400)
}

/// Days since 1970-01-01 to a calendar date (Hinnant's algorithm).
fn civil(days: u64) -> String {
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::civil;

    #[test]
    fn civil_dates() {
        assert_eq!(civil(0), "1970-01-01");
        assert_eq!(civil(20_734), "2026-10-08");
        assert_eq!(civil(11_016), "2000-02-29");
    }
}
