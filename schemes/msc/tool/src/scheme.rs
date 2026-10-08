//! CSV rows to a `SchemeDoc`.
//!
//! MSC2020 codes come in four shapes, and the shape alone gives the
//! hierarchy:
//!
//! | code    | kind                         | parent  | range         |
//! |---------|------------------------------|---------|---------------|
//! | `05-XX` | section (63 of them)         | none    | `05A00-05-99` |
//! | `05-01` | section-wide form (`-00`..`-11`) | `05-XX` | none      |
//! | `05Cxx` | subsection                   | `05-XX` | `05C00-05C99` |
//! | `05C10` | leaf                         | `05Cxx` | none          |
//!
//! The ranges let `check_code` place an unassigned code (`05C99`, `05-99`)
//! within its subsection or section. In shelf order letters sort before
//! digits, so a section's range runs from its `A` subsection to `-99`.
//! The source carries no LCC or LCSH links, so classes have no vocabulary;
//! search runs over the labels.

use std::collections::BTreeMap;

use shelfmark_core::scheme::{ClassDef, CodeRange, SchemeDoc, Source};

use crate::csv::Row;
use crate::sources::Lock;

pub const NAME: &str = "msc";
pub const VERSION: &str = "2020";
pub const TITLE: &str = "Mathematics Subject Classification 2020 (MSC2020)";
pub const CODE_GRAMMAR: &str = "[0-9]{2}-(?:XX|[0-9]{2})|[0-9]{2}[A-Z](?:xx|[0-9]{2})";

/// What a code's shape says about its place in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape {
    Section,
    SectionForm,
    Subsection,
    Leaf,
}

/// The shape of `code`, or `None` when it fits no MSC2020 shape.
pub fn shape(code: &str) -> Option<Shape> {
    let b = code.as_bytes();
    if b.len() != 5 || !b[0].is_ascii_digit() || !b[1].is_ascii_digit() {
        return None;
    }
    let digits = b[3].is_ascii_digit() && b[4].is_ascii_digit();
    match b[2] {
        b'-' if &b[3..] == b"XX" => Some(Shape::Section),
        b'-' if digits => Some(Shape::SectionForm),
        c if c.is_ascii_uppercase() && &b[3..] == b"xx" => Some(Shape::Subsection),
        c if c.is_ascii_uppercase() && digits => Some(Shape::Leaf),
        _ => None,
    }
}

/// The parent code a code's shape implies.
fn parent(code: &str, shape: &Shape) -> Option<String> {
    match shape {
        Shape::Section => None,
        Shape::SectionForm | Shape::Subsection => Some(format!("{}-XX", &code[..2])),
        Shape::Leaf => Some(format!("{}xx", &code[..3])),
    }
}

fn range(code: &str, shape: &Shape) -> Option<CodeRange> {
    let (start, end) = match shape {
        Shape::Section => (format!("{}A00", &code[..2]), format!("{}-99", &code[..2])),
        Shape::Subsection => (format!("{}00", &code[..3]), format!("{}99", &code[..3])),
        Shape::SectionForm | Shape::Leaf => return None,
    };
    Some(CodeRange { start, end })
}

/// The label and note for a row. The description is the caption plus
/// cross-references, which become the note. For the `05C99` rows the
/// caption repeats the subsection's and the description is the printed
/// caption, "None of the above, but in this section", so that is the label.
fn label_note(row: &Row) -> (String, Option<String>) {
    match row.description.strip_prefix(row.text.as_str()) {
        Some(rest) => {
            let rest = rest.trim();
            (
                row.text.clone(),
                (!rest.is_empty()).then(|| rest.to_string()),
            )
        }
        None if row.description.is_empty() => (row.text.clone(), None),
        None => (row.description.clone(), None),
    }
}

/// Build the scheme from the parsed rows and the lock they matched.
/// Classes keep the source's order; labels are the captions verbatim,
/// TeX math (`\(p\)-adic`) included.
pub fn build(rows: &[Row], lock: &Lock) -> Result<SchemeDoc, String> {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    let mut classes = Vec::with_capacity(rows.len());
    for row in rows {
        let at = format!("line {}", row.line);
        let shape = shape(&row.code)
            .ok_or_else(|| format!("{at}: `{}` is not an MSC2020 code", row.code))?;
        if let Some(first) = seen.insert(&row.code, row.line) {
            return Err(format!(
                "{at}: duplicate code `{}` (first on line {first})",
                row.code
            ));
        }
        let (label, note) = label_note(row);
        classes.push(ClassDef {
            code: row.code.clone(),
            label,
            parent: parent(&row.code, &shape),
            range: range(&row.code, &shape),
            vocab: Vec::new(),
            note,
        });
    }
    for (c, row) in classes.iter().zip(rows) {
        if let Some(p) = &c.parent {
            if !seen.contains_key(p.as_str()) {
                return Err(format!(
                    "line {}: `{}` has no parent `{p}`",
                    row.line, c.code
                ));
            }
        }
    }
    Ok(SchemeDoc {
        schema: Some(shelfmark_core::scheme::SCHEME_SCHEMA_ID.to_string()),
        name: NAME.into(),
        version: VERSION.into(),
        title: TITLE.into(),
        source: Some(Source {
            description: format!(
                "{}. Built by shelfmark schemes/msc/tool from {} ({} bytes, sha256 {}).",
                lock.attribution, lock.file, lock.bytes, lock.sha256
            ),
            url: Some(lock.url.clone()),
            license: Some(format!("{} ({})", lock.license, lock.license_url)),
        }),
        code_grammar: CODE_GRAMMAR.into(),
        classes,
    })
}

/// The scheme file's bytes: pretty JSON with a trailing newline, so the
/// same rows always give the same file.
pub fn to_json(doc: &SchemeDoc) -> Result<String, String> {
    serde_json::to_string_pretty(doc)
        .map(|s| s + "\n")
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes() {
        assert_eq!(shape("05-XX"), Some(Shape::Section));
        assert_eq!(shape("97-01"), Some(Shape::SectionForm));
        assert_eq!(shape("05Cxx"), Some(Shape::Subsection));
        assert_eq!(shape("68P27"), Some(Shape::Leaf));
        for bad in ["05C1", "5C10", "05cxx", "05CXX", "05-xx", "QA76", "05C10 "] {
            assert_eq!(shape(bad), None, "{bad}");
        }
    }

    #[test]
    fn parents_and_ranges_follow_the_shape() {
        assert_eq!(parent("05C10", &Shape::Leaf).as_deref(), Some("05Cxx"));
        assert_eq!(
            parent("05-01", &Shape::SectionForm).as_deref(),
            Some("05-XX")
        );
        let r = range("05-XX", &Shape::Section).unwrap();
        assert_eq!((r.start.as_str(), r.end.as_str()), ("05A00", "05-99"));
    }
}
