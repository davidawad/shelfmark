//! The built-in profiles (R8, R9) and the declared cost of each built-in
//! tier.
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
    pub expected_memory_mb: u32,
    pub expected_micros_per_item: u64,
}

/// Every built-in tier, in the order a full cascade runs them.
pub const TIERS: [TierSpec; 5] = [
    TierSpec {
        name: "identifier",
        needs_model: false,
        expected_memory_mb: 8,
        expected_micros_per_item: 20,
    },
    TierSpec {
        name: "vocab",
        needs_model: false,
        expected_memory_mb: 48,
        expected_micros_per_item: 300,
    },
    TierSpec {
        name: "rules",
        needs_model: false,
        expected_memory_mb: 8,
        expected_micros_per_item: 50,
    },
    // Measured: 572 MB peak RSS with the ONNX runtime, 46 ms p50 per item
    // (85 ms p95), one embedding of up to 512 tokens.
    TierSpec {
        name: "shortlist",
        needs_model: true,
        expected_memory_mb: 600,
        expected_micros_per_item: 50_000,
    },
    // Q8 weights plus runtime; scores about 20 prior candidates per item.
    TierSpec {
        name: "rerank",
        needs_model: true,
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

/// Every built-in profile, cheapest first.
pub fn builtins() -> Vec<Profile> {
    vec![lexical(), shortlist(), rerank()]
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
