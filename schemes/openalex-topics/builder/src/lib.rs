//! Builds shelfmark's OpenAlex Topics scheme
//! (`schemes/openalex-topics/openalex-topics.json`) from the OpenAlex API.
//!
//! [`generate`] is the whole build: it reads the locked, cached API pages,
//! refuses any whose sha256 has changed or whose license page no longer
//! says CC0, and renders the scheme JSON. The same lock and cache always
//! give the same bytes.

pub mod build;
pub mod sources;

use std::fmt::Write;
use std::path::Path;

use build::{Entity, Provenance, Stats};
use shelfmark_core::scheme::SchemeDoc;
use sources::{Lock, Role};

pub const LOCK: &str = "sources.lock.json";
pub const SCHEME: &str = "openalex-topics.json";

/// A finished build.
pub struct Generated {
    pub doc: SchemeDoc,
    /// The scheme file's exact text.
    pub json: String,
    pub stats: Stats,
}

fn license() -> String {
    format!(
        "CC0 1.0 (public domain dedication). OpenAlex releases its data under CC0: {}",
        sources::LICENSE_URL
    )
}

fn description(lock: &Lock, version: &str) -> String {
    let mut s = format!(
        "Domains, fields, subfields and topics from the OpenAlex API ({}), snapshot {version}. \
         Hierarchy: domain (D) > field (F) > subfield (S) > topic (T); a code is the OpenAlex id \
         with a one-letter level prefix. Notes carry OpenAlex's descriptions; each topic's \
         keywords are vocabulary weighted by OpenAlex's keyword score (source openalex-keyword), \
         and alternative names of domains, fields and subfields are vocabulary too (source \
         openalex-alt). Pinned pages (sha256):",
        sources::API
    );
    for role in [Role::Domains, Role::Fields, Role::Subfields, Role::Topics] {
        for l in lock.of(role) {
            let _ = write!(s, " {} {};", l.file, l.sha256);
        }
    }
    s.push_str(" Built by schemes/openalex-topics/builder; see schemes/openalex-topics/README.md.");
    s
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

fn entities(
    lock: &Lock,
    cache: &Path,
    role: Role,
    parent: Option<&str>,
) -> Result<Vec<Entity>, String> {
    let mut out = Vec::new();
    for locked in lock.of(role) {
        let bytes = sources::read(cache, locked)?;
        out.extend(build::parse_page(&bytes, parent).map_err(|e| format!("{}: {e}", locked.file))?);
    }
    if out.is_empty() {
        return Err(format!("the lock has no {role:?} pages"));
    }
    Ok(out)
}

/// Build the scheme from the sources locked in `data/sources.lock.json`,
/// read from `cache`.
pub fn generate(data: &Path, cache: &Path) -> Result<Generated, String> {
    let lock = Lock::read(&data.join(LOCK))?;
    let license_page = lock
        .of(Role::License)
        .next()
        .ok_or("the lock has no license page")?;
    let page = sources::read(cache, license_page)?;
    sources::check_license(&String::from_utf8_lossy(&page))?;

    let domains = entities(&lock, cache, Role::Domains, None)?;
    let fields = entities(&lock, cache, Role::Fields, Some("domain"))?;
    let subfields = entities(&lock, cache, Role::Subfields, Some("field"))?;
    let topics = entities(&lock, cache, Role::Topics, Some("subfield"))?;
    let version = lock.snapshot_date()?;
    let prov = Provenance {
        description: description(&lock, &version),
        version,
        license: license(),
    };
    let (doc, stats) = build::scheme(&domains, &fields, &subfields, &topics, &prov)?;
    let json = scheme_json(&doc)?;
    Ok(Generated { doc, json, stats })
}
