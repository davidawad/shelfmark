//! The compile cache: compiled indexes on disk, keyed by the source
//! file's content hash, plus a pointer per scheme ID naming its latest
//! compile. Layout (private, plan PD7):
//!
//! ```text
//! <dir>/objects/<hex>.json           compiled form of the file with that hash
//! <dir>/schemes/<name>@<version>.json  {content_hash, source} of the latest compile
//! ```

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use shelfmark_core::id::SchemeId;
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::{Error, ErrorCode};

use crate::index::{SchemeIndex, Stored, FORMAT};
use crate::{content_hash, read, validate_source};

/// Whether a compile reused the cached index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheStatus {
    Hit,
    Miss,
}

#[derive(Debug)]
pub struct Compiled {
    pub index: SchemeIndex,
    pub status: CacheStatus,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pointer {
    content_hash: String,
    source: String,
}

/// A directory of compiled schemes.
#[derive(Debug, Clone)]
pub struct CompileCache {
    dir: PathBuf,
}

fn store_error(what: &str, path: &Path, e: impl std::fmt::Display) -> Error {
    Error::new(
        ErrorCode::StoreError,
        format!("cannot {what} {}: {e}", path.display()),
    )
}

/// Write via a temporary file and rename, so readers never see half a file.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let dir = path.parent().expect("cache paths have a parent");
    std::fs::create_dir_all(dir).map_err(|e| store_error("create", dir, e))?;
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, bytes).map_err(|e| store_error("write", &tmp, e))?;
    std::fs::rename(&tmp, path).map_err(|e| store_error("write", path, e))
}

impl CompileCache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        CompileCache { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn object(&self, hash: &str) -> PathBuf {
        let hex = hash.strip_prefix("s256:").unwrap_or(hash);
        self.dir.join("objects").join(format!("{hex}.json"))
    }

    fn pointer(&self, scheme: &SchemeId) -> PathBuf {
        self.dir.join("schemes").join(format!("{}.json", scheme.0))
    }

    /// A cached index for `hash`, if present and current.
    fn load(&self, hash: &str) -> Option<SchemeIndex> {
        let bytes = std::fs::read(self.object(hash)).ok()?;
        let stored: Stored = serde_json::from_slice(&bytes).ok()?;
        if stored.format != FORMAT || stored.content_hash != hash {
            return None;
        }
        SchemeIndex::from_stored(stored).ok()
    }

    /// Compile a scheme file, reusing the cached index when its bytes are
    /// unchanged.
    pub fn compile_file(&self, path: &Path) -> Result<Compiled, Error> {
        self.compile_bytes(&read(path)?, &path.display().to_string())
    }

    /// Compile scheme bytes; `source` names the file in errors and in the
    /// scheme's pointer.
    pub fn compile_bytes(&self, bytes: &[u8], source: &str) -> Result<Compiled, Error> {
        let compiled = self.compile_object(bytes, source)?;
        let pointer = Pointer {
            content_hash: compiled.index.content_hash().to_string(),
            source: source.to_string(),
        };
        let json = serde_json::to_vec_pretty(&pointer)
            .map_err(|e| Error::internal(format!("pointer serializes: {e}")))?;
        write_atomic(&self.pointer(compiled.index.scheme()), &json)?;
        Ok(compiled)
    }

    /// Like [`CompileCache::compile_bytes`], but leaves the scheme's
    /// pointer alone: for callers that keep the bytes themselves and only
    /// want to skip validating and compiling them again. A hit writes
    /// nothing.
    pub fn compile_object(&self, bytes: &[u8], source: &str) -> Result<Compiled, Error> {
        let hash = content_hash(bytes);
        if let Some(index) = self.load(&hash) {
            return Ok(Compiled {
                index,
                status: CacheStatus::Hit,
            });
        }
        let doc = validate_source(bytes, source)?;
        let stored = Stored::build(&doc, hash.clone());
        let json = serde_json::to_vec(&stored)
            .map_err(|e| Error::internal(format!("compiled index serializes: {e}")))?;
        write_atomic(&self.object(&hash), &json)?;
        Ok(Compiled {
            index: SchemeIndex::from_stored(stored)?,
            status: CacheStatus::Miss,
        })
    }

    /// The latest compiled index for `scheme`. A scheme with no compiled
    /// index is `scheme_not_compiled` naming the compile command (AE7);
    /// `source`, when the caller knows the file, makes that command
    /// runnable as given.
    pub fn open(&self, scheme: &SchemeId, source: Option<&str>) -> Result<SchemeIndex, Error> {
        let pointer: Option<Pointer> = std::fs::read(self.pointer(scheme))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok());
        let found = pointer
            .as_ref()
            .and_then(|p| self.load(&p.content_hash))
            .filter(|index| index.scheme() == scheme);
        found.ok_or_else(|| {
            let known = pointer.as_ref().map(|p| p.source.as_str());
            Error::scheme_not_compiled(scheme, source.or(known))
        })
    }
}
