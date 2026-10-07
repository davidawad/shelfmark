//! The gRPC proto lists exactly the core verbs and error codes (R19).

use std::collections::BTreeSet;

use shelfmark_core::{ErrorCode, Verb};

fn proto() -> String {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../proto/shelfmark/v1/shelfmark.proto"
    );
    std::fs::read_to_string(p).unwrap()
}

/// Names declared in `enum <name> { ... }`, without the prefix and the
/// UNSPECIFIED zero value.
fn enum_values(src: &str, name: &str, prefix: &str) -> BTreeSet<String> {
    let start = src.find(&format!("enum {name} {{")).expect("enum present");
    let body = &src[start..start + src[start..].find('}').unwrap()];
    body.lines()
        .filter_map(|l| l.trim().split_once(" = "))
        .map(|(n, _)| n.trim_start_matches(prefix).to_string())
        .filter(|n| n != "UNSPECIFIED")
        .collect()
}

fn upper(s: &str) -> String {
    s.replace('-', "_").to_uppercase()
}

#[test]
fn proto_error_codes_match_core() {
    let core: BTreeSet<String> = ErrorCode::ALL.iter().map(|c| upper(c.as_str())).collect();
    assert_eq!(enum_values(&proto(), "ErrorCode", "ERROR_CODE_"), core);
}

#[test]
fn proto_verbs_match_core() {
    let core: BTreeSet<String> = Verb::ALL.iter().map(|v| upper(v.as_str())).collect();
    assert_eq!(enum_values(&proto(), "Verb", "VERB_"), core);
}
