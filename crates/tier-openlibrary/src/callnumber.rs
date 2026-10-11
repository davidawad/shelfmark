//! LC call numbers to scheme classes.
//!
//! Open Library gives call numbers two ways: raw, as catalogers wrote
//! them (`lc_classifications`: `PR6039.O32 H6 1988`, `HB171 .S65`), and
//! in a padded sort form (`lcc`: `PR-6039.00000000.O32 H6 1988`). Both
//! parse to a class number (letters, whole number, decimal) and the
//! first cutter; the book number, dates and anything after are dropped.
//! The class is the deepest one the scheme places for the class number
//! with that cutter, without it, or the whole number alone.

use std::collections::BTreeMap;

use shelfmark_core::id::ClassId;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};

/// The class-number part of an LC call number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallNumber {
    /// One to three capital letters, e.g. `QA`.
    pub letters: String,
    /// Whole number without leading zeros, e.g. `76`.
    pub number: String,
    /// Decimal digits without trailing zeros, e.g. `73`.
    pub decimal: Option<String>,
    /// First cutter, e.g. `P98`.
    pub cutter: Option<String>,
}

impl CallNumber {
    /// Parse either form. `None` for anything that does not open with one
    /// to three letters and a number (Dewey numbers, `[Fic]`, blanks).
    pub fn parse(raw: &str) -> Option<CallNumber> {
        let s: Vec<char> = raw.trim().chars().collect();
        let mut i = 0;
        let letters: String = s
            .iter()
            .take_while(|c| c.is_ascii_alphabetic())
            .map(char::to_ascii_uppercase)
            .collect();
        i += letters.len();
        if letters.is_empty() || letters.len() > 3 {
            return None;
        }
        // The sort form puts `-` between letters and number; raw forms
        // sometimes a space.
        while i < s.len() && (s[i] == '-' || s[i] == ' ') {
            i += 1;
        }
        let digits = |i: &mut usize| -> String {
            let start = *i;
            while *i < s.len() && s[*i].is_ascii_digit() {
                *i += 1;
            }
            s[start..*i].iter().collect()
        };
        // A call number has a class number; bare letters (`Fic`, `PZ`)
        // say too little to place.
        let whole = digits(&mut i);
        if whole.is_empty() {
            return None;
        }
        let number = match whole.trim_start_matches('0') {
            "" => "0".to_string(),
            n => n.to_string(),
        };
        let mut decimal = None;
        if i + 1 < s.len() && s[i] == '.' && s[i + 1].is_ascii_digit() {
            i += 1;
            let d = digits(&mut i);
            let d = d.trim_end_matches('0');
            if !d.is_empty() {
                decimal = Some(d.to_string());
            }
        }
        while i < s.len() && (s[i] == '.' || s[i] == ' ') {
            i += 1;
        }
        let mut cutter = None;
        if i + 1 < s.len() && s[i].is_ascii_alphabetic() && s[i + 1].is_ascii_digit() {
            let letter = s[i].to_ascii_uppercase();
            i += 1;
            cutter = Some(format!("{letter}{}", digits(&mut i)));
        }
        Some(CallNumber {
            letters,
            number,
            decimal,
            cutter,
        })
    }

    /// The class number, e.g. `QA76.73`.
    pub fn class_number(&self) -> String {
        let mut out = format!("{}{}", self.letters, self.number);
        if let Some(d) = &self.decimal {
            out.push('.');
            out.push_str(d);
        }
        out
    }

    /// Codes to try, most specific first: with the cutter, the class
    /// number, the whole number, the letters.
    pub fn codes(&self) -> Vec<String> {
        let base = self.class_number();
        let mut out = Vec::with_capacity(4);
        if let Some(c) = &self.cutter {
            out.push(format!("{base}.{c}"));
        }
        out.push(base);
        if self.decimal.is_some() {
            out.push(format!("{}{}", self.letters, self.number));
        }
        out.push(self.letters.clone());
        out
    }
}

/// Maps call numbers to classes of one index, remembering each code it
/// has checked (one book's editions repeat the same few numbers).
pub struct Mapper<'i> {
    index: &'i dyn CompiledIndex,
    seen: BTreeMap<String, Option<(ClassId, u32)>>,
}

impl<'i> Mapper<'i> {
    pub fn new(index: &'i dyn CompiledIndex) -> Self {
        Mapper {
            index,
            seen: BTreeMap::new(),
        }
    }

    fn check(&mut self, code: &str) -> Option<(ClassId, u32)> {
        if let Some(hit) = self.seen.get(code) {
            return hit.clone();
        }
        let hit = match self.index.check_code(code) {
            CodeCheck::Exact { class } | CodeCheck::Within { class } => {
                let level = self.index.get(&class.code).map_or(0, |r| r.level);
                Some((class, level))
            }
            CodeCheck::Unplaced | CodeCheck::Invalid { .. } => None,
        };
        self.seen.insert(code.to_string(), hit.clone());
        hit
    }

    /// The deepest class covering `raw`, if the scheme places it at all.
    /// A tie in depth keeps the more specific code's class.
    pub fn map(&mut self, raw: &str) -> Option<ClassId> {
        let call = CallNumber::parse(raw)?;
        let mut best: Option<(ClassId, u32)> = None;
        for code in call.codes() {
            if let Some((class, level)) = self.check(&code) {
                if best.as_ref().is_none_or(|(_, l)| level > *l) {
                    best = Some((class, level));
                }
            }
        }
        best.map(|(c, _)| c)
    }
}

/// Map one call number with a fresh [`Mapper`].
pub fn map_call_number(index: &dyn CompiledIndex, raw: &str) -> Option<ClassId> {
    Mapper::new(index).map(raw)
}
