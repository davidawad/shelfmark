//! The one source file, its lock (URL, fetch date, size, sha256) and the
//! fetch that fills the cache. A build reads only a cached file whose hash
//! matches the lock, so the same lock always builds the same scheme.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

/// The Publications Office page that publishes EuroVoc and its downloads.
pub const PAGE: &str =
    "https://op.europa.eu/en/web/eu-vocabularies/th-dataset/-/resource/dataset/eurovoc";
/// The SKOS-core distribution: a zip holding one RDF/XML file.
pub const SOURCE_URL: &str = "https://op.europa.eu/o/opportal-service/euvoc-download-handler?cellarURI=http%3A%2F%2Fpublications.europa.eu%2Fresource%2Fcellar%2F9f2bd600-ae7b-11e7-837e-01aa75ed71a1.0001.09%2FDOC_1&fileName=eurovoc_in_skos_core_concepts.zip";
pub const SOURCE_FILE: &str = "eurovoc_in_skos_core_concepts.zip";

/// The license check (license check), recorded with the lock so the build
/// and the README state the same terms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct License {
    pub holder: String,
    pub terms: String,
    pub terms_url: String,
    pub attribution: String,
    /// Whether the data may ship inside shelfmark's MIT/Apache binary.
    pub redistributable: bool,
}

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
    /// Scheme version, part of every class ID (`scheme:eurovoc@<version>`).
    pub version: String,
    pub license: License,
    pub source: Locked,
}

impl Lock {
    pub fn read(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map(|s| s + "\n")
            .map_err(|e| e.to_string())
    }

    /// A lock for `bytes` fetched on `fetched`, keeping this lock's version
    /// and license.
    pub fn relocked(&self, bytes: &[u8], fetched: String) -> Self {
        Self {
            version: self.version.clone(),
            license: self.license.clone(),
            source: Locked {
                file: SOURCE_FILE.to_string(),
                url: SOURCE_URL.to_string(),
                fetched,
                bytes: bytes.len() as u64,
                sha256: sha256_hex(bytes),
            },
        }
    }

    /// `bytes` are the locked source, or an error naming both hashes.
    pub fn verify(&self, bytes: &[u8]) -> Result<(), String> {
        let got = sha256_hex(bytes);
        if got == self.source.sha256 && bytes.len() as u64 == self.source.bytes {
            return Ok(());
        }
        Err(format!(
            "{} has sha256 {got} ({} bytes) but the lock pins {} ({} bytes); the Publications Office \
             changed the file or the cache is corrupt. Review the new codes, then \
             `eurovoc-scheme fetch --cache DIR --relock` (and bump `version` in the lock \
             if codes changed)",
            self.source.file,
            bytes.len(),
            self.source.sha256,
            self.source.bytes
        ))
    }
}

/// The cached source in `cache`, downloaded first when absent, checked
/// against the lock.
pub fn cached(cache: &Path, lock: &Lock) -> Result<Vec<u8>, String> {
    let path = cache.join(&lock.source.file);
    if !path.exists() {
        store(&path, &download(&lock.source.url)?)?;
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    lock.verify(&bytes)?;
    Ok(bytes)
}

/// Download the source into `cache` regardless of what is there; returns
/// its path and bytes.
pub fn fetch(cache: &Path) -> Result<(PathBuf, Vec<u8>), String> {
    let bytes = download(SOURCE_URL)?;
    let path = cache.join(SOURCE_FILE);
    store(&path, &bytes)?;
    Ok((path, bytes))
}

fn store(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    eprintln!("fetch {url}");
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

/// The one `.rdf` file inside the distribution zip. Fails on a zip that
/// holds none or several, so a changed layout is noticed.
pub fn rdf_from_zip(zip_bytes: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes))
        .map_err(|e| format!("source zip: {e}"))?;
    let names: Vec<String> = zip
        .file_names()
        .filter(|n| n.ends_with(".rdf"))
        .map(str::to_string)
        .collect();
    let [name] = names.as_slice() else {
        return Err(format!("expected one .rdf in the zip, found {names:?}"));
    };
    let mut out = Vec::new();
    zip.by_name(name)
        .map_err(|e| format!("{name}: {e}"))?
        .read_to_end(&mut out)
        .map_err(|e| format!("{name}: {e}"))?;
    Ok(out)
}

/// Today's UTC date, `YYYY-MM-DD`.
pub fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    civil(i64::try_from(secs / 86_400).unwrap_or(0))
}

/// The proleptic Gregorian date `days` after 1970-01-01 (Hinnant's
/// civil_from_days).
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
}
