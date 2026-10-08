//! Previewable mutations (PD8): preview by default, commit only at the
//! expected revision; confirm/correct supersede and add gold (R12, R15);
//! scheme compile, stale (R14, AE6) and link.

mod common;

use common::{call, commit, data, error, service, temp_dir};
use serde_json::json;
use shelfmark_core::{ErrorCode, Verb};

#[test]
fn preview_then_commit_and_a_stale_revision_conflicts() {
    let s = service("revision");
    let c = data(call(
        &s,
        Verb::Classify,
        json!({"text": "law and social sciences"}),
    ));
    let pid = c["placement"]["id"].as_str().unwrap().to_string();
    let args = json!({"placement": pid, "class": "K", "author": "david"});

    let preview = data(call(&s, Verb::Correct, args.clone()));
    assert_eq!(preview["mode"], "preview");
    let rev = preview["revision"].as_str().unwrap().to_string();
    assert_eq!(
        preview["change"]["append"]["class"],
        "class:mini-lcc@2024:K"
    );
    assert_eq!(preview["change"]["supersedes"], pid.as_str());
    assert!(preview["commit_command"]
        .as_str()
        .unwrap()
        .ends_with(&format!("--class K --commit --expected-revision {rev}")));
    // Previewing changed nothing.
    assert_eq!(
        data(call(&s, Verb::Status, json!({})))["revision"],
        rev.as_str()
    );

    let e = error(commit(&s, Verb::Correct, args.clone(), None));
    assert_eq!(e.code, ErrorCode::RevisionRequired);
    assert!(e
        .next_command
        .unwrap()
        .starts_with("shelfmark correct placement:"));

    let done = data(commit(&s, Verb::Correct, args.clone(), Some(&rev)));
    assert_eq!(done["mode"], "commit");
    assert_eq!(done["previous_revision"], rev.as_str());
    assert_ne!(done["revision"], rev.as_str());

    // Committing the same preview again: the store moved on.
    let env = commit(&s, Verb::Confirm, json!({"placement": pid}), Some(&rev));
    assert_eq!(env.exit_code(), 6);
    let e = error(env);
    assert_eq!(e.code, ErrorCode::RevisionConflict);
    assert_eq!(e.details.unwrap()["expected"], rev.as_str());
    assert_eq!(
        e.next_command.as_deref(),
        Some(format!("shelfmark confirm {pid}").as_str())
    );

    // The machine claim is superseded, not deleted; the human one is current.
    let st = data(call(&s, Verb::Status, json!({})));
    assert_eq!(st["placements"]["claims"], 2);
    assert_eq!(st["flagged_backlog"], 0);
    assert_eq!(st["gold"]["count"], 1);
    let old = data(call(&s, Verb::Resolve, json!({"id": pid})));
    assert_eq!(old["resolved"]["status"], "superseded");
    let work = c["placement"]["work"].as_str().unwrap();
    let ex = data(call(&s, Verb::Explain, json!({"id": work})));
    assert_eq!(ex["explanations"][0]["status"], "confirmed");
    assert_eq!(
        ex["explanations"][0]["placement"]["class"],
        "class:mini-lcc@2024:K"
    );
    assert_eq!(
        ex["explanations"][0]["placement"]["provenance"]["author"],
        "david"
    );

    // Superseded and human claims are never memo hits, so a reclassify
    // runs again, and the confirmed claim stays current.
    let again = data(call(
        &s,
        Verb::Classify,
        json!({"text": "law and social sciences"}),
    ));
    assert_eq!(again["memo_hit"], false);
    assert!(again["placement"].get("supersedes").is_none());
    let ex = data(call(&s, Verb::Explain, json!({"id": work})));
    assert_eq!(ex["explanations"][0]["status"], "confirmed");
}

#[test]
fn correct_rejects_codes_outside_the_scheme() {
    let s = service("badcode");
    let c = data(call(&s, Verb::Classify, json!({"text": "law"})));
    let pid = c["placement"]["id"].as_str().unwrap();
    for class in ["ZZ1", "class:other@1:K"] {
        let e = error(call(
            &s,
            Verb::Correct,
            json!({"placement": pid, "class": class}),
        ));
        assert_eq!(e.code, ErrorCode::InvalidCode, "{class}");
        assert!(e
            .next_command
            .unwrap()
            .starts_with("shelfmark scheme search"));
    }
    // A code deeper than the outline is within a class, so it is accepted.
    let ok = data(call(
        &s,
        Verb::Correct,
        json!({"placement": pid, "class": "QA76.76.C65"}),
    ));
    assert_eq!(
        ok["change"]["append"]["class"],
        "class:mini-lcc@2024:QA76.76.C65"
    );
}

#[test]
fn scheme_compile_previews_commits_and_makes_old_placements_stale() {
    let dir = temp_dir("compile");
    common::seed_mini_lcc(&dir.join("store"));
    let s = shelfmark_cli::wiring::open(&dir.join("store"));
    let c = data(call(&s, Verb::Classify, json!({"text": "law"})));
    assert_eq!(c["placement"]["class"], "class:mini-lcc@2024:K");
    assert_eq!(data(call(&s, Verb::Stale, json!({})))["total"], 0);

    // A new revision of the same scheme ID, with K split (AE6 shape).
    let mut doc: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/schemes/mini-lcc.json")).unwrap();
    doc["classes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"code": "KF", "label": "Federal law", "parent": "K"}));
    let path = dir.join("mini-lcc.json");
    std::fs::write(&path, doc.to_string()).unwrap();
    let args = json!({"path": path.to_str().unwrap()});

    let v = data(call(&s, Verb::SchemeValidate, args.clone()));
    assert_eq!(v["valid"], true);
    let preview = data(call(&s, Verb::SchemeCompile, args.clone()));
    assert_eq!(preview["change"]["classes"], 6);
    assert!(preview["change"]["replaces_content_hash"].is_string());
    let rev = preview["revision"].as_str().unwrap();
    data(commit(&s, Verb::SchemeCompile, args, Some(rev)));

    let browse = data(call(&s, Verb::SchemeBrowse, json!({"code": "KF"})));
    assert_eq!(browse["ancestors"][1]["id"], "class:mini-lcc@2024:K");
    let stale = data(call(&s, Verb::Stale, json!({})));
    assert_eq!(stale["total"], 1);
    assert_eq!(stale["stale"][0]["placement"], c["placement"]["id"]);
    assert_eq!(stale["stale"][0]["reasons"], json!(["scheme changed"]));

    // Re-running replaces the stale claim.
    let rerun = data(call(&s, Verb::Classify, json!({"text": "law"})));
    assert_eq!(rerun["memo_hit"], false);
    assert_eq!(rerun["placement"]["supersedes"], c["placement"]["id"]);
    assert_eq!(data(call(&s, Verb::Stale, json!({})))["total"], 0);

    // An invalid file is scheme_invalid with the validate command.
    let bad = dir.join("bad.json");
    std::fs::write(&bad, r#"{"name": "x"}"#).unwrap();
    let e = error(call(
        &s,
        Verb::SchemeCompile,
        json!({"path": bad.to_str().unwrap()}),
    ));
    assert_eq!(e.code, ErrorCode::SchemeInvalid);
    assert!(e
        .next_command
        .unwrap()
        .starts_with("shelfmark scheme validate --path"));
}

#[test]
fn link_previews_commits_and_resolves_the_item() {
    let dir = temp_dir("link");
    common::seed_mini_lcc(&dir.join("store"));
    let s = shelfmark_cli::wiring::open(&dir.join("store"));
    let unknown = format!("item:s256:{}", "a".repeat(64));
    let e = error(call(&s, Verb::Resolve, json!({"id": unknown})));
    assert_eq!(e.code, ErrorCode::NotFound);
    assert!(e
        .next_command
        .unwrap()
        .starts_with("shelfmark classify --file"));

    let classify = |name: &str, text: &str| {
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        data(call(&s, Verb::Classify, json!({"file": path})))
    };
    let a = classify("a.md", "law of contracts");
    let b = classify("b.md", "contract law notes");
    let work = a["placement"]["work"].as_str().unwrap();
    let item = b["item"].as_str().unwrap();
    assert_ne!(b["placement"]["work"].as_str().unwrap(), work);

    let args = json!({"item": item, "work": work});
    let rev = data(call(&s, Verb::Link, args.clone()))["revision"]
        .as_str()
        .unwrap()
        .to_string();
    data(commit(&s, Verb::Link, args, Some(&rev)));
    let r = data(call(&s, Verb::Resolve, json!({"id": item})));
    assert_eq!(r["resolved"]["work"], work);
    let w = data(call(&s, Verb::Resolve, json!({"id": work})));
    assert_eq!(w["resolved"]["items"], json!([a["item"], item]));
}
