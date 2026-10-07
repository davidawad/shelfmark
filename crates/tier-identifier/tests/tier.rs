//! The identifier tier over the fixture table (AE1 shape) and table loading.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use shelfmark_core::cascade;
use shelfmark_core::placement::{Status, TierRunState};
use shelfmark_core::scheme::SchemeDoc;
use shelfmark_core::testkit::{FixedTier, MemIndex};
use shelfmark_core::tier::{ClassifyInput, Evidence, Identifier, Profile, Tier};
use shelfmark_core::{ErrorCode, ProfileId};
use shelfmark_tier_identifier::{IdentifierTier, TableError, Tables, NAME};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn mini_lcc() -> MemIndex {
    let text = std::fs::read_to_string(fixtures().join("schemes/mini-lcc.json")).unwrap();
    MemIndex::new(&serde_json::from_str::<SchemeDoc>(&text).unwrap())
}

fn tier() -> IdentifierTier {
    IdentifierTier::new(Tables::load(&fixtures().join("tables")).unwrap())
}

fn text(t: &str) -> ClassifyInput {
    ClassifyInput {
        text: t.into(),
        ..Default::default()
    }
}

#[test]
fn ae1_isbn_in_text_decides_at_full_confidence_and_cascade_stops() {
    let ids = tier();
    let vocab = FixedTier::new("vocab", &[("H", 0.99)]);
    let profile = Profile {
        id: ProfileId::new("fast", "1"),
        description: "identifier then vocab".into(),
        tiers: vec![NAME.into(), "vocab".into()],
        default_target: 0.9,
        min_margin: 0.1,
        expected_memory_mb: 0,
        expected_micros_per_item: 0,
    };
    let tiers: BTreeMap<String, &dyn Tier> = [
        (NAME.to_string(), &ids as &dyn Tier),
        ("vocab".to_string(), &vocab as &dyn Tier),
    ]
    .into();
    let input = text("Introduction to Algorithms\nISBN 978-0-262-03384-8 (hardcover)");
    let d = cascade::run(&profile, &tiers, &mini_lcc(), &input, None, None).unwrap();

    assert_eq!(d.class.to_string(), "class:mini-lcc@2024:QA76");
    assert_eq!(d.confidence, 1.0);
    assert_eq!(d.status, Status::Machine);
    assert_eq!(d.trace.deciding_tier.as_deref(), Some(NAME));
    assert_eq!(d.trace.tiers[1].state, TierRunState::Skipped);
    assert_eq!(
        d.trace.candidates[0].evidence,
        [Evidence::Identifier {
            identifier: Identifier::Isbn("9780262033848".into()),
            table: "identifiers.jsonl:1".into(),
        }]
    );
}

#[test]
fn each_identifier_type_hits_its_row_mapped_to_the_deepest_covering_class() {
    let index = mini_lcc();
    let cases = [
        ("ISBN 0201633612", "QA76", "identifiers.jsonl:2"),
        ("LCCN 2001-000002", "H", "identifiers.jsonl:4"),
        ("doi:10.1145/359576.359579", "QA76", "identifiers.jsonl:5"),
        ("arXiv:1706.03762v5", "Q", "identifiers.jsonl:6"),
        ("arXiv:hep-th/9711200", "Q", "identifiers.jsonl:7"),
    ];
    for (t, code, row) in cases {
        let out = tier().run(&text(t), &index, &[]).unwrap();
        assert_eq!(out.candidates.len(), 1, "{t}");
        let c = &out.candidates[0];
        assert_eq!((c.class.code.as_str(), c.score), (code, 1.0), "{t}");
        let Evidence::Identifier { table, .. } = &c.evidence[0] else {
            panic!("identifier evidence expected for {t}");
        };
        assert_eq!(table, row, "{t}");
        assert_eq!(out.cost.model_calls, 0);
    }
}

#[test]
fn url_domain_falls_back_to_parent_domain() {
    let input = ClassifyInput {
        url: Some("https://www.law.cornell.edu/uscode/text/17".into()),
        ..Default::default()
    };
    let out = tier().run(&input, &mini_lcc(), &[]).unwrap();
    assert_eq!(out.candidates[0].class.code, "K");
    let sub = ClassifyInput {
        url: Some("https://supct.law.cornell.edu/".into()),
        ..Default::default()
    };
    let out = tier().run(&sub, &mini_lcc(), &[]).unwrap();
    assert_eq!(
        out.candidates[0].evidence[0],
        Evidence::Identifier {
            identifier: Identifier::UrlDomain("law.cornell.edu".into()),
            table: "identifiers.jsonl:8".into(),
        }
    );
}

#[test]
fn unknown_codes_and_unknown_identifiers_give_no_candidate() {
    // PZ7 is outside mini-lcc; 9780131103627 is in no table.
    for t in [
        "ISBN 978-0-306-40615-7",
        "ISBN 978-0-13-110362-7",
        "nothing here",
    ] {
        let out = tier().run(&text(t), &mini_lcc(), &[]).unwrap();
        assert!(out.candidates.is_empty(), "{t}");
    }
}

#[test]
fn two_identifiers_for_one_class_both_returned_for_the_cascade_to_merge() {
    let out = tier()
        .run(
            &text("ISBN 9780262033848; doi:10.1145/359576.359579"),
            &mini_lcc(),
            &[],
        )
        .unwrap();
    let codes: Vec<&str> = out
        .candidates
        .iter()
        .map(|c| c.class.code.as_str())
        .collect();
    assert_eq!(codes, ["QA76", "QA76"]);
}

#[test]
fn info_names_the_tier_and_needs_no_model() {
    let info = tier().info();
    assert_eq!((info.name.as_str(), info.needs_model), (NAME, false));
}

fn load_err(text: &str) -> (usize, String) {
    match Tables::new().load_str("t.jsonl", text) {
        Err(TableError::Line { line, reason, .. }) => (line, reason),
        other => panic!("expected a line error, got {other:?}"),
    }
}

#[test]
fn table_load_errors_name_the_line() {
    let good = r#"{"id":"isbn:9780262033848","class":"QA76.6"}"#;
    let cases = [
        (format!("{good}\nnot json"), 2, "JSON"),
        (format!("{good}\n\n{{\"id\":\"isbn:1\"}}"), 3, "JSON"),
        (
            format!("{good}\n{{\"id\":\"issn:1234-5678\",\"class\":\"Q\"}}"),
            2,
            "issn",
        ),
        (
            format!("{{\"id\":\"isbn:9780262033849\",\"class\":\"Q\"}}\n{good}"),
            1,
            "valid",
        ),
        (
            format!("{good}\n{{\"id\":\"doi:10.1000/x\",\"class\":\" \"}}"),
            2,
            "empty",
        ),
    ];
    for (text, line, word) in cases {
        let (got_line, reason) = load_err(&text);
        assert_eq!(got_line, line, "{text}");
        assert!(reason.contains(word), "{reason:?} should mention {word}");
    }
}

#[test]
fn a_failed_table_adds_nothing_and_maps_to_a_store_error() {
    let mut t = Tables::new();
    let bad = "{\"id\":\"isbn:9780262033848\",\"class\":\"QA76\"}\nnope";
    let err = t.load_str("bad.jsonl", bad).unwrap_err();
    assert!(t.is_empty());
    assert!(err.to_string().starts_with("bad.jsonl:2: "));
    let core: shelfmark_core::Error = err.into();
    assert_eq!(core.code, ErrorCode::StoreError);
    assert_eq!(core.details.unwrap()["line"], 2);
}

#[test]
fn missing_path_is_an_io_error_and_tables_hash_changes_with_content() {
    let missing = fixtures().join("tables/does-not-exist.jsonl");
    assert!(matches!(Tables::load(&missing), Err(TableError::Io { .. })));
    let mut a = Tables::new();
    a.load_str("x.jsonl", r#"{"id":"doi:10.1000/a","class":"Q"}"#)
        .unwrap();
    let mut b = Tables::new();
    b.load_str("x.jsonl", r#"{"id":"doi:10.1000/b","class":"Q"}"#)
        .unwrap();
    assert_ne!(a.content_hash(), b.content_hash());
    assert_eq!(a.content_hash(), a.clone().content_hash());
}

#[test]
fn single_file_and_directory_loads_agree() {
    let file = Tables::load(&fixtures().join("tables/identifiers.jsonl")).unwrap();
    let dir = tier();
    assert_eq!(file.len(), dir.tables().len());
    assert_eq!(file.content_hash(), dir.tables().content_hash());
    assert_eq!(file.len(), 8);
}
