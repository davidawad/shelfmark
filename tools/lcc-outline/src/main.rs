//! `lcc-outline`: build shelfmark's bundled LCC outline scheme.
//!
//! ```text
//! lcc-outline fetch --cache DIR [--data DIR]   download sources, write the lock
//! lcc-outline build --cache DIR [--data DIR]   build lcc.json and SOURCES.md
//! lcc-outline check --cache DIR [--data DIR]   rebuild; fail if output differs
//! ```
//!
//! `--data` defaults to `data/schemes/lcc`. `build` and
//! `check` read only cached sources whose sha256 matches the lock, so they
//! are deterministic and need no network.

mod build;
mod code;
mod lcco;
mod lcsh;
mod outline;
mod pdf;
mod report;
mod sources;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sources::{Lock, Role};

const LOCK: &str = "sources.lock.json";
const SCHEME: &str = "lcc.json";
const NOTES: &str = "SOURCES.md";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lcc-outline: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: lcc-outline <fetch|build|check> --cache DIR [--data DIR]".to_string()
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let cmd = it.next().ok_or_else(usage)?;
    let (mut cache, mut data) = (None, None);
    while let Some(flag) = it.next() {
        let value = it.next().ok_or_else(usage)?;
        match flag.as_str() {
            "--cache" => cache = Some(PathBuf::from(value)),
            "--data" => data = Some(PathBuf::from(value)),
            _ => return Err(usage()),
        }
    }
    let cache = cache.ok_or_else(usage)?;
    let data = data
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/schemes/lcc"));
    match cmd.as_str() {
        "fetch" => {
            let mut lock = sources::fetch(&cache)?;
            let parsed = parse_outlines(&cache, &lock)?;
            let listings = parse_listings(&cache, &lock)?;
            let mut names: Vec<String> = parsed
                .entries
                .iter()
                .filter(|e| e.caption.is_empty())
                .map(build::record_name)
                .collect();
            names.extend(build::unlisted_prefixes(&listings, &parsed));
            lock.sources.extend(sources::fetch_records(&cache, &names)?);
            std::fs::create_dir_all(&data).map_err(|e| format!("{}: {e}", data.display()))?;
            write(&data.join(LOCK), &(json_pretty(&lock)? + "\n"))
        }
        "build" => {
            let (scheme, notes) = generate(&cache, &data)?;
            write(&data.join(SCHEME), &scheme)?;
            write(&data.join(NOTES), &notes)
        }
        "check" => {
            let (scheme, notes) = generate(&cache, &data)?;
            for (file, want) in [(SCHEME, scheme), (NOTES, notes)] {
                let path = data.join(file);
                let have = std::fs::read_to_string(&path)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                if have != want {
                    return Err(format!(
                        "{} is stale; run `lcc-outline build`",
                        path.display()
                    ));
                }
            }
            eprintln!("lcc-outline: {SCHEME} and {NOTES} match the locked sources");
            Ok(())
        }
        _ => Err(usage()),
    }
}

/// Parse every locked class outline PDF, in lock order.
fn parse_listings(cache: &Path, lock: &Lock) -> Result<Vec<lcco::Listing>, String> {
    let mut out = Vec::new();
    for locked in lock.sources.iter().filter(|l| l.role == Role::ClassOutline) {
        let bytes = sources::read(cache, locked)?;
        out.push(lcco::parse(
            &pdf::pages(&bytes).map_err(|e| format!("{}: {e}", locked.file))?,
        ));
    }
    Ok(out)
}

/// Parse every locked FreeLCC outline PDF, in lock order.
fn parse_outlines(cache: &Path, lock: &Lock) -> Result<outline::Parsed, String> {
    let mut parsed = outline::Parsed::default();
    for locked in lock.sources.iter().filter(|l| l.role == Role::Outline) {
        let bytes = sources::read(cache, locked)?;
        let pages = pdf::pages(&bytes).map_err(|e| format!("{}: {e}", locked.file))?;
        outline::parse(&pages, &mut parsed);
    }
    Ok(parsed)
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

fn json_pretty<T: serde::Serialize>(v: &T) -> Result<String, String> {
    serde_json::to_string_pretty(v).map_err(|e| e.to_string())
}

/// Build the scheme JSON and SOURCES.md text from the locked sources.
fn generate(cache: &Path, data: &Path) -> Result<(String, String), String> {
    let lock_path = data.join(LOCK);
    let lock: Lock = serde_json::from_str(
        &std::fs::read_to_string(&lock_path)
            .map_err(|e| format!("{}: {e}", lock_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", lock_path.display()))?;
    let listings = parse_listings(cache, &lock)?;
    let parsed = parse_outlines(cache, &lock)?;
    let mut records = BTreeMap::new();
    let mut headings = Vec::new();
    for locked in &lock.sources {
        let ctx = |e: String| format!("{}: {e}", locked.file);
        match locked.role {
            Role::ClassOutline | Role::Outline => {}
            Role::Lcsh => {
                headings.extend(lcsh::headings(&sources::read(cache, locked)?).map_err(ctx)?)
            }
            Role::IdRecord => {
                let name = locked
                    .url
                    .rsplit('/')
                    .next()
                    .unwrap_or_default()
                    .trim_end_matches(".json");
                let bytes = sources::read(cache, locked)?;
                if let Some(label) = sources::record_label(&bytes, name) {
                    records.insert(name.to_string(), label);
                }
            }
        }
    }
    headings.sort();
    let (doc, stats) = build::build(&listings, &parsed, &records, &headings)?;
    let scheme = report::scheme_json(&doc)?;
    let notes = report::sources_md(&lock, &stats, &doc);
    Ok((scheme, notes))
}
