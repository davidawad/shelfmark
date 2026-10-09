//! The OpenAlex API pages, their lock and the fetch that writes it.
//!
//! OpenAlex serves only its current data, and the raw topic pages (25
//! keyword objects per topic) run to about 16 MB, so the pages are not
//! committed. `fetch` downloads them into a local cache and pins each
//! page's sha256 in the lock; `build` reads only cached pages whose hash
//! matches the lock. The built scheme file is the committed record.

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use shelfmark_core::id::sha256_hex;

pub const API: &str = "https://api.openalex.org";
/// The generic contact address the OpenAlex polite pool asks for. It is
/// deliberately not a personal address.
pub const MAILTO: &str = "shelfmark@example.org";
/// The page that states OpenAlex's CC0 terms.
pub const LICENSE_URL: &str = "https://help.openalex.org/hc/en-us/articles/24397762024087-Pricing";
const PER_PAGE: usize = 200;
const PAUSE: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Domains,
    Fields,
    Subfields,
    Topics,
    /// The page stating the CC0 dedication; checked, not hash-pinned.
    License,
}

impl Role {
    /// The OpenAlex entity endpoint and the fields kept from it.
    fn query(self) -> Option<(&'static str, &'static str)> {
        match self {
            Role::Domains => Some((
                "domains",
                "id,display_name,description,display_name_alternatives",
            )),
            Role::Fields => Some((
                "fields",
                "id,display_name,description,display_name_alternatives,domain",
            )),
            Role::Subfields => Some((
                "subfields",
                "id,display_name,description,display_name_alternatives,field,domain",
            )),
            Role::Topics => Some((
                "topics",
                "id,display_name,description,keywords,subfield,field,domain",
            )),
            Role::License => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locked {
    /// File name under the cache directory.
    pub file: String,
    pub url: String,
    pub role: Role,
    /// UTC date (`YYYY-MM-DD`) of the fetch.
    pub fetched: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    pub sources: Vec<Locked>,
}

impl Lock {
    pub fn read(path: &Path) -> Result<Lock, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn of(&self, role: Role) -> impl Iterator<Item = &Locked> {
        self.sources.iter().filter(move |l| l.role == role)
    }

    /// The date the snapshot began (earliest fetch), used as the version.
    pub fn snapshot_date(&self) -> Result<String, String> {
        self.sources
            .iter()
            .map(|l| l.fetched.clone())
            .min()
            .ok_or_else(|| "the lock is empty".to_string())
    }
}

/// Read a cached file; for hash-pinned roles, refuse one whose hash
/// differs from the lock.
pub fn read(cache: &Path, locked: &Locked) -> Result<Vec<u8>, String> {
    let path = cache.join(&locked.file);
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{}: {e}; run `openalex-topics-scheme fetch`",
            path.display()
        )
    })?;
    let got = sha256_hex(&bytes);
    if locked.role != Role::License && got != locked.sha256 {
        return Err(format!(
            "{} has sha256 {got}, the lock says {}; refetch and review the diff",
            path.display(),
            locked.sha256
        ));
    }
    Ok(bytes)
}

/// The statement the license page must still make.
pub fn check_license(page: &str) -> Result<(), String> {
    if page.contains("CC0") && page.contains("public-domain") {
        Ok(())
    } else {
        Err(format!(
            "{LICENSE_URL} no longer says the data is CC0 public domain; review the new terms"
        ))
    }
}

/// Fetch every source into `cache` and return the new lock.
pub fn fetch(cache: &Path) -> Result<Lock, String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let fetched = today();
    let mut sources = Vec::new();

    let page = download(LICENSE_URL)?;
    check_license(&String::from_utf8_lossy(&page))?;
    sources.push(save(
        cache,
        "license.html",
        LICENSE_URL,
        Role::License,
        &fetched,
        &page,
    )?);

    for role in [Role::Domains, Role::Fields, Role::Subfields, Role::Topics] {
        let (entity, select) = role.query().expect("data roles have a query");
        sources.extend(fetch_entity(cache, role, entity, select, &fetched)?);
    }
    Ok(Lock { sources })
}

fn fetch_entity(
    cache: &Path,
    role: Role,
    entity: &str,
    select: &str,
    fetched: &str,
) -> Result<Vec<Locked>, String> {
    let mut out = Vec::new();
    let mut seen = 0usize;
    let mut cursor = "*".to_string();
    loop {
        let url = format!(
            "{API}/{entity}?per-page={PER_PAGE}&select={select}&cursor={cursor}&mailto={MAILTO}"
        );
        sleep(PAUSE);
        let body = download(&url)?;
        let v: Value = serde_json::from_slice(&body).map_err(|e| format!("{url}: {e}"))?;
        let n = v["results"].as_array().map_or(0, Vec::len);
        let count = v["meta"]["count"].as_u64().unwrap_or(0) as usize;
        if n == 0 {
            if seen != count {
                return Err(format!("{entity}: fetched {seen} of {count}; rerun fetch"));
            }
            return Ok(out);
        }
        seen += n;
        let name = format!("{entity}-{:03}.json", out.len() + 1);
        out.push(save(cache, &name, &url, role, fetched, &body)?);
        match v["meta"]["next_cursor"].as_str() {
            Some(next) => cursor = next.to_string(),
            None => return Ok(out),
        }
    }
}

fn save(
    cache: &Path,
    name: &str,
    url: &str,
    role: Role,
    fetched: &str,
    bytes: &[u8],
) -> Result<Locked, String> {
    let path = cache.join(name);
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Locked {
        file: name.to_string(),
        url: url.to_string(),
        role,
        fetched: fetched.to_string(),
        bytes: bytes.len() as u64,
        sha256: sha256_hex(bytes),
    })
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    eprintln!("fetch {url}");
    let mut last = String::new();
    for attempt in 1..=4u64 {
        match ureq::get(url)
            .header(
                "User-Agent",
                "shelfmark-openalex-topics-scheme (mailto:shelfmark@example.org)",
            )
            .call()
        {
            Ok(mut resp) => match resp.body_mut().read_to_vec() {
                Ok(bytes) => return Ok(bytes),
                Err(e) => last = e.to_string(),
            },
            Err(e) => last = e.to_string(),
        }
        sleep(Duration::from_secs(2 * attempt));
    }
    Err(format!("fetch {url}: {last}"))
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
    fn license_page_must_say_cc0() {
        assert!(check_license("a CC0 public-domain license").is_ok());
        assert!(check_license("a CC BY license").is_err());
    }

    #[test]
    fn snapshot_date_is_the_earliest_fetch() {
        let l = |d: &str| Locked {
            file: "f".into(),
            url: "u".into(),
            role: Role::Topics,
            fetched: d.into(),
            bytes: 0,
            sha256: String::new(),
        };
        let lock = Lock {
            sources: vec![l("2026-10-10"), l("2026-10-09")],
        };
        assert_eq!(lock.snapshot_date().unwrap(), "2026-10-09");
    }
}
