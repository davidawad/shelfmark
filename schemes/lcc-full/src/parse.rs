//! One Text PDF to the classes of its schedule.
//!
//! Page layout (all of it measured from the PDFs, not guessed):
//! - the header (`y > 735`) is three runs: the subclass letters (or a
//!   table name), a title, the letters again; the footer is the page number;
//! - each body line has a number run at `x < 100`, a caption run at
//!   `x ~ 149` whose indent is the count of leading no-break spaces (three
//!   per level), and scope-note runs at 9 pt;
//! - at the top of a continued page, the ancestors of the first entry are
//!   restated, the last one ending in `-- Continued`; those lines repeat
//!   what the previous page already said and are dropped.
//!
//! Hierarchy is the indentation: an entry's parent is the nearest
//! numbered line above it with a smaller indent whose span holds it.
//! Unnumbered lines are headings; they become the entry's `note` context
//! (`Societies > America`) since a scheme class needs a code.

use crate::code::{lead, Cover, Key, Lead};
use crate::pdf::Run;

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// Subclass letters from the page header this entry sits under.
    pub letters: String,
    pub code: String,
    pub label: String,
    pub range: Option<(String, String)>,
    /// Index of the parent entry in the same schedule, if any.
    pub parent: Option<usize>,
    pub context: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Report {
    pub pages: usize,
    pub lines: usize,
    pub numbered: usize,
    pub entries: usize,
    pub headings: usize,
    pub obsolete: usize,
    pub duplicates: usize,
    /// Lines in table pages, which are templates, not classes.
    pub table_lines: usize,
    pub reparented: usize,
    /// Table names skipped, in page order.
    pub tables: Vec<String>,
    pub failures: Vec<String>,
}

#[derive(Debug)]
pub struct Schedule {
    pub entries: Vec<Entry>,
    /// Subclass letters with their page-header titles, in page order.
    pub subclasses: Vec<(String, String)>,
    pub report: Report,
}

struct Line {
    page: usize,
    number: Option<String>,
    caption: Option<String>,
    indent: usize,
    notes: Vec<String>,
}

fn clean(s: &str) -> String {
    s.replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Group a page's runs into lines.
fn page_lines(page: usize, runs: &[Run], header: &mut Option<(String, String)>) -> Vec<Line> {
    let mut rows: Vec<(f32, Vec<&Run>)> = Vec::new();
    for r in runs {
        if r.y < 30.0 {
            continue;
        }
        if r.y > 735.0 {
            if clean(&r.text) == "INDEX" {
                *header = Some(("INDEX".into(), "INDEX".into()));
            } else if header.as_ref().is_some_and(|h| h.0 == "INDEX") {
            } else if r.x < 100.0 && header.is_none() {
                *header = Some((clean(&r.text), String::new()));
            } else if r.x > 100.0 && r.x < 400.0 {
                if let Some(h) = header.as_mut() {
                    if h.1.is_empty() {
                        h.1 = clean(&r.text);
                    }
                }
            }
            continue;
        }
        match rows.iter_mut().find(|(y, _)| (y - r.y).abs() < 0.6) {
            Some((_, v)) => v.push(r),
            None => rows.push((r.y, vec![r])),
        }
    }
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut out: Vec<Line> = Vec::new();
    for (_, mut v) in rows {
        v.sort_by(|a, b| a.x.total_cmp(&b.x));
        let mut line = Line {
            page,
            number: None,
            caption: None,
            indent: 0,
            notes: Vec::new(),
        };
        let mut note_parts = Vec::new();
        for r in v {
            if r.size < 9.5 {
                note_parts.push(clean(&r.text));
            } else if r.x < 100.0 {
                line.number = Some(clean(&r.text).replace(' ', ""));
            } else if line.caption.is_none() && r.x < 175.0 {
                let lead_sp = r.text.chars().take_while(|&c| c == '\u{a0}').count();
                line.indent = lead_sp / 3;
                line.caption = Some(clean(&r.text));
            } else {
                note_parts.push(clean(&r.text));
            }
        }
        let note = note_parts.join(" ");
        if line.number.is_none() && line.caption.is_none() {
            // A note line: belongs to the line above.
            match out.last_mut() {
                Some(prev) => prev.notes.push(note),
                None => out.push(Line {
                    notes: vec![note],
                    ..line
                }),
            }
        } else {
            if !note.is_empty() {
                line.notes.push(note);
            }
            // A number wrapped onto the next line (`128.K83732A-` then
            // `.K83732Z`): join the halves.
            let tail = line.caption.is_none() && line.number.is_some();
            match out.last_mut() {
                Some(prev)
                    if tail
                        && prev.number.as_deref().is_some_and(|n| n.ends_with('-')) =>
                {
                    prev.number.as_mut().unwrap().push_str(line.number.as_deref().unwrap());
                }
                _ => out.push(line),
            }
        }
    }
    out
}

fn is_continued(l: &Line) -> bool {
    l.caption.as_deref().is_some_and(|c| c.ends_with("-- Continued"))
}

pub fn parse(pages: &[Vec<Run>]) -> Schedule {
    let mut report = Report {
        pages: pages.len(),
        ..Report::default()
    };
    let mut entries: Vec<Entry> = Vec::new();
    let mut subclasses: Vec<(String, String)> = Vec::new();
    // (indent, entry index if numbered, heading text, is table-skipped)
    let mut stack: Vec<(usize, Option<usize>, String)> = Vec::new();
    let mut by_code: std::collections::BTreeMap<String, usize> = Default::default();
    let mut covers: Vec<Cover> = Vec::new();
    // Where the last line's notes should go.
    let mut last: Option<Option<usize>> = None;
    let mut letters = String::new();
    let mut in_table = false;

    for (pi, runs) in pages.iter().enumerate() {
        let mut header = None;
        let mut lines = page_lines(pi + 1, runs, &mut header);
        if let Some((h, title)) = header {
            if h == "INDEX" {
                in_table = true;
                if report.tables.last().map(String::as_str) != Some("INDEX") {
                    report.tables.push(h);
                }
            } else if h.chars().all(|c| c.is_ascii_uppercase()) && !h.is_empty() {
                if h != letters {
                    subclasses.push((h.clone(), title));
                    stack.clear();
                }
                letters = h;
                in_table = false;
            } else {
                in_table = true;
                if report.tables.last() != Some(&h) {
                    report.tables.push(h);
                }
            }
        }
        // Drop the restated ancestry at the top of a continued page.
        if let Some(j) = lines.iter().take(12).position(is_continued) {
            lines.drain(..=j);
        }
        for line in lines.into_iter().filter(|l| l.page > 0) {
            if in_table {
                report.table_lines += 1;
                continue;
            }
            if line.caption.is_none() && line.number.is_none() {
                if let (Some(Some(i)), Some(n)) = (last, line.notes.first()) {
                    entries[i].notes.push(n.clone());
                }
                continue;
            }
            report.lines += 1;
            let caption = line.caption.clone().unwrap_or_default();
            while stack.last().is_some_and(|s| s.0 >= line.indent) {
                stack.pop();
            }
            let mut idx: Option<usize> = None;
            if let Some(num) = &line.number {
                report.numbered += 1;
                if caption.is_empty() {
                    report
                        .failures
                        .push(format!("p{} `{}` (no caption)", line.page, num));
                    last = Some(None);
                    stack.push((line.indent, None, caption));
                    continue;
                }
                let optional = num.starts_with('<');
                let num = num.trim_start_matches('<').trim_end_matches('>');
                let text = with_letters(&letters, num, &caption);
                match lead(&text) {
                    Some(Lead::Span { start, end, .. }) => {
                        let code = Lead::code(&start, &end);
                        let cover = Cover::Span(Key::of(&start), Key::of(&end));
                        if let Some(&first) = by_code.get(&code) {
                            report.duplicates += 1;
                            idx = Some(first);
                        } else {
                            let ancestor = stack.iter().rev().find_map(|s| {
                                let i = s.1?;
                                covers[i].holds(&cover).then_some(i)
                            });
                            let nearest = stack.iter().rev().find_map(|s| s.1);
                            if ancestor != nearest {
                                report.reparented += 1;
                            }
                            let from = ancestor
                                .and_then(|a| stack.iter().rposition(|s| s.1 == Some(a)))
                                .map_or(0, |p| p + 1);
                            let context: Vec<String> = stack[from..]
                                .iter()
                                .filter(|s| s.1.is_none() && !s.2.is_empty())
                                .map(|s| s.2.clone())
                                .collect();
                            let range = (start != end).then(|| (start.render(), end.render()));
                            let i = entries.len();
                            by_code.insert(code.clone(), i);
                            covers.push(cover);
                            entries.push(Entry {
                                letters: letters.clone(),
                                code,
                                label: caption.clone(),
                                range,
                                parent: ancestor,
                                context,
                                notes: optional
                                    .then(|| "Optional number (angle brackets in the schedule)".to_string())
                                    .into_iter()
                                    .chain(line.notes.iter().cloned())
                                    .collect(),
                            });
                            idx = Some(i);
                        }
                    }
                    Some(Lead::Obsolete) => report.obsolete += 1,
                    _ => report
                        .failures
                        .push(format!("p{} `{}` {}", line.page, num, caption)),
                }
            } else {
                report.headings += 1;
            }
            last = Some(idx);
            if idx.is_none() && !line.notes.is_empty() {
                last = Some(None);
            }
            stack.push((line.indent, idx, caption));
        }
    }
    report.entries = entries.len();
    Schedule {
        entries,
        subclasses,
        report,
    }
}

/// Repair the end of a printed span so the outline number grammar reads it:
/// a bare cutter end (`8.A-Z`) is `.Z`, and an end whose letters are a
/// truncation of the subclass (`187.A-S187.Z` under SB) gets the subclass.
fn fix_end(letters: &str, num: &str) -> String {
    let Some(d) = num.char_indices().skip(1).find(|&(_, c)| c == '-').map(|(i, _)| i) else {
        return num.to_string();
    };
    let (a, b) = (&num[..d], &num[d + 1..]);
    if !b.is_empty() && b.chars().all(|c| c.is_ascii_uppercase()) && a.contains('.') {
        return format!("{a}-.{b}");
    }
    let run = b.chars().take_while(char::is_ascii_uppercase).count();
    let (l, rest) = b.split_at(run);
    if !l.is_empty()
        && l != letters
        && letters.starts_with(l)
        && rest.starts_with(|c: char| c.is_ascii_digit())
    {
        return format!("{a}-{letters}{rest}");
    }
    num.to_string()
}

/// The number as the outline parser wants it: with the subclass letters
/// in front, inside any opening paren.
fn with_letters(letters: &str, num: &str, caption: &str) -> String {
    let fixed = fix_end(letters, num);
    let num = fixed.as_str();
    let first = num.chars().next().unwrap_or(' ');
    let body = if first.is_ascii_digit() {
        format!("{letters}{num}")
    } else if matches!(first, '(' | '[') && num[1..].starts_with(|c: char| c.is_ascii_digit()) {
        format!("{first}{letters}{}", &num[1..])
    } else {
        num.to_string()
    };
    format!("{body} {caption}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const NB: &str = "\u{a0}\u{a0}\u{a0}";

    fn run(x: f32, y: f32, size: f32, text: &str) -> Run {
        Run {
            x,
            y,
            size,
            font: "F1".into(),
            text: text.into(),
        }
    }

    /// A page: header, then (number, indent levels, caption) rows; a row
    /// whose number is `"~"` is a 9 pt note.
    fn page(letters: &str, title: &str, rows: &[(&str, usize, &str)]) -> Vec<Run> {
        let mut out = vec![
            run(42.0, 745.0, 10.0, letters),
            run(148.0, 745.0, 10.0, title),
            run(568.0, 745.0, 10.0, letters),
            run(300.0, 14.0, 10.0, "1"),
        ];
        let mut y = 710.0;
        for (num, indent, cap) in rows {
            if *num == "~" {
                out.push(run(165.0, y, 9.0, cap));
            } else {
                if !num.is_empty() {
                    out.push(run(42.0, y, 10.0, num));
                }
                out.push(run(149.0, y, 10.0, &format!("{}{cap}", NB.repeat(*indent))));
            }
            y -= 11.7;
        }
        out
    }

    fn codes(s: &Schedule) -> Vec<(String, Option<String>)> {
        s.entries
            .iter()
            .map(|e| (e.code.clone(), e.parent.map(|p| s.entries[p].code.clone())))
            .collect()
    }

    #[test]
    fn indentation_gives_parents_and_context() {
        let p = page(
            "Q",
            "SCIENCE",
            &[
                ("1-9", 0, "Periodicals"),
                ("1", 1, "Polyglot"),
                ("", 0, "Societies"),
                ("~", 0, "Including works about societies"),
                ("", 1, "America"),
                ("11", 2, "United States"),
                ("80.A-Z", 1, "Other, A-Z"),
                ("80.C4", 2, "Ceylon"),
            ],
        );
        let s = parse(&[p]);
        assert_eq!(
            codes(&s),
            [
                ("Q1-9".to_string(), None),
                ("Q1".to_string(), Some("Q1-9".to_string())),
                ("Q11".to_string(), None),
                ("Q80.A-80.Z".to_string(), None),
                ("Q80.C4".to_string(), Some("Q80.A-80.Z".to_string())),
            ]
        );
        assert_eq!(s.entries[2].context, ["Societies", "America"]);
        assert_eq!(s.subclasses, [("Q".to_string(), "SCIENCE".to_string())]);
    }

    #[test]
    fn drops_restated_ancestry_and_skips_tables_and_index() {
        let p1 = page("Q", "SCIENCE", &[("1", 0, "General")]);
        let mut p2 = page(
            "Q",
            "SCIENCE",
            &[
                ("", 0, "Societies"),
                ("", 1, "Other -- Continued"),
                ("(91.S95)", 1, "Swaziland"),
                ("92", 1, "Chad"),
            ],
        );
        p2.push(run(1.0, 1.0, 10.0, "ignored footer"));
        let tbl = page("Q5", "TABLE", &[(".A2", 0, "Texts")]);
        let idx = vec![run(200.0, 745.0, 10.0, "INDEX"), run(42.0, 700.0, 10.0, "A")];
        let s = parse(&[p1, p2, tbl, idx]);
        assert_eq!(s.entries.len(), 2);
        assert_eq!(s.report.obsolete, 1);
        assert_eq!(s.report.table_lines, 2);
        assert_eq!(s.report.tables, ["Q5", "INDEX"]);
        assert!(s.report.failures.is_empty(), "{:?}", s.report.failures);
    }

    #[test]
    fn joins_wrapped_numbers_and_reads_optional_numbers() {
        let mut p = page("Q", "SCIENCE", &[("<75>", 0, "Optional")]);
        p.push(run(42.0, 500.0, 10.0, "5.A-"));
        p.push(run(149.0, 500.0, 10.0, "Wrapped"));
        p.push(run(42.0, 488.0, 10.0, ".B"));
        let s = parse(&[p]);
        assert_eq!(s.entries[0].code, "Q75");
        assert!(s.entries[0].notes[0].starts_with("Optional number"));
        assert_eq!(s.entries[1].code, "Q5.A-5.B");
    }

    #[test]
    fn repairs_span_ends() {
        assert_eq!(fix_end("Z", "8.A-Z"), "8.A-.Z");
        assert_eq!(fix_end("SB", "187.A-S187.Z"), "187.A-SB187.Z");
        assert_eq!(fix_end("QA", "75-76.95"), "75-76.95");
    }

    #[test]
    fn unreadable_numbers_are_failures_not_classes() {
        let p = page("D", "HISTORY", &[("546.55th", 0, "The Fifth Army"), ("1", 0, "Ok")]);
        let s = parse(&[p]);
        assert_eq!(s.entries.len(), 1);
        assert_eq!(s.report.failures.len(), 1);
    }
}
