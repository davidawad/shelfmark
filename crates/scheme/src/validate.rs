//! Validation: the published JSON Schema first, then the checks a schema
//! cannot express (code grammar, parents, ranges). Every problem carries
//! the JSON pointer of the value at fault.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shelfmark_core::scheme::SchemeDoc;

use crate::code::{Key, Span};

/// The published scheme schema, embedded so validation needs no files.
pub const SCHEME_SCHEMA: &str = include_str!("../../../schemas/scheme.schema.json");

/// One problem in a scheme file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Diagnostic {
    /// JSON pointer (RFC 6901) to the offending value; empty for the root.
    pub pointer: String,
    pub message: String,
}

impl Diagnostic {
    fn new(pointer: impl Into<String>, message: impl Into<String>) -> Self {
        Diagnostic {
            pointer: pointer.into(),
            message: message.into(),
        }
    }
}

fn validator() -> &'static jsonschema::Validator {
    static V: OnceLock<jsonschema::Validator> = OnceLock::new();
    V.get_or_init(|| {
        let schema: Value = serde_json::from_str(SCHEME_SCHEMA).expect("embedded schema parses");
        jsonschema::validator_for(&schema).expect("embedded schema compiles")
    })
}

/// The code grammar anchored to the whole code.
pub(crate) fn grammar(pattern: &str) -> Result<Regex, regex::Error> {
    Regex::new(&format!("^(?:{pattern})$"))
}

/// Parse and validate scheme bytes. Problems come back sorted by pointer
/// so the same file always reports the same list.
pub fn validate_bytes(bytes: &[u8]) -> Result<SchemeDoc, Vec<Diagnostic>> {
    let value: Value = serde_json::from_slice(bytes).map_err(|e| {
        vec![Diagnostic::new(
            "",
            format!("not JSON (line {}, column {}): {e}", e.line(), e.column()),
        )]
    })?;
    validate_value(&value)
}

/// Validate an already-parsed scheme document.
pub fn validate_value(value: &Value) -> Result<SchemeDoc, Vec<Diagnostic>> {
    let mut errors: Vec<Diagnostic> = validator()
        .iter_errors(value)
        .map(|e| Diagnostic::new(e.instance_path().as_str(), e.to_string()))
        .collect();
    if !errors.is_empty() {
        errors.sort();
        errors.dedup();
        return Err(errors);
    }
    let doc: SchemeDoc = serde_json::from_value(value.clone()).map_err(|e| {
        vec![Diagnostic::new(
            "",
            format!("does not match the scheme model: {e}"),
        )]
    })?;
    let mut errors = check(&doc);
    if errors.is_empty() {
        Ok(doc)
    } else {
        errors.sort();
        Err(errors)
    }
}

/// The checks beyond the schema.
fn check(doc: &SchemeDoc) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let re = match grammar(&doc.code_grammar) {
        Ok(re) => Some(re),
        Err(e) => {
            out.push(Diagnostic::new(
                "/code_grammar",
                format!("not a valid regex: {e}"),
            ));
            None
        }
    };
    let gram = |ptr: String, code: &str, out: &mut Vec<Diagnostic>| {
        if let Some(re) = &re {
            if !re.is_match(code) {
                let msg = format!(
                    "`{code}` does not match code_grammar `{}`",
                    doc.code_grammar
                );
                out.push(Diagnostic::new(ptr, msg));
            }
        }
    };

    let mut first: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, c) in doc.classes.iter().enumerate() {
        let at = format!("/classes/{i}");
        gram(format!("{at}/code"), &c.code, &mut out);
        match first.get(c.code.as_str()) {
            Some(j) => {
                let msg = format!("duplicate code `{}` (first at /classes/{j})", c.code);
                out.push(Diagnostic::new(format!("{at}/code"), msg));
            }
            None => {
                first.insert(&c.code, i);
            }
        }
        if let Some(r) = &c.range {
            gram(format!("{at}/range/start"), &r.start, &mut out);
            gram(format!("{at}/range/end"), &r.end, &mut out);
            if !Span::of(r).well_ordered() {
                let msg = format!("range start `{}` sorts after end `{}`", r.start, r.end);
                out.push(Diagnostic::new(format!("{at}/range"), msg));
            }
        }
    }

    for (i, c) in doc.classes.iter().enumerate() {
        let at = format!("/classes/{i}");
        let Some(p) = &c.parent else { continue };
        let Some(&pi) = first.get(p.as_str()) else {
            out.push(Diagnostic::new(
                format!("{at}/parent"),
                format!("no class `{p}`"),
            ));
            continue;
        };
        if cycles(doc, &first, i) {
            let msg = format!("`{}` is its own ancestor", c.code);
            out.push(Diagnostic::new(format!("{at}/parent"), msg));
            continue;
        }
        let Some(r) = &c.range else { continue };
        let span = Span::of(r);
        let parent = &doc.classes[pi];
        let nested = match &parent.range {
            Some(pr) => span.within(&Span::of(pr)),
            None => {
                let pk = Key::parse(&parent.code);
                span.start.under(&pk) && span.end.under(&pk)
            }
        };
        if !nested {
            let outer = match &parent.range {
                Some(pr) => format!("{}-{}", pr.start, pr.end),
                None => parent.code.clone(),
            };
            let msg = format!(
                "range {}-{} is not inside parent `{}` ({outer})",
                r.start, r.end, parent.code
            );
            out.push(Diagnostic::new(format!("{at}/range"), msg));
        }
    }
    out
}

/// Whether following parents from class `i` returns to it.
fn cycles(doc: &SchemeDoc, first: &BTreeMap<&str, usize>, i: usize) -> bool {
    let mut seen = BTreeSet::new();
    let mut cur = i;
    while let Some(p) = &doc.classes[cur].parent {
        let Some(&next) = first.get(p.as_str()) else {
            return false;
        };
        if next == i {
            return true;
        }
        if !seen.insert(next) {
            return false;
        }
        cur = next;
    }
    false
}
