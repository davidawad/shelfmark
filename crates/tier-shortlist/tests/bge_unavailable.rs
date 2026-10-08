//! A model that cannot load makes the tier `tier_unavailable`, once and
//! for good. Its own test binary, because it points `FASTEMBED_CACHE_DIR`
//! somewhere unusable for the whole process.
#![cfg(feature = "fastembed")]

use shelfmark_core::error::ErrorCode;
use shelfmark_core::testkit::MemIndex;
use shelfmark_core::tier::{ClassifyInput, Tier};
use shelfmark_tier_shortlist::ShortlistTier;

#[test]
fn load_failure_is_tier_unavailable_and_cached() {
    let blocker = std::env::temp_dir().join(format!(
        "shelfmark-shortlist-not-a-dir-{}",
        std::process::id()
    ));
    std::fs::write(&blocker, b"a file where the model cache should be").unwrap();
    std::env::set_var("FASTEMBED_CACHE_DIR", blocker.join("cache"));

    let tier = ShortlistTier::bge_small(None);
    // Building the tier loads nothing.
    assert_eq!(tier.info().name, "shortlist");
    let input = ClassifyInput {
        text: "Anything at all".into(),
        ..ClassifyInput::default()
    };
    let index = MemIndex::sample();
    let first = tier.run(&input, &index, &[]).unwrap_err();
    assert_eq!(first.code, ErrorCode::TierUnavailable, "{first:?}");
    assert!(first.next_command.is_some());
    let details = first.details.clone().unwrap();
    assert_eq!(details["model"], "bge-small-en-v1.5-q");
    assert_eq!(details["tier"], "shortlist");
    let second = tier.run(&input, &index, &[]).unwrap_err();
    assert_eq!(first, second);
    let _ = std::fs::remove_file(&blocker);
}
