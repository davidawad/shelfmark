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
    let mut leaf_mass = vec![0.0; tree.nodes.len()];
    for copies in &tree.item_leaves {
        for &(leaf, mass) in copies {
            leaf_mass[leaf] += mass;
        }
    }
    let mut subtree_mass = vec![0.0; tree.nodes.len()];
    fn accumulate(tree: &Tree, node: usize, leaves: &[f64], totals: &mut [f64]) -> f64 {
        let total = leaves[node]
            + tree.nodes[node]
                .children
                .iter()
                .map(|&child| accumulate(tree, child, leaves, totals))
                .sum::<f64>();
        totals[node] = total;
        total
    }
    accumulate(tree, 0, &leaf_mass, &mut subtree_mass);
    for edge in &graph.edges {
        if edge.left >= n || edge.right >= n {
            continue;
        }
        let mut pair_cost = 0.0;
        let mut pair_dist = 0.0;
        for &(a, ma) in &tree.item_leaves[edge.left] {
            for &(b, mb) in &tree.item_leaves[edge.right] {
                let mass = ma * mb;
                let lca = tree.lca(a, b);
                pair_cost += mass * subtree_mass[lca];
                pair_dist += mass * tree_distance(tree, a, b) as f64;
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

fn tree_distance(tree: &Tree, a: usize, b: usize) -> usize {
    let l = tree.lca(a, b);
    tree_depth(tree, a) + tree_depth(tree, b) - 2 * tree_depth(tree, l)
}
fn tree_depth(tree: &Tree, mut node: usize) -> usize {
    let mut d = 0;
    while let Some(p) = tree.nodes[node].parent {
        d += 1;
        node = p;
    }
    d
}

fn structural_stats(tree: &Tree) -> StructuralStats {
    let mut depths = Vec::new();
    let mut classes = Vec::new();
    let mut branches = Vec::new();
    let mut nav_scan = 0.0;
    let mut nav_log = [0.0; 3];
    let n = tree.item_leaves.len().max(1) as f64;
    for leaves in &tree.item_leaves {
        for &(leaf, mass) in leaves {
            let d = tree.item_depth(leaf);
            depths.push(d);
            let mut branch_cost = 0.0;
            let mut node = leaf;
            while let Some(parent) = tree.nodes[node].parent {
                let fanout = tree.nodes[parent].children.len();
                if fanout > 1 {
                    branch_cost += (fanout as f64 + 1.0) / 2.0;
                }
                node = parent;
            }
            // Terminal scan cost includes every item attached anywhere beneath
            // the leaf's deepest class, matching a physical class shelf scan.
            let class = tree
                .ancestors(leaf)
                .into_iter()
                .skip(1)
                .find(|&v| tree.nodes[v].code.is_some())
                .unwrap_or(leaf);
            let class_items = tree
                .item_leaves
                .iter()
                .filter(|copies| {
                    copies
                        .iter()
                        .any(|(other, _)| is_descendant(tree, *other, class))
                })
                .count();
            nav_scan += mass * (branch_cost + (class_items as f64 + 1.0) / 2.0) / n;
            for a in 0..3 {
                let mut log_cost = 0.0;
                let mut node = leaf;
                while let Some(parent) = tree.nodes[node].parent {
                    let fanout = tree.nodes[parent].children.len().max(1);
                    if fanout > 1 {
                        log_cost += a as f64 + (fanout as f64).log2();
                    }
                    node = parent;
                }
                nav_log[a] += mass * log_cost / n;
            }
        }
    }
    // Number of items at each used class node (placement frequency).
    for node in 1..tree.nodes.len() {
        if tree.nodes[node].code.is_some() {
            let count = tree
                .item_leaves
                .iter()
                .filter(|copies| {
                    copies
                        .iter()
                        .any(|(leaf, _)| is_descendant(tree, *leaf, node))
                })
                .count();
            if count > 0 {
                classes.push(count as f64);
            }
        }
        if !tree.nodes[node].children.is_empty() {
            branches.push(tree.nodes[node].children.len());
        }
    }
    let mut sorted = depths.clone();
    sorted.sort_unstable();
    let quant = |v: &[usize], q: f64| {
        if v.is_empty() {
            0
        } else {
            v[((v.len() - 1) as f64 * q).round() as usize]
        }
    };
    let total = classes.iter().sum::<f64>().max(1.0);
    let entropy = -classes
        .iter()
        .map(|c| {
            let p = c / total;
            if p > 0.0 {
                p * p.ln()
            } else {
                0.0
            }
        })
        .sum::<f64>();
    let k = classes.len().max(1) as f64;
    let mut sorted_sizes = classes.clone();
    sorted_sizes.sort_by(f64::total_cmp);
    let gini = if classes.is_empty() {
        0.0
    } else {
        let sum = sorted_sizes.iter().sum::<f64>();
        let weighted = sorted_sizes
            .iter()
            .enumerate()
            .map(|(i, x)| (i + 1) as f64 * x)
            .sum::<f64>();
        2.0 * weighted / (classes.len() as f64 * sum)
            - (classes.len() as f64 + 1.0) / (classes.len() as f64)
    };
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
        p50_depth: quant(&sorted, 0.5),
        p95_depth: quant(&sorted, 0.95),
        max_depth: sorted.last().copied().unwrap_or(0),
        mean_branching,
        p95_branching: quant(&branches, 0.95),
        max_branching: branches.last().copied().unwrap_or(0),
        root_fanout: tree.nodes[0].children.len(),
        nodes_over_50_children: branches.iter().filter(|&&b| b > 50).count(),
        normalized_entropy: if k > 1.0 { entropy / k.ln() } else { 0.0 },
        effective_classes: entropy.exp(),
        gini,
        largest_class_share: classes.iter().copied().fold(0.0, f64::max) / total,
        nav_scan,
        nav_log_a0: nav_log[0],
        nav_log_a1: nav_log[1],
        nav_log_a2: nav_log[2],
    }
}
fn is_descendant(tree: &Tree, mut node: usize, ancestor: usize) -> bool {
    loop {
        if node == ancestor {
            return true;
        }
        match tree.nodes[node].parent {
            Some(p) => node = p,
            None => return false,
        }
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
