//! Accuracy check for the matcher prototype.
//!
//! Two parts, reported separately:
//!
//! - **Fixtures**: short hand-labelled passages, each with the expected LCC
//!   subclass (`QA`). Scored on main-class and subclass top-1, subclass in the
//!   top 3, and passages with no class vote at all.
//! - **Self-check**: a deterministic sample of headings that carry a direct LCC
//!   number. Each label is embedded in filler text; we check the heading is
//!   recovered and whether its own subclass wins the vote. This tests the
//!   normalisation, boundary and containment logic and how much nearby
//!   headings pull the vote, not real-world accuracy.

use serde::{Deserialize, Serialize};

use crate::lcc;
use crate::matcher::{normalize, strip_qualifiers, Matcher};

#[derive(Debug, Deserialize)]
pub struct Fixture {
    pub id: String,
    pub text: String,
    /// Expected LCC subclass letters, e.g. `QA`.
    pub expect_class: String,
}

#[derive(Debug, Default, Serialize)]
pub struct FixtureReport {
    pub cases: usize,
    pub main_class_top1: usize,
    pub subclass_top1: usize,
    pub subclass_top3: usize,
    pub no_vote: usize,
    pub results: Vec<FixtureResult>,
}

#[derive(Debug, Serialize)]
pub struct FixtureResult {
    pub id: String,
    pub expect: String,
    pub got_top3: Vec<String>,
    pub subclass_ok: bool,
    pub main_ok: bool,
    pub hits: Vec<String>,
}

pub fn run_fixtures(m: &Matcher, fixtures: &[Fixture]) -> FixtureReport {
    let mut rep = FixtureReport::default();
    for f in fixtures {
        let r = m.find(&f.text);
        let top3: Vec<String> = r
            .subclasses
            .iter()
            .take(3)
            .map(|c| c.class.clone())
            .collect();
        let subclass_ok = top3.first() == Some(&f.expect_class);
        let main_ok =
            r.main_classes.first().map(|c| c.class.as_str()) == Some(&f.expect_class[..1]);
        rep.cases += 1;
        rep.subclass_top1 += usize::from(subclass_ok);
        rep.main_class_top1 += usize::from(main_ok);
        rep.subclass_top3 += usize::from(top3.contains(&f.expect_class));
        rep.no_vote += usize::from(top3.is_empty());
        rep.results.push(FixtureResult {
            id: f.id.clone(),
            expect: f.expect_class.clone(),
            got_top3: top3,
            subclass_ok,
            main_ok,
            hits: r.hits.iter().map(|h| h.text.clone()).collect(),
        });
    }
    rep
}

#[derive(Debug, Default, Serialize)]
pub struct SelfCheckReport {
    pub sampled: usize,
    pub heading_recovered: usize,
    pub own_subclass_top1: usize,
    pub misses: Vec<String>,
}

/// FNV-1a: a stable, dependency-free sampling hash.
fn fnv(s: &str) -> u64 {
    s.bytes().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}

/// Sample up to `n` active headings with direct LCC, ordered by hash of id.
pub fn self_check(m: &Matcher, headings: &[crate::table::Heading], n: usize) -> SelfCheckReport {
    let mut pool: Vec<_> = headings
        .iter()
        .filter(|h| !h.deprecated && h.lcc_source == "direct" && h.kind == "Topic")
        .collect();
    pool.sort_by_key(|h| (fnv(&h.id), h.id.clone()));
    let mut rep = SelfCheckReport::default();
    for h in pool.into_iter().take(n) {
        let label = strip_qualifiers(&h.pref_label);
        if normalize(&label).chars().count() < 3 {
            continue;
        }
        rep.sampled += 1;
        let r = m.find(&format!("This report examines {label} in some detail."));
        let found = r
            .hits
            .iter()
            .any(|hit| hit.headings.iter().any(|x| x.id == h.id));
        rep.heading_recovered += usize::from(found);
        let own: Vec<String> = h
            .lcc
            .iter()
            .filter_map(|c| lcc::normalize(c).map(|r| r.class))
            .collect();
        let top = r.subclasses.first().map(|c| c.class.clone());
        rep.own_subclass_top1 += usize::from(top.is_some_and(|t| own.contains(&t)));
        if !found && rep.misses.len() < 25 {
            rep.misses.push(format!("{} {}", h.id, h.pref_label));
        }
    }
    rep
}
