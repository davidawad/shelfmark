//! The on-disk label-vector cache: `<cache_dir>/shortlist/<model_id>/<hex>.f32`.
//!
//! `hex` is a SHA-256 over the format version, the model ID, the label
//! prefix and every label code and text in order, so any change to what
//! would be embedded picks a new file. The file is
//!
//! ```text
//! magic "SMSHORTL" | version u32 | count u32 | dim u32
//! count x (code length u32 | code UTF-8 bytes)
//! count x dim f32
//! ```
//!
//! all little-endian. Anything that fails to read, parse or match is a
//! miss and the caller rebuilds. Writes go to a temporary file renamed into
//! place, and a failed write is ignored: the cache is an optimization.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::index::{Label, CACHE_FORMAT_VERSION};

const MAGIC: &[u8; 8] = b"SMSHORTL";

/// The cache key for embedding `labels` with `model_id` and `prefix`.
pub(crate) fn key(model_id: &str, prefix: &str, labels: &[Label]) -> String {
    let mut h = Sha256::new();
    let mut field = |bytes: &[u8]| {
        h.update((bytes.len() as u64).to_le_bytes());
        h.update(bytes);
    };
    field(&CACHE_FORMAT_VERSION.to_le_bytes());
    field(model_id.as_bytes());
    field(prefix.as_bytes());
    for l in labels {
        field(l.code.as_bytes());
        field(l.text.as_bytes());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Where the vectors for `key` live under `cache_dir`.
pub(crate) fn path(cache_dir: &Path, model_id: &str, key: &str) -> PathBuf {
    cache_dir
        .join("shortlist")
        .join(sanitize(model_id))
        .join(format!("{key}.f32"))
}

/// A model ID safe as one path component.
fn sanitize(model_id: &str) -> String {
    let s: String = model_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() || s.chars().all(|c| c == '.') {
        "_".into()
    } else {
        s
    }
}

/// The vectors stored at `path` for exactly these `labels`: `(dim, flat
/// row-major vectors)`, or `None` on any mismatch.
pub(crate) fn read(path: &Path, labels: &[Label]) -> Option<(usize, Vec<f32>)> {
    let bytes = std::fs::read(path).ok()?;
    let mut r = Reader { bytes: &bytes };
    if r.take(MAGIC.len())? != MAGIC || r.u32()? != CACHE_FORMAT_VERSION {
        return None;
    }
    let count = r.u32()? as usize;
    let dim = r.u32()? as usize;
    if count != labels.len() || dim == 0 {
        return None;
    }
    for l in labels {
        let len = r.u32()? as usize;
        if r.take(len)? != l.code.as_bytes() {
            return None;
        }
    }
    let floats = count.checked_mul(dim)?;
    let raw = r.take(floats.checked_mul(4)?)?;
    if !r.bytes.is_empty() {
        return None;
    }
    let vectors: Vec<f32> = raw
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    vectors
        .iter()
        .all(|x| x.is_finite())
        .then_some((dim, vectors))
}

/// Store `vectors` (flat, `labels.len() x dim`) at `path`, atomically.
/// Errors are ignored.
pub(crate) fn write(path: &Path, labels: &[Label], dim: usize, vectors: &[f32]) {
    let _ = try_write(path, labels, dim, vectors);
}

fn try_write(path: &Path, labels: &[Label], dim: usize, vectors: &[f32]) -> std::io::Result<()> {
    let to_u32 = |n: usize| {
        u32::try_from(n).map_err(|_| std::io::Error::other("label cache field too large"))
    };
    let mut out = Vec::with_capacity(16 + labels.len() * 8 + vectors.len() * 4);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&CACHE_FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&to_u32(labels.len())?.to_le_bytes());
    out.extend_from_slice(&to_u32(dim)?.to_le_bytes());
    for l in labels {
        out.extend_from_slice(&to_u32(l.code.len())?.to_le_bytes());
        out.extend_from_slice(l.code.as_bytes());
    }
    for x in vectors {
        out.extend_from_slice(&x.to_le_bytes());
    }
    let dir = path
        .parent()
        .ok_or_else(|| std::io::Error::other("cache path has no parent"))?;
    std::fs::create_dir_all(dir)?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let tmp = dir.join(format!(
        ".{}.{}.{nanos}.tmp",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("labels"),
        std::process::id()
    ));
    let result = std::fs::write(&tmp, &out).and_then(|()| std::fs::rename(&tmp, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.bytes.len() < n {
            return None;
        }
        let (head, rest) = self.bytes.split_at(n);
        self.bytes = rest;
        Some(head)
    }

    fn u32(&mut self) -> Option<u32> {
        let b = self.take(4)?;
        Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(code: &str, text: &str) -> Label {
        Label {
            code: code.into(),
            level: 0,
            parent: None,
            text: text.into(),
        }
    }

    fn tmp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "shelfmark-shortlist-cache-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn round_trip_and_rejects_mismatch_or_corruption() {
        let dir = tmp_dir("rt");
        let labels = vec![label("A", "General"), label("B", "Philosophy")];
        let k = key("m", "q: ", &labels);
        let p = path(&dir, "m", &k);
        let v = vec![1.0, 0.0, 0.0, 1.0];
        write(&p, &labels, 2, &v);
        assert_eq!(read(&p, &labels), Some((2, v.clone())));

        let other = vec![label("A", "General"), label("C", "History")];
        assert_eq!(read(&p, &other), None, "codes must match");
        assert_eq!(read(&p, &labels[..1]), None, "count must match");

        let mut bytes = std::fs::read(&p).unwrap();
        bytes.push(0);
        std::fs::write(&p, &bytes).unwrap();
        assert_eq!(read(&p, &labels), None, "trailing bytes");
        bytes.truncate(bytes.len() - 5);
        std::fs::write(&p, &bytes).unwrap();
        assert_eq!(read(&p, &labels), None, "truncated");
        std::fs::write(&p, b"nope").unwrap();
        assert_eq!(read(&p, &labels), None, "bad magic");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn key_covers_model_prefix_codes_and_texts() {
        let a = vec![label("A", "General")];
        let base = key("m", "p", &a);
        assert_eq!(base.len(), 64);
        assert_eq!(base, key("m", "p", &a));
        assert_ne!(base, key("n", "p", &a));
        assert_ne!(base, key("m", "", &a));
        assert_ne!(base, key("m", "p", &[label("A", "General works")]));
        assert_ne!(base, key("m", "p", &[label("B", "General")]));
        // Length-prefixed fields: moving a boundary changes the key.
        assert_ne!(key("ab", "c", &a), key("a", "bc", &a));
    }

    #[test]
    fn model_ids_become_one_path_component() {
        assert_eq!(sanitize("bge-small-en-v1.5-q"), "bge-small-en-v1.5-q");
        assert_eq!(sanitize("org/model:x"), "org_model_x");
        assert_eq!(sanitize(".."), "_");
    }

    #[test]
    fn write_failure_is_ignored() {
        let dir = tmp_dir("ro");
        std::fs::write(&dir, b"a file, not a directory").unwrap();
        let p = path(&dir, "m", "k");
        write(&p, &[label("A", "x")], 1, &[1.0]);
        assert_eq!(read(&p, &[label("A", "x")]), None);
        let _ = std::fs::remove_file(&dir);
    }
}
