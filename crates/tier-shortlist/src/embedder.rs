//! The embedding model interface the tier ranks with.

use shelfmark_core::error::Error;

/// BGE v1.5's retrieval instruction, prepended to the query side. The
/// tier's labels are the queries (the spike's "flip").
pub const BGE_QUERY_PREFIX: &str = "Represent this sentence for searching relevant passages: ";

/// A text embedding model. Labels are embedded on the query side, items on
/// the passage side; implementations add whatever prefix or instruction
/// that side needs. Vectors need not be normalized (the tier normalizes),
/// but every vector from one model must have the same dimension.
pub trait Embedder: Send + Sync {
    /// Stable model identifier, e.g. `bge-small-en-v1.5-q`. Part of the
    /// tier version and of every label-cache key, so it must change
    /// whenever the vectors would.
    fn model_id(&self) -> &str;

    /// The prefix [`Embedder::embed_labels`] puts before each label, for
    /// the label-cache key. Empty when the model takes none.
    fn label_prefix(&self) -> &str {
        ""
    }

    /// Embed scheme labels (the query side), one vector per text in order.
    fn embed_labels(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Error>;

    /// Embed item texts (the passage side), one vector per text in order.
    fn embed_items(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Error>;
}

/// Scale `v` to unit length in place; a zero (or non-finite) vector
/// becomes all zeros, so it is similar to nothing.
pub fn normalize(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm.is_finite() && norm > 0.0 {
        v.iter_mut().for_each(|x| *x /= norm);
    } else {
        v.iter_mut().for_each(|x| *x = 0.0);
    }
}

/// Dot product: the cosine of two unit vectors.
pub(crate) fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_scales_to_unit_and_zeroes_degenerate_vectors() {
        let mut v = vec![3.0, 4.0];
        normalize(&mut v);
        assert_eq!(v, vec![0.6, 0.8]);
        let mut z = vec![0.0, 0.0];
        normalize(&mut z);
        assert_eq!(z, vec![0.0, 0.0]);
        let mut n = vec![f32::NAN, 1.0];
        normalize(&mut n);
        assert_eq!(n, vec![0.0, 0.0]);
    }
}
