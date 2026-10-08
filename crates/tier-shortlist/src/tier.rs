//! [`ShortlistTier`]: coarse-to-fine cosine ranking against a
//! [`LabelIndex`].

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use shelfmark_core::error::Error;
use shelfmark_core::id::ClassId;
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{Candidate, ClassifyInput, Cost, Evidence, Tier, TierInfo, TierOutput};

use crate::embedder::{dot, normalize, Embedder};
use crate::index::LabelIndex;
use crate::{item_text, micros_since, rank};

/// Name and version of the ranking algorithm; part of
/// [`TierInfo::version`]. Bump it whenever scores could change for the
/// same model.
pub const ALGORITHM: &str = "c2f-1";

/// Default softmax temperature over the fine-stage cosines. Cosines from
/// one model sit in a narrow band, so a small temperature is needed for a
/// clear winner to score near 1 while a near-tie still shows as a small
/// margin.
pub const TEMPERATURE: f64 = 0.05;

/// Tuning knobs. [`Params::default`] is the measured `shortlist-lite`
/// configuration; anything else shows up in the tier version.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    /// Level-0 classes kept by the coarse stage (K).
    pub k_classes: usize,
    /// Candidates emitted (N).
    pub n_candidates: usize,
    /// Softmax temperature (τ); must be positive.
    pub temperature: f64,
    /// Characters of title + text embedded per item.
    pub max_chars: usize,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            k_classes: 3,
            n_candidates: 10,
            temperature: TEMPERATURE,
            max_chars: 1500,
        }
    }
}

/// The `shortlist` tier. Builds its [`LabelIndex`] on first use for an
/// index (or loads it from the disk cache) and reuses it until the index's
/// content hash changes.
pub struct ShortlistTier {
    embedder: Arc<dyn Embedder>,
    cache_dir: Option<PathBuf>,
    params: Params,
    labels: Mutex<Option<Arc<LabelIndex>>>,
}

impl ShortlistTier {
    pub const NAME: &'static str = "shortlist";
    /// Declared resident memory (spike: steady 570 MB, peak 572 MB).
    pub const EXPECTED_MEMORY_MB: u32 = 600;
    /// Declared cost per item (spike: p50 46 ms at 4 threads).
    pub const EXPECTED_MICROS_PER_ITEM: u64 = 50_000;

    /// A tier ranking with `embedder`. Label vectors are memoized under
    /// `cache_dir/shortlist/<model_id>/` when a directory is given.
    pub fn new(embedder: Arc<dyn Embedder>, cache_dir: Option<PathBuf>, params: Params) -> Self {
        ShortlistTier {
            embedder,
            cache_dir,
            params,
            labels: Mutex::new(None),
        }
    }

    /// The `shortlist-lite` tier: BGE-small-en-v1.5 int8 with default
    /// params. The model loads on the first [`Tier::run`] that needs it
    /// (not here), once; a failed load makes every run return the same
    /// `tier_unavailable` error.
    #[cfg(feature = "fastembed")]
    pub fn bge_small(cache_dir: Option<PathBuf>) -> Self {
        Self::new(
            Arc::new(crate::LazyBge::new()),
            cache_dir,
            Params::default(),
        )
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn embedder(&self) -> &Arc<dyn Embedder> {
        &self.embedder
    }

    /// The label index for `index`, built (or loaded from disk) now if the
    /// memoized one is for another index or none is memoized.
    pub fn label_index(&self, index: &dyn CompiledIndex) -> Result<Arc<LabelIndex>, Error> {
        let mut memo = self.labels.lock().unwrap_or_else(|e| e.into_inner());
        let model = self.embedder.model_id();
        if let Some(l) = memo.as_ref().filter(|l| l.is_for(index, model)) {
            return Ok(Arc::clone(l));
        }
        let built = Arc::new(LabelIndex::build(
            index,
            self.embedder.as_ref(),
            self.cache_dir.as_deref(),
        )?);
        *memo = Some(Arc::clone(&built));
        Ok(built)
    }

    /// Rank `labels` against the unit item vector `item`: the coarse-to-fine
    /// candidates, best first, with softmax scores.
    pub fn score(labels: &LabelIndex, item: &[f32], params: &Params) -> Vec<Candidate> {
        let ls = labels.labels();
        let cos: Vec<f32> = (0..ls.len()).map(|i| dot(labels.vector(i), item)).collect();
        let by_cos = |a: &usize, b: &usize| {
            cos[*b]
                .total_cmp(&cos[*a])
                .then_with(|| ls[*a].code.as_bytes().cmp(ls[*b].code.as_bytes()))
        };
        let mut tops: Vec<usize> = (0..ls.len()).filter(|&i| ls[i].level == 0).collect();
        let mut fine: Vec<usize> = if ls.iter().any(|l| l.level == 1) {
            tops.sort_by(by_cos);
            tops.truncate(params.k_classes);
            // A kept class without subclasses stands for itself.
            let childless = tops
                .iter()
                .copied()
                .filter(|&t| !ls.iter().any(|l| l.parent == Some(t)));
            ls.iter()
                .enumerate()
                .filter(|(_, l)| l.parent.is_some_and(|p| tops.contains(&p)))
                .map(|(i, _)| i)
                .chain(childless)
                .collect()
        } else {
            tops
        };
        fine.sort_by(by_cos);

        let tau = params.temperature.max(1e-6);
        let best = fine.first().map_or(0.0, |&i| f64::from(cos[i]));
        let weights: Vec<f64> = fine
            .iter()
            .map(|&i| ((f64::from(cos[i]) - best) / tau).exp())
            .collect();
        let total: f64 = weights.iter().sum();
        let mut out: Vec<Candidate> = fine
            .iter()
            .zip(&weights)
            .take(params.n_candidates)
            .map(|(&i, w)| {
                let s = w / total;
                Candidate {
                    class: ClassId::new(labels.scheme().clone(), &ls[i].code),
                    score: if s.is_finite() {
                        s.clamp(0.0, 1.0)
                    } else {
                        0.0
                    },
                    evidence: vec![Evidence::Similarity {
                        model: labels.model_id().to_string(),
                    }],
                }
            })
            .collect();
        out.sort_by(rank);
        out
    }
}

impl Tier for ShortlistTier {
    fn info(&self) -> TierInfo {
        let mut version = format!("{}/{ALGORITHM}", self.embedder.model_id());
        let p = self.params;
        if p != Params::default() {
            version.push_str(&format!(
                "+k{}n{}t{}c{}",
                p.k_classes, p.n_candidates, p.temperature, p.max_chars
            ));
        }
        TierInfo {
            name: Self::NAME.into(),
            version,
            needs_model: true,
            expected_memory_mb: Self::EXPECTED_MEMORY_MB,
            expected_micros_per_item: Self::EXPECTED_MICROS_PER_ITEM,
        }
    }

    fn run(
        &self,
        input: &ClassifyInput,
        index: &dyn CompiledIndex,
        _prior: &[Candidate],
    ) -> Result<TierOutput, Error> {
        let start = Instant::now();
        let text = item_text(input, self.params.max_chars);
        if text.is_empty() {
            return Ok(TierOutput {
                candidates: Vec::new(),
                cost: Cost {
                    micros: micros_since(start),
                    model_calls: 0,
                },
            });
        }
        let labels = self.label_index(index)?;
        if labels.labels().is_empty() {
            return Ok(TierOutput {
                candidates: Vec::new(),
                cost: Cost {
                    micros: micros_since(start),
                    model_calls: 0,
                },
            });
        }
        let mut item = self
            .embedder
            .embed_items(std::slice::from_ref(&text))?
            .pop()
            .unwrap_or_default();
        if item.len() != labels.dim() {
            return Err(Error::internal(format!(
                "embedder {} returned an item vector of dimension {} against label dimension {}",
                labels.model_id(),
                item.len(),
                labels.dim()
            )));
        }
        normalize(&mut item);
        Ok(TierOutput {
            candidates: Self::score(&labels, &item, &self.params),
            cost: Cost {
                micros: micros_since(start),
                model_calls: 1,
            },
        })
    }
}
