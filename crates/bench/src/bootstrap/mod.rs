//! Bootstrap resampling for every benchmark command. Three schemes share
//! one generator and one interval rule, and each keeps its own draw order,
//! so a published replicate is reproduced bit for bit:
//!
//! * [`weighted`]: `run --bootstrap N`. Items drawn with replacement act
//!   as weights on the fixed graph and tree; reports percentile intervals
//!   in the result record and replicates for paired skill.
//! * [`resample`]: the `bootstrap` subcommand. Each draw rebuilds the tree
//!   and graph from the drawn copies and rescores several runs and the
//!   average-linkage reference on it; reports paired differences.
//! * [`catalogue`]: `scores --boot N`. Edge, vertex and item bootstraps of
//!   the catalogue scores (graph, external-gold and kNN-probe scores).
pub mod catalogue;
pub mod resample;
pub mod weighted;

pub use resample::{
    bootstrap, draws, metric_values, resample_graph, resample_tree, BootstrapRun, BootstrapSamples,
};
pub use weighted::{multiplicities, run, BootstrapSummary, Prepared, Replicate, Replicates};

/// Edge-bootstrap confidence intervals on fixed-tree graph scores. Uses the
/// shared seeded generator and percentile rule used by every bench API.
pub fn edge_bootstrap(
    tree: &crate::Tree,
    graph: &crate::SimilarityGraph,
    reference_cost: f64,
    resamples: usize,
    seed: u64,
) -> BootstrapSummary {
    use crate::{score_tree, Edge};
    let mut rng = SplitMix64::new(seed ^ 0x4544_4745);
    let mut values = std::collections::BTreeMap::<String, Vec<f64>>::new();
    for _ in 0..resamples {
        let edges: Vec<Edge> = (0..graph.edges.len())
            .filter_map(|_| {
                graph
                    .edges
                    .get(rng.below(graph.edges.len().max(1)))
                    .cloned()
            })
            .collect();
        let sample = crate::SimilarityGraph {
            n: graph.n,
            k: graph.k,
            tau: graph.tau,
            edges,
        };
        let score = score_tree(tree, &sample);
        let skill = if reference_cost < 1.0 {
            (1.0 - score.cost_norm) / (1.0 - reference_cost)
        } else {
            0.0
        };
        for (name, value) in [
            ("cost", score.cost),
            ("revenue", score.revenue),
            ("edge_weight", score.edge_weight),
            ("cost_norm", score.cost_norm),
            ("revenue_norm", score.revenue_norm),
            ("expected_tree_distance", score.expected_tree_distance),
            ("skill", skill),
        ] {
            values.entry(name.into()).or_default().push(value);
        }
    }
    BootstrapSummary {
        resamples,
        seed,
        confidence: 0.95,
        method: "percentile; graph edges resampled with replacement on the fixed tree".into(),
        ci: values
            .iter()
            .map(|(name, vals)| (name.clone(), interval(vals, 0.95)))
            .collect(),
    }
}

/// Default seed of the benchmark bootstrap.
pub const DEFAULT_SEED: u64 = 20_261_010;

/// SplitMix64: tiny, well-distributed, and stable across platforms.
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        SplitMix64(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n` (Lemire's multiply-shift; the bias is negligible here).
    pub fn below(&mut self, n: usize) -> usize {
        ((u128::from(self.next_u64()) * n as u128) >> 64) as usize
    }
}

/// Linear-interpolation percentile of sorted values, `q` in `[0, 1]`.
pub fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let pos = q.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

/// Two-sided percentile interval of the finite values at `confidence`
/// (0.95 gives 2.5% and 97.5%).
pub fn interval(values: &[f64], confidence: f64) -> [f64; 2] {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    sorted.sort_by(f64::total_cmp);
    let alpha = (1.0 - confidence) / 2.0;
    [percentile(&sorted, alpha), percentile(&sorted, 1.0 - alpha)]
}
