//! `shelfmark-bench bootstrap`: paired item-bootstrap intervals for several
//! placement runs on one corpus and graph.
use crate::cli_scores::{need, number, options};
use crate::{load_placements, read_jsonl};
use serde_json::{json, Map, Value};
use shelfmark_bench::bootstrap::{
    bootstrap, draws, interval, metric_values, BootstrapRun, BootstrapSamples, DEFAULT_SEED,
};
use shelfmark_bench::{
    average_linkage_reference, induce_tree, score_tree, sha256_hex, CorpusRow, ItemPlacement,
    SimilarityGraph, Tree,
};
use shelfmark_core::scheme::SchemeDoc;
use std::collections::BTreeMap;

const USAGE: &str = "usage: shelfmark-bench bootstrap --corpus corpus.jsonl --graph graph.json \
--runs runs.json [--resamples 1000] [--seed 20261010] [--confidence 0.95] [--threads 8] \
[--output bootstrap.json]";

/// Metrics whose paired differences between runs are reported.
const PAIRED: [&str; 3] = ["cost_norm", "nav_log_a1", "largest_class_share"];

#[derive(serde::Deserialize)]
struct RunSpec {
    key: String,
    scheme: String,
    #[serde(default)]
    placements: Option<String>,
}

type Error = Box<dyn std::error::Error>;

fn sha256_file(path: &str) -> Result<String, Error> {
    Ok(sha256_hex(&std::fs::read(path)?))
}

pub fn run(args: &[String]) -> Result<(), Error> {
    let values = options(args).map_err(|e| format!("{e}\n{USAGE}"))?;
    let need = |key| need(&values, key).map_err(|e| format!("{e}\n{USAGE}"));
    let (corpus_path, graph_path, runs_path) = (need("corpus")?, need("graph")?, need("runs")?);
    let resamples: usize = number(&values, "resamples", 1000)?;
    let seed: u64 = number(&values, "seed", DEFAULT_SEED)?;
    let confidence: f64 = number(&values, "confidence", 0.95)?;
    let threads: usize = number(&values, "threads", 8)?;
    let corpus: Vec<CorpusRow> = read_jsonl(corpus_path)?;
    let graph_bytes = std::fs::read(graph_path)?;
    let graph: SimilarityGraph = serde_json::from_slice(&graph_bytes)?;
    if graph.n != corpus.len() {
        return Err(format!("graph has {} rows but corpus has {}", graph.n, corpus.len()).into());
    }
    let specs: Vec<RunSpec> = serde_json::from_slice(&std::fs::read(runs_path)?)?;
    let mut trees = Vec::new();
    let mut provenance = Vec::new();
    for spec in &specs {
        let (tree, placed, prov) = load_run(spec, &corpus)?;
        provenance.push(prov);
        trees.push((tree, placed));
    }
    let reference = average_linkage_reference(&graph);
    let ref_point = score_tree(&reference, &graph).cost_norm;
    let runs: Vec<_> = specs
        .iter()
        .zip(&trees)
        .map(|(spec, (tree, placed))| BootstrapRun {
            key: spec.key.clone(),
            tree,
            placed: placed.clone(),
        })
        .collect();
    let picks = draws(graph.n, resamples, seed);
    let samples = bootstrap(&graph, &reference, &runs, &picks, threads);
    let mut out_runs = Map::new();
    for ((run, prov), (tree, placed)) in runs.iter().zip(provenance).zip(&trees) {
        let coverage = placed.iter().filter(|&&p| p).count() as f64 / placed.len().max(1) as f64;
        let point = metric_values(&score_tree(tree, &graph), ref_point, coverage);
        let draws_for = &samples.runs[&run.key];
        let ci: Map<String, Value> = point
            .keys()
            .map(|m| {
                let xs: Vec<f64> = draws_for.iter().map(|d| d[m]).collect();
                (m.clone(), json!(interval(&xs, confidence)))
            })
            .collect();
        let mut entry = prov;
        entry["point"] = json!(point);
        entry["ci"] = Value::Object(ci);
        out_runs.insert(run.key.clone(), entry);
    }
    let paired = paired_differences(&runs, &samples, &out_runs, confidence);
    let result = json!({
        "method": "paired percentile bootstrap over items on the fixed graph: each draw takes n items \
    with replacement, gives a drawn item one copy per draw under its original parent, keeps every graph \
    edge once per pair of copies, prunes classes left without items, and rescores every run and the \
    average-linkage reference tree (built once on the full graph) on the same draw",
        "n": graph.n,
        "k": graph.k,
        "graph_sha256": sha256_hex(&graph_bytes),
        "corpus_sha256": sha256_file(corpus_path)?,
        "resamples": resamples,
        "seed": seed,
        "confidence": confidence,
        "reference": {"cost_norm": ref_point, "ci": interval(&samples.reference, confidence)},
        "runs": out_runs,
        "paired": paired,
    });
    let text = serde_json::to_string_pretty(&result)?;
    match values.get("output") {
        Some(path) => std::fs::write(path, text)?,
        None => println!("{text}"),
    }
    Ok(())
}

/// The induced tree, which items are placed, and the provenance of one run.
fn load_run(spec: &RunSpec, corpus: &[CorpusRow]) -> Result<(Tree, Vec<bool>, Value), Error> {
    let scheme: SchemeDoc = serde_json::from_slice(&std::fs::read(&spec.scheme)?)?;
    let mut opts = BTreeMap::new();
    if let Some(path) = &spec.placements {
        opts.insert("--placements".to_owned(), path.clone());
    }
    let mut placements = load_placements(&opts, corpus, &scheme)?;
    let items: Vec<_> = corpus
        .iter()
        .map(|row| ItemPlacement {
            id: row.id.clone(),
            classes: placements.remove(&row.id).unwrap_or_default(),
        })
        .collect();
    let placed: Vec<bool> = items.iter().map(|p| !p.classes.is_empty()).collect();
    let tree =
        induce_tree(&scheme, &items).map_err(|e| format!("{}: tree induction: {e}", spec.key))?;
    let prov = json!({
        "scheme": scheme.name,
        "version": scheme.version,
        "scheme_sha256": sha256_file(&spec.scheme)?,
        "placements_sha256": spec.placements.as_deref().map(sha256_file).transpose()?,
    });
    Ok((tree, placed, prov))
}

/// Per-draw differences `a - b` of the `PAIRED` metrics for every run pair.
fn paired_differences(
    runs: &[BootstrapRun<'_>],
    samples: &BootstrapSamples,
    out_runs: &Map<String, Value>,
    confidence: f64,
) -> Map<String, Value> {
    let mut paired = Map::new();
    for metric in PAIRED {
        let mut rows = Vec::new();
        for (i, a) in runs.iter().enumerate() {
            for b in &runs[i + 1..] {
                let (da, db) = (&samples.runs[&a.key], &samples.runs[&b.key]);
                let diffs: Vec<f64> = da
                    .iter()
                    .zip(db)
                    .map(|(x, y)| x[metric] - y[metric])
                    .collect();
                let lower =
                    diffs.iter().filter(|&&d| d < 0.0).count() as f64 / diffs.len().max(1) as f64;
                let point = out_runs[&a.key]["point"][metric]
                    .as_f64()
                    .unwrap_or(f64::NAN)
                    - out_runs[&b.key]["point"][metric]
                        .as_f64()
                        .unwrap_or(f64::NAN);
                rows.push(json!({"a": a.key, "b": b.key, "diff": point,
                    "ci": interval(&diffs, confidence), "share_a_lower": lower}));
            }
        }
        paired.insert(metric.to_owned(), Value::Array(rows));
    }
    paired
}
