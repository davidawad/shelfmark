//! The lock: which CPC title list a build reads (URL, fetch date, size,
//! sha256) and the sha256 of each scheme it must produce. A build reads
//! only a cached archive whose hash matches the lock, and `check` fails
//! unless the output hashes match too, so the same lock always builds the
//! same bytes.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

/// The bulk-download page that lists each release's archives.
pub const BULK: &str =
    "https://www.cooperativepatentclassification.org/cpcSchemeAndDefinitions/bulk";
/// Where the archives live; a release is `CPCTitleListYYYYMM.zip`.
pub const FILES: &str =
    "https://www.cooperativepatentclassification.org/sites/default/files/cpc/bulk/";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locked {
    pub file: String,
    pub url: String,
    /// UTC date of the fetch, `YYYY-MM-DD`.
    pub fetched: String,
    pub bytes: u64,
    pub sha256: String,
}

/// One scheme file the lock's source must build into.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub file: String,
    /// The `--depth` it is built at.
    pub depth: String,
    pub classes: usize,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    /// CPC release, `YYYY.MM`: the scheme version.
    pub release: String,
    pub source: Locked,
    pub outputs: Vec<Output>,
}

/// `YYYYMM` to the scheme version `YYYY.MM`.
pub fn version(release: &str) -> Result<String, String> {
    let ok = release.len() == 6
        && release.bytes().all(|b| b.is_ascii_digit())
        && matches!(
            &release[4..],
            "01" | "02" | "03" | "04" | "05" | "06" | "07" | "08" | "09" | "10" | "11" | "12"
        );
    if !ok {
        return Err(format!("release `{release}` is not YYYYMM"));
    }
    Ok(format!("{}.{}", &release[..4], &release[4..]))
}

/// The newest release whose title list the bulk page links.
pub fn latest(page: &str) -> Option<String> {
    page.match_indices("CPCTitleList")
        .filter_map(|(i, m)| {
            let rest = &page[i + m.len()..];
            let digits = rest.get(..6)?;
            (digits.bytes().all(|b| b.is_ascii_digit()) && rest[6..].starts_with(".zip"))
                .then(|| digits.to_string())
        })
        .max()
}

/// Download the title list for `release` (default: the newest on the
/// bulk page) into `cache`, returning its lock entry and release.
pub fn fetch(cache: &Path, release: Option<&str>) -> Result<(String, Locked), String> {
    let release = match release {
        Some(r) => r.to_string(),
        None => {
            eprintln!("fetch {BULK}");
            let page = String::from_utf8_lossy(&download(BULK)?).into_owned();
            latest(&page).ok_or("the bulk page links no CPCTitleListYYYYMM.zip")?
        }
    };
    let version = version(&release)?;
    let file = format!("CPCTitleList{release}.zip");
    let url = format!("{FILES}{file}");
    eprintln!("fetch {url}");
    let bytes = download(&url)?;
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let path = cache.join(&file);
    std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let locked = Locked {
        file,
        url,
        fetched: today(),
        bytes: bytes.len() as u64,
        sha256: sha256_hex(&bytes),
    };
    Ok((version, locked))
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let mut last = String::new();
    for _ in 0..3 {
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

/// Read the locked archive from `cache`, refusing one whose hash differs.
pub fn read(cache: &Path, locked: &Locked) -> Result<Vec<u8>, String> {
    let path = cache.join(&locked.file);
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{}: {e}; run `cpc-scheme fetch` or download {} there",
            path.display(),
            locked.url
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
    fn versions() {
        assert_eq!(version("202608").unwrap(), "2026.08");
        for bad in ["2026", "202613", "20260a", "2026080"] {
            assert!(version(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn finds_the_newest_release_on_the_bulk_page() {
        let page = r#"<a href="/sites/default/files/cpc/bulk/CPCTitleList202605.zip">May</a>
            <a href="/sites/default/files/cpc/bulk/CPCSchemeXML202608.zip">XML</a>
            <a href="/sites/default/files/cpc/bulk/CPCTitleList202608.zip">Aug</a>
            <a href="CPCTitleList2026.zip">bad</a>"#;
        assert_eq!(latest(page).as_deref(), Some("202608"));
        assert_eq!(latest("nothing here"), None);
    }

    #[test]
    fn refuses_a_changed_archive() {
        let dir = std::env::temp_dir().join(format!("cpc-scheme-read-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("x.zip"), b"abc").unwrap();
        let mut l = Locked {
            file: "x.zip".into(),
            url: "u".into(),
            fetched: "2026-10-08".into(),
            bytes: 3,
            sha256: sha256_hex(b"abc"),
        };
        assert_eq!(read(&dir, &l).unwrap(), b"abc");
        l.sha256 = sha256_hex(b"abd");
        assert!(read(&dir, &l).unwrap_err().contains("refetch"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
