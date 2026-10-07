//! Claim builders shared by the store tests.
#![allow(dead_code)]

use shelfmark_core::placement::{AuthorKind, InputHashes, Placement, Provenance, Status, Trace};
use shelfmark_core::tier::Cost;
use shelfmark_core::{ClassId, PlacementId, ProfileId, SchemeId, WorkId};
use shelfmark_store::Store;

pub fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("store")).unwrap();
    (dir, store)
}

pub fn lcc(version: &str) -> SchemeId {
    SchemeId::new("lcc", version)
}

/// What a claim was made from.
#[derive(Clone)]
pub struct Spec {
    pub work: &'static str,
    pub scheme: SchemeId,
    pub scheme_hash: &'static str,
    pub profile_hash: &'static str,
    pub code: &'static str,
    pub status: Status,
    pub author: AuthorKind,
    pub supersedes: Option<PlacementId>,
}

impl Spec {
    pub fn machine(work: &'static str, code: &'static str) -> Spec {
        Spec {
            work,
            scheme: lcc("2024"),
            scheme_hash: "s256:scheme-2024",
            profile_hash: "s256:lexical-1",
            code,
            status: Status::Machine,
            author: AuthorKind::Tool,
            supersedes: None,
        }
    }

    pub fn human(work: &'static str, code: &'static str, old: &Placement) -> Spec {
        Spec {
            status: Status::Confirmed,
            author: AuthorKind::Human,
            supersedes: Some(old.id.clone()),
            ..Spec::machine(work, code)
        }
    }

    pub fn build(&self) -> Placement {
        let work = WorkId::from_text(self.work);
        Placement {
            id: PlacementId::from_canonical(b"unsealed"),
            work: work.clone(),
            scheme: self.scheme.clone(),
            class: ClassId::new(self.scheme.clone(), self.code),
            profile: ProfileId::new("lexical", "1"),
            confidence: 0.9,
            status: self.status,
            supersedes: self.supersedes.clone(),
            inputs: InputHashes {
                work,
                scheme: self.scheme_hash.into(),
                profile: self.profile_hash.into(),
                config: "s256:config".into(),
            },
            trace: Trace {
                target: 0.8,
                candidates: vec![],
                margin: 1.0,
                deciding_tier: Some("lexical".into()),
                tiers: vec![],
                cost: Cost {
                    micros: 7,
                    model_calls: 0,
                },
                flag_reason: None,
            },
            provenance: Provenance {
                author_kind: self.author,
                author: "test".into(),
                model: None,
                harness: None,
                run_id: None,
                prompt_hash: None,
                at: "2026-10-07T00:00:00Z".into(),
            },
        }
        .seal()
    }
}

pub fn machine(work: &'static str, code: &'static str) -> Placement {
    Spec::machine(work, code).build()
}
