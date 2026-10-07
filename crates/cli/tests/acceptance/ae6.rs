//! AE6 (R14): a new scheme version that splits one class makes exactly
//! the Works placed in that class stale.

use serde_json::{json, Value};

use crate::common::{fixture, Bench};

/// mini-lcc@2025: QA76 split into two child classes; nothing else changes.
fn split_scheme() -> Value {
    let text = std::fs::read_to_string(fixture("schemes/mini-lcc.json")).unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    doc["version"] = json!("2025");
    let classes = doc["classes"].as_array_mut().unwrap();
    classes.push(json!({"code": "QA76.7", "parent": "QA76", "label": "Programming languages"}));
    classes.push(json!({"code": "QA76.9", "parent": "QA76", "label": "Other topics"}));
    doc
}

#[test]
fn splitting_a_class_stales_exactly_the_works_placed_in_it() {
    let b = Bench::new("ae6");
    let place = |text: &str, class: &str| {
        let out = b.ok(&["classify", "--text", text, "--target", "0.5"]);
        assert_eq!(
            out["placement"]["class"],
            format!("class:mini-lcc@2024:{class}"),
            "{out}"
        );
        out["placement"]["work"].as_str().unwrap().to_string()
    };
    let compilers = place("Compilers (Computer programs)", "QA76");
    let programs = place("Computer programs for everyone", "QA76");
    place("Law", "K");
    place("Mathematics", "QA");
    assert_eq!(b.ok(&["stale"])["total"], 0);

    let path = b.file("mini-lcc-2025.json", split_scheme().to_string());
    b.commit(&["scheme", "compile", &path]);

    let stale = b.ok(&["stale", "--scheme", "scheme:mini-lcc@2025"]);
    let mut works: Vec<&str> = stale["stale"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["work"].as_str().unwrap())
        .collect();
    works.sort();
    let mut want = vec![compilers.as_str(), programs.as_str()];
    want.sort();
    assert_eq!(works, want, "{stale}");
    assert_eq!(
        stale["stale"][0]["reasons"],
        json!(["class changed in scheme:mini-lcc@2025"])
    );
}
