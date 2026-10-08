//! The source files, their lock (URL, fetch date, size, sha256) and the
//! fetch that writes it. A build reads only cached files whose hashes
//! match the lock, so the same lock always builds the same scheme.

use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

/// The edition the lock pins; `fetch --edition` overrides it.
pub const EDITION: &str = "20260101";

pub const DOWNLOAD_AREA: &str =
    "https://www.wipo.int/classifications/data/ipc/ITSupport_and_download_area/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// The IPC scheme master file (every entry, nested by hierarchy).
    Scheme,
    /// The IPC catchword index master file.
    Catchwords,
}

impl Role {
    /// The archive name and the English member inside it.
    fn names(self, edition: &str) -> (String, String) {
        let stem = match self {
            Role::Scheme => "scheme",
            Role::Catchwords => "catchwordindex",
        };
        (
            format!("ipc_{stem}_{edition}.zip"),
            format!("EN_ipc_{stem}_{edition}.xml"),
        )
    }
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
    /// The XML member of the archive the build reads, and its sha256.
    pub member: String,
    pub member_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    pub edition: String,
    pub sources: Vec<Locked>,
}

impl Lock {
    pub fn get(&self, role: Role) -> Result<&Locked, String> {
        self.sources
            .iter()
            .find(|l| l.role == role)
            .ok_or_else(|| format!("sources.lock.json has no {role:?} source"))
    }
}

/// Download both master files of `edition` into `cache` and return
/// their lock.
pub fn fetch(cache: &Path, edition: &str) -> Result<Lock, String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    let fetched = today();
    let mut sources = Vec::new();
    for role in [Role::Scheme, Role::Catchwords] {
        let (file, member) = role.names(edition);
        let url = format!("{DOWNLOAD_AREA}{edition}/MasterFiles/{file}");
        eprintln!("fetch {url}");
        let bytes = download(&url)?;
        let xml = unzip(&bytes, &member).map_err(|e| format!("{file}: {e}"))?;
        let path = cache.join(&file);
        std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        sources.push(Locked {
            file,
            url,
            role,
            fetched: fetched.clone(),
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
            member,
            member_sha256: sha256_hex(&xml),
        });
    }
    Ok(Lock {
        edition: edition.to_string(),
        sources,
    })
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

/// One member of a zip archive.
fn unzip(archive: &[u8], member: &str) -> Result<Vec<u8>, String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(archive)).map_err(|e| e.to_string())?;
    let mut entry = zip
        .by_name(member)
        .map_err(|e| format!("no member {member}: {e}"))?;
    let mut out = Vec::new();
    entry.read_to_end(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}

/// The locked XML member from the cached archive, refusing an archive or
/// member whose hash has changed.
pub fn read(cache: &Path, locked: &Locked) -> Result<Vec<u8>, String> {
    let path = cache.join(&locked.file);
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{}: {e}; run `ipc-scheme fetch` or copy the archive there",
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
    let xml = unzip(&bytes, &locked.member).map_err(|e| format!("{}: {e}", locked.file))?;
    let got = sha256_hex(&xml);
    if got != locked.member_sha256 {
        return Err(format!(
            "{}:{} has sha256 {got}, the lock says {}",
            locked.file, locked.member, locked.member_sha256
        ));
    }
    Ok(xml)
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
    fn archive_names() {
        assert_eq!(
            Role::Scheme.names("20260101"),
            (
                "ipc_scheme_20260101.zip".to_string(),
                "EN_ipc_scheme_20260101.xml".to_string()
            )
        );
        assert_eq!(
            Role::Catchwords.names("20260101").1,
            "EN_ipc_catchwordindex_20260101.xml"
        );
    }

    #[test]
    fn read_refuses_a_changed_archive() {
        let dir = std::env::temp_dir().join(format!("ipc-scheme-read-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.zip"), b"not the locked bytes").unwrap();
        let locked = Locked {
            file: "a.zip".into(),
            url: String::new(),
            role: Role::Scheme,
            fetched: String::new(),
            bytes: 0,
            sha256: "0".repeat(64),
            member: "a.xml".into(),
            member_sha256: String::new(),
        };
        let err = read(&dir, &locked).unwrap_err();
        assert!(err.contains("the lock says"), "{err}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
