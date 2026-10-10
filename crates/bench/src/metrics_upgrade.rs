//! Query-weighted navigation and information-theoretic bounds.
//!
//! All logarithms in these metrics are base 2. Alphabetic binary search uses
//! the optimal alphabetic prefix code, computed with Knuth's O(n²) DP.
use crate::Tree;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeMetrics {
    pub query_workloads: std::collections::BTreeMap<String, NavigationPair>,
    pub popularity_proxy: String,
    pub information_bounds: InformationBounds,
    pub branch_constant_sweep: std::collections::BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct QueryCost {
    pub menu_accesses: f64,
    pub depth_traversed: f64,
    pub items_scanned: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InformationBounds {
    pub entropy_bits: f64,
    pub huffman_lower_bound_bits: f64,
    pub alphabetic_optimum_bits: f64,
    pub nav_over_entropy: f64,
    pub nav_over_alphabetic: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NavigationPair {
    pub placed_only: QueryCost,
    pub including_unplaced: QueryCost,
}

/// Complete deterministic query and bound metrics for a result record.
pub fn summarize(tree: &Tree) -> UpgradeMetrics {
    let n = tree.item_leaves.len();
    let uniform = uniform_queries(n);
    let zipf = zipf_queries_seeded(n, crate::bootstrap::DEFAULT_SEED);
    let mut workloads = std::collections::BTreeMap::new();
    workloads.insert("uniform".into(), navigation_pair(tree, &uniform));
    workloads.insert("zipf_s1_seed_20261010".into(), navigation_pair(tree, &zipf));
    let placed: Vec<f64> = tree
        .item_leaves
        .iter()
        .enumerate()
        .map(|(i, leaves)| {
            if leaves
                .iter()
                .any(|&(leaf, _)| tree.nodes[leaf].parent != Some(0))
            {
                uniform[i]
            } else {
                0.0
            }
        })
        .collect();
    workloads.insert("uniform_placed_only".into(), navigation_pair(tree, &placed));
    let nav = query_cost(tree, &uniform).menu_accesses;
    let mut sweep = std::collections::BTreeMap::new();
    for a in [0usize, 1, 2, 4] {
        let value = tree
            .item_leaves
            .iter()
            .map(|leaves| {
                leaves
                    .iter()
                    .map(|&(leaf, mass)| {
                        let mut subtotal = 0.0;
                        let mut node = leaf;
                        while let Some(parent) = tree.nodes[node].parent {
                            let fanout = tree.nodes[parent].children.len();
                            if fanout > 1 {
                                subtotal += a as f64 + (fanout as f64).log2();
                            }
                            node = parent;
                        }
                        subtotal * mass
                    })
                    .sum::<f64>()
            })
            .sum::<f64>()
            / n.max(1) as f64;
        sweep.insert(format!("a={a}"), value);
    }
    let bounds = information_bounds(&uniform, nav, 0);
    UpgradeMetrics {
        query_workloads: workloads,
        popularity_proxy:
            "none available: corpus has one sampled record per work and no edition-count field"
                .into(),
        information_bounds: bounds,
        branch_constant_sweep: sweep,
    }
}

pub fn navigation_pair(tree: &Tree, popularity: &[f64]) -> NavigationPair {
    let all = query_cost(tree, popularity);
    let placed: Vec<f64> = tree
        .item_leaves
        .iter()
        .enumerate()
        .map(|(i, leaves)| {
            if leaves
                .iter()
                .any(|&(leaf, _)| tree.nodes[leaf].parent != Some(0))
            {
                popularity[i]
            } else {
                0.0
            }
        })
        .collect();
    NavigationPair {
        placed_only: query_cost(tree, &placed),
        including_unplaced: all,
    }
}

/// Per-item path costs; direct root children are included so unplaced items
/// pay for the root menu. A placed-only aggregate filters them out.
pub fn query_cost(tree: &Tree, probabilities: &[f64]) -> QueryCost {
    assert_eq!(tree.item_leaves.len(), probabilities.len());
    let total: f64 = probabilities.iter().sum();
    if total <= 0.0 {
        return QueryCost {
            menu_accesses: 0.0,
            depth_traversed: 0.0,
            items_scanned: 0.0,
        };
    }
    let mut result = QueryCost {
        menu_accesses: 0.0,
        depth_traversed: 0.0,
        items_scanned: 0.0,
    };
    for (item, leaves) in tree.item_leaves.iter().enumerate() {
        let p = probabilities[item] / total;
        for &(leaf, mass) in leaves {
            let q = p * mass;
            let mut node = leaf;
            while let Some(parent) = tree.nodes[node].parent {
                let fanout = tree.nodes[parent].children.len();
                if fanout > 1 {
                    result.menu_accesses += q;
                    result.items_scanned += q * (fanout as f64 + 1.0) / 2.0;
                }
                result.depth_traversed += q;
                node = parent;
            }
        }
    }
    result
}

/// Uniform popularity over corpus rows.
pub fn uniform_queries(n: usize) -> Vec<f64> {
    vec![1.0; n]
}

/// Zipf popularity in item order (rank 1 is most popular), exponent s = 1.
pub fn zipf_queries(n: usize) -> Vec<f64> {
    (1..=n).map(|rank| 1.0 / rank as f64).collect()
}

/// Zipf(s=1) probabilities assigned to a seeded random ordering of items.
pub fn zipf_queries_seeded(n: usize, seed: u64) -> Vec<f64> {
    let mut order: Vec<usize> = (0..n).collect();
    let mut rng = crate::bootstrap::SplitMix64::new(seed);
    for i in (1..n).rev() {
        order.swap(i, rng.below(i + 1));
    }
    let mut weights = vec![0.0; n];
    for (rank, item) in order.into_iter().enumerate() {
        weights[item] = 1.0 / (rank + 1) as f64;
    }
    weights
}

/// Expected binary decision depth for ordered leaf probabilities using an
/// optimal alphabetic tree. Knuth's monotone-root bounds give O(n²) time.
pub fn optimal_alphabetic_cost(weights: &[f64]) -> f64 {
    let n = weights.len();
    if n <= 1 {
        return 0.0;
    }
    let mut prefix = vec![0.0; n + 1];
    for i in 0..n {
        prefix[i + 1] = prefix[i] + weights[i].max(0.0);
    }
    let mut cost = vec![vec![0.0; n]; n];
    let mut root = vec![vec![0usize; n]; n];
    for (i, row) in root.iter_mut().enumerate() {
        row[i] = i;
    }
    for len in 2..=n {
        for i in 0..=n - len {
            let j = i + len - 1;
            let lo = root[i][j - 1].max(i);
            let hi = root[i + 1][j].min(j - 1);
            let sum = prefix[j + 1] - prefix[i];
            let mut best = f64::INFINITY;
            let mut best_k = lo;
            for k in lo..=hi {
                let value = cost[i][k] + cost[k + 1][j] + sum;
                if value < best {
                    best = value;
                    best_k = k;
                }
            }
            cost[i][j] = best;
            root[i][j] = best_k;
        }
    }
    cost[0][n - 1] / prefix[n].max(f64::MIN_POSITIVE)
}

/// Shannon entropy and binary lower bounds for an ordered set of leaves.
pub fn information_bounds(
    weights: &[f64],
    nav_accesses: f64,
    branch_constant: usize,
) -> InformationBounds {
    let sum: f64 = weights.iter().map(|x| x.max(0.0)).sum();
    let probs: Vec<f64> = weights
        .iter()
        .map(|x| x.max(0.0) / sum.max(f64::MIN_POSITIVE))
        .collect();
    let entropy_bits = probs
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.log2())
        .sum();
    // Binary search's per-level fixed branch cost is charged to the observed
    // path, and the same constant is reflected in both bounds.
    let alphabetic = optimal_alphabetic_cost(&probs) + branch_constant as f64;
    InformationBounds {
        entropy_bits,
        huffman_lower_bound_bits: entropy_bits,
        alphabetic_optimum_bits: alphabetic,
        nav_over_entropy: if entropy_bits > 0.0 {
            nav_accesses / entropy_bits
        } else {
            0.0
        },
        nav_over_alphabetic: if alphabetic > 0.0 {
            nav_accesses / alphabetic
        } else {
            0.0
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{induce_tree, ItemPlacement};
    use shelfmark_core::scheme::{ClassDef, SchemeDoc};

    fn hand_tree() -> Tree {
        let scheme = SchemeDoc {
            schema: None,
            name: "toy".into(),
            version: "1".into(),
            title: "toy".into(),
            source: None,
            code_grammar: ".*".into(),
            classes: vec![
                ClassDef {
                    code: "A".into(),
                    label: "A".into(),
                    parent: None,
                    range: None,
                    vocab: vec![],
                    note: None,
                },
                ClassDef {
                    code: "B".into(),
                    label: "B".into(),
                    parent: None,
                    range: None,
                    vocab: vec![],
                    note: None,
                },
            ],
        };
        induce_tree(
            &scheme,
            &["A", "B", "", "A"]
                .iter()
                .enumerate()
                .map(|(i, c)| ItemPlacement {
                    id: i.to_string(),
                    classes: if c.is_empty() {
                        vec![]
                    } else {
                        vec![(c.to_string(), 1.0)]
                    },
                })
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    #[test]
    fn navigation_counts_root_residual_and_placed_only_queries() {
        let t = hand_tree();
        let all = query_cost(&t, &uniform_queries(4));
        let placed = query_cost(&t, &[1.0, 1.0, 0.0, 1.0]);
        assert!((all.menu_accesses - 1.5).abs() < 1e-12);
        assert!((all.items_scanned - 2.75).abs() < 1e-12);
        assert!((placed.menu_accesses - 5.0 / 3.0).abs() < 1e-12);
        assert_eq!(zipf_queries(3), vec![1.0, 0.5, 1.0 / 3.0]);
    }

    #[test]
    fn entropy_and_alphabetic_bound_match_two_equal_leaves() {
        assert!((optimal_alphabetic_cost(&[0.5, 0.5]) - 1.0).abs() < 1e-12);
        let b = information_bounds(&[1.0, 1.0], 1.0, 0);
        assert!((b.entropy_bits - 1.0).abs() < 1e-12);
        assert!((b.alphabetic_optimum_bits - 1.0).abs() < 1e-12);
        assert!((b.nav_over_alphabetic - 1.0).abs() < 1e-12);
    }
}
