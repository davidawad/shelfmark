//! Identifier and class-number normalization.
//!
//! Every function is pure and total: it returns `None` for input it cannot
//! normalize, and the builder counts those as invalid rather than guessing.

/// Normalize an ISBN-10 or ISBN-13 to its ISBN-13 numeric form.
///
/// Hyphens, spaces and other separators are ignored; the check digit must
/// verify. ISBN-10s are converted to the 978 prefix so both spellings of one
/// book share a key.
pub fn isbn13(raw: &str) -> Option<u64> {
    let chars: Vec<char> = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == 'X' || *c == 'x')
        .collect();
    match chars.len() {
        10 => {
            let mut sum = 0u32;
            for (i, c) in chars.iter().enumerate() {
                let v = match c {
                    'X' | 'x' if i == 9 => 10,
                    d if d.is_ascii_digit() => d.to_digit(10)?,
                    _ => return None,
                };
                sum += v * (10 - i as u32);
            }
            if !sum.is_multiple_of(11) {
                return None;
            }
            let body: String = std::iter::once("978".to_string())
                .chain(chars[..9].iter().map(|c| c.to_string()))
                .collect();
            let digits: Vec<u32> = body.chars().map(|c| c.to_digit(10).unwrap()).collect();
            let check = isbn13_check(&digits);
            Some(body.parse::<u64>().ok()? * 10 + check as u64)
        }
        13 => {
            let digits: Vec<u32> = chars
                .iter()
                .map(|c| c.to_digit(10))
                .collect::<Option<_>>()?;
            if !(digits[..3] == [9, 7, 8] || digits[..3] == [9, 7, 9]) {
                return None;
            }
            if isbn13_check(&digits[..12]) != digits[12] {
                return None;
            }
            digits
                .iter()
                .try_fold(0u64, |acc, d| Some(acc * 10 + *d as u64))
        }
        _ => None,
    }
}

fn isbn13_check(first12: &[u32]) -> u32 {
    let sum: u32 = first12
        .iter()
        .enumerate()
        .map(|(i, d)| if i % 2 == 0 { *d } else { d * 3 })
        .sum();
    (10 - sum % 10) % 10
}

/// Normalize an LCCN per the Library of Congress normalization rules
/// (https://www.loc.gov/marc/lccn-namespace.html): drop blanks, drop a
/// `/` and everything after it, and if a hyphen remains, drop it and
/// left-pad the part after it with zeros to six digits. The result must be
/// an alphabetic prefix plus 8 digits (prefix <= 3) or 10 digits (prefix <= 2).
pub fn lccn(raw: &str) -> Option<String> {
    let mut s: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    if let Some(i) = s.find('/') {
        s.truncate(i);
    }
    if let Some(i) = s.find('-') {
        let (left, right) = s.split_at(i);
        let right = &right[1..];
        if right.is_empty() || right.len() > 6 || !right.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        s = format!("{left}{right:0>6}");
    }
    let s = s.to_ascii_lowercase();
    let prefix_len = s.chars().take_while(|c| c.is_ascii_lowercase()).count();
    let digits = &s[prefix_len..];
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    match (prefix_len, digits.len()) {
        (0..=3, 8) | (0..=2, 10) => Some(s),
        _ => None,
    }
}

/// Normalize an OCLC control number: drop the `(OCoLC)`, `ocm`, `ocn` and
/// `on` prefixes and leading zeros. Zero and over-long numbers are invalid.
pub fn oclc(raw: &str) -> Option<u64> {
    let mut s = raw.trim();
    for prefix in ["(OCoLC)", "(ocolc)", "ocm", "ocn", "on"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.trim_start();
            break;
        }
    }
    if s.is_empty() || s.len() > 12 || !s.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    match s.parse::<u64>().ok()? {
        0 => None,
        n => Some(n),
    }
}

/// A normalized class number: the full number kept for display and the
/// coarser forms used to detect conflicts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassNumber {
    /// Full normalized call or class number, e.g. `GE145.A53 1997`.
    pub full: String,
    /// Class prefix, e.g. LCC `GE145` or DDC section `363`.
    pub prefix: String,
    /// Top level, e.g. LCC subclass letters `GE` or DDC main class `3`.
    pub top: String,
}

/// LCC subclasses outside class K, from the LC Classification Outline
/// (https://www.loc.gov/aba/cataloging/classification/lcco/).
const LCC_SUBCLASSES: &[&str] = &[
    "A", "AC", "AE", "AG", "AI", "AM", "AN", "AP", "AS", "AY", "AZ", //
    "B", "BC", "BD", "BF", "BH", "BJ", "BL", "BM", "BP", "BQ", "BR", "BS", "BT", "BV",
    "BX", //
    "C", "CB", "CC", "CD", "CE", "CJ", "CN", "CR", "CS", "CT", //
    "D", "DA", "DAW", "DB", "DC", "DD", "DE", "DF", "DG", "DH", "DJ", "DJK", "DK", "DL", "DP",
    "DQ", "DR", "DS", "DT", "DU", "DX", //
    "E", "F", //
    "G", "GA", "GB", "GC", "GE", "GF", "GN", "GR", "GT", "GV", //
    "H", "HA", "HB", "HC", "HD", "HE", "HF", "HG", "HJ", "HM", "HN", "HQ", "HS", "HT", "HV",
    "HX", //
    "J", "JA", "JC", "JF", "JJ", "JK", "JL", "JN", "JQ", "JS", "JV", "JX", "JZ", //
    "L", "LA", "LB", "LC", "LD", "LE", "LF", "LG", "LH", "LJ", "LT", //
    "M", "ML", "MT", //
    "N", "NA", "NB", "NC", "ND", "NE", "NK", "NX", //
    "P", "PA", "PB", "PC", "PD", "PE", "PF", "PG", "PH", "PJ", "PK", "PL", "PM", "PN", "PQ", "PR",
    "PS", "PT", "PZ", //
    "Q", "QA", "QB", "QC", "QD", "QE", "QH", "QK", "QL", "QM", "QP", "QR", //
    "R", "RA", "RB", "RC", "RD", "RE", "RF", "RG", "RJ", "RK", "RL", "RM", "RS", "RT", "RV", "RX",
    "RZ", //
    "S", "SB", "SD", "SF", "SH", "SK", //
    "T", "TA", "TC", "TD", "TE", "TF", "TG", "TH", "TJ", "TK", "TL", "TN", "TP", "TR", "TS", "TT",
    "TX", //
    "U", "UA", "UB", "UC", "UD", "UE", "UF", "UG", "UH", //
    "V", "VA", "VB", "VC", "VD", "VE", "VF", "VG", "VK", "VM", //
    "Z", "ZA",
];

/// Class K (law) has hundreds of jurisdiction subclasses (`KF`, `KFN`,
/// `KJV`, `KZD`, ...); accept `K` plus a second letter in use and any third.
fn is_lcc_subclass(letters: &str) -> bool {
    match letters.as_bytes() {
        [b'K'] => true,
        [b'K', second, rest @ ..] if rest.len() <= 1 => b"BDEFGHIJKLMNPQRSTUVWZ".contains(second),
        _ => LCC_SUBCLASSES.contains(&letters),
    }
}

/// Normalize an LC call number.
///
/// Whitespace is collapsed and a space before a Cutter's period is removed
/// (`GE145 .A53` and `GE145.A53` are the same number). The call number must
/// start with an LCC subclass (see [`LCC_SUBCLASSES`]) followed by a class
/// number, or be a bare subclass; anything else (LC's `LAW`, `MLCS` and `CPB`
/// shelf marks, local marks such as `MFM Orien China 230`, NLM `W` numbers)
/// is rejected.
pub fn lcc(raw: &str) -> Option<ClassNumber> {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut full = String::with_capacity(collapsed.len());
    let chars: Vec<char> = collapsed.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if *c == ' '
            && chars.get(i + 1) == Some(&'.')
            && chars.get(i + 2).is_some_and(|n| n.is_ascii_alphabetic())
        {
            continue;
        }
        full.push(*c);
    }
    let letters: String = full
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect();
    if !is_lcc_subclass(&letters) {
        return None;
    }
    let rest = &full[letters.len()..];
    if rest.is_empty() {
        return Some(ClassNumber {
            prefix: letters.clone(),
            top: letters.clone(),
            full,
        });
    }
    let number = leading_decimal(rest.trim_start());
    if number.is_empty() {
        return None;
    }
    let prefix = format!("{letters}{number}");
    Some(ClassNumber {
        full,
        prefix,
        top: letters,
    })
}

/// Normalize a DDC number.
///
/// Prime marks (`/`, `'`) and brackets are removed (a `/` directly after
/// the three-digit section stands in for the decimal point), and only the leading
/// `ddd[.d+]` is kept (trailing edition numbers like ` 22` are dropped).
/// Non-numeric marks such as `[E]`, `B` or `Fic` are rejected.
pub fn ddc(raw: &str) -> Option<ClassNumber> {
    let trimmed = raw.trim().trim_start_matches('[');
    // `362/0924`: the prime mark took the decimal point's place.
    let b = trimmed.as_bytes();
    let repaired;
    let trimmed = if b.len() > 4
        && b[..3].iter().all(u8::is_ascii_digit)
        && b[3] == b'/'
        && b[4].is_ascii_digit()
    {
        repaired = format!("{}.{}", &trimmed[..3], &trimmed[4..]);
        repaired.as_str()
    } else {
        trimmed
    };
    let cleaned: String = trimmed
        .chars()
        .filter(|c| !matches!(c, '/' | '\'' | '[' | ']'))
        .collect();
    let number = leading_decimal(&cleaned);
    let int_len = number.find('.').unwrap_or(number.len());
    if int_len != 3 {
        return None;
    }
    let full = number.trim_end_matches('.').to_string();
    Some(ClassNumber {
        prefix: full[..3].to_string(),
        top: full[..1].to_string(),
        full,
    })
}

/// The leading `digits[.digits]` of `s` (empty if `s` does not start with a digit).
fn leading_decimal(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut end = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
    if end == 0 {
        return "";
    }
    if bytes.get(end) == Some(&b'.') {
        let frac = bytes[end + 1..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        if frac > 0 {
            end += 1 + frac;
        }
    }
    &s[..end]
}
