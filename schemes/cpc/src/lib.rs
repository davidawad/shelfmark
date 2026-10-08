//! Builds shelfmark's CPC scheme (`scheme:cpc@YYYY.MM`) from the official
//! EPO/USPTO title list. The CPC terms do not let shelfmark ship the
//! data, so nothing generated is committed: the lock pins the archive and
//! the output hashes, and anyone rebuilds the identical file locally.

pub mod build;
pub mod render;
pub mod sources;
pub mod titles;

use build::{Depth, Provenance};
use shelfmark_core::scheme::SchemeDoc;
use sources::Locked;

/// The scheme built at `depth` from a locked archive's bytes, and its
/// JSON text.
pub fn generate(
    bytes: &[u8],
    locked: &Locked,
    release: &str,
    depth: Depth,
) -> Result<(SchemeDoc, String), String> {
    let titles = titles::read_zip(bytes).map_err(|e| format!("{}: {e}", locked.file))?;
    let prov = Provenance {
        release: release.to_string(),
        file: locked.file.clone(),
        url: locked.url.clone(),
        sha256: locked.sha256.clone(),
    };
    let doc = build::build(&titles, &prov, depth)?;
    let text = render::scheme_json(&doc)?;
    Ok((doc, text))
}

/// The scheme file name for a depth: `cpc.json` for the full scheme.
pub fn file_name(depth: Depth) -> String {
    match depth {
        Depth::All => "cpc.json".to_string(),
        d => format!("cpc-{}.json", d.as_str()),
    }
}

/// The depths the lock pins an output hash for.
pub const LOCKED_DEPTHS: [Depth; 2] = [Depth::All, Depth::MainGroup];
