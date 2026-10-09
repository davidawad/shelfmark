//! The ILC2 SKOS file on iskoi.org, its lock (URL, size, sha256) and the
//! download. A build reads only a cached file whose hash matches the
//! lock, so one lock always builds one scheme.
//!
//! The licence is unsettled (see README.md), so shelfmark does not ship
//! the data: the lock carries the URL and hash only, and each user
//! fetches the file.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

/// The N-Triples serialisation of ILC2 linked from
/// <https://www.iskoi.org/ilc/skos.php> (Turtle and RDF/XML hold the same
/// triples).
pub const URL: &str = "https://www.iskoi.org/ilc/2/skos/ILC2.nt";
pub const FILE: &str = "ILC2.nt";

/// The file is about 11 MB; ureq's default body limit is 10 MB.
const MAX_BYTES: u64 = 64 * 1024 * 1024;

const USER_AGENT: &str = "shelfmark-ilc2/0.1 (+https://github.com/shelfmark/shelfmark)";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locked {
    pub file: String,
    pub url: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    pub source: Locked,
}

impl Lock {
    /// `s256:` of the locked URL and hash: one value naming the input.
    pub fn digest(&self) -> String {
        let line = format!("{} {}\n", self.source.url, self.source.sha256);
        format!("s256:{}", sha256_hex(line.as_bytes()))
    }
}

/// Download the file into `cache` and lock it.
pub fn fetch(cache: &Path) -> Result<Lock, String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let bytes = download(URL)?;
    let path = cache.join(FILE);
    std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Lock {
        source: Locked {
            file: FILE.to_string(),
            url: URL.to_string(),
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
        },
    })
}

/// Download the locked file when `cache` lacks it; [`read`] then checks
/// it against the lock.
pub fn fetch_missing(cache: &Path, lock: &Lock) -> Result<(), String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let path = cache.join(&lock.source.file);
    if !path.exists() {
        let bytes = download(&lock.source.url)?;
        std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

/// GET `url` with three tries, a few seconds apart.
fn download(url: &str) -> Result<Vec<u8>, String> {
    eprintln!("fetch {url}");
    let mut last = String::new();
    for attempt in 0..3 {
        std::thread::sleep(Duration::from_secs(if attempt == 0 { 1 } else { 3 }));
        match ureq::get(url).header("User-Agent", USER_AGENT).call() {
            Ok(mut resp) => match resp.body_mut().with_config().limit(MAX_BYTES).read_to_vec() {
                Ok(bytes) => return Ok(bytes),
                Err(e) => last = e.to_string(),
            },
            Err(e) => last = e.to_string(),
        }
    }
    Err(format!("fetch {url}: {last}"))
}

/// Read the locked file from `cache`, refusing one whose hash changed.
pub fn read(cache: &Path, locked: &Locked) -> Result<String, String> {
    let path = cache.join(&locked.file);
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{}: {e}; run `ilc2 build --fetch` to download it",
            path.display()
        )
    })?;
    let got = sha256_hex(&bytes);
    if got != locked.sha256 {
        return Err(format!(
            "{} has sha256 {got}, the lock says {}; iskoi.org has changed: \
             run `ilc2 fetch` and review the diff",
            path.display(),
            locked.sha256
        ));
    }
    String::from_utf8(bytes).map_err(|e| format!("{}: not UTF-8: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock(sha: &str) -> Lock {
        Lock {
            source: Locked {
                file: FILE.into(),
                url: "u".into(),
                bytes: 1,
                sha256: sha.into(),
            },
        }
    }

    #[test]
    fn digest_names_url_and_hash() {
        assert_eq!(lock("a").digest(), lock("a").digest());
        assert_ne!(lock("a").digest(), lock("b").digest());
        assert!(lock("a").digest().starts_with("s256:"));
    }

    #[test]
    fn the_url_is_the_skos_file() {
        assert!(URL.ends_with("/ilc/2/skos/ILC2.nt"));
    }
}
