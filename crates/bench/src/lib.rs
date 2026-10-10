//! Reproducible scheme benchmark primitives. The similarity graph is built
//! once and shared by all scheme trees in a run.
mod graph;
mod metrics;
mod tree;

pub use graph::{build_knn_graph, DeterministicEmbedder, Edge, EmbeddingModel, SimilarityGraph};
pub use metrics::{score_tree, StructuralStats, TreeScore};
pub use tree::{induce_tree, ItemPlacement, Tree, TreeNode};

use serde::{Deserialize, Serialize};

/// Input row compatible with the Paper 1 arXiv JSONL, and permissive of
/// Open Library holdout rows. Placement files can supply scheme-specific
/// classes by id without changing the source corpus.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CorpusRow {
    #[serde(alias = "key", alias = "edition_key")]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "abstract", alias = "abstract_text")]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    #[serde(alias = "msc")]
    pub msc_classes: Vec<String>,
    #[serde(default)]
    pub acm_ccs: Vec<String>,
    #[serde(default)]
    #[serde(alias = "jel")]
    pub jel_codes: Vec<String>,
    #[serde(default)]
    pub ddc: Vec<String>,
    #[serde(default)]
    pub lc_classifications: Vec<String>,
    #[serde(default)]
    pub gold: std::collections::BTreeMap<String, Vec<String>>,
}

impl CorpusRow {
    pub fn text(&self) -> String {
        format!("{}. {}", self.title, self.description)
    }
}

/// Embeds corpus text with a caller-selected model, then builds its fixed graph.
pub fn embed_corpus<M: EmbeddingModel>(rows: &[CorpusRow], model: &M, k: usize) -> SimilarityGraph {
    let vectors: Vec<_> = rows.iter().map(|row| model.embed(&row.text())).collect();
    build_knn_graph(&vectors, k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_synthetic_arxiv_and_open_library_shapes() {
        let arxiv: CorpusRow = serde_json::from_str(r#"{"id":"2601.00001","title":"Synthetic paper","abstract_text":"Tiny abstract","authors":["A. Author"],"categories":["cs.AI"],"msc":["68T01"],"acm_ccs":["10002951.10003317"],"jel":["C80"]}"#).unwrap();
        let holdout: CorpusRow = serde_json::from_str(
            r#"{"id":"OL123M","title":"Synthetic book","lc_classifications":["QA76.1"]}"#,
        )
        .unwrap();
        assert_eq!(arxiv.text(), "Synthetic paper. Tiny abstract");
        assert_eq!(arxiv.msc_classes, ["68T01"]);
        assert_eq!(holdout.lc_classifications, ["QA76.1"]);
    }
}

/// JSON result record for one scheme and placement track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub scheme: String,
    pub version: String,
    pub track: String,
    pub mode: String,
    pub n: usize,
    pub k: usize,
    pub edge_weight: f64,
    pub cost: f64,
    pub cost_norm: f64,
    pub revenue: f64,
    pub revenue_norm: f64,
    pub ref_cost_norm: f64,
    pub skill: f64,
    pub coverage: f64,
    pub structure: StructuralStats,
    pub hashes: ContentHashes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHashes {
    pub graph: String,
    pub tree: String,
    pub scheme: String,
}

/// Lowercase SHA-256 digest for stable stage/result provenance.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
