//! The memo index: hits and misses, agreement with the log, rebuilds.

mod common;

use common::{machine, store, Spec};
use shelfmark_core::placement::{InputHashes, PlacementLog, Status};
use shelfmark_core::WorkId;
use shelfmark_store::{Store, MEMO};

/// The trait's default memo, computed from the log alone.
fn from_log(s: &Store, inputs: &InputHashes) -> Option<shelfmark_core::placement::Placement> {
    struct LogOnly(Vec<shelfmark_core::placement::Placement>);
    impl PlacementLog for LogOnly {
        fn append(
            &mut self,
            _: &shelfmark_core::placement::Placement,
        ) -> Result<bool, shelfmark_core::Error> {
            unreachable!()
        }
        fn claims(
            &self,
        ) -> Result<Vec<shelfmark_core::placement::Placement>, shelfmark_core::Error> {
            Ok(self.0.clone())
        }
    }
    LogOnly(s.claims().unwrap()).memo(inputs).unwrap()
}

#[test]
fn memo_hit_and_miss() {
    let (_d, mut s) = store();
    let a = machine("w1", "QA76");
    assert_eq!(s.memo(&a.inputs).unwrap(), None, "empty store misses");
    s.append(&a).unwrap();
    assert_eq!(s.memo(&a.inputs).unwrap(), Some(a.clone()));

    let mut other = a.inputs.clone();
    other.profile = "s256:lexical-2".into();
    assert_eq!(s.memo(&other).unwrap(), None, "changed profile misses");
    other = a.inputs.clone();
    other.work = WorkId::from_text("w2");
    assert_eq!(s.memo(&other).unwrap(), None, "other work misses");
}

#[test]
fn human_and_superseded_claims_are_never_hits() {
    let (_d, mut s) = store();
    let m1 = machine("w1", "QA76");
    s.append(&m1).unwrap();
    let h1 = Spec::human("w1", "QA75", &m1).build();
    s.append(&h1).unwrap();
    assert_eq!(s.memo(&m1.inputs).unwrap(), None, "superseded by a human");

    // A flagged machine re-run with the same inputs is a hit again.
    let flagged = Spec {
        status: Status::Flagged,
        supersedes: Some(h1.id.clone()),
        ..Spec::machine("w1", "QA76.9")
    }
    .build();
    s.append(&flagged).unwrap();
    assert_eq!(s.memo(&m1.inputs).unwrap(), Some(flagged));
}

#[test]
fn superseding_falls_back_to_an_earlier_live_hit() {
    let (_d, mut s) = store();
    let first = machine("w1", "QA76");
    let second = Spec {
        status: Status::Flagged,
        ..Spec::machine("w1", "QA76")
    }
    .build();
    s.append(&first).unwrap();
    s.append(&second).unwrap();
    assert_eq!(s.memo(&first.inputs).unwrap(), Some(second.clone()));
    let mut fix = Spec::human("w1", "QA75", &second);
    fix.scheme_hash = "s256:elsewhere";
    s.append(&fix.build()).unwrap();
    assert_eq!(s.memo(&first.inputs).unwrap(), Some(first.clone()));
    assert_eq!(s.memo(&first.inputs).unwrap(), from_log(&s, &first.inputs));
}

fn populate(s: &mut Store) -> Vec<InputHashes> {
    let mut inputs = Vec::new();
    for (w, c) in [("w1", "QA76"), ("w2", "PS3500"), ("w3", "B1")] {
        let m = machine(w, c);
        s.append(&m).unwrap();
        inputs.push(m.inputs.clone());
        if w == "w2" {
            s.append(&Spec::human(w, "PS3501", &m).build()).unwrap();
        }
    }
    let rerun = Spec {
        scheme_hash: "s256:scheme-2025",
        ..Spec::machine("w3", "B2")
    }
    .build();
    s.append(&rerun).unwrap();
    inputs.push(rerun.inputs);
    inputs
}

#[test]
fn incremental_index_equals_rebuild_and_the_log() {
    let (_d, mut s) = store();
    let inputs = populate(&mut s);
    let incremental = s.memo_index().unwrap();
    for i in &inputs {
        assert_eq!(s.memo(i).unwrap(), from_log(&s, i));
    }
    s.rebuild_memo().unwrap();
    assert_eq!(s.memo_index().unwrap(), incremental);
    assert_eq!(
        incremental.len(),
        3,
        "w2's only machine claim is superseded"
    );
}

#[test]
fn a_missing_or_behind_index_is_never_trusted() {
    let (_d, mut s) = store();
    let inputs = populate(&mut s);
    let want = s.memo_index().unwrap();

    std::fs::remove_file(s.root().join(MEMO)).unwrap();
    assert_eq!(s.memo_index().unwrap(), want, "rebuilt in memory");
    for i in &inputs {
        assert_eq!(s.memo(i).unwrap(), from_log(&s, i));
    }

    // Simulate a crash after the log append but before the memo append.
    s.rebuild_memo().unwrap();
    let behind = std::fs::read(s.root().join(MEMO)).unwrap();
    let extra = machine("w4", "QA1");
    s.append(&extra).unwrap();
    std::fs::write(s.root().join(MEMO), behind).unwrap();
    assert_eq!(s.memo(&extra.inputs).unwrap(), Some(extra.clone()));

    // The next append rebuilds the stale index on disk.
    s.append(&machine("w5", "QA2")).unwrap();
    let on_disk = s.memo_index().unwrap();
    s.rebuild_memo().unwrap();
    assert_eq!(s.memo_index().unwrap(), on_disk);
    assert_eq!(on_disk.get(&extra.inputs.memo_key()), Some(&extra));
}

#[test]
fn a_store_with_only_human_claims_has_a_current_empty_index() {
    let (_d, mut s) = store();
    let m = machine("w1", "QA76");
    s.append(&m).unwrap();
    s.append(&Spec::human("w1", "QA75", &m).build()).unwrap();
    s.rebuild_memo().unwrap();
    assert!(s.memo_index().unwrap().is_empty());
    assert_eq!(s.memo(&m.inputs).unwrap(), None);
}
