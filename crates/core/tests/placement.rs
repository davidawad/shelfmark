//! Placement claims (R11-R14): content IDs, the status lattice, memo keys.

use shelfmark_core::error::Error;
use shelfmark_core::id::sha256_hex;
use shelfmark_core::placement::{
    current, effective_status, superseded_ids, AuthorKind, InputHashes, Placement, PlacementLog,
    Provenance, Status, Trace,
};
use shelfmark_core::tier::Cost;
use shelfmark_core::{ClassId, PlacementId, ProfileId, SchemeId, WorkId};

fn claim(code: &str, status: Status, author: AuthorKind) -> Placement {
    let scheme = SchemeId::new("lcc", "2024");
    Placement {
        id: PlacementId::from_canonical(b"unsealed"),
        work: WorkId::from_text("a book"),
        scheme: scheme.clone(),
        class: ClassId::new(scheme, code),
        profile: ProfileId::new("lexical", "1"),
        confidence: 0.7,
        status,
        supersedes: None,
        inputs: InputHashes {
            work: WorkId::from_text("a book"),
            scheme: "s256:aa".into(),
            profile: "s256:bb".into(),
            config: "s256:cc".into(),
        },
        trace: Trace {
            target: 0.8,
            candidates: vec![],
            margin: 1.0,
            deciding_tier: None,
            tiers: vec![],
            cost: Cost {
                micros: 5,
                model_calls: 0,
            },
            flag_reason: None,
        },
        provenance: Provenance {
            author_kind: author,
            author: "shelfmark".into(),
            model: None,
            harness: None,
            run_id: None,
            prompt_hash: None,
            at: "2026-10-07T00:00:00Z".into(),
        },
    }
    .seal()
}

#[test]
fn id_is_content_derived_and_ignores_timing() {
    let a = claim("QA76", Status::Machine, AuthorKind::Tool);
    let mut b = a.clone();
    b.trace.cost.micros = 999_999;
    assert_eq!(b.compute_id(), a.id, "timing must not change the ID");
    b.confidence = 0.71;
    assert_ne!(b.compute_id(), a.id);
    assert!(a.id.to_string().starts_with("placement:s256:"));
}

#[test]
fn placement_json_round_trips() {
    let a = claim("QA76", Status::Flagged, AuthorKind::Tool);
    let json = serde_json::to_string(&a).unwrap();
    assert!(json.contains(r#""status":"flagged""#));
    assert!(json.contains(r#""class":"class:lcc@2024:QA76""#));
    let back: Placement = serde_json::from_str(&json).unwrap();
    assert_eq!(back, a);
    assert_eq!(back.compute_id(), a.id);
}

fn superseding(code: &str, status: Status, author: AuthorKind, old: &Placement) -> Placement {
    let mut p = claim(code, status, author);
    p.supersedes = Some(old.id.clone());
    p.seal()
}

#[test]
fn human_confirmation_outranks_later_machine_claims() {
    let machine = claim("H", Status::Machine, AuthorKind::Tool);
    let confirmed = superseding("K", Status::Confirmed, AuthorKind::Human, &machine);
    let later_machine = claim("QA", Status::Machine, AuthorKind::Tool);
    let log = [machine.clone(), confirmed.clone(), later_machine.clone()];
    assert_eq!(current(&log).unwrap().id, confirmed.id);
    let superseded = superseded_ids(&log);
    assert_eq!(effective_status(&machine, &superseded), Status::Superseded);
    assert_eq!(
        effective_status(&later_machine, &superseded),
        Status::Machine
    );
}

#[test]
fn a_flagged_rerun_replaces_a_stale_machine_claim() {
    let old = claim("H", Status::Machine, AuthorKind::Tool);
    let rerun = superseding("K", Status::Flagged, AuthorKind::Tool, &old);
    assert_eq!(current(&[old.clone(), rerun.clone()]).unwrap().id, rerun.id);
    assert_eq!(current(std::slice::from_ref(&old)).unwrap().id, old.id);
}

#[test]
fn ids_survive_a_write_and_read_with_awkward_floats() {
    let mut p = claim("H", Status::Machine, AuthorKind::Tool);
    p.confidence = 0.9856906946328695;
    p.trace.target = 0.1 + 0.2;
    let p = p.seal();
    let back: Placement = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    assert_eq!(back.compute_id(), p.id);
}

#[test]
fn memo_key_changes_with_any_input() {
    let base = claim("H", Status::Machine, AuthorKind::Tool).inputs;
    let mut keys = vec![base.memo_key()];
    for field in 0..3 {
        let mut i = base.clone();
        match field {
            0 => i.scheme = "s256:other".into(),
            1 => i.profile = "s256:other".into(),
            _ => i.config = "s256:other".into(),
        }
        keys.push(i.memo_key());
    }
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), 4);
    assert_eq!(base.memo_key().len(), sha256_hex(b"").len());
}

struct VecLog(Vec<Placement>);

impl PlacementLog for VecLog {
    fn append(&mut self, p: &Placement) -> Result<bool, Error> {
        if self.0.iter().any(|x| x.id == p.id) {
            return Ok(false);
        }
        self.0.push(p.clone());
        Ok(true)
    }
    fn claims(&self) -> Result<Vec<Placement>, Error> {
        Ok(self.0.clone())
    }
}

#[test]
fn log_default_methods_memoize_machine_claims_only() {
    let mut log = VecLog(vec![]);
    let human = claim("K", Status::Confirmed, AuthorKind::Human);
    assert!(log.append(&human).unwrap());
    assert!(!log.append(&human).unwrap(), "appending twice is a no-op");
    assert_eq!(
        log.memo(&human.inputs).unwrap(),
        None,
        "human claims are not memo hits"
    );
    let machine = claim("H", Status::Machine, AuthorKind::Tool);
    log.append(&machine).unwrap();
    assert_eq!(log.memo(&machine.inputs).unwrap().unwrap().id, machine.id);
    assert_eq!(
        log.claims_for(&machine.work, &machine.scheme)
            .unwrap()
            .len(),
        2
    );

    let before = log.revision().unwrap();
    let correction = superseding("K", Status::Confirmed, AuthorKind::Human, &machine);
    log.append(&correction).unwrap();
    assert_ne!(
        log.revision().unwrap(),
        before,
        "revision changes on append"
    );
    assert_eq!(
        log.memo(&machine.inputs).unwrap(),
        None,
        "superseded claims are not memo hits"
    );
}
