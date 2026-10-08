//! The taxonomy as a shelfmark scheme: group > archive > category.
//!
//! - A group (`grp_cs`) is a top-level class, an archive (`cs`, `hep-th`,
//!   the retired `alg-geom`) a child of its group and a category (`cs.AI`)
//!   a child of its archive. A category whose id is its archive's (`hep-th`,
//!   the legacy bare `astro-ph`) is the same class as the archive.
//! - The test groups, and their archives and categories, are dropped.
//! - Notes carry the description and the history the source records:
//!   retired archives and the categories that took them over, aliases,
//!   and bare archive names old submissions used.
//! - Children follow byte order of their codes, so a class's position
//!   does not depend on the order of the source file.

use std::collections::{BTreeMap, BTreeSet};

use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source};

use crate::page::Listed;
use crate::taxonomy::{Archive, Category, Taxonomy};

/// Every class code: `grp_<name>`, an archive, or `<archive>.<subject>`.
pub const CODE_GRAMMAR: &str =
    r"grp_[a-z]+(-[a-z]+)?|[a-z]+(-[a-z]+)?(\.([A-Z]{2}|[a-z]+(-[a-z]+)?))?";

/// What the build kept and dropped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    pub groups: usize,
    pub archives: usize,
    pub retired_archives: usize,
    pub categories: usize,
    pub dropped_test: usize,
}

/// Collapse runs of whitespace (the source wraps some descriptions with
/// double spaces).
fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// End a description with a full stop so the notes after it read apart.
fn sentence(s: &str) -> String {
    if s.ends_with(['.', '!', '?']) {
        s.to_string()
    } else {
        format!("{s}.")
    }
}

fn lookup<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    pairs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

fn reverse<'a>(pairs: &'a [(String, String)], value: &str) -> Vec<&'a str> {
    pairs
        .iter()
        .filter(|(_, v)| v == value)
        .map(|(k, _)| k.as_str())
        .collect()
}

/// An `alt_name` must be explained by a subsumed archive or an alias.
fn check_alt(t: &Taxonomy, c: &Category) -> Result<(), String> {
    let Some(alt) = c.alt_name.as_deref() else {
        return Ok(());
    };
    let id = c.id.as_str();
    let pair = |m: &[(String, String)], a: &str, b: &str| lookup(m, a) == Some(b);
    if pair(&t.subsumed, alt, id)
        || pair(&t.subsumed, id, alt)
        || pair(&t.aliases, alt, id)
        || pair(&t.aliases, id, alt)
    {
        Ok(())
    } else {
        Err(format!(
            "category {id} names alt_name {alt}, which no subsumed archive or alias explains"
        ))
    }
}

fn archive_note(t: &Taxonomy, a: &Archive) -> Vec<String> {
    let mut out = Vec::new();
    if !a.is_active {
        let span = match (&a.start, &a.end) {
            (Some(s), Some(e)) => format!(", {s} to {e}"),
            _ => String::new(),
        };
        let by = lookup(&t.subsumed, &a.id)
            .map(|c| format!("; its category is now {c}"))
            .unwrap_or_default();
        out.push(format!("Retired archive{span}{by}."));
    }
    let primary = lookup(&t.legacy_primary, &a.id);
    let secondary = lookup(&t.legacy_secondary, &a.id);
    let legacy = match (primary, secondary) {
        (Some(p), Some(s)) if p == s => {
            Some(format!("as a primary or secondary category until {p}"))
        }
        (Some(p), Some(s)) => Some(format!(
            "as a primary category until {p} and as a secondary until {s}"
        )),
        (Some(p), None) => Some(format!("as a primary category until {p}")),
        (None, Some(s)) => Some(format!("as a secondary category until {s}")),
        (None, None) => None,
    };
    if let Some(when) = legacy {
        out.push(format!(
            "Older submissions used the bare name {} {when}.",
            a.id
        ));
    }
    out
}

fn category_note(t: &Taxonomy, c: &Category) -> Vec<String> {
    let mut out: Vec<String> = c.description.iter().map(|d| sentence(&squash(d))).collect();
    if let Some(canonical) = lookup(&t.aliases, &c.id) {
        out.push(format!(
            "Alias of {canonical}, the canonical category; an article in either is listed in both."
        ));
    }
    for alias in reverse(&t.aliases, &c.id) {
        out.push(format!("{alias} is an alias of this category."));
    }
    for old in reverse(&t.subsumed, &c.id) {
        out.push(format!("Took over the retired {old} archive."));
    }
    if !c.is_active && c.id != c.in_archive {
        out.push("No longer accepts submissions.".into());
    }
    out
}

fn class(code: &str, label: &str, parent: Option<&str>, note: Vec<String>) -> ClassDef {
    ClassDef {
        code: code.to_string(),
        label: squash(label),
        parent: parent.map(String::from),
        range: None,
        vocab: Vec::new(),
        note: (!note.is_empty()).then(|| note.join(" ")),
    }
}

/// The scheme's classes, in tree order.
pub fn classes(t: &Taxonomy) -> Result<(Vec<ClassDef>, Stats), String> {
    let mut stats = Stats::default();
    let groups: BTreeMap<&str, _> = t
        .groups
        .iter()
        .filter(|g| !g.is_test)
        .map(|g| (g.id.as_str(), g))
        .collect();
    let all_groups: BTreeSet<&str> = t.groups.iter().map(|g| g.id.as_str()).collect();
    let mut archives: BTreeMap<&str, Vec<&Archive>> = BTreeMap::new();
    let mut kept_archives: BTreeMap<&str, &Archive> = BTreeMap::new();
    for a in &t.archives {
        if !all_groups.contains(a.in_group.as_str()) {
            return Err(format!(
                "archive {} is in unknown group {}",
                a.id, a.in_group
            ));
        }
        if groups.contains_key(a.in_group.as_str()) {
            archives.entry(a.in_group.as_str()).or_default().push(a);
            kept_archives.insert(a.id.as_str(), a);
        }
    }
    let all_archives: BTreeSet<&str> = t.archives.iter().map(|a| a.id.as_str()).collect();
    let mut categories: BTreeMap<&str, Vec<&Category>> = BTreeMap::new();
    for c in &t.categories {
        check_alt(t, c)?;
        if !all_archives.contains(c.in_archive.as_str()) {
            return Err(format!(
                "category {} is in unknown archive {}",
                c.id, c.in_archive
            ));
        }
        if kept_archives.contains_key(c.in_archive.as_str()) {
            categories.entry(c.in_archive.as_str()).or_default().push(c);
        } else {
            stats.dropped_test += 1;
        }
    }

    let mut out = Vec::new();
    for (gid, g) in &groups {
        stats.groups += 1;
        out.push(class(gid, &g.full_name, None, Vec::new()));
        let mut arch = archives.remove(gid).unwrap_or_default();
        arch.sort_by(|a, b| a.id.cmp(&b.id));
        for a in arch {
            stats.archives += 1;
            stats.retired_archives += usize::from(!a.is_active);
            let mut cats = categories.remove(a.id.as_str()).unwrap_or_default();
            cats.sort_by(|x, y| x.id.cmp(&y.id));
            let mut note = Vec::new();
            if let Some(same) = cats.iter().find(|c| c.id == a.id) {
                stats.categories += 1;
                note.extend(category_note(t, same));
            }
            note.extend(archive_note(t, a));
            out.push(class(&a.id, &a.full_name, Some(gid), note));
            for c in cats.into_iter().filter(|c| c.id != a.id) {
                stats.categories += 1;
                out.push(class(&c.id, &c.full_name, Some(&a.id), category_note(t, c)));
            }
        }
    }
    Ok((out, stats))
}

/// Provenance the scheme records about its sources.
pub struct Provenance {
    pub version: String,
    pub description: String,
    pub license: String,
}

pub fn scheme(t: &Taxonomy, p: &Provenance) -> Result<(SchemeDoc, Stats), String> {
    let (classes, stats) = classes(t)?;
    let doc = SchemeDoc {
        schema: Some(shelfmark_core::scheme::SCHEME_SCHEMA_ID.to_string()),
        name: "arxiv".into(),
        version: p.version.clone(),
        title: "arXiv category taxonomy".into(),
        source: Some(Source {
            description: p.description.clone(),
            url: Some(crate::sources::PAGE.into()),
            license: Some(p.license.clone()),
        }),
        code_grammar: CODE_GRAMMAR.into(),
        classes,
    };
    Ok((doc, stats))
}

/// Check that every category on the taxonomy page is a class with the
/// same name under the same group, and that every active category in the
/// scheme is on the page.
pub fn cross_check(doc: &SchemeDoc, listed: &[Listed], t: &Taxonomy) -> Result<(), String> {
    let by_code: BTreeMap<&str, &ClassDef> =
        doc.classes.iter().map(|c| (c.code.as_str(), c)).collect();
    let top = |code: &str| {
        let mut at = by_code.get(code).copied();
        while let Some(c) = at {
            match c.parent.as_deref() {
                Some(p) => at = by_code.get(p).copied(),
                None => return Some(c.code.clone()),
            }
        }
        None
    };
    let mut problems = Vec::new();
    for l in listed {
        let Some(c) = by_code.get(l.code.as_str()) else {
            problems.push(format!("{} is on the page but not in the scheme", l.code));
            continue;
        };
        let name = t
            .categories
            .iter()
            .find(|x| x.id == l.code)
            .map(|x| squash(&x.full_name));
        if name.as_deref() != Some(squash(&l.name).as_str()) {
            problems.push(format!(
                "{}: page says {:?}, source {:?} (class label {:?})",
                l.code, l.name, name, c.label
            ));
        }
        if top(&l.code).as_deref() != Some(l.group.as_str()) {
            problems.push(format!("{}: page lists it under {}", l.code, l.group));
        }
        if let Some(a) = &l.archive {
            if c.code != *a && c.parent.as_deref() != Some(a.as_str()) {
                problems.push(format!("{}: page lists it under archive {a}", l.code));
            }
        }
    }
    let on_page: BTreeSet<&str> = listed.iter().map(|l| l.code.as_str()).collect();
    for c in t.categories.iter().filter(|c| c.is_active) {
        if by_code.contains_key(c.id.as_str()) && !on_page.contains(c.id.as_str()) {
            problems.push(format!("active category {} is not on the page", c.id));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "the taxonomy page disagrees with the source:\n  {}",
            problems.join("\n  ")
        ))
    }
}
