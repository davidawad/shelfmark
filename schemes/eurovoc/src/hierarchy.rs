//! One parent per descriptor, chosen by a fixed rule.
//!
//! EuroVoc is polyhierarchical: 244 descriptors have more than one
//! `skos:broader`, and some belong to several microthesauri. A shelfmark
//! class has exactly one parent, so:
//!
//! - A descriptor with no broader term is a top term; its parent is its
//!   microthesaurus (the lowest code, if it belongs to several).
//! - Otherwise its parent is the broader descriptor that shares a
//!   microthesaurus with it, the lowest EuroVoc id among those; if none
//!   shares one, the lowest id among all its broader descriptors.
//! - The other broader descriptors, the other microthesauri of a top term
//!   and the scope note go into the class `note`, so no link is lost.
//!
//! Siblings are ordered by label (case-insensitive), then id.

use std::collections::BTreeMap;

use crate::parse::{id_key, Term, Thesaurus};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Parent {
    Mt(String),
    Term(String),
}

pub struct Places {
    pub parent_of: BTreeMap<String, Parent>,
    children: BTreeMap<Parent, Vec<String>>,
    by_id: BTreeMap<String, usize>,
}

impl Places {
    pub fn term<'a>(&self, t: &'a Thesaurus, id: &str) -> &'a Term {
        &t.terms[self.by_id[id]]
    }
}

/// The class code of descriptor `id`: `C5263`, or the hash-style id as is.
pub fn code(id: &str) -> String {
    if id.bytes().all(|b| b.is_ascii_digit()) {
        format!("C{id}")
    } else {
        id.to_string()
    }
}

pub fn places(t: &Thesaurus) -> Result<Places, String> {
    let by_id: BTreeMap<String, usize> = t
        .terms
        .iter()
        .enumerate()
        .map(|(i, x)| (x.id.clone(), i))
        .collect();
    let mut parent_of = BTreeMap::new();
    for x in &t.terms {
        parent_of.insert(x.id.clone(), parent(t, &by_id, x)?);
    }
    let mut children: BTreeMap<Parent, Vec<String>> = BTreeMap::new();
    for (id, p) in &parent_of {
        children.entry(p.clone()).or_default().push(id.clone());
    }
    for kids in children.values_mut() {
        kids.sort_by_key(|id| {
            let x = &t.terms[by_id[id]];
            (x.label.to_lowercase(), id_key(&x.id))
        });
    }
    Ok(Places {
        parent_of,
        children,
        by_id,
    })
}

fn parent(t: &Thesaurus, by_id: &BTreeMap<String, usize>, x: &Term) -> Result<Parent, String> {
    // `broader` is ascending, so `find` and `first` give the lowest id.
    let shares = |b: &&String| {
        t.terms[by_id[b.as_str()]]
            .mts
            .iter()
            .any(|m| x.mts.contains(m))
    };
    match x.broader.iter().find(shares).or_else(|| x.broader.first()) {
        Some(b) => Ok(Parent::Term(b.clone())),
        None => x
            .mts
            .first()
            .map(|m| Parent::Mt(m.clone()))
            .ok_or_else(|| format!("{} has no microthesaurus", x.id)),
    }
}

/// Descriptor ids under `p`, depth first, siblings in label order.
pub fn walk(places: &Places, p: &Parent) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack: Vec<String> = places
        .children
        .get(p)
        .into_iter()
        .flatten()
        .rev()
        .cloned()
        .collect();
    while let Some(id) = stack.pop() {
        stack.extend(
            places
                .children
                .get(&Parent::Term(id.clone()))
                .into_iter()
                .flatten()
                .rev()
                .cloned(),
        );
        out.push(id);
    }
    out
}

/// The scope note and every link the single parent leaves out.
pub fn note(t: &Thesaurus, places: &Places, id: &str) -> Option<String> {
    let x = places.term(t, id);
    let parent = &places.parent_of[id];
    let mut parts = Vec::new();
    if let Some(s) = &x.scope {
        parts.push(format!("Scope: {s}"));
    }
    let also: Vec<String> = x
        .broader
        .iter()
        .filter(|b| !matches!(parent, Parent::Term(p) if p == *b))
        .map(|b| format!("{} ({})", places.term(t, b).label, code(b)))
        .collect();
    if !also.is_empty() {
        parts.push(format!("Also narrower than: {}", also.join(", ")));
    }
    if let Parent::Mt(home) = parent {
        let other: Vec<String> = x
            .mts
            .iter()
            .filter(|m| *m != home)
            .map(|m| {
                let label = t
                    .mts
                    .iter()
                    .find(|y| &y.code == m)
                    .map_or("", |y| y.label.as_str());
                format!("{m} {label}")
            })
            .collect();
        if !other.is_empty() {
            parts.push(format!("Also a top term of: {}", other.join(", ")));
        }
    }
    (!parts.is_empty()).then(|| parts.join(". "))
}
