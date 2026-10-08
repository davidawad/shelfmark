//! LCC class-number grammar and the hierarchy the eval scores against.
//!
//! A class is either a code (`QA76.73.C15`) or a caption range
//! (`QB630-QB638.8`). Hierarchy levels are derived from the code itself, so
//! the harness needs no compiled scheme to score:
//!
//! | level | name     | `QA76.73.C15` | range `QB630-QB638.8` |
//! |-------|----------|---------------|-----------------------|
//! | 1     | class    | `Q`           | `Q`                   |
//! | 2     | subclass | `QA`          | `QB`                  |
//! | 3     | caption  | `QA76`        | `QB630-638.8`         |
//! | 4     | number   | `QA76.73`     | -                     |
//! | 5     | cutter   | `QA76.73.C15` | -                     |
//!
//! A single bare letter (`Q`) has depth 1, bare subclass letters (`QA`)
//! depth 2, a range depth 3.

use std::fmt;

/// Deepest level the harness scores.
pub const MAX_LEVEL: usize = 5;

pub const LEVEL_NAMES: [&str; MAX_LEVEL] = ["class", "subclass", "caption", "number", "cutter"];

/// LCC main classes (I, O, W, X and Y are unused).
const MAIN_CLASSES: &str = "ABCDEFGHJKLMNPQRSTUVZ";

#[derive(Debug, Clone, PartialEq)]
pub struct Code {
    pub letters: String,
    /// Class number as written, trailing fractional zeros removed.
    pub number: Option<String>,
    /// First topical cutter attached to the number, without the dot.
    pub cutter: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Class {
    Code(Code),
    Range {
        letters: String,
        lo: String,
        hi: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn num_value(n: &str) -> f64 {
    n.parse().unwrap_or(f64::NAN)
}

fn trim_fraction(n: &str) -> String {
    match n.split_once('.') {
        Some((i, f)) => {
            let f = f.trim_end_matches('0');
            if f.is_empty() {
                i.to_string()
            } else {
                format!("{i}.{f}")
            }
        }
        None => n.to_string(),
    }
}

/// A range end without its cutter: `BX840.A1` and `840.Z` become `BX840`
/// and `840`. Schemes write cutter spans (`BH301.A-301.Z`), and the levels
/// scored here stop at the class number for a range.
fn strip_cutter(s: &str) -> &str {
    match s.rsplit_once('.') {
        Some((head, tail))
            if head.ends_with(|c: char| c.is_ascii_digit())
                && tail.starts_with(|c: char| c.is_ascii_uppercase())
                && tail[1..].chars().all(|c| c.is_ascii_digit()) =>
        {
            head
        }
        _ => s,
    }
}

/// Parse `letters [space] number [.cutter]` from the start of `s`; returns the
/// code and the unparsed remainder.
fn parse_code_prefix(s: &str) -> Result<(Code, &str), ParseError> {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() && b[i].is_ascii_uppercase() {
        i += 1;
    }
    if i == 0 || i > 3 {
        return Err(ParseError(format!("`{s}`: expected 1-3 class letters")));
    }
    if !MAIN_CLASSES.contains(b[0] as char) {
        return Err(ParseError(format!(
            "`{s}`: `{}` is not an LCC main class",
            b[0] as char
        )));
    }
    let letters = s[..i].to_string();
    let mut j = i;
    while j < b.len() && b[j] == b' ' {
        j += 1;
    }
    let ds = j;
    while j < b.len() && b[j].is_ascii_digit() {
        j += 1;
    }
    if j == ds {
        return Ok((
            Code {
                letters,
                number: None,
                cutter: None,
            },
            &s[i..],
        ));
    }
    if j + 1 < b.len() && b[j] == b'.' && b[j + 1].is_ascii_digit() {
        j += 1;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
    }
    let number = Some(trim_fraction(&s[ds..j]));
    let mut cutter = None;
    if j + 1 < b.len() && b[j] == b'.' && b[j + 1].is_ascii_uppercase() {
        let cs = j + 1;
        let mut k = cs + 1;
        while k < b.len() && b[k].is_ascii_digit() {
            k += 1;
        }
        if k > cs + 1 {
            cutter = Some(s[cs..k].to_string());
            j = k;
        }
    }
    Ok((
        Code {
            letters,
            number,
            cutter,
        },
        &s[j..],
    ))
}

impl Class {
    /// Strict parse of a class code or range (whole string must be consumed).
    pub fn parse(raw: &str) -> Result<Class, ParseError> {
        let s = raw.trim().to_ascii_uppercase();
        if let Some((a, b)) = s.split_once('-') {
            let (lo, rest) = parse_code_prefix(strip_cutter(a.trim()))?;
            if !rest.trim().is_empty() || lo.number.is_none() || lo.cutter.is_some() {
                return Err(ParseError(format!(
                    "`{raw}`: range start must be letters+number"
                )));
            }
            let b = strip_cutter(b.trim());
            let hi_src = if b.starts_with(|c: char| c.is_ascii_digit()) {
                format!("{}{b}", lo.letters)
            } else {
                b.to_string()
            };
            let (hi, rest) = parse_code_prefix(&hi_src)?;
            if !rest.trim().is_empty() || hi.number.is_none() || hi.cutter.is_some() {
                return Err(ParseError(format!(
                    "`{raw}`: range end must be letters+number"
                )));
            }
            if hi.letters != lo.letters {
                // A span across subclasses (`KPG1-KPH4999.9`) sits under
                // one main class and no single subclass.
                return Ok(Class::Code(Code {
                    letters: lo.letters[..1].to_string(),
                    number: None,
                    cutter: None,
                }));
            }
            let (lo_n, hi_n) = (lo.number.unwrap(), hi.number.unwrap());
            if num_value(&lo_n) > num_value(&hi_n) {
                return Err(ParseError(format!("`{raw}`: range start exceeds end")));
            }
            if lo_n == hi_n {
                // Cutters within one number (`BX840.A1-840.Z`): that number.
                return Ok(Class::Code(Code {
                    letters: lo.letters,
                    number: Some(lo_n),
                    cutter: None,
                }));
            }
            return Ok(Class::Range {
                letters: lo.letters,
                lo: lo_n,
                hi: hi_n,
            });
        }
        let (code, rest) = parse_code_prefix(&s)?;
        if !rest.trim().is_empty() {
            return Err(ParseError(format!(
                "`{raw}`: unexpected trailing `{}`",
                rest.trim()
            )));
        }
        Ok(Class::Code(code))
    }

    /// Strip an L1 typed class ID prefix (`class:lcc@2024:QA76`) then parse.
    pub fn parse_id(raw: &str) -> Result<Class, ParseError> {
        let s = raw.trim();
        let s = match s.strip_prefix("class:") {
            Some(rest) => rest.rsplit_once(':').map(|(_, c)| c).unwrap_or(rest),
            None => s,
        };
        Class::parse(s)
    }

    pub fn letters(&self) -> &str {
        match self {
            Class::Code(c) => &c.letters,
            Class::Range { letters, .. } => letters,
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            Class::Range { .. } => 3,
            Class::Code(c) => match (&c.number, &c.cutter) {
                (None, _) => {
                    if c.letters.len() == 1 {
                        1
                    } else {
                        2
                    }
                }
                (Some(_), Some(_)) => 5,
                (Some(n), None) if n.contains('.') => 4,
                (Some(_), None) => 3,
            },
        }
    }

    /// Node label at `level` (1-based); `None` beyond this class's depth.
    pub fn node(&self, level: usize) -> Option<String> {
        if level == 0 || level > self.depth() {
            return None;
        }
        let l = self.letters();
        Some(match (level, self) {
            (1, _) => l[..1].to_string(),
            (2, _) => l.to_string(),
            (3, Class::Range { lo, hi, .. }) => format!("{l}{lo}-{hi}"),
            (3, Class::Code(c)) => {
                let n = c.number.as_deref().unwrap_or("");
                format!("{l}{}", n.split('.').next().unwrap_or(n))
            }
            (4, Class::Code(c)) => format!("{l}{}", c.number.as_deref().unwrap_or("")),
            (_, Class::Code(c)) => format!(
                "{l}{}.{}",
                c.number.as_deref().unwrap_or(""),
                c.cutter.as_deref().unwrap_or("")
            ),
            _ => return None,
        })
    }

    /// Root-to-leaf path of node labels.
    pub fn path(&self) -> Vec<String> {
        (1..=self.depth()).filter_map(|l| self.node(l)).collect()
    }

    /// Canonical string form.
    pub fn canonical(&self) -> String {
        self.node(self.depth()).unwrap_or_default()
    }

    /// Does `self` (a prediction) agree with `gold` at `level`? Both must
    /// reach that level.
    pub fn agrees_at(&self, gold: &Class, level: usize) -> bool {
        if level == 0 || level > self.depth() || level > gold.depth() {
            return false;
        }
        match level {
            1 => self.letters().as_bytes()[0] == gold.letters().as_bytes()[0],
            2 => self.letters() == gold.letters(),
            3 => self.letters() == gold.letters() && caption_agrees(self, gold),
            _ => self.node(level) == gold.node(level),
        }
    }

    /// Deepest level `L` such that the prediction agrees with gold at every
    /// level `1..=L` (0 when even the main class differs).
    pub fn matched_depth(&self, gold: &Class) -> usize {
        (1..=MAX_LEVEL)
            .take_while(|&l| self.agrees_at(gold, l))
            .count()
    }
}

fn in_range(n: &str, lo: &str, hi: &str) -> bool {
    let v = num_value(n);
    let hi_ok = v <= num_value(hi) || (!hi.contains('.') && v.floor() == num_value(hi));
    v >= num_value(lo) && hi_ok
}

fn caption_agrees(pred: &Class, gold: &Class) -> bool {
    match (pred, gold) {
        (Class::Code(p), Class::Code(g)) => {
            let int = |c: &Code| {
                c.number
                    .as_deref()
                    .map(|n| n.split('.').next().unwrap_or(n).to_string())
            };
            int(p).is_some() && int(p) == int(g)
        }
        (Class::Code(p), Class::Range { lo, hi, .. }) => {
            p.number.as_deref().is_some_and(|n| in_range(n, lo, hi))
        }
        (Class::Range { lo, hi, .. }, Class::Code(g)) => {
            g.number.as_deref().is_some_and(|n| in_range(n, lo, hi))
        }
        (Class::Range { lo: a, hi: b, .. }, Class::Range { lo: c, hi: d, .. }) => {
            in_range(a, c, d) && in_range(b, c, d) || in_range(c, a, b) && in_range(d, a, b)
        }
    }
}

/// Pull the class number out of a free-text LC call number such as
/// `QA76.6 .A255 1996` or `Sp.Col. NA9108 .J3 c.2`: the first token that
/// parses as letters+number, keeping a topical cutter only when it is
/// attached to the number with no space (`QH545.P4 C38` keeps `.P4`; the
/// book number in `QA76.6 .A255` is dropped).
pub fn class_of_call_number(call: &str) -> Option<Class> {
    let s = call.trim().to_ascii_uppercase();
    let mut rest = s.as_str();
    while !rest.is_empty() {
        if let Ok((code, _)) = parse_code_prefix(rest) {
            if code.number.is_some() {
                return Some(Class::Code(code));
            }
        }
        let skip = rest.find(' ').map(|i| i + 1).unwrap_or(rest.len());
        rest = rest[skip..].trim_start();
    }
    None
}

#[cfg(test)]
#[path = "lcc_tests.rs"]
mod tests;
