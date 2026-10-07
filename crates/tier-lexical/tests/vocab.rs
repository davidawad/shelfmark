//! The `vocab` tier (plan U4; R7, AE2).

mod common;

use std::collections::BTreeMap;

use common::{mini_lcc, mini_lcc_doc, mini_lcc_with_qa76_76, text};
use shelfmark_core::cascade;
use shelfmark_core::placement::Status;
use shelfmark_core::scheme::{CompiledIndex, VocabLink};
use shelfmark_core::testkit::MemIndex;
use shelfmark_core::tier::{Candidate, ClassifyInput, Evidence, Profile, Tier};
use shelfmark_core::{ErrorCode, ProfileId};
use shelfmark_tier_lexical::{VocabMatcher, VocabTier};

fn run(index: &dyn CompiledIndex, input: &ClassifyInput) -> Vec<Candidate> {
    VocabTier::new().run(input, index, &[]).unwrap().candidates
}

fn codes(c: &[Candidate]) -> Vec<&str> {
    c.iter().map(|c| c.class.code.as_str()).collect()
}

fn terms(c: &Candidate) -> Vec<&str> {
    c.evidence
        .iter()
        .map(|e| match e {
            Evidence::Vocab { term, .. } => term.as_str(),
            other => panic!("unexpected evidence {other:?}"),
        })
        .collect()
}

#[test]
fn ae2_shape_text_naming_compilers_lands_in_qa76_with_evidence() {
    let out = run(
        &mini_lcc(),
        &text("A practical guide to writing Compilers for small languages."),
    );
    assert_eq!(codes(&out), ["QA76"]);
    assert_eq!(
        out[0].evidence,
        [Evidence::Vocab {
            term: "compilers".into(),
            source: "lcsh".into(),
            count: 1
        }]
    );
    assert!(out[0].score > 0.0 && out[0].score < 1.0);
}

#[test]
fn compilers_land_in_qa76_76_when_the_scheme_has_it() {
    let out = run(
        &mini_lcc_with_qa76_76(),
        &text("Compilers translate computer programs; this book builds compilers."),
    );
    assert_eq!(codes(&out), ["QA76.76", "QA76"]);
    assert!(out[0].score > out[1].score);
    assert_eq!(terms(&out[0]), ["compilers"]);
    assert_eq!(terms(&out[1]), ["compilers", "computer programs"]);
}

#[test]
fn ae2_equal_matches_on_unrelated_classes_flag_with_both_candidates() {
    let index = MemIndex::sample();
    let input = text("The economics of statute.");
    let out = run(&index, &input);
    assert_eq!(codes(&out), ["H", "K"]);
    assert_eq!(out[0].score, out[1].score);
    assert!(out[0].score <= 0.5);

    let tier = VocabTier::new();
    let tiers: BTreeMap<String, &dyn Tier> = [("vocab".to_string(), &tier as &dyn Tier)].into();
    let profile = Profile {
        id: ProfileId::new("lexical", "1"),
        description: "test".into(),
        tiers: vec!["vocab".into()],
        default_target: 0.8,
        min_margin: 0.1,
        expected_memory_mb: 0,
        expected_micros_per_item: 0,
    };
    let d = cascade::run(&profile, &tiers, &index, &input, None, None).unwrap();
    assert_eq!(d.class.code, "H");
    assert_eq!(d.status, Status::Flagged);
    assert_eq!(codes_of_trace(&d.trace.candidates), ["H", "K"]);
}

fn codes_of_trace(c: &[shelfmark_core::placement::TraceCandidate]) -> Vec<&str> {
    c.iter().map(|c| c.class.code.as_str()).collect()
}

#[test]
fn clear_winner_scores_above_an_even_split() {
    let index = MemIndex::sample();
    let even = run(&index, &text("economics and law"));
    let clear = run(&index, &text("economics, society and economics"));
    assert_eq!(codes(&clear), ["H"]);
    assert!(clear[0].score > even[0].score);
    let more = run(
        &index,
        &text("economics society economics society economics"),
    );
    assert!(more[0].score > clear[0].score);
}

#[test]
fn matches_whole_words_after_normalization_only() {
    let index = MemIndex::sample();
    assert!(run(&index, &text("lawyers and statutes")).is_empty());
    let out = run(&index, &text("ＬＡＷ!"));
    assert_eq!(codes(&out), ["K"]);
    assert_eq!(terms(&out[0]), ["law"]);
}

#[test]
fn a_longer_phrase_hides_the_phrase_inside_it() {
    let input = ClassifyInput {
        headings: vec!["Compilers (Computer programs)".into()],
        ..ClassifyInput::default()
    };
    let out = run(&mini_lcc(), &input);
    assert_eq!(terms(&out[0]), ["compilers computer programs"]);
}

#[test]
fn matching_never_spans_two_fields() {
    let input = ClassifyInput {
        title: Some("Computer".into()),
        text: "software".into(),
        ..ClassifyInput::default()
    };
    assert!(run(&mini_lcc(), &input).is_empty());
}

#[test]
fn identical_output_across_runs_and_tier_instances() {
    let index = mini_lcc_with_qa76_76();
    let input = ClassifyInput {
        title: Some("Law and computer software".into()),
        text: "Compilers, computer programs and the social sciences of law.".into(),
        headings: vec!["Mathematics".into()],
        ..ClassifyInput::default()
    };
    let first = serde_json::to_string(&run(&index, &input)).unwrap();
    let tier = VocabTier::for_index(&index).unwrap();
    for _ in 0..20 {
        let again = tier.run(&input, &index, &[]).unwrap().candidates;
        assert_eq!(serde_json::to_string(&again).unwrap(), first);
    }
}

#[test]
fn scores_are_finite_and_in_unit_interval() {
    let index = mini_lcc_with_qa76_76();
    let long = "law ".repeat(10_000);
    for t in [
        "",
        "   ",
        "law",
        long.as_str(),
        "science mathematics computer software compilers law social sciences",
        "Ünïcödé ﬁ ligatures and computer programs",
    ] {
        for c in run(&index, &text(t)) {
            assert!(
                c.score.is_finite() && (0.0..=1.0).contains(&c.score),
                "{t:?}: {c:?}"
            );
            assert!(!c.evidence.is_empty());
        }
    }
}

#[test]
fn ties_break_by_class_code_bytes() {
    let mut doc = mini_lcc_doc();
    for code in ["K", "H"] {
        let class = doc.classes.iter_mut().find(|c| c.code == code).unwrap();
        class.vocab.push(VocabLink {
            term: "shared".into(),
            source: "test".into(),
            source_id: None,
            weight: 1.0,
        });
    }
    let out = run(&MemIndex::new(&doc), &text("shared"));
    assert_eq!(codes(&out), ["H", "K"]);
}

#[test]
fn matcher_follows_the_index_it_is_given() {
    let tier = VocabTier::new();
    let a = tier.run(&text("law"), &MemIndex::sample(), &[]).unwrap();
    assert_eq!(a.candidates[0].class.scheme, *MemIndex::sample().scheme());
    let b = tier.run(&text("law"), &mini_lcc(), &[]).unwrap();
    assert_eq!(b.candidates[0].class.scheme, *mini_lcc().scheme());
    let m = tier.matcher(&mini_lcc()).unwrap();
    assert!(m.is_for(&mini_lcc()) && !m.is_for(&MemIndex::sample()));
    assert!(m.phrase_count() >= 7);
}

#[test]
fn vocab_weight_outside_unit_interval_is_scheme_invalid() {
    let mut doc = mini_lcc_doc();
    doc.classes[0].vocab.push(VocabLink {
        term: "bad".into(),
        source: "test".into(),
        source_id: None,
        weight: 1.5,
    });
    let err = VocabMatcher::build(&MemIndex::new(&doc)).unwrap_err();
    assert_eq!(err.code, ErrorCode::SchemeInvalid);
    assert!(err.next_command.is_some());
}

#[test]
fn info_declares_a_model_free_tier() {
    let info = VocabTier::new().info();
    assert_eq!(info.name, "vocab");
    assert!(!info.needs_model);
}
