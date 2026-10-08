//! `fast-sample.json`: a small, committed slice of the full scheme for
//! tests and for a look at the data without a build. It is every seed
//! heading, its ancestors, and its descendants down to [`DEPTH`] levels,
//! with their vocabulary, as scheme `fast@2024-sample`.

use std::collections::{BTreeMap, BTreeSet};

use shelfmark_core::scheme::SchemeDoc;

/// Computer science, Machine learning, Libraries, Classification--Books,
/// Subject headings, Knowledge management, Cooking.
pub const SEEDS: &[&str] = &[
    "fst00872451",
    "fst01004795",
    "fst00997341",
    "fst00863572",
    "fst01136458",
    "fst00988184",
    "fst01754966",
];

/// Levels of descendants kept below each seed.
pub const DEPTH: usize = 2;

pub fn sample(doc: &SchemeDoc, seeds: &[&str]) -> SchemeDoc {
    let parent: BTreeMap<&str, &str> = doc
        .classes
        .iter()
        .filter_map(|c| Some((c.code.as_str(), c.parent.as_deref()?)))
        .collect();
    let mut children: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (&c, &p) in &parent {
        children.entry(p).or_default().push(c);
    }
    let mut keep: BTreeSet<&str> = BTreeSet::new();
    for &seed in seeds {
        let mut cur = Some(seed);
        while let Some(c) = cur {
            keep.insert(c);
            cur = parent.get(c).copied();
        }
        let mut level = vec![seed];
        for _ in 0..DEPTH {
            level = level
                .iter()
                .flat_map(|c| children.get(c).into_iter().flatten().copied())
                .collect();
            keep.extend(&level);
        }
    }
    let mut out = doc.clone();
    out.version = format!("{}-sample", doc.version);
    out.title = format!("{} (sample)", doc.title);
    out.classes.retain(|c| keep.contains(c.code.as_str()));
    out
}
