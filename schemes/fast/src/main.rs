//! `fast-scheme`: build shelfmark's FAST scheme from OCLC's FAST data.
//!
//! ```text
//! fast-scheme fetch --cache DIR             download the locked sources
//! fast-scheme build --cache DIR [--out DIR] build the scheme, the sample and SOURCES.md
//! fast-scheme check --cache DIR [--out DIR] rebuild; fail if any output differs
//! ```
//!
//! The full scheme (`fast.json`, about 100 MB) goes to `--out`, by default
//! `out/` next to this file, which git ignores: it is too large to commit.
//! `fast-sample.json` and `SOURCES.md` are committed here. `build` and
//! `check` read only cached sources whose sha256 matches
//! `sources.lock.json`, so they are deterministic and need no network.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use fast_scheme::{generate, sources, Outputs};

const LOCK: &str = "sources.lock.json";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fast-scheme: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: fast-scheme <fetch|build|check> --cache DIR [--out DIR]".to_string()
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let cmd = it.next().ok_or_else(usage)?;
    let (mut cache, mut out) = (None, None);
    while let Some(flag) = it.next() {
        let value = it.next().ok_or_else(usage)?;
        match flag.as_str() {
            "--cache" => cache = Some(PathBuf::from(value)),
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err(usage()),
        }
    }
    let cache = cache.ok_or_else(usage)?;
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = out.unwrap_or_else(|| here.join("out"));
    let lock = sources::read_lock(&here.join(LOCK))?;
    let targets = [
        (out.join(Outputs::SCHEME), 0),
        (here.join(Outputs::SAMPLE), 1),
        (here.join(Outputs::NOTES), 2),
    ];
    match cmd.as_str() {
        "fetch" => sources::fetch(&cache, &lock),
        "build" => {
            let o = generate(&cache, &lock)?;
            std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
            for (path, i) in &targets {
                std::fs::write(path, o.get(*i)).map_err(|e| format!("{}: {e}", path.display()))?;
            }
            eprintln!("fast-scheme: wrote {}", out.join(Outputs::SCHEME).display());
            Ok(())
        }
        "check" => {
            let o = generate(&cache, &lock)?;
            for (path, i) in &targets {
                let have = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
                if have != o.get(*i).as_bytes() {
                    return Err(format!(
                        "{} is stale; run `fast-scheme build`",
                        path.display()
                    ));
                }
            }
            eprintln!("fast-scheme: outputs match the locked sources");
            Ok(())
        }
        _ => Err(usage()),
    }
}
