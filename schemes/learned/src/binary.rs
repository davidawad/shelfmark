//! Binary hierarchy over the items: sparse average linkage, Dasgupta
//! rotation local search, and the child orientation that fixes the shelf
//! order. Items are nodes `0..n`; merges are `n..2n-1`.

use crate::graph::Graph;
use crate::rng::SplitMix64;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};

pub const NONE: usize = usize::MAX;

#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub n: usize,
    /// Children of merge node `v` at `kids[v - n]`, in shelf order once oriented.
    pub kids: Vec<[usize; 2]>,
    pub parent: Vec<usize>,
    pub size: Vec<usize>,
    pub root: usize,
}

impl Binary {
    pub fn len(&self) -> usize {
        self.parent.len()
    }
    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }
    pub fn children(&self, v: usize) -> Option<[usize; 2]> {
        (v >= self.n).then(|| self.kids[v - self.n])
    }
    fn depths(&self) -> Vec<usize> {
        let mut depth = vec![0; self.len()];
        for v in self.preorder() {
            if let Some(k) = self.children(v) {
                depth[k[0]] = depth[v] + 1;
                depth[k[1]] = depth[v] + 1;
            }
        }
        depth
    }
    /// Nodes with every parent before its children.
    pub fn preorder(&self) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.len());
        let mut stack = vec![self.root];
        while let Some(v) = stack.pop() {
            out.push(v);
            if let Some([a, b]) = self.children(v) {
                stack.push(b);
                stack.push(a);
            }
        }
        out
    }
    /// Items in shelf order (left-to-right leaves).
    pub fn leaf_order(&self) -> Vec<usize> {
        self.preorder()
            .into_iter()
            .filter(|&v| v < self.n)
            .collect()
    }
    fn lca(&self, depth: &[usize], mut a: usize, mut b: usize) -> usize {
        while depth[a] > depth[b] {
            a = self.parent[a];
        }
        while depth[b] > depth[a] {
            b = self.parent[b];
        }
        while a != b {
            a = self.parent[a];
            b = self.parent[b];
        }
        a
    }
    /// The child of `anc` on the path up from `v` (`v` must be below `anc`).
    fn toward(&self, mut v: usize, anc: usize) -> usize {
        while self.parent[v] != anc {
            v = self.parent[v];
        }
        v
    }
    /// For every edge, the merge node where its endpoints first meet.
    pub fn edge_lcas(&self, g: &Graph) -> Vec<usize> {
        let depth = self.depths();
        g.edges
            .iter()
            .map(|&(a, b, _)| self.lca(&depth, a, b))
            .collect()
    }
    /// Unnormalized Dasgupta cost `sum w_ij |leaves(lca(i, j))|`.
    pub fn dasgupta(&self, g: &Graph) -> f64 {
        self.edge_lcas(g)
            .iter()
            .zip(&g.edges)
            .map(|(&v, e)| e.2 * self.size[v] as f64)
            .sum()
    }
}

#[derive(Clone, Copy)]
struct Candidate(f64, usize, usize);
impl PartialEq for Candidate {
    fn eq(&self, o: &Self) -> bool {
        self.cmp(o) == Ordering::Equal
    }
}
impl Eq for Candidate {}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Candidate {
    /// Highest similarity first, then the lowest cluster ids.
    fn cmp(&self, o: &Self) -> Ordering {
        self.0
            .total_cmp(&o.0)
            .then_with(|| o.1.cmp(&self.1))
            .then_with(|| o.2.cmp(&self.2))
    }
}

/// Average linkage on the sparse graph: cluster similarity is the summed
/// edge weight between two clusters over the product of their sizes, with
/// missing pairs counted as 0 (Moseley-Wang's average linkage). Ties go to
/// the lowest cluster ids. Clusters with no edge between them (separate
/// components) are joined last, two smallest first, which keeps the top of
/// the tree balanced without changing its Dasgupta cost.
pub fn average_linkage(g: &Graph) -> Binary {
    let n = g.n;
    let total = (2 * n).saturating_sub(1);
    let mut size = vec![1usize; total];
    let mut parent = vec![NONE; total];
    let mut kids = Vec::with_capacity(n.saturating_sub(1));
    let mut alive = vec![false; total];
    alive[..n].iter_mut().for_each(|a| *a = true);
    let mut nbr: Vec<BTreeMap<usize, f64>> = vec![BTreeMap::new(); total];
    for &(a, b, w) in &g.edges {
        *nbr[a].entry(b).or_insert(0.0) += w;
        *nbr[b].entry(a).or_insert(0.0) += w;
    }
    let mut heap = BinaryHeap::new();
    for (a, m) in nbr.iter().enumerate().take(n) {
        for (&b, &w) in m {
            if a < b {
                heap.push(Candidate(w, a, b));
            }
        }
    }
    let mut next = n;
    let mut join = |a: usize, b: usize, next: &mut usize, size: &mut Vec<usize>| {
        let c = *next;
        *next += 1;
        size[c] = size[a] + size[b];
        parent[a] = c;
        parent[b] = c;
        kids.push([a, b]);
        c
    };
    while let Some(Candidate(_, a, b)) = heap.pop() {
        if !alive[a] || !alive[b] {
            continue;
        }
        let c = join(a, b, &mut next, &mut size);
        let mut ma = std::mem::take(&mut nbr[a]);
        let mut mb = std::mem::take(&mut nbr[b]);
        ma.remove(&b);
        mb.remove(&a);
        if ma.len() < mb.len() {
            std::mem::swap(&mut ma, &mut mb);
        }
        for (x, w) in mb {
            *ma.entry(x).or_insert(0.0) += w;
        }
        for (&x, &w) in &ma {
            nbr[x].remove(&a);
            nbr[x].remove(&b);
            nbr[x].insert(c, w);
            heap.push(Candidate(
                w / (size[c] * size[x]) as f64,
                x.min(c),
                x.max(c),
            ));
        }
        nbr[c] = ma;
        alive[a] = false;
        alive[b] = false;
        alive[c] = true;
    }
    let mut rest: BinaryHeap<std::cmp::Reverse<(usize, usize)>> = (0..next)
        .filter(|&v| alive[v])
        .map(|v| std::cmp::Reverse((size[v], v)))
        .collect();
    while rest.len() > 1 {
        let std::cmp::Reverse((_, a)) = rest.pop().unwrap();
        let std::cmp::Reverse((_, b)) = rest.pop().unwrap();
        let c = join(a.min(b), a.max(b), &mut next, &mut size);
        rest.push(std::cmp::Reverse((size[c], c)));
    }
    Binary {
        n,
        kids,
        parent,
        size,
        root: next.saturating_sub(1),
    }
}

/// Rotation local search on Dasgupta cost: at merge node `x = (A, C)` with
/// `A = (A1, A2)`, try `((A1, C), A2)` and `((A2, C), A1)`. Only the edges
/// meeting at `x` or `A` change cost, so each try is exact and local.
/// Nodes are visited in a seeded shuffled order; passes repeat until none
/// improves or `max_passes` is reached. Returns the number of rotations.
pub fn rotate(t: &mut Binary, g: &Graph, seed: u64, max_passes: usize) -> usize {
    let n = t.n;
    let mut lists: Vec<Vec<usize>> = vec![Vec::new(); t.len()];
    for (e, v) in t.edge_lcas(g).into_iter().enumerate() {
        lists[v].push(e);
    }
    let weight = |list: &[usize]| list.iter().map(|&e| g.edges[e].2).sum::<f64>();
    let mut rng = SplitMix64::new(seed);
    let mut applied = 0;
    for _ in 0..max_passes {
        let mut order: Vec<usize> = (n..t.len()).collect();
        rng.shuffle(&mut order);
        let mut improved = 0;
        for &x in &order {
            for side in 0..2 {
                let [p, q] = t.kids[x - n];
                let (a, c) = if side == 0 { (p, q) } else { (q, p) };
                let Some([a1, a2]) = t.children(a) else {
                    continue;
                };
                let (mut w1c, mut w2c) = (0.0, 0.0);
                let mut sides = Vec::with_capacity(lists[x].len());
                for &e in &lists[x] {
                    let (i, j, w) = g.edges[e];
                    let end = if t.toward(i, x) == a { i } else { j };
                    let first = t.toward(end, a) == a1;
                    if first {
                        w1c += w;
                    } else {
                        w2c += w;
                    }
                    sides.push(first);
                }
                let w12 = weight(&lists[a]);
                let (nx, s1, s2, sc) = (
                    t.size[x] as f64,
                    t.size[a1] as f64,
                    t.size[a2] as f64,
                    t.size[c] as f64,
                );
                let old = nx * (w1c + w2c) + (s1 + s2) * w12;
                let alt1 = nx * (w12 + w2c) + (s1 + sc) * w1c;
                let alt2 = nx * (w12 + w1c) + (s2 + sc) * w2c;
                let (best, keep, out) = if alt1 <= alt2 {
                    (alt1, a1, a2)
                } else {
                    (alt2, a2, a1)
                };
                if best >= old - 1e-12 * old.abs().max(1.0) {
                    continue;
                }
                // A becomes (keep, C); x becomes (A, out) in A's old slot.
                t.kids[a - n] = [keep, c];
                t.kids[x - n] = if side == 0 { [a, out] } else { [out, a] };
                t.parent[c] = a;
                t.parent[out] = x;
                t.size[a] = t.size[keep] + t.size[c];
                let keep_first = keep == a1;
                let (mut to_a, mut stay) = (Vec::new(), Vec::new());
                for (&e, &first) in lists[x].iter().zip(&sides) {
                    if first == keep_first {
                        to_a.push(e);
                    } else {
                        stay.push(e);
                    }
                }
                stay.append(&mut lists[a]);
                lists[x] = stay;
                lists[a] = to_a;
                improved += 1;
            }
        }
        applied += improved;
        if improved == 0 {
            break;
        }
    }
    applied
}

/// Orients every merge so the child more strongly tied to the items just
/// shelved to its left comes first (ties: larger child, then lower id).
/// The resulting leaf order is the shelf order every class must respect.
pub fn orient(t: &mut Binary, g: &Graph) {
    let mut pos = vec![NONE; t.n];
    let mut next = 0usize;
    let mut stack = vec![t.root];
    let leaves = |t: &Binary, v: usize| -> Vec<usize> {
        let mut out = Vec::new();
        let mut s = vec![v];
        while let Some(u) = s.pop() {
            match t.children(u) {
                Some([a, b]) => {
                    s.push(a);
                    s.push(b);
                }
                None => out.push(u),
            }
        }
        out
    };
    while let Some(v) = stack.pop() {
        let Some([a, b]) = t.children(v) else {
            pos[v] = next;
            next += 1;
            continue;
        };
        let window = next.saturating_sub(t.size[v]);
        let tie = |child: usize| -> f64 {
            leaves(t, child)
                .into_iter()
                .flat_map(|i| g.neighbors(i).iter())
                .filter(|&&(j, _)| pos[j] != NONE && pos[j] >= window)
                .map(|&(_, w)| w)
                .sum()
        };
        let (ta, tb) = if next == 0 {
            (0.0, 0.0)
        } else {
            (tie(a), tie(b))
        };
        let a_first = match ta.total_cmp(&tb) {
            Ordering::Greater => true,
            Ordering::Less => false,
            Ordering::Equal => {
                (t.size[a], std::cmp::Reverse(a)) >= (t.size[b], std::cmp::Reverse(b))
            }
        };
        let pair = if a_first { [a, b] } else { [b, a] };
        t.kids[v - t.n] = pair;
        stack.push(pair[1]);
        stack.push(pair[0]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_cliques() -> Graph {
        let mut e = Vec::new();
        for (a, b) in [(0, 1), (0, 2), (1, 2), (3, 4), (3, 5), (4, 5)] {
            e.push((a, b, 1.0));
        }
        e.push((2, 3, 0.1));
        Graph::new(6, e)
    }

    #[test]
    fn average_linkage_separates_cliques_and_is_deterministic() {
        let g = two_cliques();
        let t = average_linkage(&g);
        assert_eq!(t, average_linkage(&g));
        let [a, b] = t.kids[t.root - t.n];
        assert_eq!((t.size[a], t.size[b]), (3, 3));
        // Each clique pair meets inside its triple: 6 * 3 * ~1 + 0.1 * 6.
        let cost = t.dasgupta(&g);
        assert!(cost <= 6.0 * 3.0 + 0.6 + 1e-9, "{cost}");
    }

    #[test]
    fn components_are_joined_two_smallest_first() {
        let g = Graph::new(4, vec![(0, 1, 1.0)]);
        let t = average_linkage(&g);
        assert_eq!(t.len(), 7);
        assert_eq!(t.size[t.root], 4);
    }

    #[test]
    fn rotation_never_raises_cost_and_fixes_a_bad_merge() {
        let g = two_cliques();
        // ((0, 3), ...) is a bad tree: build by hand.
        let mut t = Binary {
            n: 6,
            kids: vec![[0, 3], [6, 1], [7, 4], [8, 2], [9, 5]],
            parent: vec![],
            size: vec![1; 11],
            root: 10,
        };
        t.parent = vec![NONE; 11];
        for (i, k) in t.kids.clone().iter().enumerate() {
            t.parent[k[0]] = i + 6;
            t.parent[k[1]] = i + 6;
            t.size[i + 6] = t.size[k[0]] + t.size[k[1]];
        }
        let before = t.dasgupta(&g);
        let moves = rotate(&mut t, &g, 7, 50);
        let after = t.dasgupta(&g);
        assert!(moves > 0 && after < before, "{before} -> {after}");
        assert_eq!(t.size[t.root], 6);
    }

    #[test]
    fn orientation_keeps_tied_items_adjacent() {
        let g = two_cliques();
        let mut t = average_linkage(&g);
        orient(&mut t, &g);
        let order = t.leaf_order();
        let p2 = order.iter().position(|&i| i == 2).unwrap();
        let p3 = order.iter().position(|&i| i == 3).unwrap();
        assert_eq!(p2.abs_diff(p3), 1, "{order:?}");
    }
}
