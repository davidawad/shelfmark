//! Builds the JEL classification (economics) as a shelfmark scheme file
//! from the American Economic Association's `classificationTree.xml`.
//!
//! The AEA does not license the codes for redistribution (README), so
//! shelfmark never bundles them: this crate fetches the XML onto the
//! user's machine, checks it against the sha256 in `sources.lock.json` and
//! writes `jel.json` there, which `shelfmark scheme compile` then loads.

pub mod parse;
pub mod source;

use std::fmt::Write;

use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, SCHEME_SCHEMA_ID};

use source::Lock;

/// Every JEL code: a letter, then one digit per level below it.
pub const CODE_GRAMMAR: &str = "[A-Z]([0-9]{1,2})?";

/// The scheme for the locked source `xml`. `xml` must already match the
/// lock ([`Lock::verify`]); the provenance written here comes from it.
pub fn scheme(xml: &[u8], lock: &Lock) -> Result<SchemeDoc, String> {
    let entries = parse::entries(xml)?;
    let classes = entries
        .into_iter()
        .map(|e| ClassDef {
            code: e.code,
            label: e.label,
            parent: e.parent,
            range: None,
            vocab: Vec::new(),
            note: None,
        })
        .collect();
    let s = &lock.source;
    let l = &lock.license;
    Ok(SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: "jel".to_string(),
        version: lock.version.clone(),
        title: "JEL Classification System (Journal of Economic Literature)".to_string(),
        source: Some(Source {
            description: format!(
                "Categories, subcategories and codes from the AEA's {file} ({url}, fetched {fetched}, \
                 {bytes} bytes, sha256 {sha}); parents from its nesting, labels from its descriptions \
                 with HTML entities decoded and bullet separators written as `; `. The source carries \
                 no LCC or LCSH links, so no class has vocabulary. Built locally by \
                 src/shelfmark/schemes/jel (`jel-scheme build`); see its README.",
                file = s.file,
                url = s.url,
                fetched = s.fetched,
                bytes = s.bytes,
                sha = s.sha256,
            ),
            url: Some(source::PAGE.to_string()),
            license: Some(format!(
                "{} {} Not redistributable: generated on the user's machine, never bundled. {}",
                l.holder, l.terms, l.attribution
            )),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    })
}

/// The scheme as JSON: header fields pretty-printed, then one class per
/// line (the layout of the bundled `lcc.json`), so a rebuild diffs line by
/// line.
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
