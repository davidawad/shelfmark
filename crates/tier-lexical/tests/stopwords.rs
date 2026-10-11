//! Labels and terms made only of stopwords never match (`vocab-2`).
//!
//! Each scheme below reproduces a benchmark failure of `vocab-1` on an
//! arXiv corpus: a class labelled `to` (ilc2), a class labelled `To`
//! (lcc-full PL2813) and a class whose index catchwords include `TO`
//! (IPC C01G) each drew most placements.

mod common;

use common::text;
use shelfmark_core::scheme::{ClassDef, CompiledIndex, SchemeDoc, VocabLink};
use shelfmark_core::testkit::MemIndex;
use shelfmark_core::tier::{Candidate, Evidence, Tier};
use shelfmark_tier_lexical::{VocabMatcher, VocabTier};

/// An abstract full of `to`, `the`, `of` and `and`.
const ABSTRACT: &str = "We propose a method to learn to rank documents and show how to \
    apply it to the problem of retrieval. To this end, the model is trained to the \
    point of convergence.";

fn class(code: &str, label: &str, parent: Option<&str>, terms: &[&str]) -> ClassDef {
    ClassDef {
        code: code.into(),
        label: label.into(),
        parent: parent.map(Into::into),
        range: None,
        vocab: terms
            .iter()
            .map(|t| VocabLink {
                term: (*t).into(),
                source: "catchword".into(),
                source_id: None,
                weight: 1.0,
            })
            .collect(),
        note: None,
    }
}

fn scheme(name: &str, classes: Vec<ClassDef>) -> MemIndex {
    MemIndex::new(&SchemeDoc {
        schema: None,
        name: name.into(),
        version: "1".into(),
        title: format!("{name}, synthetic"),
        source: None,
        code_grammar: "[A-Za-z0-9.]+".into(),
        classes,
    })
}

/// ilc2-like: one class labelled just `to`.
fn ilc2_like() -> MemIndex {
    scheme(
        "ilc2-like",
        vec![
            class("1", "to", None, &[]),
            class("2", "Physics", None, &[]),
            class("3", "History of the arts", None, &[]),
        ],
    )
}

/// lcc-full-like: PL2813 labelled `To` under Chinese literature.
fn lcc_like() -> MemIndex {
    scheme(
        "lcc-like",
        vec![
            class("PL", "Languages and literatures of Eastern Asia", None, &[]),
            class("PL2813", "To", Some("PL"), &[]),
            class(
                "QA76",
                "Computer software",
                None,
                &["Information retrieval"],
            ),
        ],
    )
}

/// IPC-like: C01G's own label is informative, but its catchwords include
/// bare function words.
fn ipc_like() -> MemIndex {
    scheme(
        "ipc-like",
        vec![
            class("C01", "Inorganic chemistry", None, &[]),
            class(
                "C01G",
                "Compounds containing metals not covered by subclasses C01D or C01F",
                Some("C01"),
                &["TO", "of the", "rare earth metals"],
            ),
        ],
    )
}

fn run(index: &dyn CompiledIndex, input: &str) -> Vec<Candidate> {
    VocabTier::new()
        .run(&text(input), index, &[])
        .unwrap()
        .candidates
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
fn class_labelled_lowercase_to_never_matches() {
    let out = run(&ilc2_like(), ABSTRACT);
    assert!(out.is_empty(), "{out:?}");
    assert!(run(&ilc2_like(), "to").is_empty());
}

#[test]
fn class_labelled_capital_to_never_matches() {
    let out = run(&lcc_like(), ABSTRACT);
    assert!(out.is_empty(), "{out:?}");
    assert!(run(&lcc_like(), "To").is_empty());
}

#[test]
fn stopword_catchwords_never_match_an_ipc_like_class() {
    let out = run(&ipc_like(), ABSTRACT);
    assert!(out.is_empty(), "{out:?}");
    assert!(run(&ipc_like(), "TO. Of the. to").is_empty());
}

#[test]
fn stopword_only_phrases_are_left_out_of_the_matcher() {
    // ilc2-like keeps `physics` and `history of the arts`, not `to`.
    assert_eq!(VocabMatcher::build(&ilc2_like()).unwrap().phrase_count(), 2);
    // ipc-like keeps the two labels and `rare earth metals`.
    assert_eq!(VocabMatcher::build(&ipc_like()).unwrap().phrase_count(), 3);
}

#[test]
fn informative_labels_still_match_beside_a_stopword_label() {
    let out = run(&ilc2_like(), "Notes to the physics of plasmas.");
    assert_eq!(codes(&out), ["2"]);
    assert_eq!(terms(&out[0]), ["physics"]);
}

#[test]
fn labels_containing_stopwords_still_match_in_full() {
    let out = run(&ilc2_like(), "A survey to the History of the Arts.");
    assert_eq!(codes(&out), ["3"]);
    assert_eq!(terms(&out[0]), ["history of the arts"]);
}

#[test]
fn informative_terms_still_match_on_a_class_with_stopword_terms() {
    let out = run(
        &ipc_like(),
        "Separation of rare earth metals to high purity.",
    );
    assert_eq!(codes(&out), ["C01G"]);
    assert_eq!(terms(&out[0]), ["rare earth metals"]);
    assert!(out[0].score > 0.0);
}

#[test]
fn vocabulary_terms_still_match_beside_a_stopword_label() {
    let out = run(
        &lcc_like(),
        "To the reader: information retrieval to scale.",
    );
    assert_eq!(codes(&out), ["QA76"]);
    assert_eq!(terms(&out[0]), ["information retrieval"]);
}

#[test]
fn tier_version_records_the_stopword_rule() {
    assert_eq!(VocabTier::new().info().version, "vocab-2");
}
