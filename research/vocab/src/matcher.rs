//! Aho-Corasick label matcher prototype over the heading table.
//!
//! Text and labels are folded the same way (NFKD, marks dropped, lower case,
//! every non-alphanumeric run collapsed to one space), so a match is a plain
//! substring hit that we then require to sit on word boundaries. Among the
//! boundary hits we keep only maximal ones: a hit wholly inside a longer hit
//! is dropped (`organic chemistry` beats `chemistry`), crossing hits both stay.
//!
//! Each kept hit votes for LCC classes: the heading's own numbers, or failing
//! that the nearest broader heading's (up to three levels, at a discount).
//! Votes aggregate by subclass letters (`QA`) and by main class (`Q`).

use std::collections::{BTreeMap, HashMap};

use aho_corasick::{AhoCorasick, AhoCorasickKind, MatchKind};
use serde::Serialize;
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

use crate::lcc;
use crate::table::Heading;

/// How a pattern was derived from its heading; later variants are weaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Pref,
    Alt,
    /// `Chemistry, Organic` read as `organic chemistry`.
    Uninverted,
    /// `Compilers (Computer programs)` without its qualifier.
    Unqualified,
}

impl Origin {
    fn weight(self) -> f64 {
        match self {
            Origin::Pref => 1.0,
            Origin::Alt => 0.9,
            Origin::Uninverted => 0.8,
            Origin::Unqualified => 0.7,
        }
    }
}

const MIN_PATTERN_CHARS: usize = 3;
/// Function words that never identify a subject on their own; a pattern made
/// only of these (e.g. `and`, left after stripping a qualifier) is dropped.
const STOPWORDS: &[&str] = &[
    "a", "about", "after", "all", "also", "an", "and", "any", "are", "as", "at", "be", "been",
    "before", "but", "by", "can", "do", "for", "from", "had", "has", "have", "he", "her", "his",
    "how", "i", "if", "in", "into", "is", "it", "its", "may", "more", "no", "not", "of", "on",
    "one", "or", "other", "our", "out", "she", "so", "some", "such", "than", "that", "the",
    "their", "them", "then", "there", "these", "they", "this", "those", "to", "up", "us", "was",
    "we", "were", "what", "when", "which", "who", "will", "with", "you",
];
const INHERIT_DEPTH: usize = 3;
const INHERIT_DISCOUNT: f64 = 0.5;

pub struct Matcher {
    headings: Vec<Heading>,
    ac: AhoCorasick,
    /// Pattern index -> (normalised text, [(heading index, origin)]).
    patterns: Vec<(String, Vec<(usize, Origin)>)>,
    /// Heading index -> LCC cells it votes with and whether they were inherited.
    class_links: Vec<(Vec<String>, bool)>,
}

#[derive(Debug, Serialize)]
pub struct Hit {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub headings: Vec<HitHeading>,
}

#[derive(Debug, Serialize)]
pub struct HitHeading {
    pub id: String,
    pub label: String,
    pub origin: Origin,
    pub lcc: Vec<String>,
    pub lcc_inherited: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassScore {
    pub class: String,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct MatchResult {
    pub normalized: String,
    pub hits: Vec<Hit>,
    pub subclasses: Vec<ClassScore>,
    pub main_classes: Vec<ClassScore>,
}

pub fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = true;
    for c in text.nfkd().filter(|c| !is_combining_mark(*c)) {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            space = false;
        } else if !space {
            out.push(' ');
            space = true;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Pattern variants of one label with their origin.
/// Drop every parenthetical qualifier: `Songs (High voice) with harp` reads
/// as `Songs with harp` in running text.
pub fn strip_qualifiers(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    let mut depth = 0usize;
    for c in label.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Pattern variants of one label with their origin.
pub fn variants(label: &str, origin: Origin) -> Vec<(String, Origin)> {
    let mut out = vec![(normalize(label), origin)];
    let bare = strip_qualifiers(label);
    if bare != label {
        out.push((normalize(&bare), Origin::Unqualified.max(origin)));
    }
    if !bare.contains("--") {
        if let Some((head, tail)) = bare.split_once(", ") {
            if !tail.contains(',') {
                let flipped = format!("{tail} {head}");
                out.push((normalize(&flipped), Origin::Uninverted.max(origin)));
            }
        }
    }
    out.retain(|(p, _)| {
        p.chars().count() >= MIN_PATTERN_CHARS && !p.split(' ').all(|w| STOPWORDS.contains(&w))
    });
    out
}

impl Matcher {
    pub fn build(headings: Vec<Heading>) -> Result<Matcher, String> {
        let mut by_pattern: BTreeMap<String, Vec<(usize, Origin)>> = BTreeMap::new();
        for (i, h) in headings.iter().enumerate().filter(|(_, h)| !h.deprecated) {
            let labels = std::iter::once((&h.pref_label, Origin::Pref))
                .chain(h.alt_labels.iter().map(|a| (a, Origin::Alt)));
            for (label, origin) in labels {
                for (p, o) in variants(label, origin) {
                    by_pattern.entry(p).or_default().push((i, o));
                }
            }
        }
        let patterns: Vec<(String, Vec<(usize, Origin)>)> = by_pattern
            .into_iter()
            .map(|(p, mut hs)| {
                hs.sort();
                hs.dedup_by_key(|(i, _)| *i);
                (p, hs)
            })
            .collect();
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::Standard)
            .kind(Some(AhoCorasickKind::ContiguousNFA))
            .build(patterns.iter().map(|(p, _)| p.as_str()))
            .map_err(|e| e.to_string())?;
        let class_links = inherit_classes(&headings);
        Ok(Matcher {
            headings,
            ac,
            patterns,
            class_links,
        })
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn heading_count(&self) -> usize {
        self.headings.len()
    }

    pub fn find(&self, text: &str) -> MatchResult {
        let norm = normalize(text);
        let bytes = norm.as_bytes();
        let mut spans: Vec<(usize, usize, usize)> = self
            .ac
            .find_overlapping_iter(&norm)
            .filter(|m| {
                let starts_word = m.start() == 0 || bytes[m.start() - 1] == b' ';
                let ends_word = m.end() == bytes.len() || bytes[m.end()] == b' ';
                starts_word && ends_word
            })
            .map(|m| (m.start(), m.end(), m.pattern().as_usize()))
            .collect();
        // Longest first, then position, so containment checks see the outer hit.
        spans.sort_by(|a, b| (b.1 - b.0).cmp(&(a.1 - a.0)).then(a.0.cmp(&b.0)));
        let mut kept: Vec<(usize, usize, usize)> = Vec::new();
        for s in spans {
            if !kept.iter().any(|k| k.0 <= s.0 && s.1 <= k.1) {
                kept.push(s);
            }
        }
        kept.sort();

        let mut sub: HashMap<String, f64> = HashMap::new();
        let mut main: HashMap<String, f64> = HashMap::new();
        let mut hits = Vec::new();
        for (start, end, pat) in kept {
            let (ptext, owners) = &self.patterns[pat];
            let tokens = ptext.split(' ').count() as f64;
            let share = 1.0 / owners.len() as f64;
            let mut hh = Vec::new();
            for &(hi, origin) in owners {
                let (codes, inherited) = &self.class_links[hi];
                let mut w = tokens * origin.weight() * share;
                if *inherited {
                    w *= INHERIT_DISCOUNT;
                }
                for code in codes {
                    let Some(r) = lcc::normalize(code) else {
                        continue;
                    };
                    let w = w / codes.len() as f64;
                    *sub.entry(r.class.clone()).or_default() += w;
                    *main.entry(r.class[..1].to_string()).or_default() += w;
                }
                let h = &self.headings[hi];
                hh.push(HitHeading {
                    id: h.id.clone(),
                    label: h.pref_label.clone(),
                    origin,
                    lcc: codes.clone(),
                    lcc_inherited: *inherited,
                });
            }
            hits.push(Hit {
                start,
                end,
                text: norm[start..end].to_string(),
                headings: hh,
            });
        }
        MatchResult {
            normalized: norm,
            hits,
            subclasses: ranked(sub),
            main_classes: ranked(main),
        }
    }
}

fn ranked(scores: HashMap<String, f64>) -> Vec<ClassScore> {
    let mut v: Vec<ClassScore> = scores
        .into_iter()
        .map(|(class, score)| ClassScore { class, score })
        .collect();
    v.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.class.cmp(&b.class)));
    v
}

/// Each heading's voting LCC: its own, else the nearest broader level's.
fn inherit_classes(headings: &[Heading]) -> Vec<(Vec<String>, bool)> {
    let index: HashMap<(&str, &str), usize> = headings
        .iter()
        .enumerate()
        .map(|(i, h)| ((h.vocab.as_str(), h.id.as_str()), i))
        .collect();
    headings
        .iter()
        .map(|h| {
            if !h.lcc.is_empty() {
                return (h.lcc.clone(), false);
            }
            let mut frontier: Vec<usize> = vec![];
            let mut seen = std::collections::HashSet::new();
            let up = |h: &Heading, out: &mut Vec<usize>| {
                for b in &h.broader {
                    if let Some(&j) = index.get(&(h.vocab.as_str(), b.as_str())) {
                        out.push(j);
                    }
                }
            };
            up(h, &mut frontier);
            for _ in 0..INHERIT_DEPTH {
                frontier.retain(|j| seen.insert(*j));
                let mut codes: Vec<String> = frontier
                    .iter()
                    .flat_map(|&j| headings[j].lcc.iter().cloned())
                    .collect();
                if !codes.is_empty() {
                    codes.sort();
                    codes.dedup();
                    return (codes, true);
                }
                let mut next = Vec::new();
                for &j in &frontier {
                    up(&headings[j], &mut next);
                }
                frontier = next;
            }
            (Vec::new(), false)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heading(id: &str, label: &str, alts: &[&str], broader: &[&str], lcc: &[&str]) -> Heading {
        Heading {
            id: id.into(),
            vocab: "lcsh".into(),
            kind: "Topic".into(),
            pref_label: label.into(),
            alt_labels: alts.iter().map(|s| s.to_string()).collect(),
            broader: broader.iter().map(|s| s.to_string()).collect(),
            lcc: lcc.iter().map(|s| s.to_string()).collect(),
            ..Heading::default()
        }
    }

    fn matcher() -> Matcher {
        Matcher::build(vec![
            heading("sh1", "Chemistry, Organic", &[], &["sh2"], &["QD241-QD441"]),
            heading("sh2", "Chemistry", &[], &[], &["QD"]),
            heading("sh3", "Compilers (Computer programs)", &[], &["sh4"], &[]),
            heading(
                "sh4",
                "Programming languages (Electronic computers)",
                &[],
                &[],
                &["QA76.7"],
            ),
            heading("sh5", "Art", &["Fine arts"], &[], &["N"]),
        ])
        .unwrap()
    }

    #[test]
    fn normalizes_case_marks_and_punctuation() {
        assert_eq!(normalize("  Café--Société, (Paris)!"), "cafe societe paris");
        assert_eq!(normalize("C++ & ML"), "c ml");
    }

    #[test]
    fn derives_uninverted_and_unqualified_variants() {
        let v = variants("Chemistry, Organic", Origin::Pref);
        assert!(v.contains(&("organic chemistry".into(), Origin::Uninverted)));
        let v = variants("Compilers (Computer programs)", Origin::Alt);
        assert!(v.contains(&("compilers".into(), Origin::Unqualified)));
        let v = variants("Songs (High voice) with harp", Origin::Pref);
        assert!(v.contains(&("songs with harp".into(), Origin::Unqualified)));
        assert!(variants("AND (Computer program language)", Origin::Pref)
            .iter()
            .all(|(p, _)| p != "and"));
    }

    #[test]
    fn keeps_maximal_word_bounded_hits() {
        let m = matcher();
        let r = m.find("Organic chemistry for artists, not party art.");
        let texts: Vec<_> = r.hits.iter().map(|h| h.text.as_str()).collect();
        // "chemistry" sits inside "organic chemistry"; "art" inside "artists"
        // and "party" is not on a word boundary.
        assert_eq!(texts, ["organic chemistry", "art"]);
        assert_eq!(r.subclasses[0].class, "QD");
        assert_eq!(r.main_classes[0].class, "Q");
    }

    #[test]
    fn inherits_lcc_from_broader_at_a_discount() {
        let m = matcher();
        let r = m.find("Writing compilers");
        let h = &r.hits[0].headings[0];
        assert_eq!(h.id, "sh3");
        assert!(h.lcc_inherited);
        assert_eq!(h.lcc, ["QA76.7"]);
        let expected = 1.0 * Origin::Unqualified.weight() * INHERIT_DISCOUNT;
        assert!((r.subclasses[0].score - expected).abs() < 1e-9);
    }

    #[test]
    fn deterministic_ranking() {
        let m = matcher();
        let a = serde_json::to_string(&m.find("fine arts and chemistry")).unwrap();
        let b = serde_json::to_string(&m.find("fine arts and chemistry")).unwrap();
        assert_eq!(a, b);
    }
}
