//! Semantic tier stubs (plan PD4).
//!
//! v1 declares the `shortlist` (embedding) and `rerank` (cross-encoder)
//! tiers so profiles and `describe` are stable, but ships no model
//! runtime. With the cargo feature on, [`shortlist`] and [`rerank`] give
//! stub tiers; without it, the builder fills the slot with a tier saying
//! which feature to build. Either way the tier reports `tier_unavailable`
//! naming the feature and the model to install, and the cascade records
//! it as `unavailable` and carries on.

use shelfmark_core::error::{Error, ErrorCode};
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{Candidate, ClassifyInput, Tier, TierInfo, TierOutput};

use crate::builtin::tier_spec;

pub const SHORTLIST_MODEL: &str = "Qwen3-Embedding-0.6B (Q8)";
pub const RERANK_MODEL: &str = "Qwen3-Reranker-0.6B (Q8)";
/// `TierInfo::version` of every stub.
pub const STUB_VERSION: &str = "stub-1";

/// What a semantic tier needs before it can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticSpec {
    pub tier: &'static str,
    pub feature: &'static str,
    pub model: &'static str,
}

pub const SEMANTIC: [SemanticSpec; 2] = [
    SemanticSpec {
        tier: "shortlist",
        feature: "semantic-shortlist",
        model: SHORTLIST_MODEL,
    },
    SemanticSpec {
        tier: "rerank",
        feature: "semantic-rerank",
        model: RERANK_MODEL,
    },
];

/// The semantic spec for `tier`, if it is a semantic tier.
pub fn spec(tier: &str) -> Option<&'static SemanticSpec> {
    SEMANTIC.iter().find(|s| s.tier == tier)
}

/// A semantic tier that cannot run in this build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticStub {
    spec: &'static SemanticSpec,
    /// Whether the cargo feature was enabled for this build.
    built: bool,
}

impl SemanticStub {
    /// The slot the builder fills for a semantic tier this build lacks.
    pub(crate) fn not_built(spec: &'static SemanticSpec) -> Self {
        SemanticStub { spec, built: false }
    }

    #[cfg(any(feature = "semantic-shortlist", feature = "semantic-rerank"))]
    fn built(tier: &str) -> Self {
        let spec = spec(tier).expect("semantic tier is declared");
        SemanticStub { spec, built: true }
    }

    pub fn spec(&self) -> &'static SemanticSpec {
        self.spec
    }

    /// The `tier_unavailable` error this tier returns.
    pub fn error(&self) -> Error {
        let s = self.spec;
        let (message, next) = if self.built {
            (
                format!(
                    "the {} tier is a v1 stub: feature {} is on, but no CPU runtime for {} ships yet",
                    s.tier, s.feature, s.model
                ),
                "shelfmark classify --profile lexical <file>".to_string(),
            )
        } else {
            (
                format!(
                    "the {} tier is not in this build: rebuild with feature {} and install model {}",
                    s.tier, s.feature, s.model
                ),
                format!("cargo install shelfmark --features {}", s.feature),
            )
        };
        Error::new(ErrorCode::TierUnavailable, message)
            .next(next)
            .details(serde_json::json!({
                "tier": s.tier,
                "feature": s.feature,
                "feature_enabled": self.built,
                "model": s.model,
            }))
    }
}

/// The embedding shortlist stub.
#[cfg(feature = "semantic-shortlist")]
pub fn shortlist() -> SemanticStub {
    SemanticStub::built("shortlist")
}

/// The cross-encoder rerank stub.
#[cfg(feature = "semantic-rerank")]
pub fn rerank() -> SemanticStub {
    SemanticStub::built("rerank")
}

impl Tier for SemanticStub {
    fn info(&self) -> TierInfo {
        let cost = tier_spec(self.spec.tier).expect("semantic tier has a declared cost");
        TierInfo {
            name: self.spec.tier.into(),
            // The same either way: a stub and a missing tier both decide
            // nothing, so toggling the feature must not stale placements.
            version: STUB_VERSION.into(),
            needs_model: true,
            expected_memory_mb: cost.expected_memory_mb,
            expected_micros_per_item: cost.expected_micros_per_item,
        }
    }

    fn run(
        &self,
        _: &ClassifyInput,
        _: &dyn CompiledIndex,
        _: &[Candidate],
    ) -> Result<TierOutput, Error> {
        Err(self.error())
    }
}
