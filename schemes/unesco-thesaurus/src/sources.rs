//! The one source file, its lock (URL, size, sha256) and the download.
//!
//! UNESCO publishes the thesaurus as a single SKOS Turtle export, always
//! at the same `latest` URL, and keeps older versions "upon request"
//! only. So the lock is what makes a build repeatable: a build reads the
//! cached file only if its sha256 matches, and the built scheme is
//! committed.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

/// The SKOS Turtle export (linked from vocabularies.unesco.org/en/about).
pub const URL: &str =
    "https://vocabularies.unesco.org/exports/thesaurus/latest/unesco-thesaurus.ttl";
/// Cache file name.
pub const FILE: &str = "unesco-thesaurus.ttl";

const USER_AGENT: &str = "shelfmark-unesco-thesaurus/0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    pub file: String,
    pub url: String,
    pub bytes: u64,
    pub sha256: String,
    /// UTC date the file was fetched (`YYYY-MM-DD`), as given by the caller.
    pub fetched: String,
}

impl Lock {
    /// `s256:` of the URL and hash: one value naming the input.
    pub fn digest(&self) -> String {
        format!(
            "s256:{}",
            sha256_hex(format!("{} {}\n", self.url, self.sha256).as_bytes())
        )
    }
}

/// Download the export into `cache` and lock it. `fetched` is the date
/// to record (the CLI passes today's).
pub fn fetch(cache: &Path, fetched: &str) -> Result<Lock, String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let bytes = download(URL)?;
    let path = cache.join(FILE);
    std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Lock {
        file: FILE.into(),
        url: URL.into(),
        bytes: bytes.len() as u64,
        sha256: sha256_hex(&bytes),
        fetched: fetched.into(),
    })
}

/// Download the locked file into `cache` when it is missing.
pub fn fetch_missing(cache: &Path, lock: &Lock) -> Result<(), String> {
    let path = cache.join(&lock.file);
    if path.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let bytes = download(&lock.url)?;
    std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    eprintln!("fetch {url}");
    let mut last = String::new();
    for attempt in 0..3 {
        if attempt > 0 {
            std::thread::sleep(Duration::from_secs(3));
        }
        match ureq::get(url).header("User-Agent", USER_AGENT).call() {
            Ok(mut resp) => match resp
                .body_mut()
                .with_config()
                .limit(64 * 1024 * 1024)
                .read_to_vec()
            {
                Ok(bytes) => return Ok(bytes),
                Err(e) => last = e.to_string(),
            },
            Err(e) => last = e.to_string(),
        }
    }
    Err(format!("fetch {url}: {last}"))
}

/// Read the locked file from `cache`, refusing one whose hash changed.
pub fn read(cache: &Path, lock: &Lock) -> Result<String, String> {
    let path = cache.join(&lock.file);
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{}: {e}; run `unesco-thesaurus build --fetch` to download it",
            path.display()
        )
    })?;
    let got = sha256_hex(&bytes);
    if got != lock.sha256 {
        return Err(format!(
            "{} has sha256 {got}, the lock says {}; UNESCO has published a new export: \
             run `unesco-thesaurus fetch` and review the diff",
            path.display(),
            lock.sha256
        ));
    }
    String::from_utf8(bytes).map_err(|e| format!("{}: not UTF-8: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_names_url_and_hash() {
        let lock = |sha: &str| Lock {
            file: FILE.into(),
            url: URL.into(),
            bytes: 1,
            sha256: sha.into(),
            fetched: "2026-10-09".into(),
        };
        assert_eq!(lock("a").digest(), lock("a").digest());
        assert_ne!(lock("a").digest(), lock("b").digest());
        assert!(lock("a").digest().starts_with("s256:"));
    }

    #[test]
    fn refuses_a_changed_or_missing_file() {
        let dir =
            std::env::temp_dir().join(format!("unesco-thesaurus-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let lock = Lock {
            file: FILE.into(),
            url: URL.into(),
            bytes: 0,
            sha256: "0".repeat(64),
            fetched: "2026-10-09".into(),
        };
        let err = read(&dir, &lock).unwrap_err();
        assert!(err.contains("build --fetch"), "{err}");
        std::fs::write(dir.join(FILE), "changed").unwrap();
        let err = read(&dir, &lock).unwrap_err();
        assert!(err.contains("new export"), "{err}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
