//! Bootstrap replicates of the catalogue scores.
//!
//! * Edge bootstrap: graph edges resampled with replacement (graph scores).
//! * Vertex bootstrap: items resampled with replacement and every edge
//!   counted `m_i · m_j` times, where `m` is the item multiplicity. Edges
//!   that share an item are dependent; resampling items keeps that
//!   dependence, so its intervals are wider and are the ones to trust on
//!   small corpora. Reported under `vertex.<score>`.
//! * Item bootstrap: the same item resample for the external-gold scores.
//! * The kNN probe resamples its labelled items.
//!
//! Each kind draws from its own generator stream. Two runs over the same
//! graph and item count therefore get identical edge and item resamples in
//! every replicate, so replicate `b` of one run pairs with replicate `b` of
//! another (paired bootstrap of score differences).
use crate::catalogue::{edge_pairs, Context};
use crate::embedscore;
use crate::goldtree::gold_tree;
use crate::partition::compare;
use crate::robust::resample;
use crate::shelf::locality;
use crate::treescore::{mean, norm_cost, tree_sampling_divergence, EdgeTable, Rng};
use crate::Hier;
use std::collections::BTreeMap;

/// Edge indices repeated by the product of their endpoints' multiplicities.
pub fn vertex_pick(t: &EdgeTable, multiplicity: &[usize]) -> Vec<usize> {
    let mut pick = vec![];
    for e in 0..t.weight.len() {
        let m = multiplicity[t.left[e]] * multiplicity[t.right[e]];
        pick.extend(std::iter::repeat_n(e, m));
    }
    pick
}

fn graph_replicate(
    h: &Hier,
    t: &EdgeTable,
    pick: &[usize],
    prefix: &str,
    push: &mut dyn FnMut(String, f64),
) {
    let n = h.items();
    let pairs = edge_pairs(t);
    push(
        format!("{prefix}dasgupta.cost_norm"),
        norm_cost(t, &t.size, n, pick),
    );
    push(
        format!("{prefix}dasgupta.binarized_cost_norm"),
        norm_cost(t, &t.binarized, n, pick),
    );
    push(format!("{prefix}triplet.accuracy"), mean(&t.triplet, pick));
    push(
        format!("{prefix}tsd.nats"),
        tree_sampling_divergence(h, t, pick),
    );
    let picked: Vec<_> = pick.iter().map(|&e| pairs[e]).collect();
    push(
        format!("{prefix}order.locality"),
        locality(&h.shelf_positions(), &picked),
    );
}

fn gold_replicate(
    h: &Hier,
    gold: &[Vec<Option<usize>>; 2],
    items: &[usize],
    push: &mut dyn FnMut(String, f64),
) {
    let [fine, coarse] = gold;
    let sub = Hier::from_parents(
        &h.parent,
        h.code.clone(),
        items.iter().map(|&i| h.item_class[i]).collect(),
    );
    let gf: Vec<_> = items.iter().map(|&i| fine[i]).collect();
    let gc: Vec<_> = items.iter().map(|&i| coarse[i]).collect();
    let leaf: Vec<_> = sub.item_class.iter().map(|&c| Some(c)).collect();
    let tf = gold_tree(&sub, &gf);
    push("gold.dendrogram_purity_fine".into(), tf.dendrogram_purity);
    push("gold.hierarchical_f_fine".into(), tf.hierarchical_f);
    push(
        "gold.dendrogram_purity_coarse".into(),
        gold_tree(&sub, &gc).dendrogram_purity,
    );
    let p = compare(&gf, &leaf);
    push("gold.ari_fine".into(), p.ari);
    push("gold.v_measure_fine".into(), p.v_measure);
}

/// Replicates per score name, `ctx.boot` of each.
pub fn bootstrap(h: &Hier, ctx: &Context, t: &EdgeTable) -> BTreeMap<String, Vec<f64>> {
    let mut out = BTreeMap::<String, Vec<f64>>::new();
    let mut edge_rng = Rng::new(ctx.seed ^ 0xb007);
    let mut item_rng = Rng::new(ctx.seed ^ 0xb008);
    let mut knn_rng = Rng::new(ctx.seed ^ 0xb009);
    let knn = embedscore::knn_probe(ctx.graph, &h.labels_at(1));
    let n = h.items();
    let mut push = |k: String, v: f64| out.entry(k).or_default().push(v);
    for _ in 0..ctx.boot {
        let pick = resample(t.weight.len(), &mut edge_rng);
        graph_replicate(h, t, &pick, "", &mut push);
        let items = resample(n, &mut item_rng);
        let mut multiplicity = vec![0; n];
        for &i in &items {
            multiplicity[i] += 1;
        }
        graph_replicate(h, t, &vertex_pick(t, &multiplicity), "vertex.", &mut push);
        let kp = resample(knn.len(), &mut knn_rng);
        push("probe.knn_d1".into(), mean(&knn, &kp));
        if let Some(gold) = &ctx.gold {
            gold_replicate(h, gold, &items, &mut push);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_pick_repeats_by_multiplicity_product() {
        let t = EdgeTable {
            left: vec![0, 1, 0],
            right: vec![1, 2, 2],
            weight: vec![1.0; 3],
            ..EdgeTable::default()
        };
        // Item 0 twice, item 1 once, item 2 never.
        assert_eq!(vertex_pick(&t, &[2, 1, 0]), [0, 0]);
        assert_eq!(vertex_pick(&t, &[1, 1, 1]), [0, 1, 2]);
    }
}
