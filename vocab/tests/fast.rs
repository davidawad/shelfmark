//! FAST ingest over real id.worldcat.org records (ODC-By, see README),
//! joined to LCC through the committed LCSH table sample.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

use vocab_ingest::{fast, rdfxml, table};

fn ingest(lcsh: Option<&HashMap<String, Vec<String>>>) -> (Vec<table::Heading>, fast::FastStats) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("sample/fast-records");
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    let mut b = fast::Builder::default();
    for f in files {
        for t in rdfxml::read_triples(BufReader::new(File::open(f).unwrap())).unwrap() {
            b.add(&t);
        }
    }
    b.finish(lcsh)
}

#[test]
fn reads_every_sample_record() {
    let (rows, stats) = ingest(None);
    assert_eq!(stats.headings, 31);
    assert_eq!(stats.subjects_without_label, 0);
    assert!(rows
        .iter()
        .all(|h| h.kind == "Topical" && h.id.starts_with("fst")));
    let botany = rows.iter().find(|h| h.id == "fst00836869").unwrap();
    assert_eq!(botany.pref_label, "Botany");
    assert!(botany.alt_labels.contains(&"Plant science".to_string()));
    assert_eq!(botany.same_as, ["sh85015976"]);
    // Without an LCSH table nothing gets an LCC number.
    assert!(rows
        .iter()
        .all(|h| h.lcc.is_empty() && h.lcc_source.is_empty()));
}

#[test]
fn joins_lcc_through_lcsh_links() {
    let lcsh = HashMap::from([
        ("sh85015976".to_string(), vec!["QK".to_string()]),
        ("sh85100849".to_string(), vec!["B-BJ".to_string()]),
    ]);
    let (rows, stats) = ingest(Some(&lcsh));
    let lcc: Vec<_> = rows
        .iter()
        .filter(|h| !h.lcc.is_empty())
        .map(|h| {
            (
                h.pref_label.as_str(),
                h.lcc.join("|"),
                h.lcc_source.as_str(),
            )
        })
        .collect();
    assert_eq!(
        lcc,
        [
            ("Botany", "QK".into(), "lcsh"),
            ("Philosophy", "B-BJ".into(), "lcsh")
        ]
    );
    assert_eq!(stats.lcsh_links_resolved, 2);
    assert_eq!(stats.with_lcsh_link, 29);
}
