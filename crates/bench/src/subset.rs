//! Restriction of a run to a subset of its items (the covered-subset mode).
//!
//! Items keep their corpus order. Graph edges are kept only when both
//! endpoints are in the subset and are renumbered to subset positions, in
//! the original edge order; vectors keep the subset rows. Every score then
//! sees only the subset, so runs restricted to the same items share the
//! same graph and are paired.
use crate::embedscore::Vectors;
use crate::{Edge, SimilarityGraph};

/// Subset position of every original index (`None` when left out).
fn positions(n: usize, keep: &[usize]) -> Vec<Option<usize>> {
    let mut pos = vec![None; n];
    for (p, &i) in keep.iter().enumerate() {
        if i < n {
            pos[i] = Some(p);
        }
    }
    pos
}

/// The graph induced on `keep` (sorted, distinct original indices).
pub fn restrict_graph(g: &SimilarityGraph, keep: &[usize]) -> SimilarityGraph {
    let pos = positions(g.n, keep);
    let edges = g
        .edges
        .iter()
        .filter_map(|e| {
            Some(Edge {
                left: pos.get(e.left).copied().flatten()?,
                right: pos.get(e.right).copied().flatten()?,
                weight: e.weight,
            })
        })
        .collect();
    SimilarityGraph {
        n: keep.len(),
        k: g.k,
        tau: g.tau,
        edges,
    }
}

/// The rows of `keep`, in order.
pub fn restrict_vectors(v: &Vectors, keep: &[usize]) -> Vectors {
    Vectors {
        dim: v.dim,
        data: keep
            .iter()
            .flat_map(|&i| v.row(i).iter().copied())
            .collect(),
    }
}

/// `k` distinct indices of `0..n` drawn without replacement (partial
/// Fisher–Yates with the given generator), sorted; all of `0..n` when
/// `k >= n`.
pub fn sample_without_replacement(
    n: usize,
    k: usize,
    rng: &mut crate::treescore::Rng,
) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..n).collect();
    let k = k.min(n);
    for i in 0..k {
        let j = i + rng.below(n - i);
        idx.swap(i, j);
    }
    idx.truncate(k);
    idx.sort_unstable();
    idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treescore::Rng;

    #[test]
    fn restrict_keeps_inner_edges_renumbered() {
        let edge = |left, right, weight| Edge {
            left,
            right,
            weight,
        };
        let g = SimilarityGraph {
            n: 5,
            k: 2,
            tau: 0.1,
            edges: vec![
                edge(0, 1, 0.5),
                edge(1, 3, 0.4),
                edge(2, 4, 0.3),
                edge(3, 4, 0.2),
            ],
        };
        let r = restrict_graph(&g, &[1, 3, 4]);
        assert_eq!(r.n, 3);
        assert_eq!(r.tau, 0.1);
        assert_eq!(r.edges, vec![edge(0, 1, 0.4), edge(1, 2, 0.2)]);
        let v = Vectors {
            dim: 2,
            data: (0..10).map(|x| x as f32).collect(),
        };
        assert_eq!(restrict_vectors(&v, &[1, 4]).data, [2.0, 3.0, 8.0, 9.0]);
    }

    #[test]
    fn sampling_is_deterministic_distinct_and_capped() {
        let a = sample_without_replacement(100, 10, &mut Rng::new(5));
        let b = sample_without_replacement(100, 10, &mut Rng::new(5));
        assert_eq!(a, b);
        assert_eq!(a.len(), 10);
        assert!(a.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(
            sample_without_replacement(4, 9, &mut Rng::new(1)),
            [0, 1, 2, 3]
        );
    }
}
