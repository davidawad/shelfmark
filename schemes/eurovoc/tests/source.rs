//! The scheme built from the real EuroVoc distribution. The zip is large and
//! not bundled, so these run only when `EUROVOC_CACHE` names a directory
//! holding the locked `eurovoc_in_skos_core_concepts.zip` (or one to
//! download it into, with `EUROVOC_FETCH=1`); otherwise each test records a
//! skip in `$TEST_SKIP_LOG`, and `EUROVOC_REQUIRE_SOURCE=1` makes that a
//! failure.

mod common;

use common::{assert_valid, build, committed_lock, skip};
use shelfmark_core::scheme::CompiledIndex;

fn source() -> Option<Vec<u8>> {
    let Some(cache) = std::env::var_os("EUROVOC_CACHE").map(std::path::PathBuf::from) else {
        skip("EUROVOC_CACHE is unset, so the EuroVoc zip is not available");
        return None;
    };
    let lock = committed_lock();
    if !cache.join(&lock.source.file).exists()
        && std::env::var("EUROVOC_FETCH").as_deref() != Ok("1")
    {
        skip("EUROVOC_CACHE has no zip and EUROVOC_FETCH is not 1");
        return None;
    }
    let zip = eurovoc_scheme::source::cached(&cache, &lock).unwrap();
    Some(eurovoc_scheme::source::rdf_from_zip(&zip).unwrap())
}

fn index(rdf: &[u8]) -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(build(rdf, &committed_lock()).as_bytes(), "eurovoc.json")
        .unwrap()
}

#[test]
fn the_locked_source_builds_a_valid_scheme() {
    let Some(rdf) = source() else { return };
    let text = build(&rdf, &committed_lock());
    let doc = assert_valid(&text);
    assert_eq!(doc.id().to_string(), "scheme:eurovoc@4.24");
    // 21 domains + 127 microthesauri + 7515 descriptors.
    assert_eq!(doc.classes.len(), 7663);
    assert_eq!(text, build(&rdf, &committed_lock()));
}

#[test]
fn has_the_twenty_one_domains_and_nests_descriptors() {
    let Some(rdf) = source() else { return };
    let idx = index(&rdf);
    assert_eq!(idx.children(None).len(), 21);
    assert_eq!(idx.get("04").unwrap().label, "POLITICS");
    assert_eq!(idx.get("6011").unwrap().label, "animal product");
    assert_eq!(idx.get("C5263").unwrap().label, "carcase");
    let ancestors: Vec<String> = idx
        .ancestors("C5263")
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    assert_eq!(ancestors, ["C5263", "C2737", "6011", "60"]);
}

#[test]
fn searches_alternative_labels() {
    let Some(rdf) = source() else { return };
    let idx = index(&rdf);
    let hits = idx.search("animal carcase", 5);
    assert!(hits.iter().any(|h| h.class.code == "C5263"), "{hits:?}");
}
