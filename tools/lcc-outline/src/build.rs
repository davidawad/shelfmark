//! Assemble the scheme: main classes, subclasses, then every numbered
//! outline span, each under the deepest earlier class that wholly holds
//! it (the nesting `shelfmark-scheme` validates). LCSH headings link to a
//! class when their LCC number is exactly that class's number or span.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use regex::Regex;
use shelfmark_core::scheme::{ClassDef, CodeRange, SchemeDoc, Source, VocabLink, SCHEME_SCHEMA_ID};

use crate::code::{lead, Cover, Key, Lead, CODE_GRAMMAR};
use crate::lcco::Listing;
use crate::lcsh::Heading;
use crate::outline::{Entry, Parsed};

pub const VERSION: &str = "2024";

/// What the build kept and dropped, for SOURCES.md.
#[derive(Debug, Default)]
pub struct Stats {
    pub classes: usize,
    pub subclasses: usize,
    pub spans: usize,
    pub merged: usize,
    pub off_grammar: Vec<String>,
    pub obsolete: usize,
    pub table_lines: usize,
    pub unparsed: usize,
    pub bad_numbers: Vec<String>,
    pub uncaptioned: Vec<String>,
    pub from_records: usize,
    pub lcsh_headings: usize,
    pub lcsh_links: usize,
    pub classes_with_lcsh: usize,
    pub unlabelled_prefixes: Vec<String>,
}

struct Node {
    def: ClassDef,
    cover: Cover,
    depth: u32,
}

fn grammar() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(&format!("^(?:{CODE_GRAMMAR})$")).expect("static regex"))
}

fn link(term: &str, source: &str, id: Option<&str>) -> VocabLink {
    VocabLink {
        term: term.to_string(),
        source: source.to_string(),
        source_id: id.map(str::to_string),
        weight: 1.0,
    }
}

struct Builder {
    nodes: Vec<Node>,
    by_code: BTreeMap<String, usize>,
    stats: Stats,
}

impl Builder {
    /// Add a class under the deepest node from `from` on that holds it.
    fn add(&mut self, from: usize, def: ClassDef, cover: Cover) -> usize {
        let parent = (from..self.nodes.len())
            .filter(|&i| self.nodes[i].cover.holds(&cover))
            .max_by_key(|&i| (self.nodes[i].depth, i));
        let mut def = def;
        def.parent = parent.map(|p| self.nodes[p].def.code.clone());
        let depth = parent.map_or(0, |p| self.nodes[p].depth + 1);
        self.by_code.insert(def.code.clone(), self.nodes.len());
        self.nodes.push(Node { def, cover, depth });
        self.nodes.len() - 1
    }

    /// Keep another caption for class `i` as an `lcc` vocabulary term,
    /// unless it is a generic one (`General`) that would match anything.
    fn alias(&mut self, i: usize, caption: &str) {
        const GENERIC: [&str; 3] = ["General", "General works", "History"];
        let def = &mut self.nodes[i].def;
        let useful = !caption.is_empty() && caption != def.label && !GENERIC.contains(&caption);
        if useful && !def.vocab.iter().any(|v| v.term == caption) {
            def.vocab.push(link(caption, "lcc", None));
        }
    }
}

fn class(code: &str, label: &str) -> ClassDef {
    ClassDef {
        code: code.to_string(),
        label: label.to_string(),
        parent: None,
        range: None,
        vocab: Vec::new(),
        note: None,
    }
}

fn span_cover(e: &Entry) -> Cover {
    if e.start == e.end {
        Cover::Under(Key::of(&e.start))
    } else {
        Cover::Span(Key::of(&e.start), Key::of(&e.end))
    }
}

fn span_range(e: &Entry) -> Option<CodeRange> {
    (e.start != e.end).then(|| CodeRange {
        start: e.start.render(),
        end: e.end.render(),
    })
}

fn note(e: &Entry) -> Option<String> {
    (!e.notes.is_empty()).then(|| e.notes.join(" "))
}

/// Letter prefixes (two or more letters) the outlines use, and the
/// two-letter subclass over each (`KN` over `KNC`), that no class outline
/// lists: their captions come from id.loc.gov.
pub fn unlisted_prefixes(listings: &[Listing], parsed: &Parsed) -> Vec<String> {
    let listed: BTreeSet<&str> = listings
        .iter()
        .flat_map(|l| &l.subclasses)
        .map(|s| s.code.as_str())
        .collect();
    let used: BTreeSet<&str> = parsed
        .entries
        .iter()
        .map(|e| e.start.letters.as_str())
        .chain(parsed.headings.iter().map(|h| h.letters.as_str()))
        .flat_map(|l| [l, &l[..l.len().min(2)]])
        .filter(|l| l.len() >= 2 && !listed.contains(l))
        .collect();
    used.into_iter().map(str::to_string).collect()
}

/// The id.loc.gov record name of an entry: both ends in full.
pub fn record_name(e: &Entry) -> String {
    if e.start == e.end {
        e.start.render()
    } else {
        format!("{}-{}", e.start.render(), e.end.render())
    }
}

/// Build the scheme from the class outlines, the parsed FreeLCC outlines,
/// captions from id.loc.gov records (by record name) for numbers the
/// outlines print without one, and the LCSH headings.
pub fn build(
    listings: &[Listing],
    parsed: &Parsed,
    records: &BTreeMap<String, String>,
    lcsh: &[Heading],
) -> Result<(SchemeDoc, Stats), String> {
    let mut classes: Vec<(String, String)> = listings
        .iter()
        .flat_map(|l| l.classes.iter().map(|c| (c.code.clone(), c.label.clone())))
        .collect();
    classes.sort();
    classes.dedup_by(|a, b| a.0 == b.0);
    if classes.len() != 21 {
        let got: Vec<_> = classes.iter().map(|c| c.0.as_str()).collect();
        return Err(format!("expected the 21 LCC main classes, found {got:?}"));
    }

    // Subclass captions: the class outlines, then id.loc.gov subclass
    // records, then FreeLCC letter headings.
    let mut sub_labels: BTreeMap<String, String> = BTreeMap::new();
    for s in listings.iter().flat_map(|l| &l.subclasses) {
        if s.code.len() >= 2 {
            sub_labels
                .entry(s.code.clone())
                .or_insert_with(|| s.label.clone());
        }
    }
    for (name, label) in records {
        if name.chars().all(|c| c.is_ascii_uppercase()) && name.len() >= 2 {
            sub_labels
                .entry(name.clone())
                .or_insert_with(|| label.clone());
        }
    }
    for h in &parsed.headings {
        sub_labels
            .entry(h.letters.clone())
            .or_insert_with(|| h.caption.clone());
    }

    let mut stats = Stats {
        obsolete: parsed.obsolete,
        table_lines: parsed.table_lines,
        unparsed: parsed.unparsed.len(),
        bad_numbers: parsed.bad_numbers.clone(),
        ..Stats::default()
    };
    let mut filled = Vec::new();
    for e in &parsed.entries {
        let mut e = e.clone();
        if e.caption.is_empty() {
            match records.get(&record_name(&e)) {
                Some(label) => {
                    e.caption = label.clone();
                    stats.from_records += 1;
                }
                None => {
                    stats.uncaptioned.push(Lead::code(&e.start, &e.end));
                    continue;
                }
            }
        }
        filled.push(e);
    }
    let entries: Vec<&Entry> = filled
        .iter()
        .filter(|e| {
            let ok = [
                e.start.render(),
                e.end.render(),
                Lead::code(&e.start, &e.end),
            ]
            .iter()
            .all(|c| grammar().is_match(c));
            if !ok {
                stats.off_grammar.push(Lead::code(&e.start, &e.end));
            }
            ok
        })
        .collect();

    // A subclass's span: its first entry, when that stays within the
    // subclass's letters and holds all its other entries.
    let mut by_prefix: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, e) in entries.iter().enumerate() {
        by_prefix
            .entry(e.start.letters.as_str())
            .or_default()
            .push(i);
    }
    let mut consumed: BTreeMap<String, usize> = BTreeMap::new();
    for (prefix, idx) in &by_prefix {
        if prefix.len() < 2 {
            continue;
        }
        let first = idx[0];
        let cover = span_cover(entries[first]);
        let inside = entries[first].end.letters == *prefix;
        if inside
            && idx[1..]
                .iter()
                .all(|&j| cover.holds(&span_cover(entries[j])))
        {
            consumed.insert(prefix.to_string(), first);
            sub_labels
                .entry(prefix.to_string())
                .or_insert_with(|| entries[first].caption.clone());
        } else if !sub_labels.contains_key(*prefix) {
            stats.unlabelled_prefixes.push(prefix.to_string());
        }
    }

    let mut b = Builder {
        nodes: Vec::new(),
        by_code: BTreeMap::new(),
        stats,
    };
    let mut emitted: BTreeSet<String> = BTreeSet::new();
    for (letter, label) in &classes {
        let from = b.nodes.len();
        b.add(from, class(letter, label), Cover::Under(Key::parse(letter)));
        b.stats.classes += 1;
        for (i, e) in entries.iter().enumerate() {
            if !e.start.letters.starts_with(letter.as_str()) {
                continue;
            }
            // The subclasses this entry falls under, shortest first (`KN`
            // before `KNC`), each emitted before its first entry.
            let prefix = e.start.letters.as_str();
            let mut absorbed = false;
            for len in 2..=prefix.len() {
                let p = &prefix[..len];
                let Some(sub) = sub_labels.get(p) else {
                    continue;
                };
                if !emitted.insert(p.to_string()) {
                    continue;
                }
                b.stats.subclasses += 1;
                if p == prefix && consumed.get(p) == Some(&i) {
                    let mut def = class(p, sub);
                    def.range = Some(CodeRange {
                        start: e.start.render(),
                        end: e.end.render(),
                    });
                    def.note = note(e);
                    let n = b.add(from, def, span_cover(e));
                    b.alias(n, &e.caption);
                    absorbed = true;
                } else {
                    b.add(from, class(p, sub), Cover::Under(Key::parse(p)));
                }
            }
            if absorbed {
                continue;
            }
            let code = Lead::code(&e.start, &e.end);
            if let Some(&n) = b.by_code.get(&code) {
                b.stats.merged += 1;
                b.alias(n, &e.caption);
                continue;
            }
            let mut def = class(&code, &e.caption);
            def.range = span_range(e);
            def.note = note(e);
            b.add(from, def, span_cover(e));
            b.stats.spans += 1;
        }
        let rest: Vec<(String, String)> = sub_labels
            .iter()
            .filter(|(c, _)| c.starts_with(letter.as_str()) && !emitted.contains(*c))
            .map(|(c, l)| (c.clone(), l.clone()))
            .collect();
        for (code, sub) in rest {
            emitted.insert(code.clone());
            b.stats.subclasses += 1;
            b.add(from, class(&code, &sub), Cover::Under(Key::parse(&code)));
        }
    }

    // Single-letter subclass captions (`Q Science (General)`) alias their class.
    for s in listings.iter().flat_map(|l| &l.subclasses) {
        if let Some(&n) = b.by_code.get(&s.code).filter(|_| s.code.len() == 1) {
            b.alias(n, &s.label);
        }
    }
    link_lcsh(&mut b, lcsh);

    let classes = b.nodes.into_iter().map(|n| n.def).collect();
    let doc = SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: "lcc".to_string(),
        version: VERSION.to_string(),
        title: "Library of Congress Classification outline".to_string(),
        source: Some(Source {
            description: "Main classes and subclasses from the LC Classification Outline; numbered spans and captions from the 2024 FreeLCC outline PDFs; LCSH links from id.loc.gov subject headings whose LCC number is exactly a class's number or span. A span's code is `<start>-<end>` with the end's letters dropped when they repeat the start's. Built by tools/lcc-outline; see data/schemes/lcc/SOURCES.md.".to_string(),
            url: Some("https://www.loc.gov/aba/publications/FreeLCC/freelcc.html".to_string()),
            license: Some("Library of Congress data: US government work, not subject to copyright in the United States".to_string()),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    };
    Ok((doc, b.stats))
}

/// The scheme code an LCSH LCC number names, if it is a plain number,
/// span or letters.
fn lcsh_code(lcc: &str) -> Option<String> {
    let lcc = lcc.trim();
    if !lcc.is_empty() && lcc.len() <= 3 && lcc.chars().all(|c| c.is_ascii_uppercase()) {
        return Some(lcc.to_string());
    }
    match lead(lcc)? {
        Lead::Span { start, end, rest } if rest.trim().is_empty() => Some(Lead::code(&start, &end)),
        _ => None,
    }
}

fn link_lcsh(b: &mut Builder, lcsh: &[Heading]) {
    // A subclass with a span also answers to the span's code.
    let mut lookup: BTreeMap<String, usize> = b.by_code.clone();
    for (i, n) in b.nodes.iter().enumerate() {
        if let Some(r) = &n.def.range {
            if let Some(Lead::Span { start, end, .. }) = lead(&format!("{}-{}", r.start, r.end)) {
                lookup.entry(Lead::code(&start, &end)).or_insert(i);
            }
        }
    }
    let mut seen: BTreeSet<(usize, String)> = BTreeSet::new();
    for h in lcsh {
        let Some(&i) = lcsh_code(&h.lcc).and_then(|c| lookup.get(&c)) else {
            continue;
        };
        b.stats.lcsh_headings += 1;
        if seen.insert((i, h.id.clone())) {
            b.nodes[i]
                .def
                .vocab
                .push(link(&h.label, "lcsh", Some(&h.id)));
            b.stats.lcsh_links += 1;
        }
    }
    for n in &mut b.nodes {
        let v = &mut n.def.vocab;
        v.sort_by(|a, c| {
            (&a.source, &a.term, &a.source_id).cmp(&(&c.source, &c.term, &c.source_id))
        });
        if v.iter().any(|l| l.source == "lcsh") {
            b.stats.classes_with_lcsh += 1;
        }
    }
}
