//! The GBV's JSKOS export (NDJSON, one concept per line) into a flat list
//! of classes in tree order, with parents taken from each concept's
//! `ancestors` (nearest first).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// One BK class as the export states it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub code: String,
    /// German preferred label.
    pub label: String,
    /// English preferred label, when the export has a non-empty one.
    pub label_en: Option<String>,
    /// German alternative labels.
    pub alt: Vec<String>,
    /// Scope note, definition and cross-reference note, joined.
    pub note: Option<String>,
    /// A heading (JSKOS `GuideTerm`) that is not used for indexing.
    pub guide: bool,
    pub parent: Option<String>,
}

const GUIDE: &str = "http://vocab.getty.edu/ontology#GuideTerm";

/// Every class, parents before children, siblings in code order. Fails on
/// anything the builder would otherwise have to guess at: invalid JSON, a
/// missing notation or German label, a repeated code, a parent that is not
/// in the export, ancestors that disagree with the parent's own, or a
/// `narrower` list that disagrees with the parents.
pub fn entries(ndjson: &[u8]) -> Result<Vec<Entry>, String> {
    let text = std::str::from_utf8(ndjson).map_err(|e| format!("source is not UTF-8: {e}"))?;
    let mut by: BTreeMap<String, (Entry, Vec<String>, Vec<String>)> = BTreeMap::new();
    for (i, line) in text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
    {
        let at = |m: String| format!("line {}: {m}", i + 1);
        let v: Value = serde_json::from_str(line).map_err(|e| at(e.to_string()))?;
        let (entry, ancestors, narrower) = concept(&v).map_err(at)?;
        if by
            .insert(entry.code.clone(), (entry, ancestors, narrower))
            .is_some()
        {
            return Err(at("code appears twice".into()));
        }
    }
    if by.is_empty() {
        return Err("source has no concepts".into());
    }
    let mut children: BTreeMap<Option<String>, Vec<String>> = BTreeMap::new();
    for (code, (e, ancestors, narrower)) in &by {
        if let Some(p) = &e.parent {
            let (_, pa, _) = by
                .get(p)
                .ok_or_else(|| format!("{code}: parent {p} is not in the export"))?;
            if ancestors[1..] != pa[..] || ancestors[0] != *p {
                return Err(format!(
                    "{code}: ancestors {ancestors:?} are not {p} followed by its own {pa:?}"
                ));
            }
        }
        for n in narrower {
            let ok = by
                .get(n)
                .is_some_and(|(c, _, _)| c.parent.as_deref() == Some(code));
            if !ok {
                return Err(format!("{code}: narrower {n} does not name it as parent"));
            }
        }
        children
            .entry(e.parent.clone())
            .or_default()
            .push(code.clone());
    }
    for (c, (e, _, narrower)) in &by {
        let kids = children.get(&Some(c.clone())).map_or(0, Vec::len);
        if kids != narrower.len() {
            return Err(format!(
                "{}: {} children by ancestors but {} in narrower",
                e.code,
                kids,
                narrower.len()
            ));
        }
    }
    for kids in children.values_mut() {
        kids.sort_by_key(|c| sort_key(c));
    }
    let mut out = Vec::with_capacity(by.len());
    let mut seen = BTreeSet::new();
    let mut stack: Vec<String> = children
        .get(&None)
        .map(|r| r.iter().rev().cloned().collect())
        .unwrap_or_default();
    while let Some(code) = stack.pop() {
        if !seen.insert(code.clone()) {
            return Err(format!("{code}: reached twice (cycle)"));
        }
        if let Some(kids) = children.get(&Some(code.clone())) {
            stack.extend(kids.iter().rev().cloned());
        }
        out.push(by[&code].0.clone());
    }
    if out.len() != by.len() {
        return Err(format!(
            "{} of {} concepts are unreachable from a root",
            by.len() - out.len(),
            by.len()
        ));
    }
    Ok(out)
}

/// Orders `5`, `7-8`, `10`, `11.20`, `11.20X` numerically, then by text.
fn sort_key(code: &str) -> (u32, String) {
    let n: String = code.chars().take_while(char::is_ascii_digit).collect();
    (n.parse().unwrap_or(u32::MAX), code.to_string())
}

type Parsed = (Entry, Vec<String>, Vec<String>);

fn concept(v: &Value) -> Result<Parsed, String> {
    let code = notation(v).ok_or("concept has no notation")?;
    let label = lang(&v["prefLabel"], "de")
        .ok_or_else(|| format!("{code} has no German label"))?
        .to_string();
    let ancestors: Vec<String> = codes(&v["ancestors"]).map_err(|e| format!("{code}: {e}"))?;
    let narrower: Vec<String> = codes(&v["narrower"]).map_err(|e| format!("{code}: {e}"))?;
    let mut alt: Vec<String> = Vec::new();
    for a in strings(&v["altLabel"]["de"]) {
        if a != label && !alt.contains(&a) {
            alt.push(a);
        }
    }
    let mut parts = Vec::new();
    for (key, name) in [
        ("scopeNote", "Scope"),
        ("definition", "Definition"),
        ("note", "Note"),
    ] {
        let s = strings(&v[key]["de"]);
        if !s.is_empty() {
            parts.push(format!("{name}: {}", s.join("; ")));
        }
    }
    let guide = v["type"]
        .as_array()
        .is_some_and(|t| t.iter().any(|x| x.as_str() == Some(GUIDE)));
    if guide {
        parts.insert(0, "Heading only; not used for indexing.".to_string());
    }
    let entry = Entry {
        parent: ancestors.first().cloned(),
        label_en: lang(&v["prefLabel"], "en").map(str::to_string),
        code,
        label,
        alt,
        note: (!parts.is_empty()).then(|| parts.join(" | ")),
        guide,
    };
    Ok((entry, ancestors, narrower))
}

fn notation(v: &Value) -> Option<String> {
    let n = v["notation"].get(0)?.as_str()?.trim();
    (!n.is_empty()).then(|| n.to_string())
}

/// The non-empty, whitespace-collapsed string for `lang` in a JSKOS
/// language map.
fn lang<'a>(map: &'a Value, lang: &str) -> Option<&'a str> {
    map.get(lang)?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn codes(v: &Value) -> Result<Vec<String>, String> {
    v.as_array()
        .ok_or("expected a concept list")?
        .iter()
        .map(|c| notation(c).ok_or_else(|| "a listed concept has no notation".to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(code: &str, de: &str, anc: &[&str], nar: &[&str]) -> String {
        let n = |cs: &[&str]| {
            cs.iter()
                .map(|c| format!(r#"{{"notation":["{c}"]}}"#))
                .collect::<Vec<_>>()
                .join(",")
        };
        format!(
            r#"{{"notation":["{code}"],"prefLabel":{{"de":"{de}","en":""}},"ancestors":[{}],"narrower":[{}]}}"#,
            n(anc),
            n(nar)
        )
    }

    #[test]
    fn tree_order_and_parents() {
        let src = [
            line("10.20", "b", &["10", "1-2"], &[]),
            line("10", "mid", &["1-2"], &["10.10", "10.20"]),
            line("2", "other", &[], &[]),
            line("1-2", "top", &[], &["10"]),
            line("10.10", "a", &["10", "1-2"], &[]),
        ]
        .join("\n");
        let e = entries(src.as_bytes()).unwrap();
        let got: Vec<_> = e
            .iter()
            .map(|e| (e.code.as_str(), e.parent.as_deref()))
            .collect();
        assert_eq!(
            got,
            [
                ("1-2", None),
                ("10", Some("1-2")),
                ("10.10", Some("10")),
                ("10.20", Some("10")),
                ("2", None)
            ]
        );
        assert_eq!(e[1].label_en, None, "an empty English label is dropped");
    }

    #[test]
    fn rejects_inconsistent_sources() {
        let dup = [line("1", "a", &[], &[]), line("1", "b", &[], &[])].join("\n");
        assert!(entries(dup.as_bytes()).unwrap_err().contains("twice"));
        let orphan = line("1.10", "a", &["1"], &[]);
        assert!(entries(orphan.as_bytes())
            .unwrap_err()
            .contains("not in the export"));
        let missing_child = [line("1", "a", &[], &["1.10"])].join("\n");
        assert!(entries(missing_child.as_bytes())
            .unwrap_err()
            .contains("narrower"));
        let unlisted = [line("1", "a", &[], &[]), line("1.10", "b", &["1"], &[])].join("\n");
        assert!(entries(unlisted.as_bytes())
            .unwrap_err()
            .contains("narrower"));
        let bad_chain = [
            line("1", "a", &[], &["1.10"]),
            line("1.10", "b", &["1"], &["1.10.10"]),
            line("1.10.10", "c", &["1.10"], &[]),
        ]
        .join("\n");
        assert!(entries(bad_chain.as_bytes())
            .unwrap_err()
            .contains("ancestors"));
        let no_de = r#"{"notation":["1"],"prefLabel":{"en":"x"},"ancestors":[],"narrower":[]}"#;
        assert!(entries(no_de.as_bytes())
            .unwrap_err()
            .contains("German label"));
        assert!(entries(b"{nope").unwrap_err().contains("line 1"));
    }

    #[test]
    fn sort_key_is_numeric_on_the_leading_number() {
        let mut v = ["11", "7-8", "5", "42.10", "42.10X", "42.9"];
        v.sort_by_key(|c| sort_key(c));
        assert_eq!(v, ["5", "7-8", "11", "42.10", "42.10X", "42.9"]);
    }
}
