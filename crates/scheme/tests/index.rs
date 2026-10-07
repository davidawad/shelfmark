//! Every `CompiledIndex` method against the mini-lcc fixture (R5, R6).

use std::path::PathBuf;

use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};
use shelfmark_core::testkit::MemIndex;
use shelfmark_scheme::{compile_file, content_hash, SchemeIndex};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/schemes/mini-lcc.json")
}

fn index() -> SchemeIndex {
    compile_file(&fixture()).expect("mini-lcc compiles")
}

fn codes(records: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    records.into_iter().map(|r| r.id.code).collect()
}

#[test]
fn identity_and_size() {
    let ix = index();
    assert_eq!(ix.scheme().to_string(), "scheme:mini-lcc@2024");
    let bytes = std::fs::read(fixture()).unwrap();
    assert_eq!(ix.content_hash(), content_hash(&bytes));
    assert!(ix.content_hash().starts_with("s256:") && ix.content_hash().len() == 5 + 64);
    assert_eq!(ix.len(), 5);
    assert!(!ix.is_empty());
}

#[test]
fn get_returns_full_records() {
    let ix = index();
    let qa76 = ix.get("QA76").unwrap();
    assert_eq!(qa76.id.to_string(), "class:mini-lcc@2024:QA76");
    assert_eq!(qa76.label, "Computer software");
    assert_eq!(qa76.parent.unwrap().code, "QA");
    assert_eq!(qa76.level, 2);
    let range = qa76.range.unwrap();
    assert_eq!(
        (range.start.as_str(), range.end.as_str()),
        ("QA76", "QA76.95")
    );
    assert_eq!(ix.get("H").unwrap().level, 0);
    assert!(ix.get("H").unwrap().parent.is_none());
    assert!(ix.get("QA77").is_none());
    assert!(ix.get("qa76").is_none(), "lookup is exact");
}

#[test]
fn agrees_with_the_testkit_reference_on_lookup() {
    let doc: SchemeDoc = serde_json::from_slice(&std::fs::read(fixture()).unwrap()).unwrap();
    let (ix, mem) = (index(), MemIndex::new(&doc));
    for c in &doc.classes {
        assert_eq!(ix.get(&c.code), mem.get(&c.code), "{}", c.code);
        assert_eq!(ix.ancestors(&c.code), mem.ancestors(&c.code), "{}", c.code);
    }
    assert_eq!(ix.vocab(), mem.vocab());
}

#[test]
fn children_in_shelf_order() {
    let ix = index();
    assert_eq!(codes(ix.children(None)), ["H", "K", "Q"]);
    assert_eq!(codes(ix.children(Some("Q"))), ["QA"]);
    assert_eq!(codes(ix.children(Some("QA"))), ["QA76"]);
    assert!(ix.children(Some("QA76")).is_empty());
    assert!(ix.children(Some("ZZ")).is_empty());
}

#[test]
fn ancestors_from_self_to_top() {
    let ix = index();
    assert_eq!(codes(ix.ancestors("QA76")), ["QA76", "QA", "Q"]);
    assert_eq!(codes(ix.ancestors("H")), ["H"]);
    assert!(
        ix.ancestors("QA76.76").is_empty(),
        "only classes have ancestors"
    );
}

#[test]
fn check_code_covers_all_four_outcomes() {
    let ix = index();
    let within = |code: &str| match ix.check_code(code) {
        CodeCheck::Within { class } => class.code,
        other => panic!("{code}: {other:?}"),
    };
    assert!(matches!(ix.check_code("QA76"), CodeCheck::Exact { class } if class.code == "QA76"));
    assert_eq!(within("QA76.76"), "QA76");
    assert_eq!(within("QA76.76.C65"), "QA76");
    assert_eq!(within("QA76.95"), "QA76");
    assert_eq!(
        within("QA76.95.C3"),
        "QA76",
        "everything under the range end"
    );
    assert_eq!(within("QA77"), "QA", "outside QA76's range, still under QA");
    assert_eq!(within("QB5"), "Q", "letter subclass under Q");
    assert_eq!(within("HB71"), "H");
    assert_eq!(ix.check_code("Z696"), CodeCheck::Unplaced);
    for bad in ["qa76", "QA 76", "", "QAAA1", "76", "QA76."] {
        assert!(
            matches!(ix.check_code(bad), CodeCheck::Invalid { ref reason } if reason.contains("code_grammar")),
            "{bad:?}"
        );
    }
}

#[test]
fn search_labels_and_vocabulary() {
    let ix = index();
    let hits = ix.search("compilers", 10);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].class.code, "QA76");
    assert_eq!(hits[0].matched, "Compilers (Computer programs)");
    // 0.9 weight * Dice(1 shared; 1 query + 3 term tokens) = 0.45.
    assert!((hits[0].score - 0.45).abs() < 1e-12, "{}", hits[0].score);

    let hits = ix.search("Computer programs", 10);
    assert_eq!(hits[0].matched, "Computer programs");
    assert!((hits[0].score - 1.0).abs() < 1e-12);

    let hits = ix.search("SCIENCE", 10);
    assert_eq!(hits[0].class.code, "Q");
    assert_eq!(hits[0].matched, "Science");
    let hits = ix.search("social law", 10);
    assert_eq!(
        hits.iter()
            .map(|h| h.class.code.as_str())
            .collect::<Vec<_>>(),
        ["K", "H"],
        "Law (1 of 1 tokens) beats Social sciences (1 of 2)"
    );
    assert_eq!(ix.search("social law", 1).len(), 1);
    assert!(ix.search("astronomy", 10).is_empty());
    assert!(ix.search("  ", 10).is_empty());
    assert!(ix.search("law", 0).is_empty());
}

#[test]
fn search_ties_break_by_code() {
    let json = br#"{"name":"t","version":"1","title":"T","code_grammar":"[A-Z]+",
        "classes":[{"code":"B","label":"Rivers"},{"code":"A","label":"Rivers"}]}"#;
    let ix = shelfmark_scheme::compile_bytes(json, "t.json").unwrap();
    let hits = ix.search("rivers", 5);
    assert_eq!(
        hits.iter()
            .map(|h| h.class.code.as_str())
            .collect::<Vec<_>>(),
        ["A", "B"]
    );
}
