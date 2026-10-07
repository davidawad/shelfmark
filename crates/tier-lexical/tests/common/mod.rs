//! Shared fixtures for the lexical tier tests.
#![allow(dead_code)]

use std::path::PathBuf;

use shelfmark_core::scheme::{ClassDef, SchemeDoc, VocabLink};
use shelfmark_core::testkit::MemIndex;
use shelfmark_core::tier::ClassifyInput;

pub fn mini_lcc_doc() -> SchemeDoc {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/schemes/mini-lcc.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

pub fn mini_lcc() -> MemIndex {
    MemIndex::new(&mini_lcc_doc())
}

/// mini-lcc plus QA76.76 under QA76, carrying the LCSH compilers heading
/// the way the full outline does.
pub fn mini_lcc_with_qa76_76() -> MemIndex {
    let mut doc = mini_lcc_doc();
    doc.classes.push(ClassDef {
        code: "QA76.76".into(),
        label: "Special topics, A-Z".into(),
        parent: Some("QA76".into()),
        range: None,
        vocab: vec![VocabLink {
            term: "Compilers (Computer programs)".into(),
            source: "lcsh".into(),
            source_id: Some("sh86007588".into()),
            weight: 1.0,
        }],
        note: None,
    });
    MemIndex::new(&doc)
}

pub fn text(t: &str) -> ClassifyInput {
    ClassifyInput {
        text: t.into(),
        ..ClassifyInput::default()
    }
}
