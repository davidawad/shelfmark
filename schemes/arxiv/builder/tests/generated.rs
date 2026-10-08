//! The committed `schemes/arxiv/arxiv.json` is a valid scheme that the
//! builder reproduces byte for byte from the committed sources, and it
//! answers the browse, search and code lookups the classifier relies on.

use std::path::PathBuf;

use arxiv_scheme::page::Listed;
use arxiv_scheme::{build, generate, taxonomy};
use serde_json::Value;
use shelfmark_core::scheme::{CodeCheck, CompiledIndex, SchemeDoc};

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn bytes() -> Vec<u8> {
    std::fs::read(dir().join("arxiv.json")).unwrap()
}

fn json(path: PathBuf) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn index() -> shelfmark_scheme::SchemeIndex {
    shelfmark_scheme::compile_bytes(&bytes(), "arxiv.json").unwrap()
}

fn codes(records: Vec<shelfmark_core::scheme::ClassRecord>) -> Vec<String> {
    records.into_iter().map(|c| c.id.code).collect()
}

#[test]
fn rebuilds_byte_for_byte_from_the_committed_sources() {
    let g = generate(&dir(), None).unwrap();
    assert_eq!(
        g.json,
        String::from_utf8(bytes()).unwrap(),
        "run `arxiv-scheme build`"
    );
    assert_eq!(g.stats.groups, 8);
    assert_eq!(g.stats.dropped_test, 9);
}

#[test]
fn validates_against_the_published_schema() {
    let schema = json(dir().join("../../schemas/scheme.schema.json"));
    let v = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = v
        .iter_errors(&json(dir().join("arxiv.json")))
        .take(10)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn round_trips_through_the_model_and_passes_the_compilers_checks() {
    let doc = json(dir().join("arxiv.json"));
    let model: SchemeDoc = serde_json::from_value(doc.clone()).unwrap();
    assert_eq!(model.id().to_string(), "scheme:arxiv@2024-04-01");
    assert_eq!(serde_json::to_value(&model).unwrap(), doc);
    if let Err(problems) = shelfmark_scheme::validate_bytes(&bytes()) {
        panic!(
            "{} problems: {:?}",
            problems.len(),
            &problems[..problems.len().min(10)]
        );
    }
}

#[test]
fn records_provenance_and_license() {
    let doc = json(dir().join("arxiv.json"));
    let source = &doc["source"];
    assert_eq!(source["url"], "https://arxiv.org/category_taxonomy");
    let desc = source["description"].as_str().unwrap();
    let lock = json(dir().join("sources.lock.json"));
    for s in lock["sources"].as_array().unwrap() {
        if s["role"] != "license" {
            assert!(desc.contains(s["sha256"].as_str().unwrap()), "{s}");
        }
    }
    let license = source["license"].as_str().unwrap();
    assert!(
        license.starts_with("MIT. Copyright (c) 2026 arXiv, Inc.;"),
        "{license}"
    );
    assert!(dir().join("sources/LICENSE").is_file());
}

#[test]
fn browses_group_archive_category() {
    let idx = index();
    assert_eq!(
        codes(idx.children(None)),
        [
            "grp_cs",
            "grp_econ",
            "grp_eess",
            "grp_math",
            "grp_physics",
            "grp_q-bio",
            "grp_q-fin",
            "grp_stat"
        ]
    );
    assert_eq!(codes(idx.children(Some("grp_cs"))), ["cmp-lg", "cs"]);
    let cs = codes(idx.children(Some("cs")));
    assert_eq!(cs.len(), 40);
    assert_eq!((cs[0].as_str(), cs[39].as_str()), ("cs.AI", "cs.SY"));
    assert_eq!(codes(idx.ancestors("cs.AI")), ["cs.AI", "cs", "grp_cs"]);
    assert_eq!(idx.get("cs.AI").unwrap().level, 2);
    assert_eq!(
        idx.get("cond-mat.mtrl-sci").unwrap().label,
        "Materials Science"
    );
    // A single-category archive is one class, directly under its group.
    assert_eq!(codes(idx.ancestors("hep-th")), ["hep-th", "grp_physics"]);
    assert!(idx.children(Some("hep-th")).is_empty());
    // Retired archives stay, so old identifiers (acc-phys/9411001) resolve.
    assert_eq!(
        idx.get("acc-phys").unwrap().parent.unwrap().code,
        "grp_physics"
    );
    assert!(idx.get("test.soft").is_none() && idx.get("grp_test").is_none());
}

#[test]
fn every_active_category_on_arxiv_is_a_class() {
    let idx = index();
    for code in [
        "astro-ph.CO",
        "cond-mat.str-el",
        "gr-qc",
        "math-ph",
        "math.MP",
        "nlin.SI",
        "physics.optics",
        "q-bio.NC",
        "q-fin.EC",
        "stat.ML",
        "eess.SY",
        "econ.EM",
        "quant-ph",
    ] {
        assert!(
            matches!(idx.check_code(code), CodeCheck::Exact { .. }),
            "{code}"
        );
    }
}

#[test]
fn searches_labels() {
    let idx = index();
    let hits = idx.search("machine learning", 5);
    // Two exact labels tie; ties break by code.
    assert_eq!(hits[0].class.code, "cs.LG");
    assert_eq!(hits[1].class.code, "stat.ML");
    assert_eq!(hits[0].matched, "Machine Learning");
    assert_eq!(
        idx.search("superconductivity", 1)[0].class.code,
        "cond-mat.supr-con"
    );
    assert!(idx.search("zzzz", 5).is_empty());
}

#[test]
fn checks_codes_against_the_grammar() {
    let idx = index();
    match idx.check_code("cs.ZZ") {
        CodeCheck::Within { class } => assert_eq!(class.code, "cs"),
        other => panic!("{other:?}"),
    }
    for bad in ["CS.AI", "cs.ai.x", "cs AI", "grp_", "cs.A"] {
        assert!(
            matches!(idx.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
}

/// The taxonomy page as the build would list it from the scheme itself.
fn listed(doc: &SchemeDoc, t: &taxonomy::Taxonomy) -> Vec<Listed> {
    let parent = |code: &str| {
        doc.classes
            .iter()
            .find(|c| c.code == code)
            .unwrap()
            .parent
            .clone()
    };
    t.categories
        .iter()
        .filter(|c| c.is_active)
        .map(|c| {
            let up = parent(&c.id).unwrap();
            let group = parent(&up).unwrap_or(up.clone());
            Listed {
                group,
                archive: None,
                code: c.id.clone(),
                name: c.full_name.clone(),
            }
        })
        .collect()
}

#[test]
fn cross_check_catches_a_page_that_disagrees() {
    let src = std::fs::read_to_string(dir().join("sources/definitions.py.txt")).unwrap();
    let t = taxonomy::parse(&src).unwrap();
    let doc = generate(&dir(), None).unwrap().doc;
    let page = listed(&doc, &t);
    build::cross_check(&doc, &page, &t).unwrap();

    let mut renamed = page.clone();
    renamed[0].name = "Artificial Stupidity".into();
    assert!(build::cross_check(&doc, &renamed, &t)
        .unwrap_err()
        .contains("Artificial Stupidity"));

    let mut moved = page.clone();
    moved[0].group = "grp_math".into();
    assert!(build::cross_check(&doc, &moved, &t).is_err());

    let missing = page[1..].to_vec();
    assert!(build::cross_check(&doc, &missing, &t)
        .unwrap_err()
        .contains("not on the page"));

    let mut extra = page;
    extra.push(Listed {
        group: "grp_cs".into(),
        archive: None,
        code: "cs.QQ".into(),
        name: "New".into(),
    });
    assert!(build::cross_check(&doc, &extra, &t)
        .unwrap_err()
        .contains("cs.QQ"));
}
