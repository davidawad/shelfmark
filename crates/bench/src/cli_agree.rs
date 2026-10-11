//! `agree` subcommand: rank agreement between scores across runs, rank
//! stability, and (optionally) the coverage analysis, paired-bootstrap
//! differences between runs and the agreement of each score's ranking with
//! a second set of values for the same runs (the covered subset).
use crate::cli_scores::{need, options, write, Res};
use shelfmark_bench::agree::{cluster_order, kendall, paired_summary, partial, spearman};
use shelfmark_bench::robust::interval;
use std::collections::BTreeMap;

/// Agreement input: `point[m][r]` (null when a run lacks the score),
/// optional bootstrap replicates `boot[m][r][b]` and noisy values `noise[m][r]`.
/// `control` names the metric to control for (partial correlations);
/// `paired` asks for paired-bootstrap differences between runs, valid only
/// when replicate `b` of every run used the same resample; `compare_point`
/// and `compare_boot` hold a second set of values for the same runs.
#[derive(serde::Deserialize)]
struct AgreeInput {
    metrics: Vec<String>,
    runs: Vec<String>,
    point: Vec<Vec<Option<f64>>>,
    #[serde(default)]
    boot: Vec<Vec<Vec<f64>>>,
    #[serde(default)]
    noise: Vec<Vec<Option<f64>>>,
    #[serde(default)]
    control: Option<String>,
    #[serde(default)]
    paired: bool,
    #[serde(default)]
    compare_point: Vec<Vec<Option<f64>>>,
    #[serde(default)]
    compare_boot: Vec<Vec<Vec<f64>>>,
}

/// Paired values of two rows where both are present.
fn paired(a: &[Option<f64>], b: &[Option<f64>]) -> (Vec<f64>, Vec<f64>) {
    a.iter()
        .zip(b)
        .filter_map(|(x, y)| Some(((*x)?, (*y)?)))
        .unzip()
}

/// Bootstrap replicate count shared by every run of a metric that has
/// replicates (0 if none has).
fn reps_of(boot: &[Vec<Vec<f64>>], m: usize) -> usize {
    boot.get(m).map_or(0, |r| {
        r.iter().map(Vec::len).filter(|&l| l > 0).min().unwrap_or(0)
    })
}

/// Replicate `b` of a metric across runs (`None` for a run without one).
fn replicate(boot: &[Vec<Vec<f64>>], m: usize, b: usize) -> Vec<Option<f64>> {
    boot[m].iter().map(|r| r.get(b).copied()).collect()
}

fn stability(input: &AgreeInput, m: usize) -> serde_json::Value {
    let point = &input.point[m];
    let mut taus = vec![];
    for b in 0..reps_of(&input.boot, m) {
        let (x, y) = paired(point, &replicate(&input.boot, m, b));
        taus.push(kendall(&x, &y));
    }
    taus.sort_by(f64::total_cmp);
    // Scores outside the noise set have no noisy values: report null.
    let noise = input.noise.get(m).and_then(|n| {
        let (x, y) = paired(point, n);
        (x.len() >= 2).then(|| kendall(&x, &y))
    });
    serde_json::json!({
        "boot_tau_mean": if taus.is_empty() { None } else { Some(taus.iter().sum::<f64>() / taus.len() as f64) },
        "boot_tau_p05": taus.get(taus.len() / 20).copied(),
        "noise_tau": noise,
    })
}

fn rho_matrix(rows: &[Vec<Option<f64>>]) -> Vec<Vec<f64>> {
    let m = rows.len();
    let mut rho = vec![vec![0.0; m]; m];
    for a in 0..m {
        for b in a..m {
            let (x, y) = paired(&rows[a], &rows[b]);
            rho[a][b] = spearman(&x, &y);
            rho[b][a] = rho[a][b];
        }
    }
    rho
}

fn partial_matrix(rho: &[Vec<f64>], c: usize) -> Vec<Vec<f64>> {
    (0..rho.len())
        .map(|a| {
            (0..rho.len())
                .map(|b| partial(rho[a][b], rho[a][c], rho[b][c]))
                .collect()
        })
        .collect()
}

/// Coverage analysis: Spearman ρ of every score with the control, the
/// partial Spearman matrix controlling for it, and their bootstrap
/// intervals (each replicate takes every metric's replicate `b`, or its
/// point value when the metric has no replicates; the control keeps its
/// point value).
fn control_analysis(input: &AgreeInput, c: usize) -> serde_json::Value {
    let m = input.metrics.len();
    let rho = rho_matrix(&input.point);
    let part = partial_matrix(&rho, c);
    let reps = (0..m).map(|k| reps_of(&input.boot, k)).max().unwrap_or(0);
    let mut with_control = vec![vec![]; m];
    let mut partial_reps = vec![vec![vec![]; m]; m];
    for b in 0..reps {
        let rows: Vec<Vec<Option<f64>>> = (0..m)
            .map(|k| {
                if k != c && reps_of(&input.boot, k) > b {
                    replicate(&input.boot, k, b)
                } else {
                    input.point[k].clone()
                }
            })
            .collect();
        let r = rho_matrix(&rows);
        let p = partial_matrix(&r, c);
        for a in 0..m {
            with_control[a].push(r[a][c]);
            for k in 0..m {
                partial_reps[a][k].push(p[a][k]);
            }
        }
    }
    let ci = |v: &Vec<f64>| v.iter().any(|x| x.is_finite()).then(|| interval(v));
    serde_json::json!({
        "metric": input.metrics[c],
        "rho": (0..m).map(|a| rho[a][c]).collect::<Vec<_>>(),
        "rho_sq": (0..m).map(|a| rho[a][c] * rho[a][c]).collect::<Vec<_>>(),
        "rho_ci": with_control.iter().map(ci).collect::<Vec<_>>(),
        "partial_spearman": part,
        "partial_ci": partial_reps.iter().map(|r| r.iter().map(ci).collect::<Vec<_>>()).collect::<Vec<_>>(),
    })
}

/// Paired-bootstrap differences `run a − run b` for every metric with
/// replicates and every pair of runs.
fn paired_differences(input: &AgreeInput) -> BTreeMap<&String, Vec<serde_json::Value>> {
    let mut out = BTreeMap::new();
    for (m, name) in input.metrics.iter().enumerate() {
        let reps = reps_of(&input.boot, m);
        if reps == 0 {
            continue;
        }
        let mut rows = vec![];
        for a in 0..input.runs.len() {
            for b in a + 1..input.runs.len() {
                let (Some(pa), Some(pb)) = (input.point[m][a], input.point[m][b]) else {
                    continue;
                };
                let (ra, rb) = (&input.boot[m][a], &input.boot[m][b]);
                if ra.len() < reps || rb.len() < reps {
                    continue;
                }
                let diffs: Vec<f64> = (0..reps).map(|k| ra[k] - rb[k]).collect();
                let (ci, p) = paired_summary(&diffs);
                rows.push(serde_json::json!({
                    "a": input.runs[a], "b": input.runs[b], "diff": pa - pb, "ci": ci, "p": p,
                }));
            }
        }
        out.insert(name, rows);
    }
    out
}

/// Agreement of each metric's run ranking with the comparison values:
/// Spearman and Kendall on the points, and the Kendall interval over
/// replicates `b` of both sets where both have them.
fn compare_analysis(input: &AgreeInput) -> BTreeMap<&String, serde_json::Value> {
    let mut out = BTreeMap::new();
    for (m, name) in input.metrics.iter().enumerate() {
        let Some(other) = input.compare_point.get(m) else {
            continue;
        };
        let (x, y) = paired(&input.point[m], other);
        let reps = reps_of(&input.boot, m).min(reps_of(&input.compare_boot, m));
        let taus: Vec<f64> = (0..reps)
            .map(|b| {
                let (x, y) = paired(
                    &replicate(&input.boot, m, b),
                    &replicate(&input.compare_boot, m, b),
                );
                kendall(&x, &y)
            })
            .collect();
        out.insert(
            name,
            serde_json::json!({
                "runs": x.len(),
                "spearman": (x.len() >= 3).then(|| spearman(&x, &y)),
                "kendall": (x.len() >= 3).then(|| kendall(&x, &y)),
                "kendall_ci": (!taus.is_empty()).then(|| interval(&taus)),
            }),
        );
    }
    out
}

pub fn agree(args: &[String]) -> Res<()> {
    let o = options(args)?;
    let input: AgreeInput = serde_json::from_slice(&std::fs::read(need(&o, "input")?)?)?;
    let m = input.metrics.len();
    let rho = rho_matrix(&input.point);
    let mut tau = vec![vec![0.0; m]; m];
    let mut pairs = vec![vec![0usize; m]; m];
    for a in 0..m {
        for b in 0..m {
            let (x, y) = paired(&input.point[a], &input.point[b]);
            tau[a][b] = kendall(&x, &y);
            pairs[a][b] = x.len();
        }
    }
    let dist: Vec<Vec<f64>> = rho
        .iter()
        .map(|r| r.iter().map(|v| 1.0 - v.abs()).collect())
        .collect();
    let order = cluster_order(&dist);
    let stable: BTreeMap<&String, serde_json::Value> = input
        .metrics
        .iter()
        .enumerate()
        .map(|(i, k)| (k, stability(&input, i)))
        .collect();
    let mut out = serde_json::json!({
        "metrics": input.metrics,
        "runs": input.runs,
        "spearman": rho,
        "kendall": tau,
        "pairs": pairs,
        "order": order,
        "rank_stability": stable,
    });
    if let Some(name) = &input.control {
        let c = input
            .metrics
            .iter()
            .position(|x| x == name)
            .ok_or(format!("control {name} is not a metric"))?;
        out["control"] = control_analysis(&input, c);
    }
    if input.paired {
        out["paired"] = serde_json::to_value(paired_differences(&input))?;
    }
    if !input.compare_point.is_empty() {
        out["compare"] = serde_json::to_value(compare_analysis(&input))?;
    }
    write(&o, &serde_json::to_string_pretty(&out)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> AgreeInput {
        let some = |v: &[f64]| v.iter().map(|&x| Some(x)).collect::<Vec<_>>();
        AgreeInput {
            metrics: vec!["cost".into(), "coverage".into()],
            runs: vec!["a".into(), "b".into(), "c".into()],
            point: vec![some(&[0.3, 0.2, 0.1]), some(&[0.5, 0.7, 0.9])],
            // Run c has no replicates; a stays above b in every replicate.
            boot: vec![
                vec![vec![0.31, 0.29, 0.30], vec![0.21, 0.19, 0.20], vec![]],
                vec![],
            ],
            noise: vec![],
            control: Some("coverage".into()),
            paired: true,
            compare_point: vec![some(&[0.1, 0.2, 0.3]), some(&[0.5, 0.7, 0.9])],
            compare_boot: vec![],
        }
    }

    #[test]
    fn control_paired_and_compare() {
        let i = input();
        let c = control_analysis(&i, 1);
        assert!((c["rho"][0].as_f64().unwrap() + 1.0).abs() < 1e-12);
        assert!((c["rho_sq"][0].as_f64().unwrap() - 1.0).abs() < 1e-12);
        // Coverage fixes the cost ranking: the partial is undefined (null).
        assert!(c["partial_spearman"][0][0].is_null());
        let p = paired_differences(&i);
        let rows = &p[&"cost".to_owned()];
        // Only a−b has replicates for both runs.
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["p"].as_f64().unwrap(), 0.0);
        assert!((rows[0]["diff"].as_f64().unwrap() - 0.1).abs() < 1e-12);
        let cmp = compare_analysis(&i);
        assert!((cmp[&"cost".to_owned()]["kendall"].as_f64().unwrap() + 1.0).abs() < 1e-12);
        assert_eq!(reps_of(&i.boot, 0), 3);
        assert_eq!(replicate(&i.boot, 0, 1), [Some(0.29), Some(0.19), None]);
    }
}
