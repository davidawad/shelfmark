//! The source files, their lock (URL, fetch date, size, sha256) and the
//! fetch that writes it. A build reads only cached files whose hashes
//! match the lock, so the same lock always builds the same scheme.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

/// The 2022 NAICS reference files on the Census Bureau's NAICS site.
pub const BASE: &str = "https://www.census.gov/naics/2022NAICS/";

/// census.gov answers 403 to clients that do not look like a browser.
const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) shelfmark-naics/0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Every 2- to 6-digit code with its US title, in structure order.
    Codes,
    /// Index entries: a 6-digit industry and a phrase classified there.
    Index,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locked {
    pub file: String,
    pub url: String,
    pub role: Role,
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
    /// The one locked source with `role`.
    pub fn get(&self, role: Role) -> Result<&Locked, String> {
        let mut it = self.sources.iter().filter(|l| l.role == role);
        match (it.next(), it.next()) {
            (Some(l), None) => Ok(l),
            _ => Err(format!("the lock needs exactly one {role:?} source")),
        }
    }
}

/// Every source as (file name, URL, role), in build order.
pub fn all() -> Vec<(String, String, Role)> {
    [
        ("2-6 digit_2022_Codes.xlsx", Role::Codes),
        ("2022_NAICS_Index_File.xlsx", Role::Index),
    ]
    .into_iter()
    .map(|(file, role)| {
        let url = format!("{BASE}{}", file.replace(' ', "%20"));
        (file.to_string(), url, role)
    })
    .collect()
}

/// Download every source into `cache` and return the lock for them.
pub fn fetch(cache: &Path) -> Result<Lock, String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let fetched = today();
    let mut sources = Vec::new();
    for (file, url, role) in all() {
        eprintln!("fetch {url}");
        let bytes = download(&url)?;
        let path = cache.join(&file);
        std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        sources.push(Locked {
            file,
            url,
            role,
            fetched: fetched.clone(),
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
        });
    }
    Ok(Lock { sources })
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let mut last = String::new();
    for _ in 0..3 {
        match ureq::get(url).header("User-Agent", USER_AGENT).call() {
            Ok(mut resp) => match resp.body_mut().with_config().limit(u64::MAX).read_to_vec() {
                Ok(bytes) => return Ok(bytes),
                Err(e) => last = e.to_string(),
            },
            Err(e) => last = e.to_string(),
        }
    }
    Err(format!("fetch {url}: {last}"))
}

/// Read a locked file from `cache`, refusing one whose hash has changed.
pub fn read(cache: &Path, locked: &Locked) -> Result<Vec<u8>, String> {
    let path = cache.join(&locked.file);
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{}: {e}; run `naics-scheme fetch` or copy the sources there",
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
    Ok(bytes)
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
    fn source_urls_escape_spaces() {
        let all = all();
        assert_eq!(all.len(), 2);
        assert_eq!(
            all[0].1,
            "https://www.census.gov/naics/2022NAICS/2-6%20digit_2022_Codes.xlsx"
        );
    }

    #[test]
    fn a_lock_needs_one_source_per_role() {
        let lock = Lock { sources: vec![] };
        assert!(lock.get(Role::Codes).is_err());
    }
}
