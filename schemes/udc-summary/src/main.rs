//! `udc-summary`: build a shelfmark scheme file from the UDC Summary.
//!
//! ```text
//! udc-summary build --cache DIR [--fetch] [--out FILE] [--lock FILE]
//! udc-summary check --cache DIR [--out FILE] [--lock FILE]
//! udc-summary fetch --cache DIR [--lock FILE]
//! ```
//!
//! `build` reads only cached pages whose sha256 matches the lock, so it
//! is deterministic; `--fetch` first downloads the locked pages missing
//! from the cache. `check` rebuilds and fails if `--out` differs.
//! `fetch` (maintainers) downloads every page and rewrites the lock.
//! `--lock` defaults to this crate's `sources.lock.json`, `--out` to
//! `out/udc-summary.json` here (git-ignored: the data is never committed).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use udc_summary::{build, generate, read_lock, sources};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("udc-summary: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: udc-summary <build|check|fetch> --cache DIR [--fetch] [--out FILE] [--lock FILE]"
        .to_string()
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let cmd = it.next().ok_or_else(usage)?;
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (mut cache, mut out, mut lock_path, mut fetch) = (None, None, None, false);
    while let Some(flag) = it.next() {
        if flag == "--fetch" {
            fetch = true;
            continue;
        }
        let value = PathBuf::from(it.next().ok_or_else(usage)?);
        match flag.as_str() {
            "--cache" => cache = Some(value),
            "--out" => out = Some(value),
            "--lock" => lock_path = Some(value),
            _ => return Err(usage()),
        }
    }
    let cache = cache.ok_or_else(usage)?;
    let out = out.unwrap_or_else(|| here.join("out/udc-summary.json"));
    let lock_path = lock_path.unwrap_or_else(|| here.join("sources.lock.json"));
    match cmd.as_str() {
        "fetch" => {
            let prior = read_lock(&lock_path).ok();
            let lock = sources::fetch(&cache, prior.as_ref())?;
            let text = serde_json::to_string_pretty(&lock).map_err(|e| e.to_string())?;
            write(&lock_path, &(text + "\n"))?;
            eprintln!(
                "udc-summary: locked {} pages as edition {}",
                lock.sources.len(),
                lock.edition
            );
            Ok(())
        }
        "build" => {
            let lock = read_lock(&lock_path)?;
            if fetch {
                sources::fetch_missing(&cache, &lock)?;
            }
            let (doc, stats) = generate(&cache, &lock)?;
            write(&out, &build::scheme_json(&doc)?)?;
            let pages: Vec<String> = stats
                .per_page
                .iter()
                .map(|(t, n)| format!("{t}:{n}"))
                .collect();
            eprintln!(
                "udc-summary: wrote {} ({} with {} classes; {})\n\
                 load it with: shelfmark scheme compile {} --commit",
                out.display(),
                doc.id(),
                doc.classes.len(),
                pages.join(" "),
                out.display()
            );
            Ok(())
        }
        "check" => {
            let lock = read_lock(&lock_path)?;
            let (doc, _) = generate(&cache, &lock)?;
            let have =
                std::fs::read_to_string(&out).map_err(|e| format!("{}: {e}", out.display()))?;
            if have != build::scheme_json(&doc)? {
                return Err(format!(
                    "{} is stale; run `udc-summary build`",
                    out.display()
                ));
            }
            eprintln!("udc-summary: {} matches the locked pages", out.display());
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}
