//! The benchmark's kNN similarity graph, read as published and held as
//! adjacency lists for the fitting passes.

use shelfmark_bench::SimilarityGraph;
use std::io::Read;
use std::path::Path;

/// Undirected weighted graph over items `0..n`, one entry per edge in
/// `edges` and both directions in the adjacency lists.
#[derive(Debug, Clone)]
pub struct Graph {
    pub n: usize,
    pub edges: Vec<(usize, usize, f64)>,
    pub total: f64,
    start: Vec<usize>,
    adj: Vec<(usize, f64)>,
}

impl Graph {
    pub fn new(n: usize, edges: Vec<(usize, usize, f64)>) -> Self {
        let mut degree = vec![0usize; n + 1];
        for &(a, b, _) in &edges {
            degree[a + 1] += 1;
            degree[b + 1] += 1;
        }
        for i in 0..n {
            degree[i + 1] += degree[i];
        }
        let start = degree.clone();
        let mut fill = degree;
        let mut adj = vec![(0, 0.0); 2 * edges.len()];
        for &(a, b, w) in &edges {
            adj[fill[a]] = (b, w);
            fill[a] += 1;
            adj[fill[b]] = (a, w);
            fill[b] += 1;
        }
        let total = edges.iter().map(|e| e.2).sum();
        Graph {
            n,
            edges,
            total,
            start,
            adj,
        }
    }

    pub fn from_similarity(g: &SimilarityGraph) -> Result<Self, String> {
        let mut edges = Vec::with_capacity(g.edges.len());
        for e in &g.edges {
            if e.left >= g.n || e.right >= g.n || e.left == e.right {
                return Err(format!("bad edge {}-{} for n = {}", e.left, e.right, g.n));
            }
            if e.weight > 0.0 {
                edges.push((e.left, e.right, e.weight));
            }
        }
        Ok(Graph::new(g.n, edges))
    }

    pub fn neighbors(&self, i: usize) -> &[(usize, f64)] {
        &self.adj[self.start[i]..self.start[i + 1]]
    }

    /// The graph induced on `keep` (old ids, in the order given); item
    /// `keep[k]` becomes item `k`.
    pub fn induced(&self, keep: &[usize]) -> Graph {
        let mut map = vec![usize::MAX; self.n];
        for (k, &i) in keep.iter().enumerate() {
            map[i] = k;
        }
        let edges = self
            .edges
            .iter()
            .filter(|e| map[e.0] != usize::MAX && map[e.1] != usize::MAX)
            .map(|&(a, b, w)| {
                let (x, y) = (map[a], map[b]);
                (x.min(y), x.max(y), w)
            })
            .collect();
        Graph::new(keep.len(), edges)
    }

    /// The same edges in the harness's serialized form (sorted by endpoints).
    pub fn to_similarity(&self, k: usize, tau: f64) -> SimilarityGraph {
        let mut edges: Vec<_> = self
            .edges
            .iter()
            .map(|&(a, b, w)| shelfmark_bench::Edge {
                left: a.min(b),
                right: a.max(b),
                weight: w,
            })
            .collect();
        edges.sort_by_key(|e| (e.left, e.right));
        SimilarityGraph {
            n: self.n,
            k,
            tau,
            edges,
        }
    }
}

/// Reads a file, gunzipping it when the name ends in `.gz`.
pub fn read_maybe_gz(path: &Path) -> Result<Vec<u8>, String> {
    let raw = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if path.extension().is_some_and(|x| x == "gz") {
        let mut out = Vec::new();
        flate2::read::GzDecoder::new(raw.as_slice())
            .read_to_end(&mut out)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(out)
    } else {
        Ok(raw)
    }
}

pub fn load_similarity(path: &Path) -> Result<SimilarityGraph, String> {
    serde_json::from_slice(&read_maybe_gz(path)?).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacency_and_induced_subgraph() {
        let g = Graph::new(4, vec![(0, 1, 1.0), (1, 2, 2.0), (2, 3, 3.0)]);
        assert_eq!(g.neighbors(1), &[(0, 1.0), (2, 2.0)]);
        assert_eq!(g.total, 6.0);
        let sub = g.induced(&[3, 2, 0]);
        assert_eq!(sub.n, 3);
        assert_eq!(sub.edges, vec![(0, 1, 3.0)]);
    }
}
