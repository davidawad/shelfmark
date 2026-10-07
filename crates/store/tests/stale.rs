//! Stale listing (AE6): scheme, profile and class changes.

mod common;

use common::{lcc, machine, store, Spec};
use shelfmark_core::placement::{Placement, PlacementLog};
use shelfmark_core::{ProfileId, WorkId};
use shelfmark_store::{StaleQuery, StaleReason, Store};

fn works(s: &Store, q: &StaleQuery) -> Vec<(WorkId, Vec<StaleReason>)> {
    s.stale(q)
        .unwrap()
        .into_iter()
        .map(|c| (c.placement.work, c.reasons))
        .collect()
}

fn w(text: &str) -> WorkId {
    WorkId::from_text(text)
}

fn seed(s: &mut Store) -> Vec<Placement> {
    let claims = vec![
        machine("w1", "QA76"),
        machine("w2", "QA76"),
        machine("w3", "PS3500"),
    ];
    for c in &claims {
        s.append(c).unwrap();
    }
    claims
}

fn query(hash: &str) -> StaleQuery {
    StaleQuery {
        scheme: lcc("2025"),
        scheme_hash: hash.into(),
        profile: None,
        class: None,
    }
}

#[test]
fn unchanged_inputs_are_not_stale() {
    let (_d, mut s) = store();
    seed(&mut s);
    let mut q = query("s256:scheme-2024");
    q.profile = Some((ProfileId::new("lexical", "1"), "s256:lexical-1".into()));
    assert!(s.stale(&q).unwrap().is_empty());
}

#[test]
fn a_split_class_lists_exactly_its_works() {
    let (_d, mut s) = store();
    seed(&mut s);
    let mut q = query("s256:scheme-2025");
    q.class = Some("QA76".into());
    assert_eq!(
        works(&s, &q),
        vec![
            (w("w1"), vec![StaleReason::Scheme]),
            (w("w2"), vec![StaleReason::Scheme]),
        ]
    );
    q.class = None;
    assert_eq!(works(&s, &q).len(), 3, "a new scheme stales every claim");
}

#[test]
fn other_schemes_are_ignored() {
    let (_d, mut s) = store();
    seed(&mut s);
    let ddc = Spec {
        scheme: shelfmark_core::SchemeId::new("ddc", "23"),
        ..Spec::machine("w4", "005")
    }
    .build();
    s.append(&ddc).unwrap();
    assert!(!works(&s, &query("s256:new")).contains(&(w("w4"), vec![StaleReason::Scheme])));
}

#[test]
fn a_profile_change_stales_machine_claims_only() {
    let (_d, mut s) = store();
    let claims = seed(&mut s);
    s.append(&Spec::human("w3", "PS3501", &claims[2]).build())
        .unwrap();
    let mut q = query("s256:scheme-2024");
    q.profile = Some((ProfileId::new("lexical", "2"), "s256:lexical-2".into()));
    assert_eq!(
        works(&s, &q),
        vec![
            (w("w1"), vec![StaleReason::Profile]),
            (w("w2"), vec![StaleReason::Profile]),
        ],
        "the human claim on w3 is not stale; the superseded machine one is not live"
    );

    q.profile = Some((ProfileId::new("shortlist", "1"), "s256:other".into()));
    assert!(
        s.stale(&q).unwrap().is_empty(),
        "other profiles' claims untouched"
    );

    q.scheme_hash = "s256:scheme-2025".into();
    q.profile = Some((ProfileId::new("lexical", "2"), "s256:lexical-2".into()));
    let all = works(&s, &q);
    assert_eq!(all[0].1, vec![StaleReason::Scheme, StaleReason::Profile]);
    assert_eq!(all[2], (w("w3"), vec![StaleReason::Scheme]));
}

#[test]
fn a_rerun_superseding_the_stale_claim_clears_it() {
    let (_d, mut s) = store();
    let claims = seed(&mut s);
    let mut rerun = Spec {
        scheme: lcc("2025"),
        scheme_hash: "s256:scheme-2025",
        ..Spec::machine("w1", "QA76.1")
    };
    rerun.supersedes = Some(claims[0].id.clone());
    s.append(&rerun.build()).unwrap();
    let mut q = query("s256:scheme-2025");
    q.class = Some("QA76".into());
    assert_eq!(works(&s, &q), vec![(w("w2"), vec![StaleReason::Scheme])]);
}
