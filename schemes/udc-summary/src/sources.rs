//! The udcsummary.info tree pages, their lock (URL, size, sha256, the
//! edition they make) and the downloads. A build reads only cached pages
//! whose hashes match the lock, so one lock always builds one scheme.
//!
//! The UDC Summary is CC BY-NC 4.0, so shelfmark never ships it: the lock
//! carries URLs and hashes only, and each user fetches the pages.

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

/// The English tree pages. `index.php?tag=X` prints the whole subtree of
/// `X` as `d.add(...)` calls.
pub const BASE: &str = "https://udcsummary.info/php/index.php";

/// The common auxiliary numbers (Tables 1c-1k) and main classes 0-9 (4
/// is vacant). The common auxiliary signs page (`tag=--`: `+`, `/`, `:`,
/// `::`, `[]`, `*`, `A/Z`) is left out: signs join numbers, they are not
/// classes a work is placed in.
pub const TAGS: [&str; 10] = ["---", "0", "1", "2", "3", "5", "6", "7", "8", "9"];

const USER_AGENT: &str = "shelfmark-udc-summary/0.1 (+https://gitlab.com/davidawad/shelfmark)";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locked {
    /// The `tag` the page was fetched with.
    pub tag: String,
    pub file: String,
    pub url: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    /// The scheme version these pages build: the UTC date (`YYYY-MM-DD`)
    /// of the first fetch that saw exactly these hashes.
    pub edition: String,
    pub sources: Vec<Locked>,
}

impl Lock {
    /// `s256:` of every locked URL and hash, one `url sha256` line each:
    /// one value naming the whole input set.
    pub fn digest(&self) -> String {
        let lines: String = self
            .sources
            .iter()
            .map(|s| format!("{} {}\n", s.url, s.sha256))
            .collect();
        format!("s256:{}", sha256_hex(lines.as_bytes()))
    }
}

/// (tag, file name, URL) for every page, in build order.
pub fn all() -> Vec<(String, String, String)> {
    TAGS.iter()
        .map(|&t| {
            let name = if t.starts_with('-') { "aux" } else { t };
            (
                t.to_string(),
                format!("udcs-{name}.html"),
                format!("{BASE}?tag={t}&lang=en"),
            )
        })
        .collect()
}

/// Download every page into `cache` and lock it. The edition of `prior`
/// is kept when nothing changed, so a refetch of the same data is a
/// no-op.
pub fn fetch(cache: &Path, prior: Option<&Lock>) -> Result<Lock, String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let mut sources = Vec::new();
    for (tag, file, url) in all() {
        let bytes = download(&url)?;
        let path = cache.join(&file);
        std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        sources.push(Locked {
            tag,
            file,
            url,
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
        });
    }
    let edition = match prior {
        Some(p) if p.sources == sources => p.edition.clone(),
        _ => today(),
    };
    Ok(Lock { edition, sources })
}

/// Download the locked pages missing from `cache`; [`read`] then checks
/// them against the lock.
pub fn fetch_missing(cache: &Path, lock: &Lock) -> Result<(), String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    for locked in &lock.sources {
        let path = cache.join(&locked.file);
        if !path.exists() {
            let bytes = download(&locked.url)?;
            std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    Ok(())
}

/// GET `url` with three tries, one second apart: the site is a small
/// consortium's server.
fn download(url: &str) -> Result<Vec<u8>, String> {
    eprintln!("fetch {url}");
    let mut last = String::new();
    for attempt in 0..3 {
        std::thread::sleep(Duration::from_secs(if attempt == 0 { 1 } else { 3 }));
        match ureq::get(url).header("User-Agent", USER_AGENT).call() {
            Ok(mut resp) => match resp.body_mut().read_to_vec() {
                Ok(bytes) => return Ok(bytes),
                Err(e) => last = e.to_string(),
            },
            Err(e) => last = e.to_string(),
        }
    }
    Err(format!("fetch {url}: {last}"))
}

/// Read a locked page from `cache`, refusing one whose hash has changed.
pub fn read(cache: &Path, locked: &Locked) -> Result<String, String> {
    let path = cache.join(&locked.file);
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{}: {e}; run `udc-summary build --fetch` to download it",
            path.display()
        )
    })?;
    let got = sha256_hex(&bytes);
    if got != locked.sha256 {
        return Err(format!(
            "{} has sha256 {got}, the lock says {}; udcsummary.info has changed: \
             run `udc-summary fetch` and review the diff",
            path.display(),
            locked.sha256
        ));
    }
    String::from_utf8(bytes).map_err(|e| format!("{}: not UTF-8: {e}", path.display()))
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
    fn ten_pages_with_distinct_files() {
        let all = all();
        assert_eq!(all.len(), 10);
        assert_eq!(all[0].1, "udcs-aux.html");
        assert_eq!(
            all[1].2,
            "https://udcsummary.info/php/index.php?tag=0&lang=en"
        );
        let mut files: Vec<_> = all.iter().map(|a| &a.1).collect();
        files.dedup();
        assert_eq!(files.len(), 10);
    }

    #[test]
    fn digest_names_urls_and_hashes() {
        let lock = |sha: &str| Lock {
            edition: "2026-10-08".into(),
            sources: vec![Locked {
                tag: "0".into(),
                file: "udcs-0.html".into(),
                url: "u".into(),
                bytes: 1,
                sha256: sha.into(),
            }],
        };
        assert_eq!(lock("a").digest(), lock("a").digest());
        assert_ne!(lock("a").digest(), lock("b").digest());
        assert!(lock("a").digest().starts_with("s256:"));
    }
}
