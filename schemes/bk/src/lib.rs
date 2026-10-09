//! Builds the Basic Classification (Basisklassifikation, BK) as a shelfmark
//! scheme file from the GBV's JSKOS export.
//!
//! The BK is CC0 (README), so the built `bk.json` is committed. This crate
//! rebuilds it: it fetches the export, checks it against the sha256 in
//! `sources.lock.json` and writes `bk.json`, which `shelfmark scheme
//! compile` then loads.

pub mod parse;
pub mod source;

use std::collections::BTreeSet;
use std::fmt::Write;

use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, VocabLink, SCHEME_SCHEMA_ID};

use source::Lock;

/// Every BK code: one or two digits, optionally a range of two such
/// numbers (`3-4`), then optionally a point and two digits (`42.82`), with
/// one trailing `X` in the single case the BK has (`74.50X`).
pub const CODE_GRAMMAR: &str = "[0-9]{1,2}(-[0-9]{1,2})?(\\.[0-9]{2}X?)?";

/// Vocabulary source token for the English preferred label.
pub const SRC_EN: &str = "bk-en";
/// Vocabulary source token for German alternative labels.
pub const SRC_ALT: &str = "bk-alt";
/// English labels are translations, so they weigh a little under the
/// German label.
pub const WEIGHT_EN: f64 = 0.9;
pub const WEIGHT_ALT: f64 = 0.8;

/// The scheme for the locked source `ndjson`. `ndjson` must already match
/// the lock ([`Lock::verify`]); the provenance written here comes from it.
pub fn scheme(ndjson: &[u8], lock: &Lock) -> Result<SchemeDoc, String> {
    let classes = parse::entries(ndjson)?.into_iter().map(class).collect();
    let s = &lock.source;
    let l = &lock.license;
    Ok(SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: "bk".to_string(),
        version: lock.version.clone(),
        title: "Basic Classification (Basisklassifikation, BK)".to_string(),
        source: Some(Source {
            description: format!(
                "Concepts from the GBV's full JSKOS export of the BK ({url}, fetched {fetched}, \
                 {bytes} bytes, sha256 {sha}); parents from each concept's ancestors, labels \
                 are the German preferred labels, English preferred labels and German \
                 alternative labels are vocabulary terms (sources `{en}` and `{alt}`), and \
                 scope notes, definitions and cross-references are class notes. The source \
                 carries no LCC or LCSH links. Built by schemes/bk (`bk-scheme build`); see \
                 its README.",
                url = s.url,
                fetched = s.fetched,
                bytes = s.bytes,
                sha = s.sha256,
                en = SRC_EN,
                alt = SRC_ALT,
            ),
            url: Some(source::PAGE.to_string()),
            license: Some(format!("{} {} {}", l.holder, l.terms, l.attribution)),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    })
}

fn class(e: parse::Entry) -> ClassDef {
    let mut seen = BTreeSet::from([e.label.to_lowercase()]);
    let mut vocab = Vec::new();
    let terms = e
        .label_en
        .iter()
        .map(|t| (t, SRC_EN, WEIGHT_EN))
        .chain(e.alt.iter().map(|t| (t, SRC_ALT, WEIGHT_ALT)));
    for (term, source, weight) in terms {
        if seen.insert(term.to_lowercase()) {
            vocab.push(VocabLink {
                term: term.clone(),
                source: source.to_string(),
                source_id: None,
                weight,
            });
        }
    }
    ClassDef {
        code: e.code,
        label: e.label,
        parent: e.parent,
        range: None,
        vocab,
        note: e.note,
    }
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
