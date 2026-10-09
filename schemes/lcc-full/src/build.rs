//! The schedules' entries merged into one scheme.
//!
//! The bundled outline scheme (`lcc@2024`) supplies the main classes, the
//! subclasses and every span the outline PDFs print, with their labels,
//! nesting and vocabulary. Each Text entry whose code the outline already
//! has keeps the outline class and gains the schedule's context and scope
//! notes; every other entry becomes a new class under the parent the
//! schedule's indentation gives it (or, for a top-level entry, the deepest
//! outline span that holds it, else its subclass).

use std::collections::BTreeMap;
use std::fmt::Write;

use shelfmark_core::scheme::{ClassDef, CodeRange, SchemeDoc, Source, SCHEME_SCHEMA_ID};

use crate::code::{Cover, Key, CODE_GRAMMAR};
use crate::parse::{Entry, Schedule};
use crate::source::{self, Lock};

/// Per-schedule counts for the coverage report.
#[derive(Debug, Default, Clone)]
pub struct Row {
    pub name: String,
    pub pages: usize,
    pub numbered: usize,
    pub parsed: usize,
    pub matched: usize,
    pub added: usize,
    pub cross_dup: usize,
    pub subclasses_added: usize,
    pub obsolete: usize,
    pub dup: usize,
    pub table_lines: usize,
    pub failures: usize,
    pub tables: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub classes: usize,
    pub depth: usize,
    pub max_branching: usize,
    pub mean_branching: f64,
    pub outline_only: usize,
}

pub struct Built {
    pub doc: SchemeDoc,
    pub rows: Vec<Row>,
    pub stats: Stats,
    /// Classes whose indentation parent failed the range check, moved up.
    pub reparented: Vec<String>,
}

fn sentence_case(title: &str) -> String {
    let lower = title.to_lowercase();
    let mut c = lower.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn note(e: &Entry) -> Option<String> {
    let mut parts = Vec::new();
    if !e.context.is_empty() {
        parts.push(format!("Under: {}", e.context.join(" > ")));
    }
    parts.extend(e.notes.iter().filter(|n| !n.is_empty()).cloned());
    (!parts.is_empty()).then(|| parts.join(". "))
}

fn cover_of(c: &ClassDef) -> Cover {
    match &c.range {
        Some(r) => Cover::Span(Key::parse(&r.start), Key::parse(&r.end)),
        None => Cover::Span(Key::parse(&c.code), Key::parse(&c.code)),
    }
}

/// Merge `schedules` (name, parsed) into the outline.
pub fn build(
    schedules: &[(String, Schedule)],
    outline: &SchemeDoc,
    lock: &Lock,
) -> Result<Built, String> {
    let mut classes: Vec<ClassDef> = outline.classes.clone();
    let outline_len = classes.len();
    let mut index: BTreeMap<String, usize> = classes
        .iter()
        .enumerate()
        .map(|(i, c)| (c.code.clone(), i))
        .collect();
    let covers: Vec<Cover> = classes.iter().map(cover_of).collect();
    let depth: Vec<usize> = (0..classes.len())
        .map(|i| {
            let (mut d, mut cur) = (0, i);
            while let Some(p) = classes[cur].parent.as_ref().and_then(|p| index.get(p)) {
                d += 1;
                cur = *p;
            }
            d
        })
        .collect();
    let mut touched = vec![false; outline_len];
    let mut rows = Vec::new();
    // Candidate holders per subclass letters, built lazily.
    let mut holders: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for (name, sched) in schedules {
        let mut row = Row {
            name: name.clone(),
            pages: sched.report.pages,
            numbered: sched.report.numbered,
            parsed: sched.entries.len(),
            obsolete: sched.report.obsolete,
            dup: sched.report.duplicates,
            table_lines: sched.report.table_lines,
            failures: sched.report.failures.len(),
            tables: sched.report.tables.len(),
            ..Row::default()
        };
        // Subclasses the outline lacks.
        for (letters, title) in &sched.subclasses {
            if index.contains_key(letters) {
                continue;
            }
            let parent = (1..letters.len())
                .rev()
                .map(|n| &letters[..n])
                .find(|p| index.contains_key(*p))
                .map(str::to_string);
            let i = classes.len();
            classes.push(ClassDef {
                code: letters.clone(),
                label: sentence_case(title),
                parent,
                range: None,
                vocab: Vec::new(),
                note: None,
            });
            index.insert(letters.clone(), i);
            row.subclasses_added += 1;
        }
        let mut fresh: Vec<(usize, String)> = Vec::new();
        for e in &sched.entries {
            match index.get(&e.code) {
                Some(&i) if i < outline_len => {
                    if !touched[i] {
                        touched[i] = true;
                        row.matched += 1;
                        if classes[i].note.is_none() {
                            classes[i].note = note(e);
                        }
                    }
                }
                Some(_) => row.cross_dup += 1,
                None => {
                    let i = classes.len();
                    classes.push(ClassDef {
                        code: e.code.clone(),
                        label: e.label.clone(),
                        parent: e.parent.map(|p| sched.entries[p].code.clone()),
                        range: e
                            .range
                            .as_ref()
                            .map(|(s, t)| CodeRange {
                                start: s.clone(),
                                end: t.clone(),
                            }),
                        vocab: Vec::new(),
                        note: note(e),
                    });
                    index.insert(e.code.clone(), i);
                    fresh.push((i, e.letters.clone()));
                    row.added += 1;
                }
            }
        }
        // Top-level entries: the deepest outline span that holds them.
        for (i, letters) in fresh {
            if classes[i].parent.is_some() {
                continue;
            }
            let list = holders.entry(letters.clone()).or_insert_with(|| {
                (0..outline_len)
                    .filter(|&k| classes[k].range.is_some() && classes[k].code.starts_with(letters.as_str()))
                    .collect()
            });
            let mine = cover_of(&classes[i]);
            let best = list
                .iter()
                .copied()
                .filter(|&k| covers[k].holds(&mine))
                .max_by_key(|&k| depth[k]);
            classes[i].parent = match best {
                Some(k) => Some(classes[k].code.clone()),
                None => (0..=letters.len())
                    .rev()
                    .map(|n| &letters[..n])
                    .find(|p| !p.is_empty() && index.contains_key(*p) && *p != classes[i].code)
                    .map(str::to_string),
            };
        }
        rows.push(row);
    }
    let outline_only = touched.iter().filter(|t| !**t).count();

    let mut doc = SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: "lcc-full".to_string(),
        version: outline.version.clone(),
        title: "Library of Congress Classification: complete schedules".to_string(),
        source: Some(Source {
            description: describe(lock),
            url: Some(source::PAGE.to_string()),
            license: Some(
                "Library of Congress data: US government work, not subject to copyright in the United States (17 U.S.C. 105). Credit: Library of Congress Classification, Library of Congress, Policy, Training, and Cooperative Programs Division."
                    .to_string(),
            ),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    };
    let reparented = repair(&mut doc)?;
    order(&mut doc);
    let stats = stats(&doc, outline_only);
    Ok(Built {
        doc,
        rows,
        stats,
        reparented,
    })
}

fn describe(lock: &Lock) -> String {
    format!(
        "Every numbered entry of the {n} FreeLCC 2024 Text schedule PDFs ({page}), \
         layered on the lcc@2024 outline scheme (outline sha256 {o}): the outline's classes, \
         labels, nesting and vocabulary are kept; each Text entry is added with its caption as \
         the label, its indentation as the hierarchy, and its unnumbered ancestor headings and \
         scope notes as the note. Codes use the lcc@2024 grammar, so ids agree. Not included: \
         the form-division and cutter tables (templates applied under other classes), the \
         schedule indexes, numbers printed in parentheses or brackets (no longer used), and the \
         few entries listed in COVERAGE.md that the number grammar cannot read. \
         Built by schemes/lcc-full (`lcc-full build`).",
        n = lock.sources.len(),
        page = source::PAGE,
        o = lock.outline_sha256,
    )
}

/// Clear any parent a range check rejects (set it to the deepest holder,
/// else drop to the subclass or main class).
fn repair(doc: &mut SchemeDoc) -> Result<Vec<String>, String> {
    let mut log = Vec::new();
    for _ in 0..4 {
        let Err(diags) = shelfmark_scheme::validate_value(
            &serde_json::to_value(&*doc).map_err(|e| e.to_string()).unwrap_or_default(),
        ) else {
            return Ok(log);
        };
        let index: BTreeMap<String, usize> = doc
            .classes
            .iter()
            .enumerate()
            .map(|(i, c)| (c.code.clone(), i))
            .collect();
        let mut fixed = false;
        for d in diags {
            let Some(i) = d
                .pointer
                .strip_prefix("/classes/")
                .and_then(|r| r.split('/').next())
                .and_then(|n| n.parse::<usize>().ok())
            else {
                continue;
            };
            if !d.pointer.ends_with("/range") && !d.pointer.ends_with("/parent") {
                continue;
            }
            let code = doc.classes[i].code.clone();
            // Walk up the ancestors until one holds this class.
            let mine = cover_of(&doc.classes[i]);
            let mut cur = doc.classes[i].parent.clone();
            let mut found = None;
            let mut guard = 0;
            while let Some(c) = cur {
                guard += 1;
                let Some(&k) = index.get(&c) else { break };
                if guard > 64 {
                    break;
                }
                if k != i && cover_of(&doc.classes[k]).holds(&mine) {
                    found = Some(c);
                    break;
                }
                cur = doc.classes[k].parent.clone();
            }
            if found != doc.classes[i].parent {
                log.push(format!(
                    "{code}: {} -> {}",
                    doc.classes[i].parent.as_deref().unwrap_or("(none)"),
                    found.as_deref().unwrap_or("(none)")
                ));
                doc.classes[i].parent = found;
                fixed = true;
            }
        }
        if !fixed {
            return Err(format!("the scheme stays invalid: {}", first_problem(doc)));
        }
    }
    Err(format!("the scheme stays invalid: {}", first_problem(doc)))
}

fn first_problem(doc: &SchemeDoc) -> String {
    let v = serde_json::to_value(doc).unwrap_or_default();
    match shelfmark_scheme::validate_value(&v) {
        Err(d) => d
            .first()
            .map(|d| format!("{}: {}", d.pointer, d.message))
            .unwrap_or_default(),
        Ok(_) => String::new(),
    }
}

/// Shelf order: depth first, siblings by range start, wider span first.
fn order(doc: &mut SchemeDoc) {
    let index: BTreeMap<&str, usize> = doc
        .classes
        .iter()
        .enumerate()
        .map(|(i, c)| (c.code.as_str(), i))
        .collect();
    let keys: Vec<(Key, Key)> = doc
        .classes
        .iter()
        .map(|c| match &c.range {
            Some(r) => (Key::parse(&r.start), Key::parse(&r.end)),
            None => (Key::parse(&c.code), Key::parse(&c.code)),
        })
        .collect();
    let mut kids: Vec<Vec<usize>> = vec![Vec::new(); doc.classes.len()];
    let mut roots = Vec::new();
    for (i, c) in doc.classes.iter().enumerate() {
        match c.parent.as_deref().and_then(|p| index.get(p)) {
            Some(&p) => kids[p].push(i),
            None => roots.push(i),
        }
    }
    let cmp = |a: &usize, b: &usize| {
        keys[*a]
            .0
            .cmp(&keys[*b].0)
            .then_with(|| keys[*b].1.cmp(&keys[*a].1))
            .then_with(|| doc.classes[*a].code.cmp(&doc.classes[*b].code))
    };
    roots.sort_by(|a, b| doc.classes[*a].code.cmp(&doc.classes[*b].code));
    for k in &mut kids {
        k.sort_by(cmp);
    }
    let mut out = Vec::with_capacity(doc.classes.len());
    let mut stack: Vec<usize> = roots.into_iter().rev().collect();
    while let Some(i) = stack.pop() {
        out.push(i);
        stack.extend(kids[i].iter().rev());
    }
    let mut old: Vec<Option<ClassDef>> = std::mem::take(&mut doc.classes).into_iter().map(Some).collect();
    doc.classes = out.into_iter().filter_map(|i| old[i].take()).collect();
}

fn stats(doc: &SchemeDoc, outline_only: usize) -> Stats {
    let index: BTreeMap<&str, usize> = doc
        .classes
        .iter()
        .enumerate()
        .map(|(i, c)| (c.code.as_str(), i))
        .collect();
    let mut kids = vec![0usize; doc.classes.len()];
    let mut depth = vec![0usize; doc.classes.len()];
    // Parents precede children in shelf order, so one pass fills depth.
    for (i, c) in doc.classes.iter().enumerate() {
        if let Some(&p) = c.parent.as_deref().and_then(|p| index.get(p)) {
            kids[p] += 1;
            depth[i] = depth[p] + 1;
        }
    }
    let inner: Vec<usize> = kids.iter().copied().filter(|&k| k > 0).collect();
    Stats {
        classes: doc.classes.len(),
        depth: depth.iter().copied().max().unwrap_or(0) + 1,
        max_branching: inner.iter().copied().max().unwrap_or(0),
        mean_branching: inner.iter().sum::<usize>() as f64 / inner.len().max(1) as f64,
        outline_only,
    }
}

/// The scheme as JSON, one class per line (the layout of `lcc.json`).
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

/// The coverage report as markdown.
pub fn coverage(built: &Built, schedules: &[(String, Schedule)]) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "# lcc-full coverage\n");
    let _ = writeln!(s, "Generated by `lcc-full build`; do not edit by hand.\n");
    let st = &built.stats;
    let _ = writeln!(
        s,
        "Scheme `lcc-full@2024`: {} classes, depth {}, branching max {} / mean {:.2}; {} outline classes have no Text entry (kept as is).\n",
        st.classes, st.depth, st.max_branching, st.mean_branching, st.outline_only
    );
    let _ = writeln!(s, "| schedule | pages | numbered lines | parsed | matched outline | added | cross-schedule dup | subclasses added | no longer used (skipped) | dup in schedule | failures | table pages (skipped) | table lines (skipped) |");
    let _ = writeln!(s, "|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    let mut t = Row::default();
    for r in &built.rows {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            r.name, r.pages, r.numbered, r.parsed, r.matched, r.added, r.cross_dup,
            r.subclasses_added, r.obsolete, r.dup, r.failures, r.tables, r.table_lines
        );
        t.pages += r.pages;
        t.numbered += r.numbered;
        t.parsed += r.parsed;
        t.matched += r.matched;
        t.added += r.added;
        t.cross_dup += r.cross_dup;
        t.subclasses_added += r.subclasses_added;
        t.obsolete += r.obsolete;
        t.dup += r.dup;
        t.failures += r.failures;
        t.tables += r.tables;
        t.table_lines += r.table_lines;
    }
    let _ = writeln!(
        s,
        "| **total** | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
        t.pages, t.numbered, t.parsed, t.matched, t.added, t.cross_dup, t.subclasses_added,
        t.obsolete, t.dup, t.failures, t.tables, t.table_lines
    );
    let _ = writeln!(s, "## Moved by the range check\n");
    let _ = writeln!(
        s,
        "{} classes whose parent (from the outline or the schedule's indentation) did not hold their range, as shelfmark's validator checks it, were moved to the nearest ancestor that does (`code: old parent -> new parent`). Mostly subclasses whose outline span is narrower than the Text schedule, and the schedules' own typos.\n",
        built.reparented.len()
    );
    for r in &built.reparented {
        let _ = writeln!(s, "- `{r}`");
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "## Parse failures\n");
    let _ = writeln!(
        s,
        "Numbered lines the number grammar cannot read, with the first {SHOWN} per schedule (`page` is the PDF page; the complete list is `coverage-failures.txt`).\n"
    );
    for (name, sched) in schedules {
        let f = &sched.report.failures;
        if f.is_empty() {
            continue;
        }
        let _ = writeln!(s, "### {name} ({})\n", f.len());
        for line in f.iter().take(SHOWN) {
            let _ = writeln!(s, "- `{}`", line.replace('`', "'"));
        }
        let _ = writeln!(s);
    }
    s
}

const SHOWN: usize = 8;

/// Every failure, one per line, prefixed by its schedule.
pub fn failures(schedules: &[(String, Schedule)]) -> String {
    let mut s = String::new();
    for (name, sched) in schedules {
        for f in &sched.report.failures {
            let _ = writeln!(s, "{name}\t{f}");
        }
    }
    s
}
