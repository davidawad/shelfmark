//! Outline entries out of the FreeLCC outline PDFs' lines.
//!
//! Each page opens with running headers (the captions of the classes the
//! page continues, often ending `- Continued`); they come before the
//! page's first numbered line and are dropped. After a numbered line, a
//! line with no number continues its caption, or, from `Including` on, is
//! its scope note: always within the same paragraph, and from a later
//! paragraph only when the text so far stops mid-phrase. A blank line
//! ends that. A number printed with no
//! caption (some pages set numbers and captions as separate columns)
//! takes no continuation; the build captions it from id.loc.gov. Lines in other scripts (the
//! Hebrew forms of BM captions) are dropped, as are K table lines, spans
//! no longer used (in parens or brackets) and everything that continues
//! them.

use std::collections::VecDeque;
use std::sync::OnceLock;

use regex::Regex;

use crate::code::{lead, Lead, Number};

/// One numbered outline line.
#[derive(Debug, Clone)]
pub struct Entry {
    pub start: Number,
    pub end: Number,
    pub caption: String,
    pub notes: Vec<String>,
}

/// A letters-only subclass heading (`KJG Albania`).
#[derive(Debug, Clone)]
pub struct Heading {
    pub letters: String,
    pub caption: String,
}

/// What a set of outline pages yields, with counts of what was dropped.
#[derive(Debug, Default)]
pub struct Parsed {
    pub entries: Vec<Entry>,
    pub headings: Vec<Heading>,
    pub obsolete: usize,
    pub table_lines: usize,
    pub unparsed: Vec<String>,
    /// Lines that start like a number but do not parse.
    pub bad_numbers: Vec<String>,
}

enum Last {
    None,
    Entry,
    Heading,
    Skipped,
}

/// Strip control characters (the PDFs carry stray `\u{2}` and `\u{80}`).
pub fn clean(text: &str) -> String {
    text.chars()
        .filter(|&c| c == '\n' || !c.is_control())
        .collect()
}

pub fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Parse one PDF's pages (each a list of paragraphs) into `parsed`.
pub fn parse(pages: &[Vec<String>], parsed: &mut Parsed) {
    for page in pages {
        let mut st = State {
            seen_number: false,
            last: Last::None,
            in_note: false,
            new_para: false,
        };
        for para in page {
            let para = clean(para);
            if para.trim().is_empty() {
                st.last = Last::None;
                continue;
            }
            // Lines are handled one by one; a number may wrap only in the
            // paragraph's leading code, which `lead` reads across the wrap.
            let mut queue: VecDeque<String> = VecDeque::from([para]);
            st.new_para = true;
            while let Some(mut unit) = queue.pop_front() {
                // A drop capital set apart from its word: `I` then
                // `ncluding adventurers`.
                let (head, tail) = unit.split_once('\n').unwrap_or((&unit, ""));
                let cap = head.trim();
                if cap.len() == 1
                    && cap.chars().all(|c| c.is_ascii_uppercase())
                    && tail.starts_with(char::is_lowercase)
                {
                    unit = format!("{cap}{tail}");
                }
                let rest = st.unit(&unit, parsed);
                st.new_para = false;
                if let Some((_, more)) = rest.split_once('\n') {
                    queue.push_front(more.to_string());
                }
            }
        }
    }
}

struct State {
    seen_number: bool,
    last: Last,
    in_note: bool,
    /// The line being handled starts a paragraph.
    new_para: bool,
}

/// Whether a line from a new paragraph continues `prev`: it starts in
/// lowercase or with `(`, it is one word after a longer caption cut off
/// mid-phrase (`... of the European` then `Communities`), or `prev` stops
/// on punctuation or a function word. A cross-reference (`Bahamas, see
/// F1650+`) never does. (`Nunavut` then `Other than Canada` is a new
/// heading; `Business associations.  Personal companies.` then `Stock
/// corporations` is one caption.)
fn continues(prev: &str, t: &str) -> bool {
    const OPEN: [&str; 12] = [
        "and", "or", "of", "the", "in", "for", "to", "with", "on", "by", "a", "an",
    ];
    if t.contains("see ") {
        return false;
    }
    let one_word = !t.contains(' ') && t.chars().all(|c| c.is_alphabetic() || c == ')');
    t.starts_with(|c: char| c.is_lowercase() || c == '(')
        || (one_word && prev.ends_with(|c: char| c.is_alphabetic()) && prev.split(' ').count() >= 3)
        || prev.ends_with([',', '-', '.', ':', ';'])
        || prev
            .rsplit(' ')
            .next()
            .is_some_and(|w| OPEN.contains(&w.to_lowercase().as_str()))
}

fn numbered() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\s*[\[(]?[A-Z]{1,3}\(?[0-9]").expect("static regex"))
}

impl State {
    /// Handle the first line of `unit` (which may span several lines);
    /// returns the text whose later lines are still to be handled.
    fn unit<'a>(&mut self, unit: &'a str, parsed: &mut Parsed) -> std::borrow::Cow<'a, str> {
        let first = |text: &str| collapse(text.split('\n').next().unwrap_or_default());
        match lead(unit) {
            Some(Lead::Span { start, end, rest }) => {
                self.seen_number = true;
                self.in_note = false;
                let caption = first(&rest);
                parsed.entries.push(Entry {
                    start,
                    end,
                    caption,
                    notes: Vec::new(),
                });
                self.last = Last::Entry;
                return rest.into();
            }
            Some(Lead::Letters { letters, rest }) if letters.len() >= 2 => {
                self.seen_number = true;
                let caption = first(&rest);
                parsed.headings.push(Heading { letters, caption });
                self.last = Last::Heading;
                return rest.into();
            }
            Some(Lead::Obsolete) => {
                parsed.obsolete += 1;
                self.seen_number = true;
                self.last = Last::Skipped;
                return unit.into();
            }
            Some(Lead::Table) => {
                parsed.table_lines += 1;
                self.seen_number = true;
                self.last = Last::Skipped;
                return unit.into();
            }
            _ => {}
        }
        let t = first(unit);
        if numbered().is_match(&t) {
            // A number we cannot read (e.g. a span whose start sorts after
            // its end): drop it and what continues it.
            parsed.bad_numbers.push(t);
            self.seen_number = true;
            self.last = Last::Skipped;
        } else {
            self.text(t, parsed);
        }
        unit.into()
    }

    /// A line with no number: a running header, a caption or note
    /// continuation, or an orphan heading.
    fn text(&mut self, t: String, parsed: &mut Parsed) {
        if t.is_empty() {
            self.last = Last::None;
            self.in_note = false;
            return;
        }
        if t.ends_with("Continued")
            || !self.seen_number
            || !t.chars().any(|c| c.is_ascii_alphabetic())
        {
            return;
        }
        let prev = match self.last {
            Last::Entry => {
                let e = parsed.entries.last().expect("last entry");
                match e.notes.last().filter(|_| self.in_note) {
                    Some(n) => n.as_str(),
                    None => e.caption.as_str(),
                }
            }
            Last::Heading => parsed.headings.last().map_or("", |h| h.caption.as_str()),
            _ => "",
        };
        let note_start = matches!(self.last, Last::Entry) && t.starts_with("Including");
        if self.new_para
            && !note_start
            && !continues(prev, &t)
            && !matches!(self.last, Last::Skipped)
        {
            self.last = Last::None;
            self.in_note = false;
        }
        match self.last {
            Last::Entry => {
                let e = parsed.entries.last_mut().expect("last entry");
                if e.caption.is_empty() {
                    // Numbers printed as a column, captions apart: this
                    // line is not the number's caption.
                    parsed.unparsed.push(t);
                    self.last = Last::None;
                } else if t.starts_with("Including") {
                    self.in_note = true;
                    if t != "Including:" {
                        e.notes.push(t);
                    }
                } else if self.in_note {
                    match e.notes.last_mut() {
                        Some(n) => {
                            n.push(' ');
                            n.push_str(&t);
                        }
                        None => e.notes.push(t),
                    }
                } else {
                    e.caption = collapse(&format!("{} {t}", e.caption));
                }
            }
            Last::Heading => {
                let h = parsed.headings.last_mut().expect("last heading");
                h.caption = collapse(&format!("{} {t}", h.caption));
            }
            Last::Skipped => {}
            Last::None => parsed.unparsed.push(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(pages: &[&[&str]]) -> Parsed {
        let pages: Vec<Vec<String>> = pages
            .iter()
            .map(|p| p.iter().map(|s| s.to_string()).collect())
            .collect();
        let mut parsed = Parsed::default();
        parse(&pages, &mut parsed);
        parsed
    }

    fn captions(p: &Parsed) -> Vec<(String, String)> {
        p.entries
            .iter()
            .map(|e| (crate::code::Lead::code(&e.start, &e.end), e.caption.clone()))
            .collect()
    }

    #[test]
    fn joins_wraps_notes_and_drops_headers() {
        let p = run(&[&[
            "Running header - Continued",
            "KJ1040 Business associations.  Personal companies. ",
            "Stock corporations ",
            "QC170-197 Atomic physics",
            "Including molecular physics, relativity,",
            "quantum theory",
            "",
            "F1141-1145.2 Nunavut",
            "Other than Canada",
            "BM497-509 Mishnah \nmore",
            " הנשמ",
            "CT9960-9998 Other miscellaneous groups",
            "I\nncluding misers",
            "GC160-177 Temperature",
            "Oceanography",
            "KJE4441-5520 Constitution of the European",
            "Communities",
        ]]);
        assert_eq!(
            captions(&p),
            vec![
                (
                    "KJ1040".into(),
                    "Business associations. Personal companies. Stock corporations".into()
                ),
                ("QC170-197".into(), "Atomic physics".into()),
                ("F1141-1145.2".into(), "Nunavut".into()),
                ("BM497-509".into(), "Mishnah more".into()),
                ("CT9960-9998".into(), "Other miscellaneous groups".into()),
                ("GC160-177".into(), "Temperature".into()),
                (
                    "KJE4441-5520".into(),
                    "Constitution of the European Communities".into()
                ),
            ]
        );
        assert_eq!(p.entries[4].notes, ["Including misers"]);
        assert_eq!(
            p.entries[1].notes,
            ["Including molecular physics, relativity, quantum theory"]
        );
        assert_eq!(p.unparsed, ["Other than Canada", "Oceanography"]);
    }

    #[test]
    fn column_pages_leave_numbers_uncaptioned() {
        let p = run(&[&[
            "Diplomatics",
            "CD1-6471 \nCD1-511 \nCD87 Forgeries",
            "Warsaw (Warszawa)",
        ]]);
        assert_eq!(
            captions(&p),
            vec![
                ("CD1-6471".into(), "".into()),
                ("CD1-511".into(), "".into()),
                ("CD87".into(), "Forgeries".into())
            ]
        );
    }

    #[test]
    fn skips_tables_and_obsolete_spans_with_their_continuations() {
        let p = run(&[&[
            "Q1 Science",
            "KL-KWX4 2 General",
            "continues the table",
            "(BX670-679) Old",
            "[BP191.52] Sunnites.",
        ]]);
        assert_eq!(p.entries.len(), 1);
        assert_eq!((p.table_lines, p.obsolete), (1, 2));
        assert!(p.unparsed.is_empty());
    }
}
