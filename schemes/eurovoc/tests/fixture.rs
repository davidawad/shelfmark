//! The builder on a synthetic source in EuroVoc's SKOS-core RDF/XML shape:
//! the output is a valid scheme, deterministic, browsable and searchable,
//! polyhierarchy follows the documented rule, and the lock guards the
//! zip. Needs no network and no EuroVoc data.

mod common;

use std::io::Write;

use common::{assert_valid, build, committed_lock, here};
use eurovoc_scheme::source::Lock;
use shelfmark_core::scheme::{ClassDef, CompiledIndex};

fn rdf() -> Vec<u8> {
    std::fs::read(here().join("fixtures/eurovoc_in_skos_core_concepts.rdf")).unwrap()
}

fn lock() -> Lock {
    committed_lock()
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(build(&rdf(), &lock()).as_bytes(), "eurovoc.json").unwrap()
}

fn class(text: &str, code: &str) -> ClassDef {
    assert_valid(text)
        .classes
        .into_iter()
        .find(|c| c.code == code)
        .unwrap_or_else(|| panic!("no class {code}"))
}

#[test]
fn builds_a_valid_scheme_with_provenance() {
    let text = build(&rdf(), &lock());
    let doc = assert_valid(&text);
    assert_eq!(doc.id().to_string(), "scheme:eurovoc@4.24");
    assert_eq!(doc.classes.len(), 11);
    let src = doc.source.unwrap();
    assert!(src.description.contains(&lock().source.sha256));
    assert_eq!(src.url.as_deref(), Some(eurovoc_scheme::source::PAGE));
    let license = src.license.unwrap();
    assert!(license.contains("Publications Office") && license.contains("Not bundled"));
}

#[test]
fn rebuilds_byte_for_byte() {
    assert_eq!(build(&rdf(), &lock()), build(&rdf(), &lock()));
}

#[test]
fn domains_microthesauri_and_descriptors_nest() {
    let idx = index();
    let codes = |v: Vec<shelfmark_core::scheme::ClassRecord>| -> Vec<String> {
        v.into_iter().map(|c| c.id.code).collect()
    };
    assert_eq!(codes(idx.children(None)), ["04", "16"]);
    assert_eq!(codes(idx.children(Some("04"))), ["0406", "0411"]);
    assert_eq!(idx.get("04").unwrap().label, "POLITICS");
    assert_eq!(idx.get("0406").unwrap().label, "political framework");
    assert_eq!(codes(idx.children(Some("0406"))), ["C1"]);
    assert_eq!(codes(idx.children(Some("C1"))), ["C2", "C3"]);
    assert_eq!(codes(idx.ancestors("C3")), ["C3", "C1", "0406", "04"]);
    assert_eq!(idx.get("C3").unwrap().level, 3);
}

#[test]
fn polyhierarchy_picks_one_parent_and_keeps_the_rest_in_the_note() {
    let text = build(&rdf(), &lock());
    // Broader C1 and C2 both share the microthesaurus: the lowest id wins.
    let c3 = class(&text, "C3");
    assert_eq!(c3.parent.as_deref(), Some("C1"));
    assert_eq!(
        c3.note.as_deref(),
        Some("Scope: Seat allocation in proportion to votes. Also narrower than: electoral system (C2)")
    );
    // Broader C3 (other microthesaurus) and C4 (shared): the sharing one wins.
    let c5 = class(&text, "C5");
    assert_eq!(c5.parent.as_deref(), Some("C4"));
    assert!(c5
        .note
        .unwrap()
        .contains("proportional representation (C3)"));
    // A top term in two microthesauri sits under the lowest code.
    let c4 = class(&text, "C4");
    assert_eq!(c4.parent.as_deref(), Some("0411"));
    assert_eq!(
        c4.note.as_deref(),
        Some("Also a top term of: 1606 economic policy")
    );
}

#[test]
fn reads_english_labels_only_and_alt_labels_as_vocabulary() {
    let text = build(&rdf(), &lock());
    let c4 = class(&text, "C4");
    assert_eq!(c4.label, "border region");
    let terms: Vec<_> = c4.vocab.iter().map(|v| v.term.as_str()).collect();
    assert_eq!(terms, ["frontier area"]);
    assert_eq!(c4.vocab[0].source, "eurovoc");
    assert_eq!(c4.vocab[0].source_id.as_deref(), Some("4"));
    // The hash-style id is kept as its own code; an alt label equal to the
    // preferred label is dropped.
    let hashed = class(&text, "c_deadbeef");
    assert_eq!(hashed.parent.as_deref(), Some("C4"));
    assert_eq!(hashed.vocab.len(), 1);
    assert_eq!(hashed.vocab[0].term, "aid for the elderly");
    assert!(!text.contains("(fr)") && !text.contains("Fremdwort"));
}

#[test]
fn searches_labels_and_alt_labels() {
    let idx = index();
    let first = |q: &str| idx.search(q, 3).first().map(|h| h.class.code.clone());
    assert_eq!(first("proportional representation").as_deref(), Some("C3"));
    assert_eq!(first("frontier area").as_deref(), Some("C4"));
    assert_eq!(first("voting paper").as_deref(), Some("C1"));
}

#[test]
fn rejects_a_broader_cycle() {
    let cyc = String::from_utf8(rdf()).unwrap().replace(
        "<notation xmlns=\"http://www.w3.org/2004/02/skos/core#\">1</notation>",
        "<broader xmlns=\"http://www.w3.org/2004/02/skos/core#\" rdf:resource=\"http://eurovoc.europa.eu/2\"/>\n\t<notation xmlns=\"http://www.w3.org/2004/02/skos/core#\">1</notation>",
    );
    let err = eurovoc_scheme::scheme(cyc.as_bytes(), &lock()).unwrap_err();
    assert!(
        err.contains("unreachable") || err.contains("cycle"),
        "{err}"
    );
}

#[test]
fn the_lock_refuses_a_changed_source() {
    let relocked = lock().relocked(b"zip bytes", "2026-10-09".into());
    relocked.verify(b"zip bytes").unwrap();
    let err = relocked.verify(b"changed").unwrap_err();
    assert!(err.contains("--relock"), "{err}");
}

#[test]
fn extracts_the_rdf_from_the_zip() {
    let zip_of = |files: &[(&str, &[u8])]| {
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, body) in files {
            w.start_file(*name, opts).unwrap();
            w.write_all(body).unwrap();
        }
        w.finish().unwrap().into_inner()
    };
    let one = zip_of(&[("eurovoc_in_skos_core_concepts.rdf", &rdf())]);
    assert_eq!(eurovoc_scheme::source::rdf_from_zip(&one).unwrap(), rdf());
    let none = zip_of(&[("readme.txt", b"x")]);
    assert!(eurovoc_scheme::source::rdf_from_zip(&none).is_err());
    let two = zip_of(&[("a.rdf", b"x"), ("b.rdf", b"y")]);
    assert!(eurovoc_scheme::source::rdf_from_zip(&two).is_err());
}

#[test]
fn build_fetches_nothing_when_the_cache_matches() {
    let dir = std::env::temp_dir().join(format!("eurovoc-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(&lock().source.file);
    let lock = lock().relocked(b"cached", "2026-10-09".into());
    std::fs::write(&file, b"cached").unwrap();
    assert_eq!(
        eurovoc_scheme::source::cached(&dir, &lock).unwrap(),
        b"cached"
    );
    std::fs::write(&file, b"tampered").unwrap();
    assert!(eurovoc_scheme::source::cached(&dir, &lock).is_err());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_committed_lock_records_the_license_check() {
    let lock = committed_lock();
    assert!(!lock.license.redistributable);
    assert!(lock.license.holder.contains("Publications Office"));
    assert_eq!(lock.source.url, eurovoc_scheme::source::SOURCE_URL);
    assert_eq!(lock.source.sha256.len(), 64);
    assert!(lock.source.sha256.chars().all(|c| c.is_ascii_hexdigit()));
    let ignore = std::fs::read_to_string(here().join(".gitignore")).unwrap();
    assert!(ignore.lines().any(|l| l == "/eurovoc.json"));
}
