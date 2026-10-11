//! Dasgupta cost on the dense similarity matrix against its kNN
//! sparsification (DESIGN.md section 1: "validate on n ≤ 5k by comparing
//! the sparse cost to the dense cost").
//!
//! On a subset of items with unit vectors, every pair gets the benchmark
//! weight `w_ij = max(0, cos_ij − τ)` with the corpus graph's τ. The dense
//! cost sums over all pairs with `w > 0`; the kNN graph keeps a pair when one
//! endpoint is among the other's `k` most similar items (similarity
//! descending, index ascending), as `build_graph_fastembed.py` does, so its
//! weights are the dense weights of the retained pairs. Costs are
//! normalised as `Σ w·|L(lca)| / (n Σ w)`.
//!
//! Bootstrap: each replicate weights every pair by an independent Poisson(1)
//! count, a hash of (seed, replicate, pair). Dense and sparse costs of every
//! scheme in a replicate use the same counts, so their rankings are paired.
use crate::embedscore::Vectors;
use crate::Hier;

/// Index of pair `i < j` in the row-major upper triangle of an `n`-item set.
pub fn pair_index(n: usize, i: usize, j: usize) -> usize {
    i * n - i * (i + 1) / 2 + (j - i - 1)
}

fn dot(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| f64::from(*x) * f64::from(*y))
        .sum()
}

/// Runs `f(i)` for every row `0..n` on `threads` threads, rows interleaved,
/// and returns the results in row order.
fn rows_parallel<T: Send>(n: usize, threads: usize, f: impl Fn(usize) -> T + Sync) -> Vec<T> {
    let threads = threads.clamp(1, n.max(1));
    let mut parts: Vec<Vec<(usize, T)>> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let f = &f;
                s.spawn(move || (t..n).step_by(threads).map(|i| (i, f(i))).collect())
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("worker panicked"))
            .collect()
    });
    let mut out: Vec<(usize, T)> = parts.iter_mut().flat_map(std::mem::take).collect();
    out.sort_by_key(|x| x.0);
    out.into_iter().map(|x| x.1).collect()
}

/// Cosine similarities of all pairs `i < j` (unit vectors), upper triangle.
pub fn cosines(v: &Vectors, threads: usize) -> Vec<f32> {
    let n = v.len();
    rows_parallel(n, threads, |i| {
        (i + 1..n)
            .map(|j| dot(v.row(i), v.row(j)) as f32)
            .collect::<Vec<f32>>()
    })
    .concat()
}

/// The pairs with positive weight: pair indices (ascending) and weights.
pub struct Positive {
    pub pair: Vec<usize>,
    pub left: Vec<u32>,
    pub right: Vec<u32>,
    pub weight: Vec<f32>,
}

pub fn positive_pairs(cos: &[f32], n: usize, tau: f64) -> Positive {
    let mut p = Positive {
        pair: vec![],
        left: vec![],
        right: vec![],
        weight: vec![],
    };
    for i in 0..n {
        for j in i + 1..n {
            let x = pair_index(n, i, j);
            let w = f64::from(cos[x]) - tau;
            if w > 0.0 {
                p.pair.push(x);
                p.left.push(i as u32);
                p.right.push(j as u32);
                p.weight.push(w as f32);
            }
        }
    }
    p
}

/// Positions in `positive` of the symmetrised kNN graph's pairs, ascending.
pub fn knn_positions(cos: &[f32], n: usize, k: usize, positive: &Positive) -> Vec<usize> {
    let mut picked = std::collections::BTreeSet::new();
    let mut row: Vec<(f32, usize)> = Vec::with_capacity(n);
    for i in 0..n {
        row.clear();
        row.extend(
            (0..n)
                .filter(|&j| j != i)
                .map(|j| (cos[pair_index(n, i.min(j), i.max(j))], j)),
        );
        let order = |a: &(f32, usize), b: &(f32, usize)| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1));
        let take = k.min(row.len());
        if take < row.len() && take > 0 {
            row.select_nth_unstable_by(take - 1, order);
        }
        for &(_, j) in &row[..take] {
            picked.insert(pair_index(n, i.min(j), i.max(j)));
        }
    }
    picked
        .into_iter()
        .filter_map(|x| positive.pair.binary_search(&x).ok())
        .collect()
}

/// Items under the LCA of every positive pair, one row of `trees.len()`
/// values per pair.
pub fn lca_sizes(trees: &[Hier], positive: &Positive, threads: usize) -> Vec<u16> {
    let s = trees.len();
    let m = positive.pair.len();
    let chunk = m.div_ceil(threads.max(1) * 8).max(1);
    rows_parallel(m.div_ceil(chunk), threads, |c| {
        let mut out = Vec::with_capacity(chunk * s);
        for k in c * chunk..((c + 1) * chunk).min(m) {
            let (i, j) = (positive.left[k] as usize, positive.right[k] as usize);
            for h in trees {
                let size = h.subtree[h.lca(h.item_class[i], h.item_class[j])];
                out.push(u16::try_from(size).unwrap_or(u16::MAX));
            }
        }
        out
    })
    .concat()
}

/// Poisson(1) count from a hash of (seed, replicate, pair); replicate
/// `u64::MAX` is the point estimate (every count 1).
pub fn poisson_count(seed: u64, rep: u64, pair: usize) -> u32 {
    if rep == u64::MAX {
        return 1;
    }
    let mut z = seed
        .wrapping_add(rep.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add((pair as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    let u = (z >> 11) as f64 / (1u64 << 53) as f64;
    // Inverse CDF of Poisson(1).
    let (mut k, mut p, mut cdf) = (0u32, (-1.0f64).exp(), (-1.0f64).exp());
    while u > cdf && k < 20 {
        k += 1;
        p /= f64::from(k);
        cdf += p;
    }
    k
}

/// The positive pairs of one item set and every tree's LCA size per pair.
pub struct PairCosts<'a> {
    pub positive: &'a Positive,
    pub sizes: &'a [u16],
    pub trees: usize,
    pub n: usize,
}

impl PairCosts<'_> {
    /// Normalised cost of every tree over the selected positive pairs
    /// (`None` = all), for replicate `rep` ([`poisson_count`]).
    pub fn costs(&self, select: Option<&[usize]>, seed: u64, rep: u64) -> Vec<f64> {
        let mut num = vec![0.0; self.trees];
        let mut den = 0.0;
        let mut add = |k: usize| {
            let c = poisson_count(seed, rep, self.positive.pair[k]);
            if c == 0 {
                return;
            }
            let w = f64::from(self.positive.weight[k]) * f64::from(c);
            den += w;
            let row = &self.sizes[k * self.trees..(k + 1) * self.trees];
            for (v, &size) in num.iter_mut().zip(row) {
                *v += w * f64::from(size);
            }
        };
        match select {
            Some(list) => list.iter().for_each(|&k| add(k)),
            None => (0..self.positive.pair.len()).for_each(&mut add),
        }
        num.iter()
            .map(|v| {
                if den > 0.0 {
                    v / (self.n as f64 * den)
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// Replicates `0..reps` for several selections on `threads` threads:
    /// `out[rep][selection][tree]`.
    pub fn boot(
        &self,
        selections: &[Option<Vec<usize>>],
        seed: u64,
        reps: usize,
        threads: usize,
    ) -> Vec<Vec<Vec<f64>>> {
        rows_parallel(reps, threads, |b| {
            selections
                .iter()
                .map(|s| self.costs(s.as_deref(), seed, b as u64))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vectors() -> Vectors {
        // Two tight pairs (0,1) and (2,3) at right angles, plus item 4 between.
        let s = std::f32::consts::FRAC_1_SQRT_2;
        let rows = [[1.0, 0.0], [0.99, 0.141], [0.0, 1.0], [0.141, 0.99], [s, s]];
        Vectors {
            dim: 2,
            data: rows.iter().flatten().copied().collect(),
        }
    }

    #[test]
    fn pair_index_is_dense_upper_triangle() {
        let n = 5;
        let mut seen = vec![];
        for i in 0..n {
            for j in i + 1..n {
                seen.push(pair_index(n, i, j));
            }
        }
        assert_eq!(seen, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn dense_and_knn_costs_hand_checked() {
        let v = vectors();
        let cos = cosines(&v, 3);
        assert_eq!(cos.len(), 10);
        let pos = positive_pairs(&cos, 5, 0.5);
        // Positive: (0,1), (0,4), (1,4), (2,3), (2,4), (3,4).
        assert_eq!(pos.pair.len(), 6);
        let knn = knn_positions(&cos, 5, 1, &pos);
        // Nearest neighbours: 0↔1, 2↔3, 4→1 or 3 (tie broken by index) .
        assert_eq!(knn.len(), 3);
        // Tree: A{0,1}, B{2,3}, item 4 unplaced at the root.
        let h = Hier::from_parents(
            &[0, 0, 0],
            ["", "A", "B"].map(String::from).to_vec(),
            vec![1, 1, 2, 2, 0],
        );
        let sizes = lca_sizes(std::slice::from_ref(&h), &pos, 2);
        let pc = PairCosts {
            positive: &pos,
            sizes: &sizes,
            trees: 1,
            n: 5,
        };
        let sparse = pc.costs(Some(&knn), 0, u64::MAX)[0];
        // kNN keeps the two in-class pairs (size 2) and one root pair (5).
        let w = |k: usize| f64::from(pos.weight[k]);
        let (a, b) = (w(0), w(3));
        let c = w(knn[1]);
        let expect = (2.0 * a + 2.0 * b + 5.0 * c) / (5.0 * (a + b + c));
        assert!((sparse - expect).abs() < 1e-9);
        let dense = pc.costs(None, 0, u64::MAX)[0];
        let reps = pc.boot(&[None, Some(knn.clone())], 0, 4, 2);
        assert_eq!((reps.len(), reps[0].len()), (4, 2));
        assert!(reps.iter().all(|r| r[1][0] > 0.0 && r[1][0] <= 1.0));
        assert!(dense > sparse, "the root pairs weigh more in the dense sum");
    }

    #[test]
    fn poisson_counts_have_mean_one() {
        let n = 200_000;
        let total: u64 = (0..n).map(|p| u64::from(poisson_count(7, 3, p))).sum();
        let mean = total as f64 / n as f64;
        assert!((mean - 1.0).abs() < 0.01, "{mean}");
        assert_eq!(poisson_count(7, 3, 11), poisson_count(7, 3, 11));
        assert_eq!(poisson_count(7, u64::MAX, 11), 1);
    }
}
