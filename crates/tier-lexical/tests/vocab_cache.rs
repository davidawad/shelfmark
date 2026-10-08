//! Saved vocab matchers: a later process loads the matcher instead of
//! rebuilding it, and anything not current is rebuilt.

mod common;

use common::{mini_lcc, mini_lcc_with_qa76_76, text};
use shelfmark_core::tier::Tier;
use shelfmark_tier_lexical::{VocabMatcher, VocabTier};

fn saved_files(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|d| {
            d.map(|e| e.unwrap().file_name().into_string().unwrap())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn a_saved_matcher_round_trips_unchanged() {
    let index = mini_lcc_with_qa76_76();
    let built = VocabMatcher::build(&index).unwrap();
    let loaded = VocabMatcher::from_bytes(&built.to_bytes(), &index).unwrap();
    assert!(loaded == built);
}

#[test]
fn a_saved_matcher_for_another_index_or_format_is_refused() {
    let built = VocabMatcher::build(&mini_lcc_with_qa76_76()).unwrap();
    let bytes = built.to_bytes();
    assert!(VocabMatcher::from_bytes(&bytes, &mini_lcc()).is_none());
    let mut other_format = bytes.clone();
    other_format[8] ^= 0xff;
    assert!(VocabMatcher::from_bytes(&other_format, &mini_lcc_with_qa76_76()).is_none());
    assert!(
        VocabMatcher::from_bytes(&bytes[..bytes.len() / 2], &mini_lcc_with_qa76_76()).is_none()
    );
    assert!(VocabMatcher::from_bytes(b"not a matcher", &mini_lcc_with_qa76_76()).is_none());
}

#[test]
fn a_tier_with_a_cache_dir_saves_once_and_a_new_tier_loads_it() {
    let dir = tempfile::tempdir().unwrap();
    let index = mini_lcc_with_qa76_76();
    let input = text("A book about compilers and computer programs.");
    let first = VocabTier::with_cache_dir(dir.path())
        .run(&input, &index, &[])
        .unwrap();
    let files = saved_files(dir.path());
    assert_eq!(files.len(), 1, "{files:?}");

    // A fresh tier (a new process) gives the same output from the file.
    let second = VocabTier::with_cache_dir(dir.path())
        .run(&input, &index, &[])
        .unwrap();
    assert_eq!(first.candidates, second.candidates);
    assert_eq!(saved_files(dir.path()), files);
}

#[test]
fn a_corrupt_saved_matcher_is_rebuilt_and_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let index = mini_lcc_with_qa76_76();
    let input = text("compilers");
    let expected = VocabTier::new().run(&input, &index, &[]).unwrap();
    VocabTier::with_cache_dir(dir.path())
        .run(&input, &index, &[])
        .unwrap();
    let file = dir.path().join(&saved_files(dir.path())[0]);
    std::fs::write(&file, b"garbage").unwrap();

    let out = VocabTier::with_cache_dir(dir.path())
        .run(&input, &index, &[])
        .unwrap();
    assert_eq!(out.candidates, expected.candidates);
    let fixed = std::fs::read(&file).unwrap();
    assert!(VocabMatcher::from_bytes(&fixed, &index).is_some());
}

#[test]
fn an_unwritable_cache_dir_still_classifies() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("file");
    std::fs::write(&blocker, b"").unwrap();
    let index = mini_lcc();
    let out = VocabTier::with_cache_dir(blocker.join("vocab"))
        .run(&text("law"), &index, &[])
        .unwrap();
    assert!(!out.candidates.is_empty());
}
