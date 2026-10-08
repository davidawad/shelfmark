//! LCSH ingest over real record blocks from the id.loc.gov MADS dump.

use std::fs::File;
use std::io::BufReader;

use vocab_ingest::{lcsh, table::Heading};

fn ingest() -> lcsh::Ingest {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/sample/lcsh-records.nt");
    lcsh::ingest(BufReader::new(File::open(path).unwrap())).unwrap()
}

fn row<'a>(rows: &'a [Heading], id: &str) -> &'a Heading {
    rows.iter()
        .find(|h| h.id == id)
        .unwrap_or_else(|| panic!("{id} missing"))
}

#[test]
fn reads_label_variants_broader_and_lcc() {
    let got = ingest();
    let h = row(&got.headings, "sh86007588");
    assert_eq!(h.pref_label, "Compilers (Computer programs)");
    assert_eq!(h.kind, "Topic");
    assert_eq!(h.alt_labels, ["Compiling programs (Computer programs)"]);
    assert_eq!(h.broader, ["sh2006006028", "sh85029524", "sh89000200"]);
    assert_eq!(h.lcc, ["QA76.76.C65"]);
    assert_eq!(h.lcc_source, "direct");
    assert!(!h.deprecated);
}

#[test]
fn classification_iri_form_counts_as_lcc() {
    let got = ingest();
    assert_eq!(row(&got.headings, "sh00000029").lcc, ["QL737.P959"]);
    assert_eq!(row(&got.headings, "sh85082139").lcc, ["QA"]);
    // A heading with no classification has no LCC and no source.
    let h = row(&got.headings, "sh85029524");
    assert!(h.lcc.is_empty() && h.lcc_source.is_empty());
}

#[test]
fn deprecated_records_keep_label_and_replacements() {
    let got = ingest();
    let h = row(&got.headings, "sh00000273");
    assert!(h.deprecated);
    assert_eq!(h.pref_label, "Child concentration camp inmates");
    assert_eq!(h.same_as, ["sh2021004026", "sh2021004027"]);
}

#[test]
fn output_is_sorted_and_stats_add_up() {
    let got = ingest();
    let ids: Vec<_> = got.headings.iter().map(|h| h.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
    assert_eq!(got.stats.records, 7);
    assert_eq!(got.stats.headings, 7);
    assert_eq!(got.stats.parse_errors, 0);
    let with_lcc = got.headings.iter().filter(|h| !h.lcc.is_empty()).count() as u64;
    assert_eq!(got.stats.with_lcc, with_lcc);
    assert!(got.stats.lcc_ranges >= 1, "sh00000490 carries a range");
}
