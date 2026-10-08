//! AE6 (R14): a new scheme version that splits one class makes exactly
//! the Works placed in that class stale.

use serde_json::{json, Value};

use crate::common::{data, Bench};

/// The software span split in two.
const SPLIT: &str = "QA76.75-76.765";

/// lcc@2025: the bundled outline with [`SPLIT`] split into two child
/// classes; nothing else changes.
fn split_scheme() -> Value {
    let text = std::fs::read_to_string(data("schemes/lcc/lcc.json")).unwrap();
    let mut doc: Value = serde_json::from_str(&text).unwrap();
    doc["version"] = json!("2025");
    let classes = doc["classes"].as_array_mut().unwrap();
    classes.push(json!({"code": "QA76.75-76.755", "parent": SPLIT, "label": "Software in general",
        "range": {"start": "QA76.75", "end": "QA76.755"}}));
    classes.push(json!({"code": "QA76.76", "parent": SPLIT, "label": "Special topics"}));
    doc
}

#[test]
fn splitting_a_class_stales_exactly_the_works_placed_in_it() {
    let b = Bench::new("ae6");
    let place = |text: &str, class: &str| {
        let out = b.ok(&["classify", "--text", text, "--target", "0.5"]);
        assert_eq!(
            out["placement"]["class"],
            format!("class:lcc@2024:{class}"),
            "{out}"
        );
        out["placement"]["work"].as_str().unwrap().to_string()
    };
    let software = place("Computer software", SPLIT);
    let everyone = place("Computer software for everyone", SPLIT);
    place("Law", "K");
    place("Mathematics", "QA");
    assert_eq!(b.ok(&["stale"])["total"], 0);

    let path = b.file("lcc-2025.json", split_scheme().to_string());
    b.commit(&["scheme", "compile", &path]);

    let stale = b.ok(&["stale", "--scheme", "scheme:lcc@2025"]);
    let mut works: Vec<&str> = stale["stale"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["work"].as_str().unwrap())
        .collect();
    works.sort();
    let mut want = vec![software.as_str(), everyone.as_str()];
    want.sort();
    assert_eq!(works, want, "{stale}");
    assert_eq!(
        stale["stale"][0]["reasons"],
        json!(["class changed in scheme:lcc@2025"])
    );
}
