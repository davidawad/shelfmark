//! The in-memory log index: a long-lived handle sees other writers,
//! survives a cut log, and never trusts a replaced or behind memo file.

mod common;

use common::{lcc, machine, store, Spec};
use shelfmark_core::placement::{InputHashes, PlacementLog};
use shelfmark_core::WorkId;
use shelfmark_store::{Store, MEMO, PLACEMENTS};

/// The trait's default memo and claims_for, computed from `claims()`.
struct LogOnly(Vec<shelfmark_core::placement::Placement>);

impl PlacementLog for LogOnly {
    fn append(
        &mut self,
        _: &shelfmark_core::placement::Placement,
    ) -> Result<bool, shelfmark_core::Error> {
        unreachable!()
    }
    fn claims(&self) -> Result<Vec<shelfmark_core::placement::Placement>, shelfmark_core::Error> {
        Ok(self.0.clone())
    }
}

fn agrees_with_log(s: &Store, inputs: &[InputHashes]) {
    let log = LogOnly(s.claims().unwrap());
    for i in inputs {
        assert_eq!(s.memo(i).unwrap(), log.memo(i).unwrap());
        assert_eq!(
            s.claims_for(&i.work, &lcc("2024")).unwrap(),
            log.claims_for(&i.work, &lcc("2024")).unwrap()
        );
    }
    let on_disk = s.memo_index().unwrap();
    s.rebuild_memo().unwrap();
    assert_eq!(s.memo_index().unwrap(), on_disk, "memo.jsonl is current");
}

#[test]
fn a_kept_handle_sees_appends_from_another_handle() {
    let (_d, mut a) = store();
    let mut b = Store::open(a.root()).unwrap();
    let first = machine("w1", "QA76");
    a.append(&first).unwrap();
    assert_eq!(a.revision().unwrap(), b.revision().unwrap());

    let fix = Spec::human("w1", "QA75", &first).build();
    b.append(&fix).unwrap();
    assert_eq!(
        a.memo(&first.inputs).unwrap(),
        None,
        "b's supersede is seen"
    );
    assert_eq!(a.revision().unwrap(), b.revision().unwrap());
    assert!(
        a.append(&fix.clone()).is_ok_and(|added| !added),
        "b's claim is a duplicate for a"
    );

    let second = machine("w2", "PS3500");
    a.append(&second).unwrap();
    b.append(&machine("w3", "B1")).unwrap();
    a.append(&machine("w4", "QA1")).unwrap();
    let inputs: Vec<InputHashes> = ["w1", "w2", "w3", "w4"]
        .iter()
        .map(|w| InputHashes {
            work: WorkId::from_text(w),
            ..second.inputs.clone()
        })
        .collect();
    agrees_with_log(&a, &inputs);
    agrees_with_log(&b, &inputs);
}

#[test]
fn claims_for_reads_only_the_pair() {
    let (_d, mut s) = store();
    let m1 = machine("w1", "QA76");
    s.append(&m1).unwrap();
    s.append(&machine("w2", "QA76")).unwrap();
    let other_scheme = Spec {
        scheme: lcc("2025"),
        ..Spec::machine("w1", "QA76")
    }
    .build();
    s.append(&other_scheme).unwrap();
    let h1 = Spec::human("w1", "QA75", &m1).build();
    s.append(&h1).unwrap();
    let w1 = WorkId::from_text("w1");
    assert_eq!(s.claims_for(&w1, &lcc("2024")).unwrap(), vec![m1, h1]);
    assert_eq!(s.claims_for(&w1, &lcc("2025")).unwrap(), vec![other_scheme]);
    assert!(s
        .claims_for(&WorkId::from_text("w9"), &lcc("2024"))
        .unwrap()
        .is_empty());
}

#[test]
fn a_log_rewritten_under_the_handle_is_parsed_again() {
    let (_d, mut s) = store();
    let a = machine("w1", "QA76");
    let b = machine("w2", "QA76");
    s.append(&a).unwrap();
    s.append(&b).unwrap();
    // Another tool replaces the log outright with a shorter one.
    let path = s.root().join(PLACEMENTS);
    let c = machine("w3", "PS3500");
    std::fs::write(&path, format!("{}\n", serde_json::to_string(&c).unwrap())).unwrap();
    assert_eq!(s.revision().unwrap(), format!("log:1:{}", c.id));
    assert_eq!(s.memo(&a.inputs).unwrap(), None);
    assert_eq!(s.memo(&c.inputs).unwrap(), Some(c));
}

/// Detecting the replacement relies on the file's inode.
#[cfg(unix)]
#[test]
fn a_memo_file_replaced_by_another_writer_is_checked_again() {
    let (_d, mut s) = store();
    let first = machine("w1", "QA76");
    s.append(&first).unwrap();
    let behind = std::fs::read(s.root().join(MEMO)).unwrap();
    s.append(&machine("w2", "QA76")).unwrap();

    // Replace the memo with a longer but behind file through a rename,
    // as a crashed rebuild elsewhere could leave it.
    let mut padded = behind.clone();
    padded.extend_from_slice(&behind);
    padded.extend_from_slice(&behind);
    let tmp = s.root().join("memo.tmp");
    std::fs::write(&tmp, &padded).unwrap();
    std::fs::rename(&tmp, s.root().join(MEMO)).unwrap();

    let third = machine("w3", "QA76");
    s.append(&third).unwrap();
    let on_disk = s.memo_index().unwrap();
    s.rebuild_memo().unwrap();
    assert_eq!(s.memo_index().unwrap(), on_disk, "the append rebuilt it");
    assert_eq!(on_disk.len(), 3);
}
