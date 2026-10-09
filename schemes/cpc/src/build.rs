//! Title list to scheme: the five CPC levels (section `A`, class `A01`,
//! subclass `A01B`, main group `A01B1/00`, subgroups `A01B1/02` ...) and
//! the parent of each.
//!
//! Above main-group level the parent is the symbol's prefix. A main
//! group's parent is its subclass. A subgroup's parent is the nearest
//! earlier group in the same main group's run of lines with one dot fewer,
//! which is how the official scheme nests them; the numbers alone do not
//! say, and a 2000-series code can sit inside another main group's run.
//!
//! No class gets a `range`. Subgroup numbers are decimal fractions
//! (`1/022` sits under `1/02`), but shelfmark's shelf-order key drops the
//! trailing zeros of every digit run after the first, so main group
//! `A01B10` keys like `A01B1` and a range would cover the wrong codes.

use std::collections::HashSet;

use regex::Regex;
use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, SCHEME_SCHEMA_ID};

use crate::titles::Title;

/// Every CPC symbol: section, class, subclass, then `group/subgroup`.
pub const CODE_GRAMMAR: &str = "[A-HY]([0-9]{2}([A-Z]([0-9]{1,4}/[0-9]{2,6})?)?)?";

/// How deep the scheme goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Depth {
    Section,
    Class,
    Subclass,
    MainGroup,
    All,
}

impl Depth {
    pub fn parse(s: &str) -> Option<Depth> {
        Some(match s {
            "section" => Depth::Section,
            "class" => Depth::Class,
            "subclass" => Depth::Subclass,
            "main-group" => Depth::MainGroup,
            "all" => Depth::All,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Depth::Section => "section",
            Depth::Class => "class",
            Depth::Subclass => "subclass",
            Depth::MainGroup => "main-group",
            Depth::All => "all",
        }
    }

    fn of(t: &Title) -> Depth {
        match (t.level, t.symbol.len()) {
            (Some(0), _) => Depth::MainGroup,
            (Some(_), _) => Depth::All,
            (None, 1) => Depth::Section,
            (None, 3) => Depth::Class,
            (None, _) => Depth::Subclass,
        }
    }
}

/// Where the titles came from, for the scheme's `source` block.
pub struct Provenance {
    /// CPC release as `YYYY.MM`, the scheme version.
    pub release: String,
    pub file: String,
    pub url: String,
    pub sha256: String,
}

pub const SITE: &str = "https://www.cooperativepatentclassification.org/";
pub const LICENSE: &str =
    "CPC is jointly owned by the European Patent Office and the US Patent and \
Trademark Office. Reused under the EPO website terms of use, section 6: cite \
www.cooperativepatentclassification.org as the source, mark changes, and pass the same terms on. \
Not redistributed with shelfmark; build it locally with schemes/cpc.";

/// Build the scheme down to `depth`.
pub fn build(titles: &[Title], prov: &Provenance, depth: Depth) -> Result<SchemeDoc, String> {
    let grammar = grammar()?;
    let mut seen: HashSet<&str> = HashSet::new();
    // The open group at each dot level of the current main group.
    let mut open: Vec<&str> = Vec::new();
    let mut classes = Vec::new();
    for t in titles {
        let code = t.symbol.as_str();
        if !grammar.is_match(code) {
            return Err(format!("`{code}` is not a CPC symbol"));
        }
        if t.title.is_empty() {
            return Err(format!("`{code}` has no title"));
        }
        if !seen.insert(code) {
            return Err(format!("`{code}` appears twice"));
        }
        let parent = match t.level {
            None if code.contains('/') => return Err(format!("group `{code}` has no level")),
            Some(_) if !code.contains('/') => {
                return Err(format!("`{code}` has a level but is not a group"))
            }
            None => match code.len() {
                1 => None,
                3 => Some(&code[..1]),
                4 => Some(&code[..3]),
                _ => return Err(format!("`{code}` is not a CPC symbol")),
            },
            Some(0) => {
                open.clear();
                open.push(code);
                Some(&code[..4])
            }
            Some(l) => {
                // Not always the same main group: a 2000-series code such
                // as A01C2001/048 sits inside main group A01C1/00.
                let l = usize::from(l);
                let parent = open
                    .get(l - 1)
                    .copied()
                    .filter(|p| p[..4] == code[..4])
                    .ok_or_else(|| format!("`{code}` (level {l}) has no open parent group"))?;
                open.truncate(l);
                open.push(code);
                Some(parent)
            }
        };
        if let Some(p) = parent {
            if !seen.contains(p) {
                return Err(format!("`{code}` comes before its parent `{p}`"));
            }
        }
        if Depth::of(t) > depth {
            continue;
        }
        classes.push(ClassDef {
            code: code.to_string(),
            label: t.title.clone(),
            parent: parent.map(str::to_string),
            range: None,
            vocab: Vec::new(),
            note: None,
        });
    }
    if classes.is_empty() {
        return Err("the title list has no classes".into());
    }
    Ok(SchemeDoc {
        schema: Some(SCHEME_SCHEMA_ID.to_string()),
        name: "cpc".to_string(),
        version: prov.release.clone(),
        title: "Cooperative Patent Classification".to_string(),
        source: Some(Source {
            description: format!(
                "CPC scheme release {release}: every symbol and title from the official title list \
                 {file} (sha256 {sha}), {url}. Hierarchy from the symbols and the title list's dot \
                 levels; titles verbatim apart from collapsed whitespace; depth {depth}. CPC carries \
                 no LCC or LCSH links. Built by schemes/cpc; see its README.md.",
                release = prov.release,
                file = prov.file,
                sha = prov.sha256,
                url = prov.url,
                depth = depth.as_str(),
            ),
            url: Some(SITE.to_string()),
            license: Some(LICENSE.to_string()),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    })
}

/// [`CODE_GRAMMAR`], anchored as the scheme validator anchors it.
fn grammar() -> Result<Regex, String> {
    Regex::new(&format!("^(?:{CODE_GRAMMAR})$")).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(symbol: &str, level: Option<u8>, title: &str) -> Title {
        Title {
            symbol: symbol.into(),
            level,
            title: title.into(),
        }
    }

    fn prov() -> Provenance {
        Provenance {
            release: "2026.08".into(),
            file: "CPCTitleList202608.zip".into(),
            url: "https://example.org/CPCTitleList202608.zip".into(),
            sha256: "00".into(),
        }
    }

    fn sample() -> Vec<Title> {
        vec![
            t("A", None, "HUMAN NECESSITIES"),
            t("A01", None, "AGRICULTURE"),
            t("A01B", None, "SOIL WORKING"),
            t("A01B1/00", Some(0), "Hand tools"),
            t("A01B1/02", Some(1), "Spades; Shovels"),
            t("A01B1/022", Some(2), "{Collapsible}"),
            t("A01B2001/024", Some(2), "{Machines}"),
            t("A01B1/04", Some(2), "with teeth"),
            t("A01B1/06", Some(1), "Hoes; Hand cultivators"),
            t("A01B3/00", Some(0), "Ploughs"),
        ]
    }

    fn parents(doc: &SchemeDoc) -> Vec<(&str, Option<&str>)> {
        doc.classes
            .iter()
            .map(|c| (c.code.as_str(), c.parent.as_deref()))
            .collect()
    }

    #[test]
    fn nests_by_symbol_and_dot_level() {
        let doc = build(&sample(), &prov(), Depth::All).unwrap();
        assert_eq!(
            parents(&doc),
            [
                ("A", None),
                ("A01", Some("A")),
                ("A01B", Some("A01")),
                ("A01B1/00", Some("A01B")),
                ("A01B1/02", Some("A01B1/00")),
                ("A01B1/022", Some("A01B1/02")),
                ("A01B2001/024", Some("A01B1/02")),
                ("A01B1/04", Some("A01B1/02")),
                ("A01B1/06", Some("A01B1/00")),
                ("A01B3/00", Some("A01B")),
            ]
        );
        assert_eq!(doc.id().to_string(), "scheme:cpc@2026.08");
        assert!(doc.classes.iter().all(|c| c.range.is_none()));
    }

    #[test]
    fn cuts_at_depth() {
        let doc = build(&sample(), &prov(), Depth::MainGroup).unwrap();
        let codes: Vec<&str> = doc.classes.iter().map(|c| c.code.as_str()).collect();
        assert_eq!(codes, ["A", "A01", "A01B", "A01B1/00", "A01B3/00"]);
        let doc = build(&sample(), &prov(), Depth::Class).unwrap();
        assert_eq!(doc.classes.len(), 2);
    }

    #[test]
    fn rejects_broken_lists() {
        let mut s = sample();
        s[4].level = Some(2);
        assert!(build(&s, &prov(), Depth::All)
            .unwrap_err()
            .contains("no open parent"));
        let mut s = sample();
        s.remove(2);
        assert!(build(&s, &prov(), Depth::All)
            .unwrap_err()
            .contains("before its parent"));
        let mut s = sample();
        s.push(t("A01B3/00", Some(0), "again"));
        assert!(build(&s, &prov(), Depth::All)
            .unwrap_err()
            .contains("twice"));
        let mut s = sample();
        s[3].level = None;
        assert!(build(&s, &prov(), Depth::All).is_err());
    }

    #[test]
    fn grammar_matches_cpc_symbols() {
        let g = grammar().unwrap();
        for ok in [
            "A",
            "Y",
            "A01",
            "A01B",
            "A01B1/00",
            "H04L9/0861",
            "Y02E10/50",
            "G06F2203/04808",
        ] {
            assert!(g.is_match(ok), "{ok}");
        }
        for bad in [
            "",
            "I",
            "A0",
            "A01b",
            "A01B1",
            "A01B/00",
            "A01B1/0",
            "A01B12345/00",
            "A01B1/00 ",
        ] {
            assert!(!g.is_match(bad), "{bad}");
        }
    }
}
