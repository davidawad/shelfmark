use super::*;

fn c(s: &str) -> Class {
    Class::parse(s).unwrap()
}

#[test]
fn parses_codes_to_depths() {
    assert_eq!(c("Q").depth(), 1);
    assert_eq!(c("QA").depth(), 2);
    assert_eq!(c("QA76").depth(), 3);
    assert_eq!(c("QA76.73").depth(), 4);
    assert_eq!(c("QA76.73.C15").depth(), 5);
    assert_eq!(
        c("qh 545.p4").path(),
        vec!["Q", "QH", "QH545", "QH545", "QH545.P4"]
    );
    assert_eq!(c("QA76.60").canonical(), "QA76.6");
}

#[test]
fn parses_ranges_including_bare_end() {
    assert_eq!(
        c("QB630-QB638.8"),
        Class::Range {
            letters: "QB".into(),
            lo: "630".into(),
            hi: "638.8".into()
        }
    );
    assert_eq!(c("JN7401-7695").canonical(), "JN7401-7695");
    assert_eq!(c("K3236-K3268.5").path(), vec!["K", "K", "K3236-3268.5"]);
}

#[test]
fn rejects_bad_codes() {
    for bad in ["", "I12", "ABCD1", "QA76 extra", "QB9-QB1"] {
        assert!(Class::parse(bad).is_err(), "{bad} should not parse");
    }
}

/// The span forms the LCC outline writes (shelfmark `data/schemes/lcc`).
#[test]
fn scheme_spans_across_subclasses_or_cutters() {
    // Across subclasses: only the main class is certain.
    assert_eq!(c("KPG1-KPH4999.9").path(), vec!["K"]);
    assert_eq!(c("QA1-QB2").depth(), 1);
    // Cutters within one number: that number.
    assert_eq!(c("BX840.A1-840.Z").canonical(), "BX840");
    assert_eq!(c("KBM524.4.A-524.4.Z").canonical(), "KBM524.4");
    assert_eq!(c("R126.H4-R126.H8").canonical(), "R126");
    // Cutters across numbers: the number range.
    assert_eq!(c("PM549.A2-2711").canonical(), "PM549-2711");
    assert_eq!(c("BS2279.42-2545.Z").canonical(), "BS2279.42-2545");
    assert_eq!(c("BX840.A1-840.Z").matched_depth(&c("BX840.B5")), 3);
}

#[test]
fn strips_typed_class_ids() {
    assert_eq!(
        Class::parse_id("class:lcc@2024:QA76.76.C65").unwrap(),
        c("QA76.76.C65")
    );
    assert_eq!(Class::parse_id("QA76").unwrap(), c("QA76"));
}

#[test]
fn agreement_by_level() {
    let gold = c("QA76.73.C15");
    assert_eq!(c("QA76.73.C15").matched_depth(&gold), 5);
    assert_eq!(c("QA76.73.P98").matched_depth(&gold), 4);
    assert_eq!(c("QA76.6").matched_depth(&gold), 3);
    assert_eq!(c("QA9").matched_depth(&gold), 2);
    assert_eq!(c("QC174").matched_depth(&gold), 1);
    assert_eq!(c("HB74").matched_depth(&gold), 0);
    // A shallow prediction cannot agree below its own depth.
    assert_eq!(c("QA").matched_depth(&gold), 2);
}

#[test]
fn ranges_match_numbers_inside_them() {
    let gold = c("B400-B491");
    assert_eq!(c("B491.5").matched_depth(&gold), 3);
    assert_eq!(c("B400").matched_depth(&gold), 3);
    assert_eq!(c("B492").matched_depth(&gold), 2);
    assert_eq!(c("BD400").matched_depth(&gold), 1);
    let frac = c("QB630-QB638.8");
    assert!(c("QB638.8").agrees_at(&frac, 3));
    assert!(!c("QB638.9").agrees_at(&frac, 3));
    // Range predictions against code gold, and nested ranges.
    assert!(c("QB630-QB638.8").agrees_at(&c("QB631.2"), 3));
    assert!(c("QB631-QB632").agrees_at(&frac, 3));
}

#[test]
fn extracts_class_numbers_from_call_numbers() {
    let cases = [
        ("QA76.6 .A255 1996", "QA76.6"),
        ("QA76.73.C15 K47 1988", "QA76.73.C15"),
        ("HB74.P8L479 2005", "HB74.P8"),
        ("Sp.Col. NA9108 .J3 c.2", "NA9108"),
        ("QH 545 .P4 C38 1962", "QH545"),
        ("B580", "B580"),
        ("E173  .O94 vol. 6", "E173"),
    ];
    for (call, want) in cases {
        assert_eq!(
            class_of_call_number(call).map(|c| c.canonical()).as_deref(),
            Some(want),
            "{call}"
        );
    }
    assert_eq!(class_of_call_number("QL751"), Some(c("QL751")));
    assert_eq!(class_of_call_number("no class here"), None);
}
