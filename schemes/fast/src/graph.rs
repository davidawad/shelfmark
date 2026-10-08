//! The FAST headings in one N-Triples file, read into plain records.
//!
//! Kept per heading: preferred and alternate labels, `skos:broader`,
//! `schema:sameAs` links to LCSH, deprecation and `dcterms:isReplacedBy`.
//! Kept globally: the `rdfs:label` the dump gives each LCSH IRI it links.
//! Everything else (GND, BnF, Wikidata and Wikipedia links, comments) is
//! dropped.

use std::collections::BTreeMap;
use std::io::BufRead;

use crate::nt::{self, Object};

pub const FAST: &str = "http://id.worldcat.org/fast/";
pub const LCSH: &str = "http://id.loc.gov/authorities/subjects/";
const SKOS: &str = "http://www.w3.org/2004/02/skos/core#";

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Heading {
    pub label: Option<String>,
    pub alts: Vec<String>,
    pub broader: Vec<u32>,
    /// LCSH record IDs, e.g. `sh85008315`.
    pub lcsh: Vec<String>,
    pub deprecated: bool,
    pub replaced_by: Vec<u32>,
}

#[derive(Debug, Default)]
pub struct Graph {
    pub headings: BTreeMap<u32, Heading>,
    /// LCSH record ID -> label as the dump gives it.
    pub lcsh_labels: BTreeMap<String, String>,
    pub triples: usize,
}

/// The FAST number in a heading IRI, e.g. 817698 in
/// `http://id.worldcat.org/fast/817698`.
pub fn fast_number(iri: &str) -> Option<u32> {
    let n = iri.strip_prefix(FAST)?;
    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    n.parse().ok()
}

fn lcsh_id(iri: &str) -> Option<&str> {
    let id = iri.strip_prefix(LCSH)?;
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric())).then_some(id)
}

impl Graph {
    /// Read one N-Triples file; call [`Graph::finish`] after the last.
    pub fn read(input: impl BufRead) -> Result<Graph, String> {
        let mut g = Graph::default();
        g.read_into(input)?;
        g.finish();
        Ok(g)
    }

    pub fn read_into(&mut self, input: impl BufRead) -> Result<(), String> {
        for (n, line) in input.lines().enumerate() {
            let line = line.map_err(|e| format!("line {}: {e}", n + 1))?;
            if let Some(t) = nt::parse_line(&line).map_err(|e| format!("line {}: {e}", n + 1))? {
                self.triples += 1;
                self.add(t);
            }
        }
        Ok(())
    }

    /// Sort and dedup every list, so input order never shows.
    pub fn finish(&mut self) {
        for h in self.headings.values_mut() {
            for v in [&mut h.broader, &mut h.replaced_by] {
                v.sort_unstable();
                v.dedup();
            }
            h.alts.sort();
            h.alts.dedup();
            h.lcsh.sort();
            h.lcsh.dedup();
        }
    }

    fn add(&mut self, t: nt::Triple) {
        let Some(id) = fast_number(&t.subject) else {
            if let (Some(sh), Object::Literal(l)) = (lcsh_id(&t.subject), &t.object) {
                if t.predicate == "http://www.w3.org/2000/01/rdf-schema#label" {
                    self.lcsh_labels.insert(sh.to_string(), l.clone());
                }
            }
            return;
        };
        let h = self.headings.entry(id).or_default();
        let pred = t.predicate.as_str();
        match (pred.strip_prefix(SKOS), t.object) {
            (Some("prefLabel"), Object::Literal(l)) => h.label = Some(l),
            (Some("altLabel"), Object::Literal(l)) => h.alts.push(l),
            (Some("broader"), Object::Iri(o)) => h.broader.extend(fast_number(&o)),
            (None, Object::Iri(o)) if pred == "http://schema.org/sameAs" => {
                h.lcsh.extend(lcsh_id(&o).map(String::from))
            }
            (None, Object::Iri(o)) if pred == "http://purl.org/dc/terms/isReplacedBy" => {
                h.replaced_by.extend(fast_number(&o))
            }
            (None, Object::Literal(l)) if pred == "http://www.w3.org/2002/07/owl#deprecated" => {
                h.deprecated |= l == "true"
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_come_only_from_heading_iris() {
        assert_eq!(
            fast_number("http://id.worldcat.org/fast/817698"),
            Some(817698)
        );
        assert_eq!(
            fast_number("http://id.worldcat.org/fast/ontology/1.0/#fast"),
            None
        );
        assert_eq!(fast_number("http://id.worldcat.org/fast/"), None);
        assert_eq!(
            lcsh_id("http://id.loc.gov/authorities/subjects/sh85008315"),
            Some("sh85008315")
        );
        assert_eq!(lcsh_id("http://d-nb.info/gnd/4143284-8"), None);
    }
}
