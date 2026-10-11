//! Exact dynamic program that collapses the oriented binary tree into a
//! class tree with 2..=max_children children per class and at most
//! max_depth levels, minimizing the objective over all such contractions.
//!
//! A class is a binary node kept as is; the binary nodes between a class
//! and its child classes are contracted into it, so their edges meet at the
//! class. A binary subtree smaller than `min_leaf` never becomes a class of
//! its own: it is glued to its sibling and its items join the leaf class
//! next to it on the shelf, which keeps every class a contiguous run.

use crate::binary::{Binary, NONE};
use crate::classtree::{leaf_nav, ClassTree, Node, Params};
use crate::graph::Graph;

const INF: f64 = f64::INFINITY;

/// A maximal run of binary nodes whose small side is glued on.
#[derive(Debug, Clone)]
struct Chain {
    top: usize,
    n: usize,
    /// Edge weight meeting anywhere on the chain or inside a glued subtree
    /// (charged at the chain's size: an upper bound for the glued part).
    meet: f64,
    /// Edge weight inside the whole subtree.
    inside: f64,
    kids: Option<[usize; 2]>,
    /// Glued binary subtrees and whether each sits left of its sibling.
    glued: Vec<(usize, bool)>,
}

fn chains(t: &Binary, meet: &[f64], inside: &[f64], min_leaf: usize) -> Vec<Chain> {
    let mut out: Vec<Chain> = Vec::new();
    let mut todo = vec![(t.root, NONE, 0usize)];
    while let Some((v, parent, slot)) = todo.pop() {
        let id = out.len();
        out.push(Chain {
            top: v,
            n: t.size[v],
            meet: 0.0,
            inside: inside[v],
            kids: None,
            glued: Vec::new(),
        });
        if parent != NONE {
            out[parent].kids.get_or_insert([NONE; 2])[slot] = id;
        }
        let mut cur = v;
        loop {
            out[id].meet += meet[cur];
            let Some([a, b]) = t.children(cur) else { break };
            let (big_a, big_b) = (t.size[a] >= min_leaf, t.size[b] >= min_leaf);
            if big_a && big_b {
                todo.push((b, id, 1));
                todo.push((a, id, 0));
                break;
            } else if big_a {
                out[id].glued.push((b, false));
                out[id].meet += inside[b];
                cur = a;
            } else if big_b {
                out[id].glued.push((a, true));
                out[id].meet += inside[a];
                cur = b;
            } else {
                break;
            }
        }
    }
    // Children are pushed after their parent; a chain with kids gets both.
    out
}

struct Dp<'a> {
    c: &'a [Chain],
    p: &'a Params,
    n: f64,
    nw: f64,
    /// f[x][d]: best cost of chain x as a class with d levels available.
    f: Vec<Vec<f64>>,
    best_m: Vec<Vec<usize>>,
}

impl Dp<'_> {
    fn leaf(&self, x: usize) -> f64 {
        let c = &self.c[x];
        c.n as f64 * c.inside / self.nw + self.p.lambda * leaf_nav(c.n, self.p.branch_cost) / self.n
    }
    /// Best cost of covering chain y's subtree with m child classes
    /// (index m), y at contraction depth k below the class.
    fn frontier(&self, y: usize, k: usize, mult: f64, d: usize) -> Vec<f64> {
        let cap = self.p.max_children - k;
        let mut arr = vec![INF; cap + 1];
        arr[1] = self.f[y][d - 1];
        if let (Some([a, b]), true) = (self.c[y].kids, cap >= 2) {
            let (fa, fb) = (
                self.frontier(a, k + 1, mult, d),
                self.frontier(b, k + 1, mult, d),
            );
            let here = mult * self.c[y].meet;
            for (m1, &x1) in fa.iter().enumerate().skip(1) {
                for (m2, &x2) in fb.iter().enumerate().skip(1) {
                    if m1 + m2 <= cap && here + x1 + x2 < arr[m1 + m2] {
                        arr[m1 + m2] = here + x1 + x2;
                    }
                }
            }
        }
        arr
    }
    fn branch(&self, x: usize, m: usize) -> f64 {
        self.p.lambda * self.c[x].n as f64 * (self.p.branch_cost + (m as f64).log2()) / self.n
    }
    /// (cost, m) of making chain x a class with child classes.
    fn internal(&self, x: usize, d: usize) -> (f64, usize) {
        let Some([a, b]) = self.c[x].kids else {
            return (INF, 0);
        };
        if d < 2 {
            return (INF, 0);
        }
        let mult = self.c[x].n as f64 / self.nw;
        let k = 1;
        let (fa, fb) = (self.frontier(a, k, mult, d), self.frontier(b, k, mult, d));
        let mut best = (INF, 0);
        for m in self.p.min_children.max(2)..=self.p.max_children {
            let mut conv = INF;
            for m1 in 1..m {
                let m2 = m - m1;
                if m1 < fa.len() && m2 < fb.len() && fa[m1] + fb[m2] < conv {
                    conv = fa[m1] + fb[m2];
                }
            }
            let cost = mult * self.c[x].meet + conv + self.branch(x, m);
            if cost < best.0 {
                best = (cost, m);
            }
        }
        best
    }
    /// Child classes of y's subtree achieving `frontier(y)[m]`, left to
    /// right, plus the chains contracted on the way.
    fn trace(&self, y: usize, k: usize, mult: f64, d: usize, m: usize, out: &mut Trace) {
        if m == 1 {
            out.classes.push(y);
            return;
        }
        let [a, b] = self.c[y].kids.expect("contracted chains have kids");
        out.contracted.push(y);
        let (fa, fb) = (
            self.frontier(a, k + 1, mult, d),
            self.frontier(b, k + 1, mult, d),
        );
        let (m1, m2) = split(&fa, &fb, m);
        self.trace(a, k + 1, mult, d, m1, out);
        self.trace(b, k + 1, mult, d, m2, out);
    }
}

/// The first (m1, m2) with m1 + m2 = m minimizing fa[m1] + fb[m2].
fn split(fa: &[f64], fb: &[f64], m: usize) -> (usize, usize) {
    let mut best = (INF, 0, 0);
    for m1 in 1..m {
        let m2 = m - m1;
        if m1 < fa.len() && m2 < fb.len() && fa[m1] + fb[m2] < best.0 {
            best = (fa[m1] + fb[m2], m1, m2);
        }
    }
    (best.1, best.2)
}

#[derive(Default)]
struct Trace {
    classes: Vec<usize>,
    contracted: Vec<usize>,
}

/// Result of the collapse: the class tree and the DP's own objective value.
/// The two agree exactly unless a run was glued above leaf level: the DP
/// charges glued edges at the gluing class (too high) and stops a glued
/// item's navigation there (too low).
pub struct Collapsed {
    pub tree: ClassTree,
    pub dp_total: f64,
}

pub fn collapse(t: &Binary, g: &Graph, p: &Params) -> Result<Collapsed, String> {
    let lcas = t.edge_lcas(g);
    let mut meet = vec![0.0; t.len()];
    for (e, &v) in lcas.iter().enumerate() {
        meet[v] += g.edges[e].2;
    }
    let mut inside = meet.clone();
    for v in t.preorder().into_iter().rev() {
        if let Some([a, b]) = t.children(v) {
            inside[v] += inside[a] + inside[b];
        }
    }
    let c = chains(t, &meet, &inside, p.min_leaf);
    if c[0].kids.is_none() {
        return Err("corpus too small to split into classes".into());
    }
    let levels = p.max_depth + 1;
    let mut dp = Dp {
        c: &c,
        p,
        n: t.n as f64,
        nw: t.n as f64 * g.total.max(f64::MIN_POSITIVE),
        f: vec![vec![INF; levels + 1]; c.len()],
        best_m: vec![vec![0; levels + 1]; c.len()],
    };
    for x in (0..c.len()).rev() {
        let leaf = dp.leaf(x);
        for d in 1..=levels {
            let (cost, m) = dp.internal(x, d);
            let (cost, m) = if x == 0 || cost < leaf {
                (cost, m)
            } else {
                (leaf, 0)
            };
            dp.f[x][d] = cost;
            dp.best_m[x][d] = m;
        }
    }
    let dp_total = dp.f[0][levels];
    if !dp_total.is_finite() {
        return Err("no class tree satisfies the bounds".into());
    }
    Ok(Collapsed {
        tree: build_tree(t, &dp, levels)?,
        dp_total,
    })
}

fn build_tree(t: &Binary, dp: &Dp, levels: usize) -> Result<ClassTree, String> {
    let c = dp.c;
    let order = t.leaf_order();
    let mut pos = vec![0; t.n];
    for (k, &i) in order.iter().enumerate() {
        pos[i] = k;
    }
    let mut span = vec![(usize::MAX, 0usize); t.len()];
    for v in t.preorder().into_iter().rev() {
        span[v] = match t.children(v) {
            None => (pos[v], pos[v] + 1),
            Some([a, b]) => (span[a].0.min(span[b].0), span[a].1.max(span[b].1)),
        };
    }
    let (lo, hi) = span[t.root];
    let mut nodes = vec![Node {
        parent: NONE,
        children: vec![],
        lo,
        hi,
    }];
    let mut glued = Vec::new();
    let mut todo = vec![(0usize, 0usize, levels)];
    while let Some((x, node, d)) = todo.pop() {
        let m = dp.best_m[x][d];
        if m == 0 {
            continue;
        }
        glued.extend(c[x].glued.iter().copied());
        let [a, b] = c[x].kids.expect("internal chains have kids");
        let mult = c[x].n as f64 / dp.nw;
        let (fa, fb) = (dp.frontier(a, 1, mult, d), dp.frontier(b, 1, mult, d));
        let (m1, m2) = split(&fa, &fb, m);
        let mut tr = Trace::default();
        dp.trace(a, 1, mult, d, m1, &mut tr);
        dp.trace(b, 1, mult, d, m2, &mut tr);
        for y in tr.contracted {
            glued.extend(c[y].glued.iter().copied());
        }
        for y in tr.classes {
            let (lo, hi) = span[c[y].top];
            let id = nodes.len();
            nodes.push(Node {
                parent: node,
                children: vec![],
                lo,
                hi,
            });
            nodes[node].children.push(id);
            todo.push((y, id, d - 1));
        }
    }
    let mut tree = ClassTree { nodes, order };
    attach_glued(&mut tree, &glued, &span);
    Ok(tree)
}

/// Gives each glued run to the leaf class it touches on its sibling's side,
/// then recomputes the runs of the classes above.
fn attach_glued(tree: &mut ClassTree, glued: &[(usize, bool)], span: &[(usize, usize)]) {
    let n = tree.n();
    let mut leaf_at = vec![NONE; n];
    for leaf in tree.leaves() {
        let node = &tree.nodes[leaf];
        leaf_at[node.lo..node.hi].iter_mut().for_each(|x| *x = leaf);
    }
    for &(g, left) in glued {
        let (lo, hi) = span[g];
        if left {
            let target = (hi..n).map(|p| leaf_at[p]).find(|&l| l != NONE);
            let target = target.expect("a leaf follows every left-glued run");
            tree.nodes[target].lo = tree.nodes[target].lo.min(lo);
        } else {
            let target = (0..lo).rev().map(|p| leaf_at[p]).find(|&l| l != NONE);
            let target = target.expect("a leaf precedes every right-glued run");
            tree.nodes[target].hi = tree.nodes[target].hi.max(hi);
        }
    }
    for v in tree.preorder().into_iter().rev() {
        let kids = &tree.nodes[v].children;
        if let (Some(&first), Some(&last)) = (kids.first(), kids.last()) {
            let (lo, hi) = (tree.nodes[first].lo, tree.nodes[last].hi);
            tree.nodes[v].lo = lo;
            tree.nodes[v].hi = hi;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::{average_linkage, orient};

    /// Four cliques of six; cliques 0-1 and 2-3 are weakly tied.
    pub fn planted() -> Graph {
        let mut e = Vec::new();
        for c in 0..4 {
            for i in 0..6 {
                for j in i + 1..6 {
                    e.push((c * 6 + i, c * 6 + j, 1.0));
                }
            }
        }
        e.push((5, 6, 0.3));
        e.push((4, 7, 0.3));
        e.push((17, 18, 0.3));
        e.push((11, 12, 0.01));
        Graph::new(24, e)
    }

    #[test]
    fn recovers_the_planted_cliques_within_the_bounds() {
        let g = planted();
        let mut t = average_linkage(&g);
        orient(&mut t, &g);
        let p = Params {
            lambda: 0.001,
            min_leaf: 3,
            max_children: 4,
            ..Params::default()
        };
        let out = collapse(&t, &g, &p).unwrap();
        out.tree.validate(&p).unwrap();
        let exact = out.tree.evaluate(&g, &p);
        assert!(
            (exact.total - out.dp_total).abs() < 1e-12,
            "{exact:?} vs {}",
            out.dp_total
        );
        let mut sizes: Vec<usize> = out
            .tree
            .leaves()
            .iter()
            .map(|&l| out.tree.nodes[l].len())
            .collect();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![6, 6, 6, 6], "{:?}", out.tree);
    }

    #[test]
    fn a_heavy_navigation_weight_flattens_the_tree() {
        let g = planted();
        let mut t = average_linkage(&g);
        orient(&mut t, &g);
        let p = Params {
            lambda: 10.0,
            min_leaf: 3,
            ..Params::default()
        };
        let out = collapse(&t, &g, &p).unwrap();
        out.tree.validate(&p).unwrap();
        assert!(out.tree.nodes[0]
            .children
            .iter()
            .all(|&c| out.tree.nodes[c].is_leaf()));
    }
}
