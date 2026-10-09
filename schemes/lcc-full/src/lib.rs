//! Builds the complete Library of Congress Classification schedules as a
//! shelfmark scheme from the FreeLCC Text PDFs, layered on the bundled
//! outline scheme. See the crate README.

pub mod build;
pub mod code;
pub mod parse;
pub mod pdf;
pub mod source;

use std::path::Path;

use shelfmark_core::id::sha256_hex;
use shelfmark_core::scheme::SchemeDoc;

use build::Built;
use parse::Schedule;
use source::Lock;

/// Everything a build produces.
pub struct Output {
    pub built: Built,
    pub schedules: Vec<(String, Schedule)>,
}

/// The sha256 of the outline scheme file.
pub fn outline_hash(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

/// Build from the cached, lock-checked PDFs and the outline file.
pub fn run(cache: &Path, outline: &Path, lock: &Lock) -> Result<Output, String> {
    let bytes = std::fs::read(outline).map_err(|e| format!("{}: {e}", outline.display()))?;
    let got = sha256_hex(&bytes);
    if got != lock.outline_sha256 {
        return Err(format!(
            "{} has sha256 {got}, the lock says {}; relock with `lcc-full fetch --relock`",
            outline.display(),
            lock.outline_sha256
        ));
    }
    let doc: SchemeDoc = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let names: Vec<&'static str> = source::all().into_iter().map(|(_, _, n)| n).collect();
    let parsed: Vec<Result<(String, Schedule), String>> = std::thread::scope(|s| {
        let handles: Vec<_> = lock
            .sources
            .iter()
            .zip(&names)
            .map(|(locked, name)| {
                s.spawn(move || {
                    let pdf = source::read(cache, locked)?;
                    let pages = pdf::pages(&pdf).map_err(|e| format!("{}: {e}", locked.file))?;
                    Ok((name.to_string(), parse::parse(&pages)))
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_else(|_| Err("parser panicked".into())))
            .collect()
    });
    let schedules = parsed.into_iter().collect::<Result<Vec<_>, _>>()?;
    let built = build::build(&schedules, &doc, lock)?;
    Ok(Output { built, schedules })
}
