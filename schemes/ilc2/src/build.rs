//! The ILC2 SKOS triples to a scheme document.
//!
//! ILC2 has two kinds of class, both kept:
//!
//! - **Concepts** (`skos:Concept`): the schedule proper, a tree by
//!   `skos:broader`. Code is `skos:notation`, label `skos:prefLabel`,
//!   `skos:altLabel` become vocabulary links.
//! - **Facet indicators** (`rdf:Property`): the relationship facets
//!   (`0` "as for perspective", `mq94` "affected by disease", ...). They
//!   form their own tree by `rdfs:subPropertyOf`. The scheme schema has no
//!   facet field, so each indicator's domain and range are written as data
//!   on the first line of its note: `facet: domain=mq range=mqad`
//!   (`*` = any class). They are classes in their own right, never
//!   flattened into the concepts.
//!
//! Each class's `note` also carries, one per line and only when present,
//! `note:` (`skos:note`), `scope:` (`skos:scopeNote`), `ddc:` (the
//! `skos:closeMatch` Dewey numbers) and `related:` (`skos:related`
//! codes). Faceted combinations such as `x5y` are not in the published
//! file (the publisher left them out of the conversion), so they are
//! built by the user from the indicators, not listed.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;

use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, VocabLink, SCHEME_SCHEMA_ID};

use crate::nt::{self, Object};
use crate::sources::Lock;

pub const NAME: &str = "ilc2";
pub const TITLE: &str = "Integrative Levels Classification, edition 2 (ILC2)";
/// Vocabulary source of `skos:altLabel` terms.
pub const ALT_SOURCE: &str = "ilc-alt";
/// Weight of an alternative label: a little below the preferred label.
pub const ALT_WEIGHT: f64 = 0.9;
pub const URL: &str = "https://www.iskoi.org/ilc/";
pub const LICENSE: &str = "Unsettled; shelfmark does not redistribute this file. The publisher's \
SKOS file declares cc:license http://creativecommons.org/licenses/by/3.0 (CC BY 3.0), BARTOC lists \
CC BY-SA 3.0, and no iskoi.org page states a licence. Attribute: Integrative Levels Classification, \
edition 2 (ILC2), ISKO Italy, Claudio Gnoli and contributors, https://www.iskoi.org/ilc/. \
Treat a built file as CC BY-SA 3.0 (the stricter reading) if you redistribute it.";
/// Notations are letters, digits, and the two top signs `*` and `~`.
pub const CODE_GRAMMAR: &str = "[A-Za-z0-9*~]+";

const CLASS: &str = "http://www.iskoi.org/ilc/2/class/";
const SCHEME_IRI: &str = "http://www.iskoi.org/ilc/2/scheme";
const SKOS: &str = "http://www.w3.org/2004/02/skos/core#";
const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDF_PROPERTY: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#Property";
const DC_ISSUED: &str = "http://purl.org/dc/terms/issued";
const DEWEY: &str = "http://dewey.info/class/";

/// Shape of a part of the tree.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Shape {
    pub classes: usize,
    pub roots: usize,
    /// Levels, a root being level 1.
    pub depth: usize,
    pub max_branching: usize,
    /// Children per class that has children.
    pub mean_branching: f64,
}

/// What a build kept.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Stats {
    pub concepts: Shape,
    pub facets: Shape,
    pub all: Shape,
    /// Facet indicators whose published label is empty; they take their
    /// parent's label and the domain class.
    pub relabelled: usize,
}

#[derive(Default)]
struct Node {
    is_property: bool,
    is_concept: bool,
    notation: Vec<String>,
    pref: Vec<String>,
    alts: Vec<String>,
    broader: Vec<String>,
    sub_of: Vec<String>,
    domain: Vec<String>,
    range: Vec<String>,
    notes: Vec<String>,
    scopes: Vec<String>,
    ddc: Vec<String>,
    related: Vec<String>,
}

/// Build the scheme from the file's text.
pub fn build(lock: &Lock, text: &str) -> Result<(SchemeDoc, Stats), String> {
    let triples = nt::parse(text)?;
    let mut nodes: BTreeMap<&str, Node> = BTreeMap::new();
    let mut issued = None;
    for t in &triples {
        let (s, p) = (t.subject.as_str(), t.predicate.as_str());
        if s == SCHEME_IRI {
            if let (DC_ISSUED, Object::Literal(v)) = (p, &t.object) {
                issued = Some(v.clone());
            }
            continue;
        }
        if !s.starts_with(CLASS) {
            return Err(format!("unexpected subject <{s}>"));
        }
        let n = nodes.entry(s).or_default();
        let (text, iri) = match &t.object {
            Object::Literal(v) => (Some(v.clone()), None),
            Object::Iri(v) => (None, Some(v.clone())),
        };
        if p == RDF_TYPE {
            match iri.as_deref() {
                Some(RDF_PROPERTY) => n.is_property = true,
                Some(x) if x.strip_prefix(SKOS) == Some("Concept") => n.is_concept = true,
                other => return Err(format!("<{s}> has unexpected type {other:?}")),
            }
            continue;
        }
        let local = p.strip_prefix(SKOS).or_else(|| p.strip_prefix(RDFS));
        match local {
            Some("notation") => n.notation.extend(text),
            Some("prefLabel") => n.pref.extend(text),
            Some("altLabel") => n.alts.extend(text),
            Some("note") => n.notes.extend(text),
            Some("scopeNote") => n.scopes.extend(text),
            Some("broader") => n.broader.extend(iri),
            Some("subPropertyOf") => n.sub_of.extend(iri),
            Some("domain") => n.domain.extend(iri),
            Some("range") => n.range.extend(iri),
            Some("related") => n.related.extend(iri),
            Some("closeMatch") => {
                let v = iri.unwrap_or_default();
                match v.strip_prefix(DEWEY) {
                    Some(d) => n.ddc.push(d.to_string()),
                    None => return Err(format!("<{s}> closeMatch to <{v}>, not Dewey")),
                }
            }
            // narrower (the inverse of broader), labels, seeAlso, inScheme and
            // topConceptOf are derived or presentational.
            _ => {}
        }
    }
    let issued = issued.ok_or("the scheme has no dct:issued date")?;
    let version = issued.trim().to_string();

    if let Some((iri, _)) = nodes.iter().find(|(_, n)| !n.is_property && !n.is_concept) {
        return Err(format!("<{iri}> is neither a Concept nor a Property"));
    }
    let codes = notation_map(&nodes)?;
    let order = tree_order(&nodes, &codes)?;

    let mut classes: Vec<ClassDef> = Vec::with_capacity(order.len());
    let mut label_of: HashMap<String, String> = HashMap::new();
    let mut relabelled = 0;
    for &(iri, parent_iri) in &order {
        let n = &nodes[iri];
        let code = codes[iri].clone();
        let parent = parent_iri.map(|p| codes[p].clone());
        // The first prefLabel is the label; a second one (the file has a few)
        // is kept as an alternative label.
        let mut label = n
            .pref
            .first()
            .ok_or_else(|| format!("<{iri}> has no prefLabel"))?
            .trim()
            .to_string();
        let alts: Vec<String> = n
            .alts
            .iter()
            .chain(n.pref.iter().skip(1))
            .cloned()
            .collect();
        if label.is_empty() {
            if !n.is_property {
                return Err(format!("`{code}` has no prefLabel"));
            }
            let base = parent
                .as_ref()
                .and_then(|p| label_of.get(p))
                .map_or(String::new(), |l| format!("{l} "));
            let dom = n.domain.first().and_then(|d| codes.get(d.as_str()));
            label = format!(
                "{base}[facet {code}{}]",
                dom.map_or(String::new(), |d| format!(" of {d}"))
            );
            relabelled += 1;
        }
        label_of.insert(code.clone(), label.clone());
        classes.push(ClassDef {
            code,
            label: label.clone(),
            parent,
            range: None,
            vocab: alt_links(&alts, &label),
            note: note(n, &codes)?,
        });
    }
    let stats = Stats {
        concepts: shape(&classes, |c| !is_facet(c)),
        facets: shape(&classes, is_facet),
        all: shape(&classes, |_| true),
        relabelled,
    };
    let doc = SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: NAME.to_string(),
        version,
        title: TITLE.to_string(),
        source: Some(Source {
            description: description(lock, &stats),
            url: Some(URL.to_string()),
            license: Some(LICENSE.to_string()),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    };
    Ok((doc, stats))
}

/// A facet indicator carries a `facet:` first note line.
pub fn is_facet(c: &ClassDef) -> bool {
    c.note.as_deref().is_some_and(|n| n.starts_with("facet:"))
}

fn one<'a>(v: &'a [String], what: &str, iri: &str) -> Result<&'a str, String> {
    match v {
        [x] => Ok(x),
        _ => Err(format!("<{iri}> has {} {what} values, expected 1", v.len())),
    }
}

/// IRI to notation, each notation unique and equal to the IRI's tail.
fn notation_map<'a>(nodes: &BTreeMap<&'a str, Node>) -> Result<HashMap<&'a str, String>, String> {
    let mut codes = HashMap::new();
    let mut seen = BTreeSet::new();
    for (&iri, n) in nodes {
        let code = one(&n.notation, "notation", iri)?.to_string();
        let tail = iri.strip_prefix(CLASS).unwrap_or_default();
        if percent_decode(tail) != code {
            return Err(format!("<{iri}> does not end in its notation `{code}`"));
        }
        if !seen.insert(code.clone()) {
            return Err(format!("notation `{code}` is used twice"));
        }
        codes.insert(iri, code);
    }
    Ok(codes)
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = (b[i] == b'%' && i + 2 < b.len())
            .then(|| std::str::from_utf8(&b[i + 1..i + 3]).ok())
            .flatten()
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match hex {
            Some(v) => {
                out.push(v);
                i += 3;
            }
            None => {
                out.push(b[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Every class once as (IRI, parent IRI), parents before children,
/// siblings by code. The parent is the single `broader` of a concept or
/// the single `subPropertyOf` of a facet when that is another ILC2 class.
fn tree_order<'a>(
    nodes: &BTreeMap<&'a str, Node>,
    codes: &HashMap<&'a str, String>,
) -> Result<Vec<(&'a str, Option<&'a str>)>, String> {
    let mut kids: BTreeMap<Option<&str>, Vec<&str>> = BTreeMap::new();
    for (&iri, n) in nodes {
        let links = if n.is_property { &n.sub_of } else { &n.broader };
        let parent = match links.as_slice() {
            [] => None,
            [p] if p.starts_with(CLASS) => {
                // A parent the file never defines leaves a root; the note
                // keeps its code (see `note`).
                match nodes.get_key_value(p.as_str()) {
                    Some((&key, target)) if target.is_property == n.is_property => Some(key),
                    Some(_) => {
                        return Err(format!(
                            "<{iri}> and its parent <{p}> are of different kinds"
                        ))
                    }
                    None => None,
                }
            }
            // subPropertyOf skos:related: a root facet.
            [p] if n.is_property && p.starts_with(SKOS) => None,
            more => return Err(format!("<{iri}> has parents {more:?}, expected at most 1")),
        };
        kids.entry(parent).or_default().push(iri);
    }
    for v in kids.values_mut() {
        v.sort_by(|a, b| codes[a].cmp(&codes[b]));
    }
    let mut out = Vec::with_capacity(nodes.len());
    let mut stack: Vec<(&str, Option<&str>)> = kids
        .get(&None)
        .into_iter()
        .flatten()
        .rev()
        .map(|&i| (i, None))
        .collect();
    while let Some((iri, parent)) = stack.pop() {
        out.push((iri, parent));
        stack.extend(
            kids.get(&Some(iri))
                .into_iter()
                .flatten()
                .rev()
                .map(|&c| (c, Some(iri))),
        );
    }
    if out.len() != nodes.len() {
        return Err(format!(
            "{} classes are in a parent cycle or unreachable",
            nodes.len() - out.len()
        ));
    }
    Ok(out)
}

fn alt_links(alts: &[String], label: &str) -> Vec<VocabLink> {
    let mut seen = BTreeSet::new();
    alts.iter()
        .map(|a| a.trim())
        .filter(|a| !a.is_empty() && *a != label && seen.insert(*a))
        .map(|a| VocabLink {
            term: a.to_string(),
            source: ALT_SOURCE.to_string(),
            source_id: None,
            weight: ALT_WEIGHT,
        })
        .collect()
}

/// The note lines for one class, `None` when there is nothing to say.
fn note(n: &Node, codes: &HashMap<&str, String>) -> Result<Option<String>, String> {
    let mut lines = Vec::new();
    if n.is_property {
        let end = |v: &[String]| -> Result<String, String> {
            match v {
                [x] => Ok(codes.get(x.as_str()).cloned().unwrap_or_else(|| "*".into())),
                _ => Err("a facet indicator needs one domain and one range".to_string()),
            }
        };
        lines.push(format!(
            "facet: domain={} range={}",
            end(&n.domain)?,
            end(&n.range)?
        ));
    }
    let mut push = |key: &str, v: &[String], sep: &str| {
        let v: Vec<&str> = v
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if !v.is_empty() {
            lines.push(format!("{key}: {}", v.join(sep)));
        }
    };
    let missing: Vec<String> = n
        .broader
        .iter()
        .chain(&n.sub_of)
        .filter(|p| p.starts_with(CLASS) && !codes.contains_key(p.as_str()))
        .map(|p| percent_decode(&p[CLASS.len()..]))
        .collect();
    push("undefined-parent", &missing, ", ");
    push("note", &n.notes, "; ");
    push("scope", &n.scopes, "; ");
    let mut ddc = n.ddc.clone();
    ddc.sort();
    ddc.dedup();
    push("ddc", &ddc, ", ");
    let mut related: Vec<String> = n
        .related
        .iter()
        .map(|r| {
            codes
                .get(r.as_str())
                .cloned()
                .ok_or_else(|| format!("related <{r}> is not a class"))
        })
        .collect::<Result<_, _>>()?;
    related.sort();
    related.dedup();
    push("related", &related, ", ");
    Ok((!lines.is_empty()).then(|| lines.join("\n")))
}

/// Shape of the classes `keep` selects. Branching counts a class's
/// children among the selected classes.
pub fn shape(classes: &[ClassDef], keep: impl Fn(&ClassDef) -> bool) -> Shape {
    let chosen: Vec<&ClassDef> = classes.iter().filter(|c| keep(c)).collect();
    let depth_of: HashMap<&str, usize> = {
        let by_code: HashMap<&str, &ClassDef> =
            classes.iter().map(|c| (c.code.as_str(), c)).collect();
        let mut memo: HashMap<&str, usize> = HashMap::new();
        for c in classes {
            let mut d = 1;
            let mut cur = c;
            while let Some(p) = cur.parent.as_deref() {
                d += 1;
                cur = by_code[p];
            }
            memo.insert(c.code.as_str(), d);
        }
        memo
    };
    let mut kids: HashMap<&str, usize> = HashMap::new();
    for c in &chosen {
        if let Some(p) = c.parent.as_deref() {
            *kids.entry(p).or_default() += 1;
        }
    }
    let internal = kids.len();
    Shape {
        classes: chosen.len(),
        roots: chosen.iter().filter(|c| c.parent.is_none()).count(),
        depth: chosen
            .iter()
            .map(|c| depth_of[c.code.as_str()])
            .max()
            .unwrap_or(0),
        max_branching: kids.values().copied().max().unwrap_or(0),
        mean_branching: if internal == 0 {
            0.0
        } else {
            kids.values().sum::<usize>() as f64 / internal as f64
        },
    }
}

fn description(lock: &Lock, stats: &Stats) -> String {
    let mut d = format!(
        "{} classes of ILC2, the Integrative Levels Classification second edition: {} concepts \
         (a tree by skos:broader) and {} facet indicators (relationship facets, a tree by \
         rdfs:subPropertyOf; each one's domain and range are on the first line of its note), \
         from the SKOS file published by ISKO Italy at iskoi.org. Alternative labels are \
         vocabulary links with source {ALT_SOURCE}; Dewey close matches, notes and related \
         classes are note lines. Faceted combinations (x5y) are not in the published file. \
         Built by schemes/ilc2 from the file locked as {} (edition issued per dct:issued):",
        stats.all.classes,
        stats.concepts.classes,
        stats.facets.classes,
        lock.digest(),
    );
    let _ = write!(d, " {} sha256 {}.", lock.source.url, lock.source.sha256);
    d
}

/// The scheme as JSON, one class per line, so diffs stay readable.
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
