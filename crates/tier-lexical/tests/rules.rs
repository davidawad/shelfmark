//! The `rules` tier (plan U4; R7).

mod common;

use common::{mini_lcc, text};
use shelfmark_core::tier::{Candidate, ClassifyInput, Evidence, Identifier, Tier};
use shelfmark_core::ErrorCode;
use shelfmark_tier_lexical::{Rule, RuleWhen, RulesConfig, RulesTier};

fn tier(json: &str) -> RulesTier {
    RulesTier::from_json(json).unwrap()
}

fn run(t: &RulesTier, input: &ClassifyInput) -> Vec<Candidate> {
    t.run(input, &mini_lcc(), &[]).unwrap().candidates
}

fn fired(c: &[Candidate]) -> Vec<(&str, f64, Vec<&str>)> {
    c.iter()
        .map(|c| {
            let rules = c
                .evidence
                .iter()
                .map(|e| match e {
                    Evidence::Rule { rule } => rule.as_str(),
                    other => panic!("unexpected evidence {other:?}"),
                })
                .collect();
            (c.class.code.as_str(), c.score, rules)
        })
        .collect()
}

fn url(u: &str) -> ClassifyInput {
    ClassifyInput {
        url: Some(u.into()),
        ..ClassifyInput::default()
    }
}

const DOMAIN: &str = r#"{"rules": [
  {"id": "arxiv", "when": {"domain": "www.arXiv.org"}, "class": "QA76", "weight": 0.7}
]}"#;

#[test]
fn domain_rule_matches_the_host_and_its_subdomains_only() {
    let t = tier(DOMAIN);
    for u in [
        "https://arxiv.org/abs/2401.00001",
        "http://user@export.arxiv.org:8080/x?y#z",
        "arxiv.org",
        "https://www.ARXIV.org/",
    ] {
        assert_eq!(
            fired(&run(&t, &url(u))),
            [("QA76", 0.7, vec!["arxiv: domain:www.arXiv.org"])],
            "{u}"
        );
    }
    for u in [
        "https://notarxiv.org/",
        "https://arxiv.org.evil.com/",
        "https://example.com/arxiv.org",
    ] {
        assert!(run(&t, &url(u)).is_empty(), "{u}");
    }
    let by_identifier = ClassifyInput {
        identifiers: vec![Identifier::UrlDomain("arxiv.org".into())],
        ..ClassifyInput::default()
    };
    assert_eq!(run(&t, &by_identifier).len(), 1);
}

#[test]
fn term_rule_matches_whole_normalized_words_in_any_text_field() {
    let t =
        tier(r#"{"rules": [{"when": {"term": "Supreme Court"}, "class": "K", "weight": 0.9}]}"#);
    assert_eq!(
        fired(&run(&t, &text("Before the SUPREME-court, today."))),
        [("K", 0.9, vec!["rule-1: term:Supreme Court"])]
    );
    let heading = ClassifyInput {
        headings: vec!["United States. Supreme Court".into()],
        ..ClassifyInput::default()
    };
    assert_eq!(run(&t, &heading).len(), 1);
    assert!(run(&t, &text("supreme courtship")).is_empty());
    assert!(run(&t, &text("supreme")).is_empty());
}

#[test]
fn regex_rule_runs_on_raw_text_title_and_url() {
    let t = tier(
        r#"{"rules": [{"id": "rfc", "when": {"regex": "\\bRFC ?\\d{3,4}\\b"}, "class": "QA76", "weight": 0.8}]}"#,
    );
    assert_eq!(run(&t, &text("See RFC 2616 for details")).len(), 1);
    let titled = ClassifyInput {
        title: Some("RFC9110".into()),
        ..ClassifyInput::default()
    };
    assert_eq!(run(&t, &titled).len(), 1);
    assert_eq!(
        run(&t, &url("https://www.rfc-editor.org/RFC 7231")).len(),
        1
    );
    assert!(
        run(&t, &text("rfc 2616")).is_empty(),
        "regex is case sensitive unless it says (?i)"
    );
}

const PRECEDENCE: &str = r#"{"rules": [
  {"id": "law-low", "when": {"term": "court"}, "class": "K", "weight": 0.4},
  {"id": "social", "when": {"term": "court"}, "class": "H", "weight": 0.6},
  {"id": "law-high", "when": {"regex": "(?i)court"}, "class": "K", "weight": 0.9},
  {"id": "math", "when": {"term": "court"}, "class": "QA", "weight": 0.6}
]}"#;

#[test]
fn precedence_is_by_weight_then_class_code_with_every_fired_rule_as_evidence() {
    let out = run(&tier(PRECEDENCE), &text("The court ruled."));
    assert_eq!(
        fired(&out),
        [
            (
                "K",
                0.9,
                vec!["law-low: term:court", "law-high: regex:(?i)court"]
            ),
            ("H", 0.6, vec!["social: term:court"]),
            ("QA", 0.6, vec!["math: term:court"]),
        ]
    );
}

#[test]
fn rules_that_do_not_fire_give_no_candidates() {
    assert!(run(&tier(PRECEDENCE), &text("nothing relevant")).is_empty());
    assert!(run(&tier(r#"{"rules": []}"#), &text("court")).is_empty());
}

#[test]
fn identical_output_across_runs_and_scores_in_unit_interval() {
    let t = tier(PRECEDENCE);
    let input = ClassifyInput {
        title: Some("Court".into()),
        text: "court court".into(),
        url: Some("https://court.example/".into()),
        ..ClassifyInput::default()
    };
    let first = serde_json::to_string(&run(&t, &input)).unwrap();
    for _ in 0..20 {
        let again = run(&tier(PRECEDENCE), &input);
        assert!(again
            .iter()
            .all(|c| c.score.is_finite() && (0.0..=1.0).contains(&c.score)));
        assert_eq!(serde_json::to_string(&again).unwrap(), first);
    }
}

#[test]
fn bad_configs_are_invalid_requests() {
    for json in [
        r#"{"rules": [{"when": {"term": "x"}, "class": "K", "weight": 0}]}"#,
        r#"{"rules": [{"when": {"term": "x"}, "class": "K", "weight": 1.5}]}"#,
        r#"{"rules": [{"when": {"term": "x"}, "class": " ", "weight": 1}]}"#,
        r#"{"rules": [{"when": {"term": "!!"}, "class": "K", "weight": 1}]}"#,
        r#"{"rules": [{"when": {"domain": " "}, "class": "K", "weight": 1}]}"#,
        r#"{"rules": [{"when": {"regex": "("}, "class": "K", "weight": 1}]}"#,
        r#"{"rules": [{"when": {"color": "red"}, "class": "K", "weight": 1}]}"#,
        r#"{"rules": [{"when": {"term": "x"}, "class": "K", "weight": 1, "extra": 1}]}"#,
        r#"{"rules": [{"id": "a", "when": {"term": "x"}, "class": "K", "weight": 1},
                      {"id": "a", "when": {"term": "y"}, "class": "H", "weight": 1}]}"#,
        "not json",
    ] {
        let err = RulesTier::from_json(json).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidRequest, "{json}");
    }
}

#[test]
fn a_rule_naming_a_class_outside_the_scheme_is_invalid_code() {
    let t = tier(r#"{"rules": [{"when": {"term": "court"}, "class": "ZZ9", "weight": 1}]}"#);
    let err = t.run(&text("court"), &mini_lcc(), &[]).unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidCode);
    assert!(err.next_command.unwrap().contains("ZZ9"));
    let deeper =
        tier(r#"{"rules": [{"when": {"term": "court"}, "class": "QA76.76.C65", "weight": 1}]}"#);
    assert_eq!(run(&deeper, &text("court"))[0].class.code, "QA76.76.C65");
}

#[test]
fn loads_from_a_file_and_versions_by_content() {
    let dir = std::env::temp_dir().join(format!("shelfmark-rules-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("rules.json");
    std::fs::write(&path, PRECEDENCE).unwrap();
    let t = RulesTier::from_path(&path).unwrap();
    assert_eq!(t.len(), 4);
    let info = t.info();
    assert_eq!(info.name, "rules");
    assert!(!info.needs_model);
    assert_eq!(info.version, tier(PRECEDENCE).info().version);
    assert_ne!(info.version, tier(DOMAIN).info().version);
    assert!(info.version.starts_with("rules-1+"));

    std::fs::write(&path, "{").unwrap();
    let err = RulesTier::from_path(&path).unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidRequest);
    assert!(err.message.contains("rules.json"));
    std::fs::remove_dir_all(&dir).unwrap();

    let err = RulesTier::from_path(&dir.join("missing.json")).unwrap_err();
    assert_eq!(err.code, ErrorCode::InputUnreadable);
}

#[test]
fn config_round_trips_through_its_json_shape() {
    let config = RulesConfig {
        rules: vec![Rule {
            id: None,
            when: RuleWhen::Domain("arxiv.org".into()),
            class: "QA76".into(),
            weight: 0.5,
        }],
    };
    let json = serde_json::to_string(&config).unwrap();
    assert_eq!(
        json,
        r#"{"rules":[{"when":{"domain":"arxiv.org"},"class":"QA76","weight":0.5}]}"#
    );
    assert_eq!(serde_json::from_str::<RulesConfig>(&json).unwrap(), config);
    assert!(RulesTier::from_config(&config).unwrap().len() == 1);
}
