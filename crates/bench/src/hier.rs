//! Single-placement class hierarchy shared by the catalogue scores.
//!
//! Node 0 is a virtual root. Class nodes are numbered in shelf pre-order
//! (children in scheme file order), so every parent index is smaller than
//! its children's and a reverse scan visits children before parents. Each
//! item sits at its primary class node, or at the root when unplaced.
use shelfmark_core::scheme::SchemeDoc;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Hier {
    pub parent: Vec<usize>,
    pub depth: Vec<usize>,
    pub code: Vec<String>,
    pub children: Vec<Vec<usize>>,
    /// Primary class node of every item (0 when unplaced).
    pub item_class: Vec<usize>,
    /// Items placed directly at each node, in item order.
    pub direct: Vec<Vec<usize>>,
    /// Items at or below each node.
    pub subtree: Vec<usize>,
}

impl Hier {
    /// Induced hierarchy of the used classes and their ancestors.
    pub fn build(scheme: &SchemeDoc, primary: &[Option<String>]) -> Result<Self, String> {
        let mut index = BTreeMap::new();
        for (i, class) in scheme.classes.iter().enumerate() {
            if index.insert(class.code.as_str(), i).is_some() {
                return Err(format!("duplicate scheme code {}", class.code));
            }
        }
        let mut used = BTreeSet::new();
        for code in primary.iter().flatten() {
            let mut current = Some(
                *index
                    .get(code.as_str())
                    .ok_or(format!("unknown code {code}"))?,
            );
            while let Some(i) = current {
                if !used.insert(i) {
                    break;
                }
                current = match scheme.classes[i].parent.as_deref() {
                    Some(p) => Some(*index.get(p).ok_or(format!("missing parent {p}"))?),
                    None => None,
                };
            }
        }
        // Raw nodes: root, then used classes in scheme file order.
        let used: Vec<usize> = used.into_iter().collect();
        let raw_of: BTreeMap<usize, usize> =
            used.iter().enumerate().map(|(r, &i)| (i, r + 1)).collect();
        let mut parent = vec![0];
        let mut code = vec![String::new()];
        for &i in &used {
            let p = scheme.classes[i]
                .parent
                .as_deref()
                .map_or(0, |p| raw_of[&index[p]]);
            parent.push(p);
            code.push(scheme.classes[i].code.clone());
        }
        let item_class = primary
            .iter()
            .map(|c| c.as_ref().map_or(0, |c| raw_of[&index[c.as_str()]]))
            .collect();
        Ok(Self::from_parents(&parent, code, item_class))
    }

    /// Hierarchy from a parent array (`parent[0]` is ignored; node 0 is the
    /// root). Siblings keep their index order. Nodes are renumbered in
    /// pre-order; item classes follow the renumbering.
    pub fn from_parents(parent: &[usize], code: Vec<String>, item_class: Vec<usize>) -> Self {
        let n = parent.len();
        let mut kids = vec![vec![]; n];
        for (v, &p) in parent.iter().enumerate().skip(1) {
            kids[p].push(v);
        }
        let mut order = Vec::with_capacity(n);
        let mut stack = vec![0];
        while let Some(v) = stack.pop() {
            order.push(v);
            stack.extend(kids[v].iter().rev());
        }
        let mut new_of = vec![0; n];
        for (new, &old) in order.iter().enumerate() {
            new_of[old] = new;
        }
        let mut h = Self {
            parent: order.iter().map(|&old| new_of[parent[old]]).collect(),
            depth: vec![0; n],
            code: order.iter().map(|&old| code[old].clone()).collect(),
            children: order
                .iter()
                .map(|&old| kids[old].iter().map(|&c| new_of[c]).collect())
                .collect(),
            item_class: item_class.iter().map(|&c| new_of[c]).collect(),
            direct: vec![vec![]; n],
            subtree: vec![0; n],
        };
        h.parent[0] = 0;
        for v in 1..n {
            h.depth[v] = h.depth[h.parent[v]] + 1;
        }
        for (item, &c) in h.item_class.iter().enumerate() {
            h.direct[c].push(item);
        }
        for v in (0..n).rev() {
            h.subtree[v] += h.direct[v].len();
            if v > 0 {
                let p = h.parent[v];
                h.subtree[p] += h.subtree[v];
            }
        }
        h
    }

    pub fn len(&self) -> usize {
        self.parent.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parent.len() <= 1
    }

    pub fn items(&self) -> usize {
        self.item_class.len()
    }

    pub fn placed(&self, item: usize) -> bool {
        self.item_class[item] != 0
    }

    pub fn lca(&self, mut a: usize, mut b: usize) -> usize {
        while self.depth[a] > self.depth[b] {
            a = self.parent[a];
        }
        while self.depth[b] > self.depth[a] {
            b = self.parent[b];
        }
        while a != b {
            a = self.parent[a];
            b = self.parent[b];
        }
        a
    }

    /// Ancestor of `node` at depth `d`, or `node` itself when it is shallower.
    pub fn ancestor_at(&self, mut node: usize, d: usize) -> usize {
        while self.depth[node] > d {
            node = self.parent[node];
        }
        node
    }

    /// The child of `ancestor` on the path to `node` (`node` must lie strictly below).
    pub fn child_toward(&self, ancestor: usize, node: usize) -> usize {
        self.ancestor_at(node, self.depth[ancestor] + 1)
    }

    /// Shelf position of every placed item: pre-order over classes, a
    /// class's own items (in item order) before its subclasses. Unplaced
    /// items get `None`.
    pub fn shelf_positions(&self) -> Vec<Option<usize>> {
        let mut pos = vec![None; self.items()];
        let mut next = 0;
        // Pre-order numbering means node order is shelf order.
        for v in 1..self.len() {
            for &item in &self.direct[v] {
                pos[item] = Some(next);
                next += 1;
            }
        }
        pos
    }

    /// Item labels at depth `d` (ancestor class), `None` when unplaced or
    /// placed above depth `d`.
    pub fn labels_at(&self, d: usize) -> Vec<Option<usize>> {
        self.item_class
            .iter()
            .map(|&c| (c != 0 && self.depth[c] >= d).then(|| self.ancestor_at(c, d)))
            .collect()
    }

    pub fn max_depth(&self) -> usize {
        self.depth.iter().copied().max().unwrap_or(0)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// root ─ A ─ A1 ─ (items 0, 1)
    ///      │   └ A2 ─ (item 2)
    ///      └ B ─ (item 3); item 4 unplaced.
    pub(crate) fn toy() -> Hier {
        let parent = [0, 0, 1, 1, 0];
        let code = ["", "A", "A1", "A2", "B"].map(String::from).to_vec();
        Hier::from_parents(&parent, code, vec![2, 2, 3, 4, 0])
    }

    #[test]
    fn toy_shape_counts_and_lca() {
        let h = toy();
        assert_eq!(h.subtree, [5, 3, 2, 1, 1]);
        assert_eq!(h.depth, [0, 1, 2, 2, 1]);
        assert_eq!(h.lca(2, 3), 1);
        assert_eq!(h.lca(2, 4), 0);
        assert_eq!(h.child_toward(0, 3), 1);
        assert_eq!(
            h.shelf_positions(),
            [Some(0), Some(1), Some(2), Some(3), None]
        );
        assert_eq!(h.labels_at(1), [Some(1), Some(1), Some(1), Some(4), None]);
        assert_eq!(h.labels_at(2), [Some(2), Some(2), Some(3), None, None]);
    }

    #[test]
    fn from_parents_renumbers_in_preorder() {
        // B listed before its parent A: renumbering still puts A first.
        let h = Hier::from_parents(
            &[0, 2, 0],
            ["", "B", "A"].map(String::from).to_vec(),
            vec![1],
        );
        assert_eq!(h.code, ["", "A", "B"]);
        assert_eq!(h.parent, [0, 0, 1]);
        assert_eq!(h.item_class, [2]);
    }
}
