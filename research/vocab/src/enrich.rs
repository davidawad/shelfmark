//! Fold a heading table into a shelfmark scheme file's vocabulary.
//!
//! Each active heading with LCC numbers becomes a vocabulary link on the
//! scheme class that holds each number: the class itself, or the deepest
//! class whose span or code covers it, as shelfmark's own compiled index
//! decides (`check_code`). A heading held by several classes splits its
//! weight evenly. The LCC outline (U2) only links headings whose number is
//! exactly a class; this links every heading the outline covers.
//!
//! With `inherit > 0`, an active heading with no LCC number of its own
//! borrows the numbers of its nearest broader headings that have one (up to
//! `inherit` levels up), at half the weight per level. That is how headings
//! without an LCC number (most of LCSH) reach a class at all.
//!
//! Terms already on a class (case-insensitively) are not repeated, and
//! headings go in table order (sorted by id), so the output is a pure
//! function of the two inputs.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;
use serde_json::{json, Value};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};

use crate::table::Heading;

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct Stats {
    /// Active (not deprecated) headings in the table.
    pub headings_active: usize,
    pub headings_with_lcc: usize,
    /// Active headings without a number that borrowed one from a broader heading.
    pub headings_inherited: usize,
    /// Headings at least one of whose numbers falls in a class.
    pub headings_placed: usize,
    pub numbers: usize,
    pub numbers_placed: usize,
    pub terms_added: usize,
    pub classes_with_vocab: usize,
}

/// `QH101-QH198` as a scheme span code (`QH101-198`); other forms as is.
fn scheme_code(lcc: &str) -> String {
    let lcc = lcc.split('(').next().unwrap_or(lcc).trim();
    let letters = |s: &str| s.chars().take_while(char::is_ascii_uppercase).count();
    if let Some((a, b)) = lcc.split_once('-') {
        let (la, lb) = (letters(a), letters(b));
        if la > 0 && la == lb && a[..la] == b[..lb] && la < a.len() {
            return format!("{a}-{}", &b[lb..]);
        }
    }
    lcc.to_string()
}

/// `Chemistry, Organic` as `Organic Chemistry`; `None` for other forms.
fn uninverted(label: &str) -> Option<String> {
    let (head, tail) = label.split_once(", ")?;
    let plain = |s: &str| !s.is_empty() && !s.contains([',', '(', ')']);
    (plain(head) && plain(tail)).then(|| format!("{tail} {head}"))
}

/// What to fold in.
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    /// Also add alternate labels and uninverted preferred labels.
    pub alt: bool,
    /// Broader levels a heading without an LCC number may borrow from (0: none).
    pub inherit: usize,
}

/// The LCC numbers of `h`'s nearest broader headings that have any, within
/// `max` levels, with the level they were found at.
fn borrowed<'a>(
    h: &Heading,
    by_id: &HashMap<&str, &'a Heading>,
    max: usize,
) -> Option<(usize, BTreeSet<&'a str>)> {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut frontier: Vec<&str> = h.broader.iter().map(String::as_str).collect();
    for level in 1..=max {
        let next: Vec<&'a Heading> = frontier
            .iter()
            .filter(|id| seen.insert(**id))
            .filter_map(|id| by_id.get(id).copied())
            .filter(|b| !b.deprecated)
            .collect();
        let numbers: BTreeSet<&str> = next
            .iter()
            .flat_map(|b| b.lcc.iter().map(String::as_str))
            .collect();
        if !numbers.is_empty() {
            return Some((level, numbers));
        }
        frontier = next
            .iter()
            .flat_map(|b| b.broader.iter().map(String::as_str))
            .collect();
        if frontier.is_empty() {
            break;
        }
    }
    None
}

/// Add the headings' labels to `scheme` (a scheme document) in place.
pub fn enrich(
    scheme: &mut Value,
    headings: &[Heading],
    index: &dyn CompiledIndex,
    opts: &Options,
) -> Result<Stats, String> {
    let mut stats = Stats::default();
    let mut holder: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut additions: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let by_id: HashMap<&str, &Heading> = headings.iter().map(|h| (h.id.as_str(), h)).collect();
    for h in headings.iter().filter(|h| !h.deprecated) {
        stats.headings_active += 1;
        let (level, numbers): (usize, Vec<&str>) = if !h.lcc.is_empty() {
            stats.headings_with_lcc += 1;
            (0, h.lcc.iter().map(String::as_str).collect())
        } else if opts.inherit > 0 {
            match borrowed(h, &by_id, opts.inherit) {
                Some((level, numbers)) => (level, numbers.into_iter().collect()),
                None => continue,
            }
        } else {
            continue;
        };
        let mut held = BTreeSet::new();
        for lcc in numbers {
            let class = holder.entry(lcc.to_string()).or_insert_with(|| {
                match index.check_code(&scheme_code(lcc)) {
                    CodeCheck::Exact { class } | CodeCheck::Within { class } => {
                        Some(class.code.clone())
                    }
                    CodeCheck::Unplaced | CodeCheck::Invalid { .. } => None,
                }
            });
            held.extend(class.clone());
        }
        if held.is_empty() {
            continue;
        }
        if level == 0 {
            stats.headings_placed += 1;
        } else {
            stats.headings_inherited += 1;
        }
        let mut terms = vec![h.pref_label.clone()];
        if opts.alt {
            terms.extend(h.alt_labels.iter().cloned());
            terms.extend(uninverted(&h.pref_label));
        }
        let weight = 0.5f64.powi(level as i32) / held.len() as f64;
        for class in held {
            for term in &terms {
                let mut link = json!({"term": term, "source": h.vocab, "source_id": h.id});
                if weight < 1.0 {
                    link["weight"] = json!((weight * 10_000.0).round() / 10_000.0);
                }
                additions.entry(class.clone()).or_default().push(link);
            }
        }
    }
    stats.numbers = holder.len();
    stats.numbers_placed = holder.values().filter(|c| c.is_some()).count();
    let classes = scheme["classes"]
        .as_array_mut()
        .ok_or("scheme has no classes array")?;
    for class in classes.iter_mut() {
        let code = class["code"].as_str().unwrap_or_default().to_string();
        if let Some(links) = additions.remove(&code) {
            let vocab = class
                .as_object_mut()
                .ok_or("a class is not an object")?
                .entry("vocab")
                .or_insert_with(|| json!([]));
            let list = vocab.as_array_mut().ok_or("a class vocab is not a list")?;
            let mut seen: BTreeSet<String> = list
                .iter()
                .filter_map(|v| v["term"].as_str().map(str::to_lowercase))
                .collect();
            for link in links {
                let term = link["term"].as_str().unwrap_or_default().to_lowercase();
                if !term.is_empty() && seen.insert(term) {
                    list.push(link);
                    stats.terms_added += 1;
                }
            }
        }
        if class["vocab"].as_array().is_some_and(|v| !v.is_empty()) {
            stats.classes_with_vocab += 1;
        }
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_codes_and_uninverted_labels() {
        assert_eq!(scheme_code("QH101-QH198"), "QH101-198");
        assert_eq!(
            scheme_code("PL772.9-PL772.988(Collections)"),
            "PL772.9-772.988"
        );
        assert_eq!(scheme_code("D-F"), "D-F");
        assert_eq!(scheme_code("QA76.76.C65"), "QA76.76.C65");
        assert_eq!(
            uninverted("Chemistry, Organic").as_deref(),
            Some("Organic Chemistry")
        );
        assert_eq!(uninverted("Songs (High voice), Arranged"), None);
    }
}
