//! Cutter numbers from the Library of Congress Cutter table (CSM G 63,
//! <https://www.loc.gov/aba/publications/FreeCSM/G063.pdf>).
//!
//! A Cutter is a capital letter followed by digits read as a decimal
//! fraction, so `C3` < `C36` < `C4`. LC catalogers adjust Cutters by hand
//! to fit an existing shelflist; this module applies the table only, so the
//! same key always gives the same Cutter. Rules beyond the table:
//!
//! - A letter the table does not list takes the digit of the nearest listed
//!   letter before it (or the first listed digit if none is before it).
//! - `Qa`-`Qt` give `2` followed by the expansion digit of the second
//!   letter (the table's 2-29 span); `Qv`-`Qz` give `9`.
//! - A key starting with a numeral uses the span `A12`-`A19` (G 63 sec. 3).
//! - Letters are folded to ASCII first (`é` -> `e`, `ß` -> `ss`); anything
//!   that is not a letter or digit is ignored.
//!
//! The table never yields 0 or 1, so no Cutter ends in either (G 63 note).

use std::fmt;

use serde::{Serialize, Serializer};
use unicode_normalization::UnicodeNormalization;

/// A Cutter number such as `C36`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cutter {
    letter: char,
    digits: String,
}

impl Cutter {
    /// The initial letter, uppercase ASCII.
    pub fn letter(&self) -> char {
        self.letter
    }

    /// The digits after the letter, read as a decimal fraction.
    pub fn digits(&self) -> &str {
        &self.digits
    }

    /// The form LC prints in call numbers, with the leading dot: `.C36`.
    pub fn lc(&self) -> String {
        format!(".{self}")
    }
}

/// Without the leading dot, so it is safe as the start of a file name.
impl fmt::Display for Cutter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.letter, self.digits)
    }
}

impl Serialize for Cutter {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

/// The Cutter for `key` with up to `digits` digits (the LC norm is 2).
/// `None` when the key has no letter or digit to work from.
pub fn cutter(key: &str, digits: usize) -> Option<Cutter> {
    let chars = fold(key);
    let (&first, rest) = chars.split_first()?;
    if first.is_ascii_digit() {
        return Some(numeral(&chars, digits));
    }
    let letters: Vec<char> = rest
        .iter()
        .copied()
        .filter(char::is_ascii_lowercase)
        .collect();
    let mut out = String::new();
    // `next` is the index in `letters` of the first letter left for the
    // expansion table.
    let next = match (first, letters.first()) {
        (_, None) => 0,
        ('a' | 'e' | 'i' | 'o' | 'u', Some(&c)) => {
            out.push(lookup(VOWELS, c));
            1
        }
        ('s', Some('c')) if letters.get(1) == Some(&'h') => {
            out.push('3');
            2
        }
        ('s', Some(&c)) => {
            out.push(lookup(S, c));
            1
        }
        ('q', Some('u')) => match letters.get(1) {
            Some(&c) => {
                out.push(lookup(QU, c));
                2
            }
            None => {
                out.push(expand('u'));
                1
            }
        },
        ('q', Some(&c)) if c <= 't' => {
            out.push('2');
            out.push(expand(c));
            1
        }
        ('q', Some(_)) => {
            out.push('9');
            1
        }
        (_, Some(&c)) => {
            out.push(lookup(CONSONANTS, c));
            1
        }
    };
    for &c in letters.iter().skip(next) {
        if out.len() >= digits.max(1) {
            break;
        }
        out.push(expand(c));
    }
    out.truncate(digits.max(1));
    Some(Cutter {
        letter: first.to_ascii_uppercase(),
        digits: out,
    })
}

/// After an initial vowel, by second letter.
const VOWELS: &[(char, char)] = &[
    ('b', '2'),
    ('d', '3'),
    ('l', '4'),
    ('n', '5'),
    ('p', '6'),
    ('r', '7'),
    ('s', '8'),
    ('u', '9'),
];

/// After an initial S, by second letter (`ch` is handled before lookup).
const S: &[(char, char)] = &[
    ('a', '2'),
    ('e', '4'),
    ('h', '5'),
    ('m', '6'),
    ('t', '7'),
    ('u', '8'),
    ('w', '9'),
];

/// After an initial Qu, by third letter.
const QU: &[(char, char)] = &[
    ('a', '3'),
    ('e', '4'),
    ('i', '5'),
    ('o', '6'),
    ('r', '7'),
    ('t', '8'),
    ('y', '9'),
];

/// After any other initial consonant, by second letter.
const CONSONANTS: &[(char, char)] = &[
    ('a', '3'),
    ('e', '4'),
    ('i', '5'),
    ('o', '6'),
    ('r', '7'),
    ('u', '8'),
    ('y', '9'),
];

/// The digit of the last table entry at or before `c`.
fn lookup(table: &[(char, char)], c: char) -> char {
    table
        .iter()
        .rev()
        .find(|(start, _)| *start <= c)
        .unwrap_or(&table[0])
        .1
}

/// The expansion table, for every letter after the first digit.
fn expand(c: char) -> char {
    match c {
        'a'..='d' => '3',
        'e'..='h' => '4',
        'i'..='l' => '5',
        'm'..='o' => '6',
        'p'..='s' => '7',
        't'..='v' => '8',
        _ => '9',
    }
}

/// `A1` plus one digit per leading numeral digit, spread over 2-9.
fn numeral(chars: &[char], digits: usize) -> Cutter {
    let mut out = String::from("1");
    for c in chars.iter().take_while(|c| c.is_ascii_digit()) {
        if out.len() >= digits.max(2) {
            break;
        }
        let d = c.to_digit(10).unwrap_or(0);
        let spread = 2 + d * 8 / 10;
        out.push(char::from_digit(spread, 10).unwrap_or('9'));
    }
    if out.len() == 1 {
        out.push('2');
    }
    Cutter {
        letter: 'A',
        digits: out,
    }
}

/// Lowercase ASCII letters and digits of `key`, diacritics removed.
fn fold(key: &str) -> Vec<char> {
    let mut out = Vec::new();
    for c in key.nfkd() {
        let lower: &str = match c {
            'ß' => "ss",
            'æ' | 'Æ' => "ae",
            'œ' | 'Œ' => "oe",
            'ø' | 'Ø' => "o",
            'đ' | 'Đ' | 'ð' | 'Ð' => "d",
            'ł' | 'Ł' => "l",
            'þ' | 'Þ' => "th",
            _ => {
                if c.is_ascii_alphanumeric() {
                    out.push(c.to_ascii_lowercase());
                }
                continue;
            }
        };
        out.extend(lower.chars());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letter_alone_has_no_digits() {
        assert_eq!(cutter("X", 2).unwrap().to_string(), "X");
    }

    #[test]
    fn empty_key_has_no_cutter() {
        assert_eq!(cutter(" -- ", 2), None);
    }
}
