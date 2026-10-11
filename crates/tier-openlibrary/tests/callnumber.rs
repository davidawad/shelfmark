//! Call number parsing and mapping into the LCC outline.

mod common;

use common::lcc;
use shelfmark_tier_openlibrary::callnumber::{map_call_number, CallNumber, Mapper};

fn parsed(raw: &str) -> (String, Option<String>) {
    let c = CallNumber::parse(raw).unwrap();
    (c.class_number(), c.cutter)
}

#[test]
fn parses_raw_call_numbers() {
    assert_eq!(
        parsed("PR6039.O32 H6 1988"),
        ("PR6039".into(), Some("O32".into()))
    );
    assert_eq!(
        parsed("QA76.73.P98 L88 2013"),
        ("QA76.73".into(), Some("P98".into()))
    );
    assert_eq!(
        parsed("HB171 .S65 1990"),
        ("HB171".into(), Some("S65".into()))
    );
    assert_eq!(parsed("qa76.6"), ("QA76.6".into(), None));
    assert_eq!(parsed("PZ7"), ("PZ7".into(), None));
}

#[test]
fn parses_the_padded_sort_form() {
    assert_eq!(
        parsed("PR-6039.00000000.O32 H6 1988"),
        ("PR6039".into(), Some("O32".into()))
    );
    assert_eq!(
        parsed("QA-0076.60000000.A255 1996"),
        ("QA76.6".into(), Some("A255".into()))
    );
    assert_eq!(
        parsed("PZ-0008.00000000.T52Ho13"),
        ("PZ8".into(), Some("T52".into()))
    );
    assert_eq!(
        parsed("PR-6039.00000000 O33 H6"),
        ("PR6039".into(), Some("O33".into()))
    );
    assert_eq!(parsed("PZ-0007.00000000"), ("PZ7".into(), None));
    // One-letter classes are padded with a second dash.
    assert_eq!(
        parsed("Q--0175.00000000.K95 1996"),
        ("Q175".into(), Some("K95".into()))
    );
    assert_eq!(
        parsed("B--3313.00000000 Z23E6 1908"),
        ("B3313".into(), Some("Z23".into()))
    );
}

#[test]
fn rejects_what_is_not_lc() {
    for raw in ["823.912", "[Fic]", "", "   ", "ABCD12", "Fic"] {
        assert_eq!(CallNumber::parse(raw), None, "{raw:?}");
    }
}

#[test]
fn codes_go_from_specific_to_general() {
    let c = CallNumber::parse("QA76.73.P98 L88").unwrap();
    assert_eq!(c.codes(), ["QA76.73.P98", "QA76.73", "QA76", "QA"]);
    let c = CallNumber::parse("HB171").unwrap();
    assert_eq!(c.codes(), ["HB171", "HB"]);
}

#[test]
fn maps_to_the_deepest_covering_outline_class() {
    let code = |raw: &str| map_call_number(lcc(), raw).map(|c| c.code);
    assert_eq!(code("PR6039.O32 H6 1988").as_deref(), Some("PR6000-6049"));
    assert_eq!(
        code("PR-6039.00000000.O32 H6 1988").as_deref(),
        Some("PR6000-6049")
    );
    assert_eq!(code("PZ7.T5744 Ho 1987").as_deref(), Some("PZ5-90"));
    assert_eq!(code("QA76.6 .A255 1996").as_deref(), Some("QA75.5-76.95"));
    assert_eq!(code("823.912"), None);
    // Letters the scheme does not have.
    assert_eq!(code("XZ100"), None);
}

#[test]
fn mapper_is_scheme_bound_and_repeatable() {
    let mut m = Mapper::new(lcc());
    let a = m.map("PS3558.E63 D8 1965");
    let b = m.map("PS-3558.00000000.E63 D86 1990");
    assert!(a.is_some());
    assert_eq!(a, b, "two editions of one book land together");
    assert_eq!(a.unwrap().scheme.to_string(), "scheme:lcc@2024");
}
