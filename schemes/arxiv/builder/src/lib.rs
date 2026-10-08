//! Builds shelfmark's arXiv category taxonomy scheme
//! (`src/shelfmark/schemes/arxiv/arxiv.json`) from arxiv-base's taxonomy
//! definitions, cross-checked against <https://arxiv.org/category_taxonomy>.
//!
//! [`generate`] is the whole build: it reads the locked sources, refuses
//! any whose sha256 has changed or whose license is no longer MIT, and
//! renders the scheme JSON. The same lock and sources always give the
//! same bytes.

pub mod build;
pub mod page;
pub mod pydef;
pub mod sources;
pub mod taxonomy;

use std::fmt::Write;
use std::path::Path;

use shelfmark_core::scheme::SchemeDoc;

use build::{Provenance, Stats};
use sources::{Lock, Role};

pub const LOCK: &str = "sources.lock.json";
pub const SCHEME: &str = "arxiv.json";

/// A finished build.
pub struct Generated {
    pub doc: SchemeDoc,
    /// The scheme file's exact text.
    pub json: String,
    pub stats: Stats,
    /// Whether the taxonomy page cross-check ran (it needs the cache).
    pub cross_checked: bool,
}

/// The MIT grant every copy must carry.
const MIT_GRANT: &str =
    "Permission is hereby granted, free of charge, to any person obtaining a copy";

/// The license field: the copyright lines of arxiv-base's MIT `LICENSE`.
/// Fails if the file is no longer an MIT grant, so a license change stops
/// the build until someone re-reviews it.
pub fn license(text: &str) -> Result<String, String> {
    let grants = text.matches(MIT_GRANT).count();
    let holders: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("Copyright (c)"))
        .collect();
    if grants == 0 || grants != holders.len() {
        return Err(format!(
            "arxiv-base LICENSE has {grants} MIT grant(s) for {} copyright line(s); \
             review the new terms before shipping the data",
            holders.len()
        ));
    }
    Ok(format!(
        "MIT. {}. The full notice, which must accompany copies, is schemes/arxiv/sources/LICENSE.",
        holders.join("; ")
    ))
}

fn description(lock: &Lock) -> Result<String, String> {
    let defs = lock.get(Role::Definitions)?;
    let page = lock.get(Role::Page)?;
    let commit = defs
        .commit
        .as_deref()
        .ok_or("the definitions source has no commit")?;
    Ok(format!(
        "Groups, archives and categories from {repo} {file} at commit {commit} \
         (sha256 {sha}), the definitions arXiv renders as {url}; that page (sha256 {psha}, \
         fetched {pdate}) lists the same active categories with the same names and groups. \
         Hierarchy: group (grp_*) > archive > category (archive.subject); a category whose id \
         is its archive's is the archive's class. Retired archives are kept under their group, \
         test groups dropped. Notes carry arXiv's descriptions, aliases and archive history. \
         The source links no LCC or LCSH headings. Built by src/shelfmark/schemes/arxiv/builder; \
         see schemes/arxiv/README.md.",
        repo = sources::REPO,
        file = sources::DEFINITIONS,
        sha = defs.sha256,
        url = sources::PAGE,
        psha = page.sha256,
        pdate = page.fetched,
    ))
}

/// The scheme as JSON: header fields pretty-printed, then one class per
/// line, so a rebuild diffs line by line.
pub fn scheme_json(doc: &SchemeDoc) -> Result<String, String> {
    let err = |e: serde_json::Error| e.to_string();
    let mut head = doc.clone();
    head.classes.clear();
    let head = serde_json::to_string_pretty(&head).map_err(err)?;
    let head = head
        .trim_end()
        .strip_suffix('}')
        .and_then(|h| h.trim_end().strip_suffix("\"classes\": []"))
        .ok_or("unexpected scheme JSON layout")?;
    let mut out = format!("{head}\"classes\": [\n");
    for (i, c) in doc.classes.iter().enumerate() {
        let sep = if i + 1 < doc.classes.len() { "," } else { "" };
        let _ = writeln!(out, "    {}{sep}", serde_json::to_string(c).map_err(err)?);
    }
    out.push_str("  ]\n}\n");
    Ok(out)
}

/// Build the scheme from the sources locked in `data/sources.lock.json`.
/// With a `cache` holding the locked taxonomy page, also cross-check it.
pub fn generate(data: &Path, cache: Option<&Path>) -> Result<Generated, String> {
    let lock = Lock::read(&data.join(LOCK))?;
    let text = |role| -> Result<String, String> {
        let locked = lock.get(role)?;
        String::from_utf8(sources::read(data, cache, locked)?)
            .map_err(|e| format!("{}: {e}", locked.file))
    };
    let taxonomy = taxonomy::parse(&text(Role::Definitions)?)?;
    let version = lock
        .get(Role::Definitions)?
        .changed
        .clone()
        .ok_or("the definitions source has no change date")?;
    let provenance = Provenance {
        version,
        description: description(&lock)?,
        license: license(&text(Role::License)?)?,
    };
    let (doc, stats) = build::scheme(&taxonomy, &provenance)?;
    let cross_checked = cache.is_some();
    if cross_checked {
        let listed = page::parse(&text(Role::Page)?)?;
        build::cross_check(&doc, &listed, &taxonomy)?;
    }
    let json = scheme_json(&doc)?;
    Ok(Generated {
        doc,
        json,
        stats,
        cross_checked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn license_lists_every_holder_of_an_mit_grant() {
        let text = format!(
            "Copyright (c) 2026 arXiv, Inc.\n\n{MIT_GRANT} of this...\n---\n\
             Copyright (c) 2017-2025 Cornell University\n\n{MIT_GRANT} of this...\n"
        );
        let got = license(&text).unwrap();
        assert!(got.starts_with(
            "MIT. Copyright (c) 2026 arXiv, Inc.; Copyright (c) 2017-2025 Cornell University."
        ));
        assert!(license("Copyright (c) 2026 arXiv, Inc.\nAll rights reserved.").is_err());
    }
}
