//! `dense` subcommand: Dasgupta cost of every run's tree on the dense
//! similarity matrix of random item subsets against kNN sparsifications of
//! the same matrix, and the rank agreement of the two scheme rankings
//! (`shelfmark_bench::dense` defines the weights and the bootstrap).
use crate::cli_scores::{need, number, options, placement_labels, vectors, write, Res};
use crate::{gold_field, read_jsonl};
use shelfmark_bench::agree::{kendall, spearman};
use shelfmark_bench::dense::{
    cosines, knn_positions, lca_sizes, pair_index, positive_pairs, PairCosts,
};
use shelfmark_bench::embedscore::Vectors;
use shelfmark_bench::robust::interval;
use shelfmark_bench::subset::{restrict_graph, restrict_vectors, sample_without_replacement};
use shelfmark_bench::treescore::Rng;
use shelfmark_bench::{CorpusRow, Hier, SimilarityGraph};
use shelfmark_core::scheme::SchemeDoc;
use std::collections::BTreeMap;

/// One run: its scheme, the corpus-format file its labels come from (the
/// gold corpus for gold runs), optional placements, and whether it enters
/// the "placed" ranking.
#[derive(serde::Deserialize)]
struct RunSpec {
    key: String,
    scheme: String,
    corpus: String,
    #[serde(default)]
    placements: Option<String>,
    #[serde(default)]
    placed: bool,
}

struct Run {
    key: String,
    scheme: SchemeDoc,
    primary: Vec<Option<String>>,
    placed: bool,
}

fn load_runs(path: &str, n: usize) -> Res<Vec<Run>> {
    let specs: Vec<RunSpec> = serde_json::from_slice(&std::fs::read(path)?)?;
    let mut corpora: BTreeMap<String, Vec<CorpusRow>> = BTreeMap::new();
    let mut out = vec![];
    for s in specs {
        if !corpora.contains_key(&s.corpus) {
            corpora.insert(s.corpus.clone(), read_jsonl(&s.corpus)?);
        }
        let corpus = &corpora[&s.corpus];
        if corpus.len() != n {
            return Err(format!("{}: {} rows, graph has {n}", s.corpus, corpus.len()).into());
        }
        let scheme: SchemeDoc = serde_json::from_slice(&std::fs::read(&s.scheme)?)?;
        let labels = match &s.placements {
            Some(p) => placement_labels(p, corpus)?,
            None => corpus
                .iter()
                .map(|r| gold_field(r, &scheme.name).to_vec())
                .collect(),
        };
        out.push(Run {
            key: s.key,
            scheme,
            primary: labels.iter().map(|l| l.first().cloned()).collect(),
            placed: s.placed,
        });
    }
    Ok(out)
}

/// Spearman and Kendall between the dense and a sparse cost over the runs
/// in `set`.
fn agreement(dense: &[f64], sparse: &[f64], set: &[usize]) -> [f64; 2] {
    let x: Vec<f64> = set.iter().map(|&r| dense[r]).collect();
    let y: Vec<f64> = set.iter().map(|&r| sparse[r]).collect();
    [spearman(&x, &y), kendall(&x, &y)]
}

/// Selection names: dense, each kNN k, and the published corpus graph
/// restricted to the subset.
fn selection_names(ks: &[usize]) -> Vec<String> {
    let mut names = vec!["dense".to_owned()];
    names.extend(ks.iter().map(|k| format!("knn{k}")));
    names.push("graph-restricted".to_owned());
    names
}

struct Setup<'a> {
    runs: &'a [Run],
    vectors: &'a Vectors,
    graph: &'a SimilarityGraph,
    ks: &'a [usize],
    seed: u64,
    threads: usize,
}

/// Every cost and agreement for one item subset; `boot` replicates (0 = none).
fn evaluate(st: &Setup, keep: &[usize], boot: usize) -> Res<serde_json::Value> {
    let n = keep.len();
    let v = restrict_vectors(st.vectors, keep);
    let cos = cosines(&v, st.threads);
    let positive = positive_pairs(&cos, n, st.graph.tau);
    let mut selections: Vec<Option<Vec<usize>>> = vec![None];
    let mut edges = vec![positive.pair.len()];
    for &k in st.ks {
        let s = knn_positions(&cos, n, k, &positive);
        edges.push(s.len());
        selections.push(Some(s));
    }
    let restricted = restrict_graph(st.graph, keep);
    let published: Vec<usize> = restricted
        .edges
        .iter()
        .filter_map(|e| {
            let x = pair_index(n, e.left.min(e.right), e.left.max(e.right));
            positive.pair.binary_search(&x).ok()
        })
        .collect();
    edges.push(published.len());
    selections.push(Some(published));
    let trees = st
        .runs
        .iter()
        .map(|r| {
            let primary: Vec<Option<String>> = keep.iter().map(|&i| r.primary[i].clone()).collect();
            Hier::build(&r.scheme, &primary).map_err(|e| format!("{}: {e}", r.key))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let sizes = lca_sizes(&trees, &positive, st.threads);
    let pc = PairCosts {
        positive: &positive,
        sizes: &sizes,
        trees: trees.len(),
        n,
    };
    let point: Vec<Vec<f64>> = selections
        .iter()
        .map(|s| pc.costs(s.as_deref(), st.seed, u64::MAX))
        .collect();
    let reps = pc.boot(&selections, st.seed, boot, st.threads);
    let names = selection_names(st.ks);
    let all: Vec<usize> = (0..st.runs.len()).collect();
    let placed: Vec<usize> = all.iter().copied().filter(|&r| st.runs[r].placed).collect();
    let mut agree = serde_json::Map::new();
    for (s, name) in names.iter().enumerate().skip(1) {
        let mut sets = serde_json::Map::new();
        for (set_name, set) in [("placed", &placed), ("all", &all)] {
            let [rho, tau] = agreement(&point[0], &point[s], set);
            let (rhos, taus): (Vec<f64>, Vec<f64>) = reps
                .iter()
                .map(|r| {
                    let [a, b] = agreement(&r[0], &r[s], set);
                    (a, b)
                })
                .unzip();
            sets.insert(
                set_name.to_owned(),
                serde_json::json!({
                    "runs": set.len(), "spearman": rho, "kendall": tau,
                    "spearman_ci": (!rhos.is_empty()).then(|| interval(&rhos)),
                    "kendall_ci": (!taus.is_empty()).then(|| interval(&taus)),
                }),
            );
        }
        agree.insert(name.clone(), serde_json::Value::Object(sets));
    }
    let mut cost = serde_json::Map::new();
    for (r, run) in st.runs.iter().enumerate() {
        let mut per = serde_json::Map::new();
        for (s, name) in names.iter().enumerate() {
            let values: Vec<f64> = reps.iter().map(|x| x[s][r]).collect();
            per.insert(
                name.clone(),
                serde_json::json!({
                    "cost_norm": point[s][r],
                    "ci": (!values.is_empty()).then(|| interval(&values)),
                }),
            );
        }
        let coverage = (0..n).filter(|&i| trees[r].placed(i)).count() as f64 / n as f64;
        per.insert("coverage".into(), coverage.into());
        cost.insert(run.key.clone(), serde_json::Value::Object(per));
    }
    Ok(serde_json::json!({
        "n": n, "pairs": n * (n - 1) / 2,
        "edges": names.iter().cloned().zip(edges).collect::<BTreeMap<_, _>>(),
        "boot": boot, "costs": cost, "agreement": agree,
    }))
}

fn list(o: &BTreeMap<String, String>, key: &str, default: &str) -> Res<Vec<usize>> {
    o.get(key)
        .map_or(default, String::as_str)
        .split(',')
        .map(|x| Ok(x.trim().parse()?))
        .collect()
}

pub fn dense(args: &[String]) -> Res<()> {
    let o = options(args)?;
    let graph: SimilarityGraph = serde_json::from_slice(&std::fs::read(need(&o, "graph")?)?)?;
    let total = graph.n;
    let vecs = vectors(need(&o, "vectors")?, number(&o, "dims", 384)?, total)?;
    let runs = load_runs(need(&o, "runs")?, total)?;
    let ks = list(&o, "ks", "10,20,50")?;
    let st = Setup {
        runs: &runs,
        vectors: &vecs,
        graph: &graph,
        ks: &ks,
        seed: number(&o, "seed", 20_261_010)?,
        threads: number(&o, "threads", 8)?,
    };
    let boot: usize = number(&o, "boot", 200)?;
    let subsets: u64 = number(&o, "subsets", 5)?;
    let mut sizes = serde_json::Map::new();
    for n in list(&o, "sizes", "1000,2000,5000")? {
        let mut draws = vec![];
        // A subset as large as the corpus is the corpus: one draw.
        let count = if n >= total { 1 } else { subsets };
        for s in 0..count {
            let mut rng = Rng::new(st.seed ^ ((n as u64) << 20) ^ s);
            let keep = sample_without_replacement(total, n, &mut rng);
            // Replicates for the first draw only; later draws give the
            // spread over subsets.
            let mut r = evaluate(&st, &keep, if s == 0 { boot } else { 0 })?;
            r["subset"] = s.into();
            draws.push(r);
            eprintln!("dense: n={n} subset {s} done");
        }
        sizes.insert(n.min(total).to_string(), serde_json::Value::Array(draws));
    }
    let out = serde_json::json!({
        "tau": graph.tau, "corpus_n": total, "ks": ks, "seed": st.seed, "boot": boot,
        "subsets": subsets, "runs": runs.iter().map(|r| &r.key).collect::<Vec<_>>(),
        "placed": runs.iter().filter(|r| r.placed).map(|r| &r.key).collect::<Vec<_>>(),
        "sizes": sizes,
    });
    write(&o, &serde_json::to_string_pretty(&out)?)
}
