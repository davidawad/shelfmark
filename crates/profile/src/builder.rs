//! From a profile and the caller's tiers to a runnable cascade (R8).
//!
//! The caller registers tier implementations by name; [`build`] checks
//! the profile against them and returns a [`Cascade`] holding the tiers in
//! profile order. A semantic tier this build lacks is filled with a
//! [`SemanticStub`], so the cascade records it `unavailable` instead of
//! failing (AE3). Any other tier name with no implementation is an error.

use std::collections::BTreeMap;

use shelfmark_core::calibration::Calibration;
use shelfmark_core::cascade::{self, Decision};
use shelfmark_core::error::{Error, ErrorCode};
use shelfmark_core::id::sha256_hex;
use shelfmark_core::placement::canonical_json;
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{ClassifyInput, Profile, Tier, TierInfo};

use crate::semantic::{self, SemanticStub, STUB_VERSION};

/// Tier implementations by name.
pub type Registry = BTreeMap<String, Box<dyn Tier>>;

/// The names in `registry` whose tier can actually run: everything but
/// the semantic stubs.
pub fn runnable(registry: &Registry) -> std::collections::BTreeSet<String> {
    registry
        .iter()
        .filter(|(_, t)| t.info().version != STUB_VERSION)
        .map(|(name, _)| name.clone())
        .collect()
}

enum Slot<'r> {
    Registered(&'r dyn Tier),
    Stub(SemanticStub),
}

impl Slot<'_> {
    fn tier(&self) -> &dyn Tier {
        match self {
            Slot::Registered(t) => *t,
            Slot::Stub(s) => s,
        }
    }
}

/// A profile with its tiers resolved, in run order.
pub struct Cascade<'r> {
    profile: Profile,
    tiers: Vec<(String, Slot<'r>)>,
}

fn invalid(profile: &Profile, message: String) -> Error {
    Error::new(ErrorCode::InvalidRequest, message)
        .next("shelfmark describe profiles")
        .details(serde_json::json!({ "profile": profile.id.to_string() }))
}

/// Resolve every tier `profile` names against `registry`.
///
/// Errors: a tier name with no implementation that is not a built-in
/// semantic tier (`tier_unavailable`), a tier listed twice, or a
/// registered tier whose `info().name` differs from its key
/// (`invalid_request`).
pub fn build<'r>(profile: Profile, registry: &'r Registry) -> Result<Cascade<'r>, Error> {
    let mut tiers: Vec<(String, Slot<'r>)> = Vec::with_capacity(profile.tiers.len());
    for name in &profile.tiers {
        if tiers.iter().any(|(n, _)| n == name) {
            return Err(invalid(
                &profile,
                format!("profile {} lists tier {name} twice", profile.id),
            ));
        }
        let slot = if let Some(tier) = registry.get(name) {
            let declared = tier.info().name;
            if &declared != name {
                return Err(invalid(
                    &profile,
                    format!("tier registered as {name} calls itself {declared}"),
                ));
            }
            Slot::Registered(tier.as_ref())
        } else if let Some(spec) = semantic::spec(name) {
            Slot::Stub(SemanticStub::not_built(spec))
        } else {
            let registered: Vec<&String> = registry.keys().collect();
            return Err(Error::new(
                ErrorCode::TierUnavailable,
                format!(
                    "profile {} names tier {name}, which nothing provides",
                    profile.id
                ),
            )
            .next("shelfmark describe profiles")
            .details(serde_json::json!({
                "profile": profile.id.to_string(),
                "tier": name,
                "registered": registered,
            })));
        };
        tiers.push((name.clone(), slot));
    }
    Ok(Cascade { profile, tiers })
}

impl<'r> Cascade<'r> {
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// Tier names and implementations in run order.
    pub fn tiers(&self) -> impl Iterator<Item = (&str, &dyn Tier)> {
        self.tiers.iter().map(|(n, s)| (n.as_str(), s.tier()))
    }

    /// The map `shelfmark_core::cascade::run` takes.
    pub fn tier_map(&self) -> BTreeMap<String, &dyn Tier> {
        self.tiers().map(|(n, t)| (n.to_string(), t)).collect()
    }

    /// Each tier's self-description, in run order.
    pub fn infos(&self) -> Vec<TierInfo> {
        self.tiers().map(|(_, t)| t.info()).collect()
    }

    /// `InputHashes.profile` for placements made by this cascade.
    pub fn hash(&self) -> String {
        profile_hash(&self.profile, &self.infos())
    }

    /// Run the cascade over one input.
    pub fn run(
        &self,
        index: &dyn CompiledIndex,
        input: &ClassifyInput,
        target: Option<f64>,
        calibration: Option<&Calibration>,
    ) -> Result<Decision, Error> {
        cascade::run(
            &self.profile,
            &self.tier_map(),
            index,
            input,
            target,
            calibration,
        )
    }
}

/// Hash of everything in a profile that can change a decision: its ID,
/// tier order, default target and minimum margin, and each tier's name,
/// version and whether it needs a model. The description and declared
/// costs are left out, so refining estimates never stales placements.
pub fn profile_hash(profile: &Profile, infos: &[TierInfo]) -> String {
    let tiers: Vec<serde_json::Value> = infos
        .iter()
        .map(|i| {
            serde_json::json!({
                "name": i.name,
                "version": i.version,
                "needs_model": i.needs_model,
            })
        })
        .collect();
    let doc = serde_json::json!({
        "id": profile.id.to_string(),
        "tiers": profile.tiers,
        "default_target": profile.default_target,
        "min_margin": profile.min_margin,
        "tier_versions": tiers,
    });
    format!("s256:{}", sha256_hex(canonical_json(&doc).as_bytes()))
}
