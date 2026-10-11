//! False-join term: the Dasgupta cost sums `w_ij · |leaves(LCA(i, j))|`
//! over similar pairs only, so two dissimilar works joined deep cost nothing
//! (`docs/validity/README.md`, stratum B). This module charges for that.
//!
//! * A **negative sample** is drawn from the kNN complement: for every item,
//!   seeded uniform draws of other items that are not an edge of the
//!   similarity graph and whose cosine is below a similarity floor (the
//!   graph's `tau`, its mean pair cosine, by default). Each pair gets the
//!   dissimilarity weight `d_ij = floor − cos_ij`. Every kNN pair above the
//!   floor is a graph edge, so the sample never contains a pair the
//!   Dasgupta term rewards joining.
//! * The **false-join cost** has two forms. Both are 0 when every
//!   dissimilar pair meets at the root (or at a class holding every item).
//!   An unplaced item hangs off the root, so its pairs cost nothing here;
//!   the Dasgupta term already charges it the maximum on every edge.
//!   - *Outside mass* (`fj_norm`): a pair costs `d_ij · (n − |leaves(LCA)|)`,
//!     normalised by `n · Σ d`. This is one minus the dissimilarity
//!     objective of Cohen-Addad et al. (`cohenaddad2017`, which maximises
//!     `Σ d_ij · |leaves(LCA)|`), normalised the same way. A pair that
//!     meets in a main class holding a tenth of the library already pays
//!     0.9, so this form mostly charges *shallow* joins and rewards a wide
//!     top level.
//!   - *Specificity* (`fjs_norm`): a pair costs `d_ij · ln(n / |leaves(LCA)|)`,
//!     normalised by `ln n · Σ d`, the information content of the class
//!     where the pair meets. Every halving of that class adds the same
//!     charge, so meeting in a small, deep class costs much more than
//!     meeting in a main class.
//! * The **combined objective** is `C + β · FJ`, both normalised
//!   (`cfj_b<β>` with the outside-mass form, `cfjs_b<β>` with specificity).
//!   β = 1 ([`PRIMARY_BETA`]) gives the two error types equal weight per
//!   unit of normalised cost; [`BETA_SWEEP`] is the sensitivity sweep.
use crate::bootstrap::SplitMix64;
use crate::graph::cosine;
use crate::metrics::PairSums;
use crate::{Edge, SimilarityGraph, Tree};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

/// Weight of the false-join term in the primary combined objective.
pub const PRIMARY_BETA: f64 = 1.0;
/// Weights of the false-join term reported as `cfj_b<β>` (β = 0 is `cost_norm`).
pub const BETA_SWEEP: [f64; 6] = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0];
/// Negative pairs drawn per item by default: about as many pairs as a k = 20 graph has edges.
pub const DEFAULT_PER_ITEM: usize = 10;
/// Draws allowed per wanted pair before an item gives up (dense neighbourhoods).
const TRIES_PER_PAIR: usize = 50;
/// Stream tag mixed into the seed so negatives never share draws with a bootstrap.
const STREAM: u64 = 0x4E45_4741_5449_5645;

/// How a negative sample is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NegativeSpec {
    /// Pairs with cosine at or above this are never negatives.
    pub floor: f64,
    /// Pairs each item tries to start.
    pub per_item: usize,
    pub seed: u64,
}

/// The false-join cost of one tree on a negative sample.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FalseJoin {
    /// `Σ d_ij` over the sample.
    pub dissimilarity: f64,
    /// `Σ d_ij · (n − |leaves(LCA)|)`.
    pub joined: f64,
    /// `joined / (n · Σ d)`, in `[0, 1)`; lower is better.
    pub fj_norm: f64,
    /// `Σ d_ij · ln(n / |leaves(LCA)|)`.
    pub specific: f64,
    /// `specific / (ln n · Σ d)`, in `[0, 1)`; lower is better.
    pub fjs_norm: f64,
}

/// Seeded kNN-complement negative sample from row-major vectors (`n × dims`).
/// The result is a [`SimilarityGraph`] whose edge weights are dissimilarities
/// `floor − cos`, `k` is `per_item` and `tau` is the floor, so the bootstrap
/// resamples it exactly like the similarity graph.
pub fn negative_sample(
    vectors: &[f32],
    dims: usize,
    graph: &SimilarityGraph,
    spec: &NegativeSpec,
) -> SimilarityGraph {
    let n = graph.n;
    assert_eq!(vectors.len(), n * dims, "vectors must be n x dims");
    let row = |i: usize| &vectors[i * dims..(i + 1) * dims];
    let edges: HashSet<(usize, usize)> = graph
        .edges
        .iter()
        .map(|e| (e.left.min(e.right), e.left.max(e.right)))
        .collect();
    let mut picked = BTreeMap::<(usize, usize), f64>::new();
    let mut rng = SplitMix64::new(spec.seed ^ STREAM);
    for i in 0..n {
        let (mut accepted, mut tries) = (0, 0);
        while accepted < spec.per_item && tries < spec.per_item * TRIES_PER_PAIR && n > 1 {
            tries += 1;
            let j = rng.below(n);
            let key = (i.min(j), i.max(j));
            if j == i || edges.contains(&key) || picked.contains_key(&key) {
                continue;
            }
            let cos = cosine(row(i), row(j));
            if cos < spec.floor {
                picked.insert(key, spec.floor - cos);
                accepted += 1;
            }
        }
    }
    SimilarityGraph {
        n,
        k: spec.per_item,
        tau: spec.floor,
        edges: picked
            .into_iter()
            .map(|((left, right), weight)| Edge {
                left,
                right,
                weight,
            })
            .collect(),
    }
}

/// False-join cost of `tree` on the negative sample `negatives`.
pub fn false_join(tree: &Tree, negatives: &SimilarityGraph) -> FalseJoin {
    let n = tree.item_leaves.len() as f64;
    let sums = PairSums::of(tree, negatives);
    let denom = n * sums.weight;
    let denom_s = n.ln() * sums.weight;
    FalseJoin {
        dissimilarity: sums.weight,
        joined: sums.outside,
        fj_norm: if denom > 0.0 {
            sums.outside / denom
        } else {
            0.0
        },
        specific: sums.specificity,
        fjs_norm: if denom_s > 0.0 {
            sums.specificity / denom_s
        } else {
            0.0
        },
    }
}

/// The combined objective `cost_norm + β · fj_norm` (lower is better).
pub fn combined(cost_norm: f64, fj_norm: f64, beta: f64) -> f64 {
    cost_norm + beta * fj_norm
}

/// Metric names of the combined objective at `beta`, outside-mass form then
/// specificity form: `cfj_b1` and `cfjs_b1`, `cfj_b0.25` and `cfjs_b0.25`, ...
pub fn combined_names(beta: f64) -> [String; 2] {
    [format!("cfj_b{beta}"), format!("cfjs_b{beta}")]
}

/// Every false-join metric name, in report order.
pub fn metric_names() -> Vec<String> {
    let mut out = vec!["fj_norm".to_owned(), "fjs_norm".to_owned()];
    out.extend(BETA_SWEEP.iter().flat_map(|&b| combined_names(b)));
    out
}

/// `fj_norm`, `fjs_norm` and both combined objectives at every β of the
/// sweep, by metric name, given the run's Dasgupta `cost_norm`.
pub fn metric_values(cost_norm: f64, fj: &FalseJoin) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::from([
        ("fj_norm".to_owned(), fj.fj_norm),
        ("fjs_norm".to_owned(), fj.fjs_norm),
    ]);
    for beta in BETA_SWEEP {
        let [outside, specific] = combined_names(beta);
        out.insert(outside, combined(cost_norm, fj.fj_norm, beta));
        out.insert(specific, combined(cost_norm, fj.fjs_norm, beta));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{induce_tree, score_tree, ItemPlacement};
    use shelfmark_core::scheme::{ClassDef, SchemeDoc};

    fn scheme(classes: &[(&str, Option<&str>)]) -> SchemeDoc {
        SchemeDoc {
            schema: None,
            name: "toy".into(),
            version: "1".into(),
            title: "Toy".into(),
            source: None,
            code_grammar: ".*".into(),
            classes: classes
                .iter()
                .map(|&(code, parent)| ClassDef {
                    code: code.into(),
                    label: code.into(),
                    parent: parent.map(Into::into),
                    range: None,
                    vocab: vec![],
                    note: None,
                })
                .collect(),
        }
    }

    fn tree(doc: &SchemeDoc, codes: &[&str]) -> Tree {
        let items: Vec<_> = codes
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
        induce_tree(doc, &items).unwrap()
    }

    fn graph(n: usize, pairs: &[(usize, usize, f64)]) -> SimilarityGraph {
        SimilarityGraph {
            n,
            k: 1,
            tau: 0.0,
            edges: pairs
                .iter()
                .map(|&(left, right, weight)| Edge {
                    left,
                    right,
                    weight,
                })
                .collect(),
        }
    }

    #[test]
    fn hand_computed_false_join() {
        // root ─ A ─ A1 {0, 1}
        //        │   └ {2}
        //        └ B {3};  item 4 unplaced (under the root).
        let doc = scheme(&[("A", None), ("A1", Some("A")), ("B", None)]);
        let t = tree(&doc, &["A1", "A1", "A", "B", ""]);
        // (0,1) meet at A1 (2 items): 5 − 2 = 3, times d = 1.
        // (0,2) meet at A (3 items): 5 − 3 = 2, times d = 0.5.
        // (0,3) and (3,4) meet at the root: 0.
        let neg = graph(5, &[(0, 1, 1.0), (0, 2, 0.5), (0, 3, 2.0), (3, 4, 1.0)]);
        let fj = false_join(&t, &neg);
        assert_eq!(fj.dissimilarity, 4.5);
        assert_eq!(fj.joined, 4.0);
        assert!((fj.fj_norm - 4.0 / 22.5).abs() < 1e-15);
        // Specificity: 1 · ln(5/2) + 0.5 · ln(5/3), over ln 5 · 4.5.
        let specific = (2.5f64).ln() + 0.5 * (5.0f64 / 3.0).ln();
        assert!((fj.specific - specific).abs() < 1e-12);
        assert!((fj.fjs_norm - specific / (5.0f64.ln() * 4.5)).abs() < 1e-12);
    }

    #[test]
    fn specificity_charges_deep_joins_more_than_shallow_ones() {
        // Eight items; one dissimilar pair meets either in a main class of
        // four (A) or in a section of two (A1) beneath it.
        let doc = scheme(&[("A", None), ("A1", Some("A")), ("B", None)]);
        let neg = graph(8, &[(0, 1, 1.0)]);
        let shallow = tree(&doc, &["A", "A", "A", "A", "B", "B", "B", "B"]);
        let deep = tree(&doc, &["A1", "A1", "A", "A", "B", "B", "B", "B"]);
        let (s, d) = (false_join(&shallow, &neg), false_join(&deep, &neg));
        // Outside mass: (8 − 4)/8 = 0.5 against (8 − 2)/8 = 0.75.
        assert_eq!((s.fj_norm, d.fj_norm), (0.5, 0.75));
        // Specificity: ln 2 / ln 8 = 1/3 against ln 4 / ln 8 = 2/3.
        assert!((s.fjs_norm - 1.0 / 3.0).abs() < 1e-12);
        assert!((d.fjs_norm - 2.0 / 3.0).abs() < 1e-12);
        assert!(d.fjs_norm / s.fjs_norm > d.fj_norm / s.fj_norm);
    }

    #[test]
    fn flat_and_single_class_trees_join_nothing() {
        let doc = scheme(&[("A", None)]);
        let neg = graph(3, &[(0, 1, 1.0), (1, 2, 0.3)]);
        assert_eq!(false_join(&tree(&doc, &["", "", ""]), &neg).fj_norm, 0.0);
        assert_eq!(false_join(&tree(&doc, &["A", "A", "A"]), &neg).fj_norm, 0.0);
        assert_eq!(false_join(&tree(&doc, &["A"]), &graph(1, &[])).fj_norm, 0.0);
    }

    #[test]
    fn dasgupta_is_blind_to_a_false_join_that_this_term_sees() {
        // Items 0 and 1 are similar; 2 and 3 are dissimilar. Joining 2 and 3
        // in one class leaves the Dasgupta cost unchanged but costs here.
        let doc = scheme(&[("A", None), ("B", None), ("C", None)]);
        let apart = tree(&doc, &["A", "A", "B", "C"]);
        let joined = tree(&doc, &["A", "A", "B", "B"]);
        let sim = graph(4, &[(0, 1, 1.0)]);
        let neg = graph(4, &[(2, 3, 1.0)]);
        assert_eq!(
            score_tree(&apart, &sim).cost,
            score_tree(&joined, &sim).cost
        );
        assert_eq!(false_join(&apart, &neg).fj_norm, 0.0);
        assert_eq!(false_join(&apart, &neg).fjs_norm, 0.0);
        // (2,3) meet at B (2 items): (4 − 2) / 4 = 0.5, and ln 2 / ln 4 = 0.5.
        assert_eq!(false_join(&joined, &neg).fj_norm, 0.5);
        assert!((false_join(&joined, &neg).fjs_norm - 0.5).abs() < 1e-15);
        let c = score_tree(&joined, &sim).cost_norm;
        assert_eq!(combined(c, 0.5, 2.0), c + 1.0);
    }

    #[test]
    fn fractional_placements_spread_the_pair_cost() {
        // Item 0 is half in A, half in B; item 1 is in A (A holds 1.5 of
        // mass). Half the pair meets at A: 0.5 · (2 − 1.5); half at the root: 0.
        let doc = scheme(&[("A", None), ("B", None)]);
        let items = [
            ItemPlacement {
                id: "0".into(),
                classes: vec![("A".into(), 0.5), ("B".into(), 0.5)],
            },
            ItemPlacement {
                id: "1".into(),
                classes: vec![("A".into(), 1.0)],
            },
        ];
        let t = induce_tree(&doc, &items).unwrap();
        let fj = false_join(&t, &graph(2, &[(0, 1, 1.0)]));
        assert!((fj.joined - 0.25).abs() < 1e-15);
        assert!((fj.fj_norm - 0.125).abs() < 1e-15);
    }

    fn unit(angle: f32) -> [f32; 2] {
        [angle.cos(), angle.sin()]
    }

    #[test]
    fn negatives_avoid_edges_respect_the_floor_and_are_seeded() {
        // Six points on a half circle: neighbours are similar, ends are not.
        let angles = [0.0f32, 0.3, 0.6, 1.6, 2.6, 3.1];
        let vectors: Vec<f32> = angles.iter().flat_map(|&a| unit(a)).collect();
        let g = graph(6, &[(0, 1, 0.5), (1, 2, 0.5), (4, 5, 0.5)]);
        let spec = NegativeSpec {
            floor: 0.2,
            per_item: 2,
            seed: 7,
        };
        let neg = negative_sample(&vectors, 2, &g, &spec);
        assert!(!neg.edges.is_empty());
        assert_eq!((neg.n, neg.k, neg.tau), (6, 2, 0.2));
        for e in &neg.edges {
            assert!(e.left < e.right);
            assert!(!g
                .edges
                .iter()
                .any(|x| (x.left, x.right) == (e.left, e.right)));
            let cos = f64::from((angles[e.left] - angles[e.right]).cos());
            assert!(cos < 0.2, "pair {e:?} has cosine {cos}");
            assert!((e.weight - (0.2 - cos)).abs() < 1e-6);
        }
        assert_eq!(neg, negative_sample(&vectors, 2, &g, &spec));
        assert!(neg.edges.len() <= 6 * 2);
        // A floor below every cosine leaves nothing to sample.
        let none = NegativeSpec {
            floor: -1.0,
            ..spec
        };
        assert!(negative_sample(&vectors, 2, &g, &none).edges.is_empty());
    }

    #[test]
    fn combined_sweep_is_named_and_starts_from_cost() {
        let fj = FalseJoin {
            dissimilarity: 1.0,
            joined: 0.1,
            fj_norm: 0.1,
            specific: 0.2,
            fjs_norm: 0.2,
        };
        let v = metric_values(0.5, &fj);
        assert_eq!((v["fj_norm"], v["fjs_norm"]), (0.1, 0.2));
        assert_eq!(v["cfj_b1"], 0.6);
        assert_eq!(v["cfjs_b1"], 0.7);
        assert_eq!(v["cfj_b0.25"], 0.525);
        assert_eq!(v["cfj_b8"], 0.5 + 0.8);
        assert_eq!(v.len(), 2 + 2 * BETA_SWEEP.len());
        let names = metric_names();
        assert_eq!(names.len(), v.len());
        assert!(names.iter().all(|m| v.contains_key(m)));
        assert!(BETA_SWEEP.contains(&PRIMARY_BETA));
    }
}
