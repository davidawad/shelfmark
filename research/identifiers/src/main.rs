use clap::{Parser, Subcommand};
use identifier_tables::rules::{ArxivMap, DomainRules};
use identifier_tables::{build, output, shelfmark};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

#[derive(Parser)]
#[command(
    version,
    about = "Build and check the materials classifier's identifier-tier data"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build lookup.tsv, conflicts.tsv and stats.json from an Open Library editions dump.
    Build {
        /// Editions dump (`.gz` is decompressed on the fly).
        input: PathBuf,
        /// Output directory.
        #[arg(long)]
        out: PathBuf,
        /// Parser threads (output does not depend on this).
        #[arg(long, default_value_t = default_jobs())]
        jobs: usize,
        /// Stop after this many dump lines.
        #[arg(long)]
        limit: Option<u64>,
        /// Print progress to stderr every N lines (0 = never).
        #[arg(long, default_value_t = 0)]
        progress_every: u64,
    },
    /// Validate the arXiv map (against a taxonomy snapshot) and the URL-domain rules.
    Check {
        #[arg(long, default_value = "data/arxiv-lcc.tsv")]
        arxiv: PathBuf,
        #[arg(long, default_value = "data/arxiv-taxonomy.tsv")]
        taxonomy: PathBuf,
        #[arg(long, default_value = "data/url-domain-rules.tsv")]
        domains: PathBuf,
    },
    /// Write the arXiv map and the URL-domain rules as one shelfmark rules-tier file.
    ShelfmarkRules {
        #[arg(long, default_value = "data/arxiv-lcc.tsv")]
        arxiv: PathBuf,
        #[arg(long, default_value = "data/url-domain-rules.tsv")]
        domains: PathBuf,
        #[arg(long, default_value = "../../src/shelfmark/data/rules/starter.json")]
        out: PathBuf,
    },
    /// Convert a built lookup.tsv (with its conflicts.tsv) to a shelfmark identifier table (JSONL).
    ShelfmarkTable {
        /// The `build` output directory.
        dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

fn read(p: &PathBuf) -> Result<String, String> {
    std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))
}

fn write(p: &PathBuf, text: &str) -> Result<(), String> {
    std::fs::write(p, text).map_err(|e| format!("{}: {e}", p.display()))
}

fn default_jobs() -> usize {
    std::thread::available_parallelism().map_or(4, |n| n.get().saturating_sub(2).max(1))
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Build {
            input,
            out,
            jobs,
            limit,
            progress_every,
        } => {
            let started = Instant::now();
            let opts = build::Options {
                jobs: jobs.max(1),
                limit,
                progress_every,
            };
            let mut table =
                build::build(&input, &opts).map_err(|e| format!("{}: {e}", input.display()))?;
            let parsed = started.elapsed();
            output::write_all(&mut table, &out).map_err(|e| format!("{}: {e}", out.display()))?;
            // Timing goes to stderr only, so the output files stay byte-identical across runs.
            eprintln!(
                "identifier-tables: {} lines, {} editions, {} keys, {} rows; parse {:.1}s, total {:.1}s",
                table.stats.dump_lines,
                table.stats.edition_records,
                table.stats.table.keys,
                table.stats.table.lookup_rows,
                parsed.as_secs_f64(),
                started.elapsed().as_secs_f64()
            );
            Ok(())
        }
        Command::Check {
            arxiv,
            taxonomy,
            domains,
        } => {
            let map =
                ArxivMap::parse(&read(&arxiv)?).map_err(|e| format!("{}: {e}", arxiv.display()))?;
            let missing = map.missing_from(&read(&taxonomy)?)?;
            if !missing.is_empty() {
                return Err(format!(
                    "{}: categories without a class: {missing:?}",
                    arxiv.display()
                ));
            }
            let rules = DomainRules::parse(&read(&domains)?)
                .map_err(|e| format!("{}: {e}", domains.display()))?;
            println!(
                "ok: {} arXiv categories, {} domain rules",
                map.0.len(),
                rules.0.len()
            );
            Ok(())
        }
        Command::ShelfmarkRules {
            arxiv,
            domains,
            out,
        } => {
            let map =
                ArxivMap::parse(&read(&arxiv)?).map_err(|e| format!("{}: {e}", arxiv.display()))?;
            let rules = DomainRules::parse(&read(&domains)?)
                .map_err(|e| format!("{}: {e}", domains.display()))?;
            let doc = shelfmark::rules(&map, &rules);
            let text = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())? + "\n";
            write(&out, &text)?;
            println!(
                "{}: {} rules",
                out.display(),
                doc["rules"].as_array().map_or(0, Vec::len)
            );
            Ok(())
        }
        Command::ShelfmarkTable { dir, out } => {
            let (jsonl, dropped) = shelfmark::table(
                &read(&dir.join("lookup.tsv"))?,
                &read(&dir.join("conflicts.tsv"))?,
            )?;
            write(&out, &jsonl)?;
            println!(
                "{}: {} identifiers, {dropped} ambiguous keys dropped",
                out.display(),
                jsonl.lines().count()
            );
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("identifier-tables: {e}");
            ExitCode::FAILURE
        }
    }
}
