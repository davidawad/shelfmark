//! The real BGE-small-en-v1.5 int8 model over the bundled LCC scheme.
//! Downloads the model on first run (into `model_dir()`: `FASTEMBED_CACHE_DIR` if set, else the user cache).
//!
//! The programming text names computer science: LCC's computing captions
//! sit four levels down (QA75.5-76.95), so the level-0/1 label texts never
//! mention computers, and a compilers blurb without that phrase lands
//! elsewhere (measured: K94, legal drafting).
#![cfg(feature = "fastembed")]

use std::path::Path;

use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{ClassifyInput, Tier};
use shelfmark_tier_shortlist::ShortlistTier;

fn top_level(index: &dyn CompiledIndex, code: &str) -> String {
    index
        .ancestors(code)
        .last()
        .map(|r| r.id.code.clone())
        .unwrap()
}

#[test]
fn real_model_places_programming_under_q_and_civil_war_under_e() {
    let lcc = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/schemes/lcc/lcc.json");
    let index = shelfmark_scheme::compile_file(&lcc).unwrap();
    let cache =
        std::env::temp_dir().join(format!("shelfmark-shortlist-bge-{}", std::process::id()));
    let tier = ShortlistTier::bge_small(Some(cache.clone()));
    assert_eq!(tier.info().version, "bge-small-en-v1.5-q/c2f-1");

    let cases = [
        (
            "Structure and Interpretation of Computer Programs",
            "An introduction to computer science and programming: procedures, data \
             abstraction, recursion, interpreters and compilers.",
            "Q",
        ),
        (
            "Battle Cry of Freedom: The Civil War Era",
            "A history of the American Civil War between the Union and the Confederacy, \
             from secession and Fort Sumter to Gettysburg, Lincoln and Reconstruction.",
            "E",
        ),
    ];
    for (title, text, want) in cases {
        let input = ClassifyInput {
            title: Some(title.into()),
            text: text.into(),
            ..ClassifyInput::default()
        };
        let out = tier.run(&input, &index, &[]).unwrap();
        let best = &out.candidates[0];
        assert_eq!(
            top_level(&index, &best.class.code),
            want,
            "{title}: {:?}",
            out.candidates
        );
        assert!(out.candidates.len() <= 10);
        assert!(out
            .candidates
            .iter()
            .all(|c| c.score.is_finite() && (0.0..=1.0).contains(&c.score)));
        assert_eq!(out.cost.model_calls, 1);
    }
    let _ = std::fs::remove_dir_all(&cache);
}
