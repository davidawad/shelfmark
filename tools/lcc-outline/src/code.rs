//! LCC class numbers as the outlines print them, and their shelf order.
//!
//! An outline line starts with a number or span: `QA76`, `QA75.5-76.95`,
//! `DT469.M21-.M38`, `BH301.A-Z`, `KG3001-KGH9499`, `QA101-(145)`. Parens
//! mark numbers no longer used; a line wholly in parens, or starting with
//! a bracketed number, is a span no longer used. Lines from the K form-division tables (`KL-KWX4 2`) carry
//! table numbers, not class numbers.
//!
//! Every class in the generated scheme has one code: the start number for
//! a single number, else `<start>-<end>` with the end's letters dropped
//! when they repeat the start's (`QA75.5-76.95`, `DT469.M21-469.M38`,
//! `KG3001-KGH9499`). Ranges always spell both ends in full.
//!
//! [`Key`] mirrors the shelf order of `shelfmark-scheme` (crates/scheme
//! src/code.rs), which validates the nesting this tool infers; the
//! `generated` test runs that validator over the output.

use std::cmp::Ordering;
use std::sync::OnceLock;

use regex::Regex;

/// The code grammar written into the scheme (anchored by the validator).
pub const CODE_GRAMMAR: &str = r"[A-Z]{1,3}([0-9]+(\.[0-9]+)?(\.[A-Z][0-9]*)*(-([A-Z]{1,3})?[0-9]+(\.[0-9]+)?(\.[A-Z][0-9]*)*)?)?";

/// One end of a span, in full.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Number {
    pub letters: String,
    /// Whole and decimal part, e.g. `76.95`; empty for a letters-only code.
    pub number: String,
    /// Cutters, each `.` plus a letter and digits, e.g. `.M38`.
    pub cutters: String,
}

impl Number {
    pub fn render(&self) -> String {
        format!("{}{}{}", self.letters, self.number, self.cutters)
    }
}

/// What starts an outline line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lead {
    /// A class number or span and the caption text after it.
    Span {
        start: Number,
        end: Number,
        rest: String,
    },
    /// Only letters (`KJG Albania`): a subclass heading.
    Letters { letters: String, rest: String },
    /// A span wholly in parens: no longer used.
    Obsolete,
    /// A K form-division table line (`KDZ-KH1 950`).
    Table,
}

impl Lead {
    /// The class code for a span (see the module docs).
    pub fn code(start: &Number, end: &Number) -> String {
        if start == end {
            return start.render();
        }
        let end_text = if end.letters == start.letters {
            format!("{}{}", end.number, end.cutters)
        } else {
            end.render()
        };
        format!("{}-{end_text}", start.render())
    }
}

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("static regex"))
}

/// Code characters, for joining a number the PDF wrapped mid-token.
fn code_char(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_digit() || matches!(c, '.' | '-' | '(' | ')')
}

/// Parse the start of an outline line, or `None` when it is not a number.
pub fn lead(text: &str) -> Option<Lead> {
    let text = text.trim_start();
    // `[BP191.52] Sunnites. see BP166-166.94`: bracketed, not used.
    static BRACKET: OnceLock<Regex> = OnceLock::new();
    if re(&BRACKET, r"^\[[A-Z]{1,3}[0-9]").is_match(text) {
        return Some(Lead::Obsolete);
    }
    // The code token: code characters, through any wrap inside it.
    let chars: Vec<char> = text.chars().collect();
    let mut token = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if code_char(c) {
            token.push(c);
        } else if c == '\n' && i > 0 && code_char(chars[i - 1]) {
            // A wrap inside a number (`CC1-96\n0`) or after its letters
            // (`D\nAW1001`), not a line break before the next number.
            let letters_only = token.chars().all(|c| c.is_ascii_uppercase());
            let next = chars.get(i + 1).copied();
            let joins = next.is_some_and(|n| {
                n.is_ascii_digit()
                    || matches!(n, '.' | '-' | '(' | ')')
                    || (letters_only && n.is_ascii_uppercase())
            });
            if !joins {
                break;
            }
        } else {
            break;
        }
        i += 1;
    }
    let mut rest: String = chars[i..].iter().collect();
    // A caption glued to the code (`CJ201-1397.22Ancient`): the code ends
    // where an uppercase letter starts a lowercase word.
    if rest.starts_with(|c: char| c.is_lowercase()) {
        let cut = token
            .char_indices()
            .rev()
            .find(|&(_, c)| c.is_ascii_uppercase())
            .map(|(j, _)| j)?;
        rest = format!("{}{rest}", &token[cut..]);
        token.truncate(cut);
    }
    static TABLE: OnceLock<Regex> = OnceLock::new();
    if re(&TABLE, r"^[A-Z]{1,3}-[A-Z]").is_match(&token) {
        return Some(Lead::Table);
    }
    static LETTERS: OnceLock<Regex> = OnceLock::new();
    if re(&LETTERS, r"^[A-Z]{1,3}$").is_match(&token) {
        let ok = rest.starts_with(char::is_whitespace)
            && rest.trim_start().starts_with(char::is_uppercase);
        return ok.then_some(Lead::Letters {
            letters: token,
            rest,
        });
    }
    let (start_text, end_text) = match token.find('-') {
        Some(d) if d > 0 => (&token[..d], Some(&token[d + 1..])),
        _ => (token.as_str(), None),
    };
    let start = endpoint(start_text, None)?;
    let end = match end_text {
        Some(e) => endpoint(e, Some(&start.0))?,
        None => start.clone(),
    };
    if start.1 && end.1 {
        return Some(Lead::Obsolete);
    }
    let (start, end) = (start.0, end.0);
    if Key::of(&start) > Key::of(&end) {
        return None;
    }
    Some(Lead::Span { start, end, rest })
}

/// Parse one end of a span; `start` is the span's start when this is its
/// end. Returns the number and whether it was in parens.
fn endpoint(text: &str, start: Option<&Number>) -> Option<(Number, bool)> {
    let paren = text.contains(['(', ')']);
    let bare: String = text.chars().filter(|c| !matches!(c, '(' | ')')).collect();
    static PART: OnceLock<Regex> = OnceLock::new();
    let caps = re(
        &PART,
        r"^([A-Z]{1,3})?([0-9]+(?:\.[0-9]+)?)?((?:\.[A-Z][0-9]*)*)(\.?[A-Z][0-9]*)?(\.[0-9]+)?$",
    )
    .captures(&bare)?;
    let get = |i: usize| caps.get(i).map_or("", |m| m.as_str());
    let (mut letters, mut number) = (get(1).to_string(), get(2).to_string());
    let mut cutters = get(3).to_string();
    let tail = get(4);
    let decimal = get(5);
    match start {
        None => {
            if letters.is_empty()
                || !decimal.is_empty()
                || (number.is_empty() && !(cutters.is_empty() && tail.is_empty()))
            {
                return None;
            }
            cutters.push_str(&dotted(tail));
        }
        Some(s) => {
            // `-.34` extends the start's whole number: `KBM524.32-.34`.
            if !decimal.is_empty() {
                if !(letters.is_empty()
                    && number.is_empty()
                    && cutters.is_empty()
                    && tail.is_empty())
                {
                    return None;
                }
                let whole = s.number.split('.').next().unwrap_or_default();
                number = format!("{whole}{decimal}");
            }
            // With a cutter at the start, a bare `Z9` or `A59` at the end
            // is a cutter, not a subclass: `BX1756.A1-Z9`, `DJ411.A5-A59`.
            let is_cutter = !s.cutters.is_empty()
                && !letters.is_empty()
                && letters != s.letters
                && !bare.contains('.')
                && letters.len() == 1;
            if is_cutter {
                cutters = format!(".{letters}{number}");
                letters.clear();
                number.clear();
            } else {
                cutters.push_str(&dotted(tail));
            }
            if letters.is_empty() {
                letters = s.letters.clone();
                if number.is_empty() {
                    number = s.number.clone();
                }
            }
            if number.is_empty() && !s.number.is_empty() {
                return None;
            }
            // `D274.5-6`: the end replaces the start's decimal.
            let shorter = letters == s.letters && cutters.is_empty() && !number.contains('.');
            if let Some((whole, _)) = s.number.split_once('.').filter(|_| shorter) {
                let as_int = Key::parse(&format!("{letters}{number}"));
                if as_int < Key::parse(&format!("{}{}", s.letters, s.number)) {
                    number = format!("{whole}.{number}");
                }
            }
        }
    }
    Some((
        Number {
            letters,
            number,
            cutters,
        },
        paren,
    ))
}

fn dotted(tail: &str) -> String {
    match tail {
        "" => String::new(),
        t if t.starts_with('.') => t.to_string(),
        t => format!(".{t}"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Alpha(String),
    Int(String),
    Frac(String),
}

impl Ord for Tok {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Tok::Alpha(a), Tok::Alpha(b)) | (Tok::Frac(a), Tok::Frac(b)) => a.cmp(b),
            (Tok::Int(a), Tok::Int(b)) => a.len().cmp(&b.len()).then_with(|| a.cmp(b)),
            (Tok::Int(_), Tok::Frac(_)) => Ordering::Greater,
            (Tok::Frac(_), Tok::Int(_)) => Ordering::Less,
            (Tok::Alpha(_), _) => Ordering::Less,
            (_, Tok::Alpha(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for Tok {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Shelf-order key of a code (same order as `shelfmark-scheme`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key(Vec<Tok>);

impl Key {
    pub fn of(n: &Number) -> Key {
        Key::parse(&n.render())
    }

    pub fn parse(code: &str) -> Key {
        let mut toks = Vec::new();
        let mut seen_int = false;
        let mut chars = code.chars().peekable();
        while let Some(&c) = chars.peek() {
            let mut run = |pred: fn(&char) -> bool| {
                let mut s = String::new();
                while let Some(c) = chars.next_if(pred) {
                    s.push(c);
                }
                s
            };
            if c.is_alphabetic() {
                toks.push(Tok::Alpha(run(|c| c.is_alphabetic())));
            } else if c.is_ascii_digit() {
                let s = run(|c| c.is_ascii_digit());
                if seen_int {
                    toks.push(Tok::Frac(s.trim_end_matches('0').to_string()));
                } else {
                    seen_int = true;
                    let t = s.trim_start_matches('0');
                    toks.push(Tok::Int(if t.is_empty() { "0" } else { t }.to_string()));
                }
            } else {
                chars.next();
            }
        }
        Key(toks)
    }

    /// `self` is `other` or a deeper code under it.
    pub fn under(&self, other: &Key) -> bool {
        let (mine, theirs) = (&self.0, &other.0);
        if theirs.is_empty() || theirs.len() > mine.len() {
            return false;
        }
        let last = theirs.len() - 1;
        theirs.iter().zip(mine).enumerate().all(|(i, (t, m))| {
            t == m
                || (i == last
                    && matches!((t, m), (Tok::Alpha(a), Tok::Alpha(b)) if b.starts_with(a.as_str())))
        })
    }
}

/// What a class covers: a span, or every code under a letters-only code.
#[derive(Debug, Clone)]
pub enum Cover {
    Span(Key, Key),
    Under(Key),
}

impl Cover {
    fn has(&self, k: &Key) -> bool {
        match self {
            Cover::Span(s, e) => s <= k && (k <= e || k.under(e)),
            Cover::Under(u) => k.under(u),
        }
    }

    /// Whether `inner` lies wholly inside `self`, as the validator checks.
    pub fn holds(&self, inner: &Cover) -> bool {
        match inner {
            Cover::Span(s, e) => self.has(s) && self.has(e),
            Cover::Under(u) => self.has(u),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(text: &str) -> (String, String, String) {
        match lead(text) {
            Some(Lead::Span { start, end, rest }) => {
                (start.render(), end.render(), rest.trim().to_string())
            }
            other => panic!("{text}: {other:?}"),
        }
    }

    #[test]
    fn reads_outline_numbers() {
        let cases = [
            (
                "QA75.5-76.95 Electronic computers",
                ("QA75.5", "QA76.95", "Electronic computers"),
            ),
            ("QA101-(145) Arithmetic", ("QA101", "QA145", "Arithmetic")),
            ("DT469.M21-.M38 X", ("DT469.M21", "DT469.M38", "X")),
            (
                "BH301.A-Z Special topics",
                ("BH301.A", "BH301.Z", "Special topics"),
            ),
            (
                "BX1756.A1-Z9 Sermons",
                ("BX1756.A1", "BX1756.Z9", "Sermons"),
            ),
            ("KBM524.32-.34 Law", ("KBM524.32", "KBM524.34", "Law")),
            ("KG3001-KGH9499 Mexico", ("KG3001", "KGH9499", "Mexico")),
            ("Z116.A5-265.5 Printing", ("Z116.A5", "Z265.5", "Printing")),
            (
                "D274.5-6 Anglo-French War",
                ("D274.5", "D274.6", "Anglo-French War"),
            ),
            ("CJ201-1397.22Ancient", ("CJ201", "CJ1397.22", "Ancient")),
            (
                "D\nAW1001-1051 Central Europe",
                ("DAW1001", "DAW1051", "Central Europe"),
            ),
            ("CC1-96\n0 Archaeology", ("CC1", "CC960", "Archaeology")),
        ];
        for (text, (s, e, rest)) in cases {
            assert_eq!(
                span(text),
                (s.to_string(), e.to_string(), rest.to_string()),
                "{text}"
            );
        }
    }

    #[test]
    fn codes_drop_repeated_letters() {
        let code = |t: &str| match lead(t) {
            Some(Lead::Span { start, end, .. }) => Lead::code(&start, &end),
            other => panic!("{other:?}"),
        };
        assert_eq!(code("QA75.5-76.95"), "QA75.5-76.95");
        assert_eq!(code("QA75.5-QA76.95"), "QA75.5-76.95");
        assert_eq!(code("KG3001-KGH9499"), "KG3001-KGH9499");
        assert_eq!(code("QA76"), "QA76");
    }

    #[test]
    fn classifies_other_leads() {
        assert_eq!(lead("(BX670-679) Montenegrin"), Some(Lead::Obsolete));
        assert_eq!(lead("[BP191.52] Sunnites."), Some(Lead::Obsolete));
        assert_eq!(lead("KL-KWX4 2 General"), Some(Lead::Table));
        assert!(matches!(lead("KJG Albania"), Some(Lead::Letters { .. })));
        assert_eq!(lead("History of Africa"), None);
        assert_eq!(lead("BJ2139-2137 Etiquette"), None);
        assert!(matches!(
            lead("DK5205-5219\nDK5223-5237"),
            Some(Lead::Span { .. })
        ));
    }

    #[test]
    fn spans_hold_what_the_validator_accepts() {
        let span = |a: &str, b: &str| Cover::Span(Key::parse(a), Key::parse(b));
        assert!(span("QA75.5", "QA76.95").holds(&span("QA76.75", "QA76.765")));
        assert!(span("QA75.5", "QA76.95").holds(&Cover::Under(Key::parse("QA76.95.C3"))));
        assert!(!span("QA75.5", "QA76.95").holds(&span("QA75", "QA76")));
        assert!(Cover::Under(Key::parse("Q")).holds(&span("QA1", "QA939")));
        assert!(Cover::Under(Key::parse("KG")).holds(&Cover::Under(Key::parse("KGA"))));
    }
}
