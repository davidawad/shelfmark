//! Order-preserving local search on the collapsed class tree, on the exact
//! objective. Moves never reorder the shelf:
//!
//! - shift: the last item of a leaf class moves into the next leaf class on
//!   the shelf, or the first item of a leaf into the previous one;
//! - hand-over: the last child class of a class becomes the first child of
//!   its next sibling (or the reverse);
//! - merge: two adjacent sibling leaf classes become one;
//! - flatten: a class is replaced by its children;
//! - group: two or three adjacent sibling classes get a new common parent.
//!
//! Every move keeps the branching, depth and leaf-size bounds. Shifts are
//! scored incrementally; the rest re-evaluate the whole tree.

use crate::classtree::{leaf_nav, ClassTree, Node, Objective, Params, NONE};
use crate::graph::Graph;
use crate::rng::SplitMix64;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct SearchStats {
    pub rounds: usize,
    pub shifts: usize,
    pub structure_moves: usize,
}

pub fn improve(
    tree: &mut ClassTree,
    g: &Graph,
    p: &Params,
    seed: u64,
    max_rounds: usize,
) -> SearchStats {
    let mut rng = SplitMix64::new(seed);
    let mut stats = SearchStats::default();
    for _ in 0..max_rounds {
        stats.rounds += 1;
        let a = shift_pass(tree, g, p);
        let b = structure_pass(tree, g, p, &mut rng);
        stats.shifts += a;
        stats.structure_moves += b;
        if a + b == 0 {
            break;
        }
    }
    *tree = tree.compact();
    stats
}

/// State for incremental item shifts: weight meeting at each node.
struct Shifter<'a> {
    g: &'a Graph,
    p: &'a Params,
    depth: Vec<usize>,
    meet: Vec<f64>,
    leaf: Vec<usize>,
}

impl Shifter<'_> {
    fn branch(&self, t: &ClassTree, v: usize) -> f64 {
        self.p.branch_cost + (t.nodes[v].children.len() as f64).log2()
    }
    /// Nodes from `c` up to, not including, `stop`.
    fn path(t: &ClassTree, mut c: usize, stop: usize) -> Vec<usize> {
        let mut out = Vec::new();
        while c != stop {
            out.push(c);
            c = t.nodes[c].parent;
        }
        out
    }
    /// Objective change of moving item `i` from leaf `c` to leaf `d`.
    fn delta(&self, t: &ClassTree, i: usize, c: usize, d: usize) -> f64 {
        let top = t.lca(&self.depth, c, d);
        let (up, down) = (Self::path(t, c, top), Self::path(t, d, top));
        let size = |v: usize| t.nodes[v].len() as f64;
        let size_after =
            |v: usize| size(v) - f64::from(up.contains(&v)) + f64::from(down.contains(&v));
        let mut own: Vec<(usize, f64)> = Vec::new();
        let mut cost = 0.0;
        for &(j, w) in self.g.neighbors(i) {
            let lj = self.leaf[j];
            let (old, new) = (t.lca(&self.depth, c, lj), t.lca(&self.depth, d, lj));
            cost += w * (size_after(new) - size(old));
            own.push((old, w));
        }
        let rest =
            |v: usize| self.meet[v] - own.iter().filter(|x| x.0 == v).map(|x| x.1).sum::<f64>();
        cost += down.iter().map(|&v| rest(v)).sum::<f64>();
        cost -= up.iter().map(|&v| rest(v)).sum::<f64>();
        let a = self.p.branch_cost;
        let (nc, nd) = (t.nodes[c].len(), t.nodes[d].len());
        let mut nav = leaf_nav(nc - 1, a) - leaf_nav(nc, a) + leaf_nav(nd + 1, a) - leaf_nav(nd, a);
        nav += down[1..].iter().map(|&v| self.branch(t, v)).sum::<f64>();
        nav -= up[1..].iter().map(|&v| self.branch(t, v)).sum::<f64>();
        let n = t.n() as f64;
        cost / (n * self.g.total) + self.p.lambda * nav / n
    }
    fn apply(&mut self, t: &mut ClassTree, i: usize, c: usize, d: usize) {
        let top = t.lca(&self.depth, c, d);
        let forward = t.nodes[c].hi == t.nodes[d].lo;
        for v in Self::path(t, c, top) {
            if forward {
                t.nodes[v].hi -= 1;
            } else {
                t.nodes[v].lo += 1;
            }
        }
        for v in Self::path(t, d, top) {
            if forward {
                t.nodes[v].lo -= 1;
            } else {
                t.nodes[v].hi += 1;
            }
        }
        for &(j, w) in self.g.neighbors(i) {
            let lj = self.leaf[j];
            self.meet[t.lca(&self.depth, c, lj)] -= w;
        }
        self.leaf[i] = d;
        for &(j, w) in self.g.neighbors(i) {
            let lj = self.leaf[j];
            self.meet[t.lca(&self.depth, d, lj)] += w;
        }
    }
}

fn shift_pass(t: &mut ClassTree, g: &Graph, p: &Params) -> usize {
    let depth = t.depths();
    let leaf = t.item_leaf();
    let mut meet = vec![0.0; t.nodes.len()];
    for &(a, b, w) in &g.edges {
        meet[t.lca(&depth, leaf[a], leaf[b])] += w;
    }
    let mut s = Shifter {
        g,
        p,
        depth,
        meet,
        leaf,
    };
    let leaves = t.leaves();
    let mut moved = 0;
    for _sweep in 0..100 {
        let mut this_sweep = 0;
        for pair in leaves.windows(2) {
            let (c, d) = (pair[0], pair[1]);
            loop {
                let fwd = (t.nodes[c].len() > p.min_leaf).then(|| {
                    let i = t.order[t.nodes[c].hi - 1];
                    (s.delta(t, i, c, d), i, c, d)
                });
                let back = (t.nodes[d].len() > p.min_leaf).then(|| {
                    let i = t.order[t.nodes[d].lo];
                    (s.delta(t, i, d, c), i, d, c)
                });
                let best = [fwd, back]
                    .into_iter()
                    .flatten()
                    .min_by(|x, y| x.0.total_cmp(&y.0));
                match best {
                    Some((delta, i, from, to)) if delta < -1e-15 => {
                        s.apply(t, i, from, to);
                        this_sweep += 1;
                    }
                    _ => break,
                }
            }
        }
        moved += this_sweep;
        if this_sweep == 0 {
            break;
        }
    }
    moved
}

#[derive(Debug, Clone, Copy)]
enum Move {
    HandForward(usize, usize),
    HandBack(usize, usize),
    Merge(usize, usize),
    Flatten(usize),
    Group(usize, usize, usize),
}

fn candidates(t: &ClassTree) -> Vec<Move> {
    let mut out = Vec::new();
    for v in t.preorder() {
        let kids = &t.nodes[v].children;
        for pair in kids.windows(2) {
            let (x, y) = (pair[0], pair[1]);
            match (t.nodes[x].is_leaf(), t.nodes[y].is_leaf()) {
                (false, false) => {
                    out.push(Move::HandForward(x, y));
                    out.push(Move::HandBack(x, y));
                }
                (true, true) => out.push(Move::Merge(x, y)),
                _ => {}
            }
        }
        for &x in kids {
            if !t.nodes[x].is_leaf() {
                out.push(Move::Flatten(x));
            }
        }
        for k in [2, 3] {
            for i in 0..kids.len().saturating_sub(k - 1) {
                if kids.len() > k {
                    out.push(Move::Group(v, kids[i], k));
                }
            }
        }
    }
    out
}

fn height(t: &ClassTree, v: usize) -> usize {
    t.nodes[v]
        .children
        .iter()
        .map(|&c| 1 + height(t, c))
        .max()
        .unwrap_or(0)
}

/// Applies `m` if it is still valid for `t`; false otherwise.
fn apply(t: &mut ClassTree, m: Move, p: &Params) -> bool {
    let live = |t: &ClassTree, v: usize| v == 0 || t.nodes[v].parent != NONE;
    match m {
        Move::HandForward(x, y) | Move::HandBack(x, y) => {
            let par = t.nodes[x].parent;
            if !live(t, x) || par != t.nodes[y].parent || t.nodes[x].hi != t.nodes[y].lo {
                return false;
            }
            let (from, to) = if matches!(m, Move::HandForward(..)) {
                (x, y)
            } else {
                (y, x)
            };
            let (nf, nt) = (t.nodes[from].children.len(), t.nodes[to].children.len());
            if nf <= p.min_children.max(2) || nt == 0 || nt >= p.max_children {
                return false;
            }
            if from == x {
                let c = t.nodes[x].children.pop().unwrap();
                t.nodes[y].children.insert(0, c);
                t.nodes[c].parent = y;
                t.nodes[x].hi = t.nodes[c].lo;
                t.nodes[y].lo = t.nodes[c].lo;
            } else {
                let c = t.nodes[y].children.remove(0);
                t.nodes[x].children.push(c);
                t.nodes[c].parent = x;
                t.nodes[x].hi = t.nodes[c].hi;
                t.nodes[y].lo = t.nodes[c].hi;
            }
            true
        }
        Move::Merge(x, y) => {
            let par = t.nodes[x].parent;
            if !live(t, x)
                || !live(t, y)
                || par != t.nodes[y].parent
                || t.nodes[x].hi != t.nodes[y].lo
            {
                return false;
            }
            if !t.nodes[x].is_leaf() || !t.nodes[y].is_leaf() {
                return false;
            }
            if par == 0 && t.nodes[0].children.len() <= 2 {
                return false;
            }
            t.nodes[x].hi = t.nodes[y].hi;
            t.nodes[par].children.retain(|&c| c != y);
            t.nodes[y].parent = NONE;
            if t.nodes[par].children.len() == 1 {
                t.nodes[par].children.clear();
                t.nodes[x].parent = NONE;
            }
            true
        }
        Move::Flatten(x) => {
            if !live(t, x) || t.nodes[x].is_leaf() {
                return false;
            }
            let par = t.nodes[x].parent;
            let kids = t.nodes[x].children.clone();
            if t.nodes[par].children.len() - 1 + kids.len() > p.max_children {
                return false;
            }
            let at = t.nodes[par].children.iter().position(|&c| c == x).unwrap();
            t.nodes[par].children.splice(at..=at, kids.iter().copied());
            for c in kids {
                t.nodes[c].parent = par;
            }
            t.nodes[x].parent = NONE;
            t.nodes[x].children.clear();
            true
        }
        Move::Group(v, first, k) => {
            if !live(t, v) || !live(t, first) || t.nodes[first].parent != v {
                return false;
            }
            let kids = &t.nodes[v].children;
            let at = kids.iter().position(|&c| c == first).unwrap();
            if kids.len() <= k || at + k > kids.len() {
                return false;
            }
            let run: Vec<usize> = kids[at..at + k].to_vec();
            let below = run.iter().map(|&c| height(t, c)).max().unwrap_or(0);
            if t.depth(v) + 2 + below > p.max_depth {
                return false;
            }
            let id = t.nodes.len();
            t.nodes.push(Node {
                parent: v,
                children: run.clone(),
                lo: t.nodes[run[0]].lo,
                hi: t.nodes[run[k - 1]].hi,
            });
            for &c in &run {
                t.nodes[c].parent = id;
            }
            t.nodes[v].children.splice(at..at + k, [id]);
            true
        }
    }
}

fn structure_pass(t: &mut ClassTree, g: &Graph, p: &Params, rng: &mut SplitMix64) -> usize {
    let mut current: Objective = t.evaluate(g, p);
    let mut moves = candidates(t);
    rng.shuffle(&mut moves);
    let mut accepted = 0;
    for m in moves {
        let mut trial = t.clone();
        if !apply(&mut trial, m, p) {
            continue;
        }
        let o = trial.evaluate(g, p);
        if o.total < current.total - 1e-15 {
            debug_assert!(trial.validate(p).is_ok(), "{m:?}");
            *t = trial;
            current = o;
            accepted += 1;
        }
    }
    accepted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classtree::tests::two_leaves;

    #[test]
    fn incremental_shift_delta_matches_full_evaluation() {
        let mut t = two_leaves();
        let g = Graph::new(4, vec![(0, 1, 1.0), (1, 2, 3.0), (2, 3, 0.5), (0, 3, 0.2)]);
        let p = Params {
            lambda: 0.05,
            min_leaf: 1,
            ..Params::default()
        };
        let before = t.evaluate(&g, &p).total;
        let depth = t.depths();
        let leaf = t.item_leaf();
        let mut meet = vec![0.0; t.nodes.len()];
        for &(a, b, w) in &g.edges {
            meet[t.lca(&depth, leaf[a], leaf[b])] += w;
        }
        let mut s = Shifter {
            g: &g,
            p: &p,
            depth,
            meet,
            leaf,
        };
        // Item 1 (last of leaf 1) moves to leaf 2.
        let d = s.delta(&t, 1, 1, 2);
        s.apply(&mut t, 1, 1, 2);
        t.validate(&p).unwrap();
        let after = t.evaluate(&g, &p).total;
        assert!(
            (after - before - d).abs() < 1e-12,
            "{d} vs {}",
            after - before
        );
    }

    #[test]
    fn search_never_worsens_and_keeps_bounds() {
        let mut t = two_leaves();
        let g = Graph::new(4, vec![(0, 1, 1.0), (1, 2, 3.0), (2, 3, 0.5)]);
        let p = Params {
            lambda: 0.01,
            min_leaf: 1,
            ..Params::default()
        };
        let before = t.evaluate(&g, &p).total;
        improve(&mut t, &g, &p, 1, 10);
        t.validate(&p).unwrap();
        assert!(t.evaluate(&g, &p).total <= before);
    }
}
