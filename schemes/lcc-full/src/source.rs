//! The source files, their lock (URL, fetch date, size, sha256) and the
//! fetch that writes it. A build reads only cached files whose hashes
//! match the lock, so the same lock always builds the same scheme.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

pub const FREELCC: &str = "https://www.loc.gov/aba/publications/FreeLCC/";
pub const PAGE: &str = "https://www.loc.gov/aba/publications/FreeLCC/freelcc.html";

/// The 2024 FreeLCC Text PDFs, one per schedule, in class order. A file is
/// `LCC_<name>2024TEXT.pdf`, except the one that spells its range
/// `PR-PS_PZ`.
pub const SCHEDULES: [&str; 44] = [
    "A",
    "B-BJ",
    "BL-BQ",
    "BR-BX",
    "C",
    "D-DR",
    "DS-DX",
    "E-F",
    "G",
    "H",
    "J",
    "K",
    "KB",
    "KD",
    "KDZ_KG-KH",
    "KE",
    "KF",
    "KI-KIL",
    "KIM-KIR",
    "KJ-KKZ",
    "KJV-KJW",
    "KK-KKC",
    "KL-KWX",
    "KVJ",
    "KZ",
    "K_TABLES",
    "L",
    "M",
    "N",
    "P-PA",
    "PB-PH",
    "PJ-PK",
    "PL-PM",
    "PN",
    "PQ",
    "PR-PS_PZ",
    "PT",
    "P_TABLES",
    "Q",
    "R",
    "S",
    "T",
    "U-V",
    "Z",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locked {
    pub file: String,
    pub url: String,
    /// UTC date of the fetch, `YYYY-MM-DD`.
    pub fetched: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    /// sha256 of the bundled outline scheme (`data/schemes/lcc/lcc.json`)
    /// whose labels, nesting and vocabulary the build keeps.
    pub outline_sha256: String,
    pub sources: Vec<Locked>,
}

/// Every source as (file name, URL, schedule name), in build order.
pub fn all() -> Vec<(String, String, &'static str)> {
    SCHEDULES
        .iter()
        .map(|&s| {
            let file = format!("LCC_{s}2024TEXT.pdf");
            let url = format!("{FREELCC}{file}");
            (file, url, s)
        })
        .collect()
}

/// Download every source into `cache` and return the lock for them.
pub fn fetch(cache: &Path, outline_sha256: &str) -> Result<Lock, String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let fetched = today();
    let mut sources = Vec::new();
    for (file, url, _) in all() {
        let path = cache.join(&file);
        let bytes = match std::fs::read(&path) {
            Ok(b) if !b.is_empty() => b,
            _ => {
                eprintln!("fetch {url}");
                let b = download(&url)?;
                std::fs::write(&path, &b).map_err(|e| format!("{}: {e}", path.display()))?;
                b
            }
        };
        sources.push(Locked {
            file,
            url,
            fetched: fetched.clone(),
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
        });
    }
    Ok(Lock {
        outline_sha256: outline_sha256.to_string(),
        sources,
    })
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let mut last = String::new();
    for _ in 0..4 {
        match ureq::get(url).call() {
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
            "{}: {e}; run `lcc-full fetch --cache {}`",
            path.display(),
            cache.display()
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
        assert_eq!(civil(11_016), "2000-02-29");
    }

    #[test]
    fn urls_follow_the_naming() {
        let all = all();
        assert_eq!(all.len(), 44);
        assert!(all.iter().any(|(f, u, _)| f == "LCC_PR-PS_PZ2024TEXT.pdf"
            && u.ends_with("/FreeLCC/LCC_PR-PS_PZ2024TEXT.pdf")));
    }
}
