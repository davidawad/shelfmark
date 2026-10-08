//! The source files, their lock (URL, fetch date, size, sha256) and the
//! fetch that writes it. A build reads only cached files whose hashes
//! match the lock, so the same lock always builds the same scheme.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

pub const FREELCC: &str = "https://www.loc.gov/aba/publications/FreeLCC/";
pub const LCCO: &str = "https://www.loc.gov/aba/cataloging/classification/lcco/";
pub const LCSH: &str = "https://id.loc.gov/download/authorities/subjects.madsrdf.nt.gz";
pub const ID_CLASSIFICATION: &str = "http://id.loc.gov/authorities/classification/";

/// The 2024 FreeLCC outline PDFs, one per schedule, in class order.
const OUTLINES: [&str; 39] = [
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
    "KJ-KKZ",
    "KJV-KJW",
    "KK-KKC",
    "KL-KWX",
    "KZ",
    "L",
    "M",
    "N",
    "P-PA",
    "PB-PH",
    "PJ-PK",
    "PL-PM",
    "PN",
    "PQ",
    "PR-PS-PZ",
    "PT",
    "Q",
    "R",
    "S",
    "T",
    "U-V",
    "Z",
];

/// The LC Classification Outline PDFs, one per main class (E and F share).
const CLASS_OUTLINES: [&str; 20] = [
    "a", "b", "c", "d", "ef", "g", "h", "j", "k", "l", "m", "n", "p", "q", "r", "s", "t", "u", "v",
    "z",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Main class and subclass captions.
    ClassOutline,
    /// Numbered spans with captions.
    Outline,
    /// LCSH headings with LCC numbers.
    Lcsh,
    /// One id.loc.gov classification record: the caption of a number the
    /// outline PDFs print without one, or of a subclass the class outlines
    /// do not list.
    IdRecord,
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

/// Every source as (file name, URL, role), in build order.
pub fn all() -> Vec<(String, String, Role)> {
    let mut out = Vec::new();
    for c in CLASS_OUTLINES {
        let file = format!("lcco_{c}.pdf");
        out.push((file.clone(), format!("{LCCO}{file}"), Role::ClassOutline));
    }
    for o in OUTLINES {
        let file = format!("LCC_{o}2024OUT.pdf");
        out.push((file.clone(), format!("{FREELCC}{file}"), Role::Outline));
    }
    out.push((
        "subjects.madsrdf.nt.gz".to_string(),
        LCSH.to_string(),
        Role::Lcsh,
    ));
    out
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

/// Download the id.loc.gov record for each name (`CD1-CD6471`) into
/// `cache`, skipping names id.loc.gov does not have.
pub fn fetch_records(cache: &Path, names: &[String]) -> Result<Vec<Locked>, String> {
    let fetched = today();
    let mut out = Vec::new();
    for name in names {
        let url = format!(
            "{}{name}.json",
            ID_CLASSIFICATION.replace("http:", "https:")
        );
        eprintln!("fetch {url}");
        let bytes = match ureq::get(&url).call() {
            Ok(mut resp) => resp
                .body_mut()
                .read_to_vec()
                .map_err(|e| format!("fetch {url}: {e}"))?,
            Err(ureq::Error::StatusCode(404)) => continue,
            Err(_) => download(&url)?,
        };
        let file = format!("id-classification/{name}.json");
        let path = cache.join(&file);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        out.push(Locked {
            file,
            url,
            role: Role::IdRecord,
            fetched: fetched.clone(),
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
        });
    }
    Ok(out)
}

/// The label of the record named `name` in an id.loc.gov classification
/// JSON-LD document.
pub fn record_label(bytes: &[u8], name: &str) -> Option<String> {
    let doc: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    let id = format!("{ID_CLASSIFICATION}{name}");
    let node = doc.as_array()?.iter().find(|n| n["@id"] == id.as_str())?;
    // Subclass records (`KM`) carry only an rdfs label.
    [
        "http://www.loc.gov/mads/rdf/v1#authoritativeLabel",
        "http://www.w3.org/2000/01/rdf-schema#label",
    ]
    .iter()
    .find_map(|key| node.get(key)?.get(0)?.get("@value")?.as_str())
    .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
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

/// Read a locked file from `cache`, refusing one whose hash has changed.
pub fn read(cache: &Path, locked: &Locked) -> Result<Vec<u8>, String> {
    let path = cache.join(&locked.file);
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{}: {e}; run `lcc-outline fetch` or copy the sources there",
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
        assert_eq!(civil(20_733), "2026-10-07");
        assert_eq!(civil(11_016), "2000-02-29");
    }

    #[test]
    fn reads_record_labels() {
        let doc = br#"[{"@id": "_:b1"}, {"@id": "http://id.loc.gov/authorities/classification/CT21-CT9999",
            "http://www.loc.gov/mads/rdf/v1#authoritativeLabel": [{"@value": "Biography"}]}]"#;
        assert_eq!(
            record_label(doc, "CT21-CT9999").as_deref(),
            Some("Biography")
        );
        assert_eq!(record_label(doc, "CT21"), None);
        let sub = br#"[{"@id": "http://id.loc.gov/authorities/classification/KL",
            "http://www.w3.org/2000/01/rdf-schema#label": [{"@value": "Ancient Orient.  Eurasia"}]}]"#;
        assert_eq!(
            record_label(sub, "KL").as_deref(),
            Some("Ancient Orient. Eurasia")
        );
    }

    #[test]
    fn sixty_sources() {
        let all = all();
        assert_eq!(all.len(), 60);
        assert!(all.iter().any(|(f, _, _)| f == "LCC_Q2024OUT.pdf"));
    }
}
