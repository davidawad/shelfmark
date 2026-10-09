//! Locked pages to a scheme document.
//!
//! Classes keep the site's hierarchy and captions verbatim. The page
//! header of the auxiliaries page (`---`, "COMMON AUXILIARY NUMBERS") is
//! a grouping, not notation: it is dropped and its children become top
//! level classes. Main classes `0`-`9` are top level as on the site.
//!
//! No ranges: the site's span codes (`004.01/.08`) are kept as classes
//! with their own captions. No LCC or LCSH links: the UDC Summary
//! publishes no mappings (its mappings section is "under development").
//! A caption is a list of equivalent terms (`Computing. Data processing`);
//! the lexical tier matches whole labels, so each term of a multi-term
//! caption is also a vocabulary link with source [`CAPTION_SOURCE`].

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, VocabLink, SCHEME_SCHEMA_ID};

use crate::page::{self, Node};
use crate::sources::Lock;

pub const NAME: &str = "udc-summary";
pub const TITLE: &str = "UDC Summary (Universal Decimal Classification)";
/// Vocabulary source of the terms split from a caption.
pub const CAPTION_SOURCE: &str = "udc-caption";
/// Weight of a caption term: a little below the whole caption.
pub const CAPTION_WEIGHT: f64 = 0.9;
pub const URL: &str = "https://udcsummary.info/";
pub const LICENSE: &str = "Creative Commons Attribution-NonCommercial 4.0 International \
(CC BY-NC 4.0), https://creativecommons.org/licenses/by-nc/4.0/. \
Attribution: Multilingual Universal Decimal Classification Summary (UDCC Publication No. 088), \
(c) UDC Consortium, 2013, released for use and distribution under the Creative Commons \
Attribution-NonCommercial 4.0 International license; UDC Consortium, PO Box 90407, \
2509 LK The Hague, The Netherlands, mail@udcc.org. Not for commercial use; \
shelfmark does not redistribute this file.";

/// A UDC number as the summary writes it, and the compound numbers built
/// from it: a digit or the opening sign of a common auxiliary (`(`, `=`,
/// `"`, `-`), then digits, `.`, auxiliary brackets and signs, the
/// special-auxiliary apostrophe and backtick, and `A/Z` letters.
pub const CODE_GRAMMAR: &str = "[0-9(=\"-][0-9A-Za-z.()=\"'`*:+/\\[\\]-]*";

/// What a build kept and dropped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    /// Classes per page tag, in lock order.
    pub per_page: Vec<(String, usize)>,
    /// Page headers dropped as groupings.
    pub headers: Vec<String>,
}

/// Build the scheme from each locked page's text, given in lock order.
pub fn build(lock: &Lock, pages: &[String]) -> Result<(SchemeDoc, Stats), String> {
    if pages.len() != lock.sources.len() {
        return Err(format!(
            "{} pages for {} locked sources",
            pages.len(),
            lock.sources.len()
        ));
    }
    let grammar = grammar_matcher(CODE_GRAMMAR);
    let mut classes = Vec::new();
    let mut stats = Stats::default();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for (locked, text) in lock.sources.iter().zip(pages) {
        let ctx = |e: String| format!("{} (tag {}): {e}", locked.file, locked.tag);
        let nodes = page::parse(text).map_err(ctx)?;
        let before = classes.len();
        for c in page_classes(&nodes, &mut stats).map_err(ctx)? {
            if !grammar(&c.code) {
                return Err(ctx(format!("`{}` does not match the code grammar", c.code)));
            }
            if let Some(first) = seen.insert(c.code.clone(), locked.tag.clone()) {
                return Err(ctx(format!("`{}` is also on page {first}", c.code)));
            }
            classes.push(c);
        }
        stats
            .per_page
            .push((locked.tag.clone(), classes.len() - before));
    }
    let doc = SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: NAME.to_string(),
        version: lock.edition.clone(),
        title: TITLE.to_string(),
        source: Some(Source {
            description: description(lock, classes.len()),
            url: Some(URL.to_string()),
            license: Some(LICENSE.to_string()),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    };
    Ok((doc, stats))
}

/// One page's classes in page order, parents resolved to codes.
fn page_classes(nodes: &[Node], stats: &mut Stats) -> Result<Vec<ClassDef>, String> {
    let mut codes: HashMap<i64, Option<&str>> = HashMap::new();
    let mut out = Vec::new();
    for n in nodes {
        let parent = match n.parent {
            -1 => None,
            p => *codes
                .get(&p)
                .ok_or_else(|| format!("`{}`: parent {p} is not listed before it", n.code))?,
        };
        if n.parent == -1 && n.code.starts_with("--") {
            stats.headers.push(n.code.clone());
            codes.insert(n.id, None);
            continue;
        }
        if n.label.is_empty() {
            return Err(format!("`{}` has no caption", n.code));
        }
        if codes.insert(n.id, Some(&n.code)).is_some() {
            return Err(format!("node id {} is listed twice", n.id));
        }
        out.push(ClassDef {
            code: n.code.clone(),
            label: n.label.clone(),
            parent: parent.map(str::to_string),
            range: None,
            vocab: caption_terms(&n.label)
                .into_iter()
                .map(|term| VocabLink {
                    term,
                    source: CAPTION_SOURCE.to_string(),
                    source_id: None,
                    weight: CAPTION_WEIGHT,
                })
                .collect(),
            note: None,
        });
    }
    Ok(out)
}

/// The terms of a caption that lists more than one: split at `. ` outside
/// parentheses, trailing `.` dropped, terms under three characters
/// skipped, each term once. Empty for a single-term caption.
pub fn caption_terms(caption: &str) -> Vec<String> {
    let mut terms: Vec<String> = Vec::new();
    let (mut depth, mut cur) = (0i32, String::new());
    let mut chars = caption.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '.' if depth <= 0 && chars.peek().is_none_or(|n| n.is_whitespace()) => {
                terms.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    terms.push(cur);
    let mut out: Vec<String> = Vec::new();
    for t in terms.iter().map(|t| t.trim()) {
        if t.chars().count() >= 3 && !out.iter().any(|o| o == t) {
            out.push(t.to_string());
        }
    }
    if out.len() < 2 {
        out.clear();
    }
    out
}

/// [`CODE_GRAMMAR`] as a matcher, without a regex dependency:
/// the grammar is one leading class and one repeated class.
fn grammar_matcher(grammar: &str) -> impl Fn(&str) -> bool {
    debug_assert_eq!(grammar, CODE_GRAMMAR);
    |code: &str| {
        let mut chars = code.chars();
        chars
            .next()
            .is_some_and(|c| c.is_ascii_digit() || "(=\"-".contains(c))
            && chars.all(|c| c.is_ascii_alphanumeric() || ".()=\"'`*:+/[]-".contains(c))
    }
}

fn description(lock: &Lock, n: usize) -> String {
    let mut d = format!(
        "{n} classes of the UDC Summary, the UDC Consortium's selection from the UDC Master \
         Reference File: the common auxiliary numbers (Tables 1c-1k) and main classes 0-9 with \
         their captions and hierarchy, from the English tree pages of udcsummary.info. The \
         common auxiliary signs are not classes and are left out. The summary has no LCC or \
         LCSH mappings; each term of a multi-term caption is a vocabulary link with source \
         udc-caption. Built by schemes/udc-summary from {} pages locked as \
         {} (edition {}):",
        lock.sources.len(),
        lock.digest(),
        lock.edition
    );
    for s in &lock.sources {
        let _ = write!(d, " {} sha256 {};", s.url, s.sha256);
    }
    d.pop();
    d.push('.');
    d
}

/// The scheme as JSON, one class per line, so diffs stay readable.
pub fn scheme_json(doc: &SchemeDoc) -> Result<String, String> {
    let err = |e: serde_json::Error| e.to_string();
    let mut head = doc.clone();
    head.classes.clear();
    let head = serde_json::to_string_pretty(&head).map_err(err)?;
    let head = head
        .trim_end()
        .strip_suffix('}')
        .and_then(|h| h.trim_end().strip_suffix("\"classes\": []"))
        .ok_or("unexpected scheme JSON layout")?;
    let mut out = format!("{head}\"classes\": [\n");
    for (i, c) in doc.classes.iter().enumerate() {
        let sep = if i + 1 < doc.classes.len() { "," } else { "" };
        let _ = writeln!(out, "    {}{sep}", serde_json::to_string(c).map_err(err)?);
    }
    out.push_str("  ]\n}\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_caption_terms() {
        assert_eq!(
            caption_terms("Computer science and technology. Computing. Data processing"),
            [
                "Computer science and technology",
                "Computing",
                "Data processing"
            ]
        );
        assert_eq!(
            caption_terms("Persons as agents (studying, making, etc.). Doers"),
            ["Persons as agents (studying, making, etc.)", "Doers"]
        );
        assert_eq!(
            caption_terms("Articles, etc. Contributions"),
            ["Articles, etc", "Contributions"]
        );
        assert_eq!(caption_terms("Pigs. Swine"), ["Pigs", "Swine"]);
        assert!(caption_terms("Dogs").is_empty());
        assert!(caption_terms("Sex. Sex").is_empty());
        assert!(
            caption_terms("Ox. Cattle").is_empty(),
            "a short term is skipped"
        );
        assert!(caption_terms("Grand-Duché de Luxembourg").is_empty());
        assert_eq!(
            caption_terms("Grade 1.5 dose. Units"),
            ["Grade 1.5 dose", "Units"]
        );
    }
}
