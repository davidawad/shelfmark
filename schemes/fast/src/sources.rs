//! The source files, their lock (URL, mirror, fetch date, size, sha256)
//! and the fetch that fills the cache. A build reads only cached files
//! whose hashes match the lock, so the same lock always builds the same
//! scheme.
//!
//! OCLC serves the files behind a bot challenge that refuses scripted
//! downloads, so each locked file also names a pinned Internet Archive
//! copy of the same release; fetch tries OCLC first and falls back to the
//! mirror. Either way the bytes must match the locked sha256.

use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

pub const DOWNLOAD_PAGE: &str =
    "https://www.oclc.org/research/areas/data-science/fast/download.html";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    /// The FAST release the files carry, as OCLC dates it.
    pub release: String,
    pub sources: Vec<Locked>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locked {
    /// File name in the cache, e.g. `FASTTopical.nt.zip`.
    pub file: String,
    /// The N-Triples member inside the zip.
    pub member: String,
    pub url: String,
    /// A pinned copy of the same bytes.
    pub mirror: String,
    /// When the locked bytes were fetched (UTC date).
    pub fetched: String,
    pub bytes: u64,
    pub sha256: String,
}

pub fn read_lock(path: &Path) -> Result<Lock, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn cached(cache: &Path, l: &Locked) -> PathBuf {
    cache.join(&l.file)
}

/// The cached bytes of a locked file, refused unless they hash to the lock.
pub fn read(cache: &Path, l: &Locked) -> Result<Vec<u8>, String> {
    let path = cached(cache, l);
    let bytes = std::fs::read(&path).map_err(|e| {
        format!(
            "{}: {e}; run `fast-scheme fetch --cache {}`",
            path.display(),
            cache.display()
        )
    })?;
    verify(&bytes, l).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(bytes)
}

pub fn verify(bytes: &[u8], l: &Locked) -> Result<(), String> {
    let got = sha256_hex(bytes);
    if got != l.sha256 {
        return Err(format!("sha256 {got} is not the locked {}", l.sha256));
    }
    Ok(())
}

/// Download every locked file the cache lacks or holds with the wrong
/// hash, from the URL or else the mirror. Never rewrites the lock.
pub fn fetch(cache: &Path, lock: &Lock) -> Result<(), String> {
    std::fs::create_dir_all(cache).map_err(|e| format!("{}: {e}", cache.display()))?;
    for l in &lock.sources {
        let path = cached(cache, l);
        if std::fs::read(&path).is_ok_and(|b| verify(&b, l).is_ok()) {
            eprintln!("fast-scheme: {} is cached", l.file);
            continue;
        }
        let mut last = String::new();
        for url in [&l.url, &l.mirror] {
            eprintln!("fast-scheme: fetching {url}");
            match get(url).and_then(|b| verify(&b, l).map(|()| b)) {
                Ok(bytes) => {
                    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
                    last.clear();
                    break;
                }
                Err(e) => {
                    eprintln!("fast-scheme: {url}: {e}");
                    last = e;
                }
            }
        }
        if !last.is_empty() {
            return Err(format!(
                "{}: no source gave the locked bytes ({last})",
                l.file
            ));
        }
    }
    Ok(())
}

fn get(url: &str) -> Result<Vec<u8>, String> {
    let resp = ureq::get(url)
        .header(
            "User-Agent",
            "shelfmark-fast-scheme/0.1 (+https://gitlab.com/davidawad/shelfmark)",
        )
        .call()
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    resp.into_body()
        .into_reader()
        .read_to_end(&mut out)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

/// The locked N-Triples member of a zip, as a reader.
pub fn member<'a>(
    zip: &'a mut zip::ZipArchive<std::io::Cursor<Vec<u8>>>,
    l: &Locked,
) -> Result<impl Read + 'a, String> {
    zip.by_name(&l.member)
        .map_err(|e| format!("{}: {}: {e}", l.file, l.member))
}
