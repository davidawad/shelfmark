//! Flat partition agreement: ARI (Hubert & Arabie 1985), AMI (Vinh, Epps &
//! Bailey 2010, arithmetic-mean normalisation), V-measure (Rosenberg &
//! Hirschberg 2007), purity and Cohen's kappa (1960). Inputs are two label
//! vectors over the same items; items where either label is `None` are
//! dropped.
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct PartitionScores {
    pub n: usize,
    pub ari: f64,
    pub ami: f64,
    pub homogeneity: f64,
    pub completeness: f64,
    pub v_measure: f64,
    pub purity: f64,
}

struct Table {
    n: usize,
    cells: BTreeMap<(usize, usize), usize>,
    rows: BTreeMap<usize, usize>,
    cols: BTreeMap<usize, usize>,
}

fn table(gold: &[Option<usize>], pred: &[Option<usize>]) -> Table {
    let mut t = Table {
        n: 0,
        cells: BTreeMap::new(),
        rows: BTreeMap::new(),
        cols: BTreeMap::new(),
    };
    for (g, p) in gold.iter().zip(pred) {
        if let (Some(g), Some(p)) = (g, p) {
            t.n += 1;
            *t.cells.entry((*g, *p)).or_default() += 1;
            *t.rows.entry(*g).or_default() += 1;
            *t.cols.entry(*p).or_default() += 1;
        }
    }
    t
}

fn pairs(x: usize) -> f64 {
    x as f64 * (x as f64 - 1.0) / 2.0
}

fn entropy<'a>(counts: impl Iterator<Item = &'a usize>, n: f64) -> f64 {
    -counts
        .map(|&c| c as f64 / n)
        .filter(|p| *p > 0.0)
        .map(|p| p * p.ln())
        .sum::<f64>()
}

/// Expected mutual information under the hypergeometric model (Vinh et al.
/// 2010, eq. 24a), in nats.
fn expected_mi(t: &Table) -> f64 {
    let n = t.n;
    let mut lf = vec![0.0; n + 1];
    for i in 1..=n {
        lf[i] = lf[i - 1] + (i as f64).ln();
    }
    let nf = n as f64;
    let mut emi = 0.0;
    for &a in t.rows.values() {
        for &b in t.cols.values() {
            let start = (a + b).saturating_sub(n).max(1);
            for nij in start..=a.min(b) {
                let x = nij as f64;
                let term = (x / nf) * (nf * x / (a as f64 * b as f64)).ln();
                let log_p = lf[a] + lf[b] + lf[n - a] + lf[n - b]
                    - lf[n]
                    - lf[nij]
                    - lf[a - nij]
                    - lf[b - nij]
                    - lf[n + nij - a - b];
                emi += term * log_p.exp();
            }
        }
    }
    emi
}

pub fn compare(gold: &[Option<usize>], pred: &[Option<usize>]) -> PartitionScores {
    let t = table(gold, pred);
    if t.n == 0 {
        return PartitionScores::default();
    }
    let n = t.n as f64;
    let sum_cells = t.cells.values().map(|&c| pairs(c)).sum::<f64>();
    let sum_rows = t.rows.values().map(|&c| pairs(c)).sum::<f64>();
    let sum_cols = t.cols.values().map(|&c| pairs(c)).sum::<f64>();
    let expected = sum_rows * sum_cols / pairs(t.n).max(f64::MIN_POSITIVE);
    let max_index = (sum_rows + sum_cols) / 2.0;
    let ari = if max_index == expected {
        1.0
    } else {
        (sum_cells - expected) / (max_index - expected)
    };
    let h_gold = entropy(t.rows.values(), n);
    let h_pred = entropy(t.cols.values(), n);
    let mi = t
        .cells
        .iter()
        .map(|(&(g, p), &c)| {
            let c = c as f64;
            c / n * (n * c / (t.rows[&g] as f64 * t.cols[&p] as f64)).ln()
        })
        .sum::<f64>();
    // H(G|P) = H(G) - MI; homogeneity = 1 - H(G|P)/H(G).
    let homogeneity = if h_gold > 0.0 { mi / h_gold } else { 1.0 };
    let completeness = if h_pred > 0.0 { mi / h_pred } else { 1.0 };
    let v_measure = if homogeneity + completeness > 0.0 {
        2.0 * homogeneity * completeness / (homogeneity + completeness)
    } else {
        0.0
    };
    let emi = expected_mi(&t);
    let mean_h = (h_gold + h_pred) / 2.0;
    let ami = if (mean_h - emi).abs() < 1e-15 {
        1.0
    } else {
        (mi - emi) / (mean_h - emi)
    };
    let mut best = BTreeMap::<usize, usize>::new();
    for (&(_, p), &c) in &t.cells {
        let slot = best.entry(p).or_default();
        *slot = (*slot).max(c);
    }
    PartitionScores {
        n: t.n,
        ari,
        ami,
        homogeneity,
        completeness,
        v_measure,
        purity: best.values().sum::<usize>() as f64 / n,
    }
}

/// Cohen's kappa for two labelings in the same label space, and the raw
/// agreement share.
pub fn kappa(a: &[Option<usize>], b: &[Option<usize>]) -> (f64, f64, usize) {
    let mut n = 0usize;
    let mut agree = 0usize;
    let mut ca = BTreeMap::<usize, usize>::new();
    let mut cb = BTreeMap::<usize, usize>::new();
    for (x, y) in a.iter().zip(b) {
        if let (Some(x), Some(y)) = (x, y) {
            n += 1;
            agree += usize::from(x == y);
            *ca.entry(*x).or_default() += 1;
            *cb.entry(*y).or_default() += 1;
        }
    }
    if n == 0 {
        return (0.0, 0.0, 0);
    }
    let nf = n as f64;
    let po = agree as f64 / nf;
    let pe = ca
        .iter()
        .map(|(l, &c)| c as f64 / nf * cb.get(l).copied().unwrap_or(0) as f64 / nf)
        .sum::<f64>();
    let k = if pe < 1.0 {
        (po - pe) / (1.0 - pe)
    } else {
        1.0
    };
    (k, po, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[usize]) -> Vec<Option<usize>> {
        v.iter().map(|&x| Some(x)).collect()
    }

    #[test]
    fn identical_partitions_score_one() {
        let g = s(&[0, 0, 1, 1, 2, 2]);
        let p = s(&[5, 5, 7, 7, 9, 9]);
        let r = compare(&g, &p);
        assert!((r.ari - 1.0).abs() < 1e-12);
        assert!((r.ami - 1.0).abs() < 1e-12);
        assert!((r.v_measure - 1.0).abs() < 1e-12);
        assert_eq!(r.purity, 1.0);
    }

    #[test]
    fn hand_checked_ari_purity_and_homogeneity() {
        // gold [0,0,0,1,1,1], pred [0,0,1,1,2,2].
        // Cells: (0,0)=2,(0,1)=1,(1,1)=1,(1,2)=2 -> sum C(nij,2)=2.
        // Rows 3,3 -> 6; cols 2,2,2 -> 3; expected 6*3/15 = 1.2; max 4.5.
        // ARI = (2-1.2)/(4.5-1.2) = 0.8/3.3.
        let r = compare(&s(&[0, 0, 0, 1, 1, 1]), &s(&[0, 0, 1, 1, 2, 2]));
        assert!((r.ari - 0.8 / 3.3).abs() < 1e-12);
        // Purity: cluster majorities 2 + 1 + 2 of 6.
        assert!((r.purity - 5.0 / 6.0).abs() < 1e-12);
        // H(G) = ln 2; H(G|P) = (1/3) ln 2 (only the middle cluster is mixed).
        let h = 1.0 - (1.0 / 3.0);
        assert!((r.homogeneity - h).abs() < 1e-12);
        assert_eq!(r.n, 6);
    }

    #[test]
    fn ami_is_near_zero_for_independent_labels() {
        // A balanced crossing has MI = 0 exactly; AMI is then -EMI/(H-EMI) < 0, close to zero.
        let r = compare(&s(&[0, 0, 1, 1, 0, 0, 1, 1]), &s(&[0, 1, 0, 1, 0, 1, 0, 1]));
        assert!(r.ami < 0.0 && r.ami > -0.3, "{}", r.ami);
        assert!(r.ari < 0.0);
    }

    #[test]
    fn kappa_hand_checked() {
        // a = [0,0,1,1], b = [0,1,1,1]: po = 0.75, pe = 0.5*0.25 + 0.5*0.75 = 0.5.
        let (k, po, n) = kappa(&s(&[0, 0, 1, 1]), &s(&[0, 1, 1, 1]));
        assert_eq!((po, n), (0.75, 4));
        assert!((k - 0.5).abs() < 1e-12);
    }
}
