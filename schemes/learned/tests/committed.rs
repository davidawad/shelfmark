//! The committed scheme: valid, fully hand-labelled, rebuilt byte for byte
//! from the committed corpus, graph and labels, and scored by this crate's
//! objective exactly as the benchmark harness scores it.

mod common;

use common::*;
use learned_scheme::scheme::{
    describe, read_labels, scheme_doc, scheme_json, source_description, Meta,
};
use learned_scheme::{fit, FitConfig, Graph};
use shelfmark_bench::{induce_tree, score_tree, ItemPlacement};
use std::collections::BTreeSet;

#[test]
fn committed_scheme_is_valid_and_every_label_is_hand_written() {
    let doc = assert_valid(&committed_text());
    assert_eq!(doc.name, "learned");
    assert!(doc
        .source
        .as_ref()
        .unwrap()
        .license
        .as_deref()
        .unwrap()
        .starts_with("CC0 1.0"));
    let labels = read_labels(&std::fs::read_to_string(here().join("labels.tsv")).unwrap()).unwrap();
    assert_eq!(labels.len(), doc.classes.len());
    for c in &doc.classes {
        assert_eq!(labels[&c.code].0, c.label, "{}", c.code);
        assert!(!c.vocab.is_empty() && c.vocab.iter().all(|v| v.source == "ctfidf"));
    }
    let parents: BTreeSet<&str> = doc
        .classes
        .iter()
        .filter_map(|c| c.parent.as_deref())
        .collect();
    for p in &parents {
        let kids = doc
            .classes
            .iter()
            .filter(|c| c.parent.as_deref() == Some(p))
            .count();
        assert!((2..=12).contains(&kids), "{p} has {kids} children");
    }
    assert!(doc.classes.iter().all(|c| c.code.split('.').count() <= 4));
}

#[test]
fn rebuild_reproduces_the_committed_scheme_and_matches_the_harness() {
    let rows = rows("corpus.jsonl");
    let sim = graph();
    let g = Graph::from_similarity(&sim).unwrap();
    let cfg = FitConfig::default();
    let fitted = fit(&g, &cfg).unwrap();
    let infos = describe(&fitted.tree, &g, &rows);
    let labels = read_labels(&std::fs::read_to_string(here().join("labels.tsv")).unwrap()).unwrap();
    let corpus_sha = shelfmark_bench::sha256_hex(
        &std::fs::read(root().join("bench/corpus/corpus.jsonl")).unwrap(),
    );
    let meta = Meta {
        version: "2026-10-10".into(),
        description: source_description(rows.len(), &corpus_sha, &graph_sha(), None, &cfg),
    };
    let doc = scheme_doc(&infos, Some(&labels), false, &meta).unwrap();
    assert_eq!(scheme_json(&doc).unwrap(), committed_text());

    // The harness, given the scheme and its own placements, computes the
    // same Dasgupta and navigation terms the builder optimized.
    let leaf_of = fitted.tree.item_leaf();
    let code_of: std::collections::BTreeMap<usize, &str> =
        infos.iter().map(|c| (c.node, c.code.as_str())).collect();
    let placements: Vec<ItemPlacement> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| ItemPlacement {
            id: r.id.clone(),
            classes: vec![(code_of[&leaf_of[i]].to_owned(), 1.0)],
        })
        .collect();
    let score = score_tree(&induce_tree(&doc, &placements).unwrap(), &sim);
    let ours = fitted.tree.evaluate(&g, &cfg.params);
    assert!(
        (score.cost_norm - ours.dasgupta).abs() < 1e-12,
        "{} vs {}",
        score.cost_norm,
        ours.dasgupta
    );
    assert!(
        (score.structure.nav_log_a1 - ours.nav).abs() < 1e-9,
        "{} vs {}",
        score.structure.nav_log_a1,
        ours.nav
    );
    // Stage log: average linkage is the harness's reference tree.
    assert!((fitted.stages[0].dasgupta - 0.15183392202502477).abs() < 1e-12);
    let last = fitted.stages.last().unwrap();
    assert!(last.total.unwrap() <= fitted.stages[2].total.unwrap());
}

#[test]
fn sources_lock_and_registry_match_the_committed_files() {
    let lock: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(here().join("sources.lock.json")).unwrap())
            .unwrap();
    for s in lock["sources"].as_array().unwrap() {
        let bytes = std::fs::read(root().join(s["file"].as_str().unwrap())).unwrap();
        assert_eq!(s["bytes"], bytes.len(), "{}", s["file"]);
        assert_eq!(
            s["sha256"],
            shelfmark_bench::sha256_hex(&bytes),
            "{}",
            s["file"]
        );
    }
    let registry: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("schemes/registry.json")).unwrap(),
    )
    .unwrap();
    let entry = registry["schemes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == "learned")
        .unwrap();
    let doc = committed();
    assert_eq!(entry["version"], doc.version.as_str());
    assert_eq!(entry["title"], doc.title.as_str());
    assert_eq!(entry["file"], "learned/learned.json");
    assert_eq!(entry["license"], "CC0 1.0");
}
