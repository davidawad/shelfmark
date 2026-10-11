use serde::{Deserialize, Serialize};
use shelfmark_core::scheme::{ClassDef, SchemeDoc};
use std::collections::{BTreeMap, BTreeSet};

/// One item's primary/fractional class placements; masses should sum to one.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemPlacement {
    pub id: String,
    pub classes: Vec<(String, f64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    pub code: Option<String>,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub item: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tree {
    pub nodes: Vec<TreeNode>,
    pub item_leaves: Vec<Vec<(usize, f64)>>,
    pub class_count: usize,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl Tree {
    pub fn new() -> Self {
        Self {
            nodes: vec![TreeNode {
                code: None,
                parent: None,
                children: vec![],
                item: None,
            }],
            item_leaves: vec![],
            class_count: 0,
        }
    }
    pub fn add_node(&mut self, parent: usize, code: Option<String>, item: Option<usize>) -> usize {
        let id = self.nodes.len();
        self.nodes.push(TreeNode {
            code,
            parent: Some(parent),
            children: vec![],
            item,
        });
        self.nodes[parent].children.push(id);
        id
    }
    pub fn leaf_count(&self, node: usize) -> usize {
        self.nodes[node]
            .children
            .iter()
            .map(|&c| self.leaf_count(c))
            .sum::<usize>()
            + usize::from(self.nodes[node].item.is_some())
    }
    pub fn ancestors(&self, mut node: usize) -> Vec<usize> {
        let mut out = vec![node];
        while let Some(p) = self.nodes[node].parent {
            out.push(p);
            node = p;
        }
        out
    }
    pub fn lca(&self, a: usize, b: usize) -> usize {
        let ancestors: BTreeSet<_> = self.ancestors(a).into_iter().collect();
        self.ancestors(b)
            .into_iter()
            .find(|x| ancestors.contains(x))
            .unwrap_or(0)
    }
    pub fn item_depth(&self, leaf: usize) -> usize {
        self.ancestors(leaf).len().saturating_sub(1)
    }
}

/// Restricts the scheme to used classes and ancestors. A virtual root holds
/// forest roots and unplaced items; duplicate class codes and unknown codes
/// are rejected instead of silently changing placements.
pub fn induce_tree(scheme: &SchemeDoc, placements: &[ItemPlacement]) -> Result<Tree, String> {
    let mut defs = BTreeMap::new();
    for class in &scheme.classes {
        if defs.insert(class.code.as_str(), class).is_some() {
            return Err(format!("duplicate scheme code {}", class.code));
        }
    }
    let mut used = BTreeSet::<String>::new();
    for item in placements {
        for (code, _) in &item.classes {
            if !defs.contains_key(code.as_str()) {
                return Err(format!("unknown placement code {code}"));
            }
            let mut current = Some(code.as_str());
            while let Some(c) = current {
                used.insert(c.to_owned());
                current = defs[c].parent.as_deref();
            }
        }
    }
    for code in &used {
        if let Some(parent) = defs[code.as_str()].parent.as_deref() {
            if !defs.contains_key(parent) {
                return Err(format!("missing parent {parent} for {code}"));
            }
        }
    }
    let mut tree = Tree::new();
    let mut nodes = BTreeMap::<String, usize>::new();
    let mut ordered: Vec<_> = used.iter().collect();
    ordered.sort_by_key(|code| depth(code, &defs));
    for code in ordered {
        let parent = defs[code.as_str()]
            .parent
            .as_ref()
            .and_then(|p| nodes.get(p))
            .copied()
            .unwrap_or(0);
        let id = tree.add_node(parent, Some(code.clone()), None);
        nodes.insert(code.clone(), id);
        tree.class_count += 1;
    }
    tree.item_leaves = vec![vec![]; placements.len()];
    for (idx, item) in placements.iter().enumerate() {
        let classes = if item.classes.is_empty() {
            vec![(String::new(), 1.0)]
        } else {
            item.classes.clone()
        };
        let total = classes.iter().map(|(_, m)| *m).sum::<f64>();
        if total <= 0.0 {
            return Err(format!(
                "placement masses must sum positive for {}",
                item.id
            ));
        }
        for (code, mass) in classes {
            let parent = if code.is_empty() { 0 } else { nodes[&code] };
            let leaf = tree.add_node(parent, None, Some(idx));
            tree.item_leaves[idx].push((leaf, mass / total));
        }
    }
    Ok(tree)
}

fn depth(code: &str, defs: &BTreeMap<&str, &ClassDef>) -> usize {
    let mut n = 0;
    let mut c = Some(code);
    while let Some(x) = c {
        n += 1;
        c = defs.get(x).and_then(|d| d.parent.as_deref());
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use shelfmark_core::scheme::ClassDef;
    fn scheme() -> SchemeDoc {
        SchemeDoc {
            schema: None,
            name: "toy".into(),
            version: "1".into(),
            title: "Toy".into(),
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
                    code: "A1".into(),
                    label: "A1".into(),
                    parent: Some("A".into()),
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
    fn induces_only_used_classes_and_ancestors_and_root_residual() {
        let p = vec![
            ItemPlacement {
                id: "x".into(),
                classes: vec![("A1".into(), 1.0)],
            },
            ItemPlacement {
                id: "y".into(),
                classes: vec![],
            },
        ];
        let t = induce_tree(&scheme(), &p).unwrap();
        assert_eq!(t.class_count, 2);
        assert_eq!(t.nodes[0].children.len(), 2);
        assert_eq!(t.leaf_count(0), 2);
        assert_eq!(t.nodes[t.item_leaves[1][0].0].parent, Some(0));
    }
}
