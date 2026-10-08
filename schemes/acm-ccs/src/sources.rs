//! The one source file, its lock (URL, size, sha256) and the download.
//!
//! The ACM data is never committed: `sources.lock.json` pins the official
//! SKOS file, and a build reads only a cached copy whose sha256 matches
//! the lock, so the same lock always builds the same scheme.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

/// Some ACM hosts challenge clients that send no browser-like agent.
const USER_AGENT: &str = "Mozilla/5.0 (compatible; shelfmark-acm-ccs/0.1)";

/// The lock for the official SKOS file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    /// Name of the file in the cache directory.
    pub file: String,
    /// Where the file is downloaded from.
    pub url: String,
    /// The ACM page that links the file.
    pub page: String,
    /// The server's `Last-Modified` header at the locked fetch.
    pub last_modified: String,
    /// UTC date of the locked fetch, `YYYY-MM-DD`.
    pub fetched: String,
    pub bytes: u64,
    pub sha256: String,
}

impl Lock {
    pub fn read(path: &Path) -> Result<Lock, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map(|s| s + "\n")
            .map_err(|e| e.to_string())
    }

    /// Check bytes against the lock; the error names both hashes.
    pub fn verify(&self, bytes: &[u8], what: &str) -> Result<(), String> {
        let got = sha256_hex(bytes);
        if got != self.sha256 {
            return Err(format!(
                "{what} has sha256 {got}, the lock says {}; ACM may have published a new \
                 release: review it, then run `acm-ccs fetch --relock`",
                self.sha256
            ));
        }
        Ok(())
    }
}

/// A completed download.
pub struct Fetched {
    pub bytes: Vec<u8>,
    pub last_modified: String,
}

/// Download `url`, retrying twice.
pub fn download(url: &str) -> Result<Fetched, String> {
    let mut last = String::new();
    for _ in 0..3 {
        match ureq::get(url).header("User-Agent", USER_AGENT).call() {
            Ok(mut resp) => {
                let last_modified = resp
                    .headers()
                    .get("last-modified")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or_default()
                    .to_string();
                match resp.body_mut().with_config().limit(u64::MAX).read_to_vec() {
                    Ok(bytes) => {
                        return Ok(Fetched {
                            bytes,
                            last_modified,
                        })
                    }
                    Err(e) => last = e.to_string(),
                }
            }
            Err(e) => last = e.to_string(),
        }
    }
    Err(format!("fetch {url}: {last}"))
}

/// Download the locked file into `cache`. With `relock`, a changed file is
/// accepted and the returned lock describes it; otherwise it must match.
pub fn fetch(cache: &Path, lock: &Lock, relock: bool) -> Result<Lock, String> {
    eprintln!("fetch {}", lock.url);
    let got = download(&lock.url)?;
    if !relock {
        lock.verify(&got.bytes, &lock.url)?;
    }
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let path = cache.join(&lock.file);
    std::fs::write(&path, &got.bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    if !relock {
        return Ok(lock.clone());
    }
    Ok(Lock {
        last_modified: got.last_modified,
        fetched: today(),
        bytes: got.bytes.len() as u64,
        sha256: sha256_hex(&got.bytes),
        ..lock.clone()
    })
}

/// Read the locked file from `cache`, refusing one whose hash differs.
/// Returns `Ok(None)` when the file is not cached yet.
pub fn read_cached(cache: &Path, lock: &Lock) -> Result<Option<Vec<u8>>, String> {
    let path = cache.join(&lock.file);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    lock.verify(&bytes, &path.display().to_string())?;
    Ok(Some(bytes))
}

/// Today's UTC date as `YYYY-MM-DD`.
fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    civil((secs / 86_400) as i64)
}

/// Days since 1970-01-01 to a proleptic Gregorian date (Hinnant's
/// `civil_from_days`).
fn civil(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
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
    use super::*;

    #[test]
    fn civil_dates() {
        assert_eq!(civil(0), "1970-01-01");
        assert_eq!(civil(20_734), "2026-10-08");
        assert_eq!(civil(11_016), "2000-02-29");
    }

    #[test]
    fn the_committed_lock_parses() {
        let lock =
            Lock::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("sources.lock.json")).unwrap();
        assert!(lock.url.starts_with("https://dl.acm.org/"));
        assert_eq!(lock.sha256.len(), 64);
        assert!(lock.verify(b"not the file", "x").is_err());
    }
}
