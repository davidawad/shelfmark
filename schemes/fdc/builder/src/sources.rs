//! The source file, its lock (URL, fetch date, size, sha256) and the fetch
//! that writes it. A build reads only a cached file whose hash matches the
//! lock, so the same lock always builds the same scheme.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

/// The FDC repository on GitHub, and the one commit that last touched
/// `fdc.txt` (2015-07-26). Pinning the commit keeps the URL stable if the
/// author ever edits master.
pub const REPO: &str = "https://github.com/JohnMarkOckerbloom/fdc";
pub const COMMIT: &str = "4d247249c4e2be7cbe13baf84f16fa939b084e0a";
pub const FILE: &str = "fdc.txt";

pub fn url() -> String {
    format!("https://raw.githubusercontent.com/JohnMarkOckerbloom/fdc/{COMMIT}/{FILE}")
}

/// Where the author states the license, for the notes.
pub const LICENSE_URLS: [&str; 3] = [
    "https://raw.githubusercontent.com/JohnMarkOckerbloom/fdc/4d247249c4e2be7cbe13baf84f16fa939b084e0a/LICENSE",
    "https://everybodyslibraries.com/free-decimal-correspondence/",
    "https://everybodyslibraries.com/about-the-free-decimal-correspondence/",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locked {
    pub file: String,
    pub url: String,
    /// Git commit the URL is pinned to.
    pub commit: String,
    /// UTC date of the fetch, `YYYY-MM-DD`.
    pub fetched: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    pub sources: Vec<Locked>,
}

impl Lock {
    /// The one locked source.
    pub fn get(&self) -> Result<&Locked, String> {
        match self.sources.as_slice() {
            [one] => Ok(one),
            _ => Err("the lock needs exactly one source".to_string()),
        }
    }
}

/// Download `fdc.txt` into `cache` and return the lock for it.
pub fn fetch(cache: &Path) -> Result<Lock, String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let url = url();
    eprintln!("fetch {url}");
    let bytes = download(&url)?;
    let path = cache.join(FILE);
    std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Lock {
        sources: vec![Locked {
            file: FILE.to_string(),
            url,
            commit: COMMIT.to_string(),
            fetched: today(),
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
        }],
    })
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let mut last = String::new();
    for _ in 0..3 {
        match ureq::get(url).call() {
            Ok(mut resp) => match resp.body_mut().read_to_vec() {
                Ok(bytes) => return Ok(bytes),
                Err(e) => last = e.to_string(),
            },
            Err(e) => last = e.to_string(),
        }
    }
    Err(format!("fetch {url}: {last}"))
}

/// Read the locked file from `cache`, refusing one whose hash has changed.
pub fn read(cache: &Path, locked: &Locked) -> Result<String, String> {
    let path = cache.join(&locked.file);
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{}: {e}; run `fdc-scheme fetch` or copy the source there",
            path.display()
        )
    })?;
    let got = sha256_hex(&bytes);
    if got != locked.sha256 {
        return Err(format!(
            "{} has sha256 {got}, the lock says {}; refetch and review the diff",
            path.display(),
            locked.sha256
        ));
    }
    String::from_utf8(bytes).map_err(|e| format!("{}: {e}", path.display()))
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
    }

    #[test]
    fn the_url_is_pinned_to_a_commit() {
        assert!(url().contains(COMMIT));
        assert!(url().ends_with("/fdc.txt"));
    }

    #[test]
    fn a_lock_needs_exactly_one_source() {
        assert!(Lock { sources: vec![] }.get().is_err());
    }
}
