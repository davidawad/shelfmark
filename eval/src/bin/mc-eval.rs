use clap::{Args, Parser, Subcommand, ValueEnum};
use mc_eval::error::{read_file, EvalError, Result};
use mc_eval::gold::{stats, GoldSet};
use mc_eval::goldbuild::{build, sha256_hex};
use mc_eval::report::{run, MissingPolicy, Options};
use mc_eval::summary::Config;
use mc_eval::{describe, pred, render};
use serde_json::{json, Value};
use std::io::{IsTerminal, Write};

#[derive(Parser)]
#[command(
    name = "mc-eval",
    version,
    about = "Evaluate materials-classifier predictions against a gold set"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Describe nouns, verbs, metrics, errors and an example call (JSON).
    Describe,
    /// Score predictions against a gold set.
    Run(RunArgs),
    /// Check a gold set (and optionally predictions) without scoring.
    Validate {
        #[arg(long)]
        gold: String,
        #[arg(long, num_args = 1..)]
        predictions: Vec<String>,
    },
    /// Build or inspect gold sets.
    #[command(subcommand)]
    Gold(GoldCmd),
}

#[derive(Subcommand)]
enum GoldCmd {
    /// Build gold NDJSON from a pinned manifest and its raw snapshots (offline).
    Build {
        #[arg(long)]
        manifest: String,
        #[arg(long)]
        out: Option<String>,
    },
    /// Coverage counts: items, main classes, sources, depths.
    Stats {
        #[arg(long)]
        gold: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Auto,
    Json,
    Text,
}

#[derive(Clone, Copy, ValueEnum)]
enum Missing {
    AsWrong,
    Skip,
}

#[derive(Args)]
struct RunArgs {
    #[arg(long)]
    gold: String,
    #[arg(long, num_args = 1.., required = true)]
    predictions: Vec<String>,
    /// Top-k ranks to report besides top-1.
    #[arg(long, value_delimiter = ',', default_value = "3,5")]
    k: Vec<usize>,
    /// Hierarchy level for calibration bins, risk-coverage and thresholds.
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=5))]
    level: u8,
    /// Bootstrap resamples (0 disables intervals).
    #[arg(long, default_value_t = 1000)]
    bootstrap: usize,
    #[arg(long, default_value_t = 20261007)]
    seed: u64,
    /// Interval coverage.
    #[arg(long, default_value_t = 0.95)]
    confidence: f64,
    /// Calibration bins.
    #[arg(long, default_value_t = 10)]
    bins: usize,
    /// Precision targets to derive confidence thresholds for.
    #[arg(long, value_delimiter = ',', default_value = "0.8,0.9,0.95")]
    precision: Vec<f64>,
    #[arg(long, value_enum, default_value = "as-wrong")]
    missing: Missing,
    #[arg(long, value_enum, default_value = "auto")]
    format: Format,
    /// Write the JSON report here as well as rendering to stdout.
    #[arg(long)]
    out: Option<String>,
}

fn usage(msg: impl Into<String>) -> EvalError {
    EvalError::new("E_USAGE", msg, "mc-eval describe")
}

fn emit(v: &Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(v).expect("JSON values serialize")
    );
}

fn write_out(path: &str, text: &str) -> Result<()> {
    std::fs::write(path, text).map_err(|e| {
        EvalError::new(
            "E_WRITE",
            format!("cannot write `{path}`: {e}"),
            "choose a writable --out path",
        )
    })
}

fn cmd_run(a: RunArgs) -> Result<()> {
    if a.bins == 0 {
        return Err(usage("--bins must be at least 1"));
    }
    if !(0.0 < a.confidence && a.confidence < 1.0) {
        return Err(usage("--confidence must be in (0, 1)"));
    }
    if a.k.contains(&0) || a.precision.iter().any(|p| !(0.0..=1.0).contains(p)) {
        return Err(usage(
            "--k values must be >= 1 and --precision values in [0, 1]",
        ));
    }
    let text = read_file(&a.gold, "pass an existing gold NDJSON file with --gold")?;
    let gold = GoldSet::parse(&text)?;
    let loaded = pred::load(&a.predictions, &gold)?;
    let mut ks = a.k.clone();
    ks.sort_unstable();
    ks.dedup();
    let opt = Options {
        metrics: Config {
            ks,
            primary_level: a.level as usize,
            bins: a.bins,
            target_precisions: a.precision.clone(),
        },
        bootstrap: a.bootstrap,
        seed: a.seed,
        confidence: a.confidence,
        missing: match a.missing {
            Missing::AsWrong => MissingPolicy::AsWrong,
            Missing::Skip => MissingPolicy::Skip,
        },
    };
    let report = run(&gold, &a.gold, sha256_hex(text.as_bytes()), &loaded, &opt);
    let json = serde_json::to_string_pretty(&report).expect("report serializes");
    if let Some(out) = &a.out {
        write_out(out, &format!("{json}\n"))?;
    }
    let text_mode = match a.format {
        Format::Text => true,
        Format::Json => false,
        Format::Auto => std::io::stdout().is_terminal(),
    };
    if text_mode {
        print!("{}", render::text(&report));
    } else {
        println!("{json}");
    }
    Ok(())
}

fn cmd_validate(gold_path: &str, preds: &[String]) -> Result<()> {
    let gold = GoldSet::load(gold_path)?;
    let mut v = json!({"ok": true, "gold": stats(&gold)});
    if !preds.is_empty() {
        let loaded = pred::load(preds, &gold)?;
        let mut profiles = std::collections::BTreeMap::<&str, usize>::new();
        for p in &loaded.predictions {
            *profiles.entry(&p.profile).or_default() += 1;
        }
        let unknown: Vec<Value> = loaded
            .unknown
            .iter()
            .map(|(ln, k)| json!({"line": ln, "item": k}))
            .collect();
        v["predictions"] = json!({"matched": loaded.predictions.len(), "profiles": profiles, "unknown_items": unknown});
    }
    emit(&v);
    Ok(())
}

fn cmd_gold(c: GoldCmd) -> Result<()> {
    match c {
        GoldCmd::Build { manifest, out } => {
            let built = build(&manifest)?;
            let mut text = String::new();
            for it in &built.items {
                text.push_str(&serde_json::to_string(it).expect("gold item serializes"));
                text.push('\n');
            }
            for n in &built.notes {
                eprintln!("{n}");
            }
            match out {
                Some(p) => write_out(&p, &text),
                None => {
                    print!("{text}");
                    Ok(())
                }
            }
        }
        GoldCmd::Stats { gold } => {
            emit(&serde_json::to_value(stats(&GoldSet::load(&gold)?)).expect("stats serialize"));
            Ok(())
        }
    }
}

fn report_error(e: &EvalError) -> ! {
    if std::io::stderr().is_terminal() {
        eprintln!("error: {e}");
    } else {
        let line = json!({"error": e}).to_string();
        let _ = writeln!(std::io::stderr(), "{line}");
    }
    std::process::exit(e.exit_code())
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e)
            if matches!(
                e.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            e.exit()
        }
        Err(e) => report_error(&usage(e.to_string().trim().to_string())),
    };
    let result = match cli.cmd {
        Cmd::Describe => {
            emit(&describe::describe());
            Ok(())
        }
        Cmd::Run(a) => cmd_run(a),
        Cmd::Validate { gold, predictions } => cmd_validate(&gold, &predictions),
        Cmd::Gold(c) => cmd_gold(c),
    };
    if let Err(e) = result {
        report_error(&e);
    }
}
