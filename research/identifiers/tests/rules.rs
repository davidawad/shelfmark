use identifier_tables::rules::{arxiv_category_from_id, ArxivMap, DomainRules};

fn data(name: &str) -> String {
    std::fs::read_to_string(format!("{}/data/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn arxiv_map_covers_the_whole_taxonomy_snapshot() {
    let map = ArxivMap::parse(&data("arxiv-lcc.tsv")).unwrap();
    assert!(map
        .missing_from(&data("arxiv-taxonomy.tsv"))
        .unwrap()
        .is_empty());
}

#[test]
fn arxiv_aliases_and_archives_resolve() {
    let map = ArxivMap::parse(&data("arxiv-lcc.tsv")).unwrap();
    assert_eq!(map.resolve("cs.LG").unwrap().lcc, "Q325.5");
    assert_eq!(
        map.resolve("stat.TH").unwrap().lcc,
        map.resolve("math.ST").unwrap().lcc
    );
    assert_eq!(
        map.resolve("solv-int").unwrap().lcc,
        map.resolve("nlin.SI").unwrap().lcc
    );
    // Unknown subject class falls back to its archive.
    assert_eq!(map.resolve("math.ZZ").unwrap().lcc, "QA");
    assert!(map.resolve("nonsense").is_none());
}

#[test]
fn arxiv_old_style_ids_carry_their_category() {
    assert_eq!(arxiv_category_from_id("hep-th/9901001"), Some("hep-th"));
    assert_eq!(
        arxiv_category_from_id("arXiv:math.AG/0101001v2"),
        Some("math.AG")
    );
    assert_eq!(arxiv_category_from_id("2101.00001"), None);
    assert_eq!(arxiv_category_from_id("hep-th/99"), None);
}

#[test]
fn arxiv_map_rejects_bad_rows() {
    let h = "category\tlcc\tfit\talias_of\theading\n";
    assert!(ArxivMap::parse(&format!("{h}cs.X\tqa76\texact\t\tx\n")).is_err());
    assert!(ArxivMap::parse(&format!("{h}cs.X\tQA76\tclose\t\tx\n")).is_err());
    assert!(ArxivMap::parse(&format!("{h}cs.X\t\texact\tcs.Y\tx\n")).is_err());
    assert!(ArxivMap::parse(&format!("{h}cs.X\tQA76\texact\tcs.Y\tx\n")).is_err());
    assert!(ArxivMap::parse(&format!(
        "{h}cs.X\tQA76\texact\t\tx\ncs.X\tQA76\texact\t\tx\n"
    ))
    .is_err());
}

#[test]
fn domain_rules_pick_the_most_specific_match() {
    let rules = DomainRules::parse(&data("url-domain-rules.tsv")).unwrap();
    let r = rules
        .resolve("https://www.ncbi.nlm.nih.gov/pmc/articles/PMC123/")
        .unwrap();
    assert_eq!(
        (r.domain.as_str(), r.path_prefix.as_str()),
        ("ncbi.nlm.nih.gov", "/pmc/")
    );
    assert!(rules
        .resolve("https://www.ncbi.nlm.nih.gov/gene/")
        .is_none());
    let r = rules
        .resolve("https://pubmed.ncbi.nlm.nih.gov/12345/")
        .unwrap();
    assert_eq!(r.domain, "pubmed.ncbi.nlm.nih.gov");
    let r = rules
        .resolve("http://export.arxiv.org/abs/2101.00001")
        .unwrap();
    assert_eq!(r.lcc, "=arxiv");
    assert!(
        rules
            .resolve("https://user@DOCS.RS:443/serde")
            .unwrap()
            .decides
    );
    assert!(rules.resolve("https://notdocs.rs/").is_none());
}

#[test]
fn domain_rules_reject_bad_rows() {
    let h = "domain\tpath_prefix\tlcc\tddc\tstrength\tnote\n";
    assert!(DomainRules::parse(&format!("{h}Example.com\t\tQA76\t\thint\tx\n")).is_err());
    assert!(DomainRules::parse(&format!("{h}example.com\tpmc\tQA76\t\thint\tx\n")).is_err());
    assert!(DomainRules::parse(&format!("{h}example.com\t\tQA76\t5\thint\tx\n")).is_err());
    assert!(DomainRules::parse(&format!("{h}example.com\t\tQA76\t\tmaybe\tx\n")).is_err());
    assert!(DomainRules::parse(&format!(
        "{h}example.com\t\tQA76\t\thint\tx\nexample.com\t\tQA\t\thint\ty\n"
    ))
    .is_err());
}
