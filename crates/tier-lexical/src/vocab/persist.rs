//! The matcher's on-disk form: a built [`VocabMatcher`] saved next to the
//! store so a new process loads it instead of rebuilding it.
//!
//! A file holds one matcher, keyed by the content hash of the index it was
//! built from. It is a cache: a file that is missing, unreadable, from
//! another format or for another index is ignored and the matcher is
//! rebuilt, and a failed write only costs the next process a rebuild.

use std::path::{Path, PathBuf};

use shelfmark_core::scheme::CompiledIndex;

use super::matcher::VocabMatcher;

/// Leading bytes of every matcher file.
const MAGIC: &[u8; 8] = b"SMVOCAB\0";
/// Bumped whenever the saved form or the build changes, so old files miss.
pub const FORMAT: u32 = 2;

/// Where the matcher for the index with `content_hash` lives in `dir`.
fn path(dir: &Path, content_hash: &str) -> PathBuf {
    let hex = content_hash.strip_prefix("s256:").unwrap_or(content_hash);
    dir.join(format!("v{FORMAT}-{hex}.bin"))
}

impl VocabMatcher {
    /// The saved form of this matcher.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&FORMAT.to_le_bytes());
        postcard::to_extend(self, out).expect("a matcher serializes")
    }

    /// A matcher saved by [`VocabMatcher::to_bytes`], if `bytes` hold one
    /// in this format built from `index` as it is now.
    pub fn from_bytes(bytes: &[u8], index: &dyn CompiledIndex) -> Option<Self> {
        let body = bytes.strip_prefix(MAGIC.as_slice())?;
        let (format, body) = body.split_first_chunk::<4>()?;
        if u32::from_le_bytes(*format) != FORMAT {
            return None;
        }
        let matcher: VocabMatcher = postcard::from_bytes(body).ok()?;
        matcher.is_for(index).then_some(matcher)
    }

    /// The matcher saved for `index` in `dir`, if there is a current one.
    pub fn load(dir: &Path, index: &dyn CompiledIndex) -> Option<Self> {
        let bytes = std::fs::read(path(dir, index.content_hash())).ok()?;
        Self::from_bytes(&bytes, index)
    }

    /// Save this matcher in `dir` via a temporary file and a rename, so a
    /// reader never sees half a file.
    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        let target = path(dir, self.content_hash());
        std::fs::create_dir_all(dir)?;
        let tmp = target.with_extension(format!("tmp.{}", std::process::id()));
        std::fs::write(&tmp, self.to_bytes())?;
        std::fs::rename(&tmp, &target)
    }
}
