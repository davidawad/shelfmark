//! `bk-scheme`: build the Basic Classification as a shelfmark scheme file.
//!
//! ```text
//! bk-scheme fetch --cache DIR [--relock] [--lock FILE]  download the source; check it, or relock to it
//! bk-scheme build --cache DIR [--out FILE] [--lock FILE] write bk.json (fetches a missing source)
//! bk-scheme check --cache DIR [--out FILE] [--lock FILE] rebuild; fail if FILE differs
//! ```
//!
//! `--lock` defaults to this crate's `sources.lock.json`, `--out` to its
//! `bk.json` (committed: the data is CC0). `build` and `check` refuse a
//! source whose sha256 differs from the lock, so the same lock always
//! builds the same file; with the source cached they need no network.

use std::path::PathBuf;
use std::process::ExitCode;

use bk_scheme::source::{self, Lock};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("bk-scheme: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: bk-scheme <fetch|build|check> --cache DIR [--out FILE] [--lock FILE] [--relock]".into()
}

struct Opts {
    cache: PathBuf,
    out: PathBuf,
    lock: PathBuf,
    relock: bool,
}

fn opts(args: impl Iterator<Item = String>) -> Result<Opts, String> {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (mut cache, mut out, mut lock, mut relock) = (None, None, None, false);
    let mut it = args;
    while let Some(flag) = it.next() {
        if flag == "--relock" {
            relock = true;
            continue;
        }
        let value = PathBuf::from(it.next().ok_or_else(usage)?);
        match flag.as_str() {
            "--cache" => cache = Some(value),
            "--out" => out = Some(value),
            "--lock" => lock = Some(value),
            _ => return Err(usage()),
        }
    }
    Ok(Opts {
        cache: cache.ok_or_else(usage)?,
        out: out.unwrap_or_else(|| here.join("bk.json")),
        lock: lock.unwrap_or_else(|| here.join("sources.lock.json")),
        relock,
    })
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let cmd = it.next().ok_or_else(usage)?;
    let o = opts(it)?;
    let lock = Lock::read(&o.lock)?;
    match cmd.as_str() {
        "fetch" => {
            let (path, bytes) = source::fetch(&o.cache)?;
            if o.relock {
                let new = lock.relocked(&bytes, source::today());
                write(&o.lock, &new.to_json()?)?;
                eprintln!(
                    "relocked {} to sha256 {}; set `version` to the export date",
                    o.lock.display(),
                    new.source.sha256
                );
                return Ok(());
            }
            lock.verify(&bytes)?;
            eprintln!("{} matches the lock", path.display());
            Ok(())
        }
        "build" => {
            let json = generate(&o, &lock)?;
            write(&o.out, &json)?;
            eprintln!(
                "wrote {}; next: shelfmark scheme compile {}",
                o.out.display(),
                o.out.display()
            );
            Ok(())
        }
        "check" => {
            let json = generate(&o, &lock)?;
            let have = std::fs::read_to_string(&o.out)
                .map_err(|e| format!("{}: {e} (run `bk-scheme build`)", o.out.display()))?;
            if have != json {
                return Err(format!(
                    "{} is stale; run `bk-scheme build`",
                    o.out.display()
                ));
            }
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn generate(o: &Opts, lock: &Lock) -> Result<String, String> {
    let src = source::cached(&o.cache, lock)?;
    bk_scheme::scheme_json(&bk_scheme::scheme(&src, lock)?)
}

fn write(path: &std::path::Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}
