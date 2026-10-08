//! `vocab-ingest` command line.
//!
//! ```text
//! vocab-ingest lcsh   --input subjects.madsrdf.nt.gz --out lcsh.tsv.gz --stats lcsh-stats.json
//! vocab-ingest fast   (--nt FASTTopical.nt | --rdfxml DIR) [--lcsh lcsh.tsv.gz] --out fast.tsv.gz --stats fast-stats.json
//! vocab-ingest sample --table lcsh.tsv.gz --out sample.tsv [--every N] [--ids a,b]
//! vocab-ingest match  --table lcsh.tsv.gz [--table fast.tsv.gz] --text "..."
//! vocab-ingest eval   --table lcsh.tsv.gz --fixtures fixtures.jsonl [--self-check N] --out report.json
//! ```

use std::process::ExitCode;

mod cli;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}
