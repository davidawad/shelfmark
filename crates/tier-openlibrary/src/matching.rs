//! Which search result is the book asked for.
//!
//! Titles and names are folded the same way: Unicode NFKD with combining
//! marks dropped, lowercased, bracketed asides removed, `&` read as
//! `and`, and anything else not alphanumeric a word break. A title also
//! loses a leading article. Title similarity is the Dice coefficient of
//! the two word sets, taken at its best over the full title, the title
//! plus subtitle, and the main title before a `:`, `;`, `/`, ` - ` or
//! `, or`, on either side, and over the work's title and the titles of
//! the editions that matched (a work filed as "Kritik der reinen
//! Vernunft" matches "Critique of Pure Reason" through its edition).
//! Author similarity is the share of the input's authors whose family
//! name is a word of some record author.
//!
//! A record's match score is `0.75 * title + 0.25 * author` when the
//! input names authors, and `0.85 * title` when it names none. The
//! search sends the first author's family name and Open Library returns
//! only works it files under that author, by any name form it knows (Homer as
//! "Ὅμηρος"), so a record whose listed names show none of the input's
//! family names still gets half the author credit. The threshold then
//! asks for a near-exact title (0.9) unless a name matches too. The best
//! record at or above the threshold wins; ties keep Open Library's
//! order.

use std::collections::BTreeSet;

use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

use crate::client::{Doc, Query};

/// Lowest match score accepted by default.
pub const DEFAULT_THRESHOLD: f64 = 0.8;

const ARTICLES: [&str; 3] = ["the", "a", "an"];

/// A record chosen for a query.
#[derive(Debug, Clone, PartialEq)]
pub struct Match<'d> {
    pub doc: &'d Doc,
    /// The match score in [0, 1].
    pub score: f64,
    pub title: f64,
    /// `None` when the query named no author.
    pub author: Option<f64>,
}

/// Fold `s` to space-separated lowercase words.
pub fn fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth = 0u32;
    for c in s.nfkd().filter(|c| !is_combining_mark(*c)) {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if depth > 0 => {}
            '&' => out.push_str(" and "),
            c if c.is_alphanumeric() => out.extend(c.to_lowercase()),
            // Apostrophes join: "Ender's" is one word.
            '\'' | '\u{2019}' => {}
            _ => out.push(' '),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn title_words(s: &str) -> BTreeSet<String> {
    let folded = fold(s);
    let mut words: Vec<&str> = folded.split(' ').filter(|w| !w.is_empty()).collect();
    if words.len() > 1 && ARTICLES.contains(&words[0]) {
        words.remove(0);
    }
    words.into_iter().map(str::to_string).collect()
}

/// The ways a title may be written: as given, and its main title alone.
fn variants(title: &str, subtitle: Option<&str>) -> Vec<BTreeSet<String>> {
    let mut out = vec![title_words(title)];
    if let Some(sub) = subtitle.filter(|s| !s.trim().is_empty()) {
        out.push(title_words(&format!("{title}: {sub}")));
    }
    let lower = title.to_lowercase();
    let cut = [":", ";", " / ", " - ", ", or ", " or, "]
        .iter()
        .filter_map(|sep| lower.find(sep))
        .min();
    if let Some(i) = cut.filter(|&i| i > 0) {
        // `find` on the lowercased copy: map the byte offset back only
        // when lowercasing kept the length (ASCII and most Latin text).
        if lower.len() == title.len() && title.is_char_boundary(i) {
            out.push(title_words(&title[..i]));
        }
    }
    out.retain(|v| !v.is_empty());
    out
}

fn dice(a: &BTreeSet<String>, b: &BTreeSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let common = a.intersection(b).count() as f64;
    2.0 * common / (a.len() + b.len()) as f64
}

/// Title similarity in [0, 1].
pub fn title_similarity(query: &str, doc_title: &str, doc_subtitle: Option<&str>) -> f64 {
    let q = variants(query, None);
    let d = variants(doc_title, doc_subtitle);
    q.iter()
        .flat_map(|a| d.iter().map(move |b| dice(a, b)))
        .fold(0.0, f64::max)
}

/// Title similarity against a record: its own title and subtitle, or the
/// title of an edition that matched.
fn doc_title_similarity(query: &str, doc: &Doc) -> f64 {
    doc.edition_titles()
        .map(|t| title_similarity(query, t, None))
        .fold(
            title_similarity(query, &doc.title, doc.subtitle.as_deref()),
            f64::max,
        )
}

/// The family name of an author as written: the part before a comma
/// ("Tolkien, J. R. R."), else the last word ("J. R. R. Tolkien").
pub fn family_name(name: &str) -> Option<String> {
    let head = match name.split_once(',') {
        Some((family, _)) => fold(family),
        None => fold(name),
    };
    let words: Vec<&str> = head.split(' ').filter(|w| w.len() > 1).collect();
    if name.contains(',') {
        words.first().map(|w| (*w).to_string())
    } else {
        words.last().map(|w| (*w).to_string())
    }
}

/// Share of `query` authors found among `doc` authors, in [0, 1].
pub fn author_similarity(query: &[String], doc: &[String]) -> f64 {
    let wanted: Vec<String> = query.iter().filter_map(|a| family_name(a)).collect();
    if wanted.is_empty() {
        return 0.0;
    }
    let have: BTreeSet<String> = doc
        .iter()
        .flat_map(|a| fold(a).split(' ').map(str::to_string).collect::<Vec<_>>())
        .collect();
    let found = wanted.iter().filter(|w| have.contains(*w)).count();
    found as f64 / wanted.len() as f64
}

/// Score one record against the query.
pub fn score<'d>(query: &Query, doc: &'d Doc) -> Match<'d> {
    let title = doc_title_similarity(&query.title, doc);
    let named = query.authors.iter().any(|a| family_name(a).is_some());
    let (score, author) = if named {
        let mut a = author_similarity(&query.authors, &doc.author_name);
        if a == 0.0 {
            // Filed under the first author by Open Library, under a name
            // form we cannot read.
            a = 0.5;
        }
        (0.75 * title + 0.25 * a, Some(a))
    } else {
        (0.85 * title, None)
    };
    Match {
        doc,
        score,
        title,
        author,
    }
}

/// Every record at or above `threshold`, best first; ties keep Open
/// Library's order.
pub fn ranked<'d>(query: &Query, docs: &'d [Doc], threshold: f64) -> Vec<Match<'d>> {
    let mut out: Vec<Match<'d>> = docs
        .iter()
        .map(|d| score(query, d))
        .filter(|m| m.score + 1e-12 >= threshold)
        .collect();
    out.sort_by(|a, b| b.score.total_cmp(&a.score));
    out
}

/// The best record at or above `threshold` that `usable` accepts (the
/// tier asks for one with a call number the scheme places).
pub fn best_match<'d>(
    query: &Query,
    docs: &'d [Doc],
    threshold: f64,
    usable: impl Fn(&Doc) -> bool,
) -> Option<Match<'d>> {
    ranked(query, docs, threshold)
        .into_iter()
        .find(|m| usable(m.doc))
}
