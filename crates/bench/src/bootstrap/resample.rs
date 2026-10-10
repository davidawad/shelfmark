//! Paired item bootstrap by resampled trees and graphs, behind
//! `shelfmark-bench bootstrap`.
//!
//! A resample draws `n` items with replacement. Every tree (scheme trees and
//! the average-linkage reference) is rescored on the same draw: an item drawn
//! `c` times appears as `c` copies under its original parent, every graph edge
//! `(i, j)` becomes `c_i * c_j` edges between the copies, and class nodes with
//! no drawn item beneath are pruned, which gives the tree the scheme would
//! induce from the drawn placements. Copies of one item share no edge, as an
//! item has no edge to itself.
use super::SplitMix64;
use crate::{score_tree, Edge, SimilarityGraph, Tree, TreeScore};
use std::collections::BTreeMap;

/// `resamples` draws of `n` item indices, each sorted.
pub fn draws(n: usize, resamples: usize, seed: u64) -> Vec<Vec<usize>> {
    let mut rng = SplitMix64::new(seed);
    (0..resamples)
        .map(|_| {
            let mut picks: Vec<usize> = (0..n).map(|_| rng.below(n)).collect();
            picks.sort_unstable();
            picks
        })
        .collect()
}

/// The tree restricted to drawn items: item `t` of the result is a copy of
/// item `picks[t]`, with the same class placements and masses.
pub fn resample_tree(tree: &Tree, picks: &[usize]) -> Tree {
    let mut keep = vec![false; tree.nodes.len()];
    keep[0] = true;
    for &item in picks {
        for &(leaf, _) in &tree.item_leaves[item] {
            let mut node = tree.nodes[leaf].parent;
            while let Some(v) = node {
                if keep[v] {
                    break;
                }
                keep[v] = true;
                node = tree.nodes[v].parent;
            }
        }
    }
    let mut out = Tree::new();
    let mut new_id = vec![usize::MAX; tree.nodes.len()];
    new_id[0] = 0;
    let mut queue = vec![0usize];
    let mut next = 0;
    while next < queue.len() {
        let node = queue[next];
        next += 1;
        for &child in &tree.nodes[node].children {
            if keep[child] && tree.nodes[child].item.is_none() {
                let code = tree.nodes[child].code.clone();
                out.class_count += usize::from(code.is_some());
                new_id[child] = out.add_node(new_id[node], code, None);
                queue.push(child);
            }
        }
    }
    out.item_leaves = picks
        .iter()
        .enumerate()
        .map(|(t, &item)| {
            tree.item_leaves[item]
                .iter()
                .map(|&(leaf, mass)| {
                    let parent = new_id[tree.nodes[leaf].parent.unwrap_or(0)];
                    (out.add_node(parent, None, Some(t)), mass)
                })
                .collect()
        })
        .collect();
    out
}

/// The graph between drawn items (see the module documentation).
pub fn resample_graph(graph: &SimilarityGraph, picks: &[usize]) -> SimilarityGraph {
    let mut copies = vec![Vec::new(); graph.n];
    for (t, &item) in picks.iter().enumerate() {
        copies[item].push(t);
    }
    let mut edges = Vec::new();
    for e in &graph.edges {
        for &a in &copies[e.left] {
            for &b in &copies[e.right] {
                edges.push(Edge {
                    left: a.min(b),
                    right: a.max(b),
                    weight: e.weight,
                });
            }
        }
    }
    SimilarityGraph {
        n: picks.len(),
        k: graph.k,
        tau: graph.tau,
        edges,
    }
}

/// Every scalar the harness reports, by name: `cost_norm`, `revenue_norm`,
/// `skill` (against `ref_cost_norm`), `coverage`, `expected_tree_distance`
/// and each structural statistic.
pub fn metric_values(
    score: &TreeScore,
    ref_cost_norm: f64,
    coverage: f64,
) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    out.insert("cost_norm".into(), score.cost_norm);
    out.insert("revenue_norm".into(), score.revenue_norm);
    let skill = if ref_cost_norm < 1.0 {
        (1.0 - score.cost_norm) / (1.0 - ref_cost_norm)
    } else {
        0.0
    };
    out.insert("skill".into(), skill);
    out.insert("coverage".into(), coverage);
    out.insert(
        "expected_tree_distance".into(),
        score.expected_tree_distance,
    );
    if let Ok(serde_json::Value::Object(fields)) = serde_json::to_value(&score.structure) {
        for (name, value) in fields {
            if let Some(x) = value.as_f64() {
                out.insert(name, x);
            }
        }
    }
    out
}

/// One tree to bootstrap and which of its items carry a placement.
pub struct BootstrapRun<'a> {
    pub key: String,
    pub tree: &'a Tree,
    pub placed: Vec<bool>,
}

/// Metric samples per run, in draw order, plus the reference cost per draw.
pub struct BootstrapSamples {
    pub reference: Vec<f64>,
    pub runs: BTreeMap<String, Vec<BTreeMap<String, f64>>>,
}

/// Rescores the reference and every run on each draw, `threads` draws at a time.
/// With a negative sample (`crate::falsejoin`), its pairs between drawn copies
/// are kept the way graph edges are, and every run also gets `fj_norm` and
/// the combined objectives on the same draw.
pub fn bootstrap(
    graph: &SimilarityGraph,
    negatives: Option<&SimilarityGraph>,
    reference: &Tree,
    runs: &[BootstrapRun<'_>],
    draws: &[Vec<usize>],
    threads: usize,
) -> BootstrapSamples {
    let one = |picks: &Vec<usize>| {
        let g = resample_graph(graph, picks);
        let neg = negatives.map(|n| resample_graph(n, picks));
        let ref_cost = score_tree(&resample_tree(reference, picks), &g).cost_norm;
        let per_run: Vec<_> = runs
            .iter()
            .map(|run| {
                let tree = resample_tree(run.tree, picks);
                let score = score_tree(&tree, &g);
                let covered = picks.iter().filter(|&&i| run.placed[i]).count();
                let mut values =
                    metric_values(&score, ref_cost, covered as f64 / picks.len().max(1) as f64);
                if let Some(neg) = &neg {
                    let fj = crate::falsejoin::false_join(&tree, neg);
                    values.extend(crate::falsejoin::metric_values(score.cost_norm, &fj));
                }
                values
            })
            .collect();
        (ref_cost, per_run)
    };
    let chunk = draws.len().div_ceil(threads.max(1)).max(1);
    let results: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = draws
            .chunks(chunk)
            .map(|part| scope.spawn(move || part.iter().map(one).collect::<Vec<_>>()))
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("bootstrap worker panicked"))
            .collect()
    });
    let mut samples = BootstrapSamples {
        reference: Vec::with_capacity(draws.len()),
        runs: runs.iter().map(|r| (r.key.clone(), Vec::new())).collect(),
    };
    for (ref_cost, per_run) in results {
        samples.reference.push(ref_cost);
        for (run, values) in runs.iter().zip(per_run) {
            samples
                .runs
                .get_mut(&run.key)
                .expect("run key")
                .push(values);
        }
    }
    samples
}

#[cfg(test)]
mod tests {
    use super::super::{interval, percentile};
    use super::*;
    use crate::{induce_tree, ItemPlacement};
    use shelfmark_core::scheme::{ClassDef, SchemeDoc};

    fn class(code: &str, parent: Option<&str>) -> ClassDef {
        ClassDef {
            code: code.into(),
            label: code.into(),
            parent: parent.map(Into::into),
            range: None,
            vocab: vec![],
            note: None,
        }
    }

    fn fixture() -> (SchemeDoc, Vec<ItemPlacement>, SimilarityGraph) {
        let scheme = SchemeDoc {
            schema: None,
            name: "toy".into(),
            version: "1".into(),
            title: "Toy".into(),
            source: None,
            code_grammar: ".*".into(),
            classes: vec![class("A", None), class("A1", Some("A")), class("B", None)],
        };
        let codes = ["A1", "A1", "A", "B", ""];
        let placements = codes
            .iter()
            .enumerate()
            .map(|(i, c)| ItemPlacement {
                id: i.to_string(),
                classes: if c.is_empty() {
                    vec![]
                } else {
                    vec![((*c).into(), 1.0)]
                },
            })
            .collect();
        let pairs = [
            (0, 1, 3.0),
            (0, 2, 1.0),
            (1, 3, 0.5),
            (2, 3, 2.0),
            (3, 4, 1.0),
        ];
        let edges = pairs
            .iter()
            .map(|&(left, right, weight)| Edge {
                left,
                right,
                weight,
            })
            .collect();
        let graph = SimilarityGraph {
            n: 5,
            k: 2,
            tau: 0.0,
            edges,
        };
        (scheme, placements, graph)
    }

    #[test]
    fn identity_draw_scores_like_the_full_sample() {
        let (scheme, placements, graph) = fixture();
        let tree = induce_tree(&scheme, &placements).unwrap();
        let picks: Vec<usize> = (0..5).collect();
        let full = score_tree(&tree, &graph);
        let again = score_tree(
            &resample_tree(&tree, &picks),
            &resample_graph(&graph, &picks),
        );
        assert!((full.cost - again.cost).abs() < 1e-12);
        assert_eq!(full.structure.mean_depth, again.structure.mean_depth);
        assert_eq!(full.structure.root_fanout, again.structure.root_fanout);
    }

    #[test]
    fn negatives_join_the_draw_and_identity_draws_reproduce_them() {
        let (scheme, placements, graph) = fixture();
        let tree = induce_tree(&scheme, &placements).unwrap();
        // Items 0 and 2 are dissimilar and meet at A (3 items of 5): (5 − 3) / 5.
        let neg = SimilarityGraph {
            n: 5,
            k: 1,
            tau: 0.0,
            edges: vec![Edge {
                left: 0,
                right: 2,
                weight: 1.0,
            }],
        };
        let runs = [BootstrapRun {
            key: "toy".into(),
            tree: &tree,
            placed: vec![true, true, true, true, false],
        }];
        let identity = vec![(0..5).collect::<Vec<_>>()];
        let s = bootstrap(&graph, Some(&neg), &tree, &runs, &identity, 1);
        let v = &s.runs["toy"][0];
        assert!((v["fj_norm"] - 0.4).abs() < 1e-15);
        assert!((v["cfj_b1"] - (v["cost_norm"] + 0.4)).abs() < 1e-15);
        // Without item 2 the pair is gone and nothing is joined falsely.
        let s = bootstrap(&graph, Some(&neg), &tree, &runs, &[vec![0, 0, 1, 3, 4]], 1);
        assert_eq!(s.runs["toy"][0]["fj_norm"], 0.0);
        let plain = bootstrap(&graph, None, &tree, &runs, &identity, 1);
        assert!(!plain.runs["toy"][0].contains_key("fj_norm"));
    }

    #[test]
    fn resampled_tree_matches_inducing_from_the_drawn_placements() {
        let (scheme, placements, graph) = fixture();
        let tree = induce_tree(&scheme, &placements).unwrap();
        let picks = vec![0, 0, 2, 4, 4];
        let drawn: Vec<_> = picks.iter().map(|&i| placements[i].clone()).collect();
        let induced = induce_tree(&scheme, &drawn).unwrap();
        let g = resample_graph(&graph, &picks);
        let a = score_tree(&resample_tree(&tree, &picks), &g);
        let b = score_tree(&induced, &g);
        assert!((a.cost - b.cost).abs() < 1e-12);
        assert_eq!(a.structure.root_fanout, b.structure.root_fanout);
        assert_eq!(a.structure.max_branching, b.structure.max_branching);
        assert_eq!(
            resample_tree(&tree, &picks).class_count,
            induced.class_count
        );
        // Edge (0,2) appears once per copy of item 0; copies of 0 share no edge.
        assert_eq!(g.edges.len(), 2);
    }

    #[test]
    fn draws_are_seeded_and_intervals_bracket_the_middle() {
        assert_eq!(draws(10, 3, 7), draws(10, 3, 7));
        assert_ne!(draws(10, 3, 7), draws(10, 3, 8));
        let values: Vec<f64> = (0..101).map(f64::from).collect();
        let [lo, hi] = interval(&values, 0.9);
        assert!((lo - 5.0).abs() < 1e-9 && (hi - 95.0).abs() < 1e-9);
        assert_eq!(percentile(&[1.0, 3.0], 0.5), 2.0);
    }
}
