use shelfmark_bench::{
    average_linkage_reference, bootstrap, embed_corpus, induce_tree, score_tree, sha256_hex,
    BenchmarkResult, ContentHashes, CorpusRow, DeterministicEmbedder, ItemPlacement, Replicates,
    SimilarityGraph, Tree,
};
use shelfmark_core::scheme::SchemeDoc;
use std::{
    collections::BTreeMap,
    env,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

#[derive(serde::Deserialize)]
struct PlacementRow {
    id: String,
    #[serde(default)]
    classes: Vec<ClassMass>,
}
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ClassMass {
    Code(String),
    Weighted {
        code: String,
        #[serde(default = "one")]
        mass: f64,
    },
}
fn one() -> f64 {
    1.0
}

/// Item id to its (class code, mass) placements.
type Placements = BTreeMap<String, Vec<(String, f64)>>;

mod cli_agree;
mod cli_dense;
mod cli_scores;

fn main() {
    let args: Vec<_> = env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("scores") => cli_scores::scores(&args),
        Some("agree") => cli_agree::agree(&args),
        Some("dense") => cli_dense::dense(&args),
        _ => run(),
    };
    if let Err(error) = result {
        eprintln!("shelfmark-bench: {error}");
        std::process::exit(2);
    }
}
/// Command-line options after `run`: `--key value` pairs plus `--k`.
fn parse_args(
    args: &[String],
) -> Result<(BTreeMap<String, String>, usize), Box<dyn std::error::Error>> {
    if args.len() < 6 || args[1] != "run" {
        return Err("usage: shelfmark-bench run --corpus corpus.jsonl --scheme scheme.json [--placements placements.jsonl] [--track A|P] [--graph graph.json] [--k 20] [--reference-cost X] [--bootstrap N [--seed S] [--reference-replicates FILE] [--replicates-out FILE]] [--output results.json]".into());
    }
    let mut values = BTreeMap::new();
    let mut k = 20usize;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--k" => {
                i += 1;
                k = args.get(i).ok_or("missing --k value")?.parse()?
            }
            key if key.starts_with("--") => {
                i += 1;
                values.insert(
                    key.to_owned(),
                    args.get(i).ok_or("missing option value")?.clone(),
                );
            }
            _ => return Err(format!("unexpected argument {}", args[i]).into()),
        }
        i += 1;
    }
    Ok((values, k))
}

/// Placements from `--placements`, or the first source label of each corpus
/// row for the scheme when no placement file is given.
fn load_placements(
    values: &BTreeMap<String, String>,
    corpus: &[CorpusRow],
    scheme: &SchemeDoc,
) -> Result<Placements, Box<dyn std::error::Error>> {
    let mut placements = Placements::new();
    if let Some(path) = values.get("--placements") {
        for row in read_jsonl::<PlacementRow>(path)? {
            let classes = row
                .classes
                .into_iter()
                .map(|c| match c {
                    ClassMass::Code(code) => (code, 1.0),
                    ClassMass::Weighted { code, mass } => (code, mass),
                })
                .collect();
            placements.insert(row.id, classes);
        }
    } else {
        for row in corpus {
            if let Some(code) = gold_field(row, &scheme.name).first() {
                placements.insert(row.id.clone(), vec![(code.clone(), 1.0)]);
            }
        }
    }
    Ok(placements)
}

/// The source labels of a corpus row for a scheme name, in source order.
pub(crate) fn gold_field<'a>(row: &'a CorpusRow, scheme: &str) -> &'a [String] {
    match scheme.to_ascii_lowercase().as_str() {
        "lcc" => row.lc_classifications.as_slice(),
        "ddc" => row.ddc.as_slice(),
        "msc" => row.msc_classes.as_slice(),
        "acm" | "acm-ccs" | "acm_ccs" => row.acm_ccs.as_slice(),
        "jel" => row.jel_codes.as_slice(),
        "arxiv" => row.categories.as_slice(),
        name => row.gold.get(name).map(Vec::as_slice).unwrap_or_default(),
    }
}

/// The similarity graph from `--graph`, or one embedded from the corpus.
fn load_graph(
    values: &BTreeMap<String, String>,
    corpus: &[CorpusRow],
    k: usize,
) -> Result<SimilarityGraph, Box<dyn std::error::Error>> {
    if let Some(path) = values.get("--graph") {
        let graph: SimilarityGraph = serde_json::from_slice(&std::fs::read(path)?)?;
        if graph.n != corpus.len() {
            return Err(
                format!("graph has {} rows but corpus has {}", graph.n, corpus.len()).into(),
            );
        }
        Ok(graph)
    } else {
        let model = DeterministicEmbedder::default();
        Ok(embed_corpus(corpus, &model, k))
    }
}

/// The reference cost from `--reference-cost`, or the average-linkage tree
/// of the graph (also returned, for bootstrap replicates of its cost).
fn reference(
    values: &BTreeMap<String, String>,
    graph: &SimilarityGraph,
) -> Result<(f64, Option<Tree>), Box<dyn std::error::Error>> {
    if let Some(value) = values.get("--reference-cost") {
        Ok((value.parse()?, None))
    } else {
        let hac = average_linkage_reference(graph);
        Ok((score_tree(&hac, graph).cost_norm, Some(hac)))
    }
}

/// Item-bootstrap replicates when `--bootstrap N` is given. Their reference
/// cost comes from `--reference-replicates` (the replicates file of the
/// reference run on the same graph, seed and resample count) or from the
/// average-linkage tree computed in this run.
fn bootstrap_replicates(
    values: &BTreeMap<String, String>,
    tree: &Tree,
    graph: &SimilarityGraph,
    hac: Option<&Tree>,
) -> Result<Option<Replicates>, Box<dyn std::error::Error>> {
    let resamples: usize = match values.get("--bootstrap") {
        Some(v) => v.parse()?,
        None => return Ok(None),
    };
    if resamples == 0 {
        return Ok(None);
    }
    let seed = match values.get("--seed") {
        Some(v) => v.parse()?,
        None => bootstrap::DEFAULT_SEED,
    };
    let reference = match values.get("--reference-replicates") {
        Some(path) => {
            let r: Replicates = serde_json::from_slice(&std::fs::read(path)?)?;
            if (r.n, r.resamples, r.seed) != (graph.n, resamples, seed) {
                return Err(format!(
                    "{path}: replicates are for another n, resample count or seed"
                )
                .into());
            }
            Some(
                r.reference_costs()
                    .ok_or("reference replicates lack ref_cost_norm")?
                    .to_vec(),
            )
        }
        None => None,
    };
    let prepared = bootstrap::Prepared::new(tree, graph, true);
    let hac = match (&reference, hac) {
        (None, Some(t)) => Some(bootstrap::Prepared::new(t, graph, false)),
        _ => None,
    };
    let replicates = bootstrap::run(
        &prepared,
        reference.as_deref(),
        hac.as_ref(),
        resamples,
        seed,
    );
    if let Some(path) = values.get("--replicates-out") {
        std::fs::write(path, serde_json::to_vec(&replicates)?)?;
    }
    Ok(Some(replicates))
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    let (values, k) = parse_args(&args)?;
    let corpus_path = values.get("--corpus").ok_or("missing --corpus")?;
    let scheme_path = values.get("--scheme").ok_or("missing --scheme")?;
    let corpus: Vec<CorpusRow> = read_jsonl(corpus_path)?;
    let scheme_bytes = std::fs::read(scheme_path)?;
    let scheme: SchemeDoc = serde_json::from_slice(&scheme_bytes)?;
    let mut placements = load_placements(&values, &corpus, &scheme)?;
    let graph = load_graph(&values, &corpus, k)?;
    let item_placements: Vec<_> = corpus
        .iter()
        .map(|row| ItemPlacement {
            id: row.id.clone(),
            classes: placements.remove(&row.id).unwrap_or_default(),
        })
        .collect();
    let tree =
        induce_tree(&scheme, &item_placements).map_err(|e| format!("tree induction: {e}"))?;
    let score = score_tree(&tree, &graph);
    let (ref_score, hac) = reference(&values, &graph)?;
    let replicates = bootstrap_replicates(&values, &tree, &graph, hac.as_ref())?;
    let skill = if ref_score < 1.0 {
        (1.0 - score.cost_norm) / (1.0 - ref_score)
    } else {
        0.0
    };
    let coverage = item_placements
        .iter()
        .filter(|p| !p.classes.is_empty())
        .count() as f64
        / corpus.len().max(1) as f64;
    let track = match values.get("--track") {
        Some(t) if t == "A" || t == "P" => t.clone(),
        Some(t) => return Err(format!("--track must be A or P, not {t}").into()),
        None if values.contains_key("--placements") => "P".into(),
        None => "A".into(),
    };
    let result = BenchmarkResult {
        scheme: scheme.name,
        version: scheme.version,
        track,
        mode: "primary".into(),
        n: corpus.len(),
        k,
        edge_weight: score.edge_weight,
        cost: score.cost,
        cost_norm: score.cost_norm,
        revenue: score.revenue,
        revenue_norm: score.revenue_norm,
        ref_cost_norm: ref_score,
        skill,
        coverage,
        structure: score.structure,
        hashes: ContentHashes {
            graph: sha256_hex(&serde_json::to_vec(&graph)?),
            tree: sha256_hex(&serde_json::to_vec(&tree)?),
            scheme: sha256_hex(&scheme_bytes),
        },
        bootstrap: replicates.map(|r| r.summary()),
    };
    let json = serde_json::to_string_pretty(&result)?;
    if let Some(path) = values.get("--output") {
        std::fs::write(path, &json)?;
    } else {
        println!("{json}");
    }
    Ok(())
}

pub(crate) fn read_jsonl<T: serde::de::DeserializeOwned>(
    path: impl AsRef<Path>,
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let reader = BufReader::new(File::open(path)?);
    reader
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            let line = line?;
            serde_json::from_str(&line).map_err(|e| format!("JSONL line {}: {e}", idx + 1).into())
        })
        .collect()
}
