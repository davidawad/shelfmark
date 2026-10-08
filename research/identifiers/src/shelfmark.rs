//! Exports into the shelfmark classifier's own formats (`src/shelfmark`):
//!
//! - [`rules`]: the arXiv category map and the URL-domain rules as one
//!   rules-tier file (`{"rules": [{id, when, class, weight}]}`), shipped
//!   as `src/shelfmark/data/rules/starter.json`.
//! - [`table`]: a built `lookup.tsv` as an identifier-tier table (plan
//!   PD5: one `{"id","class"}` JSON line per identifier), which is
//!   downloaded data and never shipped.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

use crate::rules::{ArxivMap, DomainRules};

/// Weight of a `decide` domain rule and of an arXiv category stamp: above
/// shelfmark's default target (0.8), so it places a Work on its own, but
/// short of an identifier's 1.0 because no librarian has reviewed it.
pub const DECIDE_WEIGHT: f64 = 0.9;
/// Weight of a `hint` domain rule: a prior below every target.
pub const HINT_WEIGHT: f64 = 0.4;

fn regex_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            let special = "\\.+*?()|[]{}^$#&-~".contains(c);
            special
                .then_some('\\')
                .into_iter()
                .chain(std::iter::once(c))
        })
        .collect()
}

/// The starter rules file. arXiv categories match the paper's own stamp,
/// `arXiv:2401.03616v1 [cs.LG]` (new-style IDs carry no category) or an
/// old-style ID `arXiv:hep-th/9901001`. `=arxiv` domain rows are dropped:
/// the domain alone says nothing. Rules come out sorted by id.
pub fn rules(arxiv: &ArxivMap, domains: &DomainRules) -> Value {
    let mut out: BTreeMap<String, Value> = BTreeMap::new();
    for (cat, entry) in &arxiv.0 {
        let c = regex_escape(cat);
        let id = format!("arxiv:{cat}");
        let regex = format!(r"\barXiv:(?:\S+\s+\[{c}\]|{c}/\d{{7}})");
        out.insert(
            id.clone(),
            json!({"id": id, "when": {"regex": regex}, "class": entry.lcc, "weight": DECIDE_WEIGHT}),
        );
    }
    for r in domains.0.iter().filter(|r| r.lcc != "=arxiv") {
        let weight = if r.decides {
            DECIDE_WEIGHT
        } else {
            HINT_WEIGHT
        };
        let (id, when) = if r.path_prefix.is_empty() {
            (format!("domain:{}", r.domain), json!({"domain": r.domain}))
        } else {
            let regex = format!(
                r"(?i)^https?://(?:[^/]*\.)?{}{}",
                regex_escape(&r.domain),
                regex_escape(&r.path_prefix)
            );
            (
                format!("url:{}{}", r.domain, r.path_prefix),
                json!({"regex": regex}),
            )
        };
        out.insert(
            id.clone(),
            json!({"id": id, "when": when, "class": r.lcc, "weight": weight}),
        );
    }
    json!({"rules": out.into_values().collect::<Vec<_>>()})
}

/// `lookup.tsv` and `conflicts.tsv` as identifier-tier JSONL. Per key, the
/// LCC class number (`prefix`) with the most supporting editions wins, ties
/// by byte order. A key whose classes disagree at the subclass letters
/// (`top` conflict) is dropped: the tier answers at confidence 1.0, so an
/// identifier must not be ambiguous. Returns the JSONL and the number of
/// dropped keys.
pub fn table(lookup: &str, conflicts: &str) -> Result<(String, usize), String> {
    let data = |text: &str, header: &str| -> Result<Vec<Vec<String>>, String> {
        let mut lines = text.lines().filter(|l| !l.trim().is_empty());
        if lines.next() != Some(header) {
            return Err(format!("expected header {header:?}"));
        }
        Ok(lines
            .map(|l| l.split('\t').map(str::to_string).collect())
            .collect())
    };
    let ambiguous: BTreeSet<String> = data(conflicts, "key\tscheme\tlevel\teditions\tcandidates")?
        .into_iter()
        .filter(|c| c.len() == 5 && c[1] == "lcc" && c[2] == "top")
        .map(|c| c[0].clone())
        .collect();
    let mut support: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    for (n, c) in data(lookup, "key\tscheme\tprefix\tfull\tsupport")?
        .into_iter()
        .enumerate()
    {
        if c.len() != 5 {
            return Err(format!("lookup line {}: {} columns", n + 2, c.len()));
        }
        if c[1] != "lcc" || ambiguous.contains(&c[0]) {
            continue;
        }
        let s: u64 = c[4]
            .parse()
            .map_err(|_| format!("lookup line {}: bad support", n + 2))?;
        *support
            .entry(c[0].clone())
            .or_default()
            .entry(c[2].clone())
            .or_default() += s;
    }
    let mut out = String::new();
    for (key, classes) in support {
        let best = classes
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
            .map(|(class, _)| class)
            .expect("a key has at least one class");
        out.push_str(&json!({"id": key, "class": best}).to_string());
        out.push('\n');
    }
    Ok((out, ambiguous.len()))
}
