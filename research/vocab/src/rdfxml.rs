//! Targeted RDF/XML reader for id.worldcat.org per-record FAST documents.
//!
//! Handles the shapes those documents use: node elements carrying
//! `rdf:about` (resolved against `xml:base`), property elements with either
//! `rdf:resource`, a nested node element with `rdf:about`, or text content.
//! It is not a general RDF/XML parser (no `rdf:parseType`, reification or
//! container membership); anything unrecognised is skipped, never guessed.

use std::io::BufRead;

use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;

use crate::ntriples::{Term, Triple};

const RDF: &[u8] = b"http://www.w3.org/1999/02/22-rdf-syntax-ns#";

/// Property element being read at depth 3.
struct Prop {
    predicate: String,
    lang: Option<String>,
    object: Option<String>,
    text: String,
}

#[derive(Default)]
struct State {
    base: String,
    depth: usize,
    subject: Option<String>,
    prop: Option<Prop>,
    out: Vec<Triple>,
}

pub fn read_triples(input: impl BufRead) -> Result<Vec<Triple>, String> {
    let mut reader = NsReader::from_reader(input);
    let mut buf = Vec::new();
    let mut st = State::default();
    loop {
        let (ns, event) = reader
            .read_resolved_event_into(&mut buf)
            .map_err(|e| e.to_string())?;
        let ns = match ns {
            ResolveResult::Bound(n) => n.as_ref().to_vec(),
            _ => Vec::new(),
        };
        match event {
            Event::Eof => break,
            Event::Start(e) => {
                st.depth += 1;
                st.open(&reader, &e, &ns, false)?;
            }
            Event::Empty(e) => {
                st.depth += 1;
                st.open(&reader, &e, &ns, true)?;
                st.depth -= 1;
            }
            Event::Text(t) if st.depth == 3 => {
                if let Some(p) = st.prop.as_mut() {
                    p.text.push_str(&t.unescape().map_err(|e| e.to_string())?);
                }
            }
            Event::End(_) => st.close(),
            _ => {}
        }
        buf.clear();
    }
    Ok(st.out)
}

impl State {
    fn open<R>(
        &mut self,
        reader: &NsReader<R>,
        e: &BytesStart,
        ns: &[u8],
        empty: bool,
    ) -> Result<(), String> {
        let mut about = None;
        let mut resource = None;
        let mut lang = None;
        for attr in e.attributes().flatten() {
            let value = attr
                .unescape_value()
                .map_err(|e| e.to_string())?
                .into_owned();
            let (attr_ns, local) = reader.resolve_attribute(attr.key);
            let in_rdf = matches!(attr_ns, ResolveResult::Bound(n) if n.as_ref() == RDF);
            match (in_rdf, attr.key.as_ref(), local.as_ref()) {
                (true, _, b"about") => about = Some(value),
                (true, _, b"resource") => resource = Some(value),
                (false, b"xml:base", _) => self.base = value,
                (false, b"xml:lang", _) => lang = Some(value.to_ascii_lowercase()),
                _ => {}
            }
        }
        match self.depth {
            2 => self.subject = about.map(|a| resolve(&self.base, &a)),
            3 if self.subject.is_some() => {
                let predicate = format!(
                    "{}{}",
                    String::from_utf8_lossy(ns),
                    String::from_utf8_lossy(e.local_name().as_ref())
                );
                let object = resource.map(|r| resolve(&self.base, &r));
                self.prop = Some(Prop {
                    predicate,
                    lang,
                    object,
                    text: String::new(),
                });
                if empty {
                    self.emit();
                }
            }
            4 => {
                if let (Some(p), Some(a)) = (self.prop.as_mut(), about) {
                    p.object = Some(resolve(&self.base, &a));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn close(&mut self) {
        match self.depth {
            3 => self.emit(),
            2 => self.subject = None,
            _ => {}
        }
        self.depth = self.depth.saturating_sub(1);
    }

    fn emit(&mut self) {
        let (Some(s), Some(p)) = (&self.subject, self.prop.take()) else {
            return;
        };
        let object = match p.object {
            Some(iri) => Term::Iri(iri),
            None => Term::Literal {
                value: p.text,
                lang: p.lang,
                datatype: None,
            },
        };
        self.out.push(triple(s, p.predicate, object));
    }
}

fn triple(subject: &str, predicate: String, object: Term) -> Triple {
    Triple {
        subject: Term::Iri(subject.to_string()),
        predicate,
        object,
    }
}

fn resolve(base: &str, reference: &str) -> String {
    if reference.contains("://") || reference.starts_with("info:") {
        reference.to_string()
    } else {
        format!("{base}{reference}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
        xmlns:skos="http://www.w3.org/2004/02/skos/core#" xmlns:schema="http://schema.org/"
        xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#" xml:base="http://id.worldcat.org/fast/">
      <rdf:Description rdf:about="1001522">
        <skos:inScheme rdf:resource="ontology/1.0/#facet-Topical"/>
        <skos:prefLabel>Local times &amp; more</skos:prefLabel>
        <skos:broader><rdf:Description rdf:about="1133519"><rdfs:label>x</rdfs:label></rdf:Description></skos:broader>
        <schema:sameAs><rdf:Description rdf:about="http://id.loc.gov/authorities/subjects/sh99005510"/></schema:sameAs>
      </rdf:Description>
    </rdf:RDF>"#;

    #[test]
    fn reads_fast_record_shape() {
        let t = read_triples(DOC.as_bytes()).unwrap();
        let get = |p: &str| {
            t.iter()
                .find(|t| t.predicate.ends_with(p))
                .map(|t| t.object.clone())
                .unwrap()
        };
        assert!(t
            .iter()
            .all(|t| t.subject == Term::Iri("http://id.worldcat.org/fast/1001522".into())));
        assert_eq!(get("prefLabel").as_literal(), Some("Local times & more"));
        assert_eq!(
            get("broader").as_iri(),
            Some("http://id.worldcat.org/fast/1133519")
        );
        assert_eq!(
            get("sameAs").as_iri(),
            Some("http://id.loc.gov/authorities/subjects/sh99005510")
        );
        assert_eq!(
            get("inScheme").as_iri(),
            Some("http://id.worldcat.org/fast/ontology/1.0/#facet-Topical")
        );
        assert_eq!(t.len(), 4);
    }
}
