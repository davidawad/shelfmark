//! Live check against openlibrary.org. Ignored by default; run with
//! `cargo test -p shelfmark-tier-openlibrary --features http -- --ignored`.

#![cfg(feature = "http")]

mod common;

use shelfmark_core::tier::{ClassifyInput, Tier};
use shelfmark_tier_openlibrary::OpenLibraryTier;

#[test]
#[ignore = "needs network: sends one request to openlibrary.org"]
fn live_lookup_places_a_well_known_book() {
    let dir = tempfile::tempdir().unwrap();
    let tier = OpenLibraryTier::online(Some(dir.path().to_path_buf()));
    let input = ClassifyInput {
        title: Some("Structure and Interpretation of Computer Programs".into()),
        authors: vec!["Harold Abelson".into()],
        ..ClassifyInput::default()
    };
    let out = tier.run(&input, common::lcc(), &[]).unwrap();
    let top = out.candidates.first().expect("a candidate");
    assert!(top.class.code.starts_with("QA"), "{}", top.class.code);
    assert!(std::fs::read_dir(dir.path()).unwrap().count() > 0, "cached");
}
