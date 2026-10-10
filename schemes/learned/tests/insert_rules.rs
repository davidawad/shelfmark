//! The two split triggers and the stability check, on a synthetic scheme.
use learned_scheme::insert::*;
use learned_scheme::vectors::Quantized;
use shelfmark_core::scheme::{ClassDef, SchemeDoc};

fn class(code: &str, parent: Option<&str>) -> ClassDef {
    ClassDef {
        code: code.into(),
        label: format!("Class {code}"),
        parent: parent.map(Into::into),
        range: None,
        vocab: vec![],
        note: None,
    }
}

/// Root class 01 with leaves 01.01 (along x) and 01.02 (along y).
fn setup(params: InsertParams) -> Inserter {
    let doc = SchemeDoc {
        schema: None,
        name: "t".into(),
        version: "1".into(),
        title: "t".into(),
        source: None,
        code_grammar: ".*".into(),
        classes: vec![
            class("01", None),
            class("01.01", Some("01")),
            class("01.02", Some("01")),
        ],
    };
    let leaf = |code: &str, v: &[f64]| LeafCentroid {
        code: code.into(),
        n: 20,
        v: Quantized::new(v),
    };
    let centroids = CentroidFile {
        scheme: "t@1".into(),
        model: "test".into(),
        dims: 3,
        leaves: vec![
            leaf("01.01", &[1.0, 0.0, 0.0]),
            leaf("01.02", &[0.0, 1.0, 0.0]),
        ],
    };
    Inserter::new(doc, &centroids, params).unwrap()
}

#[test]
fn a_distinct_group_splits_only_its_leaf() {
    let mut ins = setup(InsertParams {
        split_after: 6,
        ..InsertParams::default()
    });
    let before = ins.doc.clone();
    // Near 01.01 but leaning toward z: nearest leaf 01.01, distinct from its core.
    for k in 0..6 {
        let code = ins.insert(
            &format!("w{k}"),
            "tidal waves",
            &[0.4, 0.0, 0.9 + 0.01 * k as f64],
        );
        assert!(code.starts_with("01.01"));
    }
    assert_eq!(ins.splits.len(), 1);
    assert_eq!(ins.splits[0].cause, "distinct");
    check_stable(&before, &ins.doc).unwrap();
    assert_eq!(ins.class_of("w0"), Some("01.01.02"));
    assert_eq!(ins.refined("01.01"), "01.01.01");
    assert_eq!(ins.doc.classes[3].label, "Tidal / waves");
    assert_eq!(ins.doc.classes[2].label, "Class 01.01 (general)");
}

#[test]
fn a_full_leaf_splits_even_without_a_distinct_group() {
    let mut ins = setup(InsertParams {
        split_after: 1000,
        max_cos: -1.0,
        ..InsertParams::default()
    });
    for k in 0..20 {
        let t = k as f64 / 19.0;
        ins.insert(&format!("w{k}"), "x", &[1.0, 0.3 * t, 0.3 * (1.0 - t)]);
    }
    assert_eq!(ins.splits.len(), 1);
    assert_eq!(ins.splits[0].cause, "capacity");
    assert_eq!(ins.splits[0].leaf, "01.01");
}

#[test]
fn stability_check_rejects_changes_to_existing_classes() {
    let ins = setup(InsertParams::default());
    let before = ins.doc.clone();
    let mut relabelled = before.clone();
    relabelled.classes[1].label = "Other".into();
    assert!(check_stable(&before, &relabelled).is_err());
    let mut under_internal = before.clone();
    under_internal.classes.push(class("01.03", Some("01")));
    assert!(check_stable(&before, &under_internal).is_err());
    let mut reordered = before.clone();
    reordered.classes.swap(1, 2);
    assert!(check_stable(&before, &reordered).is_err());
}
