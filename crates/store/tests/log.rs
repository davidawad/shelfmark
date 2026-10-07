//! The claim log: idempotent appends, supersede chains, current placement,
//! revision, the writer lock and torn lines.

mod common;

use std::sync::mpsc;
use std::time::Duration;

use common::{lcc, machine, store, Spec};
use shelfmark_core::error::ErrorCode;
use shelfmark_core::placement::{effective_status, superseded_ids, PlacementLog, Status};
use shelfmark_core::{PlacementId, WorkId};
use shelfmark_store::{resolve_path, Store, TailRepair, PLACEMENTS};

#[test]
fn append_is_idempotent_by_id() {
    let (_d, mut s) = store();
    let a = machine("w1", "QA76");
    assert!(s.append(&a).unwrap());
    assert!(!s.append(&a).unwrap(), "same id is a no-op");
    let mut timing = a.clone();
    timing.trace.cost.micros = 123_456;
    assert!(!s.append(&timing).unwrap(), "timing does not change the id");
    assert_eq!(s.claims().unwrap(), vec![a]);
    let text = std::fs::read_to_string(s.root().join(PLACEMENTS)).unwrap();
    assert_eq!(text.lines().count(), 1);
}

#[test]
fn unsealed_claims_are_rejected() {
    let (_d, mut s) = store();
    let mut a = machine("w1", "QA76");
    a.id = PlacementId::from_canonical(b"not the content");
    assert_eq!(s.append(&a).unwrap_err().code, ErrorCode::InvalidRequest);
    assert!(s.claims().unwrap().is_empty());
}

#[test]
fn supersede_chain_never_rewrites() {
    let (_d, mut s) = store();
    let m1 = machine("w1", "QA76");
    let h1 = Spec::human("w1", "QA75", &m1).build();
    let mut rerun = Spec::machine("w1", "QA76.9");
    rerun.supersedes = Some(h1.id.clone());
    let m2 = rerun.build();
    for p in [&m1, &h1, &m2] {
        assert!(s.append(p).unwrap());
    }
    let claims = s.claims().unwrap();
    assert_eq!(
        claims,
        vec![m1.clone(), h1.clone(), m2.clone()],
        "log order, unedited"
    );
    let sup = superseded_ids(&claims);
    assert_eq!(effective_status(&claims[0], &sup), Status::Superseded);
    assert_eq!(effective_status(&claims[1], &sup), Status::Superseded);
    assert_eq!(effective_status(&claims[2], &sup), Status::Machine);
    assert_eq!(s.current(&m1.work, &lcc("2024")).unwrap(), Some(m2));
}

#[test]
fn supersede_must_name_a_live_claim_in_the_store() {
    let (_d, mut s) = store();
    let ghost = machine("w0", "QA1");
    let orphan = Spec::human("w1", "QA75", &ghost).build();
    assert_eq!(s.append(&orphan).unwrap_err().code, ErrorCode::NotFound);

    let m1 = machine("w1", "QA76");
    s.append(&m1).unwrap();
    s.append(&Spec::human("w1", "QA75", &m1).build()).unwrap();
    let fork = Spec::human("w1", "QA74", &m1).build();
    let err = s.append(&fork).unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidRequest, "{err}");
    assert_eq!(s.claims().unwrap().len(), 2);
}

#[test]
fn current_prefers_human_over_later_machine() {
    let (_d, mut s) = store();
    let m1 = machine("w1", "QA76");
    let h1 = Spec::human("w1", "QA75", &m1).build();
    let other = machine("w2", "PS3500");
    s.append(&m1).unwrap();
    s.append(&h1).unwrap();
    s.append(&other).unwrap();
    // A later machine claim that supersedes nothing does not outrank the
    // human confirmation.
    let later = Spec {
        scheme_hash: "s256:other",
        ..Spec::machine("w1", "QA70")
    }
    .build();
    s.append(&later).unwrap();
    let w1 = WorkId::from_text("w1");
    assert_eq!(s.current(&w1, &lcc("2024")).unwrap(), Some(h1.clone()));
    assert_eq!(s.current(&w1, &lcc("2025")).unwrap(), None);
    assert_eq!(s.current_all().unwrap(), vec![h1, other]);
}

#[test]
fn revision_changes_on_every_append() {
    let (_d, mut s) = store();
    let mut seen = vec![s.revision().unwrap()];
    for (w, c) in [("w1", "QA76"), ("w2", "QA76"), ("w3", "B1")] {
        s.append(&machine(w, c)).unwrap();
        let r = s.revision().unwrap();
        assert!(!seen.contains(&r), "{r} repeated");
        seen.push(r);
    }
    s.append(&machine("w1", "QA76")).unwrap();
    assert_eq!(&s.revision().unwrap(), seen.last().unwrap(), "no-op append");
}

#[test]
fn a_held_lock_blocks_a_concurrent_writer() {
    let (_d, s) = store();
    let guard = s.lock_exclusive().unwrap();
    let (tx, rx) = mpsc::channel();
    let root = s.root().to_path_buf();
    let writer = std::thread::spawn(move || {
        let mut other = Store::open(root).unwrap();
        let ok = other.append(&machine("w1", "QA76")).unwrap();
        tx.send(ok).unwrap();
    });
    assert!(
        rx.recv_timeout(Duration::from_millis(300)).is_err(),
        "writer must wait for the lock"
    );
    assert_eq!(log_lines(&s), 0, "nothing written while blocked");
    drop(guard);
    assert!(rx.recv_timeout(Duration::from_secs(10)).unwrap());
    writer.join().unwrap();
    assert_eq!(s.claims().unwrap().len(), 1);
}

/// Lines in the log, read without the lock (a shared lock would wait
/// behind the blocked writer's turn).
fn log_lines(s: &Store) -> usize {
    std::fs::read_to_string(s.root().join(PLACEMENTS))
        .map(|t| t.lines().count())
        .unwrap_or(0)
}

#[test]
fn parallel_writers_serialize_cleanly() {
    let (_d, s) = store();
    let works: Vec<&'static str> = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
    let handles: Vec<_> = works
        .iter()
        .map(|w| {
            let root = s.root().to_path_buf();
            let w = *w;
            std::thread::spawn(move || {
                let mut st = Store::open(root).unwrap();
                st.append(&machine(w, "QA76")).unwrap();
                st.append(&machine(w, "QA76")).unwrap();
            })
        })
        .collect();
    handles.into_iter().for_each(|h| h.join().unwrap());
    assert_eq!(s.claims().unwrap().len(), works.len());
    assert_eq!(s.memo_index().unwrap().len(), works.len());
}

#[test]
fn a_torn_final_line_is_reported_not_dropped() {
    let (_d, mut s) = store();
    let a = machine("w1", "QA76");
    s.append(&a).unwrap();
    let path = s.root().join(PLACEMENTS);
    let mut bytes = std::fs::read(&path).unwrap();
    let whole = serde_json::to_string(&machine("w2", "QA76")).unwrap();
    bytes.extend_from_slice(&whole.as_bytes()[..40]);
    std::fs::write(&path, &bytes).unwrap();

    let err = s.claims().unwrap_err();
    assert_eq!(err.code, ErrorCode::StoreError);
    assert_eq!(err.details.as_ref().unwrap()["reason"], "torn_final_line");
    assert_eq!(err.details.as_ref().unwrap()["torn_bytes"], 40);
    assert!(s.revision().is_err());
    assert!(s.memo(&a.inputs).is_err());
    let b = machine("w3", "B1");
    assert_eq!(s.append(&b).unwrap_err().code, ErrorCode::StoreError);

    let repair = s.repair().unwrap();
    match repair.placements {
        TailRepair::Truncated { bytes, .. } => assert_eq!(bytes, &whole[..40]),
        other => panic!("expected a cut, got {other:?}"),
    }
    assert_eq!(s.claims().unwrap(), vec![a.clone()]);
    assert!(s.append(&b).unwrap());
}

#[test]
fn a_complete_tail_missing_its_newline_is_kept_by_repair() {
    let (_d, s) = store();
    let a = machine("w1", "QA76");
    let path = s.root().join(PLACEMENTS);
    std::fs::write(&path, serde_json::to_string(&a).unwrap()).unwrap();
    assert!(s.claims().is_err());
    assert_eq!(s.repair().unwrap().placements, TailRepair::Terminated);
    assert_eq!(s.claims().unwrap(), vec![a]);
}

#[test]
fn a_corrupt_middle_line_names_its_line() {
    let (_d, s) = store();
    let path = s.root().join(PLACEMENTS);
    let a = serde_json::to_string(&machine("w1", "QA76")).unwrap();
    std::fs::write(&path, format!("{a}\nnot json\n")).unwrap();
    let err = s.claims().unwrap_err();
    assert_eq!(err.code, ErrorCode::StoreError);
    assert_eq!(err.details.unwrap()["line"], 2);
}

#[test]
fn store_path_resolution_order() {
    use std::path::{Path, PathBuf};
    let env = Some("/env/store".into());
    assert_eq!(
        resolve_path(Some(Path::new("/flag")), env.clone()),
        PathBuf::from("/flag")
    );
    assert_eq!(resolve_path(None, env), PathBuf::from("/env/store"));
    assert_eq!(
        resolve_path(None, Some("".into())),
        PathBuf::from(".shelfmark")
    );
    assert_eq!(resolve_path(None, None), PathBuf::from(".shelfmark"));
}
