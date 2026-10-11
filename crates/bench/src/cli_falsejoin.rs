//! `shelfmark-bench negatives`: the seeded kNN-complement negative sample of
//! the false-join term (`shelfmark_bench::falsejoin`), written as a graph
//! file whose edge weights are dissimilarities. `bootstrap --negatives`
//! scores runs on it.
use crate::cli_scores::{need, number, options};
use serde_json::json;
use shelfmark_bench::bootstrap::DEFAULT_SEED;
use shelfmark_bench::falsejoin::{negative_sample, NegativeSpec, DEFAULT_PER_ITEM};
use shelfmark_bench::{sha256_hex, SimilarityGraph};

const USAGE: &str = "usage: shelfmark-bench negatives --graph graph.json --vectors vectors.f32 \
[--floor <graph tau>] [--per-item 10] [--seed 20261010] --output negatives.json";

type Error = Box<dyn std::error::Error>;

/// Little-endian f32 rows, `n` of them.
fn read_vectors(path: &str, n: usize) -> Result<(Vec<f32>, usize), Error> {
    let bytes = std::fs::read(path)?;
    if n == 0 || bytes.len() % (4 * n) != 0 {
        return Err(format!("{path}: {} bytes is not {n} rows of f32", bytes.len()).into());
    }
    let vectors: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    let dims = vectors.len() / n;
    Ok((vectors, dims))
}

pub fn negatives(args: &[String]) -> Result<(), Error> {
    let values = options(args).map_err(|e| format!("{e}\n{USAGE}"))?;
    let need = |key| need(&values, key).map_err(|e| format!("{e}\n{USAGE}"));
    let (graph_path, vectors_path, output) = (need("graph")?, need("vectors")?, need("output")?);
    let graph_bytes = std::fs::read(graph_path)?;
    let graph: SimilarityGraph = serde_json::from_slice(&graph_bytes)?;
    let spec = NegativeSpec {
        floor: number(&values, "floor", graph.tau)?,
        per_item: number(&values, "per-item", DEFAULT_PER_ITEM)?,
        seed: number(&values, "seed", DEFAULT_SEED)?,
    };
    let (vectors, dims) = read_vectors(vectors_path, graph.n)?;
    let negatives = negative_sample(&vectors, dims, &graph, &spec);
    let text = serde_json::to_string(&negatives)?;
    std::fs::write(output, &text)?;
    let weight: f64 = negatives.edges.iter().map(|e| e.weight).sum();
    println!(
        "{}",
        json!({
            "n": graph.n,
            "dimensions": dims,
            "graph_sha256": sha256_hex(&graph_bytes),
            "vectors_sha256": sha256_hex(&std::fs::read(vectors_path)?),
            "negatives_sha256": sha256_hex(text.as_bytes()),
            "spec": spec,
            "pairs": negatives.edges.len(),
            "mean_dissimilarity": weight / negatives.edges.len().max(1) as f64,
        })
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vectors_must_fill_whole_rows() {
        let dir = std::env::temp_dir().join(format!("negatives-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("v.f32");
        let rows: Vec<u8> = [1.0f32, 0.0, 0.0, 1.0, 0.6, 0.8]
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect();
        std::fs::write(&path, &rows).unwrap();
        let p = path.to_str().unwrap();
        let (v, dims) = read_vectors(p, 3).unwrap();
        assert_eq!((v.len(), dims, v[4]), (6, 2, 0.6));
        assert!(read_vectors(p, 4).is_err());
        assert!(read_vectors(p, 0).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
