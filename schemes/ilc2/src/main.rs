//! `ilc2`: build a shelfmark scheme file from ILC2.
//!
//! ```text
//! ilc2 build --cache DIR [--fetch] [--out FILE] [--lock FILE]
//! ilc2 check --cache DIR [--out FILE] [--lock FILE]
//! ilc2 fetch --cache DIR [--lock FILE]
//! ```
//!
//! `build` reads only the cached SKOS file whose sha256 matches the lock,
//! so it is deterministic; `--fetch` first downloads it when the cache
//! lacks it. `check` rebuilds and fails if `--out` differs. `fetch`
//! (maintainers) downloads the file and rewrites the lock. `--lock`
//! defaults to this crate's `sources.lock.json`, `--out` to
//! `out/ilc2.json` here (git-ignored: the data is never committed).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ilc2::build::{self, Shape};
use ilc2::{generate, read_lock, sources};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ilc2: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: ilc2 <build|check|fetch> --cache DIR [--fetch] [--out FILE] [--lock FILE]".to_string()
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
    let out = out.unwrap_or_else(|| here.join("out/ilc2.json"));
    let lock_path = lock_path.unwrap_or_else(|| here.join("sources.lock.json"));
    match cmd.as_str() {
        "fetch" => {
            let lock = sources::fetch(&cache)?;
            let text = serde_json::to_string_pretty(&lock).map_err(|e| e.to_string())?;
            write(&lock_path, &(text + "\n"))?;
            eprintln!(
                "ilc2: locked {} bytes, {}",
                lock.source.bytes,
                lock.digest()
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
            eprintln!(
                "ilc2: wrote {} ({} with {} classes)\n  concepts: {}\n  facets:   {}\n  all:      {}\n\
                 load it with: shelfmark scheme compile {} --commit",
                out.display(),
                doc.id(),
                doc.classes.len(),
                line(&stats.concepts),
                line(&stats.facets),
                line(&stats.all),
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
                return Err(format!("{} is stale; run `ilc2 build`", out.display()));
            }
            eprintln!("ilc2: {} matches the locked file", out.display());
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn line(s: &Shape) -> String {
    format!(
        "{} classes, {} roots, depth {}, branching max {} mean {:.2}",
        s.classes, s.roots, s.depth, s.max_branching, s.mean_branching
    )
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}
