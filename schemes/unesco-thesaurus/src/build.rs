//! The SKOS export to a scheme document.
//!
//! Hierarchy: domain (7) > microthesaurus (88) > concept. Every concept
//! is a member of exactly one microthesaurus (checked), so the first two
//! levels partition the concepts. Below that the tree follows
//! `skos:broader`, with one rule for concepts that have several broader
//! terms (the thesaurus allows polyhierarchy, the scheme format is a
//! tree):
//!
//! 1. The parent is the broader term in the **same microthesaurus** with
//!    the lowest numeric concept id.
//! 2. A concept with no broader term in its own microthesaurus (a
//!    top concept, or one whose broader terms are all in other
//!    microthesauri) hangs directly under its microthesaurus.
//! 3. Every other broader term is kept as vocabulary: a link with source
//!    [`BROADER_SOURCE`] whose term is that broader term's label, so a
//!    text naming it still reaches this class, and the class note lists
//!    them.
//!
//! English only: the label is the `@en` `skos:prefLabel`, vocabulary
//! links are the `@en` `skos:altLabel` ([`ALT_SOURCE`]) and `@en`
//! `skos:hiddenLabel` ([`HIDDEN_SOURCE`]), the note is the `@en`
//! `skos:scopeNote`. `skos:related` is not used.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, VocabLink, SCHEME_SCHEMA_ID};

use crate::sources::Lock;
use crate::ttl::{self, Term, Triple};

pub const NAME: &str = "unesco-thesaurus";
pub const TITLE: &str = "UNESCO Thesaurus";
pub const URL: &str = "https://vocabularies.unesco.org/";
pub const ALT_SOURCE: &str = "unesco-alt";
pub const HIDDEN_SOURCE: &str = "unesco-hidden";
pub const BROADER_SOURCE: &str = "unesco-broader";
pub const ALT_WEIGHT: f64 = 0.9;
pub const HIDDEN_WEIGHT: f64 = 0.7;
pub const BROADER_WEIGHT: f64 = 0.5;

pub const LICENSE: &str = "Creative Commons Attribution-ShareAlike 3.0 IGO \
(CC BY-SA 3.0 IGO), https://creativecommons.org/licenses/by-sa/3.0/igo/. \
Attribution: UNESCO Thesaurus, (c) UNESCO, https://vocabularies.unesco.org/. \
Adapted by shelfmark (English labels only; hierarchy flattened to one parent per concept; \
related terms and other languages dropped); not endorsed by UNESCO. \
This file is an adaptation and stays under CC BY-SA 3.0 IGO.";

/// `1` (domain), `1.05` (microthesaurus), `c460` (concept).
pub const CODE_GRAMMAR: &str = "[0-9]+(?:\\.[0-9]+)?|c[0-9]+";

/// Whether `code` matches [`CODE_GRAMMAR`], without a regex dependency.
pub fn code_ok(code: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    match code.strip_prefix('c') {
        Some(rest) => digits(rest),
        None => match code.split_once('.') {
            Some((a, b)) => digits(a) && digits(b),
            None => digits(code),
        },
    }
}

/// What a build kept, and the shape of the tree.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Stats {
    pub classes: usize,
    pub domains: usize,
    pub microthesauri: usize,
    pub concepts: usize,
    /// Concepts with more than one broader term.
    pub multi_broader: usize,
    /// Broader links not used as the parent (kept as vocabulary).
    pub demoted_links: usize,
    /// Concepts with broader terms, none in their own microthesaurus.
    pub cross_only: usize,
    /// Levels, domain = 1.
    pub depth: usize,
    /// Largest number of children of any class.
    pub max_branching: usize,
    /// Mean children over classes that have any.
    pub mean_branching: f64,
    pub alt_links: usize,
    pub hidden_links: usize,
}

#[derive(Default)]
struct Subject {
    types: Vec<String>,
    props: Vec<(String, Term)>,
}

impl Subject {
    fn names(&self, pred: &str) -> Vec<&str> {
        self.props
            .iter()
            .filter(|(p, _)| p == pred)
            .filter_map(|(_, t)| match t {
                Term::Name(n) => Some(n.as_str()),
                Term::Lit { .. } => None,
            })
            .collect()
    }

    fn lits(&self, pred: &str, lang: &str) -> Vec<&str> {
        self.props
            .iter()
            .filter(|(p, _)| p == pred)
            .filter_map(|(_, t)| match t {
                Term::Lit { text, lang: l } if l == lang => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    fn one(&self, pred: &str, lang: &str, who: &str) -> Result<String, String> {
        match self.lits(pred, lang)[..] {
            [one] => Ok(one.to_string()),
            ref many => Err(format!(
                "{who}: {} `{pred}`@{lang} values, want 1",
                many.len()
            )),
        }
    }
}

fn local(name: &str) -> &str {
    name.strip_prefix(':').unwrap_or(name)
}

fn number(id: &str) -> u64 {
    id.trim_start_matches(|c: char| !c.is_ascii_digit())
        .parse()
        .unwrap_or(u64::MAX)
}

struct Concept {
    id: String,
    label: String,
    mt: String,
    broader: Vec<String>,
    subject: Subject,
}

/// Build the scheme from the export's Turtle text.
pub fn build(lock: &Lock, text: &str) -> Result<(SchemeDoc, Stats), String> {
    let triples = ttl::parse(text)?;
    let mut subjects: HashMap<String, Subject> = HashMap::new();
    for Triple {
        subject,
        predicate,
        object,
    } in triples
    {
        let s = subjects.entry(local(&subject).to_string()).or_default();
        match (predicate.as_str(), &object) {
            ("rdf:type", Term::Name(t)) => s.types.push(t.clone()),
            _ => s.props.push((predicate, object)),
        }
    }
    let version = scheme_version(&subjects)?;
    let of_type = |t: &str| -> Vec<&String> {
        let mut ids: Vec<&String> = subjects
            .iter()
            .filter(|(_, s)| s.types.iter().any(|x| x == t))
            .map(|(id, _)| id)
            .collect();
        ids.sort_by_key(|id| (number(id), (*id).clone()));
        ids
    };

    // Domains and microthesauri.
    let mut classes: Vec<ClassDef> = Vec::new();
    let mut by_code: HashMap<String, usize> = HashMap::new();
    let mut push = |classes: &mut Vec<ClassDef>, c: ClassDef| -> Result<(), String> {
        if !code_ok(&c.code) {
            return Err(format!("`{}` does not match the code grammar", c.code));
        }
        if by_code.insert(c.code.clone(), classes.len()).is_some() {
            return Err(format!("duplicate code `{}`", c.code));
        }
        classes.push(c);
        Ok(())
    };
    let mut domain_code: HashMap<String, String> = HashMap::new();
    for id in of_type("unescovoc:Domain") {
        let s = &subjects[id];
        let code = s.one("skos:notation", "", id)?;
        let label = s.one("skos:prefLabel", "en", id)?;
        domain_code.insert(id.clone(), code.clone());
        push(&mut classes, plain(code, label, None))?;
    }
    let mut mt_code: HashMap<String, String> = HashMap::new();
    let mut member_of: HashMap<String, String> = HashMap::new();
    for id in of_type("unescovoc:MicroThesaurus") {
        let s = &subjects[id];
        let code = s.one("skos:notation", "", id)?;
        let label = s.one("skos:prefLabel", "en", id)?;
        let parent = match s.names("iso-thes:superGroup")[..] {
            [d] => domain_code
                .get(local(d))
                .ok_or_else(|| format!("{id}: unknown domain {d}"))?
                .clone(),
            ref many => return Err(format!("{id}: {} super groups, want 1", many.len())),
        };
        for m in s.names("skos:member") {
            if let Some(first) = member_of.insert(local(m).to_string(), code.clone()) {
                return Err(format!("{m} is in microthesauri {first} and {code}"));
            }
        }
        mt_code.insert(id.clone(), code.clone());
        push(&mut classes, plain(code, label, Some(parent)))?;
    }

    // Concepts.
    let mut concepts: BTreeMap<String, Concept> = BTreeMap::new();
    for id in of_type("skos:Concept") {
        let s = &subjects[id];
        let mt = member_of
            .get(id)
            .ok_or_else(|| format!("{id} is in no microthesaurus"))?
            .clone();
        concepts.insert(
            id.clone(),
            Concept {
                id: id.clone(),
                label: s.one("skos:prefLabel", "en", id)?,
                mt,
                broader: s
                    .names("skos:broader")
                    .iter()
                    .map(|b| local(b).to_string())
                    .collect(),
                subject: Subject {
                    types: Vec::new(),
                    props: s.props.clone(),
                },
            },
        );
    }
    if let Some(m) = member_of.keys().find(|m| !concepts.contains_key(*m)) {
        return Err(format!("microthesaurus member {m} is not a concept"));
    }

    let mut stats = Stats {
        domains: domain_code.len(),
        microthesauri: mt_code.len(),
        concepts: concepts.len(),
        ..Stats::default()
    };
    let mut parent_of: HashMap<String, String> = HashMap::new();
    let mut labels: HashMap<&str, &str> = HashMap::new();
    for c in concepts.values() {
        labels.insert(&c.id, &c.label);
    }
    for c in concepts.values() {
        for b in &c.broader {
            if !concepts.contains_key(b) {
                return Err(format!("{}: broader {b} is not a concept", c.id));
            }
        }
        let mut same: Vec<&String> = c
            .broader
            .iter()
            .filter(|b| concepts[*b].mt == c.mt)
            .collect();
        // Numeric id is the documented preference. Break numeric ties by
        // the full source identifier to avoid input or hash iteration order.
        same.sort_by_key(|b| (number(b), (*b).clone()));
        let parent = match same.first() {
            Some(p) => format!("c{}", number(p)),
            None => c.mt.clone(),
        };
        if c.broader.len() > 1 {
            stats.multi_broader += 1;
        }
        if same.is_empty() && !c.broader.is_empty() {
            stats.cross_only += 1;
        }
        parent_of.insert(format!("c{}", number(&c.id)), parent);
    }
    for id in concepts.keys() {
        let (mut at, mut hops) = (format!("c{}", number(id)), 0);
        while let Some(p) = parent_of.get(at.as_str()) {
            at = p.clone();
            hops += 1;
            if hops > concepts.len() {
                return Err(format!("{id}: broader terms form a cycle"));
            }
        }
    }

    // Emit in tree order: concepts under their parent, by label.
    let mut kids: HashMap<String, Vec<&Concept>> = HashMap::new();
    for c in concepts.values() {
        kids.entry(parent_of[&format!("c{}", number(&c.id))].clone())
            .or_default()
            .push(c);
    }
    for v in kids.values_mut() {
        v.sort_by_key(|c| (c.label.to_lowercase(), number(&c.id)));
    }
    let heads = std::mem::take(&mut classes);
    let mut ordered: Vec<ClassDef> = Vec::new();
    let mut ctx = Emit {
        kids: &kids,
        labels: &labels,
        stats: &mut stats,
    };
    for d in heads.iter().filter(|c| c.parent.is_none()) {
        ordered.push(d.clone());
        for m in heads
            .iter()
            .filter(|m| m.parent.as_deref() == Some(&d.code))
        {
            ordered.push(m.clone());
            ctx.walk(&m.code, &mut ordered)?;
        }
    }
    if ordered.len() != by_code.len() + concepts.len() {
        return Err(format!(
            "{} classes emitted for {} expected",
            ordered.len(),
            by_code.len() + concepts.len()
        ));
    }
    shape(&ordered, &mut stats);
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
        classes: ordered,
    };
    Ok((doc, stats))
}

fn plain(code: String, label: String, parent: Option<String>) -> ClassDef {
    ClassDef {
        code,
        label,
        parent,
        range: None,
        vocab: Vec::new(),
        note: None,
    }
}

struct Emit<'a> {
    kids: &'a HashMap<String, Vec<&'a Concept>>,
    labels: &'a HashMap<&'a str, &'a str>,
    stats: &'a mut Stats,
}

impl Emit<'_> {
    /// Emit the concepts under `parent` depth first.
    fn walk(&mut self, parent: &str, out: &mut Vec<ClassDef>) -> Result<(), String> {
        let Some(children) = self.kids.get(parent) else {
            return Ok(());
        };
        for c in children {
            let def = self.class(c, parent)?;
            let code = def.code.clone();
            out.push(def);
            self.walk(&code, out)?;
        }
        Ok(())
    }

    fn class(&mut self, c: &Concept, parent: &str) -> Result<ClassDef, String> {
        let code = format!("c{}", number(&c.id));
        let mut seen: Vec<String> = vec![c.label.to_lowercase()];
        let mut vocab = Vec::new();
        let mut add = |term: &str,
                       source: &str,
                       id: Option<String>,
                       weight: f64,
                       vocab: &mut Vec<VocabLink>| {
            let term = term.trim();
            if term.is_empty() || seen.iter().any(|s| *s == term.to_lowercase()) {
                return false;
            }
            seen.push(term.to_lowercase());
            vocab.push(VocabLink {
                term: term.to_string(),
                source: source.to_string(),
                source_id: id,
                weight,
            });
            true
        };
        for t in c.subject.lits("skos:altLabel", "en") {
            if add(t, ALT_SOURCE, None, ALT_WEIGHT, &mut vocab) {
                self.stats.alt_links += 1;
            }
        }
        for t in c.subject.lits("skos:hiddenLabel", "en") {
            if add(t, HIDDEN_SOURCE, None, HIDDEN_WEIGHT, &mut vocab) {
                self.stats.hidden_links += 1;
            }
        }
        let primary = parent.strip_prefix('c').map(number);
        let mut others: Vec<&String> = c
            .broader
            .iter()
            .filter(|b| Some(number(b)) != primary)
            .collect();
        others.sort_by_key(|b| (number(b), (*b).clone()));
        let mut other_labels = Vec::new();
        for b in others {
            let label = self.labels[b.as_str()];
            self.stats.demoted_links += 1;
            other_labels.push(label.to_string());
            add(
                label,
                BROADER_SOURCE,
                Some(b.clone()),
                BROADER_WEIGHT,
                &mut vocab,
            );
        }
        let mut note = c.subject.lits("skos:scopeNote", "en").join(" ");
        if !other_labels.is_empty() {
            if !note.is_empty() {
                note.push(' ');
            }
            let _ = write!(note, "Also narrower than: {}.", other_labels.join("; "));
        }
        Ok(ClassDef {
            code,
            label: c.label.clone(),
            parent: Some(parent.to_string()),
            range: None,
            vocab,
            note: (!note.is_empty()).then_some(note),
        })
    }
}

/// Depth and branching of the finished tree.
fn shape(classes: &[ClassDef], stats: &mut Stats) {
    let mut kids: HashMap<&str, usize> = HashMap::new();
    let mut level: HashMap<&str, usize> = HashMap::new();
    for c in classes {
        let l = c.parent.as_deref().map_or(1, |p| {
            *kids.entry(p).or_default() += 1;
            level.get(p).copied().unwrap_or(0) + 1
        });
        level.insert(&c.code, l);
    }
    stats.classes = classes.len();
    stats.depth = level.values().copied().max().unwrap_or(0);
    stats.max_branching = kids.values().copied().max().unwrap_or(0);
    stats.mean_branching = if kids.is_empty() {
        0.0
    } else {
        kids.values().sum::<usize>() as f64 / kids.len() as f64
    };
}

fn scheme_version(subjects: &HashMap<String, Subject>) -> Result<String, String> {
    let s = subjects
        .get("http://vocabularies.unesco.org/thesaurus")
        .ok_or("the export has no skos:ConceptScheme")?;
    let v = s.one("dct:modified", "", "the concept scheme")?;
    let ok = v.len() == 10 && v.bytes().all(|b| b.is_ascii_digit() || b == b'-');
    if ok {
        Ok(v)
    } else {
        Err(format!("scheme dct:modified `{v}` is not a date"))
    }
}

fn description(lock: &Lock, st: &Stats) -> String {
    format!(
        "{} classes of the UNESCO Thesaurus (SKOS): {} domains, {} microthesauri and {} concepts, \
         English labels, alternative and hidden labels as vocabulary. A concept's parent is its \
         lowest-id broader term in the same microthesaurus, else its microthesaurus; other broader \
         terms are vocabulary links with source {BROADER_SOURCE}. Built by \
         schemes/unesco-thesaurus from {} sha256 {} ({}, fetched {}), locked as {}.",
        st.classes,
        st.domains,
        st.microthesauri,
        st.concepts,
        lock.url,
        lock.sha256,
        lock.file,
        lock.fetched,
        lock.digest()
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_grammar_matches_the_three_levels() {
        for ok in ["1", "7", "1.05", "c460", "c1"] {
            assert!(code_ok(ok), "{ok}");
        }
        for bad in ["", "c", "1.", ".5", "1.05.1", "concept1", "C1", "1 2"] {
            assert!(!code_ok(bad), "{bad}");
        }
    }
}
