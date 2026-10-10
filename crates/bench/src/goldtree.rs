//! Hierarchy-against-gold scores, over the items that carry a gold label.
//!
//! * Dendrogram purity (Heller & Ghahramani 2005): the mean, over pairs of
//!   items with the same gold class, of the share of that class among the
//!   gold-labelled items under the pair's LCA.
//! * Hierarchical F (Larsen & Aone 1999): for every gold class the best F1
//!   over all tree nodes (and single items), averaged with class-size
//!   weights.
use crate::Hier;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct GoldTree {
    pub dendrogram_purity: f64,
    pub hierarchical_f: f64,
    pub labelled: usize,
}

fn pairs(x: usize) -> f64 {
    x as f64 * (x as f64 - 1.0) / 2.0
}

pub fn gold_tree(h: &Hier, gold: &[Option<usize>]) -> GoldTree {
    let mut totals = BTreeMap::<usize, usize>::new();
    for g in gold.iter().flatten() {
        *totals.entry(*g).or_default() += 1;
    }
    let labelled = totals.values().sum::<usize>();
    if labelled == 0 {
        return GoldTree::default();
    }
    // Per-node gold counts, built bottom-up (children have larger indices).
    let mut counts: Vec<BTreeMap<usize, usize>> = vec![BTreeMap::new(); h.len()];
    for (item, g) in gold.iter().enumerate() {
        if let Some(g) = g {
            *counts[h.item_class[item]].entry(*g).or_default() += 1;
        }
    }
    let mut best_f: BTreeMap<usize, f64> = totals
        .iter()
        .map(|(&k, &n)| (k, 2.0 / (1.0 + n as f64)))
        .collect();
    let mut purity_sum = 0.0;
    for v in (0..h.len()).rev() {
        let mut own = std::mem::take(&mut counts[v]);
        let mut child_pairs = BTreeMap::<usize, f64>::new();
        for &c in &h.children[v] {
            for (&k, &x) in &counts[c] {
                *child_pairs.entry(k).or_default() += pairs(x);
                *own.entry(k).or_default() += x;
            }
            counts[c] = BTreeMap::new();
        }
        let size = own.values().sum::<usize>() as f64;
        for (&k, &x) in &own {
            let at_v = pairs(x) - child_pairs.get(&k).copied().unwrap_or(0.0);
            purity_sum += at_v * x as f64 / size;
            let f = 2.0 * x as f64 / (size + totals[&k] as f64);
            let slot = best_f.entry(k).or_default();
            *slot = slot.max(f);
        }
        counts[v] = own;
    }
    let total_pairs = totals.values().map(|&n| pairs(n)).sum::<f64>();
    GoldTree {
        dendrogram_purity: if total_pairs > 0.0 {
            purity_sum / total_pairs
        } else {
            0.0
        },
        hierarchical_f: totals
            .iter()
            .map(|(k, &n)| n as f64 / labelled as f64 * best_f[k])
            .sum(),
        labelled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hier::tests::toy;

    #[test]
    fn perfect_tree_scores_one() {
        // Gold matches A vs B exactly for items 0..3.
        let g = gold_tree(&toy(), &[Some(0), Some(0), Some(0), Some(1), None]);
        assert!((g.dendrogram_purity - 1.0).abs() < 1e-12);
        assert!((g.hierarchical_f - 1.0).abs() < 1e-12);
        assert_eq!(g.labelled, 4);
    }

    #[test]
    fn hand_checked_impure_tree() {
        // toy: A{A1{0,1},A2{2}}, B{3}, root{4}. Gold: 0,1 -> x; 2,3 -> y; 4 -> x.
        // x pairs: (0,1) LCA A1, purity 1; (0,4),(1,4) LCA root, purity 3/5.
        // y pair: (2,3) LCA root, purity 2/5. DP = (1 + 0.6 + 0.6 + 0.4)/4 = 0.65.
        let g = gold_tree(&toy(), &[Some(0), Some(0), Some(1), Some(1), Some(0)]);
        assert!((g.dendrogram_purity - 0.65).abs() < 1e-12);
        // Best F: x (3 items): A1 = 2*2/(2+3) = 0.8 vs root 2*3/(5+3) = 0.75 -> 0.8.
        // y (2 items): A2 or B (1 of 1) = 2/(1+2) = 2/3; root 2*2/(5+2) = 4/7 -> 2/3.
        let f = 0.6 * 0.8 + 0.4 * (2.0 / 3.0);
        assert!((g.hierarchical_f - f).abs() < 1e-12);
    }
}
