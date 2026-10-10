//! Shelf-order scores. Placed items get a linear shelf position (classes in
//! pre-order, a class's own items before its subclasses).
//!
//! * Order locality: `1 - mean |pos_i - pos_j| / E|U - V|` over similarity
//!   edges between placed items (weighted), where `E|U - V| = (m² - 1)/3m`
//!   for two uniform positions among `m`. 0 = random shelving, 1 = every
//!   neighbour adjacent.
//! * Kendall τ-b (Kendall 1938) between similarity and shelf proximity over
//!   graph edges plus as many sampled random pairs (similarity 0).
//! * Author collocation: share of same-author pairs shelved in the same
//!   class, and their shelf locality (Cutter's collocation objective, with
//!   shared author names as the only available co-work signal).
//! * Insertion: items arrive in corpus order; a class holding more than
//!   `capacity` own items must be subdivided each time its count passes a
//!   multiple of the capacity, re-shelving every item it holds.
use crate::treescore::Rng;
use crate::Hier;
use std::collections::BTreeMap;

fn expected_random_gap(m: usize) -> f64 {
    let m = m as f64;
    if m > 0.0 {
        (m * m - 1.0) / (3.0 * m)
    } else {
        0.0
    }
}

/// Weighted locality over `(i, j, w)` pairs; pairs with an unplaced item
/// are skipped.
pub fn locality(pos: &[Option<usize>], pairs: &[(usize, usize, f64)]) -> f64 {
    let m = pos.iter().flatten().count();
    let (mut num, mut den) = (0.0, 0.0);
    for &(i, j, w) in pairs {
        if let (Some(a), Some(b)) = (pos[i], pos[j]) {
            num += w * a.abs_diff(b) as f64;
            den += w;
        }
    }
    let random = expected_random_gap(m);
    if den > 0.0 && random > 0.0 {
        1.0 - num / den / random
    } else {
        0.0
    }
}

/// Kendall τ-b between two equal-length samples, O(n²).
pub fn kendall_tau_b(x: &[f64], y: &[f64]) -> f64 {
    let (mut conc, mut disc, mut tx, mut ty) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for a in 0..x.len() {
        for b in a + 1..x.len() {
            let dx = x[a] - x[b];
            let dy = y[a] - y[b];
            if dx == 0.0 && dy == 0.0 {
                continue;
            } else if dx == 0.0 {
                tx += 1.0;
            } else if dy == 0.0 {
                ty += 1.0;
            } else if (dx > 0.0) == (dy > 0.0) {
                conc += 1.0;
            } else {
                disc += 1.0;
            }
        }
    }
    let den = ((conc + disc + tx) * (conc + disc + ty)).sqrt();
    if den > 0.0 {
        (conc - disc) / den
    } else {
        0.0
    }
}

/// τ-b between edge similarity and negative shelf distance, over up to
/// `max` sampled edges between placed items plus as many random placed pairs.
pub fn shelf_tau(
    pos: &[Option<usize>],
    edges: &[(usize, usize, f64)],
    max: usize,
    seed: u64,
) -> f64 {
    let placed: Vec<usize> = (0..pos.len()).filter(|&i| pos[i].is_some()).collect();
    let usable: Vec<_> = edges
        .iter()
        .filter(|e| pos[e.0].is_some() && pos[e.1].is_some())
        .collect();
    if usable.is_empty() || placed.len() < 2 {
        return 0.0;
    }
    let mut rng = Rng::new(seed);
    let (mut sim, mut close) = (vec![], vec![]);
    for _ in 0..max.min(usable.len()) {
        let e = usable[rng.below(usable.len())];
        sim.push(e.2);
        close.push(-(pos[e.0].unwrap_or(0).abs_diff(pos[e.1].unwrap_or(0)) as f64));
        let (a, b) = (
            placed[rng.below(placed.len())],
            placed[rng.below(placed.len())],
        );
        sim.push(0.0);
        close.push(-(pos[a].unwrap_or(0).abs_diff(pos[b].unwrap_or(0)) as f64));
    }
    kendall_tau_b(&sim, &close)
}

/// Pairs of items sharing an exact author name (authors with more than
/// `cap` items are skipped as likely name collisions).
pub fn author_pairs(authors: &[Vec<String>], cap: usize) -> Vec<(usize, usize, f64)> {
    let mut by = BTreeMap::<&str, Vec<usize>>::new();
    for (i, list) in authors.iter().enumerate() {
        for a in list {
            by.entry(a.as_str()).or_default().push(i);
        }
    }
    let mut out = BTreeMap::new();
    for items in by.values().filter(|v| v.len() > 1 && v.len() <= cap) {
        for (x, &a) in items.iter().enumerate() {
            for &b in &items[x + 1..] {
                if a != b {
                    out.insert((a.min(b), a.max(b)), 1.0);
                }
            }
        }
    }
    out.into_iter().map(|((a, b), w)| (a, b, w)).collect()
}

/// Share of pairs (both placed) in the same class, and the same share for
/// two independent items drawn from the class distribution.
pub fn same_class(h: &Hier, pairs: &[(usize, usize, f64)]) -> (f64, f64, usize) {
    let both: Vec<_> = pairs
        .iter()
        .filter(|p| h.placed(p.0) && h.placed(p.1))
        .collect();
    let same = both
        .iter()
        .filter(|p| h.item_class[p.0] == h.item_class[p.1])
        .count();
    let placed = h.item_class.iter().filter(|&&c| c != 0).count().max(1) as f64;
    let chance = h
        .direct
        .iter()
        .skip(1)
        .map(|d| (d.len() as f64 / placed).powi(2))
        .sum::<f64>();
    let share = if both.is_empty() {
        0.0
    } else {
        same as f64 / both.len() as f64
    };
    (share, chance, both.len())
}

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct Insertion {
    pub splits_per_1000: f64,
    pub churn_per_insert: f64,
}

pub fn insertion(h: &Hier, capacity: usize) -> Insertion {
    let mut held = vec![0usize; h.len()];
    let (mut splits, mut churn, mut inserted) = (0usize, 0usize, 0usize);
    for &c in h.item_class.iter().filter(|&&c| c != 0) {
        held[c] += 1;
        inserted += 1;
        if held[c] > capacity && (held[c] - 1) % capacity == 0 {
            splits += 1;
            churn += held[c];
        }
    }
    let n = inserted.max(1) as f64;
    Insertion {
        splits_per_1000: splits as f64 * 1000.0 / n,
        churn_per_insert: churn as f64 / n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hier::tests::toy;

    #[test]
    fn locality_hand_checked() {
        // Positions 0..3 (m = 4): E|U-V| = 15/12 = 1.25. One adjacent pair: 1 - 1/1.25 = 0.2.
        let pos = toy().shelf_positions();
        assert!((locality(&pos, &[(0, 1, 1.0)]) - 0.2).abs() < 1e-12);
        // Pairs with the unplaced item 4 are ignored.
        assert!((locality(&pos, &[(0, 1, 1.0), (0, 4, 5.0)]) - 0.2).abs() < 1e-12);
        assert!((locality(&pos, &[(0, 3, 1.0)]) - (1.0 - 3.0 / 1.25)).abs() < 1e-12);
    }

    #[test]
    fn kendall_hand_checked() {
        assert!((kendall_tau_b(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]) - 1.0).abs() < 1e-12);
        assert!((kendall_tau_b(&[1.0, 2.0, 3.0], &[3.0, 2.0, 1.0]) + 1.0).abs() < 1e-12);
        // x = [1,1,2], y = [1,2,3]: pairs (tie-x), (C), (C): (2-0)/sqrt(2*3).
        let t = kendall_tau_b(&[1.0, 1.0, 2.0], &[1.0, 2.0, 3.0]);
        assert!((t - 2.0 / 6f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn authors_and_same_class() {
        let authors = vec![
            vec!["a".into()],
            vec!["a".into(), "b".into()],
            vec!["b".into()],
            vec![],
            vec![],
        ];
        let p = author_pairs(&authors, 10);
        assert_eq!(p, [(0, 1, 1.0), (1, 2, 1.0)]);
        // toy: 0,1 in A1 (same), 1 in A1 and 2 in A2 (different).
        let (share, chance, n) = same_class(&toy(), &p);
        assert_eq!((share, n), (0.5, 2));
        // Classes A1 (2), A2 (1), B (1) of 4 placed: 0.25 + 1/16 + 1/16.
        assert!((chance - 0.375).abs() < 1e-12);
    }

    #[test]
    fn insertion_counts_capacity_splits() {
        // Five items in one class, capacity 2: splits at the 3rd and 5th item.
        let h = Hier::from_parents(&[0, 0], ["", "A"].map(String::from).to_vec(), vec![1; 5]);
        let r = insertion(&h, 2);
        assert!((r.splits_per_1000 - 400.0).abs() < 1e-9);
        assert!((r.churn_per_insert - 8.0 / 5.0).abs() < 1e-12);
    }
}
