//! The builder over synthetic pages in the udcsummary.info format
//! (invented captions; the real summary is CC BY-NC and never committed):
//! the scheme validates and compiles, and browse, search and code checks
//! answer from it.

mod common;

use shelfmark_core::scheme::{CodeCheck, CompiledIndex};
use udc_summary::build::{self, Stats};
use udc_summary::sources::{Lock, Locked};

const PAGES: [(&str, &str); 2] = [("---", "udcs-aux.html"), ("0", "udcs-0.html")];

fn lock() -> Lock {
    Lock {
        edition: "2026-10-08".into(),
        sources: PAGES
            .iter()
            .map(|(tag, file)| Locked {
                tag: tag.to_string(),
                file: file.to_string(),
                url: format!("https://udcsummary.info/php/index.php?tag={tag}&lang=en"),
                bytes: 0,
                sha256: "0".repeat(64),
            })
            .collect(),
    }
}

fn pages() -> Vec<String> {
    PAGES
        .iter()
        .map(|(_, f)| {
            std::fs::read_to_string(common::crate_dir().join("tests/fixtures").join(f)).unwrap()
        })
        .collect()
}

fn built() -> (String, Stats) {
    let (doc, stats) = build::build(&lock(), &pages()).unwrap();
    (build::scheme_json(&doc).unwrap(), stats)
}

#[test]
fn validates_and_names_its_provenance() {
    let (json, stats) = built();
    let index = common::validate_and_compile(&json);
    assert_eq!(index.scheme().to_string(), "scheme:udc-summary@2026-10-08");
    assert_eq!(index.len(), 17);
    assert_eq!(
        stats.per_page,
        vec![("---".to_string(), 8), ("0".to_string(), 9)]
    );
    assert_eq!(stats.headers, vec!["---".to_string()]);
    let doc: serde_json::Value = serde_json::from_str(&json).unwrap();
    let source = &doc["source"];
    assert_eq!(source["url"], "https://udcsummary.info/");
    assert!(source["license"].as_str().unwrap().contains("CC BY-NC 4.0"));
    assert!(source["license"]
        .as_str()
        .unwrap()
        .contains("UDC Consortium"));
    let desc = source["description"].as_str().unwrap();
    assert!(desc.contains(&lock().digest()), "{desc}");
    assert!(
        desc.contains("index.php?tag=0&lang=en sha256 0000"),
        "{desc}"
    );
}

#[test]
fn links_each_term_of_a_multi_term_caption() {
    let index = common::validate_and_compile(&built().0);
    let terms: Vec<(&str, &str)> = index.vocab()["004.738"]
        .iter()
        .map(|l| (l.term.as_str(), l.source.as_str()))
        .collect();
    assert_eq!(
        terms,
        [
            ("Fixture internetworking", "udc-caption"),
            ("Zebra's gateway", "udc-caption")
        ]
    );
    assert!(index.vocab()["004.7"].is_empty());
}

#[test]
fn is_deterministic() {
    assert_eq!(built().0, built().0);
}

#[test]
fn browses_the_hierarchy_without_the_page_header() {
    let index = common::validate_and_compile(&built().0);
    let roots: Vec<String> = index
        .children(None)
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    for top in ["=...", "(1/9)", "\"...\"", "0"] {
        assert!(roots.contains(&top.to_string()), "{top} in {roots:?}");
    }
    assert!(index.get("---").is_none());
    let kids: Vec<String> = index
        .children(Some("004"))
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    assert_eq!(kids.len(), 3, "{kids:?}");
    assert!(kids.contains(&"004.7".to_string()));
    let up: Vec<String> = index
        .ancestors("004.738")
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    assert_eq!(up, ["004.738", "004.7", "004", "0"]);
    let rec = index.get("004.738").unwrap();
    assert_eq!(rec.id.to_string(), "class:udc-summary@2026-10-08:004.738");
    assert_eq!(rec.label, "Fixture internetworking. Zebra's gateway");
    assert_eq!(rec.level, 3);
    assert_eq!(index.get("0").unwrap().label, "FIXTURE GENERALITIES");
    assert_eq!(
        index.ancestors("=...`01").len(),
        3,
        "auxiliary chains keep their depth"
    );
}

#[test]
fn searches_captions() {
    let index = common::validate_and_compile(&built().0);
    let hits = index.search("internetworking gateway", 5);
    assert_eq!(hits[0].class.code, "004.738", "{hits:?}");
    let hits = index.search("federation", 5);
    assert_eq!(hits[0].class.code, "(470+571)", "{hits:?}");
}

#[test]
fn checks_codes_against_the_grammar() {
    let index = common::validate_and_compile(&built().0);
    for code in ["004.738", "(470+571)", "=...`01", "\"19\"", "030*1", "06-0"] {
        assert!(
            matches!(index.check_code(code), CodeCheck::Exact { .. }),
            "{code}"
        );
    }
    for bad in ["QA76", "a004", ".5", "004 738", ""] {
        assert!(
            matches!(index.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
    // A compound number is grammatical even where no class names it.
    assert!(!matches!(
        index.check_code("004.738:37(4)\"19\""),
        CodeCheck::Invalid { .. }
    ));
}

#[test]
fn rejects_bad_pages() {
    let mut p = pages();
    p[1] = p[1].replace("'004.7','<div", "'004 7','<div");
    let err = build::build(&lock(), &p).unwrap_err();
    assert!(err.contains("code grammar"), "{err}");

    let mut p = pages();
    p[1] = p[1].replace("'Fixture networks','',", "'','',");
    p[1] = p[1].replace("&nbsp;&nbsp;Fixture networks", "");
    let err = build::build(&lock(), &p).unwrap_err();
    assert!(err.contains("no caption"), "{err}");

    let mut p = pages();
    p[0] = p[1].clone();
    let err = build::build(&lock(), &p).unwrap_err();
    assert!(err.contains("also on page"), "{err}");

    let err = build::build(&lock(), &pages()[..1]).unwrap_err();
    assert!(err.contains("locked sources"), "{err}");
}

#[test]
fn refuses_a_cached_page_whose_hash_changed() {
    let dir = std::env::temp_dir().join(format!("udc-summary-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("udcs-0.html"), "changed").unwrap();
    let locked = &lock().sources[1];
    let err = udc_summary::sources::read(&dir, locked).unwrap_err();
    assert!(err.contains("udcsummary.info has changed"), "{err}");
    let err = udc_summary::sources::read(&dir, &lock().sources[0]).unwrap_err();
    assert!(err.contains("build --fetch"), "{err}");
    std::fs::remove_dir_all(&dir).unwrap();
}
