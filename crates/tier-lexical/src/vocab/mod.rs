//! The `vocab` tier: scheme labels and vocabulary links matched against
//! the input text.
//!
//! Scoring, per matched class `c`:
//!
//! - `raw(c)` sums, over each phrase that hit and links to `c`,
//!   `link weight x phrase idf x (1 + ln count)`.
//! - `support(c)` is `raw(c)` plus the raw of every matched ancestor:
//!   broader classes on the same path agree with `c` rather than compete,
//!   so the deepest consistent class ranks first.
//! - `rival(c)` is the raw of every matched class neither above nor
//!   below `c`.
//! - `score(c) = (1 - e^-support) x support / (support + rival)`: the first
//!   factor grows with the amount of evidence, the second is the share
//!   against unrelated classes, so two classes matched equally both score
//!   at most one half and the cascade flags the decision (AE2).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use shelfmark_core::error::Error;
use shelfmark_core::id::ClassId;
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{Candidate, ClassifyInput, Cost, Evidence, Tier, TierInfo, TierOutput};

use crate::{micros_since, normalized_fields, rank};

mod matcher;

pub use matcher::{VocabMatcher, LABEL_SOURCE, QUALIFIER_DISCOUNT};

/// The `vocab` tier. Builds its [`VocabMatcher`] on first use for an
/// index and reuses it until the index's content hash changes.
#[derive(Debug, Default)]
pub struct VocabTier {
    cache: Mutex<Option<Arc<VocabMatcher>>>,
}

impl VocabTier {
    pub const NAME: &'static str = "vocab";
    pub const VERSION: &'static str = "vocab-1";

    pub fn new() -> Self {
        VocabTier::default()
    }

    /// A tier with its matcher already built for `index`.
    pub fn for_index(index: &dyn CompiledIndex) -> Result<Self, Error> {
        Ok(VocabTier {
            cache: Mutex::new(Some(Arc::new(VocabMatcher::build(index)?))),
        })
    }

    /// The matcher for `index`, built now if the cached one is for another
    /// index or none is cached.
    pub fn matcher(&self, index: &dyn CompiledIndex) -> Result<Arc<VocabMatcher>, Error> {
        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(m) = cache.as_ref().filter(|m| m.is_for(index)) {
            return Ok(Arc::clone(m));
        }
        let built = Arc::new(VocabMatcher::build(index)?);
        *cache = Some(Arc::clone(&built));
        Ok(built)
    }

    /// Score `input` with an already-built matcher.
    pub fn score(matcher: &VocabMatcher, input: &ClassifyInput) -> Vec<Candidate> {
        let counts = matcher.count(&normalized_fields(input));
        let mut raw: BTreeMap<&str, f64> = BTreeMap::new();
        let mut evidence: BTreeMap<&str, Vec<Evidence>> = BTreeMap::new();
        for (&pid, &count) in &counts {
            let phrase = &matcher.phrases[pid];
            let tf = 1.0 + f64::from(count).ln();
            for link in &phrase.links {
                *raw.entry(&link.class).or_insert(0.0) += link.weight * phrase.idf * tf;
                evidence
                    .entry(&link.class)
                    .or_default()
                    .push(Evidence::Vocab {
                        term: phrase.text.clone(),
                        source: link.source.clone(),
                        count,
                    });
            }
        }

        let no_ancestors = Vec::new();
        let ancestors = |c: &str| matcher.ancestors.get(c).unwrap_or(&no_ancestors);
        let related = |a: &str, b: &str| {
            ancestors(a).iter().any(|x| x == b) || ancestors(b).iter().any(|x| x == a)
        };
        let mut out: Vec<Candidate> = raw
            .iter()
            .map(|(&code, &own)| {
                let support = own
                    + ancestors(code)
                        .iter()
                        .filter_map(|a| raw.get(a.as_str()))
                        .sum::<f64>();
                let rival: f64 = raw
                    .iter()
                    .filter(|(&other, _)| other != code && !related(code, other))
                    .map(|(_, r)| r)
                    .sum();
                let score = (1.0 - (-support).exp()) * support / (support + rival);
                let mut ev = evidence.remove(code).unwrap_or_default();
                ev.sort_by(|a, b| evidence_key(a).cmp(&evidence_key(b)));
                Candidate {
                    class: ClassId::new(matcher.scheme().clone(), code),
                    score: if score.is_finite() {
                        score.clamp(0.0, 1.0)
                    } else {
                        0.0
                    },
                    evidence: ev,
                }
            })
            .collect();
        out.sort_by(rank);
        out
    }
}

fn evidence_key(e: &Evidence) -> (&str, &str) {
    match e {
        Evidence::Vocab { term, source, .. } => (term, source),
        _ => ("", ""),
    }
}

impl Tier for VocabTier {
    fn info(&self) -> TierInfo {
        TierInfo {
            name: Self::NAME.into(),
            version: Self::VERSION.into(),
            needs_model: false,
            expected_memory_mb: 64,
            expected_micros_per_item: 500,
        }
    }

    fn run(
        &self,
        input: &ClassifyInput,
        index: &dyn CompiledIndex,
        _prior: &[Candidate],
    ) -> Result<TierOutput, Error> {
        let start = Instant::now();
        let matcher = self.matcher(index)?;
        let candidates = Self::score(&matcher, input);
        Ok(TierOutput {
            candidates,
            cost: Cost {
                micros: micros_since(start),
                model_calls: 0,
            },
        })
    }
}
