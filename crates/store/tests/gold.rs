//! The gold set: append-only, idempotent, hashed per scheme version.

mod common;

use common::{lcc, store};
use shelfmark_core::calibration::{gold_set_hash, GoldExample};
use shelfmark_core::placement::{AuthorKind, Provenance};
use shelfmark_core::tier::ClassifyInput;
use shelfmark_core::{ClassId, SchemeId, WorkId};
use shelfmark_store::{TailRepair, GOLD};

fn example(text: &str, scheme: SchemeId, code: &str) -> GoldExample {
    GoldExample {
        work: WorkId::from_text(text),
        item: None,
        scheme: scheme.clone(),
        class: ClassId::new(scheme, code),
        input: ClassifyInput {
            text: text.into(),
            ..ClassifyInput::default()
        },
        from_placement: None,
        provenance: Provenance {
            author_kind: AuthorKind::Human,
            author: "librarian".into(),
            model: None,
            harness: None,
            run_id: None,
            prompt_hash: None,
            at: "2026-10-07T00:00:00Z".into(),
        },
    }
}

#[test]
fn gold_appends_are_idempotent_and_ordered() {
    let (_d, s) = store();
    let a = example("a book", lcc("2024"), "QA76");
    let b = example("b book", lcc("2024"), "PS3500");
    let c = example("c book", lcc("2025"), "QA76.1");
    assert!(s.append_gold(&a).unwrap());
    assert!(!s.append_gold(&a).unwrap());
    assert!(s.append_gold(&b).unwrap());
    assert!(s.append_gold(&c).unwrap());
    assert_eq!(s.gold().unwrap(), vec![a.clone(), b.clone(), c.clone()]);
    assert_eq!(
        s.gold_for(&lcc("2024")).unwrap(),
        vec![a.clone(), b.clone()]
    );
    assert_eq!(s.gold_hash(&lcc("2024")).unwrap(), gold_set_hash(&[a, b]));
    assert_ne!(
        s.gold_hash(&lcc("2024")).unwrap(),
        s.gold_hash(&lcc("2025")).unwrap()
    );
}

#[test]
fn a_torn_gold_line_is_reported_then_repaired() {
    let (_d, s) = store();
    let a = example("a book", lcc("2024"), "QA76");
    s.append_gold(&a).unwrap();
    let path = s.root().join(GOLD);
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.extend_from_slice(b"{\"work\":");
    std::fs::write(&path, bytes).unwrap();
    let err = s.gold().unwrap_err();
    assert_eq!(err.details.unwrap()["reason"], "torn_final_line");
    assert!(s.append_gold(&example("b", lcc("2024"), "B1")).is_err());
    assert!(matches!(
        s.repair().unwrap().gold,
        TailRepair::Truncated { .. }
    ));
    assert_eq!(s.gold().unwrap(), vec![a]);
}
