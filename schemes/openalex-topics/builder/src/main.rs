//! `openalex-topics-scheme`: build shelfmark's OpenAlex Topics scheme.
//!
//! ```text
//! openalex-topics-scheme fetch --cache DIR [--data DIR]   download the API pages, write the lock
//! openalex-topics-scheme build --cache DIR [--data DIR]   write openalex-topics.json
//! openalex-topics-scheme check --cache DIR [--data DIR]   rebuild; fail if the file differs
//! ```
//!
//! `--data` defaults to the scheme directory (`schemes/openalex-topics`).
//! `build` and `check` read only cached pages whose sha256 matches the
//! lock and need no network.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use openalex_topics_scheme::{generate, sources, LOCK, SCHEME};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("openalex-topics-scheme: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: openalex-topics-scheme <fetch|build|check> --cache DIR [--data DIR]".to_string()
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
    let scheme = data.join(SCHEME);
    match cmd.as_str() {
        "fetch" => {
            let lock = sources::fetch(&cache)?;
            let text = serde_json::to_string_pretty(&lock).map_err(|e| e.to_string())? + "\n";
            write(&data.join(LOCK), &text)
        }
        "build" => {
            let g = generate(&data, &cache)?;
            write(&scheme, &g.json)?;
            eprintln!(
                "openalex-topics-scheme: wrote {} ({:?})",
                scheme.display(),
                g.stats
            );
            Ok(())
        }
        "check" => {
            let g = generate(&data, &cache)?;
            let have = std::fs::read_to_string(&scheme)
                .map_err(|e| format!("{}: {e}", scheme.display()))?;
            if have != g.json {
                return Err(format!(
                    "{} is stale; run `openalex-topics-scheme build`",
                    scheme.display()
                ));
            }
            eprintln!("openalex-topics-scheme: {SCHEME} matches the locked sources");
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}
