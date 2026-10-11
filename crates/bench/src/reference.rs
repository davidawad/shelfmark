//! Average-linkage reference tree.
use crate::{SimilarityGraph, Tree};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

#[derive(Clone, Copy, Debug)]
struct Candidate(f64, usize, usize);
impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}
impl Eq for Candidate {}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .total_cmp(&other.0)
            .then_with(|| other.1.cmp(&self.1))
            .then_with(|| other.2.cmp(&self.2))
    }
}

/// Deterministic average-linkage HAC. Missing graph edges have similarity 0,
/// so a merge only updates the clusters adjacent to one of its two parts;
/// every other pair stays at 0. Ties go to the lowest cluster ids, and when
/// no positive similarity remains the two lowest active ids merge.
pub fn average_linkage_reference(graph: &SimilarityGraph) -> Tree {
    let n = graph.n;
    let mut tree = Tree::new();
    tree.item_leaves = vec![vec![]; n];
    if n == 0 {
        return tree;
    }
    let mut cluster_nodes: Vec<usize> = (0..n)
        .map(|i| {
            let leaf = tree.add_node(0, None, Some(i));
            tree.item_leaves[i] = vec![(leaf, 1.0)];
            leaf
        })
        .collect();
    let mut sizes = vec![1usize; n];
    let mut adjacent = vec![BTreeMap::<usize, f64>::new(); n];
    let mut candidates = BinaryHeap::new();
    for e in &graph.edges {
        let pair = (e.left.min(e.right), e.left.max(e.right));
        if e.weight != 0.0 {
            adjacent[pair.0].insert(pair.1, e.weight);
            adjacent[pair.1].insert(pair.0, e.weight);
        } else {
            adjacent[pair.0].remove(&pair.1);
            adjacent[pair.1].remove(&pair.0);
        }
        if e.weight > 0.0 {
            candidates.push(Candidate(e.weight, pair.0, pair.1));
        }
    }
    let mut active: BTreeSet<usize> = (0..n).collect();
    let mut next_id = n;
    while active.len() > 1 {
        let (a, b) = loop {
            match candidates.pop() {
                Some(Candidate(_, a, b)) if active.contains(&a) && active.contains(&b) => {
                    break (a, b);
                }
                Some(_) => continue,
                None => {
                    let mut ids = active.iter().copied();
                    break (ids.next().unwrap(), ids.next().unwrap());
                }
            }
        };
        let node = tree.add_node(0, None, None); // reparent merged subtree under its merge node
        for child in [cluster_nodes[a], cluster_nodes[b]] {
            tree.nodes[0].children.retain(|&x| x != child);
            tree.nodes[child].parent = Some(node);
            tree.nodes[node].children.push(child);
        }
        let combined_size = sizes[a] + sizes[b];
        active.remove(&a);
        active.remove(&b);
        let left_map = std::mem::take(&mut adjacent[a]);
        let right_map = std::mem::take(&mut adjacent[b]);
        let others: BTreeSet<usize> = left_map
            .keys()
            .chain(right_map.keys())
            .copied()
            .filter(|o| active.contains(o))
            .collect();
        let mut merged = BTreeMap::new();
        for other in others {
            let left = left_map.get(&other).copied().unwrap_or(0.0);
            let right = right_map.get(&other).copied().unwrap_or(0.0);
            let similarity =
                (sizes[a] as f64 * left + sizes[b] as f64 * right) / combined_size as f64;
            adjacent[other].remove(&a);
            adjacent[other].remove(&b);
            if similarity != 0.0 {
                adjacent[other].insert(next_id, similarity);
                merged.insert(other, similarity);
            }
            if similarity > 0.0 {
                candidates.push(Candidate(
                    similarity,
                    other.min(next_id),
                    other.max(next_id),
                ));
            }
        }
        adjacent.push(merged);
        active.insert(next_id);
        cluster_nodes.push(node);
        sizes.push(combined_size);
        next_id += 1;
    }
    tree
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{score_tree, Edge};

    #[test]
    fn average_linkage_merges_the_highest_similarity_pair_first() {
        let graph = SimilarityGraph {
            n: 3,
            k: 1,
            tau: 0.0,
            edges: vec![
                Edge {
                    left: 0,
                    right: 1,
                    weight: 4.0,
                },
                Edge {
                    left: 1,
                    right: 2,
                    weight: 1.0,
                },
            ],
        };
        let tree = average_linkage_reference(&graph);
        let score = score_tree(&tree, &graph);
        assert_eq!(score.cost, 4.0 * 2.0 + 1.0 * 3.0);
        assert_eq!(tree, average_linkage_reference(&graph));
    }

    #[test]
    fn disconnected_components_join_at_zero_similarity() {
        let graph = SimilarityGraph {
            n: 4,
            k: 1,
            tau: 0.0,
            edges: vec![
                Edge {
                    left: 0,
                    right: 1,
                    weight: 2.0,
                },
                Edge {
                    left: 2,
                    right: 3,
                    weight: 1.0,
                },
            ],
        };
        let tree = average_linkage_reference(&graph);
        // Pairs meet in their own 2-leaf cluster; the root joins both.
        assert_eq!(score_tree(&tree, &graph).cost, 2.0 * 2.0 + 2.0);
        assert_eq!(tree.nodes[0].children.len(), 1);
    }
}
