//! `unesco-thesaurus`: build shelfmark's UNESCO Thesaurus scheme.
//!
//! ```text
//! unesco-thesaurus fetch --cache DIR [--lock FILE]                 download, rewrite the lock
//! unesco-thesaurus build --cache DIR [--fetch] [--out F] [--lock F] write the scheme file
//! unesco-thesaurus check --cache DIR [--out F] [--lock F]          fail if the scheme file differs
//! ```
//!
//! `build` reads only an export whose sha256 matches the lock, so it is
//! deterministic; `--fetch` first downloads the locked URL when the cache
//! lacks it. `--lock` defaults to this crate's `sources.lock.json`, `--out`
//! to `unesco-thesaurus.json` beside it.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use unesco_thesaurus::{build, generate, read_lock, sources, LOCK, SCHEME};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("unesco-thesaurus: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: unesco-thesaurus <fetch|build|check> --cache DIR [--fetch] [--out FILE] [--lock FILE]"
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
    let out = out.unwrap_or_else(|| here.join(SCHEME));
    let lock_path = lock_path.unwrap_or_else(|| here.join(LOCK));
    match cmd.as_str() {
        "fetch" => {
            let lock = sources::fetch(&cache, &today())?;
            let text = serde_json::to_string_pretty(&lock).map_err(|e| e.to_string())?;
            write(&lock_path, &(text + "\n"))?;
            eprintln!(
                "unesco-thesaurus: locked {} ({} bytes)",
                lock.url, lock.bytes
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
                "unesco-thesaurus: wrote {} ({}): {stats:?}",
                out.display(),
                doc.id()
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
                    "{} is stale; run `unesco-thesaurus build`",
                    out.display()
                ));
            }
            eprintln!(
                "unesco-thesaurus: {} matches the locked export",
                out.display()
            );
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

/// Today's UTC date as `YYYY-MM-DD` (Hinnant's `civil_from_days`).
fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}
