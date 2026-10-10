//! Null models, placement noise and bootstrap resampling for the catalogue.
//!
//! * Shuffled labels: placed items' classes are permuted among the placed
//!   items (class sizes and tree kept, the item-to-class link broken).
//! * Random shape: at every depth the class nodes are reassigned to the
//!   same multiset of parent slots in random order (level sizes and fan-outs
//!   kept, the semantic nesting and shelf order broken); items stay with
//!   their class.
//! * Noise `p`: each placed item moves, with probability `p`, to the class
//!   of a uniformly drawn placed item (the empirical class distribution).
use crate::treescore::Rng;
use crate::Hier;

fn shuffle<T>(v: &mut [T], rng: &mut Rng) {
    for i in (1..v.len()).rev() {
        v.swap(i, rng.below(i + 1));
    }
}

fn rebuild(h: &Hier, item_class: Vec<usize>) -> Hier {
    Hier::from_parents(&h.parent, h.code.clone(), item_class)
}

pub fn shuffled_labels(h: &Hier, rng: &mut Rng) -> Hier {
    let placed: Vec<usize> = (0..h.items()).filter(|&i| h.placed(i)).collect();
    let mut classes: Vec<usize> = placed.iter().map(|&i| h.item_class[i]).collect();
    shuffle(&mut classes, rng);
    let mut item_class = h.item_class.clone();
    for (&i, c) in placed.iter().zip(classes) {
        item_class[i] = c;
    }
    rebuild(h, item_class)
}

pub fn random_shape(h: &Hier, rng: &mut Rng) -> Hier {
    let mut parent = h.parent.clone();
    for d in 1..=h.max_depth() {
        let nodes: Vec<usize> = (1..h.len()).filter(|&v| h.depth[v] == d).collect();
        let mut slots: Vec<usize> = nodes.iter().map(|&v| h.parent[v]).collect();
        shuffle(&mut slots, rng);
        for (&v, p) in nodes.iter().zip(slots) {
            parent[v] = p;
        }
    }
    Hier::from_parents(&parent, h.code.clone(), h.item_class.clone())
}

pub fn noisy(h: &Hier, p: f64, rng: &mut Rng) -> Hier {
    let placed: Vec<usize> = (0..h.items()).filter(|&i| h.placed(i)).collect();
    let mut item_class = h.item_class.clone();
    if placed.is_empty() {
        return rebuild(h, item_class);
    }
    for &i in &placed {
        if rng.unit() < p {
            item_class[i] = h.item_class[placed[rng.below(placed.len())]];
        }
    }
    rebuild(h, item_class)
}

/// `n` indices drawn with replacement from `0..n`.
pub fn resample(n: usize, rng: &mut Rng) -> Vec<usize> {
    (0..n).map(|_| rng.below(n.max(1))).collect()
}

/// Percentile interval (2.5%, 97.5%) of bootstrap values.
pub fn interval(values: &[f64]) -> [f64; 2] {
    let mut v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return [f64::NAN, f64::NAN];
    }
    v.sort_by(f64::total_cmp);
    let at = |q: f64| v[((v.len() - 1) as f64 * q).round() as usize];
    [at(0.025), at(0.975)]
}

pub fn mean_sd(values: &[f64]) -> (f64, f64) {
    let n = values.len().max(1) as f64;
    let m = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    (m, var.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hier::tests::toy;

    #[test]
    fn shuffled_labels_keep_class_sizes() {
        let h = toy();
        let s = shuffled_labels(&h, &mut Rng::new(3));
        let mut a: Vec<_> = h.direct.iter().map(Vec::len).collect();
        let mut b: Vec<_> = s.direct.iter().map(Vec::len).collect();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b);
        assert_eq!(s.item_class[4], 0, "unplaced item stays unplaced");
    }

    #[test]
    fn random_shape_keeps_level_sizes_and_fanouts() {
        let h = toy();
        for seed in 0..10 {
            let s = random_shape(&h, &mut Rng::new(seed));
            let level = |x: &Hier, d| (1..x.len()).filter(|&v| x.depth[v] == d).count();
            assert_eq!((level(&s, 1), level(&s, 2)), (2, 2));
            let mut fan: Vec<_> = s.children.iter().map(Vec::len).collect();
            fan.sort_unstable();
            assert_eq!(fan, [0, 0, 0, 2, 2]);
            assert_eq!(s.subtree[0], 5);
        }
    }

    #[test]
    fn noise_zero_is_identity_and_interval_is_ordered() {
        let h = toy();
        assert_eq!(noisy(&h, 0.0, &mut Rng::new(1)), h);
        let [lo, hi] = interval(&[3.0, 1.0, 2.0, 5.0, 4.0]);
        assert!(lo <= hi);
        let (m, sd) = mean_sd(&[1.0, 3.0]);
        assert_eq!(m, 2.0);
        assert!((sd - 2f64.sqrt()).abs() < 1e-12);
    }
}
