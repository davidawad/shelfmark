//! Expected search cost of a scheme against optimal-code bounds, and a
//! two-part MDL code length.
//!
//! Terminal slots are the shelves items rest on: a class's own items (one
//! slot per class that holds items directly) and the root's unplaced items.
//! The scheme's search tree replaces every node's options (its own-items
//! slot, if any, then its subclasses holding items, in shelf order) by a
//! balanced binary tree, so its cost is achievable by binary questions in
//! shelf order. The bounds over the same slot weights are entropy `H`,
//! Huffman (1952), the optimum without order, and Hu–Tucker (1971), the
//! optimum preserving shelf order, computed by Garsia–Wachs (1977).
//! `H <= Huffman <= Hu–Tucker <= scheme` always holds.
use crate::Hier;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct SearchCost {
    /// Expected binary questions per item in the scheme's balanced tree.
    pub scheme_bits: f64,
    pub entropy_bits: f64,
    pub huffman_bits: f64,
    pub hu_tucker_bits: f64,
    /// Expected questions on the deepest item's path.
    pub worst_bits: f64,
    pub slots: usize,
}

/// Depth of each of `m` options in a balanced binary tree (halves split
/// ceil/floor, left first).
fn balanced_depths(m: usize) -> Vec<usize> {
    fn go(m: usize, d: usize, out: &mut Vec<usize>) {
        if m <= 1 {
            out.push(d);
        } else {
            go(m.div_ceil(2), d + 1, out);
            go(m / 2, d + 1, out);
        }
    }
    let mut out = Vec::with_capacity(m);
    if m > 0 {
        go(m, 0, &mut out);
    }
    out
}

/// Minimum expected codeword length (sum of merged weights), integer weights.
pub fn huffman_cost(weights: &[u64]) -> u64 {
    let mut heap: BinaryHeap<Reverse<u64>> = weights.iter().map(|&w| Reverse(w)).collect();
    let mut cost = 0;
    while heap.len() > 1 {
        let Reverse(a) = heap.pop().unwrap_or(Reverse(0));
        let Reverse(b) = heap.pop().unwrap_or(Reverse(0));
        cost += a + b;
        heap.push(Reverse(a + b));
    }
    cost
}

/// Optimal alphabetic (order-preserving) tree cost by Garsia–Wachs, with
/// infinite sentinels at both ends: merge the leftmost pair `(w[a], w[a+1])`
/// with `w[a] <= w[a+2]`, then reinsert the sum just right of the nearest
/// weight to its left that is at least as large. The total of merged weights
/// equals the Hu–Tucker optimum. O(n²).
pub fn hu_tucker_cost(weights: &[u64]) -> u64 {
    let mut w: Vec<u64> = Vec::with_capacity(weights.len() + 2);
    w.push(u64::MAX);
    w.extend_from_slice(weights);
    w.push(u64::MAX);
    let mut cost = 0;
    while w.len() > 3 {
        let last = w.len() - 3;
        let a = (1..=last).find(|&a| w[a] <= w[a + 2]).unwrap_or(last);
        let x = w[a] + w[a + 1];
        cost += x;
        w.drain(a..=a + 1);
        let j = (0..a).rev().find(|&j| w[j] >= x).unwrap_or(0);
        w.insert(j + 1, x);
    }
    cost
}

pub fn search_cost(h: &Hier) -> SearchCost {
    // Options of each node: own-items slot (if any) then subclasses with items.
    let option_bits: Vec<Vec<usize>> = (0..h.len())
        .map(|v| {
            let own = usize::from(!h.direct[v].is_empty());
            let kids = h.children[v].iter().filter(|&&c| h.subtree[c] > 0).count();
            balanced_depths(own + kids)
        })
        .collect();
    // Bits to reach node v's own slot = path questions + own-slot choice at v.
    let mut reach = vec![0usize; h.len()];
    let mut slot_bits = Vec::new();
    let mut weights = Vec::new();
    for v in 0..h.len() {
        if v > 0 {
            let p = h.parent[v];
            let own = usize::from(!h.direct[p].is_empty());
            let rank = h.children[p]
                .iter()
                .filter(|&&c| h.subtree[c] > 0)
                .position(|&c| c == v);
            if let Some(r) = rank {
                reach[v] = reach[p] + option_bits[p][own + r];
            }
        }
        if !h.direct[v].is_empty() {
            slot_bits.push(reach[v] + option_bits[v][0]);
            weights.push(h.direct[v].len() as u64);
        }
    }
    let total = weights.iter().sum::<u64>().max(1) as f64;
    let scheme_bits = slot_bits
        .iter()
        .zip(&weights)
        .map(|(&b, &w)| b as f64 * w as f64)
        .sum::<f64>()
        / total;
    let entropy_bits = -weights
        .iter()
        .map(|&w| w as f64 / total)
        .map(|p| p * p.log2())
        .sum::<f64>();
    SearchCost {
        scheme_bits,
        entropy_bits,
        huffman_bits: huffman_cost(&weights) as f64 / total,
        hu_tucker_bits: hu_tucker_cost(&weights) as f64 / total,
        worst_bits: slot_bits.iter().copied().max().unwrap_or(0) as f64,
        slots: weights.len(),
    }
}

/// Two-part code length in bits per item: the scheme (one parent pointer
/// per class, `log2(classes + 1)` bits each, the root included as a target)
/// plus the items' slots coded at their empirical entropy.
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct Mdl {
    pub model_bits: f64,
    pub data_bits_per_item: f64,
    pub total_bits_per_item: f64,
}

pub fn mdl(h: &Hier, scheme_classes: usize) -> Mdl {
    let n = h.items().max(1) as f64;
    let model_bits = scheme_classes as f64 * ((scheme_classes + 1) as f64).log2();
    let data = -h
        .direct
        .iter()
        .filter(|d| !d.is_empty())
        .map(|d| d.len() as f64 / n)
        .map(|p| p * p.log2())
        .sum::<f64>();
    Mdl {
        model_bits,
        data_bits_per_item: data,
        total_bits_per_item: data + model_bits / n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hier::tests::toy;

    #[test]
    fn huffman_and_hu_tucker_hand_checked() {
        assert_eq!(huffman_cost(&[1, 1, 1, 1]), 8);
        assert_eq!(hu_tucker_cost(&[1, 1, 1, 1]), 8);
        assert_eq!(hu_tucker_cost(&[1, 2, 3]), 9);
        // Order matters: [1,10,1] needs 23 alphabetically, Huffman gets 14.
        assert_eq!(hu_tucker_cost(&[1, 10, 1]), 23);
        assert_eq!(huffman_cost(&[1, 10, 1]), 14);
        assert_eq!(hu_tucker_cost(&[5]), 0);
    }

    #[test]
    fn hu_tucker_matches_brute_force_on_small_sequences() {
        fn brute(w: &[u64]) -> u64 {
            if w.len() <= 1 {
                return 0;
            }
            let total: u64 = w.iter().sum();
            (1..w.len())
                .map(|c| brute(&w[..c]) + brute(&w[c..]))
                .min()
                .unwrap_or(0)
                + total
        }
        let mut rng = crate::treescore::Rng::new(5);
        for len in 1..9 {
            for _ in 0..30 {
                let w: Vec<u64> = (0..len).map(|_| 1 + rng.below(20) as u64).collect();
                assert_eq!(hu_tucker_cost(&w), brute(&w), "{w:?}");
            }
        }
    }

    #[test]
    fn toy_search_cost() {
        // Root options: own slot (item 4), A, B -> depths [1,2,1]... balanced(3) = [2,2,1].
        // A options: A1, A2 -> [1,1]; A1 own slot only -> [0].
        // Slots: root-own 2 bits (w1), A1 2+1+0 = 3 (w2), A2 3 (w1), B 1+0 = 1 (w1).
        let s = search_cost(&toy());
        assert_eq!(s.slots, 4);
        assert!((s.scheme_bits - (2.0 + 6.0 + 3.0 + 1.0) / 5.0).abs() < 1e-12);
        assert_eq!(s.worst_bits, 3.0);
        // Weights in shelf order [1 (root), 2 (A1), 1 (A2), 1 (B)].
        assert_eq!(hu_tucker_cost(&[1, 2, 1, 1]), 10);
        assert!((s.hu_tucker_bits - 2.0).abs() < 1e-12);
        assert!(s.entropy_bits <= s.huffman_bits && s.huffman_bits <= s.hu_tucker_bits);
        assert!(s.hu_tucker_bits <= s.scheme_bits);
    }

    #[test]
    fn mdl_hand_checked() {
        // Four slots with weights 1,2,1,1 out of 5 items; 3 scheme classes.
        let m = mdl(&toy(), 3);
        assert!((m.model_bits - 3.0 * 2.0).abs() < 1e-12);
        let h = -(3.0 * 0.2 * 0.2f64.log2() + 0.4 * 0.4f64.log2());
        assert!((m.data_bits_per_item - h).abs() < 1e-12);
        assert!((m.total_bits_per_item - (h + 6.0 / 5.0)).abs() < 1e-12);
    }
}
