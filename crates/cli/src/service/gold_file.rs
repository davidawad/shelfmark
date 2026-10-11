//! `eval --gold FILE`: labelled examples from NDJSON (R15, R16). A line is
//! either a [`GoldExample`] as the store writes it, or a gold item in the
//! mc-eval format (`title`, `text`, `identifiers`, `acceptable`,
//! `source`). An item becomes one example per acceptable class that
//! resolves in the scheme; together they are its acceptable set.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{json, Value};
use shelfmark_core::calibration::GoldExample;
use shelfmark_core::placement::{AuthorKind, Provenance};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};
use shelfmark_core::tier::ClassifyInput;
use shelfmark_core::{ClassId, Error, ErrorCode};

use super::classify::with_work;
use crate::wiring::parse_identifier;

/// What a gold file adds to the store's gold set.
pub(super) struct Imported {
    /// New examples, in file order.
    pub examples: Vec<GoldExample>,
    /// Lines that gave no example, with why.
    pub skipped: Vec<Value>,
    /// Examples the gold set already holds.
    pub duplicates: usize,
}

#[derive(Deserialize)]
struct Item {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    text: String,
    #[serde(default)]
    identifiers: BTreeMap<String, Value>,
    acceptable: Vec<Acceptable>,
    #[serde(default)]
    source: Option<Source>,
}

#[derive(Deserialize)]
struct Acceptable {
    code: String,
}

#[derive(Deserialize)]
struct Source {
    kind: String,
    #[serde(default)]
    record: Option<String>,
    #[serde(default)]
    retrieved_at: Option<String>,
}

/// The typed identifier prefixes an item's identifier keys map to. The
/// source URL is never passed: it is where the label came from.
const IDENTIFIER_KEYS: [(&str, &str); 6] = [
    ("isbn", "isbn"),
    ("isbn10", "isbn"),
    ("isbn13", "isbn"),
    ("lccn", "lccn"),
    ("doi", "doi"),
    ("arxiv", "arxiv"),
];

fn unreadable(path: &str, line: usize, what: impl std::fmt::Display) -> Error {
    Error::new(
        ErrorCode::InputUnreadable,
        format!("{path} line {line}: {what}"),
    )
    .next("shelfmark describe eval")
}

/// Read `path`, skipping examples `existing` already holds.
pub(super) fn read(
    path: &str,
    index: &dyn CompiledIndex,
    existing: &[GoldExample],
) -> Result<Imported, Error> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        Error::new(
            ErrorCode::InputUnreadable,
            format!("cannot read {path}: {e}"),
        )
        .next("shelfmark describe eval")
    })?;
    let mut out = Imported {
        examples: Vec::new(),
        skipped: Vec::new(),
        duplicates: 0,
    };
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line).map_err(|e| unreadable(path, n, e))?;
        let examples = if value.get("acceptable").is_some() {
            let item: Item = serde_json::from_value(value).map_err(|e| unreadable(path, n, e))?;
            match item_examples(&item, index) {
                Ok(examples) => examples,
                Err(reason) => {
                    out.skipped
                        .push(json!({ "line": n, "id": item.id, "reason": reason }));
                    continue;
                }
            }
        } else {
            vec![serde_json::from_value(value).map_err(|e| unreadable(path, n, e))?]
        };
        for e in examples {
            let held = |g: &GoldExample| {
                g.work == e.work
                    && g.scheme == e.scheme
                    && g.class == e.class
                    && g.from_placement == e.from_placement
            };
            if existing.iter().any(held) || out.examples.iter().any(held) {
                out.duplicates += 1;
            } else {
                out.examples.push(e);
            }
        }
    }
    Ok(out)
}

/// One example per acceptable class that resolves in `index`, or why
/// there are none.
fn item_examples(item: &Item, index: &dyn CompiledIndex) -> Result<Vec<GoldExample>, String> {
    let mut identifiers: Vec<_> = IDENTIFIER_KEYS
        .iter()
        .filter_map(|(key, prefix)| {
            let v = item.identifiers.get(*key)?.as_str()?;
            parse_identifier(&format!("{prefix}:{v}"))
        })
        .collect();
    identifiers.sort();
    identifiers.dedup();
    let input = with_work(ClassifyInput {
        work: None,
        title: item.title.clone(),
        authors: item.authors.clone(),
        text: item.text.clone(),
        identifiers,
        url: None,
        headings: Vec::new(),
    })
    .ok_or("no text, title or identifiers to classify")?;
    let mut classes: Vec<ClassId> = Vec::new();
    for a in &item.acceptable {
        if let CodeCheck::Exact { class } | CodeCheck::Within { class } = index.check_code(&a.code)
        {
            if !classes.contains(&class) {
                classes.push(class);
            }
        }
    }
    if classes.is_empty() {
        let codes: Vec<&str> = item.acceptable.iter().map(|a| a.code.as_str()).collect();
        return Err(format!(
            "no acceptable class ({}) is in {}",
            codes.join(", "),
            index.scheme()
        ));
    }
    let author = match &item.source {
        Some(s) => format!("{}:{}", s.kind, s.record.as_deref().unwrap_or("?")),
        None => item.id.clone().unwrap_or_else(|| "gold file".into()),
    };
    let at = item
        .source
        .as_ref()
        .and_then(|s| s.retrieved_at.clone())
        .unwrap_or_else(crate::time::now_rfc3339);
    let work = input.work.clone().expect("with_work sets the work");
    Ok(classes
        .into_iter()
        .map(|class| GoldExample {
            work: work.clone(),
            item: None,
            scheme: index.scheme().clone(),
            class,
            input: input.clone(),
            from_placement: None,
            provenance: Provenance {
                author_kind: AuthorKind::Tool,
                author: author.clone(),
                model: None,
                harness: Some(format!(
                    "shelfmark-cli {} eval --gold",
                    env!("CARGO_PKG_VERSION")
                )),
                run_id: None,
                prompt_hash: None,
                at: at.clone(),
            },
        })
        .collect())
}
