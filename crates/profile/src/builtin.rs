//! The built-in profiles (R8, R9) and the declared cost of each built-in
//! tier.
//!
//! A profile with a network tier (`openlibrary`) is never picked on its
//! own: not as the recommended profile and not by a default eval. It runs
//! when a caller names it.
//!
//! Profile costs are the sum of their tiers' declared costs, so the two
//! never drift apart. The `shortlist` figures are the CPU benchmark
//! spike's (spike 4) for BGE-small-en-v1.5
//! int8 on 4 threads; `rerank` is still an estimate for
//! Qwen3-Reranker-0.6B at Q8, which no build ships. Costs are not part of
//! the profile hash, so updating them does not make placements stale.

use shelfmark_core::error::{Error, ErrorCode};
use shelfmark_core::tier::Profile;
use shelfmark_core::ProfileId;

/// Version of every built-in profile definition. Every built-in runs the
/// `vocab` tier, so all move together: 2 stopped matching labels and
/// terms made only of stopwords (`vocab-2`).
pub const PROFILE_VERSION: &str = "2";
/// Default target confidence of the built-in profiles (R10).
pub const DEFAULT_TARGET: f64 = 0.8;
/// Margin below which a decision is flagged even at the target.
pub const MIN_MARGIN: f64 = 0.1;

/// Declared cost of one built-in tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierSpec {
    pub name: &'static str,
    /// Whether the tier needs a model download.
    pub needs_model: bool,
    /// Whether the tier sends requests to a remote service. A profile
    /// with such a tier runs only when asked for by name.
    pub needs_network: bool,
    pub expected_memory_mb: u32,
    pub expected_micros_per_item: u64,
}

/// Every built-in tier, in the order a full cascade runs them.
pub const TIERS: [TierSpec; 6] = [
    TierSpec {
        name: "identifier",
        needs_model: false,
        needs_network: false,
        expected_memory_mb: 8,
        expected_micros_per_item: 20,
    },
    // One Open Library search, paced at one request a second; a cache
    // hit costs well under a millisecond.
    TierSpec {
        name: "openlibrary",
        needs_model: false,
        needs_network: true,
        expected_memory_mb: 4,
        expected_micros_per_item: 1_000_000,
    },
    TierSpec {
        name: "vocab",
        needs_model: false,
        needs_network: false,
        expected_memory_mb: 48,
        expected_micros_per_item: 300,
    },
    TierSpec {
        name: "rules",
        needs_model: false,
        needs_network: false,
        expected_memory_mb: 8,
        expected_micros_per_item: 50,
    },
    // Measured: 572 MB peak RSS with the ONNX runtime, 46 ms p50 per item
    // (85 ms p95), one embedding of up to 512 tokens.
    TierSpec {
        name: "shortlist",
        needs_model: true,
        needs_network: false,
        expected_memory_mb: 600,
        expected_micros_per_item: 50_000,
    },
    // Q8 weights plus runtime; scores about 20 prior candidates per item.
    TierSpec {
        name: "rerank",
        needs_model: true,
        needs_network: false,
        expected_memory_mb: 900,
        expected_micros_per_item: 400_000,
    },
];

/// The declared cost of a built-in tier, if `name` is one.
pub fn tier_spec(name: &str) -> Option<&'static TierSpec> {
    TIERS.iter().find(|t| t.name == name)
}

const LEXICAL_TIERS: [&str; 3] = ["identifier", "vocab", "rules"];

fn profile(name: &str, description: &str, tiers: &[&str]) -> Profile {
    let specs = || tiers.iter().filter_map(|t| tier_spec(t));
    Profile {
        id: ProfileId::new(name, PROFILE_VERSION),
        description: description.into(),
        tiers: tiers.iter().map(|t| (*t).into()).collect(),
        default_target: DEFAULT_TARGET,
        min_margin: MIN_MARGIN,
        expected_memory_mb: specs().map(|s| s.expected_memory_mb).sum(),
        expected_micros_per_item: specs().map(|s| s.expected_micros_per_item).sum(),
    }
}

/// Identifier lookup, vocabulary match and config rules. No model.
pub fn lexical() -> Profile {
    profile(
        "lexical",
        "Identifier lookup, vocabulary match and config rules; no model",
        &LEXICAL_TIERS,
    )
}

/// The identifier tier, then an Open Library title and author lookup,
/// then the vocabulary and rules tiers. Sends requests to
/// openlibrary.org, so it never runs unless asked for by name.
pub fn openlibrary() -> Profile {
    profile(
        "openlibrary",
        "Identifier lookup, then Open Library title+author lookup (network), then vocabulary match and rules",
        &["identifier", "openlibrary", "vocab", "rules"],
    )
}

/// Whether any tier of `profile` sends requests to a remote service.
pub fn needs_network(profile: &Profile) -> bool {
    profile
        .tiers
        .iter()
        .any(|t| tier_spec(t).is_some_and(|s| s.needs_network))
}

/// The lexical tiers, then an embedding shortlist.
pub fn shortlist() -> Profile {
    let mut tiers = LEXICAL_TIERS.to_vec();
    tiers.push("shortlist");
    profile(
        "shortlist",
        "Lexical tiers, then an embedding shortlist (BGE-small-en-v1.5 int8)",
        &tiers,
    )
}

/// The shortlist profile, then a cross-encoder rerank.
pub fn rerank() -> Profile {
    let mut tiers = LEXICAL_TIERS.to_vec();
    tiers.extend(["shortlist", "rerank"]);
    profile(
        "rerank",
        "Shortlist profile, then a cross-encoder rerank (Qwen3-Reranker-0.6B Q8)",
        &tiers,
    )
}

/// Every built-in profile: `lexical`, the network `openlibrary`, then
/// the model profiles, cheapest first.
pub fn builtins() -> Vec<Profile> {
    vec![lexical(), openlibrary(), shortlist(), rerank()]
}

/// Look up a built-in profile by bare name (`lexical`) or full ID
/// (`profile:lexical@2`).
pub fn find(name_or_id: &str) -> Result<Profile, Error> {
    builtins()
        .into_iter()
        .find(|p| p.id.0.name == name_or_id || p.id.to_string() == name_or_id)
        .ok_or_else(|| {
            let known: Vec<String> = builtins().iter().map(|p| p.id.to_string()).collect();
            Error::new(
                ErrorCode::ProfileNotFound,
                format!(
                    "no profile named {name_or_id}; known profiles: {}",
                    known.join(", ")
                ),
            )
            .next("shelfmark describe profiles")
            .details(serde_json::json!({ "profile": name_or_id, "known": known }))
        })
}
