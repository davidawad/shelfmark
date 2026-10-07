//! Validation and canonical forms for each identifier type.
//!
//! Every function takes a raw string as a person or a catalog would write
//! it and returns the canonical value `Identifier` carries, or `None` when
//! the string is not a valid identifier of that type. Tables and extracted
//! identifiers go through the same functions, so a table row and a text
//! match for the same identifier always meet on the same key.

use std::sync::LazyLock;

use regex::Regex;
use shelfmark_core::tier::Identifier;

/// ISBN-10 or ISBN-13, with or without hyphens, spaces or an `ISBN`
/// prefix. Returns the 13 digits; an ISBN-10 becomes its 978 form.
pub fn isbn(raw: &str) -> Option<String> {
    let mut body = strip_prefix_ci(raw.trim(), "isbn").unwrap_or(raw.trim());
    for label in ["-10", "-13"] {
        if let Some(rest) = body.strip_prefix(label) {
            if rest.starts_with([':', ' ']) {
                body = rest;
            }
        }
    }
    let chars: String = body
        .chars()
        .filter(|c| !matches!(c, '-' | ' ' | ':'))
        .map(|c| c.to_ascii_uppercase())
        .collect();
    match chars.len() {
        10 if isbn10_valid(&chars) => Some(isbn10_to_13(&chars)),
        13 if isbn13_valid(&chars) => Some(chars),
        _ => None,
    }
}

/// Ten characters: nine digits and a check digit or `X`, weighted sum
/// 10..1 divisible by 11.
pub fn isbn10_valid(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || !b[..9].iter().all(u8::is_ascii_digit) {
        return false;
    }
    let check = match b[9] {
        b'X' | b'x' => 10,
        d if d.is_ascii_digit() => u32::from(d - b'0'),
        _ => return false,
    };
    let sum: u32 = b[..9]
        .iter()
        .enumerate()
        .map(|(i, d)| (10 - i as u32) * u32::from(d - b'0'))
        .sum();
    (sum + check) % 11 == 0
}

/// Thirteen digits starting 978 or 979, weights 1,3 alternating, sum
/// divisible by 10.
pub fn isbn13_valid(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 13 || !b.iter().all(u8::is_ascii_digit) {
        return false;
    }
    if !(s.starts_with("978") || s.starts_with("979")) {
        return false;
    }
    let sum: u32 = b
        .iter()
        .enumerate()
        .map(|(i, d)| u32::from(d - b'0') * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    sum % 10 == 0
}

/// `978` + the first nine digits + a fresh ISBN-13 check digit. The caller
/// has already validated `isbn10`.
pub fn isbn10_to_13(isbn10: &str) -> String {
    let mut out = format!("978{}", &isbn10[..9]);
    let sum: u32 = out
        .bytes()
        .enumerate()
        .map(|(i, d)| u32::from(d - b'0') * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    out.push(char::from(b'0' + ((10 - sum % 10) % 10) as u8));
    out
}

static LCCN_SHAPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:[a-z]{0,3}[0-9]{8}|[a-z]{0,2}[0-9]{10})$").unwrap());

/// Library of Congress Control Number, normalized per the LoC rules
/// (<https://www.loc.gov/marc/lccn-namespace.html>): drop blanks; drop a
/// `/` and everything after it; drop a hyphen and left-pad the digits
/// after it to six. Prefix letters are lowercased. The result must have
/// the pre-2001 (prefix + 8 digits) or 2001+ (prefix + 10 digits) shape.
pub fn lccn(raw: &str) -> Option<String> {
    let raw = strip_prefix_ci(raw.trim(), "lccn").unwrap_or(raw);
    let compact: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    let compact = compact.trim_start_matches(':');
    let mut s = compact.split('/').next().unwrap_or("").to_ascii_lowercase();
    if let Some((left, right)) = s.split_once('-') {
        if right.is_empty() || right.len() > 6 || !right.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        s = format!("{left}{right:0>6}");
    }
    LCCN_SHAPE.is_match(&s).then_some(s)
}

static DOI_SHAPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^10\.[0-9]{4,9}/[^\s]+$").unwrap());

/// DOI, lowercased, without a `doi:` or `https://doi.org/` prefix and
/// without trailing sentence punctuation or an unbalanced closing bracket.
pub fn doi(raw: &str) -> Option<String> {
    let mut s = raw.trim().to_ascii_lowercase();
    for prefix in [
        "https://doi.org/",
        "http://doi.org/",
        "https://dx.doi.org/",
        "http://dx.doi.org/",
        "doi.org/",
        "doi:",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.trim_start().to_string();
            break;
        }
    }
    loop {
        let before = s.len();
        s = s.trim_end_matches(['.', ',', ';', ':']).to_string();
        for (open, close) in [('(', ')'), ('[', ']')] {
            if s.ends_with(close) && s.matches(close).count() > s.matches(open).count() {
                s.pop();
            }
        }
        if s.len() == before {
            break;
        }
    }
    DOI_SHAPE.is_match(&s).then_some(s)
}

static ARXIV_NEW: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([0-9]{2})([0-9]{2})\.([0-9]{4,5})$").unwrap());
static ARXIV_OLD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([a-z]+(?:-[a-z]+)?)(?:\.[a-z-]+)?/([0-9]{2})([0-9]{2})([0-9]{3})$").unwrap()
});
static ARXIV_VERSION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"v[0-9]+$").unwrap());

/// arXiv ID without version: new style `YYMM.NNNN` (2007-04 to 2014-12)
/// or `YYMM.NNNNN` (2015 on); old style `archive/YYMMNNN` (1991-08 to
/// 2007-03), dropping any subject class (`math.GT/0309136` becomes
/// `math/0309136`).
pub fn arxiv(raw: &str) -> Option<String> {
    let mut s = raw.trim().to_ascii_lowercase();
    for prefix in [
        "https://arxiv.org/abs/",
        "http://arxiv.org/abs/",
        "https://arxiv.org/pdf/",
        "http://arxiv.org/pdf/",
        "arxiv.org/abs/",
        "arxiv.org/pdf/",
        "arxiv:",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.trim_start().to_string();
            break;
        }
    }
    if let Some(rest) = s.strip_suffix(".pdf") {
        s = rest.to_string();
    }
    let s = ARXIV_VERSION.replace(&s, "").into_owned();
    if let Some(c) = ARXIV_NEW.captures(&s) {
        let yymm: u32 = format!("{}{}", &c[1], &c[2]).parse().ok()?;
        let month: u32 = c[2].parse().ok()?;
        let serial = c[3].len();
        let ok = (1..=12).contains(&month)
            && yymm >= 704
            && ((serial == 4 && yymm <= 1412) || (serial == 5 && yymm >= 1501));
        return ok.then_some(s);
    }
    let c = ARXIV_OLD.captures(&s)?;
    let year: u32 = c[2].parse().ok()?;
    let month: u32 = c[3].parse().ok()?;
    // 91..99 are the 1990s, 00..07 the 2000s; old style ended 2007-03.
    let yyyymm = if year >= 91 { 1900 + year } else { 2000 + year } * 100 + month;
    let ok = (1..=12).contains(&month) && (199108..=200703).contains(&yyyymm);
    ok.then(|| format!("{}/{}{}{}", &c[1], &c[2], &c[3], &c[4]))
}

/// Host of a URL (or a bare host), lowercased, without port, user info,
/// trailing dot or a leading `www.`. Needs at least two labels; IP
/// addresses are rejected.
pub fn domain(raw: &str) -> Option<String> {
    let s = raw.trim().to_lowercase();
    let s = match s.find("://") {
        Some(i) => &s[i + 3..],
        None => &s,
    };
    let host = s.split(['/', '?', '#']).next()?;
    let host = host.rsplit('@').next()?;
    let host = host.split(':').next()?.trim_end_matches('.');
    let host = host.strip_prefix("www.").unwrap_or(host);
    let labels: Vec<&str> = host.split('.').collect();
    let label_ok = |l: &&str| {
        !l.is_empty()
            && !l.starts_with('-')
            && !l.ends_with('-')
            && l.chars().all(|c| c.is_alphanumeric() || c == '-')
    };
    let numeric = labels.iter().all(|l| l.bytes().all(|b| b.is_ascii_digit()));
    (labels.len() >= 2 && labels.iter().all(label_ok) && !numeric).then(|| host.to_string())
}

/// The domain and each parent with at least two labels, most specific
/// first: `cs.example.ac.uk` gives itself, `example.ac.uk`, `ac.uk`.
pub fn domain_suffixes(host: &str) -> Vec<String> {
    let labels: Vec<&str> = host.split('.').collect();
    (0..labels.len().saturating_sub(1))
        .map(|i| labels[i..].join("."))
        .collect()
}

/// Canonical form of an already-typed identifier, or `None` if invalid.
pub fn identifier(id: &Identifier) -> Option<Identifier> {
    Some(match id {
        Identifier::Isbn(v) => Identifier::Isbn(isbn(v)?),
        Identifier::Lccn(v) => Identifier::Lccn(lccn(v)?),
        Identifier::Doi(v) => Identifier::Doi(doi(v)?),
        Identifier::Arxiv(v) => Identifier::Arxiv(arxiv(v)?),
        Identifier::UrlDomain(v) => Identifier::UrlDomain(domain(v)?),
    })
}

/// Why a table key was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyError {
    /// No `type:` prefix, or one this tier does not know.
    UnknownType(String),
    /// The type is known but the value fails validation.
    InvalidValue(String),
}

/// Parse a table key (`isbn:9780262033848`, plan PD5) into its canonical
/// identifier. Values are normalized, so `isbn:0-262-03384-4` and
/// `isbn:9780262033848` are the same key.
pub fn parse_key(key: &str) -> Result<Identifier, KeyError> {
    let (kind, value) = key
        .split_once(':')
        .ok_or_else(|| KeyError::UnknownType(key.to_string()))?;
    let id = match kind {
        "isbn" => isbn(value).map(Identifier::Isbn),
        "lccn" => lccn(value).map(Identifier::Lccn),
        "doi" => doi(value).map(Identifier::Doi),
        "arxiv" => arxiv(value).map(Identifier::Arxiv),
        "domain" => domain(value).map(Identifier::UrlDomain),
        other => return Err(KeyError::UnknownType(other.to_string())),
    };
    id.ok_or_else(|| KeyError::InvalidValue(key.to_string()))
}

fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let head = s.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| s[prefix.len()..].trim_start())
}
