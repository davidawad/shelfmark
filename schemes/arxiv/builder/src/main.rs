//! `arxiv-scheme`: build shelfmark's arXiv category taxonomy scheme.
//!
//! ```text
//! arxiv-scheme fetch --cache DIR [--data DIR]   download sources, write the lock
//! arxiv-scheme build --cache DIR [--data DIR]   cross-check the page, write arxiv.json
//! arxiv-scheme check [--cache DIR] [--data DIR] rebuild; fail if arxiv.json differs
//! ```
//!
//! `--data` defaults to `src/shelfmark/schemes/arxiv`. `build` and `check`
//! read only sources whose sha256 matches the lock and need no network.
//! `check` cross-checks the taxonomy page only when given `--cache`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use arxiv_scheme::{generate, sources, LOCK, SCHEME};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("arxiv-scheme: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: arxiv-scheme <fetch|build|check> [--cache DIR] [--data DIR]".to_string()
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
    let data = data.unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join(".."));
    let scheme = data.join(SCHEME);
    match cmd.as_str() {
        "fetch" => {
            let cache = cache.ok_or_else(usage)?;
            let lock = sources::fetch(&data, &cache)?;
            let text = serde_json::to_string_pretty(&lock).map_err(|e| e.to_string())? + "\n";
            write(&data.join(LOCK), &text)
        }
        "build" => {
            let cache = cache.ok_or_else(usage)?;
            let g = generate(&data, Some(&cache))?;
            write(&scheme, &g.json)?;
            eprintln!("arxiv-scheme: wrote {} ({:?})", scheme.display(), g.stats);
            Ok(())
        }
        "check" => {
            let g = generate(&data, cache.as_deref())?;
            let have = std::fs::read_to_string(&scheme)
                .map_err(|e| format!("{}: {e}", scheme.display()))?;
            if have != g.json {
                return Err(format!(
                    "{} is stale; run `arxiv-scheme build`",
                    scheme.display()
                ));
            }
            let page = if g.cross_checked {
                "cross-checked"
            } else {
                "not cross-checked (no --cache)"
            };
            eprintln!("arxiv-scheme: {SCHEME} matches the locked sources; taxonomy page {page}");
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}
