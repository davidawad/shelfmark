//! Assemble the scheme: every section, class, subclass, main group and
//! subgroup of the master file, in its order and under its parent, with
//! catchwords and guidance headings as vocabulary.

use std::collections::{BTreeSet, HashMap};

use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, VocabLink, SCHEME_SCHEMA_ID};

use crate::catchword::{Catchword, Target};
use crate::master::{Entry, Parsed};
use crate::sources::{Lock, Role};
use crate::symbol::CODE_GRAMMAR;

pub const CATCHWORD: &str = "ipc-catchword";
pub const HEADING: &str = "ipc-heading";
/// Guidance headings cover a run of groups, so they weigh less than a
/// catchword, which WIPO points at one place.
const HEADING_WEIGHT: f64 = 0.5;

/// What the build kept and dropped, for SOURCES.md.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub sections: usize,
    pub classes: usize,
    pub subclasses: usize,
    pub main_groups: usize,
    pub subgroups: usize,
    pub indexing: usize,
    pub skipped: usize,
    pub images: usize,
    pub catchwords: usize,
    pub catchword_links: usize,
    /// Catchword targets with no class in the scheme.
    pub unresolved: Vec<String>,
    pub headings: usize,
    pub heading_links: usize,
}

/// The scheme document and its stats.
pub fn build(
    parsed: &Parsed,
    words: &[Catchword],
    lock: &Lock,
) -> Result<(SchemeDoc, Stats), String> {
    let mut stats = Stats {
        skipped: parsed.skipped,
        images: parsed.images,
        ..Stats::default()
    };
    let entries = &parsed.entries;
    let mut pos: HashMap<&str, usize> = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        if pos.insert(&e.code, i).is_some() {
            return Err(format!("duplicate symbol `{}`", e.code));
        }
        count(&mut stats, e);
    }
    let mut vocab: Vec<Vec<VocabLink>> = vec![Vec::new(); entries.len()];
    let mut seen = BTreeSet::new();
    let mut add = |i: usize, link: VocabLink, seen: &mut BTreeSet<_>| {
        let fresh = seen.insert((i, link.source.clone(), link.term.clone()));
        if fresh {
            vocab[i].push(link);
        }
        fresh
    };
    for w in words {
        stats.catchwords += 1;
        for t in &w.targets {
            let Some(hit) = resolve(entries, &pos, t) else {
                stats.unresolved.push(match t {
                    Target::Code(c) => c.clone(),
                    Target::Span(a, b) => format!("{a}-{b}"),
                });
                continue;
            };
            for i in hit {
                let link = VocabLink {
                    term: w.term.clone(),
                    source: CATCHWORD.to_string(),
                    source_id: Some(w.id.clone()),
                    weight: 1.0,
                };
                stats.catchword_links += usize::from(add(i, link, &mut seen));
            }
        }
    }
    for h in &parsed.headings {
        stats.headings += 1;
        let span = Target::Span(h.start.clone(), h.end.clone());
        let hit = resolve(entries, &pos, &span)
            .ok_or_else(|| format!("guidance heading `{}` spans unknown groups", h.text))?;
        for i in hit {
            let link = VocabLink {
                term: h.text.clone(),
                source: HEADING.to_string(),
                source_id: None,
                weight: HEADING_WEIGHT,
            };
            stats.heading_links += usize::from(add(i, link, &mut seen));
        }
    }
    stats.unresolved.sort();
    stats.unresolved.dedup();
    let classes = entries
        .iter()
        .zip(vocab)
        .map(|(e, vocab)| ClassDef {
            code: e.code.clone(),
            label: e.label.clone(),
            parent: e.parent.clone(),
            range: None,
            vocab,
            note: note(e),
        })
        .collect();
    let doc = SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: "ipc".to_string(),
        version: parsed.version.clone(),
        title: "International Patent Classification (IPC)".to_string(),
        source: Some(source(lock)?),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    };
    Ok((doc, stats))
}

fn count(s: &mut Stats, e: &Entry) {
    let n = match e.code.len() {
        1 => &mut s.sections,
        3 => &mut s.classes,
        4 => &mut s.subclasses,
        _ if e.code.ends_with("/00") => &mut s.main_groups,
        _ => &mut s.subgroups,
    };
    *n += 1;
    s.indexing += usize::from(e.indexing);
}

/// The classes a catchword or heading target names: one code, or the
/// top-most entries from `start` to `end` in scheme order (an entry whose
/// ancestor is already in the span is covered by it). A span may run from
/// a main group into its subgroups (`A23K40/00-A23K40/25`).
fn resolve(entries: &[Entry], pos: &HashMap<&str, usize>, t: &Target) -> Option<Vec<usize>> {
    match t {
        Target::Code(c) => pos.get(c.as_str()).map(|&i| vec![i]),
        Target::Span(a, b) => {
            let (&i, &j) = (pos.get(a.as_str())?, pos.get(b.as_str())?);
            if i > j {
                return None;
            }
            let mut covered: BTreeSet<&str> = BTreeSet::new();
            let mut top = Vec::new();
            for (k, e) in entries.iter().enumerate().take(j + 1).skip(i) {
                if !e.parent.as_deref().is_some_and(|p| covered.contains(p)) {
                    top.push(k);
                }
                covered.insert(&e.code);
            }
            Some(top)
        }
    }
}

fn note(e: &Entry) -> Option<String> {
    let mut parts = Vec::new();
    if e.indexing {
        parts.push("Indexing scheme entry".to_string());
    }
    if !e.references.is_empty() {
        parts.push(format!("References: {}", e.references.join("; ")));
    }
    (!parts.is_empty()).then(|| parts.join(". "))
}

fn source(lock: &Lock) -> Result<Source, String> {
    let scheme = lock.get(Role::Scheme)?;
    let words = lock.get(Role::Catchwords)?;
    Ok(Source {
        description: format!(
            "Every section, class, subclass, main group and subgroup of IPC edition {ed}, \
             titles verbatim (title parts joined by `; `, symbol references in compact form, \
             images as `[image]`), parents from the master file's nesting. Vocabulary: WIPO \
             catchword index entries (`{CATCHWORD}`) and guidance headings (`{HEADING}`). \
             A group's code is the printed symbol without its space (`A01B 1/02` is `A01B1/02`). \
             Sources: {s} (sha256 {ss}; {sm} sha256 {sms}) and {w} (sha256 {ws}; {wm} sha256 {wms}). \
             Built by schemes/ipc/builder; see schemes/ipc/SOURCES.md.",
            ed = lock.edition,
            s = scheme.url,
            ss = scheme.sha256,
            sm = scheme.member,
            sms = scheme.member_sha256,
            w = words.url,
            ws = words.sha256,
            wm = words.member,
            wms = words.member_sha256,
        ),
        url: Some("https://www.wipo.int/classifications/ipc/en/".to_string()),
        license: Some(
            "CC BY 4.0 (WIPO Open Access Policy); source: International Patent \
             Classification, World Intellectual Property Organization (WIPO). Adapted: \
             reformatted, not endorsed by WIPO"
                .to_string(),
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(code: &str, parent: Option<&str>) -> Entry {
        Entry {
            code: code.into(),
            label: code.into(),
            parent: parent.map(Into::into),
            references: Vec::new(),
            indexing: false,
        }
    }

    #[test]
    fn spans_resolve_to_their_top_most_entries() {
        let entries = [
            entry("A01B", None),
            entry("A01B1/00", Some("A01B")),
            entry("A01B1/02", Some("A01B1/00")),
            entry("A01B1/04", Some("A01B1/00")),
            entry("A01B1/06", Some("A01B1/04")),
            entry("A01B3/00", Some("A01B")),
            entry("A01B5/00", Some("A01B")),
        ];
        let pos: HashMap<&str, usize> = entries
            .iter()
            .enumerate()
            .map(|(i, e)| (e.code.as_str(), i))
            .collect();
        let span = |a: &str, b: &str| resolve(&entries, &pos, &Target::Span(a.into(), b.into()));
        assert_eq!(span("A01B1/00", "A01B3/00"), Some(vec![1, 5]));
        assert_eq!(span("A01B1/00", "A01B1/04"), Some(vec![1]));
        assert_eq!(span("A01B1/02", "A01B3/00"), Some(vec![2, 3, 5]));
        assert_eq!(span("A01B3/00", "A01B1/00"), None);
        assert_eq!(span("A01B1/00", "A01B9/00"), None);
        let one = resolve(&entries, &pos, &Target::Code("A01B1/06".into()));
        assert_eq!(one, Some(vec![4]));
    }

    #[test]
    fn notes_mark_indexing_entries_and_carry_references() {
        let mut e = entry("C12R1/01", Some("C12R1/00"));
        assert_eq!(note(&e), None);
        e.indexing = true;
        e.references = vec!["waveguides H01P".into(), "cables H01B".into()];
        assert_eq!(
            note(&e).as_deref(),
            Some("Indexing scheme entry. References: waveguides H01P; cables H01B")
        );
    }
}
