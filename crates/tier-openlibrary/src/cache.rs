//! The on-disk response cache: `<dir>/<sha256 of the URL>.json`, the raw
//! body of a 200 response. There is no expiry; delete the directory to
//! refresh. A file that cannot be read is a miss, and a failed write is
//! ignored: the cache only saves requests.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct DiskCache {
    dir: PathBuf,
}

impl DiskCache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        DiskCache { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Where the body for `url` lives.
    pub fn path(&self, url: &str) -> PathBuf {
        let hex: String = Sha256::digest(url.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        self.dir.join(format!("{hex}.json"))
    }

    pub fn get(&self, url: &str) -> Option<String> {
        std::fs::read_to_string(self.path(url)).ok()
    }

    /// Store `body` for `url` through a temporary file renamed into
    /// place, so a reader never sees half a body.
    pub fn put(&self, url: &str, body: &str) {
        let path = self.path(url);
        if std::fs::create_dir_all(&self.dir).is_err() {
            return;
        }
        let tmp = path.with_extension(format!("tmp{}", std::process::id()));
        if std::fs::write(&tmp, body).is_ok() && std::fs::rename(&tmp, &path).is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
    }
}
