//! `fdc-scheme`: build shelfmark's Free Decimal Correspondence scheme file.
//!
//! ```text
//! fdc-scheme fetch --cache DIR [--data DIR]   download fdc.txt, write the lock
//! fdc-scheme build --cache DIR [--data DIR]   build fdc.json and SOURCES.md
//! fdc-scheme check --cache DIR [--data DIR]   rebuild; fail if output differs
//! ```
//!
//! `--data` defaults to `schemes/fdc`. `build` and `check` read only a
//! cached source whose sha256 matches the lock, so they are deterministic
//! and need no network.

mod build;
mod report;
mod sources;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sources::Lock;

const LOCK: &str = "sources.lock.json";
const SCHEME: &str = "fdc.json";
const NOTES: &str = "SOURCES.md";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fdc-scheme: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: fdc-scheme <fetch|build|check> --cache DIR [--data DIR]".to_string()
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
    let data = data.unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join(".."));
    match cmd.as_str() {
        "fetch" => {
            let lock = sources::fetch(&cache)?;
            generate_from(&cache, &lock)?;
            std::fs::create_dir_all(&data).map_err(|e| format!("{}: {e}", data.display()))?;
            let text = serde_json::to_string_pretty(&lock).map_err(|e| e.to_string())?;
            write(&data.join(LOCK), &(text + "\n"))
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
                        "{} is stale; run `fdc-scheme build`",
                        path.display()
                    ));
                }
            }
            eprintln!("fdc-scheme: {SCHEME} and {NOTES} match the locked source");
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Build the scheme JSON and SOURCES.md text from the lock in `data`.
fn generate(cache: &Path, data: &Path) -> Result<(String, String), String> {
    let lock_path = data.join(LOCK);
    let lock: Lock = serde_json::from_str(
        &std::fs::read_to_string(&lock_path)
            .map_err(|e| format!("{}: {e}", lock_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", lock_path.display()))?;
    generate_from(cache, &lock)
}

/// Build from `lock`; a fetch runs this before it writes the lock, so a
/// source whose layout changed never gets locked.
fn generate_from(cache: &Path, lock: &Lock) -> Result<(String, String), String> {
    let locked = lock.get()?;
    let text = sources::read(cache, locked)?;
    let mut stats = build::Stats::default();
    let entries = build::parse(&text, &mut stats)?;
    let (doc, stats) = build::build(&entries, locked, stats)?;
    Ok((
        report::scheme_json(&doc)?,
        report::sources_md(lock, &stats, &doc),
    ))
}
