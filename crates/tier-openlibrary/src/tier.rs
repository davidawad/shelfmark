//! [`OpenLibraryTier`]: search, match, read call numbers, map.

use std::collections::BTreeMap;
use std::time::Instant;

use shelfmark_core::error::Error;
use shelfmark_core::id::ClassId;
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{Candidate, ClassifyInput, Cost, Evidence, Tier, TierInfo, TierOutput};

use crate::callnumber::Mapper;
use crate::client::{Client, Doc, Query};
use crate::matching::{ranked, DEFAULT_THRESHOLD};
use crate::{CATALOG, NAME, VERSION};

/// Most candidates one run returns.
const MAX_CANDIDATES: usize = 5;

/// Looks the input's title and authors up on Open Library and returns
/// the classes the best matching work's LC call numbers place.
///
/// An input without a title yields nothing and sends no request. Each
/// class the matched work's call numbers reach scores
/// `match * (0.5 + 0.5 * share)`, where `share` is the fraction of the
/// work's placed call numbers in that class: one class across every
/// edition scores the match itself, a split between, say, PR and PZ
/// scores each lower. Evidence names the work key and the first call
/// number seen for the class. When Open Library cannot be reached the
/// run fails `tier_unavailable` and the cascade carries on without it.
pub struct OpenLibraryTier {
    client: Client,
    threshold: f64,
}

impl OpenLibraryTier {
    pub const NAME: &'static str = NAME;

    pub fn new(client: Client) -> Self {
        OpenLibraryTier {
            client,
            threshold: DEFAULT_THRESHOLD,
        }
    }

    /// The tier over openlibrary.org, caching responses in `cache_dir`.
    #[cfg(feature = "http")]
    pub fn online(cache_dir: Option<std::path::PathBuf>) -> Self {
        OpenLibraryTier::new(Client::new(
            Box::new(crate::http::UreqTransport::new()),
            cache_dir.map(crate::cache::DiskCache::new),
        ))
    }

    /// Accept matches scoring at least `threshold` (default
    /// [`DEFAULT_THRESHOLD`]).
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    /// The query for `input`, if it has a title.
    pub fn query(input: &ClassifyInput) -> Option<Query> {
        let title = input
            .title
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())?;
        Some(Query {
            title: title.to_string(),
            authors: input.authors.clone(),
        })
    }

    /// Candidates for already-fetched search results.
    pub fn candidates(
        &self,
        query: &Query,
        docs: &[Doc],
        index: &dyn CompiledIndex,
    ) -> Vec<Candidate> {
        let mut mapper = Mapper::new(index);
        for m in ranked(query, docs, self.threshold) {
            let tally = tally(m.doc, &mut mapper);
            let total: u32 = tally.values().map(|t| t.count).sum();
            if total == 0 {
                continue;
            }
            let mut out: Vec<Candidate> = tally
                .into_values()
                .map(|t| Candidate {
                    class: t.class,
                    score: (m.score * (0.5 + 0.5 * f64::from(t.count) / f64::from(total)))
                        .clamp(0.0, 1.0),
                    evidence: vec![Evidence::Catalog {
                        catalog: CATALOG.into(),
                        record: m.doc.key.clone(),
                        call_number: t.call_number,
                        similarity: m.score,
                    }],
                })
                .collect();
            out.sort_by(|a, b| {
                b.score
                    .total_cmp(&a.score)
                    .then_with(|| a.class.code.as_bytes().cmp(b.class.code.as_bytes()))
            });
            out.truncate(MAX_CANDIDATES);
            return out;
        }
        Vec::new()
    }
}

struct Tally {
    class: ClassId,
    count: u32,
    call_number: String,
}

/// Placed call numbers of `doc` per class code. Raw
/// `lc_classifications` come first, so evidence quotes them when given.
fn tally(doc: &Doc, mapper: &mut Mapper<'_>) -> BTreeMap<String, Tally> {
    let mut out: BTreeMap<String, Tally> = BTreeMap::new();
    for raw in doc.lc_classifications.iter().chain(doc.lcc.iter()) {
        let Some(class) = mapper.map(raw) else {
            continue;
        };
        out.entry(class.code.clone())
            .and_modify(|t| t.count += 1)
            .or_insert_with(|| Tally {
                class,
                count: 1,
                call_number: raw.trim().to_string(),
            });
    }
    out
}

impl Tier for OpenLibraryTier {
    fn info(&self) -> TierInfo {
        TierInfo {
            name: NAME.into(),
            version: VERSION.into(),
            needs_model: false,
            expected_memory_mb: 4,
            // One paced request; a cache hit is a few hundred microseconds.
            expected_micros_per_item: 1_000_000,
        }
    }

    fn run(
        &self,
        input: &ClassifyInput,
        index: &dyn CompiledIndex,
        _prior: &[Candidate],
    ) -> Result<TierOutput, Error> {
        let start = Instant::now();
        let candidates = match Self::query(input) {
            Some(q) => {
                let docs = self.client.search(&q)?;
                self.candidates(&q, &docs, index)
            }
            None => Vec::new(),
        };
        Ok(TierOutput {
            candidates,
            cost: Cost {
                micros: u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX),
                model_calls: 0,
            },
        })
    }
}
