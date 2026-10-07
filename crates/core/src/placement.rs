//! L3 placement claims (R11-R14).
//!
//! A placement is an append-only claim that a Work belongs in a class. It
//! records its inputs' hashes (memoization and staleness), a full trace,
//! and a provenance block in the same shape the `kd` resource uses.

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::id::{sha256_hex, ClassId, PlacementId, ProfileId, SchemeId, WorkId};
use crate::tier::{Cost, Evidence};

/// Claim status. Aligned with `kd`'s lattice (machine/flagged ~ staged,
/// confirmed ~ promoted, superseded ~ superseded).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Machine,
    Flagged,
    Confirmed,
    Superseded,
}

impl Status {
    /// Rank for deriving the current placement: a person's confirmation
    /// outranks any machine claim; superseded claims never win (R12).
    pub fn rank(self) -> u8 {
        match self {
            Status::Superseded => 0,
            Status::Flagged => 1,
            Status::Machine => 2,
            Status::Confirmed => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorKind {
    Human,
    Agent,
    Tool,
}

/// Who made a claim. Same field names and optionality as `kd`'s provenance
/// block (author_kind, author, model, harness, run_id, prompt_hash, at),
/// without depending on it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub author_kind: AuthorKind,
    pub author: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<String>,
    /// RFC 3339 timestamp.
    pub at: String,
}

/// Hashes of everything that determined a placement (R14). Equal inputs
/// mean the cached placement is reused; a changed scheme or profile hash
/// names the placements that are stale.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InputHashes {
    pub work: WorkId,
    /// Content hash of the scheme file (`CompiledIndex::content_hash`).
    pub scheme: String,
    /// Hash of the profile definition plus its tiers' versions.
    pub profile: String,
    /// Hash of the caller's config (rules, tables, target confidence).
    pub config: String,
}

impl InputHashes {
    /// The memo key: one hash over all four inputs.
    pub fn memo_key(&self) -> String {
        sha256_hex(
            format!(
                "{}\n{}\n{}\n{}",
                self.work, self.scheme, self.profile, self.config
            )
            .as_bytes(),
        )
    }
}

/// Per-tier score for one candidate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TierScore {
    pub tier: String,
    pub score: f64,
}

/// One candidate as the trace reports it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceCandidate {
    pub class: ClassId,
    /// Best score across tiers (what ranking used).
    pub score: f64,
    pub per_tier: Vec<TierScore>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TierRunState {
    Ran,
    /// Not reached: an earlier tier decided.
    Skipped,
    /// Not available on this build or machine.
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TierRun {
    pub tier: String,
    pub state: TierRunState,
    #[serde(default)]
    pub cost: Cost,
}

/// Everything an agent needs to explain or replay a placement (R13).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub target: f64,
    /// Ranked best first.
    pub candidates: Vec<TraceCandidate>,
    /// Top score minus runner-up score (1.0 with a single candidate).
    pub margin: f64,
    /// The tier that decided, or `None` when no tier reached the target
    /// and the top-ranked class was taken flagged.
    pub deciding_tier: Option<String>,
    pub tiers: Vec<TierRun>,
    pub cost: Cost,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag_reason: Option<String>,
}

/// A placement claim (R11).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    pub id: PlacementId,
    pub work: WorkId,
    pub scheme: SchemeId,
    pub class: ClassId,
    pub profile: ProfileId,
    pub confidence: f64,
    pub status: Status,
    /// The claim this one replaces (never edited, only superseded).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<PlacementId>,
    pub inputs: InputHashes,
    pub trace: Trace,
    pub provenance: Provenance,
}

impl Placement {
    /// Content ID: SHA-256 over the canonical JSON of the claim with `id`
    /// removed and every cost zeroed, so timing never changes an ID.
    pub fn compute_id(&self) -> PlacementId {
        let mut v = serde_json::to_value(self).expect("placement serializes");
        let obj = v.as_object_mut().expect("placement is an object");
        obj.remove("id");
        zero_costs(&mut v);
        PlacementId::from_canonical(canonical_json(&v).as_bytes())
    }

    /// Set `id` to [`Placement::compute_id`].
    pub fn seal(mut self) -> Self {
        self.id = self.compute_id();
        self
    }
}

fn zero_costs(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(m) => {
            for (k, x) in m.iter_mut() {
                if k == "cost" {
                    *x = serde_json::to_value(Cost::default()).expect("cost serializes");
                } else {
                    zero_costs(x);
                }
            }
        }
        serde_json::Value::Array(a) => a.iter_mut().for_each(zero_costs),
        _ => {}
    }
}

/// JSON with object keys sorted and no whitespace.
pub fn canonical_json(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Object(m) => {
            let mut keys: Vec<_> = m.keys().collect();
            keys.sort();
            let body: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    format!(
                        "{}:{}",
                        serde_json::Value::String(k.clone()),
                        canonical_json(&m[k])
                    )
                })
                .collect();
            format!("{{{}}}", body.join(","))
        }
        serde_json::Value::Array(a) => {
            format!(
                "[{}]",
                a.iter().map(canonical_json).collect::<Vec<_>>().join(",")
            )
        }
        other => other.to_string(),
    }
}

/// Pick the current placement of one Work in one scheme from its claims
/// (R12): highest status rank, then latest in log order.
pub fn current<'a>(claims: impl IntoIterator<Item = &'a Placement>) -> Option<&'a Placement> {
    let mut best: Option<&Placement> = None;
    for p in claims {
        if p.status == Status::Superseded {
            continue;
        }
        if best.is_none_or(|b| p.status.rank() >= b.status.rank()) {
            best = Some(p);
        }
    }
    best
}

/// The append-only claim store (implemented by the store crate, plan PD6).
pub trait PlacementLog {
    /// Append a sealed claim. Appending an ID already present is a no-op
    /// that returns `false`.
    fn append(&mut self, placement: &Placement) -> Result<bool, Error>;
    /// Every claim in append order.
    fn claims(&self) -> Result<Vec<Placement>, Error>;
    /// Claims for one Work in one scheme, in append order.
    fn claims_for(&self, work: &WorkId, scheme: &SchemeId) -> Result<Vec<Placement>, Error> {
        Ok(self
            .claims()?
            .into_iter()
            .filter(|p| &p.work == work && &p.scheme == scheme)
            .collect())
    }
    /// A claim whose inputs match, for memoization (R14).
    fn memo(&self, inputs: &InputHashes) -> Result<Option<Placement>, Error> {
        let key = inputs.memo_key();
        Ok(self
            .claims()?
            .into_iter()
            .rev()
            .find(|p| p.provenance.author_kind != AuthorKind::Human && p.inputs.memo_key() == key))
    }
}
