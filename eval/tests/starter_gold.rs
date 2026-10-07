//! The shipped starter gold set: size and coverage targets, provenance that
//! matches its raw snapshots, and an offline rebuild that reproduces it.

use mc_eval::gold::{stats, GoldSet};
use mc_eval::goldbuild::{build, sha256_hex};

const DIR: &str = env!("CARGO_MANIFEST_DIR");

fn starter() -> GoldSet {
    GoldSet::load(&format!("{DIR}/gold/starter-v1.ndjson")).unwrap()
}

#[test]
fn meets_size_and_coverage_targets() {
    let g = starter();
    let s = stats(&g);
    assert!(
        (90..=120).contains(&s.items),
        "about 100 items, got {}",
        s.items
    );
    assert!(
        s.main_classes.len() >= 15,
        "only {} LCC main classes",
        s.main_classes.len()
    );
    for kind in ["arxiv", "openlibrary", "wikipedia"] {
        assert!(
            s.by_source.get(kind).copied().unwrap_or(0) >= 20,
            "too few {kind} items"
        );
    }
}

#[test]
fn every_label_has_provenance_matching_its_snapshot() {
    let g = starter();
    for it in &g.items {
        let p = it
            .source
            .as_ref()
            .unwrap_or_else(|| panic!("{} has no provenance", it.id));
        assert!(
            !p.label_rule.is_empty()
                && !p.url.is_empty()
                && !p.license.is_empty()
                && !p.retrieved_at.is_empty(),
            "{}",
            it.id
        );
        let raw = std::fs::read(format!("{DIR}/gold/{}", p.raw)).unwrap();
        assert_eq!(sha256_hex(&raw), p.raw_sha256, "{} snapshot changed", it.id);
        assert!(
            !it.text.is_empty() || it.source.as_ref().unwrap().kind == "openlibrary",
            "{} has no text",
            it.id
        );
    }
}

#[test]
fn rebuild_from_snapshots_is_identical() {
    let built = build(&format!("{DIR}/gold/manifest.json")).unwrap();
    let mut text = String::new();
    for it in &built.items {
        text.push_str(&serde_json::to_string(it).unwrap());
        text.push('\n');
    }
    let shipped = std::fs::read_to_string(format!("{DIR}/gold/starter-v1.ndjson")).unwrap();
    assert!(text == shipped, "gold/starter-v1.ndjson is stale: run `mc-eval gold build --manifest gold/manifest.json --out gold/starter-v1.ndjson`");
}
