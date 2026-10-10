use shelfmark_bench::{
    embed_corpus, induce_tree, score_tree, sha256_hex, BenchmarkResult, ContentHashes, CorpusRow,
    DeterministicEmbedder, ItemPlacement,
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

fn main() {
    if let Err(error) = run() {
        eprintln!("shelfmark-bench: {error}");
        std::process::exit(2);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 6 || args[1] != "run" {
        return Err("usage: shelfmark-bench run --corpus corpus.jsonl --scheme scheme.json [--placements placements.jsonl] [--k 20] [--output results.json]".into());
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
    let corpus_path = values.get("--corpus").ok_or("missing --corpus")?;
    let scheme_path = values.get("--scheme").ok_or("missing --scheme")?;
    let corpus: Vec<CorpusRow> = read_jsonl(corpus_path)?;
    let scheme_bytes = std::fs::read(scheme_path)?;
    let scheme: SchemeDoc = serde_json::from_slice(&scheme_bytes)?;
    let mut placements = BTreeMap::<String, Vec<(String, f64)>>::new();
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
        for row in &corpus {
            let labels = match scheme.name.to_ascii_lowercase().as_str() {
                "lcc" => row.lc_classifications.as_slice(),
                "ddc" => row.ddc.as_slice(),
                "msc" => row.msc_classes.as_slice(),
                "acm" | "acm-ccs" | "acm_ccs" => row.acm_ccs.as_slice(),
                "jel" => row.jel_codes.as_slice(),
                "arxiv" => row.categories.as_slice(),
                name => row.gold.get(name).map(Vec::as_slice).unwrap_or_default(),
            };
            if let Some(code) = labels.first() {
                placements.insert(row.id.clone(), vec![(code.clone(), 1.0)]);
            }
        }
    }
    let graph = if let Some(path) = values.get("--graph") {
        let graph: shelfmark_bench::SimilarityGraph =
            serde_json::from_slice(&std::fs::read(path)?)?;
        if graph.n != corpus.len() {
            return Err(
                format!("graph has {} rows but corpus has {}", graph.n, corpus.len()).into(),
            );
        }
        graph
    } else {
        let model = DeterministicEmbedder::default();
        embed_corpus(&corpus, &model, k)
    };
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
    let ref_score = if let Some(value) = values.get("--reference-cost") {
        value.parse()?
    } else {
        let hac = average_linkage_reference(&graph);
        score_tree(&hac, &graph).cost_norm
    };
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
    let result = BenchmarkResult {
        scheme: scheme.name,
        version: scheme.version,
        track: if values.contains_key("--placements") {
            "P"
        } else {
            "A"
        }
        .into(),
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
    };
    let json = serde_json::to_string_pretty(&result)?;
    if let Some(path) = values.get("--output") {
        std::fs::write(path, &json)?;
    } else {
        println!("{json}");
    }
    Ok(())
}

fn read_jsonl<T: serde::de::DeserializeOwned>(
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

/// Deterministic average-linkage HAC. Missing graph edges have similarity 0.
fn average_linkage_reference(graph: &shelfmark_bench::SimilarityGraph) -> shelfmark_bench::Tree {
    use shelfmark_bench::Tree;
    use std::cmp::Ordering;
    use std::collections::{BTreeSet, BinaryHeap};
    #[derive(Clone, Copy, Debug)]
    struct Candidate(f64, usize, usize);
    impl PartialEq for Candidate {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0 && self.1 == other.1 && self.2 == other.2
        }
    }
    impl Eq for Candidate {}
    impl PartialOrd for Candidate {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Candidate {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0
                .total_cmp(&other.0)
                .then_with(|| other.1.cmp(&self.1))
                .then_with(|| other.2.cmp(&self.2))
        }
    }
    let n = graph.n;
    let mut tree = Tree::new();
    tree.item_leaves = vec![vec![]; n];
    if n == 0 {
        return tree;
    }
    let mut cluster_nodes: Vec<usize> = (0..n)
        .map(|i| {
            let leaf = tree.add_node(0, None, Some(i));
            tree.item_leaves[i] = vec![(leaf, 1.0)];
            leaf
        })
        .collect();
    let mut sizes = vec![1usize; n];
    let mut similarities = BTreeMap::<(usize, usize), f64>::new();
    let mut candidates = BinaryHeap::new();
    for e in &graph.edges {
        let pair = (e.left.min(e.right), e.left.max(e.right));
        similarities.insert(pair, e.weight);
        if e.weight > 0.0 {
            candidates.push(Candidate(e.weight, pair.0, pair.1));
        }
    }
    let mut active: BTreeSet<usize> = (0..n).collect();
    let mut next_id = n;
    while active.len() > 1 {
        // Only similarities touched by a positive edge need heap entries.
        // If none remain, deterministic zero-similarity merges complete the tree.
        let best = loop {
            match candidates.pop() {
                Some(Candidate(score, a, b)) if active.contains(&a) && active.contains(&b) => {
                    break (score, a, b);
                }
                Some(_) => continue,
                None => {
                    let mut ids = active.iter().copied();
                    break (0.0, ids.next().unwrap(), ids.next().unwrap());
                }
            }
        };
        let (a, b) = (best.1, best.2);
        let node = tree.add_node(0, None, None); // reparent merged subtree under its merge node
        for child in [cluster_nodes[a], cluster_nodes[b]] {
            tree.nodes[0].children.retain(|&x| x != child);
            tree.nodes[child].parent = Some(node);
            tree.nodes[node].children.push(child);
        }
        let combined_size = sizes[a] + sizes[b];
        active.remove(&a);
        active.remove(&b);
        for &other in &active {
            let left = similarities
                .get(&(a.min(other), a.max(other)))
                .copied()
                .unwrap_or(0.0);
            let right = similarities
                .get(&(b.min(other), b.max(other)))
                .copied()
                .unwrap_or(0.0);
            let similarity =
                (sizes[a] as f64 * left + sizes[b] as f64 * right) / combined_size as f64;
            let pair = (next_id.min(other), next_id.max(other));
            similarities.insert(pair, similarity);
            if similarity > 0.0 {
                candidates.push(Candidate(similarity, pair.0, pair.1));
            }
        }
        active.insert(next_id);
        cluster_nodes.push(node);
        sizes.push(combined_size);
        next_id += 1;
    }
    tree
}

#[cfg(test)]
mod tests {
    use super::*;
    use shelfmark_bench::Edge;
    use shelfmark_bench::{score_tree, SimilarityGraph};

    #[test]
    fn average_linkage_merges_the_highest_similarity_pair_first() {
        let graph = SimilarityGraph {
            n: 3,
            k: 1,
            tau: 0.0,
            edges: vec![
                Edge {
                    left: 0,
                    right: 1,
                    weight: 4.0,
                },
                Edge {
                    left: 1,
                    right: 2,
                    weight: 1.0,
                },
            ],
        };
        let tree = average_linkage_reference(&graph);
        let score = score_tree(&tree, &graph);
        assert_eq!(score.cost, 4.0 * 2.0 + 1.0 * 3.0);
        assert_eq!(tree, average_linkage_reference(&graph));
    }
}
