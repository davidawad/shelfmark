//! Scheme-level librarian scores: notation, hospitality, granularity and
//! literary warrant. Notation scores read the whole scheme; usage scores
//! read the induced hierarchy.
//!
//! * Notation length: mean characters of the items' class codes.
//! * Expressiveness: share of non-top classes whose code extends the parent
//!   code (the notation shows the hierarchy).
//! * Regularity: share of classes whose code shape (letters as `A`, digits
//!   as `9`, runs collapsed) is the most common shape at their depth.
//! * Hospitality in array (Ranganathan): share of adjacent siblings, in
//!   scheme order and ascending natural order, between which the code
//!   grammar admits an unused code, so a new class fits without renumbering.
//! * Granularity: used classes per 1000 placed works, works per used leaf,
//!   and the share of scheme classes (and of scheme leaves) left empty.
//! * Literary warrant: share of all items placed at depth 2 or deeper.
use crate::agree::spearman;
use crate::Hier;
use shelfmark_core::scheme::SchemeDoc;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct SchemeShape {
    pub classes: usize,
    pub expressiveness: f64,
    pub regularity: f64,
    pub length_depth_spearman: f64,
    pub hospitality_array: f64,
    /// Share of adjacent sibling pairs listed in ascending natural order.
    pub ordered_siblings: f64,
    pub lengthening: f64,
}

fn shape(code: &str) -> String {
    let mut out = String::new();
    for ch in code.chars() {
        let s = if ch.is_ascii_digit() {
            '9'
        } else if ch.is_alphabetic() {
            'A'
        } else {
            ch
        };
        if !out.ends_with(s) || !(s == '9' || s == 'A') {
            out.push(s);
        }
    }
    out
}

fn chunks(s: &str) -> Vec<(bool, &str)> {
    let mut out = vec![];
    let mut start = 0;
    let b = s.as_bytes();
    for i in 1..=b.len() {
        if i == b.len() || b[i].is_ascii_digit() != b[start].is_ascii_digit() {
            out.push((b[start].is_ascii_digit(), &s[start..i]));
            start = i;
        }
    }
    out
}

/// Natural order: digit runs compare numerically, other runs as text.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    for (x, y) in chunks(a).into_iter().zip(chunks(b)) {
        let o = if x.0 && y.0 {
            let (tx, ty) = (x.1.trim_start_matches('0'), y.1.trim_start_matches('0'));
            tx.len().cmp(&ty.len()).then(tx.cmp(ty))
        } else {
            x.1.cmp(y.1)
        };
        if o != Ordering::Equal {
            return o;
        }
    }
    chunks(a).len().cmp(&chunks(b).len()).then(a.cmp(b))
}

/// Codes a new sibling right after `a` could take.
fn candidates(a: &str) -> Vec<String> {
    let mut out = vec![];
    let digits_end = a.trim_end_matches(|c: char| !c.is_ascii_digit()).len();
    let digits_start = a[..digits_end]
        .trim_end_matches(|c: char| c.is_ascii_digit())
        .len();
    if digits_start < digits_end {
        if let Ok(x) = a[digits_start..digits_end].parse::<u128>() {
            let width = digits_end - digits_start;
            out.push(format!(
                "{}{:0width$}{}",
                &a[..digits_start],
                x + 1,
                &a[digits_end..]
            ));
        }
    }
    for suffix in [".5", "5", "1", ".1", "A", "a", "-1"] {
        out.push(format!("{a}{suffix}"));
    }
    out
}

fn depth_of<'a>(parent: &BTreeMap<&'a str, Option<&'a str>>, mut code: &'a str) -> usize {
    let mut d = 1;
    while let Some(Some(p)) = parent.get(code) {
        d += 1;
        code = p;
        if d > 64 {
            break;
        }
    }
    d
}

pub fn scheme_shape(scheme: &SchemeDoc) -> SchemeShape {
    let codes: BTreeSet<&str> = scheme.classes.iter().map(|c| c.code.as_str()).collect();
    let parent: BTreeMap<&str, Option<&str>> = scheme
        .classes
        .iter()
        .map(|c| (c.code.as_str(), c.parent.as_deref()))
        .collect();
    let grammar = regex::Regex::new(&format!("^(?:{})$", scheme.code_grammar)).ok();
    let mut shapes = BTreeMap::<usize, BTreeMap<String, usize>>::new();
    let (mut lens, mut depths) = (vec![], vec![]);
    let (mut child_total, mut extends, mut longer) = (0usize, 0usize, 0usize);
    let mut siblings = BTreeMap::<Option<&str>, Vec<&str>>::new();
    for c in &scheme.classes {
        let d = depth_of(&parent, &c.code);
        *shapes
            .entry(d)
            .or_default()
            .entry(shape(&c.code))
            .or_default() += 1;
        lens.push(c.code.chars().count() as f64);
        depths.push(d as f64);
        if let Some(p) = c.parent.as_deref() {
            child_total += 1;
            extends += usize::from(c.code.starts_with(p) && c.code != p);
            longer += usize::from(c.code.len() >= p.len());
        }
        siblings
            .entry(c.parent.as_deref())
            .or_default()
            .push(&c.code);
    }
    let (mut ordered, mut roomy, mut adjacent) = (0usize, 0usize, 0usize);
    for list in siblings.values() {
        for w in list.windows(2) {
            adjacent += 1;
            if natural_cmp(w[0], w[1]) != Ordering::Less {
                continue;
            }
            ordered += 1;
            let fits = |c: &String| {
                !codes.contains(c.as_str())
                    && grammar.as_ref().is_none_or(|g| g.is_match(c))
                    && natural_cmp(w[0], c) == Ordering::Less
                    && natural_cmp(c, w[1]) == Ordering::Less
            };
            roomy += usize::from(candidates(w[0]).iter().any(fits));
        }
    }
    let n = scheme.classes.len().max(1) as f64;
    let share = |a: usize, b: usize| if b > 0 { a as f64 / b as f64 } else { 0.0 };
    SchemeShape {
        classes: scheme.classes.len(),
        expressiveness: share(extends, child_total),
        regularity: shapes
            .values()
            .map(|m| m.values().copied().max().unwrap_or(0))
            .sum::<usize>() as f64
            / n,
        length_depth_spearman: spearman(&lens, &depths),
        hospitality_array: share(roomy, ordered),
        ordered_siblings: share(ordered, adjacent),
        lengthening: share(longer, child_total),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize)]
pub struct Usage {
    pub notation_length: f64,
    pub classes_per_1000_works: f64,
    pub works_per_leaf_mean: f64,
    pub works_per_leaf_median: f64,
    pub empty_class_share: f64,
    pub empty_leaf_share: f64,
    pub useful_depth_share: f64,
}

pub fn usage(h: &Hier, scheme: &SchemeDoc) -> Usage {
    let placed: Vec<usize> = h.item_class.iter().copied().filter(|&c| c != 0).collect();
    let np = placed.len().max(1) as f64;
    let holding = (1..h.len()).filter(|&v| !h.direct[v].is_empty()).count();
    let mut leaves: Vec<usize> = (1..h.len())
        .filter(|&v| h.children[v].is_empty())
        .map(|v| h.direct[v].len())
        .collect();
    leaves.sort_unstable();
    let with_kids: BTreeSet<&str> = scheme
        .classes
        .iter()
        .filter_map(|c| c.parent.as_deref())
        .collect();
    let used: BTreeSet<&str> = h.code.iter().skip(1).map(String::as_str).collect();
    let scheme_leaves: Vec<&str> = scheme
        .classes
        .iter()
        .map(|c| c.code.as_str())
        .filter(|c| !with_kids.contains(c))
        .collect();
    let empty_leaves = scheme_leaves.iter().filter(|c| !used.contains(*c)).count();
    let classes = scheme.classes.len().max(1) as f64;
    Usage {
        notation_length: placed
            .iter()
            .map(|&c| h.code[c].chars().count() as f64)
            .sum::<f64>()
            / np,
        classes_per_1000_works: holding as f64 * 1000.0 / np,
        works_per_leaf_mean: if leaves.is_empty() {
            0.0
        } else {
            leaves.iter().sum::<usize>() as f64 / leaves.len() as f64
        },
        works_per_leaf_median: if leaves.is_empty() {
            0.0
        } else {
            leaves[leaves.len() / 2] as f64
        },
        empty_class_share: 1.0 - (h.len() - 1) as f64 / classes,
        empty_leaf_share: if scheme_leaves.is_empty() {
            0.0
        } else {
            empty_leaves as f64 / scheme_leaves.len() as f64
        },
        useful_depth_share: placed.iter().filter(|&&c| h.depth[c] >= 2).count() as f64
            / h.items().max(1) as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shelfmark_core::scheme::ClassDef;

    fn class(code: &str, parent: Option<&str>) -> ClassDef {
        ClassDef {
            code: code.into(),
            label: code.into(),
            parent: parent.map(Into::into),
            range: None,
            vocab: vec![],
            note: None,
        }
    }

    fn scheme() -> SchemeDoc {
        SchemeDoc {
            schema: None,
            name: "toy".into(),
            version: "1".into(),
            title: "toy".into(),
            source: None,
            code_grammar: "[A-C]([0-9])?".into(),
            classes: vec![
                class("A", None),
                class("A1", Some("A")),
                class("A2", Some("A")),
                class("A4", Some("A")),
                class("B", None),
                class("C", None),
            ],
        }
    }

    #[test]
    fn natural_order_and_shapes() {
        assert_eq!(natural_cmp("QA9", "QA76"), Ordering::Less);
        assert_eq!(natural_cmp("QA76", "QA76.5"), Ordering::Less);
        assert_eq!(shape("QA76.5"), "A9.9");
    }

    #[test]
    fn scheme_shape_hand_checked() {
        let s = scheme_shape(&scheme());
        // All three children extend their parent code and are longer.
        assert_eq!((s.expressiveness, s.lengthening), (1.0, 1.0));
        // Depth 1: shape A x3; depth 2: A9 x3 -> fully regular.
        assert_eq!(s.regularity, 1.0);
        // Sibling pairs A1|A2, A2|A4, A|B, B|C, all ascending. A1|A2 has no
        // room (the grammar allows one digit); A3, A5 and B5 fit the others.
        assert_eq!(s.ordered_siblings, 1.0);
        assert!(
            (s.hospitality_array - 0.75).abs() < 1e-12,
            "{}",
            s.hospitality_array
        );
        assert!(s.length_depth_spearman > 0.99);
    }

    #[test]
    fn usage_hand_checked() {
        // Items: A1, A1, A2, unplaced.
        let primary = [Some("A1"), Some("A1"), Some("A2"), None].map(|c| c.map(String::from));
        let h = Hier::build(&scheme(), &primary).unwrap();
        let u = usage(&h, &scheme());
        assert!((u.notation_length - 2.0).abs() < 1e-12);
        assert!((u.classes_per_1000_works - 2000.0 / 3.0).abs() < 1e-9);
        assert_eq!((u.works_per_leaf_mean, u.works_per_leaf_median), (1.5, 2.0));
        // Used A, A1, A2 of 6 classes; leaves A1,A2,A4,B,C with A4,B,C empty.
        assert!((u.empty_class_share - 0.5).abs() < 1e-12);
        assert!((u.empty_leaf_share - 0.6).abs() < 1e-12);
        assert!((u.useful_depth_share - 0.75).abs() < 1e-12);
    }
}
