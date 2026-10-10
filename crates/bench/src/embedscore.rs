//! Embedding-space scores of a scheme's classes at a given depth.
//!
//! * Silhouette (Rousseeuw 1987), cosine distance, on a fixed item sample;
//!   singleton clusters score 0.
//! * Davies–Bouldin (1979), Euclidean, over all labelled items.
//! * Label predictability: leave-one-out weighted kNN vote over the
//!   similarity graph, and a nearest-centroid linear probe (5 folds by item
//!   index), each next to the majority-class baseline.
//! * Cophenetic correlation (Sokal & Rohlf 1962): Pearson correlation
//!   between cosine similarity and LCA depth over all pairs of the sample.
use crate::{Hier, SimilarityGraph};
use std::collections::BTreeMap;

/// Row-major `n x dim` unit vectors.
#[derive(Debug, Clone)]
pub struct Vectors {
    pub dim: usize,
    pub data: Vec<f32>,
}

impl Vectors {
    pub fn row(&self, i: usize) -> &[f32] {
        &self.data[i * self.dim..(i + 1) * self.dim]
    }
    pub fn len(&self) -> usize {
        self.data.len() / self.dim.max(1)
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

fn dot(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| f64::from(*x) * f64::from(*y))
        .sum()
}

/// Cosine similarity matrix of a sample of unit vectors.
pub fn sample_similarity(v: &Vectors, sample: &[usize]) -> Vec<f64> {
    let s = sample.len();
    let mut m = vec![1.0; s * s];
    for a in 0..s {
        for b in a + 1..s {
            let x = dot(v.row(sample[a]), v.row(sample[b]));
            m[a * s + b] = x;
            m[b * s + a] = x;
        }
    }
    m
}

/// Mean silhouette over sampled items with a label (cosine distance).
pub fn silhouette(sim: &[f64], sample: &[usize], labels: &[Option<usize>]) -> f64 {
    let s = sample.len();
    let lab: Vec<Option<usize>> = sample.iter().map(|&i| labels[i]).collect();
    let mut total = 0.0;
    let mut count = 0usize;
    for a in 0..s {
        let Some(la) = lab[a] else { continue };
        let mut sums = BTreeMap::<usize, (f64, usize)>::new();
        for b in 0..s {
            if let Some(lb) = lab[b] {
                if b != a {
                    let e = sums.entry(lb).or_default();
                    e.0 += 1.0 - sim[a * s + b];
                    e.1 += 1;
                }
            }
        }
        count += 1;
        let Some(&(own, k)) = sums.get(&la) else {
            continue;
        };
        if k == 0 {
            continue;
        }
        let inner = own / k as f64;
        let outer = sums
            .iter()
            .filter(|(l, _)| **l != la)
            .map(|(_, &(d, c))| d / c as f64)
            .fold(f64::INFINITY, f64::min);
        if outer.is_finite() {
            total += (outer - inner) / inner.max(outer).max(1e-12);
        }
    }
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

fn centroids(
    v: &Vectors,
    labels: &[Option<usize>],
    skip_fold: Option<usize>,
) -> BTreeMap<usize, (Vec<f64>, usize)> {
    let mut c = BTreeMap::<usize, (Vec<f64>, usize)>::new();
    for (i, l) in labels.iter().enumerate() {
        let Some(l) = l else { continue };
        if skip_fold == Some(i % 5) {
            continue;
        }
        let e = c.entry(*l).or_insert_with(|| (vec![0.0; v.dim], 0));
        for (x, y) in e.0.iter_mut().zip(v.row(i)) {
            *x += f64::from(*y);
        }
        e.1 += 1;
    }
    for (sum, k) in c.values_mut() {
        sum.iter_mut().for_each(|x| *x /= *k as f64);
    }
    c
}

fn dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

pub fn davies_bouldin(v: &Vectors, labels: &[Option<usize>]) -> f64 {
    let c = centroids(v, labels, None);
    if c.len() < 2 {
        return 0.0;
    }
    let mut scatter = BTreeMap::<usize, f64>::new();
    for (i, l) in labels.iter().enumerate() {
        if let Some(l) = l {
            let row: Vec<f64> = v.row(i).iter().map(|&x| f64::from(x)).collect();
            *scatter.entry(*l).or_default() += dist(&row, &c[l].0) / c[l].1 as f64;
        }
    }
    let keys: Vec<usize> = c.keys().copied().collect();
    let mut total = 0.0;
    for &a in &keys {
        let mut worst: f64 = 0.0;
        for &b in &keys {
            if a != b {
                let d = dist(&c[&a].0, &c[&b].0).max(1e-12);
                worst = worst.max((scatter[&a] + scatter[&b]) / d);
            }
        }
        total += worst;
    }
    total / keys.len() as f64
}

/// Share of labelled items whose label is the most common one.
pub fn majority_share(labels: &[Option<usize>]) -> f64 {
    let mut counts = BTreeMap::<usize, usize>::new();
    for l in labels.iter().flatten() {
        *counts.entry(*l).or_default() += 1;
    }
    let total = counts.values().sum::<usize>();
    if total == 0 {
        0.0
    } else {
        counts.values().copied().max().unwrap_or(0) as f64 / total as f64
    }
}

/// Leave-one-out weighted vote of graph neighbours; returns one outcome
/// (1 correct, 0 wrong) per labelled item, in item order.
pub fn knn_probe(graph: &SimilarityGraph, labels: &[Option<usize>]) -> Vec<f64> {
    let mut votes = vec![BTreeMap::<usize, f64>::new(); labels.len()];
    for e in &graph.edges {
        if e.left >= labels.len() || e.right >= labels.len() {
            continue;
        }
        if let Some(l) = labels[e.right] {
            *votes[e.left].entry(l).or_default() += e.weight;
        }
        if let Some(l) = labels[e.left] {
            *votes[e.right].entry(l).or_default() += e.weight;
        }
    }
    labels
        .iter()
        .zip(&votes)
        .filter_map(|(l, v)| {
            let l = (*l)?;
            // Highest vote; ties go to the smallest label.
            let best = v
                .iter()
                .fold(None, |best: Option<(usize, f64)>, (&k, &w)| match best {
                    Some((_, bw)) if bw >= w => best,
                    _ => Some((k, w)),
                });
            Some(f64::from(u8::from(best.map(|b| b.0) == Some(l))))
        })
        .collect()
}

/// Nearest-centroid (cosine) probe with 5 folds by item index; one outcome
/// per labelled item.
pub fn centroid_probe(v: &Vectors, labels: &[Option<usize>]) -> Vec<f64> {
    let folds: Vec<_> = (0..5).map(|f| centroids(v, labels, Some(f))).collect();
    labels
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            let l = (*l)?;
            let row = v.row(i);
            let mut best = (f64::NEG_INFINITY, usize::MAX);
            for (&k, (c, _)) in &folds[i % 5] {
                let norm = c.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
                let s = row
                    .iter()
                    .zip(c)
                    .map(|(x, y)| f64::from(*x) * y)
                    .sum::<f64>()
                    / norm;
                if s > best.0 {
                    best = (s, k);
                }
            }
            Some(f64::from(u8::from(best.1 == l)))
        })
        .collect()
}

/// Pearson correlation between cosine similarity and LCA depth over all
/// pairs of the sample.
pub fn cophenetic(h: &Hier, sim: &[f64], sample: &[usize]) -> f64 {
    let s = sample.len();
    let (mut sx, mut sy, mut sxx, mut syy, mut sxy, mut n) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for a in 0..s {
        for b in a + 1..s {
            let x = sim[a * s + b];
            let y = h.depth[h.lca(h.item_class[sample[a]], h.item_class[sample[b]])] as f64;
            sx += x;
            sy += y;
            sxx += x * x;
            syy += y * y;
            sxy += x * y;
            n += 1.0;
        }
    }
    let cov = sxy / n - sx / n * sy / n;
    let var = (sxx / n - (sx / n).powi(2)) * (syy / n - (sy / n).powi(2));
    if var > 0.0 {
        cov / var.sqrt()
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Edge;

    /// Two tight clusters on the unit circle: items 0,1 near (1,0), 2,3 near (0,1).
    fn vectors() -> Vectors {
        let a = 0.1f32;
        let (c, s) = (a.cos(), a.sin());
        Vectors {
            dim: 2,
            data: vec![1.0, 0.0, c, s, 0.0, 1.0, s, c],
        }
    }

    #[test]
    fn separated_clusters_score_well() {
        let v = vectors();
        let labels = vec![Some(0), Some(0), Some(1), Some(1)];
        let sample = [0, 1, 2, 3];
        let sim = sample_similarity(&v, &sample);
        let good = silhouette(&sim, &sample, &labels);
        let bad = silhouette(&sim, &sample, &[Some(0), Some(1), Some(0), Some(1)]);
        assert!(good > 0.9 && bad < 0.0, "{good} {bad}");
        // DB: scatter = half the within-pair chord, centroids ~ sqrt(2) apart.
        let db = davies_bouldin(&v, &labels);
        let chord = 2.0 * (0.05f64).sin();
        assert!(db > 0.0 && db < chord, "{db}");
        assert_eq!(centroid_probe(&v, &labels), [1.0; 4]);
        assert_eq!(majority_share(&labels), 0.5);
    }

    #[test]
    fn knn_probe_votes_by_weight() {
        let g = SimilarityGraph {
            n: 3,
            k: 2,
            tau: 0.0,
            edges: vec![
                Edge {
                    left: 0,
                    right: 1,
                    weight: 2.0,
                },
                Edge {
                    left: 0,
                    right: 2,
                    weight: 1.0,
                },
            ],
        };
        // Item 0 (label 5): neighbours vote 5 (w2) vs 6 (w1) -> correct.
        // Item 1 (label 5): only neighbour 0 votes 5 -> correct. Item 2 (6): votes 5 -> wrong.
        assert_eq!(knn_probe(&g, &[Some(5), Some(5), Some(6)]), [1.0, 1.0, 0.0]);
    }

    #[test]
    fn cophenetic_tracks_shared_depth() {
        let h = Hier::from_parents(
            &[0, 0, 0],
            ["", "A", "B"].map(String::from).to_vec(),
            vec![1, 1, 2, 2],
        );
        let v = vectors();
        let sample = [0, 1, 2, 3];
        let r = cophenetic(&h, &sample_similarity(&v, &sample), &sample);
        assert!(r > 0.99, "{r}");
    }
}
