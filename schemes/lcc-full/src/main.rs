//! `lcc-full`: build the complete LCC schedules as a shelfmark scheme file.
//!
//! ```text
//! lcc-full fetch --cache DIR [--relock]   download the Text PDFs; check them, or rewrite the lock
//! lcc-full build --cache DIR              write lcc-full.json(.gz), COVERAGE.md, coverage-failures.txt
//! lcc-full check --cache DIR              rebuild; fail if any of those differ
//! ```
//!
//! Options: `--lock FILE` (default `sources.lock.json` in the crate),
//! `--outline FILE` (default the repo's `data/schemes/lcc/lcc.json`),
//! `--out DIR` (default the crate directory). `build` and `check` refuse a
//! PDF whose sha256 differs from the lock and need no network.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use flate2::write::GzEncoder;
use flate2::Compression;
use lcc_full::{build, source};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lcc-full: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: lcc-full <fetch|build|check> --cache DIR [--lock FILE] [--outline FILE] [--out DIR] [--relock]"
        .into()
}

struct Opts {
    cache: PathBuf,
    lock: PathBuf,
    outline: PathBuf,
    out: PathBuf,
    relock: bool,
}

fn opts(args: impl Iterator<Item = String>) -> Result<Opts, String> {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (mut cache, mut lock, mut outline, mut out, mut relock) = (None, None, None, None, false);
    let mut it = args;
    while let Some(flag) = it.next() {
        if flag == "--relock" {
            relock = true;
            continue;
        }
        let value = PathBuf::from(it.next().ok_or_else(usage)?);
        match flag.as_str() {
            "--cache" => cache = Some(value),
            "--lock" => lock = Some(value),
            "--outline" => outline = Some(value),
            "--out" => out = Some(value),
            _ => return Err(usage()),
        }
    }
    Ok(Opts {
        cache: cache.ok_or_else(usage)?,
        lock: lock.unwrap_or_else(|| here.join("sources.lock.json")),
        outline: outline.unwrap_or_else(|| here.join("../../data/schemes/lcc/lcc.json")),
        out: out.unwrap_or(here),
        relock,
    })
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let cmd = it.next().ok_or_else(usage)?;
    let o = opts(it)?;
    match cmd.as_str() {
        "fetch" => {
            let hash = lcc_full::outline_hash(&o.outline)?;
            let new = source::fetch(&o.cache, &hash)?;
            if o.relock {
                write(&o.lock, new.to_json()?.as_bytes())?;
                eprintln!("relocked {}", o.lock.display());
                return Ok(());
            }
            let old = source::Lock::read(&o.lock)?;
            for (a, b) in old.sources.iter().zip(&new.sources) {
                if a.sha256 != b.sha256 {
                    return Err(format!("{} changed upstream; rerun with --relock", a.file));
                }
            }
            eprintln!("{} files match the lock", new.sources.len());
            Ok(())
        }
        "build" | "check" => {
            let lock = source::Lock::read(&o.lock)?;
            let out = lcc_full::run(&o.cache, &o.outline, &lock)?;
            let json = build::scheme_json(&out.built.doc)?;
            let files = [
                ("lcc-full.json.gz", gzip(json.as_bytes())?),
                (
                    "COVERAGE.md",
                    build::coverage(&out.built, &out.schedules).into_bytes(),
                ),
                (
                    "coverage-failures.txt",
                    build::failures(&out.schedules).into_bytes(),
                ),
            ];
            if cmd == "build" {
                for (name, bytes) in &files {
                    write(&o.out.join(name), bytes)?;
                }
                // The uncompressed file, for `shelfmark scheme compile`.
                write(&o.out.join("lcc-full.json"), json.as_bytes())?;
                let s = &out.built.stats;
                eprintln!(
                    "wrote {} classes, depth {}, branching max {} mean {:.2}; next: shelfmark scheme compile {}",
                    s.classes,
                    s.depth,
                    s.max_branching,
                    s.mean_branching,
                    o.out.join("lcc-full.json").display()
                );
                return Ok(());
            }
            for (name, bytes) in &files {
                let path = o.out.join(name);
                let have = std::fs::read(&path)
                    .map_err(|e| format!("{}: {e} (run `lcc-full build`)", path.display()))?;
                if &have != bytes {
                    return Err(format!("{} is stale; run `lcc-full build`", path.display()));
                }
            }
            Ok(())
        }
        _ => Err(usage()),
    }
}

/// Gzip with fixed settings, so the same bytes always compress the same.
fn gzip(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::best());
    enc.write_all(bytes).map_err(|e| e.to_string())?;
    enc.finish().map_err(|e| e.to_string())
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}
