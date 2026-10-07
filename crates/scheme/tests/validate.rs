//! Invalid scheme files fail with pointer-precise problems (R4).

use serde_json::{json, Value};
use shelfmark_core::ErrorCode;
use shelfmark_scheme::{compile_bytes, validate_value, Diagnostic};

fn base() -> Value {
    json!({
        "name": "t", "version": "1", "title": "T",
        "code_grammar": "[A-Z]{1,3}([0-9]+(\\.[0-9]+)?)?",
        "classes": [
            {"code": "Q", "label": "Science"},
            {"code": "QA", "label": "Mathematics", "parent": "Q",
             "range": {"start": "QA1", "end": "QA939"}},
            {"code": "QA76", "label": "Computer software", "parent": "QA",
             "range": {"start": "QA75.5", "end": "QA76.95"}}
        ]
    })
}

fn problems(doc: &Value) -> Vec<Diagnostic> {
    validate_value(doc).expect_err("should be invalid")
}

fn pointers(doc: &Value) -> Vec<String> {
    problems(doc).into_iter().map(|d| d.pointer).collect()
}

#[test]
fn the_base_is_valid() {
    validate_value(&base()).unwrap();
}

#[test]
fn schema_errors_name_the_value() {
    let mut d = base();
    d["classes"][1]["label"] = json!("");
    d["classes"][2]["vocab"] = json!([{"term": "x", "source": "lcsh", "weight": 2}]);
    d["classes"][0]["colour"] = json!("red");
    d["name"] = json!("has space");
    assert_eq!(
        pointers(&d),
        [
            "/classes/0",
            "/classes/1/label",
            "/classes/2/vocab/0/weight",
            "/name"
        ]
    );
    let mut d = base();
    d.as_object_mut().unwrap().remove("title");
    let p = problems(&d);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].pointer, "");
    assert!(p[0].message.contains("title"), "{}", p[0].message);
}

#[test]
fn codes_must_match_the_grammar() {
    let mut d = base();
    d["classes"][0]["code"] = json!("q");
    d["classes"][1]["parent"] = json!("q");
    d["classes"][2]["range"]["end"] = json!("QA76.95.C3");
    let p = problems(&d);
    assert_eq!(
        p.iter().map(|d| d.pointer.as_str()).collect::<Vec<_>>(),
        [
            "/classes/0/code",
            "/classes/1/range",
            "/classes/2/range/end"
        ],
        "QA's range no longer sits under its renamed parent either"
    );
    assert!(p[0].message.contains("`q` does not match code_grammar"));
}

#[test]
fn bad_grammar_regex() {
    let mut d = base();
    d["code_grammar"] = json!("[A-Z");
    assert_eq!(pointers(&d), ["/code_grammar"]);
}

#[test]
fn ranges_are_well_ordered() {
    let mut d = base();
    d["classes"][2]["range"] = json!({"start": "QA76.95", "end": "QA75.5"});
    let p = problems(&d);
    assert_eq!(p[0].pointer, "/classes/2/range");
    assert!(p[0].message.contains("sorts after"), "{}", p[0].message);
}

#[test]
fn ranges_nest_in_their_parent() {
    // Parent with a range: QA940 is past QA939.
    let mut d = base();
    d["classes"][2]["range"]["end"] = json!("QA940");
    let p = problems(&d);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].pointer, "/classes/2/range");
    assert!(
        p[0].message.contains("not inside parent `QA` (QA1-QA939)"),
        "{}",
        p[0].message
    );
    // Parent without a range: the range must sit under the parent's code.
    let mut d = base();
    d["classes"][1]["range"] = json!({"start": "QA1", "end": "R5"});
    assert_eq!(pointers(&d), ["/classes/1/range"]);
    d["classes"][1]["range"] = json!({"start": "QA1", "end": "QB5"});
    validate_value(&d).expect("QB5 is still under Q");
}

#[test]
fn parents_exist_are_acyclic_and_codes_unique() {
    let mut d = base();
    d["classes"][1]["parent"] = json!("Z");
    assert_eq!(pointers(&d), ["/classes/1/parent"]);
    let mut d = base();
    d["classes"][0]["parent"] = json!("QA76");
    assert!(pointers(&d).contains(&"/classes/0/parent".to_string()));
    let mut d = base();
    d["classes"][2]["code"] = json!("QA");
    d["classes"][2].as_object_mut().unwrap().remove("range");
    let p = problems(&d);
    assert_eq!(p[0].pointer, "/classes/2/code");
    assert!(p[0]
        .message
        .contains("duplicate code `QA` (first at /classes/1)"));
}

#[test]
fn compile_wraps_problems_in_scheme_invalid() {
    let mut d = base();
    d["classes"][1]["label"] = json!("");
    let err = compile_bytes(d.to_string().as_bytes(), "bad.json").unwrap_err();
    assert_eq!(err.code, ErrorCode::SchemeInvalid);
    assert_eq!(
        err.next_command.as_deref(),
        Some("shelfmark scheme validate bad.json")
    );
    let details = err.details.unwrap();
    assert_eq!(details["errors"][0]["pointer"], "/classes/1/label");
    assert!(err.message.contains("/classes/1/label"), "{}", err.message);

    let err = compile_bytes(b"{ not json", "broken.json").unwrap_err();
    assert_eq!(err.code, ErrorCode::SchemeInvalid);
    assert!(err.message.contains("line 1"), "{}", err.message);
}
