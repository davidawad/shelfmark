//! `cpc-scheme`: build shelfmark's CPC scheme from the official title list.
//!
//! ```text
//! cpc-scheme fetch --cache DIR [--release YYYYMM]   download, rewrite sources.lock.json
//! cpc-scheme build --cache DIR [--depth D] [--out FILE]   write the scheme file
//! cpc-scheme check --cache DIR                      rebuild; fail unless the lock's hashes match
//! ```
//!
//! `--depth` is `section`, `class`, `subclass`, `main-group` or `all`
//! (default). `--out` defaults to `cpc.json` (or `cpc-<depth>.json`) next
//! to the lock, which is gitignored. `build` and `check` read only a cached
//! archive whose sha256 matches the lock, so they need no network.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cpc_scheme::build::Depth;
use cpc_scheme::sources::{self, Lock, Output};
use cpc_scheme::{file_name, generate, LOCKED_DEPTHS};
use shelfmark_core::id::sha256_hex;

const LOCK: &str = "sources.lock.json";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("cpc-scheme: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: cpc-scheme <fetch|build|check> --cache DIR [--release YYYYMM] [--depth D] [--out FILE]"
        .to_string()
}

#[derive(Default)]
struct Args {
    cache: Option<PathBuf>,
    release: Option<String>,
    depth: Option<Depth>,
    out: Option<PathBuf>,
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let cmd = it.next().ok_or_else(usage)?;
    let mut a = Args::default();
    while let Some(flag) = it.next() {
        let value = it.next().ok_or_else(usage)?;
        match flag.as_str() {
            "--cache" => a.cache = Some(PathBuf::from(value)),
            "--release" => a.release = Some(value),
            "--depth" => a.depth = Some(Depth::parse(&value).ok_or_else(usage)?),
            "--out" => a.out = Some(PathBuf::from(value)),
            _ => return Err(usage()),
        }
    }
    let cache = a.cache.ok_or_else(usage)?;
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lock_path = here.join(LOCK);
    match cmd.as_str() {
        "fetch" => {
            let (release, source) = sources::fetch(&cache, a.release.as_deref())?;
            let bytes = sources::read(&cache, &source)?;
            let mut outputs = Vec::new();
            for depth in LOCKED_DEPTHS {
                let (doc, text) = generate(&bytes, &source, &release, depth)?;
                outputs.push(Output {
                    file: file_name(depth),
                    depth: depth.as_str().to_string(),
                    classes: doc.classes.len(),
                    bytes: text.len() as u64,
                    sha256: sha256_hex(text.as_bytes()),
                });
            }
            let lock = Lock {
                release,
                source,
                outputs,
            };
            let text = serde_json::to_string_pretty(&lock).map_err(|e| e.to_string())? + "\n";
            write(&lock_path, &text)
        }
        "build" => {
            let lock = read_lock(&lock_path)?;
            let depth = a.depth.unwrap_or(Depth::All);
            let bytes = sources::read(&cache, &lock.source)?;
            let (doc, text) = generate(&bytes, &lock.source, &lock.release, depth)?;
            let out = a.out.unwrap_or_else(|| here.join(file_name(depth)));
            write(&out, &text)?;
            eprintln!(
                "cpc-scheme: wrote {} ({} classes, sha256 {})",
                out.display(),
                doc.classes.len(),
                sha256_hex(text.as_bytes())
            );
            Ok(())
        }
        "check" => {
            let lock = read_lock(&lock_path)?;
            let bytes = sources::read(&cache, &lock.source)?;
            for want in &lock.outputs {
                let depth = Depth::parse(&want.depth)
                    .ok_or_else(|| format!("{LOCK}: unknown depth `{}`", want.depth))?;
                let (_, text) = generate(&bytes, &lock.source, &lock.release, depth)?;
                let got = sha256_hex(text.as_bytes());
                if got != want.sha256 {
                    return Err(format!(
                        "{} builds to sha256 {got}, the lock says {}; the builder changed",
                        want.file, want.sha256
                    ));
                }
            }
            eprintln!("cpc-scheme: every locked output rebuilds byte for byte");
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn read_lock(path: &Path) -> Result<Lock, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}
