//! The `shelfmark` binary with stdout piped (not a TTY): every output is an
//! envelope, the exit code follows it, and batches resume (R19, R21).

mod common;

use std::path::Path;
use std::process::{Command, Output};

use common::temp_dir;
use serde_json::{json, Value};
use shelfmark_core::envelope::BatchLine;
use shelfmark_core::Envelope;

fn run(store: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_shelfmark"))
        .arg("--store")
        .arg(store)
        .args(args)
        .env_remove("SHELFMARK_STORE")
        .output()
        .unwrap()
}

/// stdout parsed as one envelope, checked against the exit code.
fn envelope(store: &Path, args: &[&str]) -> Value {
    let out = run(store, args);
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(text.lines().count(), 1, "{args:?}: {text}");
    let env: Envelope =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{args:?}: {e}: {text}"));
    assert_eq!(out.status.code(), Some(env.exit_code()), "{args:?}: {text}");
    serde_json::to_value(env).unwrap()
}

#[test]
fn non_tty_output_parses_as_an_envelope_for_every_subcommand() {
    let dir = temp_dir("bin");
    let store = dir.join("store");
    let c = envelope(
        &store,
        &[
            "classify",
            "--text",
            "compilers (computer programs)",
            "--identifier",
            "isbn:9780262033848",
        ],
    );
    let pid = c["data"]["placement"]["id"].as_str().unwrap().to_string();
    let work = c["data"]["placement"]["work"].as_str().unwrap().to_string();
    let scheme = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/schemes/mini-lcc.json"
    ))
    .unwrap();
    let scheme_path = dir.join("s.json");
    std::fs::write(&scheme_path, scheme).unwrap();
    let sp = scheme_path.to_str().unwrap();
    let doc = dir.join("doc.md");
    std::fs::write(&doc, "# Notes\n\nOn compilers.").unwrap();
    let f = envelope(&store, &["classify", "--file", doc.to_str().unwrap()]);
    let item = f["data"]["item"].as_str().unwrap().to_string();
    let ok_calls: Vec<Vec<&str>> = vec![
        vec!["describe"],
        vec!["describe", "classify"],
        vec!["status"],
        vec!["resolve", &work],
        vec!["explain", &pid],
        vec!["list-flagged", "--limit", "5"],
        vec!["confirm", &pid],
        vec!["correct", &pid, "--class", "QA"],
        vec!["stale"],
        vec!["scheme", "compile", sp],
        vec!["scheme", "browse", "QA"],
        vec!["scheme", "search", "law"],
        vec!["scheme", "validate", "QA76"],
        vec!["scheme", "validate", "--path", sp],
        vec!["name", &work, "--title", "T"],
        vec!["link", "--item", &item, "--work", &work],
        vec!["call", r#"{"verb": "status"}"#],
    ];
    for args in ok_calls {
        assert_eq!(envelope(&store, &args)["ok"], true, "{args:?}");
    }
    let e = envelope(&store, &["eval"]);
    assert_eq!(e["error"]["code"], "tier_unavailable");
}

#[test]
fn errors_carry_code_next_command_and_exit_code() {
    let store = temp_dir("binerr").join("store");
    let cases: [(&[&str], &str, i32); 5] = [
        (&["resolve", "nonsense"], "invalid_id", 2),
        (
            &["classify", "--scheme", "scheme:lcc@2024", "--text", "x"],
            "scheme_not_compiled",
            4,
        ),
        (&["resolve", "profile:none@1"], "profile_not_found", 3),
        (&["no-such-verb"], "invalid_request", 2),
        (
            &["classify", "--target", "not-a-number"],
            "invalid_request",
            2,
        ),
    ];
    for (args, code, exit) in cases {
        let out = run(&store, args);
        assert_eq!(out.status.code(), Some(exit), "{args:?}");
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["ok"], false);
        assert_eq!(v["error"]["code"], code, "{args:?}");
        assert!(
            v["error"]["next_command"]
                .as_str()
                .unwrap()
                .starts_with("shelfmark"),
            "{args:?}"
        );
    }
}

#[test]
fn commit_with_a_stale_expected_revision_is_a_revision_conflict() {
    let store = temp_dir("bincommit").join("store");
    let c = envelope(&store, &["classify", "--text", "law and social sciences"]);
    let pid = c["data"]["placement"]["id"].as_str().unwrap().to_string();
    let preview = envelope(&store, &["confirm", &pid]);
    let rev = preview["data"]["revision"].as_str().unwrap().to_string();
    envelope(&store, &["classify", "--text", "something else about law"]);
    let out = run(
        &store,
        &["confirm", &pid, "--commit", "--expected-revision", &rev],
    );
    assert_eq!(out.status.code(), Some(6));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["code"], "revision_conflict");
    // Preview again, commit what it says.
    let preview = envelope(&store, &["confirm", &pid]);
    let cmd = preview["data"]["commit_command"].as_str().unwrap();
    let args: Vec<&str> = cmd.split(' ').skip(1).collect();
    assert_eq!(envelope(&store, &args)["data"]["mode"], "commit");
}

#[test]
fn batch_writes_one_line_per_input_and_resume_skips_completed_ones() {
    let dir = temp_dir("batch");
    let store = dir.join("store");
    let files: Vec<String> = ["law", "compilers (computer programs)", "mathematics"]
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let p = dir.join(format!("{i}.txt"));
            std::fs::write(&p, text).unwrap();
            p.to_str().unwrap().to_string()
        })
        .collect();
    let missing = dir.join("missing.txt").to_str().unwrap().to_string();
    let out = dir.join("out.ndjson");
    let o = out.to_str().unwrap();

    // The summary is an ok envelope; the exit code is the first failure's.
    let summary = |args: &[&str]| -> (Value, i32) {
        let out = run(&store, args);
        (
            serde_json::from_slice(&out.stdout).unwrap(),
            out.status.code().unwrap(),
        )
    };
    let (first, code) = summary(&["batch", &files[0], &files[1], &missing, "--out", o]);
    assert_eq!(code, 7);
    assert_eq!(
        first["data"],
        json!({"total": 3, "skipped": 0, "ok": 2, "failed": 1, "exit_code": 7})
    );
    let lines = |p: &Path| -> Vec<BatchLine> {
        std::fs::read_to_string(p)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    };
    let l1 = lines(&out);
    assert_eq!(l1.len(), 3);
    assert!(l1[0].input.starts_with("item:s256:") && l1[0].envelope.is_ok());
    assert_eq!(l1[2].input, format!("path:{missing}"));

    let (second, code) = summary(&[
        "batch", &files[0], &files[1], &files[2], &missing, &files[0], "--out", o, "--resume",
    ]);
    assert_eq!(code, 7);
    assert_eq!(
        second["data"],
        json!({"total": 5, "skipped": 3, "ok": 1, "failed": 1, "exit_code": 7})
    );
    let l2 = lines(&out);
    assert_eq!(l2.len(), 5, "appended the new file and the retried failure");
    assert_eq!(&l2[..3], &l1[..]);
    let ok: Vec<&str> = l2
        .iter()
        .filter(|l| l.envelope.is_ok())
        .map(|l| l.input.as_str())
        .collect();
    assert_eq!(ok.len(), 3);

    // Request batches stream to stdout, keyed by request hash.
    let reqs = dir.join("reqs.ndjson");
    std::fs::write(&reqs, "{\"verb\":\"status\"}\nnot json\n").unwrap();
    let res = run(&store, &["batch", "--requests", reqs.to_str().unwrap()]);
    assert_eq!(res.status.code(), Some(2));
    let text = String::from_utf8(res.stdout).unwrap();
    let streamed: Vec<BatchLine> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(streamed.len(), 2);
    assert!(streamed[0].input.starts_with("request:s256:") && streamed[0].envelope.is_ok());
    assert!(!streamed[1].envelope.is_ok());

    let e = envelope(&store, &["batch", &files[0], "--resume"]);
    assert_eq!(e["error"]["code"], "invalid_request");
}
