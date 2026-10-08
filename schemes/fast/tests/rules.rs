//! The build rules on `tests/data/mini.nt`, a hand-written slice with one
//! case per rule: parents from broader, from dropped subdivisions, a
//! refused cycle, deprecated headings as vocabulary, LCSH links, and the
//! links and facets the build drops.

use std::path::PathBuf;

use fast_scheme::build::{self, Stats};
use fast_scheme::graph::Graph;
use fast_scheme::report;
use shelfmark_core::scheme::{ClassDef, SchemeDoc, VocabLink};

fn mini() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/mini.nt");
    std::fs::read_to_string(p).unwrap()
}

fn built(text: &str) -> (SchemeDoc, Stats) {
    let g = Graph::read(text.as_bytes()).unwrap();
    build::build(&g, "test")
}

fn class<'a>(doc: &'a SchemeDoc, code: &str) -> &'a ClassDef {
    doc.classes.iter().find(|c| c.code == code).unwrap()
}

fn parent(doc: &SchemeDoc, code: &str) -> Option<String> {
    class(doc, code).parent.clone()
}

fn link(term: &str, source: &str, id: Option<&str>, weight: f64) -> VocabLink {
    VocabLink {
        term: term.into(),
        source: source.into(),
        source_id: id.map(String::from),
        weight,
    }
}

#[test]
fn every_live_labelled_heading_is_a_class_coded_fst_and_eight_digits() {
    let (doc, _) = built(&mini());
    let codes: Vec<&str> = doc.classes.iter().map(|c| c.code.as_str()).collect();
    assert_eq!(
        codes,
        [
            "fst00000100",
            "fst00000101",
            "fst00000102",
            "fst00000103",
            "fst00000200",
            "fst00000300",
            "fst00000400",
            "fst00000401",
            "fst00000600"
        ]
    );
    assert_eq!(doc.id().to_string(), "scheme:fast@2024");
    assert_eq!(class(&doc, "fst00000600").label, "Cafés \"and\" bistros");
}

#[test]
fn parents_follow_broader_then_dropped_subdivisions() {
    let (doc, _) = built(&mini());
    // Lowest-numbered live broader; the other-facet 9999 is skipped.
    assert_eq!(parent(&doc, "fst00000200").as_deref(), Some("fst00000100"));
    assert_eq!(parent(&doc, "fst00000101").as_deref(), Some("fst00000100"));
    assert_eq!(parent(&doc, "fst00000102").as_deref(), Some("fst00000101"));
    // "Computers--Design and construction" is no heading: skip to Computers.
    assert_eq!(parent(&doc, "fst00000103").as_deref(), Some("fst00000100"));
    assert_eq!(parent(&doc, "fst00000100"), None);
}

#[test]
fn a_broader_link_that_would_cycle_is_refused() {
    let (doc, s) = built(&mini());
    assert_eq!(parent(&doc, "fst00000400").as_deref(), Some("fst00000401"));
    assert_eq!(parent(&doc, "fst00000401"), None);
    assert_eq!(s.broader_cycles, 1);
}

#[test]
fn vocabulary_is_alternates_lcsh_links_and_replaced_headings() {
    let (doc, _) = built(&mini());
    assert_eq!(
        class(&doc, "fst00000100").vocab,
        [
            link("Calculating machines", "fast", Some("fst00000500"), 0.5),
            link("Electronic computers", "fast", None, 1.0),
            link("Computers", "lcsh", Some("sh85029552"), 1.0),
        ]
    );
    // No LCSH label in the dump: the FAST label stands in.
    assert_eq!(
        class(&doc, "fst00000300").vocab,
        [
            link("Calculating machines", "fast", Some("fst00000500"), 0.5),
            link(
                "Electronic digital computers",
                "lcsh",
                Some("sh00000001"),
                1.0
            ),
        ]
    );
    assert!(class(&doc, "fst00000600").vocab.is_empty());
}

#[test]
fn stats_count_what_was_kept_and_dropped() {
    let (_, s) = built(&mini());
    let want = Stats {
        headings: 10,
        deprecated: 1,
        unlabelled: 1,
        classes: 9,
        parent_broader: 2,
        parent_subdivision: 3,
        top_level: 4,
        multiple_broader: 1,
        broader_cycles: 1,
        broader_dangling: 1,
        max_depth: 2,
        alt_terms: 1,
        lcsh_links: 2,
        lcsh_unlabelled: 1,
        replaced_terms: 2,
    };
    assert_eq!(s, want);
}

#[test]
fn input_order_does_not_change_the_output() {
    let text = mini();
    let mut lines: Vec<&str> = text.lines().collect();
    lines.reverse();
    let (a, _) = built(&text);
    let (b, _) = built(&lines.join("\n"));
    assert_eq!(report::scheme_json(&a), report::scheme_json(&b));
}

#[test]
fn the_built_scheme_passes_the_scheme_compiler() {
    let (doc, _) = built(&mini());
    let json = report::scheme_json(&doc);
    let index = shelfmark_scheme::compile_bytes(json.as_bytes(), "mini").unwrap();
    use shelfmark_core::scheme::CompiledIndex;
    assert_eq!(index.len(), 9);
    let up: Vec<String> = index
        .ancestors("fst00000102")
        .into_iter()
        .map(|r| r.id.code)
        .collect();
    assert_eq!(up, ["fst00000102", "fst00000101", "fst00000100"]);
}
