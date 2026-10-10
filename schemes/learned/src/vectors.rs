//! Embedding vectors: the raw f32 file written by the graph script, and the
//! compact i8 form (one scale per vector) committed for insertion.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

/// Reads `n x dims` little-endian f32 rows.
pub fn read_f32(path: &Path, n: usize) -> Result<Vec<Vec<f32>>, String> {
    let raw = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if n == 0 || raw.len() % (4 * n) != 0 {
        return Err(format!(
            "{}: {} bytes is not {n} f32 rows",
            path.display(),
            raw.len()
        ));
    }
    let dims = raw.len() / 4 / n;
    Ok(raw
        .chunks_exact(4 * dims)
        .map(|row| {
            row.as_chunks::<4>()
                .0
                .iter()
                .map(|b| f32::from_le_bytes(*b))
                .collect()
        })
        .collect())
}

/// One vector as `scale * q`, with `scale = max |x| / 127`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantized {
    pub scale: f64,
    pub q: Vec<i8>,
}

impl Quantized {
    pub fn new(v: &[f64]) -> Self {
        let max = v.iter().fold(0.0f64, |m, x| m.max(x.abs()));
        let scale = if max > 0.0 { max / 127.0 } else { 1.0 };
        let q = v
            .iter()
            .map(|x| (x / scale).round().clamp(-127.0, 127.0) as i8)
            .collect();
        // Round the scale to 9 significant digits so the JSON is stable.
        let scale = format!("{scale:.9e}").parse().unwrap_or(scale);
        Quantized { scale, q }
    }
    /// The unit-length vector this stands for.
    pub fn unit(&self) -> Vec<f64> {
        let mut v: Vec<f64> = self.q.iter().map(|&x| f64::from(x) * self.scale).collect();
        normalize(&mut v);
        v
    }
}

pub fn normalize(v: &mut [f64]) {
    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm > 0.0 {
        v.iter_mut().for_each(|x| *x /= norm);
    }
}

pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Committed vectors for a set of rows (e.g. the holdout).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorFile {
    pub model: String,
    pub dims: usize,
    pub rows: Vec<VectorRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRow {
    pub id: String,
    #[serde(flatten)]
    pub v: Quantized,
}

pub const MODEL: &str = "sentence-transformers/all-MiniLM-L6-v2 (Qdrant/all-MiniLM-L6-v2-onnx d13954661f83248295ba75c1ed411eef3b7b936e, fastembed 0.9.0)";

/// Writes JSON, gzipped when the name ends in `.gz` (fixed header, so the
/// bytes depend only on the content).
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let mut text = serde_json::to_string(value).map_err(|e| e.to_string())?;
    text.push('\n');
    let bytes = if path.extension().is_some_and(|x| x == "gz") {
        let mut enc = flate2::GzBuilder::new()
            .mtime(0)
            .write(Vec::new(), flate2::Compression::best());
        enc.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        enc.finish().map_err(|e| e.to_string())?
    } else {
        text.into_bytes()
    };
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&crate::graph::read_maybe_gz(path)?)
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantization_keeps_direction() {
        let v = vec![0.3, -0.5, 0.1, 0.8];
        let q = Quantized::new(&v);
        assert_eq!(q.q[3], 127);
        let mut unit = v.clone();
        normalize(&mut unit);
        assert!(dot(&q.unit(), &unit) > 0.9999);
    }
}
