//! Finding identifiers in a [`ClassifyInput`].
//!
//! Sources, in order: identifiers the caller supplied (normalized, invalid
//! ones dropped), then the title, extracted text and URL. Text matches
//! need enough context to be unambiguous: an ISBN-10 or LCCN needs its
//! label, an arXiv ID needs `arXiv:` or an arxiv.org URL, and a bare
//! ISBN-13 needs its 978/979 prefix and checksum. The URL domain comes only
//! from `input.url`, never from links in the text, because a document's
//! references would otherwise all count as identifiers of the document.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;
use shelfmark_core::tier::{ClassifyInput, Identifier};

use crate::normalize;

static ISBN_LABEL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bISBN(?:-?1[03])?(?:\s*\([^)\n]{0,20}\))?[\s:#]*").unwrap());
static ISBN13_BARE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b97[89](?:[- ]?[0-9]){10}\b").unwrap());
static LCCN_LABELLED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        \b(?: LCCN
            | Library \s+ of \s+ Congress \s+
              (?: Control | Catalog(?:ue)? \s+ Card ) \s+ (?: Number | No\.? )
        ) \s* :? \s*
        ( [a-z]{0,3} \s? [0-9]{2,4} - [0-9]{1,6}
        | [a-z]{0,3} \s? [0-9]{8,10} )
        \b",
    )
    .unwrap()
});
static LCCN_URL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)lccn\.loc\.gov/([a-z]{0,3}[0-9]{8,10})\b").unwrap());
static DOI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b10\.[0-9]{4,9}/[-._;()/:a-z0-9\[\]]+").unwrap());
static ARXIV: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        (?: \barxiv \s* : \s* | arxiv\.org/(?:abs|pdf)/ )
        ( [a-z][a-z.\-]*/[0-9]{7} | [0-9]{4}\.[0-9]{4,5} )
        (?: v[0-9]+ )?",
    )
    .unwrap()
});

/// Every distinct valid identifier in `input`, in a stable order.
pub fn extract(input: &ClassifyInput) -> BTreeSet<Identifier> {
    let mut out: BTreeSet<Identifier> = input
        .identifiers
        .iter()
        .filter_map(normalize::identifier)
        .collect();
    for text in [
        input.title.as_deref(),
        Some(input.text.as_str()),
        input.url.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        scan_text(text, &mut out);
    }
    if let Some(d) = input.url.as_deref().and_then(normalize::domain) {
        out.insert(Identifier::UrlDomain(d));
    }
    out
}

/// ISBN, LCCN, DOI and arXiv identifiers in free text.
pub fn scan_text(text: &str, out: &mut BTreeSet<Identifier>) {
    for m in ISBN_LABEL.find_iter(text) {
        if let Some(isbn) = isbn_after_label(&text[m.end()..]) {
            out.insert(Identifier::Isbn(isbn));
        }
    }
    for m in ISBN13_BARE.find_iter(text) {
        if let Some(isbn) = normalize::isbn(m.as_str()) {
            out.insert(Identifier::Isbn(isbn));
        }
    }
    for c in LCCN_LABELLED
        .captures_iter(text)
        .chain(LCCN_URL.captures_iter(text))
    {
        if let Some(lccn) = normalize::lccn(&c[1]) {
            out.insert(Identifier::Lccn(lccn));
        }
    }
    for m in DOI.find_iter(text) {
        if let Some(doi) = normalize::doi(m.as_str()) {
            out.insert(Identifier::Doi(doi));
        }
    }
    for c in ARXIV.captures_iter(text) {
        if let Some(id) = normalize::arxiv(&c[1]) {
            out.insert(Identifier::Arxiv(id));
        }
    }
}

/// The ISBN right after an `ISBN` label. Collects digits across single
/// hyphens or spaces, so `0-262-03384-4 (hbk.)` works and a following
/// number does not get glued on: the first 13 digits are tried as an
/// ISBN-13, then the first 10 as an ISBN-10.
fn isbn_after_label(rest: &str) -> Option<String> {
    let mut digits = String::with_capacity(13);
    let mut gap = false;
    for c in rest.chars() {
        match c {
            '0'..='9' => {
                digits.push(c);
                gap = false;
            }
            'X' | 'x' if digits.len() == 9 => {
                digits.push('X');
                break;
            }
            '-' | ' ' if !digits.is_empty() && !gap => gap = true,
            _ => break,
        }
        if digits.len() == 13 {
            break;
        }
    }
    if digits.len() == 13 && normalize::isbn13_valid(&digits) {
        return Some(digits);
    }
    digits.truncate(10);
    normalize::isbn10_valid(&digits).then(|| normalize::isbn10_to_13(&digits))
}
