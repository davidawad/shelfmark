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
//!
//! Labels and terms made only of English stopwords are never matched
//! (`vocab-2`); `vocab-1` matched them, so a class labelled `To` drew
//! most inputs.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use shelfmark_core::error::Error;
use shelfmark_core::id::ClassId;
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{Candidate, ClassifyInput, Cost, Evidence, Tier, TierInfo, TierOutput};

use crate::{micros_since, normalized_fields, rank};

mod matcher;
mod persist;
mod stopwords;
mod trie;

pub use matcher::{VocabMatcher, LABEL_SOURCE, QUALIFIER_DISCOUNT};

/// The `vocab` tier. Builds its [`VocabMatcher`] on first use for an
/// index and reuses it until the index's content hash changes. With a
/// cache directory it also saves each matcher there and loads it in later
/// processes instead of rebuilding it.
#[derive(Debug, Default)]
pub struct VocabTier {
    cache: Mutex<Option<Arc<VocabMatcher>>>,
    dir: Option<PathBuf>,
}

impl VocabTier {
    pub const NAME: &'static str = "vocab";
    pub const VERSION: &'static str = "vocab-2";

    pub fn new() -> Self {
        VocabTier::default()
    }

    /// A tier that keeps built matchers in `dir` (created on first save).
    pub fn with_cache_dir(dir: impl Into<PathBuf>) -> Self {
        VocabTier {
            cache: Mutex::default(),
            dir: Some(dir.into()),
        }
    }

    /// A tier with its matcher already built for `index`.
    pub fn for_index(index: &dyn CompiledIndex) -> Result<Self, Error> {
        Ok(VocabTier {
            cache: Mutex::new(Some(Arc::new(VocabMatcher::build(index)?))),
            dir: None,
        })
    }

    /// The matcher for `index`: the one in memory if it is for this index,
    /// else the one saved in the cache directory, else built now (and
    /// saved, ignoring a failed write).
    pub fn matcher(&self, index: &dyn CompiledIndex) -> Result<Arc<VocabMatcher>, Error> {
        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(m) = cache.as_ref().filter(|m| m.is_for(index)) {
            return Ok(Arc::clone(m));
        }
        let saved = self
            .dir
            .as_deref()
            .and_then(|d| VocabMatcher::load(d, index));
        let matcher = match saved {
            Some(m) => m,
            None => {
                let built = VocabMatcher::build(index)?;
                if let Some(dir) = &self.dir {
                    let _ = built.save(dir);
                }
                built
            }
        };
        let matcher = Arc::new(matcher);
        *cache = Some(Arc::clone(&matcher));
        Ok(matcher)
    }

    /// Score `input` with an already-built matcher.
    pub fn score(matcher: &VocabMatcher, input: &ClassifyInput) -> Vec<Candidate> {
        let counts = matcher.count(&normalized_fields(input));
        // Keyed by class number, which follows code byte order, so sums
        // and ties run in code order.
        let mut raw: BTreeMap<u32, f64> = BTreeMap::new();
        let mut evidence: BTreeMap<u32, Vec<Evidence>> = BTreeMap::new();
        for (&pid, &count) in &counts {
            let phrase = &matcher.phrases[pid];
            let tf = 1.0 + f64::from(count).ln();
            for link in &phrase.links {
                *raw.entry(link.class).or_insert(0.0) += link.weight * phrase.idf * tf;
                evidence
                    .entry(link.class)
                    .or_default()
                    .push(Evidence::Vocab {
                        term: phrase.text.clone(),
                        source: matcher.sources[link.source as usize].clone(),
                        count,
                    });
            }
        }

        let ancestors = |c: u32| &matcher.ancestors[c as usize];
        let related = |a: u32, b: u32| ancestors(a).contains(&b) || ancestors(b).contains(&a);
        let mut out: Vec<Candidate> = raw
            .iter()
            .map(|(&class, &own)| {
                let support = own
                    + ancestors(class)
                        .iter()
                        .filter_map(|a| raw.get(a))
                        .sum::<f64>();
                let rival: f64 = raw
                    .iter()
                    .filter(|(&other, _)| other != class && !related(class, other))
                    .map(|(_, r)| r)
                    .sum();
                let score = (1.0 - (-support).exp()) * support / (support + rival);
                let mut ev = evidence.remove(&class).unwrap_or_default();
                ev.sort_by(|a, b| evidence_key(a).cmp(&evidence_key(b)));
                Candidate {
                    class: ClassId::new(matcher.scheme().clone(), &matcher.codes[class as usize]),
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
