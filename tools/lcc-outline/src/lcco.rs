//! Main classes and subclass captions from the LC Classification Outline
//! PDFs (`lcco_*.pdf`), whose first pages read `CLASS Q - SCIENCE` and then
//! `Subclass QA Mathematics`, one per subclass. A `Subclasses` entry names
//! a group (`KD-KDK`) rather than one subclass and is skipped. Only the
//! list before the first numbered line counts.

use std::sync::OnceLock;

use regex::Regex;

use crate::outline::{clean, collapse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    pub code: String,
    pub label: String,
}

/// Classes and subclasses named in one outline PDF.
#[derive(Debug, Default)]
pub struct Listing {
    pub classes: Vec<Named>,
    pub subclasses: Vec<Named>,
}

fn class_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"CLASS\s+([A-Z])(?:-([A-Z]))?\s*-\s*").expect("static regex"))
}

fn split_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"Subclass(es)?\b").expect("static regex"))
}

fn item_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\s+([A-Z]{1,3})\s+(\S[\s\S]*)$").expect("static regex"))
}

fn numbered_line() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?m)^\s*\(?[A-Z]{1,3}\(?[0-9]").expect("static regex"))
}

/// Cut a caption at the first marker that ends the subclass list.
fn cut(text: &str) -> &str {
    let mut end = text.len();
    for marker in ["(Click", "(Continued", "For military", "For naval", "\n\n"] {
        if let Some(i) = text.find(marker) {
            end = end.min(i);
        }
    }
    if let Some(m) = numbered_line().find(text) {
        end = end.min(m.start());
    }
    &text[..end]
}

/// Parse an outline PDF's pages (paragraph lists).
pub fn parse(pages: &[Vec<String>]) -> Listing {
    let text = clean(
        &pages
            .iter()
            .map(|p| p.join("\n"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    // Only the list before the outline proper: later pages repeat
    // `Subclass KQ` as a page heading over a column layout.
    let text = match numbered_line().find(&text) {
        Some(m) => &text[..m.start()],
        None => &text[..],
    };
    let mut out = Listing::default();
    if let Some(c) = class_re().captures(text) {
        let after = &text[c.get(0).expect("match").end()..];
        let label = sentence_case(&collapse(cut(after)));
        for letter in [c.get(1), c.get(2)].into_iter().flatten() {
            out.classes.push(Named {
                code: letter.as_str().to_string(),
                label: label.clone(),
            });
        }
    }
    let mut pieces = split_re().split(text);
    pieces.next();
    let mut kinds = split_re().captures_iter(text);
    for piece in pieces {
        let group = kinds.next().is_some_and(|k| k.get(1).is_some());
        if group {
            continue;
        }
        let Some(c) = item_re().captures(piece) else {
            continue;
        };
        let code = c[1].to_string();
        let label = collapse(cut(&c[2]));
        if !label.is_empty() && !out.subclasses.iter().any(|s| s.code == code) {
            out.subclasses.push(Named { code, label });
        }
    }
    out
}

/// `WORLD HISTORY AND HISTORY OF EUROPE, ASIA` to `World history and
/// history of Europe, Asia`: lowercase, capitalize each sentence and
/// parenthesis, and restore the place names the class captions use.
pub fn sentence_case(text: &str) -> String {
    const PROPER: [&str; 7] = [
        "Europe",
        "Asia",
        "Africa",
        "Australia",
        "New",
        "Zealand",
        "Americas",
    ];
    let lower = text.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut cap = true;
    for c in lower.chars() {
        if cap && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            cap = false;
        } else {
            out.push(c);
        }
        if matches!(c, '.' | '(') {
            cap = true;
        }
    }
    out.split(' ')
        .map(|w| {
            let bare = w.trim_end_matches([',', '.', ')']);
            match PROPER.iter().find(|p| p.eq_ignore_ascii_case(bare)) {
                Some(p) => format!("{p}{}", &w[bare.len()..]),
                None => w.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentence_case_keeps_places_and_sentences() {
        assert_eq!(
            sentence_case("WORLD HISTORY AND HISTORY OF EUROPE, ASIA, NEW ZEALAND, ETC."),
            "World history and history of Europe, Asia, New Zealand, etc."
        );
        assert_eq!(
            sentence_case("BIBLIOGRAPHY. LIBRARY SCIENCE. INFORMATION RESOURCES (GENERAL)"),
            "Bibliography. Library science. Information resources (General)"
        );
    }

    #[test]
    fn parses_classes_and_subclasses() {
        let pages = vec![vec![
            "CLASS K - LAW\u{2} ".to_string(),
            "(Click each subclass for details) ".to_string(),
            "Subclass K ".to_string(),
            "Law in general. Comparative and uniform law. ".to_string(),
            "Jurisprudence ".to_string(),
            "Subclasses ".to_string(),
            "United Kingdom and Ireland".to_string(),
            " KD-KDK ".to_string(),
            "Subclass KDZ ".to_string(),
            "America. North America ".to_string(),
            "K1-7720 Law in general".to_string(),
            "Subclass KV \nAsia and Eurasia \nPacific Area".to_string(),
        ]];
        let l = parse(&pages);
        assert_eq!(
            l.classes,
            vec![Named {
                code: "K".into(),
                label: "Law".into()
            }]
        );
        let subs: Vec<_> = l
            .subclasses
            .iter()
            .map(|s| (s.code.as_str(), s.label.as_str()))
            .collect();
        assert_eq!(
            subs,
            vec![
                (
                    "K",
                    "Law in general. Comparative and uniform law. Jurisprudence"
                ),
                ("KDZ", "America. North America"),
            ]
        );
    }
}
