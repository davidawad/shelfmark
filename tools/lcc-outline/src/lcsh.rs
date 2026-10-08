//! LCSH headings and the LCC numbers they carry, from id.loc.gov's bulk
//! `subjects.madsrdf.nt.gz`. A heading links to its class numbers through
//! blank nodes:
//!
//! ```text
//! <…/subjects/sh85082139> <…mads/rdf/v1#authoritativeLabel> "Mathematics"@en .
//! <…/subjects/sh85082139> <…mads/rdf/v1#classification> _:b1 .
//! _:b1 <…mads/rdf/v1#code> "QA" .
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader};

use flate2::read::GzDecoder;

const SUBJECTS: &str = "<http://id.loc.gov/authorities/subjects/";
const LABEL: &str = "<http://www.loc.gov/mads/rdf/v1#authoritativeLabel>";
const CLASSIFICATION: &str = "<http://www.loc.gov/mads/rdf/v1#classification>";
const CODE: &str = "<http://www.loc.gov/mads/rdf/v1#code>";

/// One heading with an LCC number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Heading {
    pub label: String,
    /// `sh85082139`.
    pub id: String,
    /// The LCC number as LCSH records it, e.g. `QA75.5-QA76.95`.
    pub lcc: String,
}

/// Every heading that has both an authoritative label and an LCC number,
/// sorted.
pub fn headings(gz: &[u8]) -> Result<Vec<Heading>, String> {
    headings_from(BufReader::new(GzDecoder::new(gz)))
}

fn headings_from(reader: impl BufRead) -> Result<Vec<Heading>, String> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    let mut class_nodes: Vec<(String, String)> = Vec::new();
    let mut codes: BTreeMap<String, String> = BTreeMap::new();
    for line in reader.lines() {
        let line = line.map_err(|e| format!("reading LCSH dump: {e}"))?;
        let Some((subject, rest)) = line.split_once(' ') else {
            continue;
        };
        let Some((predicate, object)) = rest.split_once(' ') else {
            continue;
        };
        let object = object.trim_end().trim_end_matches('.').trim_end();
        match predicate {
            LABEL => {
                if let (Some(id), Some(text)) = (subject_id(subject), literal(object)) {
                    labels.insert(id, text);
                }
            }
            CLASSIFICATION => {
                if let Some(id) = subject_id(subject) {
                    class_nodes.push((id, object.to_string()));
                }
            }
            CODE if subject.starts_with("_:") => {
                if let Some(text) = literal(object) {
                    codes.insert(subject.to_string(), text);
                }
            }
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    for (id, node) in class_nodes {
        if let (Some(label), Some(lcc)) = (labels.get(&id), codes.get(&node)) {
            out.insert(Heading {
                label: label.clone(),
                id,
                lcc: lcc.clone(),
            });
        }
    }
    Ok(out.into_iter().collect())
}

fn subject_id(subject: &str) -> Option<String> {
    subject
        .strip_prefix(SUBJECTS)?
        .strip_suffix('>')
        .map(str::to_string)
}

/// The text of an N-Triples literal (`"…"`, `"…"@en`, `"…"^^<…>`).
fn literal(object: &str) -> Option<String> {
    let body = object.strip_prefix('"')?;
    let close = body.rfind('"')?;
    unescape(&body[..close])
}

fn unescape(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next()? {
            't' => out.push('\t'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            'b' => out.push('\u{8}'),
            'f' => out.push('\u{c}'),
            '"' => out.push('"'),
            '\'' => out.push('\''),
            '\\' => out.push('\\'),
            'u' => out.push(hex(&mut chars, 4)?),
            'U' => out.push(hex(&mut chars, 8)?),
            _ => return None,
        }
    }
    Some(out)
}

fn hex(chars: &mut impl Iterator<Item = char>, n: usize) -> Option<char> {
    let digits: String = chars.take(n).collect();
    char::from_u32(u32::from_str_radix(&digits, 16).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_labels_to_codes_through_blank_nodes() {
        let nt = r#"<http://id.loc.gov/authorities/subjects/sh1> <http://www.loc.gov/mads/rdf/v1#classification> _:b1 .
_:b1 <http://www.loc.gov/mads/rdf/v1#code> "QA" .
<http://id.loc.gov/authorities/subjects/sh1> <http://www.loc.gov/mads/rdf/v1#authoritativeLabel> "Mathématiques \"x\""@en .
<http://id.loc.gov/authorities/subjects/sh2> <http://www.loc.gov/mads/rdf/v1#authoritativeLabel> "No class"@en .
<http://id.loc.gov/authorities/subjects/sh3> <http://www.loc.gov/mads/rdf/v1#code> "n-us"^^<http://id.loc.gov/datatypes/codes/gac> .
"#;
        let got = headings_from(nt.as_bytes()).unwrap();
        assert_eq!(
            got,
            vec![Heading {
                label: "Mathématiques \"x\"".into(),
                id: "sh1".into(),
                lcc: "QA".into()
            }]
        );
    }
}
