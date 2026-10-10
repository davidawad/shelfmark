use serde::{Deserialize, Serialize};

/// Plug-in point for a pinned run-time embedding model.
pub trait EmbeddingModel {
    fn embed(&self, text: &str) -> Vec<f32>;
}

/// Stable feature-hash embedder for tests and reproducible smoke runs.
#[derive(Debug, Clone)]
pub struct DeterministicEmbedder {
    pub dimensions: usize,
}

impl Default for DeterministicEmbedder {
    fn default() -> Self {
        Self { dimensions: 256 }
    }
}

impl EmbeddingModel for DeterministicEmbedder {
    fn embed(&self, text: &str) -> Vec<f32> {
        let mut vector = vec![0.0; self.dimensions.max(1)];
        for token in text.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
            if token.is_empty() {
                continue;
            }
            let hash = token.bytes().fold(2166136261u32, |h, b| {
                (h ^ u32::from(b)).wrapping_mul(16777619)
            });
            let slot = hash as usize % vector.len();
            vector[slot] += if hash & 0x8000_0000 == 0 { 1.0 } else { -1.0 };
        }
        normalize(&mut vector);
        vector
    }
}

fn normalize(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        v.iter_mut().for_each(|x| *x /= norm);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub left: usize,
    pub right: usize,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimilarityGraph {
    pub n: usize,
    pub k: usize,
    pub tau: f64,
    pub edges: Vec<Edge>,
}

/// Exact cosine kNN with deterministic mean-pair offset and max symmetrization.
pub fn build_knn_graph(vectors: &[Vec<f32>], k: usize) -> SimilarityGraph {
    let n = vectors.len();
    let mut cosines = Vec::with_capacity(n.saturating_mul(n.saturating_sub(1)) / 2);
    for i in 0..n {
        for j in i + 1..n {
            cosines.push((i, j, cosine(&vectors[i], &vectors[j])));
        }
    }
    let tau = if cosines.is_empty() {
        0.0
    } else {
        cosines.iter().map(|x| x.2).sum::<f64>() / cosines.len() as f64
    };
    let mut best = vec![Vec::<(usize, f64)>::new(); n];
    for &(i, j, sim) in &cosines {
        best[i].push((j, sim));
        best[j].push((i, sim));
    }
    let mut weights = std::collections::BTreeMap::<(usize, usize), f64>::new();
    for (i, list) in best.iter_mut().enumerate() {
        list.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        for &(j, sim) in list.iter().take(k) {
            let key = (i.min(j), i.max(j));
            let w = (sim - tau).max(0.0);
            weights
                .entry(key)
                .and_modify(|v| *v = v.max(w))
                .or_insert(w);
        }
    }
    SimilarityGraph {
        n,
        k,
        tau,
        edges: weights
            .into_iter()
            .filter(|(_, w)| *w > 0.0)
            .map(|((left, right), weight)| Edge {
                left,
                right,
                weight,
            })
            .collect(),
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot = a
        .iter()
        .zip(b)
        .map(|(x, y)| f64::from(*x) * f64::from(*y))
        .sum::<f64>();
    let na = a.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
    let nb = b.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
    if na * nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_embedder_and_graph() {
        let model = DeterministicEmbedder::default();
        let texts = ["alpha beta", "alpha gamma", "delta epsilon"];
        let a: Vec<_> = texts.iter().map(|t| model.embed(t)).collect();
        let b: Vec<_> = texts.iter().map(|t| model.embed(t)).collect();
        assert_eq!(a, b);
        assert_eq!(build_knn_graph(&a, 1), build_knn_graph(&b, 1));
    }
}
