//! The tier end to end over recorded pages and the LCC outline, and in
//! a cascade.

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use common::{client, fixture, lcc, query, url};
use shelfmark_core::cascade;
use shelfmark_core::placement::{Status, TierRunState};
use shelfmark_core::tier::{ClassifyInput, Evidence, Profile, Tier};
use shelfmark_core::ProfileId;
use shelfmark_tier_openlibrary::{FixtureTransport, OpenLibraryTier, NAME};

fn input(title: &str, authors: &[&str]) -> ClassifyInput {
    ClassifyInput {
        title: Some(title.into()),
        authors: authors.iter().map(|a| (*a).to_string()).collect(),
        ..ClassifyInput::default()
    }
}

fn tier_over(pages: &[(&str, &[&str], &str)]) -> (OpenLibraryTier, Arc<FixtureTransport>) {
    let mut t = FixtureTransport::new();
    for (title, authors, page) in pages {
        t = t.with(url(&query(title, authors)), fixture(page));
    }
    let t = Arc::new(t);
    let (c, _) = client(t.clone(), None);
    (OpenLibraryTier::new(c), t)
}

#[test]
fn info_names_the_tier() {
    let (tier, _) = tier_over(&[]);
    let info = tier.info();
    assert_eq!((info.name.as_str(), info.version.as_str()), (NAME, "1"));
    assert!(!info.needs_model);
}

#[test]
fn sicp_lands_in_computer_science_with_provenance() {
    let (tier, _) = tier_over(&[(
        "Structure and Interpretation of Computer Programs",
        &["Harold Abelson", "Gerald Jay Sussman"],
        "sicp.json",
    )]);
    let out = tier
        .run(
            &input(
                "Structure and Interpretation of Computer Programs",
                &["Harold Abelson", "Gerald Jay Sussman"],
            ),
            lcc(),
            &[],
        )
        .unwrap();
    assert_eq!(out.candidates.len(), 1);
    let top = &out.candidates[0];
    assert_eq!(top.class.code, "QA75.5-76.95");
    assert!((top.score - 1.0).abs() < 1e-12, "every call number agrees");
    match &top.evidence[..] {
        [Evidence::Catalog {
            catalog,
            record,
            call_number,
            similarity,
        }] => {
            assert_eq!(catalog, "openlibrary");
            assert_eq!(record, "/works/OL3267304W");
            assert_eq!(call_number, "QA-0076.60000000");
            assert!((*similarity - 1.0).abs() < 1e-12);
        }
        other => panic!("evidence {other:?}"),
    }
}

#[test]
fn split_call_numbers_share_the_confidence() {
    let (tier, _) = tier_over(&[("The Hobbit", &["J. R. R. Tolkien"], "hobbit.json")]);
    let out = tier
        .run(&input("The Hobbit", &["J. R. R. Tolkien"]), lcc(), &[])
        .unwrap();
    // Its editions split between juvenile fiction (PZ, the majority in
    // the recorded page) and Tolkien's own number in English literature.
    let codes: Vec<&str> = out
        .candidates
        .iter()
        .map(|c| c.class.code.as_str())
        .collect();
    assert_eq!(codes[..2], ["PZ5-90", "PR6000-6049"]);
    let (first, second) = (out.candidates[0].score, out.candidates[1].score);
    assert!(
        first > second && first < 1.0 && second >= 0.5,
        "{first} {second}"
    );
    assert!(out
        .candidates
        .iter()
        .all(|c| (0.0..=1.0).contains(&c.score)));
}

#[test]
fn raw_lc_classifications_are_quoted_first() {
    let (tier, _) = tier_over(&[("Capital", &["Karl Marx"], "synthetic.json")]);
    let out = tier
        .run(&input("Capital", &["Karl Marx"]), lcc(), &[])
        .unwrap();
    let top = &out.candidates[0];
    assert!(top.class.code.starts_with("HB"), "{}", top.class.code);
    let Evidence::Catalog { call_number, .. } = &top.evidence[0] else {
        panic!("catalog evidence")
    };
    assert_eq!(call_number, "HB501 .M37 1990");
}

#[test]
fn a_record_without_placeable_numbers_gives_way() {
    let (tier, _) = tier_over(&[("Night Thoughts", &["Edward Young"], "synthetic.json")]);
    let out = tier
        .run(&input("Night Thoughts", &["Edward Young"]), lcc(), &[])
        .unwrap();
    let Evidence::Catalog { record, .. } = &out.candidates[0].evidence[0] else {
        panic!("catalog evidence")
    };
    assert_eq!(record, "/works/OL5SYNW");
}

#[test]
fn below_threshold_returns_nothing() {
    let page = [("Dune Messiah", &["Jane Austen"][..], "synthetic.json")];
    let (tier, _) = tier_over(&page);
    let out = tier
        .run(&input("Dune Messiah", &["Jane Austen"]), lcc(), &[])
        .unwrap();
    assert!(out.candidates.is_empty());
    // A looser threshold takes the partial match, at a lower score.
    let (tier, _) = tier_over(&page);
    let out = tier
        .with_threshold(0.4)
        .run(&input("Dune Messiah", &["Jane Austen"]), lcc(), &[])
        .unwrap();
    assert!(out.candidates[0].score < 0.8, "{}", out.candidates[0].score);
}

#[test]
fn an_original_language_work_places_by_its_edition_title() {
    let (tier, _) = tier_over(&[(
        "Critique of Pure Reason",
        &["Immanuel Kant"],
        "synthetic.json",
    )]);
    let out = tier
        .run(
            &input("Critique of Pure Reason", &["Immanuel Kant"]),
            lcc(),
            &[],
        )
        .unwrap();
    assert!(
        out.candidates[0].class.code.starts_with('B'),
        "{:?}",
        out.candidates
    );
}

#[test]
fn no_title_sends_no_request() {
    let (tier, t) = tier_over(&[]);
    let mut i = input("  ", &["Frank Herbert"]);
    i.text = "a desert planet".into();
    assert!(tier.run(&i, lcc(), &[]).unwrap().candidates.is_empty());
    i.title = None;
    assert!(tier.run(&i, lcc(), &[]).unwrap().candidates.is_empty());
    assert!(t.requests().is_empty());
}

fn profile() -> Profile {
    Profile {
        id: ProfileId::new("openlibrary", "1"),
        description: "test".into(),
        tiers: vec![NAME.into()],
        default_target: 0.8,
        min_margin: 0.1,
        expected_memory_mb: 4,
        expected_micros_per_item: 1,
    }
}

#[test]
fn decides_in_a_cascade() {
    let (tier, _) = tier_over(&[(
        "Structure and Interpretation of Computer Programs",
        &["Harold Abelson"],
        "sicp.json",
    )]);
    let tiers: BTreeMap<String, &dyn Tier> = [(NAME.to_string(), &tier as &dyn Tier)].into();
    let d = cascade::run(
        &profile(),
        &tiers,
        lcc(),
        &input(
            "Structure and Interpretation of Computer Programs",
            &["Harold Abelson"],
        ),
        None,
        None,
    )
    .unwrap();
    assert_eq!(d.status, Status::Machine);
    assert_eq!(d.class.code, "QA75.5-76.95");
    assert_eq!(d.trace.deciding_tier.as_deref(), Some(NAME));
}

#[test]
fn unreachable_open_library_is_unavailable_not_fatal() {
    let (tier, _) = tier_over(&[]);
    let tiers: BTreeMap<String, &dyn Tier> = [(NAME.to_string(), &tier as &dyn Tier)].into();
    let d = cascade::run(&profile(), &tiers, lcc(), &input("Dune", &[]), None, None).unwrap();
    assert_eq!(d.status, Status::Flagged);
    assert_eq!(d.trace.tiers[0].state, TierRunState::Unavailable);
}
