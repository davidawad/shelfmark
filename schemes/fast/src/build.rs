//! FAST headings to a shelfmark scheme.
//!
//! Every live (not deprecated) heading with a preferred label is a class,
//! coded `fst` plus its FAST number in eight digits, the form OCLC prints
//! (`fst00817698`). FAST is a heading list rather than a tree, so the
//! parent is chosen by rule, deterministically:
//!
//! 1. the lowest-numbered live `skos:broader` heading that does not make
//!    the class its own ancestor;
//! 2. otherwise, for a subdivided heading (`Computers--History`), the
//!    nearest live heading named by dropping trailing subdivisions;
//! 3. otherwise none: a top-level class.
//!
//! Classes are assigned in FAST-number order, and a candidate is refused
//! when the parents already assigned lead from it back to the class, so
//! the hierarchy never cycles.
//!
//! Vocabulary per class: each alternate label (`fast`), each LCSH heading
//! the class is `schema:sameAs` (`lcsh`, with its LCSH ID), and the label
//! of each deprecated heading `dcterms:isReplacedBy` the class (`fast`,
//! with the old FAST code, its weight split across the replacements).

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, VocabLink, SCHEME_SCHEMA_ID};

use crate::graph::{Graph, Heading};

pub const NAME: &str = "fast";
pub const VERSION: &str = "2024";
pub const GRAMMAR: &str = "fst[0-9]{8}";

/// What the build kept, dropped and decided, for SOURCES.md.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Stats {
    /// Headings with a preferred label, live or deprecated.
    pub headings: usize,
    pub deprecated: usize,
    /// Subjects with only an `rdfs:label`: headings of other facets the
    /// topical ones point at. Not classes.
    pub unlabelled: usize,
    pub classes: usize,
    pub parent_broader: usize,
    pub parent_subdivision: usize,
    pub top_level: usize,
    /// Classes with more than one live broader heading (all but one dropped).
    pub multiple_broader: usize,
    /// Broader links refused because they would close a cycle.
    pub broader_cycles: usize,
    /// Broader links to a deprecated, non-topical or absent heading.
    pub broader_dangling: usize,
    pub max_depth: usize,
    pub alt_terms: usize,
    pub lcsh_links: usize,
    /// LCSH links whose label the dump does not give (the FAST label is used).
    pub lcsh_unlabelled: usize,
    pub replaced_terms: usize,
}

pub fn code(n: u32) -> String {
    format!("fst{n:08}")
}

/// The scheme's `source`; `provenance` names the input files.
pub fn source(provenance: &str) -> Source {
    Source {
        description: format!(
            "FAST (Faceted Application of Subject Terminology) topical headings from OCLC's \
             FAST data files ({provenance}). Parents follow skos:broader, else the heading \
             with trailing subdivisions dropped; vocabulary is alternate labels, the LCSH \
             headings each FAST heading is sameAs, and the labels of deprecated headings \
             it replaces. This scheme contains information from FAST (Faceted Application \
             of Subject Terminology) Data (https://www.oclc.org/research/areas/data-science/fast/download.html) \
             which is made available by OCLC Online Computer Library Center, Inc. under the \
             ODC Attribution License (https://www.oclc.org/research/activities/fast/odcby.html). \
             Built by src/shelfmark/schemes/fast; see SOURCES.md there."
        ),
        url: Some("https://www.oclc.org/research/areas/data-science/fast/download.html".into()),
        license: Some("ODC-By-1.0".into()),
    }
}

pub fn build(g: &Graph, provenance: &str) -> (SchemeDoc, Stats) {
    let mut s = Stats::default();
    let live: BTreeMap<u32, (&Heading, &str)> = g
        .headings
        .iter()
        .filter_map(|(&n, h)| {
            let Some(label) = h.label.as_deref() else {
                s.unlabelled += 1;
                return None;
            };
            s.headings += 1;
            s.deprecated += usize::from(h.deprecated);
            (!h.deprecated).then_some((n, (h, label)))
        })
        .collect();
    let mut by_label: BTreeMap<&str, u32> = BTreeMap::new();
    for (&n, &(_, label)) in &live {
        by_label.entry(label).or_insert(n);
    }

    let parents = parents(&live, &by_label, &mut s);
    let replaced = replaced_terms(g, &live);
    let mut classes = Vec::with_capacity(live.len());
    for (&n, &(h, label)) in &live {
        let mut vocab = Vec::new();
        for alt in h.alts.iter().filter(|a| a.as_str() != label) {
            vocab.push(link(alt, "fast", None, 1.0));
            s.alt_terms += 1;
        }
        for sh in &h.lcsh {
            let term = g.lcsh_labels.get(sh).map_or_else(
                || {
                    s.lcsh_unlabelled += 1;
                    label
                },
                String::as_str,
            );
            vocab.push(link(term, "lcsh", Some(sh.clone()), 1.0));
            s.lcsh_links += 1;
        }
        for (term, old, weight) in replaced.get(&n).into_iter().flatten() {
            vocab.push(link(term, "fast", Some(code(*old)), *weight));
            s.replaced_terms += 1;
        }
        vocab.sort_by(|a, b| {
            (&a.source, &a.term, &a.source_id)
                .cmp(&(&b.source, &b.term, &b.source_id))
                .then(a.weight.total_cmp(&b.weight))
        });
        vocab.dedup_by(|a, b| {
            (&a.source, &a.term, &a.source_id) == (&b.source, &b.term, &b.source_id)
        });
        classes.push(ClassDef {
            code: code(n),
            label: label.to_string(),
            parent: parents.get(&n).map(|&p| code(p)),
            range: None,
            vocab,
            note: None,
        });
    }
    s.classes = classes.len();
    s.max_depth = max_depth(&parents);
    let doc = SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: NAME.into(),
        version: VERSION.into(),
        title: "FAST (Faceted Application of Subject Terminology) topical headings".into(),
        source: Some(source(provenance)),
        code_grammar: GRAMMAR.into(),
        classes,
    };
    (doc, s)
}

fn link(term: &str, source: &str, source_id: Option<String>, weight: f64) -> VocabLink {
    VocabLink {
        term: term.to_string(),
        source: source.into(),
        source_id,
        weight,
    }
}

/// Child -> parent, by the rules in the module doc.
fn parents(
    live: &BTreeMap<u32, (&Heading, &str)>,
    by_label: &BTreeMap<&str, u32>,
    s: &mut Stats,
) -> BTreeMap<u32, u32> {
    let mut parent: BTreeMap<u32, u32> = BTreeMap::new();
    let reaches = |parent: &BTreeMap<u32, u32>, from: u32, to: u32| {
        let mut cur = Some(from);
        while let Some(c) = cur {
            if c == to {
                return true;
            }
            cur = parent.get(&c).copied();
        }
        false
    };
    for (&n, &(h, label)) in live {
        let candidates: Vec<u32> = h
            .broader
            .iter()
            .copied()
            .filter(|b| {
                let ok = *b != n && live.contains_key(b);
                s.broader_dangling += usize::from(!ok && *b != n);
                ok
            })
            .collect();
        s.multiple_broader += usize::from(candidates.len() > 1);
        let mut chosen = None;
        for &b in &candidates {
            if reaches(&parent, b, n) {
                s.broader_cycles += 1;
            } else {
                chosen = Some(b);
                break;
            }
        }
        if chosen.is_some() {
            s.parent_broader += 1;
        } else {
            let mut head = label;
            while let Some((up, _)) = head.rsplit_once("--") {
                head = up;
                if let Some(&p) = by_label.get(up) {
                    if p != n && !reaches(&parent, p, n) {
                        chosen = Some(p);
                        s.parent_subdivision += 1;
                        break;
                    }
                }
            }
        }
        match chosen {
            Some(p) => {
                parent.insert(n, p);
            }
            None => s.top_level += 1,
        }
    }
    parent
}

/// Live class -> (deprecated label, old FAST number, weight).
fn replaced_terms(
    g: &Graph,
    live: &BTreeMap<u32, (&Heading, &str)>,
) -> BTreeMap<u32, Vec<(String, u32, f64)>> {
    let mut out: BTreeMap<u32, Vec<(String, u32, f64)>> = BTreeMap::new();
    for (&old, h) in g.headings.iter().filter(|(_, h)| h.deprecated) {
        let Some(label) = &h.label else { continue };
        let to: BTreeSet<u32> = h
            .replaced_by
            .iter()
            .copied()
            .filter(|r| live.contains_key(r))
            .collect();
        let weight = 1.0 / to.len().max(1) as f64;
        for r in to {
            out.entry(r).or_default().push((label.clone(), old, weight));
        }
    }
    out
}

fn max_depth(parent: &BTreeMap<u32, u32>) -> usize {
    let mut depth: BTreeMap<u32, usize> = BTreeMap::new();
    let mut max = 0;
    for &start in parent.keys() {
        // Climb until a class of known depth or a top-level class (0).
        let (mut chain, mut cur) = (Vec::new(), start);
        let base = loop {
            if let Some(&d) = depth.get(&cur) {
                break d;
            }
            match parent.get(&cur) {
                Some(&p) => {
                    chain.push(cur);
                    cur = p;
                }
                None => break 0,
            }
        };
        for (i, c) in chain.iter().rev().enumerate() {
            depth.insert(*c, base + i + 1);
        }
        max = max.max(base + chain.len());
    }
    max
}
