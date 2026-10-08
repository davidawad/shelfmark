//! Assemble the scheme from the two Census tables: every code in the
//! 2-6 digit file becomes a class under the code one digit shorter (a
//! subsector under the sector whose span holds it), and every index entry
//! becomes a `naics-index` vocabulary term on its 6-digit industry.

use std::collections::{BTreeMap, BTreeSet};

use shelfmark_core::scheme::{ClassDef, CodeRange, SchemeDoc, Source, VocabLink, SCHEME_SCHEMA_ID};

pub const NAME: &str = "naics";
pub const VERSION: &str = "2022";
/// Two-digit sectors, one sector span (`31-33`), or 3 to 6 digits.
pub const CODE_GRAMMAR: &str = "[0-9]{2}(-[0-9]{2}|[0-9]{1,4})?";
/// Index entries for the 6-digit industry they classify.
pub const INDEX_SOURCE: &str = "naics-index";
pub const SOURCE_URL: &str = "https://www.census.gov/naics/";
pub const LICENSE: &str = "NAICS United States, 2022 (Executive Office of the President, \
Office of Management and Budget; published by the US Census Bureau): US government work, not \
subject to copyright in the United States (17 U.S.C. 105). Credit: Source: U.S. Census Bureau, \
2022 NAICS";

/// What the build kept and dropped, for SOURCES.md.
#[derive(Debug, Default)]
pub struct Stats {
    /// Classes by code length: sectors (2 digits or a span), 3, 4, 5, 6.
    pub by_level: [usize; 5],
    pub sector_spans: Vec<String>,
    pub index_rows: usize,
    pub index_terms: usize,
    /// `******` rows: "see" cross-references with no code.
    pub cross_references: usize,
    /// Rows repeating a term already on the same code.
    pub duplicates: usize,
    /// Terms that only repeat the industry's own title.
    pub echoes: usize,
    pub unknown_codes: Vec<String>,
    pub industries_without_terms: Vec<String>,
}

/// One whitespace-collapsed, trimmed string.
pub fn clean(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_code(s: &str) -> bool {
    let b = s.as_bytes();
    let digits = |r: &[u8]| r.iter().all(u8::is_ascii_digit);
    ((2..=6).contains(&b.len()) && digits(b))
        || (b.len() == 5 && b[2] == b'-' && digits(&b[..2]) && digits(&b[3..]))
}

/// Codes and titles from the 2-6 digit file: a header row naming the
/// columns `Seq. No.`, code and title, then one row per code.
pub fn codes(rows: &[Vec<String>]) -> Result<Vec<(String, String)>, String> {
    let header = rows.first().ok_or("the codes table is empty")?;
    if !header.get(1).is_some_and(|h| h.contains("NAICS")) {
        return Err(format!("unexpected codes header {header:?}"));
    }
    let mut out = Vec::new();
    for (i, row) in rows.iter().enumerate().skip(1) {
        let cell = |c: usize| row.get(c).map_or(String::new(), |s| clean(s));
        let (code, title) = (cell(1), cell(2));
        if code.is_empty() && title.is_empty() {
            continue;
        }
        if !is_code(&code) || title.is_empty() {
            return Err(format!("codes row {}: unexpected {row:?}", i + 1));
        }
        out.push((code, title));
    }
    Ok(out)
}

/// Index entries as (code, term): a header row `NAICS22`, then one row
/// per entry. `******` rows are cross-references and are counted, not kept.
pub fn index(rows: &[Vec<String>], stats: &mut Stats) -> Result<Vec<(String, String)>, String> {
    let header = rows.first().ok_or("the index table is empty")?;
    if !header.first().is_some_and(|h| h.starts_with("NAICS")) {
        return Err(format!("unexpected index header {header:?}"));
    }
    let mut out = Vec::new();
    for (i, row) in rows.iter().enumerate().skip(1) {
        let cell = |c: usize| row.get(c).map_or(String::new(), |s| clean(s));
        let (code, term) = (cell(0), cell(1));
        if code.is_empty() && term.is_empty() {
            continue;
        }
        stats.index_rows += 1;
        if code.chars().all(|c| c == '*') {
            stats.cross_references += 1;
            continue;
        }
        if code.len() != 6 || !is_code(&code) || term.is_empty() {
            return Err(format!("index row {}: unexpected {row:?}", i + 1));
        }
        out.push((code, term));
    }
    Ok(out)
}

/// The parent of `code` among `known` codes: the code one digit shorter,
/// or for a subsector the sector (or sector span) that holds its first two
/// digits.
fn parent(
    code: &str,
    known: &BTreeSet<String>,
    spans: &[(u32, u32, String)],
) -> Result<Option<String>, String> {
    if code.len() <= 2 || code.contains('-') {
        return Ok(None);
    }
    let up = &code[..code.len() - 1];
    if known.contains(up) {
        return Ok(Some(up.to_string()));
    }
    if up.len() == 2 {
        let n: u32 = up.parse().map_err(|_| format!("bad code {code}"))?;
        if let Some((_, _, span)) = spans.iter().find(|(a, b, _)| (*a..=*b).contains(&n)) {
            return Ok(Some(span.clone()));
        }
    }
    Err(format!("`{code}` has no parent `{up}` in the codes table"))
}

/// The scheme document from parsed codes and index entries.
pub fn build(
    codes: &[(String, String)],
    index: &[(String, String)],
    mut stats: Stats,
) -> Result<(SchemeDoc, Stats), String> {
    let mut known = BTreeSet::new();
    let mut spans = Vec::new();
    for (code, _) in codes {
        if !known.insert(code.clone()) {
            return Err(format!("`{code}` appears twice in the codes table"));
        }
        if let Some((a, b)) = code.split_once('-') {
            let n = |s: &str| s.parse::<u32>().map_err(|_| format!("bad span {code}"));
            spans.push((n(a)?, n(b)?, code.clone()));
            stats.sector_spans.push(code.clone());
        }
    }
    let mut terms: BTreeMap<&str, Vec<VocabLink>> = BTreeMap::new();
    let labels: BTreeMap<&str, &str> = codes
        .iter()
        .map(|(c, t)| (c.as_str(), t.as_str()))
        .collect();
    for (code, term) in index {
        let Some(label) = labels.get(code.as_str()) else {
            stats.unknown_codes.push(format!("{code} ({term})"));
            continue;
        };
        if term.eq_ignore_ascii_case(label) {
            stats.echoes += 1;
            continue;
        }
        let links = terms.entry(code).or_default();
        if links.iter().any(|l| l.term.eq_ignore_ascii_case(term)) {
            stats.duplicates += 1;
            continue;
        }
        links.push(VocabLink {
            term: term.clone(),
            source: INDEX_SOURCE.to_string(),
            source_id: None,
            weight: 1.0,
        });
        stats.index_terms += 1;
    }
    let mut classes = Vec::new();
    for (code, title) in codes {
        let level = if code.contains('-') {
            0
        } else {
            code.len() - 2
        };
        stats.by_level[level] += 1;
        let range = code.split_once('-').map(|(a, b)| CodeRange {
            start: a.to_string(),
            end: b.to_string(),
        });
        let vocab = terms.remove(code.as_str()).unwrap_or_default();
        if code.len() == 6 && vocab.is_empty() {
            stats.industries_without_terms.push(code.clone());
        }
        classes.push(ClassDef {
            code: code.clone(),
            label: title.clone(),
            parent: parent(code, &known, &spans)?,
            range,
            vocab,
            note: None,
        });
    }
    let doc = SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: NAME.to_string(),
        version: VERSION.to_string(),
        title: "North American Industry Classification System (NAICS), United States, 2022"
            .to_string(),
        source: Some(Source {
            description:
                "Every 2- to 6-digit code and US title from the Census Bureau's 2022 NAICS \
                2-6 digit codes file; each code sits under the code one digit shorter, and each \
                3-digit subsector under its sector (31-33, 44-45 and 48-49 are sector spans). \
                Vocabulary: the 2022 NAICS index entries, as `naics-index` terms on their 6-digit \
                industry. NAICS publishes no LCC or LCSH links. Built by \
                src/shelfmark/schemes/naics/builder; see schemes/naics/SOURCES.md."
                    .to_string(),
            url: Some(SOURCE_URL.to_string()),
            license: Some(LICENSE.to_string()),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    };
    Ok((doc, stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn recognises_codes() {
        for ok in ["11", "111", "111110", "31-33"] {
            assert!(is_code(ok), "{ok}");
        }
        for bad in ["1", "1111100", "31-3", "abc", "", "******"] {
            assert!(!is_code(bad), "{bad}");
        }
    }

    #[test]
    fn nests_subsectors_under_sector_spans() {
        let codes = codes(&[
            row(&["Seq. No.", "2022 NAICS US   Code", "2022 NAICS US Title"]),
            row(&["", "", ""]),
            row(&["1", "31-33", "Manufacturing"]),
            row(&["2", "311", "Food  Manufacturing "]),
            row(&["3", "3111", "Animal Food Manufacturing"]),
        ])
        .unwrap();
        assert_eq!(
            codes[1],
            ("311".to_string(), "Food Manufacturing".to_string())
        );
        let mut stats = Stats::default();
        let index = index(
            &[
                row(&["NAICS22", "INDEX ITEM DESCRIPTION"]),
                row(&["******", "Wholesale -- see type of product"]),
                row(&["999999", "Nowhere"]),
            ],
            &mut stats,
        )
        .unwrap();
        let (doc, stats) = build(&codes, &index, stats).unwrap();
        let parents: Vec<_> = doc.classes.iter().map(|c| c.parent.as_deref()).collect();
        assert_eq!(parents, [None, Some("31-33"), Some("311")]);
        let r = doc.classes[0].range.as_ref().unwrap();
        assert_eq!((r.start.as_str(), r.end.as_str()), ("31", "33"));
        assert_eq!(stats.cross_references, 1);
        assert_eq!(stats.unknown_codes.len(), 1);
    }

    #[test]
    fn keeps_each_index_term_once_and_skips_title_echoes() {
        let codes = vec![
            ("11".to_string(), "Agriculture".to_string()),
            ("111".to_string(), "Crop Production".to_string()),
            ("1111".to_string(), "Oilseed".to_string()),
            ("11111".to_string(), "Soybean Farming".to_string()),
            ("111110".to_string(), "Soybean Farming".to_string()),
        ];
        let index: Vec<_> = ["Soybean farming", "Soybean seed", "soybean SEED"]
            .iter()
            .map(|t| ("111110".to_string(), t.to_string()))
            .collect();
        let (doc, stats) = build(&codes, &index, Stats::default()).unwrap();
        let terms: Vec<_> = doc.classes[4]
            .vocab
            .iter()
            .map(|v| v.term.as_str())
            .collect();
        assert_eq!(terms, ["Soybean seed"]);
        assert_eq!(
            (stats.echoes, stats.duplicates, stats.index_terms),
            (1, 1, 1)
        );
    }

    #[test]
    fn a_code_without_its_parent_is_an_error() {
        let codes = vec![("1111".to_string(), "Orphan".to_string())];
        assert!(build(&codes, &[], Stats::default()).is_err());
    }
}
