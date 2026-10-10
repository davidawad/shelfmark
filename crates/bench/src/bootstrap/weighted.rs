//! Multiplicity-weighted item bootstrap on a fixed graph and tree, behind
//! `shelfmark-bench run --bootstrap N` (DESIGN.md, step 4).
//!
//! Each replicate draws `n` items with replacement. An item drawn `c` times
//! counts `c` times everywhere: its leaf mass in every subtree, its graph
//! edges (an edge `(i, j)` weighs `w_ij * c_i * c_j`), coverage, class sizes
//! and navigation costs. The graph and the tree shape stay fixed, and a
//! duplicated item adds no self-pair (the graph has no self-edges). Draws
//! depend only on `n`, the seed and the replicate number, so every scheme
//! scored on one graph sees the same resamples: intervals and differences
//! are paired across schemes, and skill uses the reference tree's cost on
//! the same resample.
use super::{interval, SplitMix64};
use crate::{SimilarityGraph, Tree};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How often each of `n` items is drawn in replicate `replicate`.
pub fn multiplicities(n: usize, seed: u64, replicate: usize) -> Vec<f64> {
    let mut start = SplitMix64::new(seed ^ (replicate as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
    let mut rng = SplitMix64::new(start.next_u64());
    let mut counts = vec![0.0; n];
    for _ in 0..n {
        counts[rng.below(n)] += 1.0;
    }
    counts
}

/// One graph edge with the LCA nodes of its leaf pairs and their masses.
struct PairEdge {
    left: usize,
    right: usize,
    weight: f64,
    lcas: Vec<(usize, f64)>,
}

/// One item leaf with its fixed, tree-shape-only costs.
struct LeafCost {
    depth: f64,
    mass: f64,
    scan_branch: f64,
    nav_log: [f64; 4],
    shelf: usize,
}

/// Per-item tables for the structural metrics.
#[derive(Default)]
struct Structure {
    leaves: Vec<Vec<LeafCost>>,
    /// The distinct nodes at or above any of the item's leaves.
    covers: Vec<Vec<usize>>,
    class_nodes: Vec<usize>,
    placed: Vec<bool>,
}

/// Everything a replicate needs, computed once from the tree and graph.
pub struct Prepared {
    node_count: usize,
    /// (node, parent), children before parents.
    post_order: Vec<(usize, usize)>,
    item_leaves: Vec<Vec<(usize, f64)>>,
    edges: Vec<PairEdge>,
    structure: Option<Structure>,
}

/// Item-weighted metrics of one replicate, by result field name.
pub type Replicate = BTreeMap<&'static str, f64>;

impl Prepared {
    /// Precomputes the LCAs of every edge. With `structure`, also the
    /// per-leaf navigation costs and class membership the structural
    /// metrics need (a reference tree needs only its cost).
    pub fn new(tree: &Tree, graph: &SimilarityGraph, structure: bool) -> Self {
        let n = tree.item_leaves.len();
        let edges = graph
            .edges
            .iter()
            .filter(|e| e.left < n && e.right < n)
            .map(|e| PairEdge {
                left: e.left,
                right: e.right,
                weight: e.weight,
                lcas: lca_masses(tree, e.left, e.right),
            })
            .collect();
        Prepared {
            node_count: tree.nodes.len(),
            post_order: post_order(tree),
            item_leaves: tree.item_leaves.clone(),
            edges,
            structure: structure.then(|| prepare_structure(tree)),
        }
    }

    /// Metrics under item weights `c` (one weight per item).
    pub fn replicate(&self, c: &[f64]) -> Replicate {
        let mut mass = vec![0.0; self.node_count];
        for (item, leaves) in self.item_leaves.iter().enumerate() {
            for &(leaf, m) in leaves {
                mass[leaf] += c[item] * m;
            }
        }
        for &(node, parent) in &self.post_order {
            mass[parent] += mass[node];
        }
        let (mut cost, mut weight) = (0.0, 0.0);
        for e in &self.edges {
            let w = e.weight * c[e.left] * c[e.right];
            if w != 0.0 {
                cost += w * e.lcas.iter().map(|&(v, m)| m * mass[v]).sum::<f64>();
                weight += w;
            }
        }
        let denom = c.iter().sum::<f64>() * weight;
        let cost_norm = if denom > 0.0 { cost / denom } else { 0.0 };
        let mut out = Replicate::new();
        out.insert("cost_norm", cost_norm);
        out.insert(
            "revenue_norm",
            if denom > 0.0 { 1.0 - cost_norm } else { 0.0 },
        );
        if let Some(s) = &self.structure {
            structure_metrics(s, c, self.node_count, &mut out);
        }
        out
    }
}

fn prepare_structure(tree: &Tree) -> Structure {
    use crate::metrics::{branch_scan_cost, log_branch_cost, shelf_class};
    let mut s = Structure::default();
    for copies in &tree.item_leaves {
        let mut cover = Vec::new();
        let mut leaves = Vec::new();
        for &(leaf, mass) in copies {
            cover.extend(tree.ancestors(leaf));
            leaves.push(LeafCost {
                depth: tree.item_depth(leaf) as f64,
                mass,
                scan_branch: branch_scan_cost(tree, leaf),
                nav_log: [0, 1, 2, 4].map(|a| log_branch_cost(tree, leaf, a)),
                shelf: shelf_class(tree, leaf),
            });
        }
        cover.sort_unstable();
        cover.dedup();
        s.covers.push(cover);
        s.leaves.push(leaves);
        s.placed.push(
            copies
                .iter()
                .any(|&(leaf, _)| tree.nodes[leaf].parent != Some(0)),
        );
    }
    s.class_nodes = (1..tree.nodes.len())
        .filter(|&v| tree.nodes[v].code.is_some())
        .collect();
    s
}

/// Coverage, depth, balance and navigation metrics under item weights `c`,
/// computed as in `metrics::structural_stats` with every item counted
/// `c` times.
fn structure_metrics(s: &Structure, c: &[f64], node_count: usize, out: &mut Replicate) {
    use crate::metrics::{entropy_nats, gini};
    let total_items = c.iter().sum::<f64>().max(1.0);
    let mut count = vec![0.0; node_count];
    for (item, cover) in s.covers.iter().enumerate() {
        for &v in cover {
            count[v] += c[item];
        }
    }
    let (mut depth_sum, mut depth_n, mut nav_scan) = (0.0, 0.0, 0.0);
    let mut nav_log = [0.0; 4];
    let mut nav_placed = 0.0;
    let mut placed_weight = 0.0;
    for (item, leaves) in s.leaves.iter().enumerate() {
        for l in leaves {
            depth_sum += c[item] * l.depth;
            depth_n += c[item];
            nav_scan +=
                c[item] * l.mass * (l.scan_branch + (count[l.shelf] + 1.0) / 2.0) / total_items;
            if s.placed[item] {
                nav_placed += c[item] * l.mass * (l.scan_branch + (count[l.shelf] + 1.0) / 2.0);
                placed_weight += c[item] * l.mass;
            }
            for (slot, cost) in nav_log.iter_mut().zip(l.nav_log) {
                *slot += c[item] * l.mass * cost / total_items;
            }
        }
    }
    let classes: Vec<f64> = s
        .class_nodes
        .iter()
        .map(|&v| count[v])
        .filter(|&x| x > 0.0)
        .collect();
    let total = classes.iter().sum::<f64>().max(1.0);
    let entropy = entropy_nats(&classes, total);
    let k = classes.len().max(1) as f64;
    let placed = s
        .placed
        .iter()
        .zip(c)
        .filter(|(p, _)| **p)
        .map(|(_, w)| w)
        .sum::<f64>();
    out.insert("coverage", placed / total_items);
    out.insert(
        "mean_depth",
        if depth_n > 0.0 {
            depth_sum / depth_n
        } else {
            0.0
        },
    );
    out.insert(
        "normalized_entropy",
        if k > 1.0 { entropy / k.ln() } else { 0.0 },
    );
    out.insert("effective_classes", entropy.exp());
    out.insert("gini", gini(&classes));
    out.insert(
        "largest_class_share",
        classes.iter().copied().fold(0.0, f64::max) / total,
    );
    out.insert("nav_scan", nav_scan);
    out.insert(
        "nav_scan_placed",
        if placed_weight > 0.0 {
            nav_placed / placed_weight
        } else {
            0.0
        },
    );
    out.insert("nav_log_a0", nav_log[0]);
    out.insert("nav_log_a1", nav_log[1]);
    out.insert("nav_log_a2", nav_log[2]);
    out.insert("nav_log_a4", nav_log[3]);
}

/// The LCA node and mass product of every leaf pair of two items.
fn lca_masses(tree: &Tree, left: usize, right: usize) -> Vec<(usize, f64)> {
    let mut out = Vec::new();
    for &(a, ma) in &tree.item_leaves[left] {
        for &(b, mb) in &tree.item_leaves[right] {
            out.push((tree.lca(a, b), ma * mb));
        }
    }
    out
}

/// (node, parent) pairs, children before parents.
fn post_order(tree: &Tree) -> Vec<(usize, usize)> {
    let mut order = Vec::with_capacity(tree.nodes.len());
    let mut stack = vec![0usize];
    while let Some(v) = stack.pop() {
        for &child in &tree.nodes[v].children {
            order.push((child, v));
            stack.push(child);
        }
    }
    order.reverse();
    order
}

/// Replicate values of every metric, plus the reference cost and skill.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Replicates {
    pub n: usize,
    pub resamples: usize,
    pub seed: u64,
    pub values: BTreeMap<String, Vec<f64>>,
}

/// Percentile intervals of every replicated metric.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BootstrapSummary {
    pub resamples: usize,
    pub seed: u64,
    pub confidence: f64,
    pub method: String,
    pub ci: BTreeMap<String, [f64; 2]>,
}

/// Runs `resamples` replicates of `tree`. The reference cost of each
/// replicate comes from `reference` (replicates of an earlier reference
/// run) or, when absent, from `reference_tree` on the same resample.
pub fn run(
    tree: &Prepared,
    reference: Option<&[f64]>,
    reference_tree: Option<&Prepared>,
    resamples: usize,
    seed: u64,
) -> Replicates {
    let n = tree.item_leaves.len();
    let mut values = BTreeMap::<String, Vec<f64>>::new();
    for r in 0..resamples {
        let c = multiplicities(n, seed, r);
        let mut rep = tree.replicate(&c);
        let ref_cost = match (reference, reference_tree) {
            (Some(values), _) => values.get(r).copied().unwrap_or(f64::NAN),
            (None, Some(t)) => t.replicate(&c)["cost_norm"],
            (None, None) => f64::NAN,
        };
        let skill = if ref_cost < 1.0 {
            (1.0 - rep["cost_norm"]) / (1.0 - ref_cost)
        } else {
            0.0
        };
        rep.insert("ref_cost_norm", ref_cost);
        rep.insert("skill", skill);
        for (key, value) in rep {
            values.entry(key.to_owned()).or_default().push(value);
        }
    }
    Replicates {
        n,
        resamples,
        seed,
        values,
    }
}

impl Replicates {
    /// The reference (average-linkage) cost of each replicate, for
    /// `run(.., Some(costs), ..)` of another tree on the same resamples.
    /// This is `ref_cost_norm`, never `cost_norm`: a reference run scores
    /// some scheme's tree too, and its `cost_norm` is that tree's cost.
    pub fn reference_costs(&self) -> Option<&[f64]> {
        self.values.get("ref_cost_norm").map(Vec::as_slice)
    }

    /// 95% percentile intervals of every metric with finite replicates.
    pub fn summary(&self) -> BootstrapSummary {
        let confidence = 0.95;
        BootstrapSummary {
            resamples: self.resamples,
            seed: self.seed,
            confidence,
            method: "percentile; items resampled with replacement on the fixed graph and tree"
                .into(),
            ci: self
                .values
                .iter()
                .filter(|(_, v)| v.iter().any(|x| x.is_finite()))
                .map(|(k, v)| (k.clone(), interval(v, confidence)))
                .collect(),
        }
    }
}
