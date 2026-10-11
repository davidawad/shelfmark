//! Inserting the 500-record Open Library holdout as new works: every
//! existing class keeps its code, label, parent and position, only leaf
//! classes are split, and the result is a valid, deterministic scheme.

mod common;

use common::*;
use learned_scheme::insert::{check_stable, CentroidFile, InsertParams, Inserter};
use learned_scheme::vectors::{read_json, VectorFile};
use std::collections::{BTreeMap, BTreeSet};

fn insert_holdout() -> Inserter {
    let centroids: CentroidFile = read_json(&here().join("data/centroids.json.gz")).unwrap();
    let vf: VectorFile = read_json(&here().join("data/holdout-vectors.json.gz")).unwrap();
    let by_id: BTreeMap<&str, _> = vf.rows.iter().map(|r| (r.id.as_str(), &r.v)).collect();
    let mut ins = Inserter::new(committed(), &centroids, InsertParams::default()).unwrap();
    for r in rows("holdout.jsonl") {
        ins.insert(&r.id, &r.title, &by_id[r.id.as_str()].unit());
    }
    ins
}

#[test]
fn centroids_cover_exactly_the_leaf_classes() {
    let doc = committed();
    let centroids: CentroidFile = read_json(&here().join("data/centroids.json.gz")).unwrap();
    let parents: BTreeSet<&str> = doc
        .classes
        .iter()
        .filter_map(|c| c.parent.as_deref())
        .collect();
    let leaves: BTreeSet<&str> = doc
        .classes
        .iter()
        .map(|c| c.code.as_str())
        .filter(|c| !parents.contains(c))
        .collect();
    let with: BTreeSet<&str> = centroids.leaves.iter().map(|c| c.code.as_str()).collect();
    assert_eq!(leaves, with);
    assert_eq!(centroids.leaves.iter().map(|c| c.n).sum::<usize>(), 10_000);
}

#[test]
fn holdout_insertion_keeps_every_existing_id_and_parent_link() {
    let before = committed();
    let ins = insert_holdout();
    let after = &ins.doc;
    check_stable(&before, after).unwrap();
    // The same guarantee spelled out: every pre-existing class is still
    // there with the same code, parent and label, in the same order.
    let now: BTreeMap<&str, _> = after.classes.iter().map(|c| (c.code.as_str(), c)).collect();
    for c in &before.classes {
        let n = now[c.code.as_str()];
        assert_eq!(
            (&n.parent, &n.label, &n.vocab),
            (&c.parent, &c.label, &c.vocab),
            "{}",
            c.code
        );
    }
    let old: Vec<&str> = before.classes.iter().map(|c| c.code.as_str()).collect();
    let kept: Vec<&str> = after
        .classes
        .iter()
        .map(|c| c.code.as_str())
        .filter(|c| old.contains(c))
        .collect();
    assert_eq!(kept, old);
    // Only leaf classes were split: each new class hangs under a class
    // that was a leaf before or was itself created by insertion.
    let old_parents: BTreeSet<&str> = before
        .classes
        .iter()
        .filter_map(|c| c.parent.as_deref())
        .collect();
    let new: Vec<_> = after
        .classes
        .iter()
        .filter(|c| !old.contains(&c.code.as_str()))
        .collect();
    assert!(
        !new.is_empty(),
        "the holdout should split at least one leaf"
    );
    for c in &new {
        let p = c.parent.as_deref().unwrap();
        assert!(
            !old_parents.contains(p),
            "{} under internal class {p}",
            c.code
        );
        assert!(c.code.starts_with(&format!("{p}.")));
    }
    assert_eq!(new.len(), 2 * ins.splits.len());
    // Every new work sits in a current leaf class.
    let parents: BTreeSet<&str> = after
        .classes
        .iter()
        .filter_map(|c| c.parent.as_deref())
        .collect();
    for r in rows("holdout.jsonl") {
        let code = ins.class_of(&r.id).unwrap();
        assert!(
            now.contains_key(code) && !parents.contains(code),
            "{} in {code}",
            r.id
        );
    }
    // A split leaf's earlier works move only to its descendant core child.
    for s in &ins.splits {
        assert!(ins.refined(&s.leaf).starts_with(&format!("{}.", s.leaf)));
    }
    assert_valid(&learned_scheme::scheme::scheme_json(after).unwrap());
}

#[test]
fn insertion_is_deterministic() {
    let a = insert_holdout();
    let b = insert_holdout();
    assert_eq!(
        serde_json::to_string(&a.doc).unwrap(),
        serde_json::to_string(&b.doc).unwrap()
    );
}
