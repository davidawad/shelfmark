//! `ipc-scheme`: build shelfmark's IPC scheme from the WIPO master files.
//!
//! ```text
//! ipc-scheme fetch --cache DIR [--data DIR] [--edition YYYYMMDD]   download, write the lock
//! ipc-scheme build --cache DIR [--data DIR]                        build ipc.json and SOURCES.md
//! ipc-scheme check --cache DIR [--data DIR]                        rebuild; fail if output differs
//! ```
//!
//! `--data` defaults to `schemes/ipc`. `build` and `check`
//! read only cached archives whose sha256 matches the lock, so they are
//! deterministic and need no network.

mod build;
mod catchword;
mod master;
mod report;
mod sources;
mod symbol;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sources::{Lock, Role};

const LOCK: &str = "sources.lock.json";
const SCHEME: &str = "ipc.json";
const NOTES: &str = "SOURCES.md";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ipc-scheme: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: ipc-scheme <fetch|build|check> --cache DIR [--data DIR] [--edition YYYYMMDD]"
        .to_string()
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let cmd = it.next().ok_or_else(usage)?;
    let (mut cache, mut data, mut edition) = (None, None, None);
    while let Some(flag) = it.next() {
        let value = it.next().ok_or_else(usage)?;
        match flag.as_str() {
            "--cache" => cache = Some(PathBuf::from(value)),
            "--data" => data = Some(PathBuf::from(value)),
            "--edition" if cmd == "fetch" => edition = Some(value),
            _ => return Err(usage()),
        }
    }
    let cache = cache.ok_or_else(usage)?;
    let data = data.unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join(".."));
    match cmd.as_str() {
        "fetch" => {
            let edition = edition.unwrap_or_else(|| sources::EDITION.to_string());
            let lock = sources::fetch(&cache, &edition)?;
            let json = serde_json::to_string_pretty(&lock).map_err(|e| e.to_string())?;
            write(&data.join(LOCK), &(json + "\n"))
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
                        "{} is stale; run `ipc-scheme build`",
                        path.display()
                    ));
                }
            }
            eprintln!("ipc-scheme: {SCHEME} and {NOTES} match the locked sources");
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Build the scheme JSON and SOURCES.md text from the locked sources.
fn generate(cache: &Path, data: &Path) -> Result<(String, String), String> {
    let lock_path = data.join(LOCK);
    let lock: Lock = serde_json::from_str(
        &std::fs::read_to_string(&lock_path)
            .map_err(|e| format!("{}: {e}", lock_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", lock_path.display()))?;
    let text = |role: Role| -> Result<String, String> {
        let locked = lock.get(role)?;
        String::from_utf8(sources::read(cache, locked)?)
            .map_err(|e| format!("{}: {e}", locked.member))
    };
    let ctx = |role: Role| {
        let member = lock.get(role).map(|l| l.member.clone()).unwrap_or_default();
        move |e: String| format!("{member}: {e}")
    };
    let parsed = master::parse(&text(Role::Scheme)?).map_err(ctx(Role::Scheme))?;
    let words = catchword::parse(&text(Role::Catchwords)?).map_err(ctx(Role::Catchwords))?;
    let (doc, stats) = build::build(&parsed, &words, &lock)?;
    Ok((
        report::scheme_json(&doc)?,
        report::sources_md(&lock, &stats, &doc),
    ))
}
