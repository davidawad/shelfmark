//! The tiers for a store directory: the lexical ones (U3, U4), plus the
//! BGE-small `shortlist` tier when built with feature `semantic-shortlist`
//! and the `openlibrary` tier when built with feature `openlibrary`, and
//! identifier parsing for request arguments.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{Candidate, ClassifyInput, Identifier, Tier, TierInfo, TierOutput};
use shelfmark_core::{Error, ErrorCode};
use shelfmark_tier_identifier::{normalize, IdentifierTier, Tables};
use shelfmark_tier_lexical::{RulesConfig, RulesTier, VocabTier};

/// Identifier tables directory inside a store (PD5).
pub const TABLES_DIR: &str = "tables";
/// Rules file inside a store; it replaces the bundled starter rules.
pub const RULES_FILE: &str = "rules.json";

/// The bundled starter rules (`data/rules/starter.json`): arXiv category
/// stamps and URL domains to LCC, generated from the identifier spike's
/// hand-assigned, not yet librarian-reviewed maps. A `decide` rule weighs
/// 0.9 and a `hint` 0.4.
const STARTER_RULES: &str = include_str!("../../../../data/rules/starter.json");

/// The bundled starter rules as a rules file.
pub fn starter_rules() -> RulesConfig {
    serde_json::from_str(STARTER_RULES).expect("bundled rules parse")
}

/// `identifier`, `vocab` and `rules` (the store's `rules.json`, else the
/// bundled starter rules), and `shortlist` and `openlibrary` in a build
/// with them. With a `cache` directory the vocab tier saves built
/// matchers under `vocab/`, the shortlist tier its label vectors under
/// `shortlist/` and the openlibrary tier search responses under
/// `openlibrary/`. A table or
/// rules file that does not load leaves a tier that reports why on every
/// run, so the error reaches the caller with its code instead of being
/// dropped at startup.
pub fn registry(dir: &Path, cache: Option<PathBuf>) -> BTreeMap<String, Arc<dyn Tier>> {
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
        Arc::new(RulesTier::from_config(&starter_rules()).expect("bundled rules compile"))
    };
    let vocab = match &cache {
        Some(c) => VocabTier::with_cache_dir(c.join("vocab")),
        None => VocabTier::new(),
    };
    let tiers: [(&str, Arc<dyn Tier>); 3] = [
        (shelfmark_tier_identifier::NAME, identifier),
        (VocabTier::NAME, Arc::new(vocab)),
        (RulesTier::NAME, rules),
    ];
    tiers
        .into_iter()
        .chain(openlibrary(cache.as_deref()))
        .chain(shortlist(cache))
        .map(|(n, t)| (n.to_string(), t))
        .collect()
}

/// The Open Library tier. It sends nothing until a run of the
/// `openlibrary` profile asks it to.
#[cfg(feature = "openlibrary")]
fn openlibrary(cache: Option<&Path>) -> Option<(&'static str, Arc<dyn Tier>)> {
    use shelfmark_tier_openlibrary::OpenLibraryTier;
    Some((
        OpenLibraryTier::NAME,
        Arc::new(OpenLibraryTier::online(
            cache.map(|c| c.join(OpenLibraryTier::NAME)),
        )),
    ))
}

/// This build has no Open Library tier; the `openlibrary` profile is
/// listed but not runnable.
#[cfg(not(feature = "openlibrary"))]
fn openlibrary(_cache: Option<&Path>) -> Option<(&'static str, Arc<dyn Tier>)> {
    None
}

/// The BGE-small shortlist tier. The model loads on the first run that
/// needs it, not here.
#[cfg(feature = "semantic-shortlist")]
fn shortlist(cache: Option<PathBuf>) -> Option<(&'static str, Arc<dyn Tier>)> {
    use shelfmark_tier_shortlist::ShortlistTier;
    Some((
        ShortlistTier::NAME,
        Arc::new(ShortlistTier::bge_small(cache)),
    ))
}

/// This build has no shortlist tier; the profile builder fills the slot
/// with a stub naming the feature.
#[cfg(not(feature = "semantic-shortlist"))]
fn shortlist(_cache: Option<PathBuf>) -> Option<(&'static str, Arc<dyn Tier>)> {
    None
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
