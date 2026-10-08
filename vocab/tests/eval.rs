//! The fixture file parses and the eval runs end to end on the committed
//! LCSH sample. Accuracy numbers come from the full table (see README).

use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

use vocab_ingest::{eval, matcher::Matcher, table};

#[test]
fn fixtures_parse_and_eval_runs_on_sample() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = fs::read_to_string(root.join("fixtures/lcc-fixtures.jsonl")).unwrap();
    let fixtures: Vec<eval::Fixture> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(fixtures.len(), 38);
    assert!(fixtures.iter().all(|f| !f.expect_class.is_empty()));

    let rows = table::read_table(BufReader::new(
        File::open(root.join("sample/lcsh-sample.tsv")).unwrap(),
    ))
    .unwrap();
    let m = Matcher::build(rows.clone()).unwrap();
    let rep = eval::run_fixtures(&m, &fixtures);
    assert_eq!(rep.cases, 38);
    assert_eq!(rep.results.len(), 38);
    let sc = eval::self_check(&m, &rows, 50);
    assert!(sc.sampled > 0);
    assert_eq!(sc.heading_recovered, sc.sampled, "misses: {:?}", sc.misses);
}
