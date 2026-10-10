//! `eval` (R16, R17, PD3): gold from corrections carries the classify
//! input, eval previews metrics and a fitted calibration, commit writes
//! both and `status` reports them, and a gold file imports once.

mod common;

use common::{call, commit, data, error, service, temp_dir};
use serde_json::{json, Value};
use shelfmark_cli::Shelfmark;
use shelfmark_core::{ErrorCode, Verb};

/// Classify `args` and correct (or confirm) the claim to `class`, committed.
fn label(s: &Shelfmark, args: Value, class: Option<&str>) {
    let pid = data(call(s, Verb::Classify, args))["placement"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let (verb, args) = match class {
        Some(c) => (Verb::Correct, json!({"placement": pid, "class": c})),
        None => (Verb::Confirm, json!({"placement": pid})),
    };
    let rev = data(call(s, verb, args.clone()))["revision"]
        .as_str()
        .unwrap()
        .to_string();
    data(commit(s, verb, args, Some(&rev)));
}

#[test]
fn corrections_feed_eval_and_a_commit_sets_calibration() {
    let s = service("eval");
    let e = error(call(&s, Verb::Eval, json!({})));
    assert_eq!(e.code, ErrorCode::NotFound, "no gold yet");
    assert_eq!(
        e.next_command.as_deref(),
        Some("shelfmark eval --gold <gold.ndjson>")
    );

    label(&s, json!({"text": "law and social sciences"}), Some("K"));
    label(
        &s,
        json!({"title": "Engineering a Compiler", "text": "Compilers (Computer programs)"}),
        None,
    );

    let preview = data(call(&s, Verb::Eval, json!({})));
    assert_eq!(preview["mode"], "preview");
    let change = &preview["change"];
    // Both labels kept their text, so both are re-run.
    assert_eq!(change["gold"]["evaluated"], 2);
    assert_eq!(change["gold"]["unusable"]["no_input"], 0);
    let lexical = &change["eval"]["per_profile"]["profile:lexical@2"];
    assert_eq!(lexical["n"], 2);
    assert_eq!(lexical["k"], 5);
    // Only built-in profiles are evaluated; shortlist only with its feature.
    let shortlist = cfg!(feature = "semantic-shortlist");
    assert!(change["eval"]["per_tier"]
        .as_object()
        .unwrap()
        .keys()
        .all(|k| k.starts_with("profile:lexical@2/")
            || (shortlist && k.starts_with("profile:shortlist@2/"))));
    assert_eq!(change["eval"]["per_level"]["profile:lexical@2/1"]["n"], 2);
    assert!(change["calibration"]["tiers"]["vocab"]["bins"]
        .as_array()
        .is_some_and(|b| b.len() == 10));
    assert_eq!(change["calibration"]["gold_count"], 2);
    // Profiles whose tiers are not built in (rerank always, shortlist
    // without its feature) are reported as skipped rather than failing.
    let skipped = if shortlist { 1 } else { 2 };
    assert_eq!(
        change["skipped_profiles"].as_array().unwrap().len(),
        skipped
    );
    let st = data(call(&s, Verb::Status, json!({})));
    assert!(
        st["eval"].is_null() && st["calibration"].is_null(),
        "preview wrote nothing"
    );

    let rev = preview["revision"].as_str().unwrap();
    let done = data(commit(&s, Verb::Eval, json!({}), Some(rev)));
    assert_eq!(done["mode"], "commit");
    let st = data(call(&s, Verb::Status, json!({})));
    assert_eq!(st["calibration"]["stale"], false);
    assert_eq!(st["calibration"]["gold_count"], 2);
    assert_eq!(st["eval"]["gold_hash"], st["gold"]["hash"]);
    assert_eq!(st["eval"]["per_profile"]["profile:lexical@2"]["n"], 2);

    // The calibration now feeds the config hash, so earlier claims do not
    // memo-hit; a new correction makes it stale (R17).
    let again = data(call(
        &s,
        Verb::Classify,
        json!({"text": "law and social sciences"}),
    ));
    assert_eq!(again["memo_hit"], false);
    label(
        &s,
        json!({"text": "computer programs for science"}),
        Some("Q"),
    );
    let st = data(call(&s, Verb::Status, json!({})));
    assert_eq!(st["calibration"]["stale"], true);
}

#[test]
fn eval_errors_name_a_next_command() {
    let s = service("eval-errors");
    label(&s, json!({"text": "law and social sciences"}), Some("K"));
    let e = error(call(&s, Verb::Eval, json!({"profile": "profile:rerank@2"})));
    assert_eq!(e.code, ErrorCode::TierUnavailable);
    let missing = if cfg!(feature = "semantic-shortlist") {
        "rerank"
    } else {
        "shortlist"
    };
    assert!(e.message.contains(missing), "{}", e.message);
    let e = error(call(
        &s,
        Verb::Eval,
        json!({"gold": "/no/such/gold.ndjson"}),
    ));
    assert_eq!(e.code, ErrorCode::InputUnreadable);
    let e = error(commit(&s, Verb::Eval, json!({}), None));
    assert_eq!(e.code, ErrorCode::RevisionRequired);
}

#[test]
fn a_gold_file_imports_once_and_skips_classes_outside_the_scheme() {
    let s = service("eval-gold");
    let dir = temp_dir("eval-gold-file");
    let path = dir.join("gold.ndjson");
    let items = [
        json!({"id": "gold:a", "title": "Engineering a Compiler",
               "text": "Compilers (Computer programs)",
               "identifiers": {"isbn10": "0262033844", "openlibrary": "OL1M"},
               "acceptable": [{"code": "QA76.73"}, {"code": "QA76.9"}],
               "source": {"kind": "openlibrary", "record": "OL1M",
                          "retrieved_at": "2026-10-07T00:00:00Z"}}),
        json!({"id": "gold:b", "title": "Phonetics", "text": "speech sounds",
               "acceptable": [{"code": "P221"}]}),
    ];
    let text: String = items.iter().map(|i| format!("{i}\n")).collect();
    std::fs::write(&path, text).unwrap();
    let args = json!({"gold": path.to_str().unwrap()});

    let preview = data(call(&s, Verb::Eval, args.clone()));
    let gold = &preview["change"]["gold"];
    // Both codes fall within QA76, so gold:a is one example.
    assert_eq!(gold["imported"], 1);
    assert_eq!(gold["evaluated"], 1);
    assert_eq!(gold["skipped"][0]["id"], "gold:b");
    let per = &preview["change"]["eval"]["per_profile"]["profile:lexical@2"];
    assert_eq!(per["top1"], 1.0, "the vocab tier finds QA76: {per}");

    let rev = preview["revision"].as_str().unwrap();
    data(commit(&s, Verb::Eval, args.clone(), Some(rev)));
    let st = data(call(&s, Verb::Status, json!({})));
    assert_eq!(st["gold"]["count"], 1);
    assert_eq!(st["calibration"]["stale"], false);

    let again = data(call(&s, Verb::Eval, args));
    assert_eq!(again["change"]["gold"]["imported"], 0);
    assert_eq!(again["change"]["gold"]["duplicates"], 1);
    assert_eq!(again["revision"], st["revision"]);
}
