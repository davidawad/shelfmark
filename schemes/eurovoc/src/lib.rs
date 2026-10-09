//! Builds the EuroVoc thesaurus as a shelfmark scheme file from the
//! Publications Office's SKOS-core distribution.
//!
//! The reuse terms for the EU Vocabularies are stated inconsistently across
//! the sources (README), so shelfmark does not bundle the data: this crate
//! fetches the zip onto the user's machine, checks it against the sha256 in
//! `sources.lock.json` and writes `eurovoc.json` there, which
//! `shelfmark scheme compile` then loads.
//!
//! Hierarchy: domain > microthesaurus > descriptors along `skos:broader`.
//! A shelfmark class has one parent, so a descriptor with several broader
//! terms is placed under one by a fixed rule ([`hierarchy`]) and the others
//! are kept in its `note`.

pub mod hierarchy;
pub mod parse;
pub mod source;

use std::fmt::Write;

use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, VocabLink, SCHEME_SCHEMA_ID};

use source::Lock;

/// A domain (`04`), a microthesaurus (`0406`) or a descriptor (`C` and its
/// numeric EuroVoc id, or the hash-style id `c_…` as is).
pub const CODE_GRAMMAR: &str = "[0-9]{2}|[0-9]{4}|C[0-9]+|c_[0-9a-f]+";
/// Vocabulary source name of every alternative label.
pub const VOCAB_SOURCE: &str = "eurovoc";

/// The scheme for the locked source `xml` (the RDF inside the zip). The zip
/// must already match the lock ([`Lock::verify`]); provenance comes from it.
pub fn scheme(xml: &[u8], lock: &Lock) -> Result<SchemeDoc, String> {
    let t = parse::thesaurus(xml)?;
    let s = &lock.source;
    let l = &lock.license;
    Ok(SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: "eurovoc".to_string(),
        version: lock.version.clone(),
        title: "EuroVoc (multilingual thesaurus of the EU Publications Office)".to_string(),
        source: Some(Source {
            description: format!(
                "English labels, domains, microthesauri and descriptors from {file} ({url}, fetched \
                 {fetched}, {bytes} bytes, sha256 {sha}). Hierarchy: domain > microthesaurus > \
                 descriptors by skos:broader; a descriptor with several broader terms sits under the \
                 one sharing a microthesaurus with it (lowest id on ties) and lists the others in its \
                 note. English alternative labels are vocabulary (source `{vs}`, source_id the \
                 EuroVoc id). Built locally by schemes/eurovoc (`eurovoc-scheme build`); see its README.",
                file = s.file,
                url = s.url,
                fetched = s.fetched,
                bytes = s.bytes,
                sha = s.sha256,
                vs = VOCAB_SOURCE,
            ),
            url: Some(source::PAGE.to_string()),
            license: Some(format!(
                "{} {} Not bundled: generated on the user's machine. {}",
                l.holder, l.terms, l.attribution
            )),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes: classes(&t)?,
    })
}

fn classes(t: &parse::Thesaurus) -> Result<Vec<ClassDef>, String> {
    let class = |code: String, label: &str, parent: Option<String>| ClassDef {
        code,
        label: label.to_string(),
        parent,
        range: None,
        vocab: Vec::new(),
        note: None,
    };
    let places = hierarchy::places(t)?;
    let mut out = Vec::with_capacity(t.domains.len() + t.mts.len() + t.terms.len());
    for d in &t.domains {
        out.push(class(d.code.clone(), &d.label, None));
        for m in t.mts.iter().filter(|m| m.domain == d.code) {
            out.push(class(m.code.clone(), &m.label, Some(d.code.clone())));
            for id in hierarchy::walk(&places, &hierarchy::Parent::Mt(m.code.clone())) {
                out.push(term_class(t, &places, &id));
            }
        }
    }
    if out.len() != t.domains.len() + t.mts.len() + t.terms.len() {
        return Err(
            "some descriptors are unreachable from a microthesaurus (broader cycle)".into(),
        );
    }
    Ok(out)
}

fn term_class(t: &parse::Thesaurus, places: &hierarchy::Places, id: &str) -> ClassDef {
    let term = places.term(t, id);
    let parent = match &places.parent_of[id] {
        hierarchy::Parent::Mt(code) => code.clone(),
        hierarchy::Parent::Term(p) => hierarchy::code(p),
    };
    ClassDef {
        code: hierarchy::code(id),
        label: term.label.clone(),
        parent: Some(parent),
        range: None,
        vocab: term
            .alt
            .iter()
            .map(|a| VocabLink {
                term: a.clone(),
                source: VOCAB_SOURCE.to_string(),
                source_id: Some(id.to_string()),
                weight: 1.0,
            })
            .collect(),
        note: hierarchy::note(t, places, id),
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
