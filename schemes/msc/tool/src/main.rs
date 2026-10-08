//! `msc-scheme`: build shelfmark's MSC2020 scheme file.
//!
//! ```text
//! msc-scheme build --out FILE [--cache DIR] [--lock FILE] [--offline]
//! msc-scheme check --out FILE [--cache DIR] [--lock FILE] [--offline]
//! msc-scheme relock [--cache DIR] [--lock FILE]
//! ```
//!
//! `build` fetches `MSC_2020.csv` into the cache when it is not there,
//! refuses it unless its sha256 matches the lock, and writes the scheme.
//! `check` fails when FILE differs from a fresh build. `relock` re-pins
//! the lock to whatever upstream serves now. `--lock` defaults to
//! `schemes/msc/sources.lock.json`; `--cache` to
//! `$XDG_CACHE_HOME/shelfmark/msc` (or `~/.cache/shelfmark/msc`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use msc_scheme::sources::{self, Lock};
use msc_scheme::{csv, scheme};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("msc-scheme: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: msc-scheme <build|check> --out FILE [--cache DIR] [--lock FILE] [--offline]\n       \
     msc-scheme relock [--cache DIR] [--lock FILE]"
        .to_string()
}

struct Opts {
    out: Option<PathBuf>,
    cache: PathBuf,
    lock: PathBuf,
    offline: bool,
}

fn parse(args: impl Iterator<Item = String>) -> Result<Opts, String> {
    let mut it = args;
    let (mut out, mut cache, mut lock, mut offline) = (None, None, None, false);
    while let Some(flag) = it.next() {
        if flag == "--offline" {
            offline = true;
            continue;
        }
        let value = PathBuf::from(it.next().ok_or_else(usage)?);
        match flag.as_str() {
            "--out" => out = Some(value),
            "--cache" => cache = Some(value),
            "--lock" => lock = Some(value),
            _ => return Err(usage()),
        }
    }
    Ok(Opts {
        out,
        cache: cache.map_or_else(default_cache, Ok)?,
        lock: lock
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../sources.lock.json")),
        offline,
    })
}

fn default_cache() -> Result<PathBuf, String> {
    let base = match std::env::var_os("XDG_CACHE_HOME").filter(|v| !v.is_empty()) {
        Some(x) => PathBuf::from(x),
        None => {
            PathBuf::from(std::env::var_os("HOME").ok_or("no HOME; pass --cache")?).join(".cache")
        }
    };
    Ok(base.join("shelfmark/msc"))
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let cmd = it.next().ok_or_else(usage)?;
    let o = parse(it)?;
    let lock = Lock::load(&o.lock)?;
    match cmd.as_str() {
        "build" | "check" => {
            let out = o.out.ok_or_else(usage)?;
            let bytes = sources::obtain(&o.cache, &lock, o.offline)?;
            let doc = scheme::build(&csv::parse(&bytes)?, &lock)?;
            let json = scheme::to_json(&doc)?;
            if cmd == "check" {
                let old =
                    std::fs::read_to_string(&out).map_err(|e| format!("{}: {e}", out.display()))?;
                if old != json {
                    return Err(format!(
                        "{} is stale; run `msc-scheme build`",
                        out.display()
                    ));
                }
                eprintln!("{} is up to date", out.display());
                return Ok(());
            }
            std::fs::write(&out, json).map_err(|e| format!("{}: {e}", out.display()))?;
            eprintln!(
                "wrote {} ({} classes); next: shelfmark scheme compile {}",
                out.display(),
                doc.classes.len(),
                out.display()
            );
            Ok(())
        }
        "relock" => {
            let new = sources::relock(&o.cache, &lock, &sources::today())?;
            std::fs::write(&o.lock, new.to_json()?)
                .map_err(|e| format!("{}: {e}", o.lock.display()))?;
            eprintln!(
                "{}: sha256 {} ({} bytes)",
                o.lock.display(),
                new.sha256,
                new.bytes
            );
            Ok(())
        }
        _ => Err(usage()),
    }
}
