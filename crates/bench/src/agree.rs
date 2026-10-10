//! Rank agreement between scores across schemes: Spearman (1904) ρ and
//! Kendall (1938) τ-b, and an average-linkage ordering of scores by
//! `1 - |ρ|` for the clustered heatmap.
use crate::shelf::kendall_tau_b;

/// Ranks starting at 1, ties sharing their average rank.
pub fn ranks(x: &[f64]) -> Vec<f64> {
    let mut idx: Vec<usize> = (0..x.len()).collect();
    idx.sort_by(|&a, &b| x[a].total_cmp(&x[b]).then(a.cmp(&b)));
    let mut r = vec![0.0; x.len()];
    let mut i = 0;
    while i < idx.len() {
        let mut j = i;
        while j + 1 < idx.len() && x[idx[j + 1]] == x[idx[i]] {
            j += 1;
        }
        let avg = (i + j) as f64 / 2.0 + 1.0;
        for &k in &idx[i..=j] {
            r[k] = avg;
        }
        i = j + 1;
    }
    r
}

pub fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    if x.len() < 2 {
        return 0.0;
    }
    let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
    let cov = x
        .iter()
        .zip(y)
        .map(|(a, b)| (a - mx) * (b - my))
        .sum::<f64>();
    let vx = x.iter().map(|a| (a - mx).powi(2)).sum::<f64>();
    let vy = y.iter().map(|b| (b - my).powi(2)).sum::<f64>();
    if vx > 0.0 && vy > 0.0 {
        cov / (vx * vy).sqrt()
    } else {
        0.0
    }
}

/// Spearman ρ: Pearson correlation of tie-averaged ranks.
pub fn spearman(x: &[f64], y: &[f64]) -> f64 {
    pearson(&ranks(x), &ranks(y))
}

pub fn kendall(x: &[f64], y: &[f64]) -> f64 {
    kendall_tau_b(x, y)
}

/// Leaf order of an average-linkage dendrogram over a symmetric distance
/// matrix; ties merge the lowest indices first, left cluster first.
pub fn cluster_order(dist: &[Vec<f64>]) -> Vec<usize> {
    let mut clusters: Vec<Vec<usize>> = (0..dist.len()).map(|i| vec![i]).collect();
    while clusters.len() > 1 {
        let mut best = (f64::INFINITY, 0, 1);
        for a in 0..clusters.len() {
            for b in a + 1..clusters.len() {
                let total: f64 = clusters[a]
                    .iter()
                    .flat_map(|&i| clusters[b].iter().map(move |&j| dist[i][j]))
                    .sum();
                let d = total / (clusters[a].len() * clusters[b].len()) as f64;
                if d < best.0 {
                    best = (d, a, b);
                }
            }
        }
        let right = clusters.remove(best.2);
        clusters[best.1].extend(right);
    }
    clusters.pop().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_average_ties() {
        assert_eq!(ranks(&[10.0, 20.0, 10.0, 5.0]), [2.5, 4.0, 2.5, 1.0]);
    }

    #[test]
    fn spearman_and_kendall_hand_checked() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((spearman(&x, &[2.0, 4.0, 6.0, 8.0, 100.0]) - 1.0).abs() < 1e-12);
        // One adjacent swap: d² = 2, ρ = 1 - 6·2/(5·24) = 0.9; τ = (9-1)/10 = 0.8.
        let y = [1.0, 2.0, 4.0, 3.0, 5.0];
        assert!((spearman(&x, &y) - 0.9).abs() < 1e-12);
        assert!((kendall(&x, &y) - 0.8).abs() < 1e-12);
    }

    #[test]
    fn cluster_order_groups_close_items() {
        // 0 and 2 are close, 1 and 3 are close.
        let d = vec![
            vec![0.0, 0.9, 0.1, 0.8],
            vec![0.9, 0.0, 0.85, 0.2],
            vec![0.1, 0.85, 0.0, 0.9],
            vec![0.8, 0.2, 0.9, 0.0],
        ];
        assert_eq!(cluster_order(&d), [0, 2, 1, 3]);
    }
}
