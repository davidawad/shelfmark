//! Extracting identifier and class facts from one Open Library dump line.

use crate::normalize::{self, ClassNumber};
use serde::Deserialize;
use serde_json::Value;

/// The fields of an edition record this builder reads; everything else is skipped.
#[derive(Deserialize, Default)]
#[serde(default)]
struct RawEdition {
    isbn_10: Value,
    isbn_13: Value,
    lccn: Value,
    oclc_numbers: Value,
    lc_classifications: Value,
    dewey_decimal_class: Value,
    identifiers: Value,
    source_records: Value,
}

/// A lookup key: one normalized identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Key {
    Isbn(u64),
    Lccn(Box<str>),
    Oclc(u64),
}

impl Key {
    pub fn kind(&self) -> &'static str {
        match self {
            Key::Isbn(_) => "isbn",
            Key::Lccn(_) => "lccn",
            Key::Oclc(_) => "oclc",
        }
    }

    /// The table's textual key, e.g. `isbn:9780521413824`.
    pub fn render(&self) -> String {
        match self {
            Key::Isbn(n) => format!("isbn:{n:013}"),
            Key::Lccn(s) => format!("lccn:{s}"),
            Key::Oclc(n) => format!("oclc:{n}"),
        }
    }
}

/// Validity counters for one raw field, kept per edition parse.
#[derive(Debug, Default, Clone, Copy)]
pub struct FieldCount {
    pub valid: u64,
    pub invalid: u64,
}

/// Everything one edition contributes: its keys, its class numbers, and
/// per-field counts for the statistics.
#[derive(Debug, Default)]
pub struct EditionFacts {
    pub keys: Vec<Key>,
    pub lcc: Vec<ClassNumber>,
    pub ddc: Vec<ClassNumber>,
    /// Indexed by [`FIELDS`].
    pub counts: [FieldCount; FIELDS.len()],
    /// Field present in the record at all, indexed by [`FIELDS`].
    pub present: [bool; FIELDS.len()],
    /// Rejected raw values, by field index (kept only for small invalid samples).
    pub rejected: Vec<(usize, String)>,
    /// Distinct `source_records` prefixes (e.g. `marc`, `ia`, `amazon`).
    pub sources: Vec<String>,
}

pub const FIELDS: [&str; 6] = ["isbn_10", "isbn_13", "lccn", "oclc", "lcc", "ddc"];
const F_ISBN10: usize = 0;
const F_ISBN13: usize = 1;
const F_LCCN: usize = 2;
const F_OCLC: usize = 3;
const F_LCC: usize = 4;
const F_DDC: usize = 5;

/// Parse one dump line (`type \t key \t revision \t last_modified \t json`).
///
/// Returns `Ok(None)` for non-edition records and `Err` for lines whose JSON
/// does not parse.
pub fn parse_line(line: &str) -> Result<Option<EditionFacts>, String> {
    let mut cols = line.splitn(5, '\t');
    let kind = cols.next().unwrap_or("");
    if kind != "/type/edition" {
        return Ok(None);
    }
    let json = cols
        .nth(3)
        .ok_or_else(|| "missing json column".to_string())?;
    let raw: RawEdition = serde_json::from_str(json).map_err(|e| e.to_string())?;
    Ok(Some(extract(&raw)))
}

fn strings(v: &Value) -> Vec<&str> {
    match v {
        Value::String(s) => vec![s.as_str()],
        Value::Array(items) => items.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

fn extract(raw: &RawEdition) -> EditionFacts {
    let mut f = EditionFacts::default();
    let ids = &raw.identifiers;
    let field_values: [Vec<&str>; FIELDS.len()] = [
        strings(&raw.isbn_10),
        strings(&raw.isbn_13),
        [strings(&raw.lccn), strings(&ids["lccn"])].concat(),
        [strings(&raw.oclc_numbers), strings(&ids["oclc"])].concat(),
        strings(&raw.lc_classifications),
        strings(&raw.dewey_decimal_class),
    ];
    for (field, values) in field_values.iter().enumerate() {
        f.present[field] = values.iter().any(|v| !v.trim().is_empty());
        for value in values.iter().filter(|v| !v.trim().is_empty()) {
            let ok = match field {
                F_ISBN10 | F_ISBN13 => normalize::isbn13(value).map(|n| f.keys.push(Key::Isbn(n))),
                F_LCCN => normalize::lccn(value).map(|s| f.keys.push(Key::Lccn(s.into()))),
                F_OCLC => normalize::oclc(value).map(|n| f.keys.push(Key::Oclc(n))),
                F_LCC => normalize::lcc(value).map(|c| f.lcc.push(c)),
                F_DDC => normalize::ddc(value).map(|c| f.ddc.push(c)),
                _ => unreachable!(),
            }
            .is_some();
            let count = &mut f.counts[field];
            if ok {
                count.valid += 1;
            } else {
                count.invalid += 1;
                f.rejected.push((field, value.to_string()));
            }
        }
    }
    f.keys.sort();
    f.keys.dedup();
    dedup_classes(&mut f.lcc);
    dedup_classes(&mut f.ddc);
    if !f.lcc.is_empty() || !f.ddc.is_empty() {
        let mut sources: Vec<String> = strings(&raw.source_records)
            .iter()
            .map(|s| source_prefix(s).to_string())
            .collect();
        sources.sort();
        sources.dedup();
        f.sources = sources;
    }
    f
}

/// `marc:marc_loc_2016/...` -> `marc`; `marc_overdrive/x.mrc` -> `marc_overdrive`;
/// a bare Internet Archive identifier -> `(bare)`.
fn source_prefix(s: &str) -> &str {
    match s.split_once(':') {
        Some((p, _)) => p.split('/').next().unwrap_or(p),
        None if s.contains('/') => s.split('/').next().unwrap_or(s),
        None => "(bare)",
    }
}

fn dedup_classes(v: &mut Vec<ClassNumber>) {
    v.sort_by(|a, b| a.full.cmp(&b.full));
    v.dedup_by(|a, b| a.full == b.full);
}
