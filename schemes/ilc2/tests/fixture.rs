//! The builder over a small synthetic N-Triples file in the iskoi.org ILC2
//! format (invented labels; the real data is never committed): the scheme
//! validates and compiles, facets stay data, and browse, search and code
//! checks answer from it.

mod common;

use ilc2::build::{self, Stats};
use ilc2::sources::{Lock, Locked};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};

fn lock() -> Lock {
    Lock {
        source: Locked {
            file: "ILC2.nt".into(),
            url: "https://www.iskoi.org/ilc/2/skos/ILC2.nt".into(),
            bytes: 0,
            sha256: "0".repeat(64),
        },
    }
}

fn text() -> String {
    std::fs::read_to_string(common::crate_dir().join("tests/fixtures/mini.nt")).unwrap()
}

fn built() -> (String, Stats) {
    let (doc, stats) = build::build(&lock(), &text()).unwrap();
    (build::scheme_json(&doc).unwrap(), stats)
}

fn doc() -> serde_json::Value {
    serde_json::from_str(&built().0).unwrap()
}

fn class(code: &str) -> serde_json::Value {
    doc()["classes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == code)
        .unwrap_or_else(|| panic!("no class {code}"))
        .clone()
}

#[test]
fn validates_and_names_its_provenance() {
    let (json, stats) = built();
    let index = common::validate_and_compile(&json);
    assert_eq!(index.scheme().to_string(), "scheme:ilc2@2019-10-17");
    assert_eq!(index.len(), 13);
    assert_eq!((stats.concepts.classes, stats.facets.classes), (7, 6));
    assert_eq!(stats.relabelled, 1);
    let source = &doc()["source"];
    assert_eq!(source["url"], "https://www.iskoi.org/ilc/");
    let license = source["license"].as_str().unwrap();
    assert!(license.contains("CC BY 3.0") && license.contains("CC BY-SA 3.0"));
    let desc = source["description"].as_str().unwrap();
    assert!(desc.contains(&lock().digest()), "{desc}");
    assert!(desc.contains("ILC2.nt sha256 0000"), "{desc}");
}

#[test]
fn keeps_facet_indicators_as_classes_with_their_domain_and_range() {
    let index = common::validate_and_compile(&built().0);
    let f = class("mq94");
    assert_eq!(f["parent"], "mq9");
    assert!(f["note"]
        .as_str()
        .unwrap()
        .starts_with("facet: domain=mq range=mqa"));
    let any = class("00");
    assert_eq!(any["note"], "facet: domain=* range=*");
    assert_eq!(any["parent"], "0");
    assert_eq!(
        index.get("0").unwrap().level,
        0,
        "skos:related is no parent"
    );
    assert_eq!(
        index
            .ancestors("00")
            .into_iter()
            .map(|c| c.id.code)
            .collect::<Vec<_>>(),
        ["00", "0"]
    );
    // Concepts carry no facet line.
    assert!(class("mq")
        .get("note")
        .is_some_and(|n| !n.as_str().unwrap().contains("facet:")));
}

#[test]
fn notes_dewey_scope_and_related_and_alt_labels() {
    let mq = class("mq");
    assert_eq!(
        mq["note"],
        "scope: invented scope\nddc: 590, 591\nrelated: z"
    );
    assert_eq!(mq["vocab"][0]["term"], "fixture metazoa");
    assert_eq!(mq["vocab"][0]["source"], "ilc-alt");
    assert_eq!(class("mqb")["note"], "note: invented note");
    assert_eq!(class("mqb")["label"], "fixture \"quoted\" label");
}

#[test]
fn keeps_gaps_in_the_source_visible() {
    // A parent the file never defines: a root, with the code in the note.
    let f = class("8389");
    assert!(f.get("parent").is_none());
    assert!(f["note"]
        .as_str()
        .unwrap()
        .contains("undefined-parent: 838"));
    // An empty facet label gets a stand-in; a second prefLabel is an alt.
    assert_eq!(class("mq05")["label"], "[facet mq05 of mq]");
    assert_eq!(class("t")["label"], "fixture knowledge");
    assert_eq!(class("t")["vocab"][0]["term"], "fixture second label");
}

#[test]
fn is_deterministic() {
    assert_eq!(built().0, built().0);
}

#[test]
fn browses_the_concept_tree() {
    let index = common::validate_and_compile(&built().0);
    let kids: Vec<String> = index
        .children(Some("mq"))
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    assert_eq!(kids, ["mqa", "mqb"]);
    let up: Vec<String> = index
        .ancestors("mqa")
        .into_iter()
        .map(|c| c.id.code)
        .collect();
    assert_eq!(up, ["mqa", "mq", "m"]);
    assert_eq!(index.get("~").unwrap().label, "fixture infinity");
}

#[test]
fn reports_branching_and_depth() {
    let (_, stats) = built();
    assert_eq!(stats.concepts.depth, 3);
    assert_eq!(stats.concepts.roots, 3);
    assert_eq!(stats.concepts.max_branching, 2);
    // m has 2 children, mq has 2: mean 2.
    assert!((stats.concepts.mean_branching - 2.0).abs() < 1e-9);
    assert_eq!(stats.facets.depth, 3);
}

#[test]
fn searches_labels_and_alternatives() {
    let index = common::validate_and_compile(&built().0);
    assert_eq!(index.search("fixture whales", 5)[0].class.code, "mqa");
    assert_eq!(index.search("metazoa", 5)[0].class.code, "mq");
}

#[test]
fn checks_codes_against_the_grammar() {
    let index = common::validate_and_compile(&built().0);
    for code in ["mq", "mq94", "~", "m"] {
        assert!(
            matches!(index.check_code(code), CodeCheck::Exact { .. }),
            "{code}"
        );
    }
    for bad in ["m q", "", "m-q", "m.q"] {
        assert!(
            matches!(index.check_code(bad), CodeCheck::Invalid { .. }),
            "{bad}"
        );
    }
}

#[test]
fn rejects_bad_input() {
    let no_date: String = text()
        .lines()
        .filter(|l| !l.contains("terms/issued"))
        .map(|l| format!("{l}\n"))
        .collect();
    assert!(build::build(&lock(), &no_date)
        .unwrap_err()
        .contains("dct:issued"));

    let dup = text().replace("\"mqa\"^^", "\"mq\"^^");
    let err = build::build(&lock(), &dup).unwrap_err();
    assert!(err.contains("does not end in its notation"), "{err}");

    let two_parents = format!(
        "{}<http://www.iskoi.org/ilc/2/class/mqa> <http://www.w3.org/2004/02/skos/core#broader> \
         <http://www.iskoi.org/ilc/2/class/z> .\n",
        text()
    );
    let err = build::build(&lock(), &two_parents).unwrap_err();
    assert!(err.contains("at most 1"), "{err}");

    let cycle = format!(
        "{}<http://www.iskoi.org/ilc/2/class/m> <http://www.w3.org/2004/02/skos/core#broader> \
         <http://www.iskoi.org/ilc/2/class/mqa> .\n",
        text()
    );
    let err = build::build(&lock(), &cycle).unwrap_err();
    assert!(err.contains("cycle"), "{err}");

    let err = build::build(&lock(), "<a> <b> junk .\n").unwrap_err();
    assert!(err.contains("line 1"), "{err}");
}

#[test]
fn refuses_a_cached_file_whose_hash_changed() {
    let dir = std::env::temp_dir().join(format!("ilc2-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let locked = lock().source;
    let err = ilc2::sources::read(&dir, &locked).unwrap_err();
    assert!(err.contains("build --fetch"), "{err}");
    std::fs::write(dir.join("ILC2.nt"), "changed").unwrap();
    let err = ilc2::sources::read(&dir, &locked).unwrap_err();
    assert!(err.contains("iskoi.org has changed"), "{err}");
    std::fs::remove_dir_all(&dir).unwrap();
}
