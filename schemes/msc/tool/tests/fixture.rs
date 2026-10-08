//! The builder over a small hand-written sample in the official format
//! (tab-separated, quoted, CRLF, Latin-1). The labels are made up: the
//! real MSC2020 text is CC BY-NC-SA and is not stored in this repository.
//! The scheme it builds must validate, browse, search and check codes.

use std::path::PathBuf;

use msc_scheme::sources::{self, Lock};
use msc_scheme::{csv, scheme};
use shelfmark_core::id::sha256_hex;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

fn sample() -> Vec<u8> {
    std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.tsv"))
        .unwrap()
}

fn lock_for(bytes: &[u8]) -> Lock {
    let mut lock =
        Lock::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sources.lock.json"))
            .unwrap();
    lock.file = "sample.tsv".into();
    lock.bytes = bytes.len() as u64;
    lock.sha256 = sha256_hex(bytes);
    lock
}

fn built() -> (SchemeDoc, String) {
    let bytes = sample();
    let doc = scheme::build(&csv::parse(&bytes).unwrap(), &lock_for(&bytes)).unwrap();
    let json = scheme::to_json(&doc).unwrap();
    (doc, json)
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(built().1.as_bytes(), "sample").unwrap()
}

fn tempdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("msc-scheme-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn the_shipped_lock_records_the_license_and_forbids_bundling() {
    let lock = Lock::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sources.lock.json"))
        .unwrap();
    assert_eq!(lock.license, "CC BY-NC-SA 4.0");
    assert!(!lock.bundle);
    assert!(
        lock.attribution.contains("zbMATH") && lock.attribution.contains("Mathematical Reviews")
    );
    assert_eq!(lock.sha256.len(), 64);
}

#[test]
fn validates_against_the_published_schema_and_compiler_checks() {
    let (doc, json) = built();
    let checked = shelfmark_scheme::validate_bytes(json.as_bytes()).unwrap();
    assert_eq!(checked, doc);
    assert_eq!(doc.id().to_string(), "scheme:msc@2020");
    assert_eq!(doc.classes.len(), 17);
}

#[test]
fn provenance_names_url_hash_and_license() {
    let (doc, _) = built();
    let src = doc.source.unwrap();
    assert_eq!(src.url.as_deref(), Some("https://msc2020.org/MSC_2020.csv"));
    assert!(
        src.description.contains(&sha256_hex(&sample())),
        "{}",
        src.description
    );
    assert!(src.license.unwrap().starts_with("CC BY-NC-SA 4.0"));
}

#[test]
fn labels_are_verbatim_and_cross_references_become_notes() {
    let (doc, _) = built();
    let get = |c: &str| doc.classes.iter().find(|x| x.code == c).unwrap().clone();
    assert_eq!(get("11N30").label, "Turán-style estimates");
    assert_eq!(get("11S80").label, "\\(p\\)-adic analysis in number theory");
    assert_eq!(get("05C10").note.as_deref(), Some("[See also 57M15]"));
    assert_eq!(
        get("00A07").note.as_deref(),
        Some("\\{For open problems, see 00A27\\}")
    );
    assert_eq!(get("05C15").note, None);
    let other = get("05C99");
    assert_eq!(
        (other.label.as_str(), other.note),
        ("None of the above, but in this section", None)
    );
}

#[test]
fn browse_follows_the_code_shapes() {
    let ix = index();
    let roots: Vec<String> = ix
        .children(None)
        .iter()
        .map(|r| r.id.code.clone())
        .collect();
    assert_eq!(roots, ["00-XX", "05-XX", "11-XX"]);
    let kids: Vec<String> = ix
        .children(Some("05-XX"))
        .iter()
        .map(|r| r.id.code.clone())
        .collect();
    assert_eq!(kids, ["05Cxx", "05-01"], "letters shelve before digits");
    let up: Vec<String> = ix
        .ancestors("05C10")
        .iter()
        .map(|r| r.id.code.clone())
        .collect();
    assert_eq!(up, ["05C10", "05Cxx", "05-XX"]);
    assert_eq!(ix.get("05C10").unwrap().level, 2);
}

#[test]
fn search_finds_classes_by_label() {
    let ix = index();
    let hits = ix.search("graph theory", 3);
    assert_eq!(hits[0].class.code, "05Cxx");
    let hits = ix.search("p-adic analysis", 1);
    assert_eq!(hits[0].class.code, "11S80");
    assert_eq!(ix.search("turán", 1)[0].class.code, "11N30");
}

#[test]
fn check_code_places_assigned_unassigned_and_foreign_codes() {
    let ix = index();
    let code = |c: &str| match ix.check_code(c) {
        CodeCheck::Exact { class } => format!("exact {}", class.code),
        CodeCheck::Within { class } => format!("within {}", class.code),
        CodeCheck::Unplaced => "unplaced".into(),
        CodeCheck::Invalid { .. } => "invalid".into(),
    };
    assert_eq!(code("05C10"), "exact 05C10");
    assert_eq!(code("05C99"), "exact 05C99");
    assert_eq!(code("05C50"), "within 05Cxx");
    assert_eq!(code("05-99"), "within 05-XX");
    assert_eq!(code("05Z10"), "within 05-XX");
    assert_eq!(code("42A10"), "unplaced");
    assert_eq!(code("QA76"), "invalid");
}

#[test]
fn the_build_is_deterministic() {
    assert_eq!(built().1, built().1);
}

#[test]
fn bad_rows_fail_with_their_line() {
    let lock = lock_for(b"");
    let parse = |s: &str| csv::parse(s.as_bytes()).unwrap();
    let head = "code\ttext\tdescription\r\n";
    let orphan = parse(&format!(
        "{head}\"05-XX\"\t\"a\"\t\"a\"\r\n\"05C10\"\t\"b\"\t\"b\"\r\n"
    ));
    assert_eq!(
        scheme::build(&orphan, &lock).unwrap_err(),
        "line 3: `05C10` has no parent `05Cxx`"
    );
    let odd = parse(&format!("{head}\"QA76\"\t\"a\"\t\"a\"\r\n"));
    assert_eq!(
        scheme::build(&odd, &lock).unwrap_err(),
        "line 2: `QA76` is not an MSC2020 code"
    );
    let dup = parse(&format!(
        "{head}\"05-XX\"\t\"a\"\t\"a\"\r\n\"05-XX\"\t\"a\"\t\"a\"\r\n"
    ));
    assert!(scheme::build(&dup, &lock)
        .unwrap_err()
        .contains("duplicate code `05-XX`"));
}

#[test]
fn a_cached_file_must_match_the_lock() {
    let bytes = sample();
    let lock = lock_for(&bytes);
    let cache = tempdir("cache");
    std::fs::write(cache.join("sample.tsv"), &bytes).unwrap();
    assert_eq!(sources::obtain(&cache, &lock, true).unwrap(), bytes);

    std::fs::write(cache.join("sample.tsv"), b"tampered").unwrap();
    let err = sources::obtain(&cache, &lock, true).unwrap_err();
    assert!(
        err.contains("the lock pins") && err.contains("relock"),
        "{err}"
    );

    std::fs::remove_file(cache.join("sample.tsv")).unwrap();
    let err = sources::obtain(&cache, &lock, true).unwrap_err();
    assert!(err.contains("--offline"), "{err}");
    std::fs::remove_dir_all(&cache).ok();
}
