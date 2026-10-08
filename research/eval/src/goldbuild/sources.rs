//! Parsers for the raw source snapshots (arXiv Atom, Open Library JSON,
//! Wikipedia summary + Wikidata claims JSON).

use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ArxivEntry {
    pub title: String,
    pub summary: String,
    pub primary_category: String,
    pub doi: Option<String>,
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn element<'a>(xml: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}");
    let start = xml.find(&open)?;
    let body = start + xml[start..].find('>')? + 1;
    let end = body + xml[body..].find(&format!("</{name}>"))?;
    Some(&xml[body..end])
}

/// The first `<entry>` of an arXiv API Atom response.
pub fn arxiv_entry(xml: &str) -> Option<ArxivEntry> {
    let entry = element(xml, "entry")?;
    let tag = "<arxiv:primary_category";
    let pc = &entry[entry.find(tag)? + tag.len()..];
    let term = &pc[pc.find("term=\"")? + 6..];
    let primary_category = term[..term.find('"')?].to_string();
    Some(ArxivEntry {
        title: squash(&unescape(element(entry, "title")?)),
        summary: squash(&unescape(element(entry, "summary")?)),
        primary_category,
        doi: element(entry, "arxiv:doi").map(|d| squash(&unescape(d))),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenLibrary {
    pub title: String,
    pub description: String,
    pub lc_classifications: Vec<String>,
    pub identifiers: BTreeMap<String, String>,
}

fn text_of(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(o) => o.get("value").and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

fn strs(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub fn openlibrary(edition: &str, work: &str) -> Result<OpenLibrary, String> {
    let ed: Value = serde_json::from_str(edition).map_err(|e| format!("edition JSON: {e}"))?;
    let wk: Value = serde_json::from_str(work).map_err(|e| format!("work JSON: {e}"))?;
    let mut title = ed
        .get("title")
        .and_then(Value::as_str)
        .ok_or("edition has no title")?
        .to_string();
    if let Some(sub) = ed.get("subtitle").and_then(Value::as_str) {
        title = format!("{title}: {sub}");
    }
    let description = wk
        .get("description")
        .and_then(text_of)
        .or_else(|| ed.get("description").and_then(text_of))
        .map(|d| squash(&d))
        .unwrap_or_default();
    let mut identifiers = BTreeMap::new();
    for (key, field) in [("isbn", "isbn_13"), ("isbn10", "isbn_10"), ("lccn", "lccn")] {
        if let Some(first) = strs(&ed, field).into_iter().next() {
            identifiers.insert(key.to_string(), first);
        }
    }
    if let Some(w) = wk.get("key").and_then(Value::as_str) {
        identifiers.insert(
            "openlibrary_work".into(),
            w.trim_start_matches("/works/").to_string(),
        );
    }
    Ok(OpenLibrary {
        title,
        description,
        lc_classifications: strs(&ed, "lc_classifications"),
        identifiers,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Wikipedia {
    pub title: String,
    pub page: String,
    pub revision: String,
    pub extract: String,
    pub lcc: Vec<String>,
}

pub fn wikipedia(summary: &str, claims: &str) -> Result<Wikipedia, String> {
    let s: Value = serde_json::from_str(summary).map_err(|e| format!("summary JSON: {e}"))?;
    let c: Value = serde_json::from_str(claims).map_err(|e| format!("claims JSON: {e}"))?;
    let title = s
        .get("title")
        .and_then(Value::as_str)
        .ok_or("summary has no title")?
        .to_string();
    let page = s
        .pointer("/titles/canonical")
        .and_then(Value::as_str)
        .unwrap_or(&title)
        .to_string();
    let lcc = c
        .pointer("/claims/P1149")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter(|cl| cl.get("rank").and_then(Value::as_str) != Some("deprecated"))
                .filter_map(|cl| {
                    cl.pointer("/mainsnak/datavalue/value")
                        .and_then(Value::as_str)
                })
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Ok(Wikipedia {
        title,
        page,
        revision: s
            .get("revision")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        extract: squash(s.get("extract").and_then(Value::as_str).unwrap_or("")),
        lcc,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_arxiv_atom() {
        let xml = r#"<feed><title>q</title><entry><id>http://arxiv.org/abs/2401.1v1</id>
<title>A  &amp; B
 study</title><summary> Lines
 joined. </summary><arxiv:doi>10.1/x</arxiv:doi>
<arxiv:primary_category term="cs.CL" scheme="http://arxiv.org/schemas/atom"/></entry></feed>"#;
        let e = arxiv_entry(xml).unwrap();
        assert_eq!(e.title, "A & B study");
        assert_eq!(e.summary, "Lines joined.");
        assert_eq!(e.primary_category, "cs.CL");
        assert_eq!(e.doi.as_deref(), Some("10.1/x"));
        assert!(arxiv_entry("<feed></feed>").is_none());
    }

    #[test]
    fn parses_openlibrary_and_wikidata() {
        let ed = r#"{"title":"SICP","subtitle":"2e","lc_classifications":["QA76.6 .A255 1996"],"isbn_10":["0262011530"]}"#;
        let wk = r#"{"key":"/works/OL1W","description":{"type":"/type/text","value":"About\n programs."}}"#;
        let ol = openlibrary(ed, wk).unwrap();
        assert_eq!(ol.title, "SICP: 2e");
        assert_eq!(ol.description, "About programs.");
        assert_eq!(ol.identifiers["isbn10"], "0262011530");
        assert_eq!(ol.identifiers["openlibrary_work"], "OL1W");
        let sum = r#"{"title":"Oak","titles":{"canonical":"Oak"},"revision":"7","extract":"An oak is a tree."}"#;
        let claims = r#"{"claims":{"P1149":[{"rank":"normal","mainsnak":{"datavalue":{"value":"SB413.O34"}}},
            {"rank":"deprecated","mainsnak":{"datavalue":{"value":"QK1"}}}]}}"#;
        let wp = wikipedia(sum, claims).unwrap();
        assert_eq!(wp.lcc, vec!["SB413.O34"]);
        assert_eq!(wp.revision, "7");
    }
}
