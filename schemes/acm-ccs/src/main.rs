//! `acm-ccs`: fetch the ACM CCS 2012 and build it as a shelfmark scheme.
//!
//! ```text
//! acm-ccs fetch [--cache DIR] [--relock]   download the SKOS file (network)
//! acm-ccs build [--cache DIR] [--out FILE] [--offline]
//! ```
//!
//! `--cache` defaults to `cache/` and `--out` to `out/acm-ccs.json`, both
//! under this crate and git-ignored. `build` downloads the file when it is
//! not cached (unless `--offline`), refuses one whose sha256 differs from
//! `sources.lock.json`, and writes the same bytes on every run.
//! `fetch --relock` accepts a changed file and rewrites the lock.

use std::path::PathBuf;
use std::process::ExitCode;

use acm_ccs::sources::{self, Lock};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("acm-ccs: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: acm-ccs fetch [--cache DIR] [--relock] | \
     acm-ccs build [--cache DIR] [--out FILE] [--offline]"
        .to_string()
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let cmd = it.next().ok_or_else(usage)?;
    let (mut cache, mut out, mut relock, mut offline) = (None, None, false, false);
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--cache" => cache = Some(PathBuf::from(it.next().ok_or_else(usage)?)),
            "--out" => out = Some(PathBuf::from(it.next().ok_or_else(usage)?)),
            "--relock" if cmd == "fetch" => relock = true,
            "--offline" if cmd == "build" => offline = true,
            _ => return Err(usage()),
        }
    }
    let cache = cache.unwrap_or_else(|| acm_ccs::home().join("cache"));
    let lock_path = acm_ccs::lock_path();
    let lock = Lock::read(&lock_path)?;
    match cmd.as_str() {
        "fetch" if out.is_none() => {
            let new = sources::fetch(&cache, &lock, relock)?;
            if new != lock {
                std::fs::write(&lock_path, new.to_json()?)
                    .map_err(|e| format!("{}: {e}", lock_path.display()))?;
                eprintln!("acm-ccs: rewrote {}", lock_path.display());
            }
            eprintln!(
                "acm-ccs: {} matches the lock",
                cache.join(&new.file).display()
            );
            Ok(())
        }
        "build" => {
            let out = out.unwrap_or_else(|| acm_ccs::home().join("out/acm-ccs.json"));
            let bytes = acm_ccs::source_bytes(&cache, &lock, offline)?;
            let (_, json, stats) = acm_ccs::build_bytes(&bytes, &lock)?;
            if let Some(dir) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
                std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            std::fs::write(&out, json).map_err(|e| format!("{}: {e}", out.display()))?;
            eprintln!(
                "acm-ccs: wrote {} ({} classes, {} top-level, depth {}, {} concepts at more \
                 than one path); next: shelfmark scheme compile {}",
                out.display(),
                stats.classes,
                stats.top,
                stats.depth,
                stats.poly,
                out.display()
            );
            Ok(())
        }
        _ => Err(usage()),
    }
}
