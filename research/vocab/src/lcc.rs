//! Normalisation of Library of Congress Classification numbers as they occur
//! in LCSH authority records (MARC 053 / `madsrdf:classification`).
//!
//! The dumps carry hand-keyed strings: `QA76.76.C65`, `PJ2491-PJ2517`,
//! `QH101-198`, `Ql430.P6`, `P.325.5.O76`, `PL772.9-PL772.988(Collections)`,
//! and bare class letters such as `PS`. We keep the raw text for provenance and
//! emit one canonical form: upper-case class letters glued to the number, a
//! range end that repeats the class letters, and any trailing parenthetical
//! moved into `qualifier`.

/// Initial letters LCC actually uses (I, O, W, X and Y are unassigned).
const LCC_FIRST: &str = "ABCDEFGHJKLMNPQRSTUVZ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LccRef {
    /// Canonical text: `QA76.76.C65` or `PJ2491-PJ2517`.
    pub code: String,
    /// Class letters of the start, e.g. `QA`.
    pub class: String,
    /// Parenthetical caption the cataloguer attached, e.g. `Collections`.
    pub qualifier: Option<String>,
}

impl LccRef {
    pub fn is_range(&self) -> bool {
        self.code.contains('-')
    }

    /// The table cell form: code plus `(qualifier)` when present.
    pub fn cell(&self) -> String {
        match &self.qualifier {
            Some(q) => format!("{}({q})", self.code),
            None => self.code.clone(),
        }
    }
}

/// Code from an `http://id.loc.gov/authorities/classification/<code>` IRI.
pub fn from_classification_iri(iri: &str) -> Option<LccRef> {
    let tail = iri.strip_prefix("http://id.loc.gov/authorities/classification/")?;
    normalize(&percent_decode(tail))
}

pub fn normalize(raw: &str) -> Option<LccRef> {
    let mut text = raw.trim();
    let mut qualifier = None;
    if let Some(open) = text.find('(') {
        let q = text[open + 1..].trim_end_matches(')').trim();
        if !q.is_empty() {
            qualifier = Some(q.to_string());
        }
        text = text[..open].trim();
    }
    let (start, end) = match text.split_once('-') {
        Some((a, b)) => (a.trim(), Some(b.trim())),
        None => (text, None),
    };
    let (class, start_num) = split_class(start)?;
    let mut code = join(&class, &start_num);
    if let Some(end) = end {
        if start_num.is_empty() {
            // `D-F` and `B-BJ` are class-letter ranges; `n-us-ca` is a MARC
            // geographic area code that only looks like one.
            let end_class = split_class(end).filter(|(_, n)| n.is_empty());
            let (end_class, _) = end_class.filter(|_| !end.contains('-'))?;
            return Some(LccRef {
                code: format!("{class}-{end_class}"),
                class,
                qualifier,
            });
        }
        let end_part = if end.starts_with(|c: char| c.is_ascii_digit()) {
            Some(join(&class, end))
        } else if end.starts_with('.') {
            // `PR9408.R8-.R83`: the end replaces the start's last dotted part.
            let stem = code.rfind('.').map_or(code.as_str(), |i| &code[..i]);
            Some(format!("{stem}{end}"))
        } else {
            split_class(end).map(|(c, n)| join(&c, &n))
        };
        match end_part {
            Some(e) => {
                code.push('-');
                code.push_str(&e);
            }
            // `QL638.S2-Zoology`: a caption keyed after a hyphen, not a range.
            None if qualifier.is_none() && end.chars().any(|c| c.is_alphabetic()) => {
                qualifier = Some(end.to_string());
            }
            None => return None,
        }
    }
    Some(LccRef {
        code,
        class,
        qualifier,
    })
}

/// Split `Ql430.P6` into (`QL`, `430.P6`); `None` unless it looks like LCC.
fn split_class(s: &str) -> Option<(String, String)> {
    let letters: String = s.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    if letters.is_empty() || letters.len() > 3 {
        return None;
    }
    let class = letters.to_ascii_uppercase();
    if !LCC_FIRST.contains(&class[..1]) {
        return None;
    }
    let rest = s[letters.len()..].trim_start_matches(['.', ' ']);
    if !rest.is_empty() && !rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let num: String = rest.split_whitespace().collect::<Vec<_>>().join(" ");
    Some((class, num))
}

fn join(class: &str, num: &str) -> String {
    format!("{class}{num}")
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(v) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(raw: &str) -> Option<String> {
        normalize(raw).map(|r| r.cell())
    }

    #[test]
    fn canonicalises_observed_forms() {
        assert_eq!(code("QA76.76.C65").as_deref(), Some("QA76.76.C65"));
        assert_eq!(code("Ql430.P6").as_deref(), Some("QL430.P6"));
        assert_eq!(code("P.325.5.O76").as_deref(), Some("P325.5.O76"));
        assert_eq!(code("QH101-198").as_deref(), Some("QH101-QH198"));
        assert_eq!(code("PJ2491-PJ2517").as_deref(), Some("PJ2491-PJ2517"));
        assert_eq!(
            code("PL772.9-PL772.988(Collections)").as_deref(),
            Some("PL772.9-PL772.988(Collections)")
        );
        assert_eq!(code("PS").as_deref(), Some("PS"));
        assert_eq!(
            code("PR9408.R8-.R83").as_deref(),
            Some("PR9408.R8-PR9408.R83")
        );
        assert_eq!(
            code("PL5489.3-.395").as_deref(),
            Some("PL5489.3-PL5489.395")
        );
        assert_eq!(
            code("QL638.S2-Zoology").as_deref(),
            Some("QL638.S2(Zoology)")
        );
        assert_eq!(code("D-F").as_deref(), Some("D-F"));
        assert_eq!(code("B-BJ").as_deref(), Some("B-BJ"));
        assert_eq!(normalize("QA76").unwrap().class, "QA");
        assert!(normalize("QH101-198").unwrap().is_range());
    }

    #[test]
    fn rejects_non_lcc() {
        assert_eq!(code("n-us-ca"), None);
        assert_eq!(code("e-uk-en"), None);
        assert_eq!(code("IX123"), None);
        assert_eq!(code("ABCD1"), None);
        assert_eq!(code(""), None);
    }

    #[test]
    fn decodes_classification_iris() {
        let r = from_classification_iri("http://id.loc.gov/authorities/classification/QL737.P959")
            .unwrap();
        assert_eq!(r.code, "QL737.P959");
        let r = from_classification_iri(
            "http://id.loc.gov/authorities/classification/PN1993.5.U6%20A2",
        )
        .unwrap();
        assert_eq!(r.code, "PN1993.5.U6 A2");
        assert!(from_classification_iri("http://example.org/QA76").is_none());
    }
}
