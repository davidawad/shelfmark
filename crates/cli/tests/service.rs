//! The service in process: describe (R20, F1), errors (R21, AE7),
//! classify with memoization (R14) and the read verbs.

mod common;

use std::collections::BTreeSet;

use common::{call, commit, data, error, service};
use serde_json::{json, Value};
use shelfmark_core::{ErrorCode, Verb};

#[test]
fn describe_lists_every_verb_profile_noun_and_scheme() {
    let s = service("describe");
    let d = data(call(&s, Verb::Describe, json!({})));
    let verbs: BTreeSet<&str> = d["verbs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap())
        .collect();
    let all: BTreeSet<&str> = Verb::ALL.iter().map(|v| v.as_str()).collect();
    assert_eq!(verbs, all);
    for v in d["verbs"].as_array().unwrap() {
        assert!(
            v["example"].as_str().unwrap().starts_with("shelfmark "),
            "{v}"
        );
        let name: Verb = serde_json::from_value(v["name"].clone()).unwrap();
        assert_eq!(v["mutates"], json!(name.mutates()));
    }
    let profiles: Vec<&str> = d["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        profiles,
        [
            "profile:lexical@1",
            "profile:shortlist@1",
            "profile:rerank@1"
        ]
    );
    let recommended: Vec<&Value> = d["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["recommended"] == json!(true))
        .collect();
    assert_eq!(recommended.len(), 1);
    assert_eq!(d["nouns"].as_array().unwrap().len(), 6);
    assert_eq!(d["schemes"][0]["id"], "scheme:mini-lcc@2024");
    assert_eq!(
        d["error_codes"].as_array().unwrap().len(),
        ErrorCode::ALL.len()
    );
    assert!(d["example"]
        .as_str()
        .unwrap()
        .starts_with("shelfmark classify"));
}

#[test]
fn describe_topics_narrow_to_one_verb_ids_or_errors() {
    let s = service("topics");
    let c = data(call(&s, Verb::Describe, json!({"topic": "scheme-compile"})));
    assert_eq!(c["name"], "scheme-compile");
    assert_eq!(c["cli"], "shelfmark scheme compile");
    assert!(data(call(&s, Verb::Describe, json!({"topic": "ids"})))["nouns"].is_array());
    assert!(data(call(&s, Verb::Describe, json!({"topic": "errors"})))["error_codes"].is_array());
    let e = error(call(&s, Verb::Describe, json!({"topic": "nope"})));
    assert_eq!(e.code, ErrorCode::InvalidRequest);
    assert_eq!(e.next_command.as_deref(), Some("shelfmark describe"));
}

/// R21: every error carries a code, a message and a next command, and its
/// exit code follows the code.
#[test]
fn every_error_has_a_code_a_next_command_and_its_exit_code() {
    let s = service("errors");
    let cases: Vec<(Verb, Value, ErrorCode)> = vec![
        (Verb::Resolve, json!({}), ErrorCode::InvalidRequest),
        (
            Verb::Resolve,
            json!({"id": "nonsense"}),
            ErrorCode::InvalidId,
        ),
        (
            Verb::Resolve,
            json!({"id": "scheme:nope@1"}),
            ErrorCode::NotFound,
        ),
        (
            Verb::Resolve,
            json!({"id": "class:mini-lcc@2024:ZZ9"}),
            ErrorCode::NotFound,
        ),
        (
            Verb::Resolve,
            json!({"id": "profile:nope@1"}),
            ErrorCode::ProfileNotFound,
        ),
        (
            Verb::Status,
            json!({"unexpected": 1}),
            ErrorCode::InvalidRequest,
        ),
        (Verb::Status, json!([1, 2]), ErrorCode::InvalidRequest),
        (Verb::Classify, json!({}), ErrorCode::InvalidRequest),
        (
            Verb::Classify,
            json!({"text": "x", "target": 1.5}),
            ErrorCode::InvalidRequest,
        ),
        (
            Verb::Classify,
            json!({"text": "x", "scheme": "scheme:fast@2024"}),
            ErrorCode::SchemeNotCompiled,
        ),
        (
            Verb::Classify,
            json!({"text": "x", "identifiers": ["bogus:1"]}),
            ErrorCode::InvalidRequest,
        ),
        (
            Verb::Classify,
            json!({"file": "/definitely/not/here.txt"}),
            ErrorCode::InputUnreadable,
        ),
        (
            Verb::Explain,
            json!({"id": "scheme:mini-lcc@2024"}),
            ErrorCode::InvalidRequest,
        ),
        (
            Verb::Confirm,
            json!({"placement": format!("placement:s256:{}", "0".repeat(64))}),
            ErrorCode::NotFound,
        ),
        (
            Verb::SchemeCompile,
            json!({"path": "/definitely/not/here.json"}),
            ErrorCode::InputUnreadable,
        ),
        (Verb::SchemeValidate, json!({}), ErrorCode::InvalidRequest),
        (
            Verb::SchemeBrowse,
            json!({"code": "ZZ"}),
            ErrorCode::NotFound,
        ),
        (Verb::Eval, json!({}), ErrorCode::TierUnavailable),
        (
            Verb::Link,
            json!({"item": "item:s256:00", "work": "work:x"}),
            ErrorCode::InvalidId,
        ),
    ];
    for (verb, args, code) in cases {
        let env = call(&s, verb, args.clone());
        let exit = env.exit_code();
        let e = error(env);
        assert_eq!(e.code, code, "{verb:?} {args}: {e:?}");
        assert!(
            e.next_command
                .as_deref()
                .is_some_and(|c| c.starts_with("shelfmark ")),
            "{verb:?} {args}: {e:?}"
        );
        assert!(!e.message.is_empty());
        assert_eq!(exit, code.exit_code());
    }
    // Commit on a verb that does not mutate.
    let e = error(commit(&s, Verb::Status, json!({}), Some("r")));
    assert_eq!(e.code, ErrorCode::InvalidRequest);
    assert!(e.next_command.is_some());
}

#[test]
fn ae7_uncompiled_scheme_names_the_compile_command() {
    let s = service("ae7");
    let env = call(
        &s,
        Verb::SchemeBrowse,
        json!({"scheme": "scheme:fast@2024"}),
    );
    assert_eq!(env.exit_code(), 4);
    let e = error(env);
    assert_eq!(e.code, ErrorCode::SchemeNotCompiled);
    assert!(e
        .next_command
        .unwrap()
        .starts_with("shelfmark scheme compile"));
}

#[test]
fn classify_decides_flags_and_memoizes() {
    let s = service("classify");
    // The real vocab tier scores an exact heading about 0.6 uncalibrated.
    let args = json!({"title": "Engineering a Compiler", "text": "Compilers (Computer programs)", "target": 0.5});
    let first = data(call(&s, Verb::Classify, args.clone()));
    let p = &first["placement"];
    assert_eq!(first["memo_hit"], false);
    assert_eq!(p["class"], "class:mini-lcc@2024:QA76");
    assert_eq!(p["status"], "machine");
    assert_eq!(p["trace"]["deciding_tier"], "vocab");
    let again = data(call(&s, Verb::Classify, args));
    assert_eq!(again["memo_hit"], true);
    assert_eq!(again["placement"]["id"], p["id"]);

    // AE2 shape: H and K tie under the lexical profile, so it is flagged.
    let flagged = data(call(
        &s,
        Verb::Classify,
        json!({"text": "law and social sciences"}),
    ));
    assert_eq!(flagged["placement"]["status"], "flagged");
    assert_eq!(
        flagged["placement"]["trace"]["candidates"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(flagged["next_command"]
        .as_str()
        .unwrap()
        .starts_with("shelfmark explain placement:"));

    // AE1 shape: the identifier tier decides and the trace names the ISBN.
    let isbn = data(call(
        &s,
        Verb::Classify,
        json!({"identifiers": ["isbn:978-0-262-03384-8"]}),
    ));
    assert_eq!(isbn["placement"]["trace"]["deciding_tier"], "identifier");
    assert_eq!(isbn["placement"]["confidence"], 1.0);
    assert_eq!(
        isbn["placement"]["trace"]["candidates"][0]["evidence"][0]["identifier"]["value"],
        "9780262033848"
    );

    // AE3 shape: the rerank profile stops at the lexical tier.
    let rr = data(call(
        &s,
        Verb::Classify,
        json!({"text": "compilers (computer programs) again", "profile": "profile:rerank@1", "target": 0.5}),
    ));
    let states: Vec<&str> = rr["placement"]["trace"]["tiers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["state"].as_str().unwrap())
        .collect();
    assert_eq!(states, ["ran", "ran", "skipped", "skipped", "skipped"]);
    assert_eq!(rr["placement"]["trace"]["cost"]["model_calls"], 0);

    let st = data(call(&s, Verb::Status, json!({})));
    assert_eq!(st["flagged_backlog"], 1);
    assert_eq!(st["placements"]["claims"], 4);
    assert_eq!(st["recommended_profile"], "profile:lexical@1");
    let lf = data(call(&s, Verb::ListFlagged, json!({})));
    assert_eq!(lf["total"], 1);
    assert_eq!(lf["flagged"][0]["placement"], flagged["placement"]["id"]);
}

#[test]
fn resolve_explain_browse_search_validate_and_name() {
    let s = service("read");
    let c = data(call(
        &s,
        Verb::Classify,
        json!({"text": "compilers (computer programs)", "target": 0.5}),
    ));
    let pid = c["placement"]["id"].as_str().unwrap();
    let work = c["placement"]["work"].as_str().unwrap();

    let r = data(call(&s, Verb::Resolve, json!({"id": work})));
    assert_eq!(r["kind"], "work");
    assert_eq!(r["resolved"]["placements"][0]["id"], pid);
    let cls = data(call(
        &s,
        Verb::Resolve,
        json!({"id": "class:mini-lcc@2024:QA76"}),
    ));
    let anc: Vec<&str> = cls["resolved"]["ancestors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        anc,
        [
            "class:mini-lcc@2024:QA76",
            "class:mini-lcc@2024:QA",
            "class:mini-lcc@2024:Q"
        ]
    );
    assert_eq!(
        data(call(&s, Verb::Resolve, json!({"id": pid})))["resolved"]["status"],
        "machine"
    );
    assert_eq!(
        data(call(&s, Verb::Resolve, json!({"id": "profile:lexical@1"})))["resolved"]["runnable"],
        true
    );

    let ex = data(call(&s, Verb::Explain, json!({"id": work})));
    assert!(ex["explanations"][0]["summary"]
        .as_str()
        .unwrap()
        .contains("tier `vocab`"));
    assert!(ex["explanations"][0]["placement"]["trace"]["candidates"].is_array());

    let top = data(call(&s, Verb::SchemeBrowse, json!({})));
    assert_eq!(top["children"].as_array().unwrap().len(), 3);
    let qa = data(call(&s, Verb::SchemeBrowse, json!({"code": "QA"})));
    assert_eq!(qa["children"][0]["id"], "class:mini-lcc@2024:QA76");
    let hits = data(call(&s, Verb::SchemeSearch, json!({"text": "software"})));
    assert_eq!(hits["hits"][0]["class"], "class:mini-lcc@2024:QA76");
    let within = data(call(&s, Verb::SchemeValidate, json!({"code": "QA76.76"})));
    assert_eq!(within["check"]["status"], "within");

    let name = data(call(
        &s,
        Verb::Name,
        json!({"id": work, "title": "Dragon: Book", "ext": "md"}),
    ));
    assert_eq!(
        name["path"],
        "Q Science/QA Mathematics/QA76 Computer software/D73 Dragon_ Book.md"
    );
}
