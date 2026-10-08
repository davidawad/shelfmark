//! The source files, their lock and the fetch that writes it.
//!
//! The taxonomy (`definitions.py`, kept as `definitions.py.txt`) and
//! arxiv-base's MIT `LICENSE` are fetched at pinned commits and kept in
//! the repository under `sources/`, so a build needs no network. The taxonomy page is arXiv's website,
//! not covered by that license, so it stays in a local cache and is only
//! used to cross-check. A build reads only files whose sha256 matches the
//! lock.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::sha256_hex;

pub const PAGE: &str = "https://arxiv.org/category_taxonomy";
pub const REPO: &str = "arXiv/arxiv-base";
/// arxiv-base's default branch.
pub const BRANCH: &str = "develop";
pub const DEFINITIONS: &str = "arxiv/taxonomy/definitions.py";
pub const LICENSE: &str = "LICENSE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// arxiv-base's taxonomy definitions; kept under `sources/`.
    Definitions,
    /// arxiv-base's license; kept under `sources/`.
    License,
    /// The rendered taxonomy page; kept in the cache.
    Page,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locked {
    /// Path under the scheme directory (`sources/...`) or, for the page,
    /// under the cache.
    pub file: String,
    pub url: String,
    pub role: Role,
    /// The arxiv-base commit the file was fetched at.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// UTC date (`YYYY-MM-DD`) of the last commit that changed the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed: Option<String>,
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
    pub fn read(path: &Path) -> Result<Lock, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn get(&self, role: Role) -> Result<&Locked, String> {
        self.sources
            .iter()
            .find(|l| l.role == role)
            .ok_or_else(|| format!("the lock has no {role:?} source"))
    }
}

/// Where a locked file lives: `sources/` files under the scheme
/// directory, the page under the cache.
pub fn path(data: &Path, cache: Option<&Path>, locked: &Locked) -> Result<PathBuf, String> {
    match locked.role {
        Role::Definitions | Role::License => Ok(data.join(&locked.file)),
        Role::Page => cache
            .map(|c| c.join(&locked.file))
            .ok_or_else(|| "the taxonomy page cross-check needs --cache DIR".into()),
    }
}

/// Read a locked file, refusing one whose hash differs from the lock.
pub fn read(data: &Path, cache: Option<&Path>, locked: &Locked) -> Result<Vec<u8>, String> {
    let path = path(data, cache, locked)?;
    let bytes = std::fs::read(&path)
        .map_err(|e| format!("{}: {e}; run `arxiv-scheme fetch`", path.display()))?;
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

/// The latest commit on [`BRANCH`] that changed `file`: (sha, UTC date).
fn last_commit(file: &str) -> Result<(String, String), String> {
    let url =
        format!("https://api.github.com/repos/{REPO}/commits?path={file}&sha={BRANCH}&per_page=1");
    let body = download(&url)?;
    let v: serde_json::Value = serde_json::from_slice(&body).map_err(|e| format!("{url}: {e}"))?;
    let c = v.get(0).ok_or_else(|| format!("{url}: no commits"))?;
    let sha = c["sha"].as_str().ok_or_else(|| format!("{url}: no sha"))?;
    let date = c["commit"]["committer"]["date"]
        .as_str()
        .and_then(|d| d.get(..10))
        .ok_or_else(|| format!("{url}: no commit date"))?;
    Ok((sha.to_string(), date.to_string()))
}

/// Fetch every source: the two arxiv-base files into `data/sources/`, the
/// page into `cache`. Returns the new lock.
pub fn fetch(data: &Path, cache: &Path) -> Result<Lock, String> {
    let fetched = today();
    let mut sources = Vec::new();
    // The Python file is kept as `.txt`: it is data here, never run, and
    // repository Python rules should not apply to arXiv's code.
    let files = [
        (DEFINITIONS, "sources/definitions.py.txt", Role::Definitions),
        (LICENSE, "sources/LICENSE", Role::License),
    ];
    for (file, name, role) in files {
        let (commit, changed) = last_commit(file)?;
        let url = format!("https://raw.githubusercontent.com/{REPO}/{commit}/{file}");
        let bytes = save(&url, &data.join(name))?;
        sources.push(locked(
            name.into(),
            url,
            role,
            Some(commit),
            Some(changed),
            &fetched,
            &bytes,
        ));
    }
    let name = "category_taxonomy.html".to_string();
    let bytes = save(PAGE, &cache.join(&name))?;
    sources.push(locked(
        name,
        PAGE.into(),
        Role::Page,
        None,
        None,
        &fetched,
        &bytes,
    ));
    Ok(Lock { sources })
}

fn locked(
    file: String,
    url: String,
    role: Role,
    commit: Option<String>,
    changed: Option<String>,
    fetched: &str,
    bytes: &[u8],
) -> Locked {
    Locked {
        file,
        url,
        role,
        commit,
        changed,
        fetched: fetched.to_string(),
        bytes: bytes.len() as u64,
        sha256: sha256_hex(bytes),
    }
}

fn save(url: &str, path: &Path) -> Result<Vec<u8>, String> {
    eprintln!("fetch {url}");
    let bytes = download(url)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(bytes)
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let mut last = String::new();
    for _ in 0..3 {
        match ureq::get(url)
            .header("User-Agent", "shelfmark-arxiv-scheme")
            .call()
        {
            Ok(mut resp) => match resp.body_mut().read_to_vec() {
                Ok(bytes) => return Ok(bytes),
                Err(e) => last = e.to_string(),
            },
            Err(e) => last = e.to_string(),
        }
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
    fn the_page_needs_a_cache() {
        let page = locked(
            "p.html".into(),
            PAGE.into(),
            Role::Page,
            None,
            None,
            "x",
            b"",
        );
        assert!(path(Path::new("d"), None, &page).is_err());
        assert_eq!(
            path(Path::new("d"), Some(Path::new("c")), &page).unwrap(),
            Path::new("c/p.html")
        );
    }
}
