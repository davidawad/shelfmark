//! Read the ACM CCS SKOS/RDF-XML file into concepts.
//!
//! The file is one `skos:ConceptScheme` naming the top concepts, then one
//! `skos:Concept` per position in the tree. A concept's `rdf:about` is its
//! dotted path of 8-digit concept IDs (`10002944.10011122`), so a concept
//! filed under two parents (the 2012 CCS is poly-hierarchical) appears once
//! per path. Each concept has an English `prefLabel`, at most one `broader`
//! and any number of `narrower`.

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

/// One `skos:Concept`, as written in the file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Concept {
    /// `rdf:about`: the dotted path of concept IDs.
    pub path: String,
    pub label: String,
    pub broader: Option<String>,
    pub narrower: Vec<String>,
    /// Has `skos:topConceptOf`.
    pub top: bool,
}

/// The parsed file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Skos {
    /// `rdf:about` of the `skos:ConceptScheme`.
    pub scheme: String,
    /// The scheme's `hasTopConcept` targets, in file order.
    pub top: Vec<String>,
    /// Every concept, in file order.
    pub concepts: Vec<Concept>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum In {
    Nothing,
    Scheme,
    Concept,
    Label,
}

pub fn parse(bytes: &[u8]) -> Result<Skos, String> {
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);
    let mut out = Skos::default();
    let mut state = In::Nothing;
    let mut concept = Concept::default();
    let mut buf = Vec::new();
    loop {
        let at = reader.buffer_position();
        let ev = reader
            .read_event_into(&mut buf)
            .map_err(|e| format!("XML error at byte {at}: {e}"))?;
        let err = |msg: String| format!("byte {at}: {msg}");
        let empty = matches!(ev, Event::Empty(_));
        match ev {
            Event::Start(e) | Event::Empty(e) => {
                let name = e.name();
                match (state, name.as_ref()) {
                    (In::Nothing, b"skos:ConceptScheme") => {
                        out.scheme = attr(&e, "rdf:about").map_err(err)?;
                        state = In::Scheme;
                    }
                    (In::Scheme, b"skos:hasTopConcept") => {
                        out.top.push(attr(&e, "rdf:resource").map_err(err)?)
                    }
                    (In::Nothing, b"skos:Concept") => {
                        concept = Concept {
                            path: attr(&e, "rdf:about").map_err(err)?,
                            ..Concept::default()
                        };
                        state = In::Concept;
                    }
                    (In::Concept, b"skos:prefLabel") => {
                        let lang = attr(&e, "lang")
                            .or_else(|_| attr(&e, "xml:lang"))
                            .unwrap_or_default();
                        if lang != "en" {
                            return Err(err(format!(
                                "{}: prefLabel in language `{lang}`, expected `en`",
                                concept.path
                            )));
                        }
                        if !concept.label.is_empty() {
                            return Err(err(format!("{}: two prefLabels", concept.path)));
                        }
                        if !empty {
                            state = In::Label;
                        }
                    }
                    (In::Concept, b"skos:broader") => {
                        let b = attr(&e, "rdf:resource").map_err(err)?;
                        if concept.broader.replace(b).is_some() {
                            return Err(err(format!("{}: two broader concepts", concept.path)));
                        }
                    }
                    (In::Concept, b"skos:narrower") => concept
                        .narrower
                        .push(attr(&e, "rdf:resource").map_err(err)?),
                    (In::Concept, b"skos:topConceptOf") => concept.top = true,
                    (In::Nothing, b"rdf:RDF") => {}
                    (_, other) => {
                        return Err(err(format!(
                            "unexpected element <{}>",
                            String::from_utf8_lossy(other)
                        )))
                    }
                }
                if empty && matches!(state, In::Scheme | In::Concept) && is_container(&e) {
                    finish(&mut state, &mut concept, &mut out);
                }
            }
            Event::Text(t) if state == In::Label => {
                let text = t.unescape().map_err(|e| err(e.to_string()))?;
                concept.label = text.split_whitespace().collect::<Vec<_>>().join(" ");
            }
            Event::End(e) => match (state, e.name().as_ref()) {
                (In::Label, b"skos:prefLabel") => state = In::Concept,
                (In::Scheme, b"skos:ConceptScheme") | (In::Concept, b"skos:Concept") => {
                    finish(&mut state, &mut concept, &mut out)
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    if state != In::Nothing {
        return Err("file ends inside an element".to_string());
    }
    Ok(out)
}

fn is_container(e: &BytesStart) -> bool {
    matches!(e.name().as_ref(), b"skos:ConceptScheme" | b"skos:Concept")
}

fn finish(state: &mut In, concept: &mut Concept, out: &mut Skos) {
    if *state == In::Concept {
        out.concepts.push(std::mem::take(concept));
    }
    *state = In::Nothing;
}

fn attr(e: &BytesStart, key: &str) -> Result<String, String> {
    let element = String::from_utf8_lossy(e.name().as_ref()).into_owned();
    let a = e
        .try_get_attribute(key)
        .map_err(|err| format!("<{element}>: {err}"))?
        .ok_or_else(|| format!("<{element}> has no `{key}`"))?;
    a.unescape_value()
        .map(|v| v.into_owned())
        .map_err(|err| format!("<{element}> `{key}`: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_scheme_concepts_and_entities() {
        let xml = br#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="r" xmlns:skos="s">
  <skos:ConceptScheme rdf:about="ccs"><skos:hasTopConcept rdf:resource="1" /></skos:ConceptScheme>
  <skos:Concept rdf:about="1">
    <skos:prefLabel lang="en">Tops &amp;  bottoms</skos:prefLabel>
    <skos:topConceptOf rdf:resource="ccs" />
    <skos:narrower rdf:resource="1.2" />
  </skos:Concept>
  <skos:Concept rdf:about="1.2">
    <skos:prefLabel lang="en">Leaf</skos:prefLabel>
    <skos:broader rdf:resource="1" />
  </skos:Concept>
</rdf:RDF>"#;
        let s = parse(xml).unwrap();
        assert_eq!(s.scheme, "ccs");
        assert_eq!(s.top, ["1"]);
        assert_eq!(s.concepts.len(), 2);
        assert_eq!(s.concepts[0].label, "Tops & bottoms");
        assert!(s.concepts[0].top);
        assert_eq!(s.concepts[0].narrower, ["1.2"]);
        assert_eq!(s.concepts[1].broader.as_deref(), Some("1"));
    }

    #[test]
    fn rejects_unknown_elements_and_languages() {
        let other = br#"<rdf:RDF><skos:Concept rdf:about="1"><skos:altLabel lang="en">x</skos:altLabel></skos:Concept></rdf:RDF>"#;
        assert!(parse(other).unwrap_err().contains("skos:altLabel"));
        let fr = br#"<rdf:RDF><skos:Concept rdf:about="1"><skos:prefLabel lang="fr">x</skos:prefLabel></skos:Concept></rdf:RDF>"#;
        assert!(parse(fr).unwrap_err().contains("`fr`"));
    }
}
