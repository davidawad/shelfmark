//! `scores` and `agree` subcommands: the scoring catalogue for one run, and
//! the rank agreement between scores across runs.
use crate::{gold_field, read_jsonl, ClassMass, PlacementRow};
use shelfmark_bench::agree::{cluster_order, kendall, spearman};
use shelfmark_bench::catalogue::{catalogue, Context};
use shelfmark_bench::consistency::consistency;
use shelfmark_bench::embedscore::Vectors;
use shelfmark_bench::{sha256_hex, CorpusRow, Hier, SimilarityGraph};
use shelfmark_core::scheme::SchemeDoc;
use std::collections::BTreeMap;
use std::error::Error;

type Res<T> = Result<T, Box<dyn Error>>;

fn options(args: &[String]) -> Res<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    let mut i = 2;
    while i < args.len() {
        let key = args[i]
            .strip_prefix("--")
            .ok_or(format!("unexpected argument {}", args[i]))?;
        let value = args
            .get(i + 1)
            .ok_or(format!("missing value for --{key}"))?;
        out.insert(key.to_owned(), value.clone());
        i += 2;
    }
    Ok(out)
}

fn need<'a>(o: &'a BTreeMap<String, String>, key: &str) -> Res<&'a String> {
    Ok(o.get(key).ok_or(format!("missing --{key}"))?)
}

fn number<T: std::str::FromStr>(o: &BTreeMap<String, String>, key: &str, default: T) -> Res<T>
where
    T::Err: Error + 'static,
{
    Ok(match o.get(key) {
        Some(v) => v.parse()?,
        None => default,
    })
}

/// Labels per corpus row, highest mass first (ties keep source order).
fn placement_labels(path: &str, corpus: &[CorpusRow]) -> Res<Vec<Vec<String>>> {
    let mut by_id = BTreeMap::new();
    for row in read_jsonl::<PlacementRow>(path)? {
        let mut classes: Vec<(String, f64)> = row
            .classes
            .into_iter()
            .map(|c| match c {
                ClassMass::Code(code) => (code, 1.0),
                ClassMass::Weighted { code, mass } => (code, mass),
            })
            .collect();
        classes.sort_by(|a, b| b.1.total_cmp(&a.1));
        by_id.insert(row.id, classes.into_iter().map(|c| c.0).collect::<Vec<_>>());
    }
    Ok(corpus
        .iter()
        .map(|r| by_id.remove(&r.id).unwrap_or_default())
        .collect())
}

#[derive(serde::Deserialize)]
struct External {
    id: String,
    fine: Option<String>,
    coarse: Option<String>,
}

/// Fine and coarse external gold labels as dense ids (sorted label order).
fn external(path: &str, corpus: &[CorpusRow]) -> Res<[Vec<Option<usize>>; 2]> {
    let rows: BTreeMap<String, External> = read_jsonl::<External>(path)?
        .into_iter()
        .map(|r| (r.id.clone(), r))
        .collect();
    let mut out: [Vec<Option<usize>>; 2] = Default::default();
    for (slot, pick) in out.iter_mut().zip([0, 1]) {
        let label = |r: &External| {
            if pick == 0 {
                r.fine.clone()
            } else {
                r.coarse.clone()
            }
        };
        let names: BTreeMap<String, usize> = {
            let set: std::collections::BTreeSet<String> = rows.values().filter_map(label).collect();
            set.into_iter().enumerate().map(|(i, s)| (s, i)).collect()
        };
        *slot = corpus
            .iter()
            .map(|c| rows.get(&c.id).and_then(label).map(|l| names[&l]))
            .collect();
    }
    Ok(out)
}

fn vectors(path: &str, dim: usize, n: usize) -> Res<Vectors> {
    let bytes = std::fs::read(path)?;
    if bytes.len() != n * dim * 4 {
        return Err(format!("{path}: {} bytes, expected {n} x {dim} f32", bytes.len()).into());
    }
    let data = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    Ok(Vectors { dim, data })
}

fn file_hash(path: Option<&String>) -> Res<Option<String>> {
    Ok(match path {
        Some(p) => Some(sha256_hex(&std::fs::read(p)?)),
        None => None,
    })
}

pub fn scores(args: &[String]) -> Res<()> {
    let o = options(args)?;
    let corpus: Vec<CorpusRow> = read_jsonl(need(&o, "corpus")?)?;
    let scheme_bytes = std::fs::read(need(&o, "scheme")?)?;
    let scheme: SchemeDoc = serde_json::from_slice(&scheme_bytes)?;
    let graph_bytes = std::fs::read(need(&o, "graph")?)?;
    let graph: SimilarityGraph = serde_json::from_slice(&graph_bytes)?;
    if graph.n != corpus.len() {
        return Err(format!("graph has {} rows but corpus has {}", graph.n, corpus.len()).into());
    }
    let labels = match o.get("placements") {
        Some(p) => placement_labels(p, &corpus)?,
        None => corpus
            .iter()
            .map(|r| gold_field(r, &scheme.name).to_vec())
            .collect(),
    };
    let primary: Vec<Option<String>> = labels.iter().map(|l| l.first().cloned()).collect();
    let hier = Hier::build(&scheme, &primary).map_err(|e| format!("hierarchy: {e}"))?;
    let vecs = match o.get("vectors") {
        Some(p) => Some(vectors(p, number(&o, "dims", 384)?, corpus.len())?),
        None => None,
    };
    let gold = match o.get("external") {
        Some(p) => Some(external(p, &corpus)?),
        None => None,
    };
    let authors: Vec<Vec<String>> = corpus.iter().map(|r| r.authors.clone()).collect();
    let counts: Vec<usize> = labels.iter().map(Vec::len).collect();
    let ctx = Context {
        scheme: &scheme,
        graph: &graph,
        vectors: vecs.as_ref(),
        gold,
        authors: &authors,
        label_counts: &counts,
        seed: number(&o, "seed", 20_261_010)?,
        reps: number(&o, "reps", 10)?,
        boot: number(&o, "boot", 200)?,
        sample: number(&o, "sample", 2000)?,
    };
    let result = catalogue(&hier, &ctx);
    // Reference labels for inter-classifier consistency: source labels in a
    // corpus-format file, or another placement file.
    let reference: Option<Vec<Vec<String>>> =
        match (o.get("compare-gold"), o.get("compare-placements")) {
            (Some(p), _) => {
                let rows: BTreeMap<String, CorpusRow> = read_jsonl::<CorpusRow>(p)?
                    .into_iter()
                    .map(|r| (r.id.clone(), r))
                    .collect();
                Some(
                    corpus
                        .iter()
                        .map(|c| {
                            rows.get(&c.id)
                                .map(|r| gold_field(r, &scheme.name).to_vec())
                                .unwrap_or_default()
                        })
                        .collect(),
                )
            }
            (None, Some(p)) => Some(placement_labels(p, &corpus)?),
            (None, None) => None,
        };
    let compared = reference.map(|r| {
        let first: Vec<Option<String>> = r.iter().map(|l| l.first().cloned()).collect();
        consistency(&scheme, &first, &primary)
    });
    let out = serde_json::json!({
        "scheme": scheme.name,
        "version": scheme.version,
        "n": corpus.len(),
        "params": {"seed": ctx.seed, "reps": ctx.reps, "boot": ctx.boot, "sample": ctx.sample,
                   "insertion_capacity": 50, "author_cap": 50, "noise": [0.05, 0.1, 0.2]},
        "catalogue": result,
        "consistency": compared,
        "hashes": {
            "graph": sha256_hex(&graph_bytes),
            "scheme": sha256_hex(&scheme_bytes),
            "corpus": file_hash(o.get("corpus"))?,
            "placements": file_hash(o.get("placements"))?,
            "vectors": file_hash(o.get("vectors"))?,
            "external": file_hash(o.get("external"))?,
            "compare": file_hash(o.get("compare-gold").or(o.get("compare-placements")))?,
        },
    });
    write(&o, &serde_json::to_string_pretty(&out)?)
}

fn write(o: &BTreeMap<String, String>, json: &str) -> Res<()> {
    match o.get("output") {
        Some(p) => std::fs::write(p, json)?,
        None => println!("{json}"),
    }
    Ok(())
}

/// Agreement input: `point[m][r]` (null when a run lacks the score),
/// optional bootstrap replicates `boot[m][r][b]` and noisy values `noise[m][r]`.
#[derive(serde::Deserialize)]
struct AgreeInput {
    metrics: Vec<String>,
    runs: Vec<String>,
    point: Vec<Vec<Option<f64>>>,
    #[serde(default)]
    boot: Vec<Vec<Vec<f64>>>,
    #[serde(default)]
    noise: Vec<Vec<Option<f64>>>,
}

/// Paired values of two rows where both are present.
fn paired(a: &[Option<f64>], b: &[Option<f64>]) -> (Vec<f64>, Vec<f64>) {
    a.iter()
        .zip(b)
        .filter_map(|(x, y)| Some(((*x)?, (*y)?)))
        .unzip()
}

fn stability(input: &AgreeInput, m: usize) -> serde_json::Value {
    let point = &input.point[m];
    let mut taus = vec![];
    let reps = input
        .boot
        .get(m)
        .map_or(0, |r| r.iter().map(Vec::len).min().unwrap_or(0));
    for b in 0..reps {
        let replicate: Vec<Option<f64>> = (0..input.runs.len())
            .map(|r| Some(input.boot[m][r][b]))
            .collect();
        let (x, y) = paired(point, &replicate);
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

pub fn agree(args: &[String]) -> Res<()> {
    let o = options(args)?;
    let input: AgreeInput = serde_json::from_slice(&std::fs::read(need(&o, "input")?)?)?;
    let m = input.metrics.len();
    let mut rho = vec![vec![0.0; m]; m];
    let mut tau = vec![vec![0.0; m]; m];
    let mut pairs = vec![vec![0usize; m]; m];
    for a in 0..m {
        for b in 0..m {
            let (x, y) = paired(&input.point[a], &input.point[b]);
            rho[a][b] = spearman(&x, &y);
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
    let out = serde_json::json!({
        "metrics": input.metrics,
        "runs": input.runs,
        "spearman": rho,
        "kendall": tau,
        "pairs": pairs,
        "order": order,
        "rank_stability": stable,
    });
    write(&o, &serde_json::to_string_pretty(&out)?)
}
