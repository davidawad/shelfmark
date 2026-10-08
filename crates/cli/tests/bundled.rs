//! The data compiled into the binary: the LCC outline (U2) is the default
//! scheme of an empty store, and every starter rule names a class of it.

mod common;

use common::{call, data, temp_dir};
use serde_json::json;
use shelfmark_core::scheme::CodeCheck;
use shelfmark_core::Verb;

#[test]
fn the_lcc_outline_is_the_default_scheme() {
    let s = shelfmark_cli::wiring::open(&temp_dir("bundled").join("store"));
    let d = data(call(&s, Verb::Status, json!({})));
    let lcc = &d["schemes"][0];
    assert_eq!(lcc["id"], "scheme:lcc@2024");
    assert_eq!(lcc["compiled"], true);
    assert!(lcc["classes"].as_u64().unwrap() > 7_000, "{lcc}");

    let c = data(call(&s, Verb::Classify, json!({"text": "Museums"})));
    assert_eq!(c["placement"]["scheme"], "scheme:lcc@2024");
}

#[test]
fn every_starter_rule_names_a_class_of_the_outline() {
    let index = shelfmark_cli::wiring::bundled_lcc();
    let rules = shelfmark_cli::wiring::starter_rules().rules;
    assert!(rules.len() > 200, "{}", rules.len());
    for rule in &rules {
        match index.check_code(&rule.class) {
            CodeCheck::Exact { .. } | CodeCheck::Within { .. } => {}
            other => panic!("{:?} -> {}: {other:?}", rule.id, rule.class),
        }
    }
}

#[test]
fn starter_rules_decide_on_an_arxiv_stamp_and_a_decide_domain_only() {
    let s = shelfmark_cli::wiring::open(&temp_dir("starter").join("store"));
    let place = |args: serde_json::Value| data(call(&s, Verb::Classify, args))["placement"].clone();

    let stamped = place(json!({"text": "arXiv:2401.03616v1 [cs.LG] 7 Jan 2024"}));
    assert_eq!(stamped["class"], "class:lcc@2024:Q325.5");
    assert_eq!(stamped["trace"]["deciding_tier"], "rules");

    let docs = place(json!({"text": "serde", "url": "https://docs.rs/serde"}));
    assert_eq!(docs["class"], "class:lcc@2024:QA76.73.R87");
    assert_eq!(docs["status"], "machine");

    // A hint is a prior: it may be the best guess, never a decision.
    let hinted = place(json!({"text": "bar", "url": "https://github.com/foo/bar"}));
    assert_eq!(hinted["status"], "flagged");
}

#[test]
fn a_store_rules_file_replaces_the_starter_rules() {
    let store = temp_dir("own-rules").join("store");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::write(
        store.join(shelfmark_cli::wiring::RULES_FILE),
        r#"{"rules": [{"when": {"term": "statute"}, "class": "K", "weight": 0.9}]}"#,
    )
    .unwrap();
    let s = shelfmark_cli::wiring::open(&store);
    let env = call(&s, Verb::Classify, json!({"text": "serde", "url": "https://docs.rs/serde"}));
    let starter_silent = match env {
        shelfmark_core::Envelope::Ok { data, .. } => data["placement"]["trace"]["deciding_tier"] != "rules",
        shelfmark_core::Envelope::Err { .. } => true,
    };
    assert!(starter_silent, "the starter docs.rs rule must not fire");
}
