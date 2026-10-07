//! The published scheme schema and the serde model agree (R4, plan PD7).

use std::path::PathBuf;

use serde_json::{json, Value};
use shelfmark_core::scheme::{SchemeDoc, SCHEME_SCHEMA_ID};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(rel: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(rel)).unwrap()).unwrap()
}

fn validator() -> jsonschema::Validator {
    jsonschema::validator_for(&load("schemas/scheme.schema.json")).unwrap()
}

#[test]
fn schema_id_matches_the_constant() {
    assert_eq!(load("schemas/scheme.schema.json")["$id"], SCHEME_SCHEMA_ID);
}

#[test]
fn fixture_validates_and_round_trips_through_the_model() {
    let doc = load("fixtures/schemes/mini-lcc.json");
    let v = validator();
    let errors: Vec<String> = v.iter_errors(&doc).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:?}");
    let model: SchemeDoc = serde_json::from_value(doc.clone()).unwrap();
    assert_eq!(model.id().to_string(), "scheme:mini-lcc@2024");
    assert_eq!(
        serde_json::to_value(&model).unwrap(),
        doc,
        "model drops or renames fields"
    );
}

/// Every mutation the schema rejects, the model rejects too (and the
/// reverse for the accepted minimal form), so neither is looser.
#[test]
fn schema_and_model_reject_the_same_shapes() {
    let base = json!({"name": "x", "version": "1", "title": "X", "code_grammar": "[A-Z]+",
                      "classes": [{"code": "A", "label": "A"}]});
    let v = validator();
    assert!(v.is_valid(&base));
    assert!(serde_json::from_value::<SchemeDoc>(base.clone()).is_ok());

    let mut cases = Vec::new();
    for key in ["name", "version", "title", "code_grammar", "classes"] {
        let mut d = base.clone();
        d.as_object_mut().unwrap().remove(key);
        cases.push((format!("missing {key}"), d));
    }
    let mut extra = base.clone();
    extra["colour"] = json!("red");
    cases.push(("unknown top-level field".into(), extra));
    let mut extra_class = base.clone();
    extra_class["classes"][0]["colour"] = json!("red");
    cases.push(("unknown class field".into(), extra_class));
    let mut extra_vocab = base.clone();
    extra_vocab["classes"][0]["vocab"] = json!([{"term": "t", "source": "s", "colour": 1}]);
    cases.push(("unknown vocab field".into(), extra_vocab));

    for (name, doc) in cases {
        assert!(!v.is_valid(&doc), "schema accepted {name}");
        assert!(
            serde_json::from_value::<SchemeDoc>(doc).is_err(),
            "model accepted {name}"
        );
    }
}

#[test]
fn schema_enforces_value_rules_the_model_leaves_to_the_compiler() {
    let v = validator();
    let bad_token = json!({"name": "has space", "version": "1", "title": "X", "code_grammar": "x",
                           "classes": [{"code": "A", "label": "A"}]});
    assert!(!v.is_valid(&bad_token));
    let bad_weight = json!({"name": "x", "version": "1", "title": "X", "code_grammar": "x",
                            "classes": [{"code": "A", "label": "A",
                                         "vocab": [{"term": "t", "source": "s", "weight": 0}]}]});
    assert!(!v.is_valid(&bad_weight));
    let empty =
        json!({"name": "x", "version": "1", "title": "X", "code_grammar": "x", "classes": []});
    assert!(!v.is_valid(&empty));
}
