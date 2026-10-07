//! Accretion contracts (R15-R17): gold-set hashing, calibration, config hash.

use std::collections::BTreeMap;

use serde_json::json;
use shelfmark_core::calibration::{
    config_hash, gold_set_hash, Calibration, GoldExample, TierCalibration,
};
use shelfmark_core::placement::{AuthorKind, Provenance};
use shelfmark_core::tier::ClassifyInput;
use shelfmark_core::{ClassId, SchemeId, WorkId};

fn example(code: &str) -> GoldExample {
    let scheme = SchemeId::new("lcc", "2024");
    GoldExample {
        work: WorkId::from_text(code),
        item: None,
        scheme: scheme.clone(),
        class: ClassId::new(scheme, code),
        input: ClassifyInput {
            text: format!("about {code}"),
            ..Default::default()
        },
        from_placement: None,
        provenance: Provenance {
            author_kind: AuthorKind::Human,
            author: "david".into(),
            model: None,
            harness: None,
            run_id: None,
            prompt_hash: None,
            at: "2026-10-07T00:00:00Z".into(),
        },
    }
}

#[test]
fn gold_hash_changes_when_a_label_is_added_and_round_trips() {
    let one = vec![example("H")];
    let two = vec![example("H"), example("K")];
    assert_ne!(gold_set_hash(&one), gold_set_hash(&two));
    assert_eq!(gold_set_hash(&one), gold_set_hash(&[example("H")]));
    let json = serde_json::to_string(&two[1]).unwrap();
    assert_eq!(serde_json::from_str::<GoldExample>(&json).unwrap(), two[1]);
}

#[test]
fn calibration_bins_raw_scores_and_reports_staleness() {
    let t = TierCalibration {
        bins: (0..10).map(|i| i as f64 / 20.0).collect(),
        support: 10,
    };
    assert_eq!(t.apply(0.0), 0.0);
    assert_eq!(t.apply(0.55), 0.25);
    assert_eq!(t.apply(1.0), 0.45, "1.0 falls in the last bin");
    let cal = Calibration {
        scheme: SchemeId::new("lcc", "2024"),
        gold_hash: gold_set_hash(&[example("H")]),
        gold_count: 1,
        fitted_at: "2026-10-07T00:00:00Z".into(),
        tiers: BTreeMap::from([("vocab".to_string(), t)]),
    };
    assert_eq!(cal.apply("vocab", 0.55), 0.25);
    assert_eq!(
        cal.apply("identifier", 0.55),
        0.55,
        "uncalibrated tiers keep raw scores"
    );
    assert!(!cal.is_stale(&gold_set_hash(&[example("H")])));
    assert!(cal.is_stale(&gold_set_hash(&[example("H"), example("K")])));
}

#[test]
fn config_hash_ignores_key_order() {
    let a = json!({"target": 0.8, "rules": [{"class": "QA76"}]});
    let b = json!({"rules": [{"class": "QA76"}], "target": 0.8});
    assert_eq!(config_hash(&a), config_hash(&b));
    assert_ne!(config_hash(&a), config_hash(&json!({"target": 0.9})));
}
