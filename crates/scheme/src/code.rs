//! Shelf order and coverage for class codes.
//!
//! A code splits into letter runs and digit runs; `.` and other
//! punctuation only separate. The first digit run is a whole number
//! (`QA76` is 76) and every later one a decimal fraction, so `QA76.76`
//! sorts before `QA76.9` and the cutter `.C65` before `.C7`, as on an LCC
//! shelf. A cutter letter sorts before a fraction at the same position
//! (`QA76.C65` is shelved under QA76, before QA76.1).

use std::cmp::Ordering;

use shelfmark_core::scheme::CodeRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Tok {
    Alpha(String),
    /// Whole number, leading zeros stripped.
    Int(String),
    /// Decimal fraction digits, trailing zeros stripped.
    Frac(String),
}

impl Tok {
    fn rank(&self) -> u8 {
        match self {
            Tok::Alpha(_) => 0,
            Tok::Int(_) | Tok::Frac(_) => 1,
        }
    }
}

impl Ord for Tok {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Tok::Alpha(a), Tok::Alpha(b)) | (Tok::Frac(a), Tok::Frac(b)) => a.cmp(b),
            (Tok::Int(a), Tok::Int(b)) => a.len().cmp(&b.len()).then_with(|| a.cmp(b)),
            // A whole number reads as n.0, a fraction as 0.f.
            (Tok::Int(_), Tok::Frac(_)) => Ordering::Greater,
            (Tok::Frac(_), Tok::Int(_)) => Ordering::Less,
            _ => self.rank().cmp(&other.rank()),
        }
    }
}

impl PartialOrd for Tok {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A code's shelf-order key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Key(Vec<Tok>);

impl Key {
    pub(crate) fn parse(code: &str) -> Key {
        let mut toks = Vec::new();
        let mut seen_int = false;
        let mut chars = code.chars().peekable();
        while let Some(&c) = chars.peek() {
            if c.is_alphabetic() {
                let mut s = String::new();
                while let Some(&c) = chars.peek().filter(|c| c.is_alphabetic()) {
                    s.push(c);
                    chars.next();
                }
                toks.push(Tok::Alpha(s));
            } else if c.is_ascii_digit() {
                let mut s = String::new();
                while let Some(&c) = chars.peek().filter(|c| c.is_ascii_digit()) {
                    s.push(c);
                    chars.next();
                }
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

    /// True when `self` is `other` or a deeper code under it: `other`'s
    /// tokens are a prefix of ours, and its last letter run may be a
    /// prefix of ours (`Q` covers `QA76`, `QA76` covers `QA76.76`, but
    /// `QA7` does not cover `QA76`).
    pub(crate) fn under(&self, other: &Key) -> bool {
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

/// A parsed range: inclusive of `start`, `end` and everything under `end`.
#[derive(Debug, Clone)]
pub(crate) struct Span {
    pub start: Key,
    pub end: Key,
}

impl Span {
    pub(crate) fn of(range: &CodeRange) -> Span {
        Span {
            start: Key::parse(&range.start),
            end: Key::parse(&range.end),
        }
    }

    pub(crate) fn well_ordered(&self) -> bool {
        self.start <= self.end
    }

    pub(crate) fn contains(&self, key: &Key) -> bool {
        &self.start <= key && (key <= &self.end || key.under(&self.end))
    }

    pub(crate) fn within(&self, outer: &Span) -> bool {
        outer.contains(&self.start) && outer.contains(&self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(s: &str) -> Key {
        Key::parse(s)
    }

    #[test]
    fn shelf_order() {
        let sorted = [
            "H",
            "K",
            "Q",
            "QA",
            "QA7",
            "QA75.5",
            "QA76",
            "QA76.C65",
            "QA76.1",
            "QA76.76",
            "QA76.76.C65",
            "QA76.76.C7",
            "QA76.9",
            "QA76.95",
            "QA77",
            "QA100",
            "QB",
        ];
        for pair in sorted.windows(2) {
            assert!(k(pair[0]) < k(pair[1]), "{} < {}", pair[0], pair[1]);
        }
        assert_eq!(k("QA76.50"), k("QA76.5"));
        assert_eq!(k("QA076"), k("QA76"));
    }

    #[test]
    fn coverage() {
        assert!(k("QA76.76").under(&k("QA76")));
        assert!(k("QA76").under(&k("Q")));
        assert!(k("QA76").under(&k("QA76")));
        assert!(!k("QA76").under(&k("QA7")));
        assert!(!k("QA76.76").under(&k("QA76.7")));
        assert!(!k("Q").under(&k("QA")));
    }

    #[test]
    fn ranges() {
        let r = |a: &str, b: &str| Span {
            start: k(a),
            end: k(b),
        };
        let lcc = r("QA75.5", "QA76.95");
        for inside in [
            "QA75.5",
            "QA76",
            "QA76.76",
            "QA76.76.C65",
            "QA76.95",
            "QA76.95.C3",
        ] {
            assert!(lcc.contains(&k(inside)), "{inside}");
        }
        for outside in ["QA75", "QA75.4", "QA76.96", "QA77", "QB76"] {
            assert!(!lcc.contains(&k(outside)), "{outside}");
        }
        assert!(r("QA76", "QA76.95").within(&lcc));
        assert!(!r("QA75", "QA76").within(&lcc));
        assert!(!r("QA77", "QA76").well_ordered());
    }
}
