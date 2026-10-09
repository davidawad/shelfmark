//! Turn the parsed SKOS file into a shelfmark scheme.
//!
//! A class's code is the concept's dotted path, the identifier ACM's own
//! CCSXML index terms carry, so the code alone fixes the parent (drop the
//! last segment). Classes are written in pre-order with siblings in code
//! order, independent of the file's order. A concept filed under more than
//! one parent gets a note naming its other paths. The source has no LCC or
//! LCSH mappings, so no class carries vocabulary links.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, SCHEME_SCHEMA_ID};

use crate::skos::Skos;
use crate::sources::Lock;

pub const NAME: &str = "acm-ccs";
pub const VERSION: &str = "2012";
/// A dotted path of 8-digit ACM concept IDs.
pub const CODE_GRAMMAR: &str = r"[0-9]{8}(\.[0-9]{8})*";
pub const LICENSE: &str = "Copyright ACM, Inc. ACM makes the CCS freely available for educational \
     and research purposes and grants no license to redistribute it, so shelfmark does not \
     bundle it: this file is built locally from ACM's download. Credit: ACM Computing \
     Classification System (2012), Association for Computing Machinery, https://dl.acm.org/ccs";

/// What the build saw, for the summary line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stats {
    pub classes: usize,
    pub top: usize,
    pub depth: usize,
    /// Concept IDs filed at more than one path.
    pub poly: usize,
}

pub fn build(skos: &Skos, lock: &Lock) -> Result<(SchemeDoc, Stats), String> {
    let by_path = check(skos)?;
    // Concept ID (last segment) -> every path it is filed at.
    let mut places: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for path in by_path.keys() {
        places.entry(leaf(path)).or_default().push(path);
    }
    let mut children: BTreeMap<Option<&str>, Vec<&str>> = BTreeMap::new();
    for path in by_path.keys() {
        children.entry(parent(path)).or_default().push(path);
    }
    let mut classes = Vec::with_capacity(by_path.len());
    let mut stack: Vec<&str> = children.get(&None).cloned().unwrap_or_default();
    stack.reverse();
    while let Some(path) = stack.pop() {
        let others: Vec<&str> = places[leaf(path)]
            .iter()
            .copied()
            .filter(|p| *p != path)
            .collect();
        let note = (!others.is_empty()).then(|| {
            format!(
                "ACM concept {} is also filed at: {}",
                leaf(path),
                others.join(", ")
            )
        });
        classes.push(ClassDef {
            code: path.to_string(),
            label: by_path[path].clone(),
            parent: parent(path).map(str::to_string),
            range: None,
            vocab: Vec::new(),
            note,
        });
        if let Some(kids) = children.get(&Some(path)) {
            stack.extend(kids.iter().rev());
        }
    }
    let stats = Stats {
        classes: classes.len(),
        top: skos.top.len(),
        depth: by_path
            .keys()
            .map(|p| p.split('.').count())
            .max()
            .unwrap_or(0),
        poly: places.values().filter(|p| p.len() > 1).count(),
    };
    let doc = SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: NAME.to_string(),
        version: VERSION.to_string(),
        title: "ACM Computing Classification System".to_string(),
        source: Some(Source {
            description: format!(
                "The 2012 ACM Computing Classification System from ACM's SKOS file \
                 ({file}, {bytes} bytes, sha256 {sha}, Last-Modified {lm}), linked from {page}. \
                 A class's code is the concept's dotted path of ACM concept IDs, as in ACM's \
                 CCSXML index terms; a concept filed under several parents appears once per \
                 path. The source maps no concept to LCC or LCSH. Built by \
                 schemes/acm-ccs; see its README.",
                file = lock.file,
                bytes = lock.bytes,
                sha = lock.sha256,
                lm = lock.last_modified,
                page = lock.page,
            ),
            url: Some(lock.url.clone()),
            license: Some(LICENSE.to_string()),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    };
    Ok((doc, stats))
}

fn leaf(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or(path)
}

fn parent(path: &str) -> Option<&str> {
    path.rsplit_once('.').map(|(p, _)| p)
}

fn segment_ok(s: &str) -> bool {
    s.len() == 8 && s.bytes().all(|b| b.is_ascii_digit())
}

/// Check the file's tree is the one its paths describe; return path ->
/// label. Every problem is reported, not just the first.
fn check(skos: &Skos) -> Result<BTreeMap<&str, String>, String> {
    let mut problems = Vec::new();
    let mut by_path = BTreeMap::new();
    for c in &skos.concepts {
        if !c.path.split('.').all(segment_ok) {
            problems.push(format!("{}: not a dotted path of 8-digit IDs", c.path));
        }
        if c.label.is_empty() {
            problems.push(format!("{}: no label", c.path));
        }
        if by_path.insert(c.path.as_str(), c.label.clone()).is_some() {
            problems.push(format!("{}: appears twice", c.path));
        }
        if c.broader.as_deref() != parent(&c.path) {
            problems.push(format!(
                "{}: broader is {:?}, its path says {:?}",
                c.path,
                c.broader,
                parent(&c.path)
            ));
        }
        if c.top != parent(&c.path).is_none() {
            problems.push(format!("{}: topConceptOf disagrees with its path", c.path));
        }
    }
    let narrower: BTreeSet<(&str, &str)> = skos
        .concepts
        .iter()
        .flat_map(|c| {
            c.narrower
                .iter()
                .map(move |n| (c.path.as_str(), n.as_str()))
        })
        .collect();
    let broader: BTreeSet<(&str, &str)> = skos
        .concepts
        .iter()
        .filter_map(|c| Some((c.broader.as_deref()?, c.path.as_str())))
        .collect();
    for (p, c) in narrower.symmetric_difference(&broader) {
        problems.push(format!("{p} -> {c}: narrower and broader disagree"));
    }
    let top: BTreeSet<&str> = skos.top.iter().map(String::as_str).collect();
    let roots: BTreeSet<&str> = by_path
        .keys()
        .copied()
        .filter(|p| !p.contains('.'))
        .collect();
    if top != roots || top.len() != skos.top.len() {
        problems.push(format!(
            "hasTopConcept {top:?} does not match the top-level concepts {roots:?}"
        ));
    }
    if by_path.is_empty() {
        problems.push("no concepts".to_string());
    }
    if problems.is_empty() {
        return Ok(by_path);
    }
    let mut msg = format!("{} problems in the SKOS file:", problems.len());
    for p in problems.iter().take(20) {
        let _ = write!(msg, "\n  {p}");
    }
    Err(msg)
}

/// The scheme as JSON: header fields pretty-printed, then one class per
/// line, so two builds diff line by line.
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
