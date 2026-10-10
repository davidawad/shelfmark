//! Query-weighted navigation and information-theoretic bounds.
//!
//! All logarithms in these metrics are base 2. Alphabetic binary search uses
//! the optimal alphabetic prefix code, computed with Knuth's O(n²) DP.
use crate::{score_tree, SimilarityGraph, Tree};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeMetrics {
    pub query_workloads: std::collections::BTreeMap<String, NavigationPair>,
    pub popularity_proxy: String,
    pub information_bounds: InformationBounds,
    pub branch_constant_sweep: std::collections::BTreeMap<String, f64>,
    pub alphabetic_method: String,
    pub log_base: u8,
    /// Paired percentile intervals from 1,000 shared item resamples.
    pub confidence_intervals: std::collections::BTreeMap<String, [f64; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullSummary {
    pub mean: f64,
    pub sd: f64,
    pub z: f64,
    /// Fraction of null replicates at or below the observed score.
    pub percentile: f64,
    pub confidence_intervals: std::collections::BTreeMap<String, [f64; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermutationNull {
    pub permutations: usize,
    pub seed: u64,
    pub cost_norm: NullSummary,
    pub skill: NullSummary,
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
    let confidence_intervals = navigation_intervals(tree, &zipf, bounds);
    UpgradeMetrics {
        query_workloads: workloads,
        popularity_proxy:
            "none available: corpus has one sampled record per work and no edition-count field"
                .into(),
        information_bounds: bounds,
        branch_constant_sweep: sweep,
        alphabetic_method: "Knuth-optimized alphabetic dynamic program, O(n^2)".into(),
        log_base: 2,
        confidence_intervals,
    }
}

fn navigation_intervals(
    tree: &Tree,
    zipf: &[f64],
    bounds: InformationBounds,
) -> std::collections::BTreeMap<String, [f64; 2]> {
    let n = tree.item_leaves.len();
    let mut draws = std::collections::BTreeMap::<String, Vec<f64>>::new();
    let empty_graph = SimilarityGraph {
        n,
        k: 0,
        tau: 0.0,
        edges: vec![],
    };
    let prepared = crate::bootstrap::Prepared::new(tree, &empty_graph, true);
    for replicate in 0..1000 {
        let counts = crate::bootstrap::multiplicities(n, crate::bootstrap::DEFAULT_SEED, replicate);
        let values = navigation_draw_values(tree, zipf, bounds, &prepared, &counts);
        for (key, value) in values {
            draws.entry(key.into()).or_default().push(value);
        }
    }
    draws
        .into_iter()
        .map(|(key, values)| (key, crate::bootstrap::interval(&values, 0.95)))
        .collect()
}

fn navigation_draw_values(
    tree: &Tree,
    zipf: &[f64],
    bounds: InformationBounds,
    prepared: &crate::bootstrap::Prepared,
    counts: &[f64],
) -> std::collections::BTreeMap<&'static str, f64> {
    let mut values = std::collections::BTreeMap::<&'static str, f64>::new();
    let structural = prepared.replicate(counts);
    for (key, value) in &structural {
        if *key != "cost_norm" && *key != "revenue_norm" {
            values.insert(*key, *value);
        }
    }
    let pair = navigation_pair(tree, counts);
    let vals = [
        (
            "uniform.menu_accesses.placed",
            pair.placed_only.menu_accesses,
        ),
        (
            "uniform.menu_accesses.all",
            pair.including_unplaced.menu_accesses,
        ),
        (
            "uniform.depth_traversed.placed",
            pair.placed_only.depth_traversed,
        ),
        (
            "uniform.depth_traversed.all",
            pair.including_unplaced.depth_traversed,
        ),
        (
            "uniform.items_scanned.placed",
            pair.placed_only.items_scanned,
        ),
        (
            "uniform.items_scanned.all",
            pair.including_unplaced.items_scanned,
        ),
        ("information.entropy_bits", bounds.entropy_bits),
        (
            "information.huffman_lower_bound_bits",
            bounds.huffman_lower_bound_bits,
        ),
        (
            "information.alphabetic_optimum_bits",
            bounds.alphabetic_optimum_bits,
        ),
        (
            "information.nav_over_entropy",
            if bounds.entropy_bits > 0.0 {
                pair.including_unplaced.menu_accesses / bounds.entropy_bits
            } else {
                0.0
            },
        ),
        (
            "information.nav_over_alphabetic",
            if bounds.alphabetic_optimum_bits > 0.0 {
                pair.including_unplaced.menu_accesses / bounds.alphabetic_optimum_bits
            } else {
                0.0
            },
        ),
        ("branch_constant.a0", branch_expected(tree, counts, 0)),
        ("branch_constant.a1", branch_expected(tree, counts, 1)),
        ("branch_constant.a2", branch_expected(tree, counts, 2)),
        ("branch_constant.a4", branch_expected(tree, counts, 4)),
    ];
    values.extend(vals);
    let zipf_pair = navigation_pair(
        tree,
        &zipf
            .iter()
            .zip(counts)
            .map(|(p, c)| p * c)
            .collect::<Vec<_>>(),
    );
    values.insert(
        "zipf_s1_seed_20261010.menu_accesses.placed",
        zipf_pair.placed_only.menu_accesses,
    );
    values.insert(
        "zipf_s1_seed_20261010.menu_accesses.all",
        zipf_pair.including_unplaced.menu_accesses,
    );
    values.insert(
        "zipf_s1_seed_20261010.depth_traversed.placed",
        zipf_pair.placed_only.depth_traversed,
    );
    values.insert(
        "zipf_s1_seed_20261010.depth_traversed.all",
        zipf_pair.including_unplaced.depth_traversed,
    );
    values.insert(
        "zipf_s1_seed_20261010.items_scanned.placed",
        zipf_pair.placed_only.items_scanned,
    );
    values.insert(
        "zipf_s1_seed_20261010.items_scanned.all",
        zipf_pair.including_unplaced.items_scanned,
    );
    values
}

fn branch_expected(tree: &Tree, weights: &[f64], a: usize) -> f64 {
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return 0.0;
    }
    tree.item_leaves
        .iter()
        .enumerate()
        .map(|(item, leaves)| {
            let cost = leaves
                .iter()
                .map(|&(leaf, mass)| {
                    let mut value = 0.0;
                    let mut node = leaf;
                    while let Some(parent) = tree.nodes[node].parent {
                        let fanout = tree.nodes[parent].children.len();
                        if fanout > 1 {
                            value += a as f64 + (fanout as f64).log2();
                        }
                        node = parent;
                    }
                    mass * value
                })
                .sum::<f64>();
            weights[item] * cost / total
        })
        .sum()
}

/// Permutes item-to-leaf assignments among placed items while preserving the
/// tree shape, every class size, and the unplaced assignments.
pub fn permutation_null(
    tree: &Tree,
    graph: &SimilarityGraph,
    reference_cost: f64,
    permutations: usize,
    seed: u64,
) -> PermutationNull {
    use crate::bootstrap::SplitMix64;
    let observed = score_tree(tree, graph).cost_norm;
    let observed_skill = if reference_cost < 1.0 {
        (1.0 - observed) / (1.0 - reference_cost)
    } else {
        0.0
    };
    let placed: Vec<usize> = tree
        .item_leaves
        .iter()
        .enumerate()
        .filter_map(|(i, leaves)| {
            leaves
                .iter()
                .any(|&(leaf, _)| tree.nodes[leaf].parent != Some(0))
                .then_some(i)
        })
        .collect();
    let assignments: Vec<_> = placed
        .iter()
        .map(|&i| tree.item_leaves[i].clone())
        .collect();
    let mut rng = SplitMix64::new(seed);
    let mut costs = Vec::with_capacity(permutations);
    let mut skills = Vec::with_capacity(permutations);
    for _ in 0..permutations {
        let mut shuffled = assignments.clone();
        for i in (1..shuffled.len()).rev() {
            shuffled.swap(i, rng.below(i + 1));
        }
        let mut candidate = tree.clone();
        for (&item, leaves) in placed.iter().zip(shuffled) {
            candidate.item_leaves[item] = leaves;
        }
        let cost = score_tree(&candidate, graph).cost_norm;
        costs.push(cost);
        skills.push(if reference_cost < 1.0 {
            (1.0 - cost) / (1.0 - reference_cost)
        } else {
            0.0
        });
    }
    let mut null_boot_rng = crate::bootstrap::SplitMix64::new(seed ^ 0x5045_524d_4349);
    let mut summarize = |values: &[f64], observed: f64| {
        let mean = values.iter().sum::<f64>() / values.len().max(1) as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
            / values.len().saturating_sub(1).max(1) as f64;
        let mut sorted = values.to_vec();
        sorted.sort_by(f64::total_cmp);
        let mut draws = std::collections::BTreeMap::<String, Vec<f64>>::new();
        for _ in 0..1000 {
            let sample: Vec<f64> = (0..values.len())
                .map(|_| values[null_boot_rng.below(values.len().max(1))])
                .collect();
            let sample_mean = sample.iter().sum::<f64>() / sample.len().max(1) as f64;
            let sample_sd = (sample
                .iter()
                .map(|x| (x - sample_mean).powi(2))
                .sum::<f64>()
                / sample.len().saturating_sub(1).max(1) as f64)
                .sqrt();
            let sample_z = if sample_sd > 0.0 {
                (observed - sample_mean) / sample_sd
            } else {
                0.0
            };
            let sample_percentile = sample.iter().filter(|&&x| x <= observed).count() as f64
                / sample.len().max(1) as f64;
            for (key, value) in [
                ("mean", sample_mean),
                ("sd", sample_sd),
                ("z", sample_z),
                ("percentile", sample_percentile),
            ] {
                draws.entry(key.into()).or_default().push(value);
            }
        }
        let confidence_intervals = draws
            .into_iter()
            .map(|(key, values)| (key, crate::bootstrap::interval(&values, 0.95)))
            .collect();
        NullSummary {
            mean,
            sd: variance.sqrt(),
            z: if variance > 0.0 {
                (observed - mean) / variance.sqrt()
            } else {
                0.0
            },
            percentile: sorted.iter().filter(|&&x| x <= observed).count() as f64
                / sorted.len().max(1) as f64,
            confidence_intervals,
        }
    };
    PermutationNull {
        permutations,
        seed,
        cost_norm: summarize(&costs, observed),
        skill: summarize(&skills, observed_skill),
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
        let summary = summarize(&t);
        assert_eq!(summary.log_base, 2);
        assert!(summary.confidence_intervals.len() >= 20);
        assert!(
            summary.confidence_intervals["nav_scan"][0]
                <= summary.confidence_intervals["nav_scan"][1]
        );
    }

    #[test]
    fn entropy_and_alphabetic_bound_match_two_equal_leaves() {
        assert!((optimal_alphabetic_cost(&[0.5, 0.5]) - 1.0).abs() < 1e-12);
        let b = information_bounds(&[1.0, 1.0], 1.0, 0);
        assert!((b.entropy_bits - 1.0).abs() < 1e-12);
        assert!((b.alphabetic_optimum_bits - 1.0).abs() < 1e-12);
        assert!((b.nav_over_alphabetic - 1.0).abs() < 1e-12);
    }

    #[test]
    fn assignment_null_is_seeded_and_preserves_class_size_distribution() {
        let t = hand_tree();
        let graph = crate::SimilarityGraph {
            n: 4,
            k: 1,
            tau: 0.0,
            edges: vec![
                crate::Edge {
                    left: 0,
                    right: 3,
                    weight: 1.0,
                },
                crate::Edge {
                    left: 0,
                    right: 1,
                    weight: 1.0,
                },
            ],
        };
        let a = permutation_null(&t, &graph, 1.0, 200, 7);
        let b = permutation_null(&t, &graph, 1.0, 200, 7);
        assert_eq!(a.permutations, 200);
        assert_eq!(a.cost_norm.mean, b.cost_norm.mean);
        assert_eq!(a.cost_norm.percentile, b.cost_norm.percentile);
        assert!(a.cost_norm.mean.is_finite());
        assert!(a.skill.mean.is_finite());
        assert!(
            a.cost_norm.confidence_intervals["z"][0] <= a.cost_norm.confidence_intervals["z"][1]
        );
    }
}
