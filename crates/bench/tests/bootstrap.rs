use shelfmark_bench::bootstrap::{self, multiplicities, Prepared};
use shelfmark_bench::{
    average_linkage_reference, induce_tree, score_tree, Edge, ItemPlacement, SimilarityGraph,
};
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

fn scheme() -> SchemeDoc {
    SchemeDoc {
        schema: None,
        name: "toy".into(),
        version: "1".into(),
        title: "Toy".into(),
        source: None,
        code_grammar: ".*".into(),
        classes: vec![
            class("A", None),
            class("A1", Some("A")),
            class("A2", Some("A")),
            class("B", None),
        ],
    }
}

fn place(id: &str, classes: &[(&str, f64)]) -> ItemPlacement {
    ItemPlacement {
        id: id.into(),
        classes: classes.iter().map(|(c, m)| ((*c).into(), *m)).collect(),
    }
}

fn edge(left: usize, right: usize, weight: f64) -> Edge {
    Edge {
        left,
        right,
        weight,
    }
}

fn placements() -> Vec<ItemPlacement> {
    vec![
        place("0", &[("A1", 1.0)]),
        place("1", &[("A1", 1.0)]),
        place("2", &[("A2", 0.5), ("B", 0.5)]),
        place("3", &[("B", 1.0)]),
        place("4", &[]),
        place("5", &[("A", 1.0)]),
    ]
}

fn graph() -> SimilarityGraph {
    SimilarityGraph {
        n: 6,
        k: 2,
        tau: 0.0,
        edges: vec![
            edge(0, 1, 0.9),
            edge(0, 2, 0.3),
            edge(1, 5, 0.4),
            edge(2, 3, 0.7),
            edge(3, 4, 0.2),
            edge(4, 5, 0.1),
        ],
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

#[test]
fn unit_weights_reproduce_the_point_scores() {
    let tree = induce_tree(&scheme(), &placements()).unwrap();
    let g = graph();
    let score = score_tree(&tree, &g);
    let rep = Prepared::new(&tree, &g, true).replicate(&[1.0; 6]);
    let s = &score.structure;
    for (key, want) in [
        ("cost_norm", score.cost_norm),
        ("revenue_norm", score.revenue_norm),
        ("coverage", 5.0 / 6.0),
        ("mean_depth", s.mean_depth),
        ("normalized_entropy", s.normalized_entropy),
        ("effective_classes", s.effective_classes),
        ("gini", s.gini),
        ("largest_class_share", s.largest_class_share),
        ("nav_scan", s.nav_scan),
        ("nav_log_a0", s.nav_log_a0),
        ("nav_log_a1", s.nav_log_a1),
        ("nav_log_a2", s.nav_log_a2),
    ] {
        assert!(close(rep[key], want), "{key}: {} != {want}", rep[key]);
    }
}

#[test]
fn a_weight_of_two_equals_a_duplicated_item_without_a_self_pair() {
    // Item 0 counted twice equals a corpus where item 6 copies item 0: same
    // class, same edges, and no edge between the two copies.
    let mut dup = placements();
    dup.push(place("6", &[("A1", 1.0)]));
    let mut g = graph();
    g.n = 7;
    g.edges.extend([edge(1, 6, 0.9), edge(2, 6, 0.3)]);
    let dup_tree = induce_tree(&scheme(), &dup).unwrap();
    let want = score_tree(&dup_tree, &g);
    let tree = induce_tree(&scheme(), &placements()).unwrap();
    let rep = Prepared::new(&tree, &graph(), true).replicate(&[2.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    assert!(close(rep["cost_norm"], want.cost_norm));
    assert!(close(rep["coverage"], 6.0 / 7.0));
    assert!(close(rep["gini"], want.structure.gini));
    assert!(close(
        rep["effective_classes"],
        want.structure.effective_classes
    ));
}

#[test]
fn multiplicities_are_seeded_and_sum_to_n() {
    let a = multiplicities(500, 7, 3);
    assert_eq!(a, multiplicities(500, 7, 3));
    assert_ne!(a, multiplicities(500, 7, 4));
    assert_ne!(a, multiplicities(500, 8, 3));
    assert_eq!(a.iter().sum::<f64>(), 500.0);
}

#[test]
fn replicates_are_paired_and_intervals_bracket_the_estimate() {
    let tree = induce_tree(&scheme(), &placements()).unwrap();
    let g = graph();
    let hac = average_linkage_reference(&g);
    let p = Prepared::new(&tree, &g, true);
    let r = Prepared::new(&hac, &g, false);
    let first = bootstrap::run(&p, None, Some(&r), 200, 11);
    assert_eq!(first, bootstrap::run(&p, None, Some(&r), 200, 11));
    // The reference replicates of one run reproduce the skill of another.
    let refs = first.reference_costs().unwrap();
    let second = bootstrap::run(&p, Some(refs), None, 200, 11);
    assert_eq!(first.values["skill"], second.values["skill"]);
    let ci = first.summary().ci;
    let point = score_tree(&tree, &g).cost_norm;
    assert!(ci["cost_norm"][0] <= point && point <= ci["cost_norm"][1]);
    assert!(ci["skill"][0] <= ci["skill"][1]);
    assert_eq!(first.values["cost_norm"].len(), 200);
}

#[test]
fn percentile_interpolates() {
    assert_eq!(bootstrap::percentile(&[1.0, 2.0, 3.0], 0.5), 2.0);
    assert_eq!(bootstrap::percentile(&[0.0, 10.0], 0.25), 2.5);
    assert_eq!(bootstrap::interval(&[3.0, 1.0, 2.0], 1.0), [1.0, 3.0]);
}

#[test]
fn reference_costs_are_the_reference_trees_not_the_scored_trees() {
    // A reference run scores some scheme (here with nothing placed) next to
    // the average-linkage tree; another run must take the latter's costs.
    let g = graph();
    let hac = average_linkage_reference(&g);
    let r = Prepared::new(&hac, &g, false);
    let unplaced: Vec<_> = placements().iter().map(|p| place(&p.id, &[])).collect();
    let empty = induce_tree(&scheme(), &unplaced).unwrap();
    let reference_run = bootstrap::run(&Prepared::new(&empty, &g, true), None, Some(&r), 50, 3);
    let costs = reference_run.reference_costs().unwrap();
    assert_ne!(costs, reference_run.values["cost_norm"].as_slice());
    let tree = induce_tree(&scheme(), &placements()).unwrap();
    let p = Prepared::new(&tree, &g, true);
    let direct = bootstrap::run(&p, None, Some(&r), 50, 3);
    let reused = bootstrap::run(&p, Some(costs), None, 50, 3);
    assert_eq!(direct.values["skill"], reused.values["skill"]);
    assert_eq!(
        direct.values["ref_cost_norm"],
        reused.values["ref_cost_norm"]
    );
}
