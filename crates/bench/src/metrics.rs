use crate::{SimilarityGraph, Tree};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralStats {
    pub mean_depth: f64,
    pub p50_depth: usize,
    pub p95_depth: usize,
    pub max_depth: usize,
    pub mean_branching: f64,
    pub p95_branching: usize,
    pub max_branching: usize,
    pub root_fanout: usize,
    pub nodes_over_50_children: usize,
    pub normalized_entropy: f64,
    pub effective_classes: f64,
    pub gini: f64,
    pub largest_class_share: f64,
    pub nav_scan: f64,
    pub nav_log_a0: f64,
    pub nav_log_a1: f64,
    pub nav_log_a2: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeScore {
    pub cost: f64,
    pub revenue: f64,
    pub edge_weight: f64,
    pub cost_norm: f64,
    pub revenue_norm: f64,
    pub expected_tree_distance: f64,
    pub structure: StructuralStats,
}

/// Dasgupta (2016): `cost(T) = Σ_{i<j} w_ij · |leaves(LCA_T(i,j))|`.
/// For unit mass leaves this is the number of corpus items below the LCA.
/// Fractional placements replace that count by the summed leaf mass.
/// Moseley–Wang (2023): `revenue(T) = n·Σw_ij − cost(T)`.
pub fn score_tree(tree: &Tree, graph: &SimilarityGraph) -> TreeScore {
    let n = tree.item_leaves.len();
    let mut cost = 0.0;
    let mut weight = 0.0;
    let mut distance = 0.0;
    let order = top_down_order(tree);
    let mut depths = vec![0usize; tree.nodes.len()];
    for &node in &order[1..] {
        depths[node] = tree.nodes[node].parent.map_or(0, |p| depths[p] + 1);
    }
    let mut leaf_mass = vec![0.0; tree.nodes.len()];
    for copies in &tree.item_leaves {
        for &(leaf, mass) in copies {
            leaf_mass[leaf] += mass;
        }
    }
    // Children before parents, each summed in child order as a recursion would.
    let mut subtree_mass = vec![0.0; tree.nodes.len()];
    for &node in order.iter().rev() {
        subtree_mass[node] = leaf_mass[node]
            + tree.nodes[node]
                .children
                .iter()
                .map(|&child| subtree_mass[child])
                .sum::<f64>();
    }
    for edge in &graph.edges {
        if edge.left >= n || edge.right >= n {
            continue;
        }
        let mut pair_cost = 0.0;
        let mut pair_dist = 0.0;
        for &(a, ma) in &tree.item_leaves[edge.left] {
            for &(b, mb) in &tree.item_leaves[edge.right] {
                let mass = ma * mb;
                let lca = lca_by_depth(tree, &depths, a, b);
                pair_cost += mass * subtree_mass[lca];
                pair_dist += mass * (depths[a] + depths[b] - 2 * depths[lca]) as f64;
            }
        }
        cost += edge.weight * pair_cost;
        distance += edge.weight * pair_dist;
        weight += edge.weight;
    }
    let denom = n as f64 * weight;
    let revenue = denom - cost;
    TreeScore {
        cost,
        revenue,
        edge_weight: weight,
        cost_norm: if denom > 0.0 { cost / denom } else { 0.0 },
        revenue_norm: if denom > 0.0 { revenue / denom } else { 0.0 },
        expected_tree_distance: if weight > 0.0 { distance / weight } else { 0.0 },
        structure: structural_stats(tree),
    }
}

/// Nodes reachable from the root, every parent before its children.
fn top_down_order(tree: &Tree) -> Vec<usize> {
    let mut order = vec![0];
    let mut next = 0;
    while next < order.len() {
        order.extend_from_slice(&tree.nodes[order[next]].children);
        next += 1;
    }
    order
}

fn lca_by_depth(tree: &Tree, depths: &[usize], mut a: usize, mut b: usize) -> usize {
    while depths[a] > depths[b] {
        a = tree.nodes[a].parent.unwrap_or(0);
    }
    while depths[b] > depths[a] {
        b = tree.nodes[b].parent.unwrap_or(0);
    }
    while a != b {
        a = tree.nodes[a].parent.unwrap_or(0);
        b = tree.nodes[b].parent.unwrap_or(0);
    }
    a
}

/// Menu-scan cost from a leaf to the root: half the fan-out of every
/// branching ancestor, plus half a step for the choice itself.
pub(crate) fn branch_scan_cost(tree: &Tree, leaf: usize) -> f64 {
    let mut cost = 0.0;
    let mut node = leaf;
    while let Some(parent) = tree.nodes[node].parent {
        let fanout = tree.nodes[parent].children.len();
        if fanout > 1 {
            cost += (fanout as f64 + 1.0) / 2.0;
        }
        node = parent;
    }
    cost
}

/// Binary-search menu cost from a leaf to the root with per-branch cost `a`.
pub(crate) fn log_branch_cost(tree: &Tree, leaf: usize, a: usize) -> f64 {
    let mut cost = 0.0;
    let mut node = leaf;
    while let Some(parent) = tree.nodes[node].parent {
        let fanout = tree.nodes[parent].children.len().max(1);
        if fanout > 1 {
            cost += a as f64 + (fanout as f64).log2();
        }
        node = parent;
    }
    cost
}

/// The deepest class (a node with a code) at or above a leaf's parent, or the
/// leaf itself when none exists.
pub(crate) fn shelf_class(tree: &Tree, leaf: usize) -> usize {
    tree.ancestors(leaf)
        .into_iter()
        .skip(1)
        .find(|&v| tree.nodes[v].code.is_some())
        .unwrap_or(leaf)
}

/// Number of items with at least one placement at or beneath each node.
fn items_under(tree: &Tree) -> Vec<usize> {
    let mut counts = vec![0usize; tree.nodes.len()];
    let mut seen = vec![usize::MAX; tree.nodes.len()];
    for (item, copies) in tree.item_leaves.iter().enumerate() {
        for &(leaf, _) in copies {
            let mut node = Some(leaf);
            // Stop at a node this item already counted: its ancestors are too.
            while let Some(v) = node {
                if seen[v] == item {
                    break;
                }
                seen[v] = item;
                counts[v] += 1;
                node = tree.nodes[v].parent;
            }
        }
    }
    counts
}

/// Placement frequency of every used class node, and the child count of every
/// branching node.
fn class_sizes_and_branches(tree: &Tree, under: &[usize]) -> (Vec<f64>, Vec<usize>) {
    let mut classes = Vec::new();
    let mut branches = Vec::new();
    for (node, &count) in tree.nodes.iter().zip(under).skip(1) {
        if node.code.is_some() && count > 0 {
            classes.push(count as f64);
        }
        if !node.children.is_empty() {
            branches.push(node.children.len());
        }
    }
    (classes, branches)
}

fn quantile(v: &[usize], q: f64) -> usize {
    if v.is_empty() {
        0
    } else {
        v[((v.len() - 1) as f64 * q).round() as usize]
    }
}

pub(crate) fn entropy_nats(classes: &[f64], total: f64) -> f64 {
    -classes
        .iter()
        .map(|c| {
            let p = c / total;
            if p > 0.0 {
                p * p.ln()
            } else {
                0.0
            }
        })
        .sum::<f64>()
}

pub(crate) fn gini(classes: &[f64]) -> f64 {
    if classes.is_empty() {
        return 0.0;
    }
    let mut sorted_sizes = classes.to_vec();
    sorted_sizes.sort_by(f64::total_cmp);
    let sum = sorted_sizes.iter().sum::<f64>();
    let weighted = sorted_sizes
        .iter()
        .enumerate()
        .map(|(i, x)| (i + 1) as f64 * x)
        .sum::<f64>();
    2.0 * weighted / (classes.len() as f64 * sum)
        - (classes.len() as f64 + 1.0) / (classes.len() as f64)
}

fn structural_stats(tree: &Tree) -> StructuralStats {
    let mut depths = Vec::new();
    let mut nav_scan = 0.0;
    let mut nav_log = [0.0; 3];
    let n = tree.item_leaves.len().max(1) as f64;
    let under = items_under(tree);
    for leaves in &tree.item_leaves {
        for &(leaf, mass) in leaves {
            depths.push(tree.item_depth(leaf));
            // Terminal scan cost includes every item attached anywhere beneath
            // the leaf's deepest class, matching a physical class shelf scan.
            let class_items = under[shelf_class(tree, leaf)];
            nav_scan +=
                mass * (branch_scan_cost(tree, leaf) + (class_items as f64 + 1.0) / 2.0) / n;
            for (a, slot) in nav_log.iter_mut().enumerate() {
                *slot += mass * log_branch_cost(tree, leaf, a) / n;
            }
        }
    }
    let (classes, mut branches) = class_sizes_and_branches(tree, &under);
    let mut sorted = depths.clone();
    sorted.sort_unstable();
    let total = classes.iter().sum::<f64>().max(1.0);
    let entropy = entropy_nats(&classes, total);
    let k = classes.len().max(1) as f64;
    branches.sort_unstable();
    let mean_branching = if branches.is_empty() {
        0.0
    } else {
        branches.iter().sum::<usize>() as f64 / branches.len() as f64
    };
    StructuralStats {
        mean_depth: if depths.is_empty() {
            0.0
        } else {
            depths.iter().sum::<usize>() as f64 / depths.len() as f64
        },
        p50_depth: quantile(&sorted, 0.5),
        p95_depth: quantile(&sorted, 0.95),
        max_depth: sorted.last().copied().unwrap_or(0),
        mean_branching,
        p95_branching: quantile(&branches, 0.95),
        max_branching: branches.last().copied().unwrap_or(0),
        root_fanout: tree.nodes[0].children.len(),
        nodes_over_50_children: branches.iter().filter(|&&b| b > 50).count(),
        normalized_entropy: if k > 1.0 { entropy / k.ln() } else { 0.0 },
        effective_classes: entropy.exp(),
        gini: gini(&classes),
        largest_class_share: classes.iter().copied().fold(0.0, f64::max) / total,
        nav_scan,
        nav_log_a0: nav_log[0],
        nav_log_a1: nav_log[1],
        nav_log_a2: nav_log[2],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Edge;
    use crate::{induce_tree, ItemPlacement};
    use shelfmark_core::scheme::{ClassDef, SchemeDoc};
    fn doc() -> SchemeDoc {
        SchemeDoc {
            schema: None,
            name: "t".into(),
            version: "1".into(),
            title: "t".into(),
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
        }
    }
    #[test]
    fn hand_computed_cost_and_mw_revenue() {
        let t = induce_tree(
            &doc(),
            &[
                ItemPlacement {
                    id: "1".into(),
                    classes: vec![("A".into(), 1.0)],
                },
                ItemPlacement {
                    id: "2".into(),
                    classes: vec![("A".into(), 1.0)],
                },
                ItemPlacement {
                    id: "3".into(),
                    classes: vec![("B".into(), 1.0)],
                },
            ],
        )
        .unwrap();
        let g = SimilarityGraph {
            n: 3,
            k: 2,
            tau: 0.0,
            edges: vec![
                Edge {
                    left: 0,
                    right: 1,
                    weight: 1.0,
                },
                Edge {
                    left: 0,
                    right: 2,
                    weight: 1.0,
                },
                Edge {
                    left: 1,
                    right: 2,
                    weight: 1.0,
                },
            ],
        };
        let s = score_tree(&t, &g);
        assert_eq!(s.cost, 8.0);
        assert_eq!(s.revenue, 1.0);
        assert_eq!(s.cost + s.revenue, 9.0);
    }
    #[test]
    fn root_only_tree_has_unit_cost() {
        let t = induce_tree(
            &doc(),
            &[
                ItemPlacement {
                    id: "1".into(),
                    classes: vec![],
                },
                ItemPlacement {
                    id: "2".into(),
                    classes: vec![],
                },
            ],
        )
        .unwrap();
        let g = SimilarityGraph {
            n: 2,
            k: 1,
            tau: 0.,
            edges: vec![Edge {
                left: 0,
                right: 1,
                weight: 1.,
            }],
        };
        assert_eq!(score_tree(&t, &g).cost_norm, 1.0);
    }
}
