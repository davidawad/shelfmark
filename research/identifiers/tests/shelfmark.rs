//! The shelfmark exports: the shipped starter rules are exactly what the
//! rule files generate, and a built table converts to unambiguous rows.

use identifier_tables::rules::{ArxivMap, DomainRules};
use identifier_tables::shelfmark;

fn read(rel: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)).unwrap()
}

#[test]
fn shipped_starter_rules_match_the_rule_files() {
    let map = ArxivMap::parse(&read("data/arxiv-lcc.tsv")).unwrap();
    let domains = DomainRules::parse(&read("data/url-domain-rules.tsv")).unwrap();
    let doc = shelfmark::rules(&map, &domains);
    let want = serde_json::to_string_pretty(&doc).unwrap() + "\n";
    assert_eq!(
        read("../../data/rules/starter.json"),
        want,
        "regenerate: cargo run -- shelfmark-rules"
    );
    // `=arxiv` is not a rule; every category and every other domain is.
    let rules = doc["rules"].as_array().unwrap();
    assert_eq!(rules.len(), map.0.len() + domains.0.len() - 1);
}

#[test]
fn table_keeps_the_best_supported_class_and_drops_ambiguous_keys() {
    let lookup = "key\tscheme\tprefix\tfull\tsupport\n\
        isbn:1\tlcc\tQA76\tQA76 .A1\t1\n\
        isbn:1\tlcc\tQA76.6\tQA76.6 .B2\t3\n\
        isbn:1\tddc\t005\t005.1\t9\n\
        isbn:2\tlcc\tPR6060\tPR6060.O22\t1\n\
        isbn:2\tlcc\tPZ4\tPZ4.J63\t1\n\
        lccn:3\tlcc\tHB171\tHB171\t1\n";
    let conflicts = "key\tscheme\tlevel\teditions\tcandidates\n\
        isbn:2\tlcc\ttop\t1\tPR6060=1;PZ4=1\n";
    let (jsonl, dropped) = shelfmark::table(lookup, conflicts).unwrap();
    assert_eq!(dropped, 1);
    assert_eq!(
        jsonl,
        "{\"class\":\"QA76.6\",\"id\":\"isbn:1\"}\n{\"class\":\"HB171\",\"id\":\"lccn:3\"}\n"
    );
}
