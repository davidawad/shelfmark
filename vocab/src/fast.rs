//! FAST ingest from OCLC triples (bulk `FAST*.nt` or per-record RDF/XML).
//!
//! FAST records carry no LCC numbers of their own. They carry `schema:sameAs`
//! links to the LCSH heading each was derived from, so LCC is joined through
//! the LCSH table and marked `lcc_source = lcsh`.
//!
//! The bulk N-Triples are not grouped by subject, so headings accumulate in a
//! map keyed by FAST id and are emitted sorted when the input ends.

use std::collections::{BTreeMap, HashMap};
use std::io::BufRead;

use serde::Serialize;

use crate::ntriples::{parse_line, Triple};
use crate::table::Heading;

const SKOS: &str = "http://www.w3.org/2004/02/skos/core#";
const LCSH: &str = "http://id.loc.gov/authorities/subjects/";
const SAME_AS: &[&str] = &[
    "http://schema.org/sameAs",
    "http://www.w3.org/2002/07/owl#sameAs",
    "http://www.w3.org/2004/02/skos/core#exactMatch",
    "http://www.w3.org/2004/02/skos/core#closeMatch",
];

#[derive(Debug, Default, Serialize)]
pub struct FastStats {
    pub triples: u64,
    pub parse_errors: u64,
    pub subjects: u64,
    /// Subjects without a prefLabel (deprecated `fstNNN` aliases and the like).
    pub subjects_without_label: u64,
    pub headings: u64,
    pub by_kind: BTreeMap<String, u64>,
    pub with_alt_labels: u64,
    pub with_broader: u64,
    pub with_lcsh_link: u64,
    /// LCSH links whose target exists in the LCSH table given with `--lcsh`.
    pub lcsh_links_resolved: u64,
    pub lcsh_links_unresolved: u64,
    /// Headings that inherit at least one LCC number through an LCSH link.
    pub with_lcc_via_lcsh: u64,
    pub lcsh_table_loaded: bool,
}

#[derive(Default)]
pub struct Builder {
    headings: BTreeMap<String, Heading>,
    pub stats: FastStats,
}

/// `fst01001522` from `http://id.worldcat.org/fast/1001522` (or `/fst01001522`).
pub fn fast_id(iri: &str) -> Option<String> {
    let tail = iri
        .strip_prefix("http://id.worldcat.org/fast/")
        .or_else(|| iri.strip_prefix("https://id.worldcat.org/fast/"))?;
    let digits = tail.strip_prefix("fst").unwrap_or(tail);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u64 = digits.parse().ok()?;
    Some(format!("fst{n:08}"))
}

impl Builder {
    pub fn add_ntriples(&mut self, input: impl BufRead) -> std::io::Result<()> {
        let mut input = input;
        let mut buf = Vec::new();
        loop {
            buf.clear();
            if input.read_until(b'\n', &mut buf)? == 0 {
                return Ok(());
            }
            match parse_line(String::from_utf8_lossy(&buf).trim_end()) {
                Ok(Some(t)) => self.add(&t),
                Ok(None) => {}
                Err(_) => self.stats.parse_errors += 1,
            }
        }
    }

    pub fn add(&mut self, t: &Triple) {
        self.stats.triples += 1;
        let Some(id) = t.subject.as_iri().and_then(fast_id) else {
            return;
        };
        let h = self.headings.entry(id.clone()).or_insert_with(|| Heading {
            id,
            vocab: "fast".into(),
            ..Heading::default()
        });
        let p = t.predicate.as_str();
        if let Some(local) = p.strip_prefix(SKOS) {
            match local {
                "prefLabel" => {
                    if let Some(v) = english(t) {
                        h.pref_label = v.to_string();
                    }
                }
                "altLabel" => {
                    if let Some(v) = english(t) {
                        h.alt_labels.push(v.to_string());
                    }
                }
                "broader" => {
                    if let Some(b) = t.object.as_iri().and_then(fast_id) {
                        h.broader.push(b);
                    }
                }
                "inScheme" => {
                    let facet = t.object.as_iri().and_then(|i| i.rsplit_once("#facet-"));
                    if let Some((_, facet)) = facet {
                        h.kind = facet.to_string();
                    }
                }
                _ => {}
            }
        }
        if SAME_AS.contains(&p) {
            if let Some(sh) = t.object.as_iri().and_then(|i| i.strip_prefix(LCSH)) {
                h.same_as.push(sh.to_string());
            }
        }
    }

    /// Emit sorted headings, joining LCC through `lcsh` (id -> LCC cells).
    pub fn finish(
        mut self,
        lcsh: Option<&HashMap<String, Vec<String>>>,
    ) -> (Vec<Heading>, FastStats) {
        let mut stats = std::mem::take(&mut self.stats);
        stats.lcsh_table_loaded = lcsh.is_some();
        let mut out = Vec::new();
        for (_, mut h) in self.headings {
            stats.subjects += 1;
            if h.pref_label.is_empty() {
                stats.subjects_without_label += 1;
                continue;
            }
            if h.kind.is_empty() {
                h.kind = "Unknown".into();
            }
            h.canonicalize();
            if let Some(table) = lcsh {
                for sh in &h.same_as {
                    match table.get(sh) {
                        Some(codes) => {
                            stats.lcsh_links_resolved += 1;
                            h.lcc.extend(codes.iter().cloned());
                        }
                        None => stats.lcsh_links_unresolved += 1,
                    }
                }
                h.canonicalize();
                if !h.lcc.is_empty() {
                    h.lcc_source = "lcsh".into();
                    stats.with_lcc_via_lcsh += 1;
                }
            }
            stats.headings += 1;
            *stats.by_kind.entry(h.kind.clone()).or_default() += 1;
            stats.with_alt_labels += u64::from(!h.alt_labels.is_empty());
            stats.with_broader += u64::from(!h.broader.is_empty());
            stats.with_lcsh_link += u64::from(!h.same_as.is_empty());
            out.push(h);
        }
        (out, stats)
    }
}

fn english(t: &Triple) -> Option<&str> {
    match t.object.lang() {
        None | Some("en") => t.object.as_literal(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_fast_ids() {
        assert_eq!(
            fast_id("http://id.worldcat.org/fast/1001522").as_deref(),
            Some("fst01001522")
        );
        assert_eq!(
            fast_id("http://id.worldcat.org/fast/fst01001522").as_deref(),
            Some("fst01001522")
        );
        assert_eq!(fast_id("http://id.worldcat.org/fast/ontology/1.0/"), None);
        assert_eq!(fast_id("http://example.org/1"), None);
    }

    #[test]
    fn joins_lcc_through_lcsh() {
        let nt = r#"<http://id.worldcat.org/fast/872540> <http://www.w3.org/2004/02/skos/core#prefLabel> "Compilers (Computer programs)" .
<http://id.worldcat.org/fast/872540> <http://www.w3.org/2004/02/skos/core#altLabel> "Compiling programs (Computer programs)" .
<http://id.worldcat.org/fast/872540> <http://schema.org/sameAs> <http://id.loc.gov/authorities/subjects/sh86007588> .
<http://id.worldcat.org/fast/872540> <http://www.w3.org/2004/02/skos/core#inScheme> <http://id.worldcat.org/fast/ontology/1.0/#facet-Topical> .
<http://id.worldcat.org/fast/fst00872540> <http://www.w3.org/2000/01/rdf-schema#comment> "deprecated alias" .
<http://id.worldcat.org/fast/9> <http://www.w3.org/2000/01/rdf-schema#comment> "no label" .
"#;
        let mut b = Builder::default();
        b.add_ntriples(nt.as_bytes()).unwrap();
        let lcsh = HashMap::from([("sh86007588".to_string(), vec!["QA76.76.C65".to_string()])]);
        let (rows, stats) = b.finish(Some(&lcsh));
        assert_eq!(rows.len(), 1);
        let h = &rows[0];
        assert_eq!(h.id, "fst00872540");
        assert_eq!(h.kind, "Topical");
        assert_eq!(h.lcc, ["QA76.76.C65"]);
        assert_eq!(h.lcc_source, "lcsh");
        assert_eq!(stats.subjects_without_label, 1);
        assert_eq!(stats.with_lcc_via_lcsh, 1);
    }
}
