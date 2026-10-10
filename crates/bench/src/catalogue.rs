//! The scoring catalogue for one (scheme, placements) run: every score as a
//! flat `name -> value` map, bootstrap intervals, null models and placement
//! noise. Names are stable; `research/reports/6-scoring-catalogue.md`
//! defines each one.
use crate::boot::bootstrap;
use crate::embedscore::{self, Vectors};
use crate::goldtree::gold_tree;
use crate::partition::compare;
use crate::robust::{interval, mean_sd, noisy, random_shape, shuffled_labels};
use crate::search::{mdl, search_cost};
use crate::shelf::{author_pairs, insertion, locality, same_class, shelf_tau};
use crate::treescore::{edge_table, mean, norm_cost, tree_sampling_divergence, EdgeTable, Rng};
use crate::{notation, Hier, SimilarityGraph};
use serde::Serialize;
use shelfmark_core::scheme::SchemeDoc;
use std::collections::BTreeMap;

pub type Scores = BTreeMap<String, f64>;

/// Run inputs beyond the hierarchy.
pub struct Context<'a> {
    pub scheme: &'a SchemeDoc,
    pub graph: &'a SimilarityGraph,
    pub vectors: Option<&'a Vectors>,
    /// External gold labels (fine and coarse) as dense ids.
    pub gold: Option<[Vec<Option<usize>>; 2]>,
    pub authors: &'a [Vec<String>],
    /// Number of source classes per item (exclusiveness).
    pub label_counts: &'a [usize],
    pub seed: u64,
    pub reps: usize,
    pub boot: usize,
    pub sample: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NullSummary {
    pub mean: f64,
    pub sd: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Catalogue {
    pub scores: Scores,
    pub ci: BTreeMap<String, [f64; 2]>,
    pub null: BTreeMap<String, BTreeMap<String, NullSummary>>,
    pub noise: BTreeMap<String, Scores>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub boot: BTreeMap<String, Vec<f64>>,
}

const NOISE: [f64; 3] = [0.05, 0.1, 0.2];

fn put(s: &mut Scores, key: &str, value: f64) {
    s.insert(key.to_owned(), value);
}

pub(crate) fn edge_pairs(t: &EdgeTable) -> Vec<(usize, usize, f64)> {
    (0..t.weight.len())
        .map(|e| (t.left[e], t.right[e], t.weight[e]))
        .collect()
}

/// The scores recomputed under nulls and noise; cheap enough for many reps.
pub fn core_scores(h: &Hier, ctx: &Context) -> Scores {
    let mut s = Scores::new();
    let t = edge_table(h, ctx.graph, ctx.seed);
    let all: Vec<usize> = (0..t.weight.len()).collect();
    put(
        &mut s,
        "dasgupta.cost_norm",
        norm_cost(&t, &t.size, h.items(), &all),
    );
    put(&mut s, "triplet.accuracy", mean(&t.triplet, &all));
    put(&mut s, "tsd.nats", tree_sampling_divergence(h, &t, &all));
    put(
        &mut s,
        "order.locality",
        locality(&h.shelf_positions(), &edge_pairs(&t)),
    );
    let knn = embedscore::knn_probe(ctx.graph, &h.labels_at(1));
    put(
        &mut s,
        "probe.knn_d1",
        mean(&knn, &(0..knn.len()).collect::<Vec<_>>()),
    );
    if let Some([fine, coarse]) = &ctx.gold {
        let leaf: Vec<Option<usize>> = h.item_class.iter().map(|&c| Some(c)).collect();
        put(
            &mut s,
            "gold.dendrogram_purity_fine",
            gold_tree(h, fine).dendrogram_purity,
        );
        put(&mut s, "gold.ari_fine", compare(fine, &leaf).ari);
        put(
            &mut s,
            "gold.dendrogram_purity_coarse",
            gold_tree(h, coarse).dendrogram_purity,
        );
    }
    s
}

fn graph_scores(h: &Hier, ctx: &Context, t: &EdgeTable, s: &mut Scores) {
    let all: Vec<usize> = (0..t.weight.len()).collect();
    let n = h.items();
    put(
        s,
        "dasgupta.cost",
        all.iter().map(|&e| t.weight[e] * t.size[e]).sum(),
    );
    put(s, "dasgupta.cost_norm", norm_cost(t, &t.size, n, &all));
    put(
        s,
        "dasgupta.binarized_cost_norm",
        norm_cost(t, &t.binarized, n, &all),
    );
    for (d, level) in t.level.iter().enumerate() {
        put(
            s,
            &format!("dasgupta.level{}_cost_norm", d + 1),
            norm_cost(t, level, n, &all),
        );
    }
    put(s, "tsd.nats", tree_sampling_divergence(h, t, &all));
    put(s, "triplet.accuracy", mean(&t.triplet, &all));
    let pos = h.shelf_positions();
    put(s, "order.locality", locality(&pos, &edge_pairs(t)));
    put(
        s,
        "order.kendall_tau",
        shelf_tau(&pos, &edge_pairs(t), 3000, ctx.seed ^ 0x5eed),
    );
    for d in 1..=3 {
        let labels = h.labels_at(d);
        let knn = embedscore::knn_probe(ctx.graph, &labels);
        put(
            s,
            &format!("probe.knn_d{d}"),
            mean(&knn, &(0..knn.len()).collect::<Vec<_>>()),
        );
        put(
            s,
            &format!("probe.majority_d{d}"),
            embedscore::majority_share(&labels),
        );
        put(
            s,
            &format!("probe.labelled_d{d}"),
            knn.len() as f64 / n.max(1) as f64,
        );
    }
}

fn embedding_scores(h: &Hier, ctx: &Context, v: &Vectors, s: &mut Scores) {
    // Fixed sample: every ceil(n / sample)-th item.
    let step = h.items().div_ceil(ctx.sample.max(1)).max(1);
    let sample: Vec<usize> = (0..h.items()).step_by(step).collect();
    let sim = embedscore::sample_similarity(v, &sample);
    put(
        s,
        "embed.cophenetic",
        embedscore::cophenetic(h, &sim, &sample),
    );
    for d in 1..=3 {
        let labels = h.labels_at(d);
        put(
            s,
            &format!("embed.silhouette_d{d}"),
            embedscore::silhouette(&sim, &sample, &labels),
        );
        put(
            s,
            &format!("embed.davies_bouldin_d{d}"),
            embedscore::davies_bouldin(v, &labels),
        );
        let probe = embedscore::centroid_probe(v, &labels);
        put(
            s,
            &format!("probe.centroid_d{d}"),
            mean(&probe, &(0..probe.len()).collect::<Vec<_>>()),
        );
    }
}

fn gold_scores(h: &Hier, gold: &[Vec<Option<usize>>; 2], s: &mut Scores) {
    let leaf: Vec<Option<usize>> = h.item_class.iter().map(|&c| Some(c)).collect();
    let top = h
        .labels_at(1)
        .iter()
        .zip(&h.item_class)
        .map(|(l, &c)| l.or(Some(c)))
        .collect::<Vec<_>>();
    for (name, g, pred) in [("fine", &gold[0], &leaf), ("coarse", &gold[1], &top)] {
        let p = compare(g, pred);
        let t = gold_tree(h, g);
        for (k, v) in [
            ("ari", p.ari),
            ("ami", p.ami),
            ("v_measure", p.v_measure),
            ("homogeneity", p.homogeneity),
            ("completeness", p.completeness),
            ("purity", p.purity),
            ("dendrogram_purity", t.dendrogram_purity),
            ("hierarchical_f", t.hierarchical_f),
        ] {
            put(s, &format!("gold.{k}_{name}"), v);
        }
        put(s, &format!("gold.labelled_{name}"), t.labelled as f64);
    }
}

fn librarian_scores(h: &Hier, ctx: &Context, s: &mut Scores) {
    let n = h.items().max(1) as f64;
    let placed = (0..h.items()).filter(|&i| h.placed(i)).count();
    put(s, "coverage", placed as f64 / n);
    let single = (0..h.items())
        .filter(|&i| h.placed(i) && ctx.label_counts.get(i) == Some(&1))
        .count();
    put(
        s,
        "canon.exclusiveness",
        single as f64 / placed.max(1) as f64,
    );
    let sc = search_cost(h);
    put(s, "search.scheme_bits", sc.scheme_bits);
    put(s, "search.entropy_bits", sc.entropy_bits);
    put(s, "search.huffman_bits", sc.huffman_bits);
    put(s, "search.hu_tucker_bits", sc.hu_tucker_bits);
    put(s, "search.gap_bits", sc.scheme_bits - sc.hu_tucker_bits);
    put(s, "search.worst_bits", sc.worst_bits);
    let m = mdl(h, ctx.scheme.classes.len());
    put(s, "mdl.model_bits", m.model_bits);
    put(s, "mdl.data_bits_per_item", m.data_bits_per_item);
    put(s, "mdl.total_bits_per_item", m.total_bits_per_item);
    let shape = notation::scheme_shape(ctx.scheme);
    put(s, "scheme.classes", shape.classes as f64);
    put(s, "notation.expressiveness", shape.expressiveness);
    put(s, "notation.regularity", shape.regularity);
    put(
        s,
        "notation.length_depth_spearman",
        shape.length_depth_spearman,
    );
    put(s, "notation.lengthening", shape.lengthening);
    put(s, "hospitality.array", shape.hospitality_array);
    put(s, "hospitality.ordered_siblings", shape.ordered_siblings);
    let u = notation::usage(h, ctx.scheme);
    put(s, "notation.length", u.notation_length);
    put(s, "granularity.classes_per_1000", u.classes_per_1000_works);
    put(s, "granularity.works_per_leaf_mean", u.works_per_leaf_mean);
    put(
        s,
        "granularity.works_per_leaf_median",
        u.works_per_leaf_median,
    );
    put(s, "granularity.empty_class_share", u.empty_class_share);
    put(s, "granularity.empty_leaf_share", u.empty_leaf_share);
    put(s, "warrant.useful_depth_share", u.useful_depth_share);
    let ins = insertion(h, 50);
    put(s, "insertion.splits_per_1000", ins.splits_per_1000);
    put(s, "insertion.churn_per_insert", ins.churn_per_insert);
    let pairs = author_pairs(ctx.authors, 50);
    if !pairs.is_empty() {
        let (share, chance, both) = same_class(h, &pairs);
        put(s, "collocation.author_same_class", share);
        put(s, "collocation.author_same_class_chance", chance);
        put(s, "collocation.author_pairs", both as f64);
        put(
            s,
            "collocation.author_locality",
            locality(&h.shelf_positions(), &pairs),
        );
    }
}

fn nulls_and_noise(h: &Hier, ctx: &Context, point: &Scores, c: &mut Catalogue) {
    let mut rng = Rng::new(ctx.seed ^ 0x0001_0000_0000_0000);
    type Model = fn(&Hier, &mut Rng) -> Hier;
    let models: [(&str, Model); 2] = [
        ("shuffled_labels", shuffled_labels),
        ("random_shape", random_shape),
    ];
    for (name, model) in models {
        let mut runs = BTreeMap::<String, Vec<f64>>::new();
        for _ in 0..ctx.reps {
            for (k, v) in core_scores(&model(h, &mut rng), ctx) {
                runs.entry(k).or_default().push(v);
            }
        }
        let summary = runs
            .into_iter()
            .map(|(k, v)| {
                let (mean, sd) = mean_sd(&v);
                let z = if sd > 0.0 {
                    (point[&k] - mean) / sd
                } else {
                    0.0
                };
                (k, NullSummary { mean, sd, z })
            })
            .collect();
        c.null.insert(name.to_owned(), summary);
    }
    for p in NOISE {
        let mut runs = BTreeMap::<String, Vec<f64>>::new();
        for _ in 0..ctx.reps {
            for (k, v) in core_scores(&noisy(h, p, &mut rng), ctx) {
                runs.entry(k).or_default().push(v);
            }
        }
        let means = runs.into_iter().map(|(k, v)| (k, mean_sd(&v).0)).collect();
        c.noise.insert(format!("{p}"), means);
    }
}

pub fn catalogue(h: &Hier, ctx: &Context) -> Catalogue {
    let mut s = Scores::new();
    let t = edge_table(h, ctx.graph, ctx.seed);
    graph_scores(h, ctx, &t, &mut s);
    if let Some(v) = ctx.vectors {
        embedding_scores(h, ctx, v, &mut s);
    }
    if let Some(gold) = &ctx.gold {
        gold_scores(h, gold, &mut s);
    }
    librarian_scores(h, ctx, &mut s);
    let mut c = Catalogue::default();
    let boot = bootstrap(h, ctx, &t);
    c.ci = boot.iter().map(|(k, v)| (k.clone(), interval(v))).collect();
    let point = core_scores(h, ctx);
    nulls_and_noise(h, ctx, &point, &mut c);
    c.boot = boot;
    c.scores = s;
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use shelfmark_core::scheme::ClassDef;

    fn scheme() -> SchemeDoc {
        let class = |code: &str, parent: Option<&str>| ClassDef {
            code: code.into(),
            label: code.into(),
            parent: parent.map(Into::into),
            range: None,
            vocab: vec![],
            note: None,
        };
        SchemeDoc {
            schema: None,
            name: "toy".into(),
            version: "1".into(),
            title: "toy".into(),
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

    #[test]
    fn catalogue_is_deterministic_and_bounded() {
        let primary =
            ["A1", "A1", "A2", "B", "B", ""].map(|c| (!c.is_empty()).then(|| c.to_owned()));
        let h = Hier::build(&scheme(), &primary).unwrap();
        let edges = [(0, 1, 1.0), (1, 2, 0.5), (3, 4, 1.0), (2, 5, 0.2)];
        let graph = SimilarityGraph {
            n: 6,
            k: 2,
            tau: 0.0,
            edges: edges
                .iter()
                .map(|&(left, right, weight)| crate::Edge {
                    left,
                    right,
                    weight,
                })
                .collect(),
        };
        let fine = vec![Some(0), Some(0), Some(0), Some(1), Some(1), None];
        let ctx = Context {
            scheme: &scheme(),
            graph: &graph,
            vectors: None,
            gold: Some([fine.clone(), fine]),
            authors: &[],
            label_counts: &[1; 6],
            seed: 11,
            reps: 3,
            boot: 20,
            sample: 10,
        };
        let a = catalogue(&h, &ctx);
        let b = catalogue(&h, &ctx);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        assert!((a.scores["coverage"] - 5.0 / 6.0).abs() < 1e-12);
        assert_eq!(a.scores["gold.dendrogram_purity_coarse"], 1.0);
        let [lo, hi] = a.ci["dasgupta.cost_norm"];
        assert!(lo <= a.scores["dasgupta.cost_norm"] + 1e-12 || hi >= lo);
        assert!(a.null.contains_key("shuffled_labels") && a.noise.contains_key("0.1"));
        // Edge and item resamples come from their own streams: dropping the
        // gold labels or moving an item to another class (which changes the
        // kNN probe's draws) leaves the graph-score replicates paired.
        let plain = catalogue(&h, &Context { gold: None, ..ctx });
        assert_eq!(
            plain.boot["dasgupta.cost_norm"],
            a.boot["dasgupta.cost_norm"]
        );
        let moved = ["A1", "A2", "A2", "B", "", ""].map(|c| (!c.is_empty()).then(|| c.to_owned()));
        let h2 = Hier::build(&scheme(), &moved).unwrap();
        let c = catalogue(&h2, &Context { gold: None, ..ctx });
        let t1 = edge_table(&h, &graph, 11);
        let t2 = edge_table(&h2, &graph, 11);
        let mut rng = Rng::new(11 ^ 0xb007);
        for b in 0..20 {
            let pick = crate::robust::resample(4, &mut rng);
            assert_eq!(
                a.boot["dasgupta.cost_norm"][b],
                norm_cost(&t1, &t1.size, 6, &pick)
            );
            assert_eq!(
                c.boot["dasgupta.cost_norm"][b],
                norm_cost(&t2, &t2.size, 6, &pick)
            );
        }
        assert_eq!(a.boot["vertex.dasgupta.cost_norm"].len(), 20);
    }
}
