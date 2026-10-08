//! End-to-end: the committed sample builds to the committed golden output,
//! regardless of thread count.

use identifier_tables::{build, output};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn build_into(dir: &Path, jobs: usize) {
    let opts = build::Options {
        jobs,
        limit: None,
        progress_every: 0,
    };
    let mut table = build::build(&root().join("samples/editions-sample.txt"), &opts).unwrap();
    output::write_all(&mut table, dir).unwrap();
}

fn tmpdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("identifier-tables-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn sample_matches_golden_output_at_any_thread_count() {
    for jobs in [1, 3, 8] {
        let out = tmpdir(&format!("jobs{jobs}"));
        build_into(&out, jobs);
        for file in ["lookup.tsv", "conflicts.tsv", "stats.json"] {
            let got = std::fs::read_to_string(out.join(file)).unwrap();
            let want = std::fs::read_to_string(root().join("samples/expected").join(file)).unwrap();
            assert!(
                got == want,
                "{file} differs from samples/expected with --jobs {jobs}"
            );
        }
        std::fs::remove_dir_all(&out).unwrap();
    }
}

#[test]
fn lookup_is_sorted_and_names_isbn_with_lc_call_number() {
    let lookup = std::fs::read_to_string(root().join("samples/expected/lookup.tsv")).unwrap();
    let mut lines = lookup.lines();
    assert_eq!(lines.next(), Some(output::LOOKUP_HEADER));
    let rows: Vec<&str> = lines.collect();
    let keys: Vec<&str> = rows.iter().map(|r| r.split('\t').next().unwrap()).collect();
    assert!(
        keys.windows(2).all(|w| w[0] <= w[1]),
        "lookup.tsv keys must be sorted"
    );
    // AE1 needs an ISBN that resolves to a full LC call number.
    assert!(rows.contains(&"isbn:9780002721196\tlcc\tDA632\tDA632.R33 1986\t1"));
}

#[test]
fn merged_library_disagreement_is_a_single_edition_conflict() {
    let conflicts = std::fs::read_to_string(root().join("samples/expected/conflicts.tsv")).unwrap();
    assert!(conflicts
        .lines()
        .any(|l| l == "isbn:9780002312110\tlcc\ttop\t1\tPR=1;PZ=1"));
}

#[test]
fn non_edition_and_malformed_lines_are_counted_not_fatal() {
    let dir = tmpdir("malformed");
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("in.txt");
    std::fs::write(
        &input,
        "/type/author\t/authors/OL1A\t1\t2020-01-01\t{}\n/type/edition\t/books/OL1M\t1\t2020-01-01\t{not json\n\
         /type/edition\t/books/OL2M\t1\t2020-01-01\t{\"isbn_10\":[\"0521413826\"],\"lc_classifications\":[\"QA76.76.C65 S5\"]}\n",
    )
    .unwrap();
    let opts = build::Options {
        jobs: 2,
        limit: None,
        progress_every: 0,
    };
    let table = build::build(&input, &opts).unwrap();
    assert_eq!(
        (
            table.stats.dump_lines,
            table.stats.edition_records,
            table.stats.parse_errors
        ),
        (3, 1, 1)
    );
    assert_eq!(table.entries.len(), 1);
    std::fs::remove_dir_all(&dir).unwrap();
}
