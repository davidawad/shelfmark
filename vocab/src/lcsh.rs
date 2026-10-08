//! LCSH ingest from the id.loc.gov `subjects.madsrdf.nt.gz` bulk dump.
//!
//! The dump wraps every authority in `# BEGIN /authorities/subjects/<id>` ...
//! `# END` comments, and blank-node labels are only unique inside a block, so
//! we parse one block at a time. Per heading we read:
//!
//! - `madsrdf:authoritativeLabel` (or `variantLabel` on deprecated records)
//! - `madsrdf:hasVariant` -> `madsrdf:variantLabel` as alt labels
//! - `madsrdf:hasBroaderAuthority` as broader
//! - `madsrdf:classification`, either a blank `lcc:ClassNumber` node with
//!   `madsrdf:code`, or an `id.loc.gov/authorities/classification/<code>` IRI
//! - `madsrdf:useInstead` on deprecated records (stored in `same_as`)
//!
//! `madsrdf:code` is also used for MARC geographic area codes (`n-us-ca`), so a
//! code only counts as LCC when it hangs off `madsrdf:classification`.

use std::collections::{BTreeMap, HashMap};
use std::io::BufRead;

use serde::Serialize;

use crate::lcc;
use crate::ntriples::{parse_line, Term, Triple};
use crate::table::Heading;

const SUBJECTS: &str = "http://id.loc.gov/authorities/subjects/";
const MADS: &str = "http://www.loc.gov/mads/rdf/v1#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// Kind precedence when a record carries several MADS types.
const KINDS: &[&str] = &[
    "ComplexSubject",
    "Geographic",
    "HierarchicalGeographic",
    "GenreForm",
    "Temporal",
    "CorporateName",
    "ConferenceName",
    "FamilyName",
    "PersonalName",
    "NameTitle",
    "Title",
    "Topic",
];

#[derive(Debug, Default, Serialize)]
pub struct LcshStats {
    pub lines: u64,
    pub parse_errors: u64,
    pub triples_outside_blocks: u64,
    pub records: u64,
    pub records_without_label: u64,
    pub headings: u64,
    pub deprecated: u64,
    pub by_kind: BTreeMap<String, u64>,
    /// Headings (any status) with at least one LCC link.
    pub with_lcc: u64,
    /// Non-deprecated headings with at least one LCC link: the headline number.
    pub active_with_lcc: u64,
    pub active_with_lcc_by_kind: BTreeMap<String, u64>,
    pub lcc_links: u64,
    pub lcc_ranges: u64,
    pub lcc_from_class_number_node: u64,
    pub lcc_from_classification_iri: u64,
    pub lcc_distinct_codes: u64,
    pub lcc_by_first_letter: BTreeMap<String, u64>,
    pub lcc_rejected: u64,
    pub lcc_rejected_examples: Vec<String>,
    pub with_alt_labels: u64,
    pub alt_labels: u64,
    pub with_broader: u64,
}

pub struct Ingest {
    pub headings: Vec<Heading>,
    pub stats: LcshStats,
}

pub fn ingest(input: impl BufRead) -> std::io::Result<Ingest> {
    let mut stats = LcshStats::default();
    let mut headings = Vec::new();
    let mut block: Option<(String, Vec<Triple>)> = None;
    let mut input = input;
    let mut buf = Vec::new();
    loop {
        buf.clear();
        if input.read_until(b'\n', &mut buf)? == 0 {
            break;
        }
        stats.lines += 1;
        let line = String::from_utf8_lossy(&buf);
        let line = line.trim_end();
        if let Some(path) = line.strip_prefix("# BEGIN /authorities/subjects/") {
            block = Some((path.trim().to_string(), Vec::new()));
            continue;
        }
        if line.starts_with("# END") {
            if let Some((id, triples)) = block.take() {
                stats.records += 1;
                match build(&id, &triples, &mut stats) {
                    Some(h) => headings.push(h),
                    None => stats.records_without_label += 1,
                }
            }
            continue;
        }
        match parse_line(line) {
            Ok(Some(t)) => match block.as_mut() {
                Some((_, triples)) => triples.push(t),
                None => stats.triples_outside_blocks += 1,
            },
            Ok(None) => {}
            Err(_) => stats.parse_errors += 1,
        }
    }
    headings.sort_by(|a, b| a.id.cmp(&b.id));
    finish_stats(&headings, &mut stats);
    Ok(Ingest { headings, stats })
}

fn key(t: &Term) -> Option<String> {
    match t {
        Term::Iri(s) => Some(format!("<{s}")),
        Term::Blank(s) => Some(format!("_{s}")),
        Term::Literal { .. } => None,
    }
}

/// Turn one record block into a heading; `None` when it has no usable label.
pub fn build(id: &str, triples: &[Triple], stats: &mut LcshStats) -> Option<Heading> {
    let mut by_subject: HashMap<String, Vec<(&str, &Term)>> = HashMap::new();
    for t in triples {
        if let Some(k) = key(&t.subject) {
            by_subject
                .entry(k)
                .or_default()
                .push((t.predicate.as_str(), &t.object));
        }
    }
    let empty = Vec::new();
    let props_of = |t: &Term| key(t).and_then(|k| by_subject.get(&k)).unwrap_or(&empty);
    let me = Term::Iri(format!("{SUBJECTS}{id}"));
    let props = props_of(&me);

    let mut h = Heading {
        id: id.to_string(),
        vocab: "lcsh".into(),
        ..Heading::default()
    };
    let mut types = Vec::new();
    let mut variant_label = None;
    for &(p, o) in props {
        let Some(local) = p.strip_prefix(MADS) else {
            if p == RDF_TYPE {
                if let Some(t) = o.as_iri().and_then(|t| t.strip_prefix(MADS)) {
                    types.push(t);
                }
            }
            continue;
        };
        match local {
            "authoritativeLabel" => {
                if let Some(v) = english(o) {
                    h.pref_label = v.to_string();
                }
            }
            "variantLabel" => variant_label = english(o).map(str::to_string),
            "hasVariant" => {
                for &(vp, vo) in props_of(o) {
                    if vp == format!("{MADS}variantLabel") {
                        if let Some(v) = english(vo) {
                            h.alt_labels.push(v.to_string());
                        }
                    }
                }
            }
            "hasBroaderAuthority" => {
                if let Some(b) = o.as_iri().and_then(|i| i.strip_prefix(SUBJECTS)) {
                    h.broader.push(b.to_string());
                }
            }
            "useInstead" => {
                if let Some(b) = o.as_iri().and_then(|i| i.strip_prefix(SUBJECTS)) {
                    h.same_as.push(b.to_string());
                }
            }
            "classification" => classification(o, props_of(o), &mut h, stats),
            _ => {}
        }
    }
    h.deprecated = types.contains(&"DeprecatedAuthority");
    h.kind = KINDS
        .iter()
        .find(|k| types.contains(k))
        .map(|k| k.to_string())
        .unwrap_or_else(|| "Other".into());
    if h.pref_label.is_empty() {
        h.pref_label = variant_label?;
    }
    h.canonicalize();
    if !h.lcc.is_empty() {
        h.lcc_source = "direct".into();
    }
    Some(h)
}

fn english(t: &Term) -> Option<&str> {
    match t.lang() {
        None | Some("en") => t.as_literal(),
        _ => None,
    }
}

fn classification(o: &Term, node: &[(&str, &Term)], h: &mut Heading, stats: &mut LcshStats) {
    if let Some(iri) = o.as_iri() {
        match lcc::from_classification_iri(iri) {
            Some(r) => {
                stats.lcc_from_classification_iri += 1;
                h.lcc.push(r.cell());
            }
            None => reject(stats, iri),
        }
        return;
    }
    for &(p, v) in node {
        if p != format!("{MADS}code") {
            continue;
        }
        let Some(raw) = v.as_literal() else { continue };
        match lcc::normalize(raw) {
            Some(r) => {
                stats.lcc_from_class_number_node += 1;
                h.lcc.push(r.cell());
            }
            None => reject(stats, raw),
        }
    }
}

fn reject(stats: &mut LcshStats, raw: &str) {
    stats.lcc_rejected += 1;
    if stats.lcc_rejected_examples.len() < 20 {
        stats.lcc_rejected_examples.push(raw.to_string());
    }
}

fn finish_stats(headings: &[Heading], stats: &mut LcshStats) {
    let mut distinct = std::collections::BTreeSet::new();
    for h in headings {
        stats.headings += 1;
        *stats.by_kind.entry(h.kind.clone()).or_default() += 1;
        if h.deprecated {
            stats.deprecated += 1;
        }
        if !h.alt_labels.is_empty() {
            stats.with_alt_labels += 1;
            stats.alt_labels += h.alt_labels.len() as u64;
        }
        if !h.broader.is_empty() {
            stats.with_broader += 1;
        }
        if h.lcc.is_empty() {
            continue;
        }
        stats.with_lcc += 1;
        if !h.deprecated {
            stats.active_with_lcc += 1;
            *stats
                .active_with_lcc_by_kind
                .entry(h.kind.clone())
                .or_default() += 1;
        }
        for code in &h.lcc {
            stats.lcc_links += 1;
            if code.contains('-') {
                stats.lcc_ranges += 1;
            }
            let first = code.chars().next().map(String::from).unwrap_or_default();
            *stats.lcc_by_first_letter.entry(first).or_default() += 1;
            distinct.insert(code.clone());
        }
    }
    stats.lcc_distinct_codes = distinct.len() as u64;
}
