//! The feature-gated semantic stubs (plan PD4).
#![cfg(any(feature = "semantic-shortlist", feature = "semantic-rerank"))]

use shelfmark_core::error::ErrorCode;
use shelfmark_core::testkit::MemIndex;
use shelfmark_core::tier::{ClassifyInput, Tier};
use shelfmark_profile::semantic::{SemanticStub, STUB_VERSION};

fn check(stub: SemanticStub, feature: &str, model: &str) {
    let info = stub.info();
    assert!(info.needs_model);
    assert_eq!(info.version, STUB_VERSION);
    assert!(info.expected_memory_mb > 0);
    let err = stub
        .run(&ClassifyInput::default(), &MemIndex::sample(), &[])
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::TierUnavailable);
    assert!(err.message.contains(feature) && err.message.contains(model));
    let d = err.details.unwrap();
    assert_eq!(d["feature"], feature);
    assert_eq!(d["model"], model);
    assert_eq!(d["feature_enabled"], true);
}

#[cfg(feature = "semantic-shortlist")]
#[test]
fn shortlist_stub_names_feature_and_model() {
    let s = shelfmark_profile::semantic::shortlist();
    assert_eq!(s.info().name, "shortlist");
    check(s, "semantic-shortlist", "BGE-small-en-v1.5 (int8 ONNX)");
}

#[cfg(feature = "semantic-rerank")]
#[test]
fn rerank_stub_names_feature_and_model() {
    let s = shelfmark_profile::semantic::rerank();
    assert_eq!(s.info().name, "rerank");
    check(s, "semantic-rerank", "Qwen3-Reranker-0.6B (Q8)");
}
