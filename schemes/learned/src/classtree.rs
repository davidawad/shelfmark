//! The class tree over a fixed shelf order, and the exact objective.
//!
//! Every class is a contiguous run `lo..hi` of the shelf order and its
//! children tile that run left to right (the tree is order-preserving).
//! Node 0 is the virtual root, which the harness also adds; leaf classes
//! hold the items of their run.

use crate::graph::Graph;
use serde::Serialize;

pub const NONE: usize = usize::MAX;

/// Objective weights and shape bounds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Params {
    /// Weight of the navigation term against normalized Dasgupta cost.
    pub lambda: f64,
    /// Per-branch overhead in bits (the harness's `nav_log` `a`).
    pub branch_cost: f64,
    pub min_children: usize,
    pub max_children: usize,
    /// Deepest class level (top-level classes are level 1).
    pub max_depth: usize,
    /// Smallest leaf class the fit creates.
    pub min_leaf: usize,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            lambda: 0.005,
            branch_cost: 1.0,
            min_children: 2,
            max_children: 12,
            max_depth: 4,
            min_leaf: 5,
        }
    }
}

/// `J = dasgupta + lambda * nav`; see the crate README for the definitions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Objective {
    pub dasgupta: f64,
    pub nav: f64,
    pub total: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub parent: usize,
    pub children: Vec<usize>,
    pub lo: usize,
    pub hi: usize,
}

impl Node {
    pub fn len(&self) -> usize {
        self.hi - self.lo
    }
    pub fn is_empty(&self) -> bool {
        self.hi == self.lo
    }
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassTree {
    pub nodes: Vec<Node>,
    /// Shelf position -> item.
    pub order: Vec<usize>,
}

/// Items in a leaf of `k` items pay `branch_cost + log2 k` to pick theirs.
pub fn leaf_nav(k: usize, branch_cost: f64) -> f64 {
    if k > 1 {
        k as f64 * (branch_cost + (k as f64).log2())
    } else {
        0.0
    }
}

impl ClassTree {
    pub fn n(&self) -> usize {
        self.order.len()
    }
    /// Live nodes, parents before children, siblings in shelf order.
    pub fn preorder(&self) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack = vec![0];
        while let Some(v) = stack.pop() {
            out.push(v);
            stack.extend(self.nodes[v].children.iter().rev());
        }
        out
    }
    pub fn leaves(&self) -> Vec<usize> {
        self.preorder()
            .into_iter()
            .filter(|&v| v != 0 && self.nodes[v].is_leaf())
            .collect()
    }
    pub fn depth(&self, mut v: usize) -> usize {
        let mut d = 0;
        while self.nodes[v].parent != NONE {
            v = self.nodes[v].parent;
            d += 1;
        }
        d
    }
    /// Item -> leaf class node.
    pub fn item_leaf(&self) -> Vec<usize> {
        let mut out = vec![NONE; self.n()];
        for leaf in self.leaves() {
            let node = &self.nodes[leaf];
            for &item in &self.order[node.lo..node.hi] {
                out[item] = leaf;
            }
        }
        out
    }
    pub fn lca(&self, depth: &[usize], mut a: usize, mut b: usize) -> usize {
        while depth[a] > depth[b] {
            a = self.nodes[a].parent;
        }
        while depth[b] > depth[a] {
            b = self.nodes[b].parent;
        }
        while a != b {
            a = self.nodes[a].parent;
            b = self.nodes[b].parent;
        }
        a
    }
    pub fn depths(&self) -> Vec<usize> {
        let mut depth = vec![0; self.nodes.len()];
        for v in self.preorder() {
            for &c in &self.nodes[v].children {
                depth[c] = depth[v] + 1;
            }
        }
        depth
    }
    /// Fan-out a reader faces at `v`: child classes, or items in a leaf.
    pub fn fanout(&self, v: usize) -> usize {
        let node = &self.nodes[v];
        if node.is_leaf() {
            node.len()
        } else {
            node.children.len()
        }
    }

    /// Exact objective. `dasgupta = sum_e w_e |items(lca_e)| / (n W)` with
    /// items under their leaf class (the harness's `cost_norm`), and
    /// `nav = (1/n) sum_v |items(v)| (a + log2 fanout(v))` over nodes with
    /// fan-out above 1 (the harness's `nav_log` for `a = branch_cost`).
    pub fn evaluate(&self, g: &Graph, p: &Params) -> Objective {
        let n = self.n();
        let leaf = self.item_leaf();
        let depth = self.depths();
        let mut cost = 0.0;
        for &(a, b, w) in &g.edges {
            let v = self.lca(&depth, leaf[a], leaf[b]);
            cost += w * self.nodes[v].len() as f64;
        }
        let mut nav = 0.0;
        for v in self.preorder() {
            let b = self.fanout(v);
            if b > 1 {
                nav += self.nodes[v].len() as f64 * (p.branch_cost + (b as f64).log2());
            }
        }
        let dasgupta = if g.total > 0.0 && n > 0 {
            cost / (n as f64 * g.total)
        } else {
            0.0
        };
        let nav = if n > 0 { nav / n as f64 } else { 0.0 };
        Objective {
            dasgupta,
            nav,
            total: dasgupta + p.lambda * nav,
        }
    }

    /// Checks order preservation (runs tile), branching and depth bounds.
    pub fn validate(&self, p: &Params) -> Result<(), String> {
        let root = &self.nodes[0];
        if root.parent != NONE || root.lo != 0 || root.hi != self.n() {
            return Err("root must span the whole shelf".into());
        }
        let depth = self.depths();
        for v in self.preorder() {
            let node = &self.nodes[v];
            if node.is_empty() {
                return Err(format!("node {v} is empty"));
            }
            if v != 0 && depth[v] > p.max_depth {
                return Err(format!("node {v} at depth {} > {}", depth[v], p.max_depth));
            }
            if node.is_leaf() {
                if v == 0 {
                    return Err("root has no classes".into());
                }
                continue;
            }
            let k = node.children.len();
            if k < p.min_children || k > p.max_children {
                return Err(format!("node {v} has {k} children"));
            }
            let mut at = node.lo;
            for &c in &node.children {
                let child = &self.nodes[c];
                if child.parent != v || child.lo != at {
                    return Err(format!("child {c} of {v} breaks the shelf order"));
                }
                at = child.hi;
            }
            if at != node.hi {
                return Err(format!("children of {v} do not cover it"));
            }
        }
        Ok(())
    }

    /// Drops unreachable nodes and renumbers the rest in preorder.
    pub fn compact(&self) -> ClassTree {
        let order = self.preorder();
        let mut map = vec![NONE; self.nodes.len()];
        for (k, &v) in order.iter().enumerate() {
            map[v] = k;
        }
        let nodes = order
            .iter()
            .map(|&v| {
                let node = &self.nodes[v];
                Node {
                    parent: if node.parent == NONE {
                        NONE
                    } else {
                        map[node.parent]
                    },
                    children: node.children.iter().map(|&c| map[c]).collect(),
                    lo: node.lo,
                    hi: node.hi,
                }
            })
            .collect();
        ClassTree {
            nodes,
            order: self.order.clone(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Root with leaves {0,1} and {2,3}.
    pub fn two_leaves() -> ClassTree {
        let node = |parent, children: Vec<usize>, lo, hi| Node {
            parent,
            children,
            lo,
            hi,
        };
        ClassTree {
            nodes: vec![
                node(NONE, vec![1, 2], 0, 4),
                node(0, vec![], 0, 2),
                node(0, vec![], 2, 4),
            ],
            order: vec![0, 1, 2, 3],
        }
    }

    #[test]
    fn hand_computed_objective() {
        let t = two_leaves();
        let g = Graph::new(4, vec![(0, 1, 1.0), (2, 3, 1.0), (1, 2, 2.0)]);
        let p = Params {
            lambda: 0.5,
            min_leaf: 1,
            ..Params::default()
        };
        let o = t.evaluate(&g, &p);
        // Cost: 1*2 + 1*2 + 2*4 = 12 over n W = 4 * 4.
        assert!((o.dasgupta - 12.0 / 16.0).abs() < 1e-12);
        // Nav: root 4 * (1 + 1) + each leaf 2 * (1 + 1), over 4 items.
        assert!((o.nav - 4.0).abs() < 1e-12);
        assert!((o.total - (0.75 + 2.0)).abs() < 1e-12);
        t.validate(&p).unwrap();
    }

    #[test]
    fn validation_rejects_broken_runs() {
        let mut t = two_leaves();
        t.nodes[2].lo = 3;
        assert!(t.validate(&Params::default()).is_err());
    }
}
