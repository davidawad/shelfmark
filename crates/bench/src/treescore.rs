//! Similarity-graph scores of a single-placement hierarchy, computed per
//! edge so that bootstrap resampling only re-sums the per-edge values.
//!
//! * Dasgupta (2016) cost: an edge pays `w · |items under LCA|`.
//! * Binarized cost: every multiway node is replaced by a weight-balanced
//!   binary tree over its slots (own items first, then subclasses) in shelf
//!   order, and the edge pays the items under its LCA in that tree.
//! * Per-level cost: the tree is cut below depth `L` and items below are
//!   collapsed into their depth-`L` class.
//! * Tree-sampling divergence (Charpentier & Bonald 2019): KL between the
//!   LCA distribution of a sampled edge and of two independently sampled
//!   degree-weighted items.
//! * Triplet accuracy: for an edge (i, j) and a sampled non-neighbour k of i,
//!   the triplet is correct when LCA(i, j) is deeper than LCA(i, k); ties
//!   count one half.
use crate::{Hier, SimilarityGraph};
use std::collections::BTreeSet;

/// Per-edge values, in graph edge order (edges touching an item outside the
/// hierarchy are dropped).
#[derive(Debug, Clone, Default)]
pub struct EdgeTable {
    pub left: Vec<usize>,
    pub right: Vec<usize>,
    pub weight: Vec<f64>,
    pub lca: Vec<usize>,
    /// Items under the LCA (Dasgupta pair cost).
    pub size: Vec<f64>,
    pub binarized: Vec<f64>,
    /// Pair cost with the tree cut at depth 1, 2, 3.
    pub level: [Vec<f64>; 3],
    /// Triplet outcome in {0, 0.5, 1} against a sampled non-neighbour.
    pub triplet: Vec<f64>,
}

/// Deterministic SplitMix64 generator used by every sampled score.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform integer in `0..n` (n > 0).
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Slot of `item` under `node`: own items first (by position), then
/// subclasses in shelf order.
fn slot(h: &Hier, node: usize, item: usize) -> usize {
    let c = h.item_class[item];
    if c == node {
        h.direct[node].binary_search(&item).unwrap_or(0)
    } else {
        let child = h.child_toward(node, c);
        h.direct[node].len()
            + h.children[node]
                .iter()
                .position(|&x| x == child)
                .unwrap_or(0)
    }
}

fn slot_weight(h: &Hier, node: usize, s: usize) -> f64 {
    let own = h.direct[node].len();
    if s < own {
        1.0
    } else {
        h.subtree[h.children[node][s - own]] as f64
    }
}

/// Items under the LCA of slots `a < b` in the weight-balanced binary tree
/// over a node's slots (`prefix` holds the slot weight prefix sums). Each
/// range is cut where the left weight is closest to half; ties cut left.
fn binary_lca_size(prefix: &[f64], a: usize, b: usize) -> f64 {
    let (mut lo, mut hi) = (0, prefix.len() - 1);
    while hi - lo > 1 {
        let half = (prefix[lo] + prefix[hi]) / 2.0;
        let first = lo + 1 + prefix[lo + 1..hi].partition_point(|&x| x < half);
        let left = first.saturating_sub(1).max(lo + 1);
        let right = first.min(hi - 1);
        let cut = if (prefix[left] - half).abs() <= (prefix[right] - half).abs() {
            left
        } else {
            right
        };
        if b < cut {
            hi = cut;
        } else if a >= cut {
            lo = cut;
        } else {
            break;
        }
    }
    prefix[hi] - prefix[lo]
}

fn prefix_sums(h: &Hier, node: usize) -> Vec<f64> {
    let slots = h.direct[node].len() + h.children[node].len();
    let mut prefix = vec![0.0; slots + 1];
    for s in 0..slots {
        prefix[s + 1] = prefix[s] + slot_weight(h, node, s);
    }
    prefix
}

pub fn edge_table(h: &Hier, graph: &SimilarityGraph, seed: u64) -> EdgeTable {
    let n = h.items();
    let mut neighbours = vec![BTreeSet::new(); n];
    for e in &graph.edges {
        if e.left < n && e.right < n {
            neighbours[e.left].insert(e.right);
            neighbours[e.right].insert(e.left);
        }
    }
    let mut prefixes = vec![None; h.len()];
    let mut rng = Rng::new(seed);
    let mut t = EdgeTable::default();
    for e in &graph.edges {
        let (i, j) = (e.left, e.right);
        if i >= n || j >= n {
            continue;
        }
        let (ci, cj) = (h.item_class[i], h.item_class[j]);
        let lca = h.lca(ci, cj);
        let size = h.subtree[lca] as f64;
        let prefix = prefixes[lca].get_or_insert_with(|| prefix_sums(h, lca));
        let (si, sj) = (slot(h, lca, i), slot(h, lca, j));
        let binarized = binary_lca_size(prefix, si.min(sj), si.max(sj));
        let level = [1, 2, 3].map(|d| h.subtree[h.ancestor_at(lca, d)] as f64);
        // Non-neighbour k of i, other than i and j; a few draws, then give up.
        let mut triplet = 0.5;
        for _ in 0..32 {
            let k = rng.below(n);
            if k != i && k != j && !neighbours[i].contains(&k) {
                let near = h.depth[lca];
                let far = h.depth[h.lca(ci, h.item_class[k])];
                triplet = match near.cmp(&far) {
                    std::cmp::Ordering::Greater => 1.0,
                    std::cmp::Ordering::Equal => 0.5,
                    std::cmp::Ordering::Less => 0.0,
                };
                break;
            }
        }
        t.left.push(i);
        t.right.push(j);
        t.weight.push(e.weight);
        t.lca.push(lca);
        t.size.push(size);
        t.binarized.push(binarized);
        for (slot, v) in t.level.iter_mut().zip(level) {
            slot.push(v);
        }
        t.triplet.push(triplet);
    }
    t
}

/// `Σ w·x / (n · Σ w)` over the selected edges (Dasgupta-normalised cost).
pub fn norm_cost(t: &EdgeTable, values: &[f64], n: usize, pick: &[usize]) -> f64 {
    let (mut num, mut den) = (0.0, 0.0);
    for &e in pick {
        num += t.weight[e] * values[e];
        den += t.weight[e];
    }
    if den > 0.0 {
        num / (n as f64 * den)
    } else {
        0.0
    }
}

/// Unweighted mean over the selected edges.
pub fn mean(values: &[f64], pick: &[usize]) -> f64 {
    if pick.is_empty() {
        return 0.0;
    }
    pick.iter().map(|&e| values[e]).sum::<f64>() / pick.len() as f64
}

/// Tree-sampling divergence in nats over the selected edges.
pub fn tree_sampling_divergence(h: &Hier, t: &EdgeTable, pick: &[usize]) -> f64 {
    let n = h.items();
    let mut degree = vec![0.0; n];
    let mut p = vec![0.0; h.len()];
    let mut total = 0.0;
    for &e in pick {
        let w = t.weight[e];
        degree[t.left[e]] += w;
        degree[t.right[e]] += w;
        p[t.lca[e]] += w;
        total += w;
    }
    if total <= 0.0 {
        return 0.0;
    }
    // pi_i = degree / 2W; Pi(x) = mass under x; q(x) = Pi(x)^2 - sum_children Pi(c)^2 - sum_direct pi_i^2.
    let mut under = vec![0.0; h.len()];
    let mut direct_sq = vec![0.0; h.len()];
    for (item, &c) in h.item_class.iter().enumerate() {
        let pi = degree[item] / (2.0 * total);
        under[c] += pi;
        direct_sq[c] += pi * pi;
    }
    for v in (1..h.len()).rev() {
        let p = h.parent[v];
        under[p] += under[v];
    }
    let mut tsd = 0.0;
    for x in 0..h.len() {
        if p[x] <= 0.0 {
            continue;
        }
        let q = under[x] * under[x]
            - h.children[x]
                .iter()
                .map(|&c| under[c] * under[c])
                .sum::<f64>()
            - direct_sq[x];
        let px = p[x] / total;
        if q > 0.0 {
            tsd += px * (px / q).ln();
        }
    }
    tsd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hier::tests::toy;
    use crate::Edge;

    fn graph(edges: &[(usize, usize, f64)]) -> SimilarityGraph {
        SimilarityGraph {
            n: 5,
            k: 1,
            tau: 0.0,
            edges: edges
                .iter()
                .map(|&(left, right, weight)| Edge {
                    left,
                    right,
                    weight,
                })
                .collect(),
        }
    }

    #[test]
    fn dasgupta_sizes_levels_and_binarized() {
        // toy: A{A1{0,1}, A2{2}}, B{3}, root{4}.
        let h = toy();
        let t = edge_table(
            &h,
            &graph(&[(0, 1, 1.0), (0, 2, 1.0), (2, 3, 1.0), (3, 4, 1.0)]),
            7,
        );
        assert_eq!(t.size, [2.0, 3.0, 5.0, 5.0]);
        // Cut at depth 1: (0,1) collapses into A (3 items).
        assert_eq!(t.level[0], [3.0, 3.0, 5.0, 5.0]);
        assert_eq!(t.level[1], [2.0, 3.0, 5.0, 5.0]);
        // Root slots: item 4 (w1), A (w3), B (w1); the tied cut goes left,
        // {4}|{A,B}. (2,3): A vs B -> {A,B}, 4; (3,4): 4 vs B -> root, 5.
        // A's slots A1 (2), A2 (1): (0,2) -> 3. A1's slots are items 0,1 -> 2.
        assert_eq!(t.binarized, [2.0, 3.0, 4.0, 5.0]);
        assert!((norm_cost(&t, &t.size, 5, &[0, 1, 2, 3]) - 15.0 / 20.0).abs() < 1e-12);
    }

    #[test]
    fn binarized_splits_a_flat_node() {
        // Four items directly at one class: balanced tree ((0,1),(2,3)).
        let h = Hier::from_parents(&[0, 0], ["", "A"].map(String::from).to_vec(), vec![1; 4]);
        let t = edge_table(&h, &graph(&[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0)]), 1);
        assert_eq!(t.size, [4.0, 4.0, 4.0]);
        assert_eq!(t.binarized, [2.0, 4.0, 2.0]);
    }

    #[test]
    fn tsd_hand_checked() {
        // Two classes, two items each, one edge inside each class.
        let h = Hier::from_parents(
            &[0, 0, 0],
            ["", "A", "B"].map(String::from).to_vec(),
            vec![1, 1, 2, 2],
        );
        let g = SimilarityGraph {
            n: 4,
            k: 1,
            tau: 0.0,
            edges: vec![
                Edge {
                    left: 0,
                    right: 1,
                    weight: 1.0,
                },
                Edge {
                    left: 2,
                    right: 3,
                    weight: 1.0,
                },
            ],
        };
        let t = edge_table(&h, &g, 3);
        // pi = 1/4 each. q(A) = (1/2)^2 - 2/16 = 1/8; p(A) = 1/2.
        // TSD = 2 * 1/2 * ln(4) = ln 4.
        let tsd = tree_sampling_divergence(&h, &t, &[0, 1]);
        assert!((tsd - 4f64.ln()).abs() < 1e-12);
        // Each edge's sampled non-neighbour is in the other class: correct triplets.
        assert_eq!(t.triplet, [1.0, 1.0]);
    }

    #[test]
    fn rng_is_deterministic() {
        let a: Vec<_> = (0..4)
            .scan(Rng::new(9), |r, _| Some(r.below(100)))
            .collect();
        let b: Vec<_> = (0..4)
            .scan(Rng::new(9), |r, _| Some(r.below(100)))
            .collect();
        assert_eq!(a, b);
    }
}
