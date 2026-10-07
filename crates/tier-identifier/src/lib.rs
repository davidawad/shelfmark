//! The `identifier` tier (plan U3; R7, AE1, PD5).
//!
//! Extracts exact identifiers (ISBN-10/13, LCCN, DOI, arXiv ID, URL
//! domain) from a [`ClassifyInput`], looks them up in JSONL tables loaded
//! at run time, and returns each table class the scheme resolves at
//! confidence 1.0 with evidence naming the identifier and the table row.
//! A class the scheme does not cover is dropped. When two identifiers name
//! the same class this tier returns both; the cascade merges them.

pub mod extract;
pub mod normalize;
pub mod table;

use std::time::Instant;

use shelfmark_core::error::Error;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};
use shelfmark_core::tier::{
    Candidate, ClassifyInput, Cost, Evidence, Identifier, Tier, TierInfo, TierOutput,
};

pub use extract::extract;
pub use table::{Row, TableError, Tables};

/// The tier's name in profiles.
pub const NAME: &str = "identifier";
/// Version of the extraction and lookup logic (part of the profile hash).
/// The tables themselves go into the config hash via
/// [`Tables::content_hash`].
pub const VERSION: &str = "1";

pub struct IdentifierTier {
    tables: Tables,
}

impl IdentifierTier {
    pub fn new(tables: Tables) -> Self {
        IdentifierTier { tables }
    }

    pub fn tables(&self) -> &Tables {
        &self.tables
    }

    /// Candidates for already-extracted identifiers, in a stable order.
    pub fn lookup<'a>(
        &self,
        ids: impl IntoIterator<Item = &'a Identifier>,
        index: &dyn CompiledIndex,
    ) -> Vec<Candidate> {
        let mut out = Vec::new();
        for id in ids {
            let Some((matched, rows)) = self.tables.lookup(id) else {
                continue;
            };
            for row in rows {
                let class = match index.check_code(&row.class) {
                    CodeCheck::Exact { class } | CodeCheck::Within { class } => class,
                    CodeCheck::Unplaced | CodeCheck::Invalid { .. } => continue,
                };
                out.push(Candidate {
                    class,
                    score: 1.0,
                    evidence: vec![Evidence::Identifier {
                        identifier: matched.clone(),
                        table: row.reference(),
                    }],
                });
            }
        }
        out
    }
}

impl Tier for IdentifierTier {
    fn info(&self) -> TierInfo {
        TierInfo {
            name: NAME.to_string(),
            version: VERSION.to_string(),
            needs_model: false,
            // Roughly 200 bytes per row in memory.
            expected_memory_mb: 1 + (self.tables.len() / 5_000) as u32,
            expected_micros_per_item: 200,
        }
    }

    fn run(
        &self,
        input: &ClassifyInput,
        index: &dyn CompiledIndex,
        _prior: &[Candidate],
    ) -> Result<TierOutput, Error> {
        let start = Instant::now();
        let ids = extract(input);
        let candidates = self.lookup(&ids, index);
        Ok(TierOutput {
            candidates,
            cost: Cost {
                micros: u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX),
                model_calls: 0,
            },
        })
    }
}
