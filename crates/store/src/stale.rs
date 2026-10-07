//! Stale listing (R14, AE6): live claims whose recorded inputs no longer
//! match the current scheme or profile.

use serde::Serialize;
use shelfmark_core::placement::{effective_status, superseded_ids, AuthorKind, Placement, Status};
use shelfmark_core::{ProfileId, SchemeId};

/// What the caller has now. Claims are matched by scheme and profile
/// *name*, so claims made under `lcc@2024` are checked against `lcc@2025`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleQuery {
    pub scheme: SchemeId,
    /// `CompiledIndex::content_hash` of the current scheme.
    pub scheme_hash: String,
    /// The current profile and its hash; `None` checks the scheme only.
    pub profile: Option<(ProfileId, String)>,
    /// Only claims placed in this class code (e.g. the class a new scheme
    /// version split).
    pub class: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StaleReason {
    /// The claim was made against different scheme content.
    Scheme,
    /// A machine claim made by a different version of the profile.
    Profile,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StaleClaim {
    pub placement: Placement,
    pub reasons: Vec<StaleReason>,
}

/// Live claims in `claims` (append order) that `query` makes stale, in
/// append order. A profile change never stales a human claim: no profile
/// made it.
pub fn stale(claims: &[Placement], query: &StaleQuery) -> Vec<StaleClaim> {
    let superseded = superseded_ids(claims);
    claims
        .iter()
        .filter(|p| effective_status(p, &superseded) != Status::Superseded)
        .filter(|p| p.scheme.0.name == query.scheme.0.name)
        .filter(|p| query.class.as_ref().is_none_or(|c| &p.class.code == c))
        .filter_map(|p| {
            let mut reasons = Vec::new();
            if p.inputs.scheme != query.scheme_hash {
                reasons.push(StaleReason::Scheme);
            }
            if let Some((profile, hash)) = &query.profile {
                if p.provenance.author_kind != AuthorKind::Human
                    && p.profile.0.name == profile.0.name
                    && &p.inputs.profile != hash
                {
                    reasons.push(StaleReason::Profile);
                }
            }
            (!reasons.is_empty()).then(|| StaleClaim {
                placement: p.clone(),
                reasons,
            })
        })
        .collect()
}
