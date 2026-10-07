//! The lexical tiers for a store directory (U3, U4), and identifier
//! parsing for request arguments.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{Candidate, ClassifyInput, Identifier, Tier, TierInfo, TierOutput};
use shelfmark_core::{Error, ErrorCode};
use shelfmark_tier_identifier::{normalize, IdentifierTier, Tables};
use shelfmark_tier_lexical::{RulesConfig, RulesTier, VocabTier};

/// Identifier tables directory inside a store (PD5).
pub const TABLES_DIR: &str = "tables";
/// Rules file inside a store.
pub const RULES_FILE: &str = "rules.json";

/// `identifier`, `vocab` and `rules`. A table or rules file that does not
/// load leaves a tier that reports why on every run, so the error reaches
/// the caller with its code instead of being dropped at startup.
pub fn registry(dir: &Path) -> BTreeMap<String, Arc<dyn Tier>> {
    let identifier: Arc<dyn Tier> = match load_tables(&dir.join(TABLES_DIR)) {
        Ok(tables) => Arc::new(IdentifierTier::new(tables)),
        Err(e) => Arc::new(Broken::new(shelfmark_tier_identifier::NAME, e)),
    };
    let rules_path = dir.join(RULES_FILE);
    let rules: Arc<dyn Tier> = if rules_path.exists() {
        match RulesTier::from_path(&rules_path) {
            Ok(t) => Arc::new(t),
            Err(e) => Arc::new(Broken::new(RulesTier::NAME, e)),
        }
    } else {
        Arc::new(RulesTier::from_config(&RulesConfig::default()).expect("empty rules compile"))
    };
    let tiers: [(&str, Arc<dyn Tier>); 3] = [
        (shelfmark_tier_identifier::NAME, identifier),
        (VocabTier::NAME, Arc::new(VocabTier::new())),
        (RulesTier::NAME, rules),
    ];
    tiers.into_iter().map(|(n, t)| (n.to_string(), t)).collect()
}

fn load_tables(dir: &Path) -> Result<Tables, Error> {
    let mut tables = Tables::new();
    if dir.is_dir() {
        tables.load_dir(dir).map_err(|e| {
            Error::new(
                ErrorCode::StoreError,
                format!("identifier tables in {}: {e}", dir.display()),
            )
            .next(format!("fix or remove the table under {}", dir.display()))
        })?;
    }
    Ok(tables)
}

/// A tier whose data failed to load.
struct Broken {
    name: &'static str,
    error: Error,
}

impl Broken {
    fn new(name: &'static str, error: Error) -> Self {
        Broken { name, error }
    }
}

impl Tier for Broken {
    fn info(&self) -> TierInfo {
        TierInfo {
            name: self.name.into(),
            version: "unloaded".into(),
            needs_model: false,
            expected_memory_mb: 0,
            expected_micros_per_item: 0,
        }
    }

    fn run(
        &self,
        _: &ClassifyInput,
        _: &dyn CompiledIndex,
        _: &[Candidate],
    ) -> Result<TierOutput, Error> {
        Err(self.error.clone())
    }
}

/// A typed identifier argument (`isbn:0-262-03384-4`), normalized and
/// checksummed the way the identifier tier keys its tables.
pub fn parse_identifier(s: &str) -> Option<Identifier> {
    normalize::parse_key(s.trim()).ok()
}

/// The domain identifier of a URL.
pub fn url_domain(url: &str) -> Option<Identifier> {
    normalize::domain(url).map(Identifier::UrlDomain)
}
