//! The CLI contract agents rely on: JSON when piped, coded errors with a
//! next command on stderr, stable exit codes.

use serde_json::Value;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_mc-eval");
const DIR: &str = env!("CARGO_MANIFEST_DIR");

fn mc(args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .current_dir(DIR)
        .output()
        .expect("binary runs")
}

fn json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes)
        .unwrap_or_else(|e| panic!("not JSON ({e}): {}", String::from_utf8_lossy(bytes)))
}

#[test]
fn describe_lists_every_verb_and_error_code() {
    let out = mc(&["describe"]);
    assert!(out.status.success());
    let v = json(&out.stdout);
    for verb in ["describe", "run", "validate", "gold build", "gold stats"] {
        assert!(v["verbs"][verb].is_string(), "missing verb {verb}");
    }
    let codes = v["errors"]["codes"].as_array().unwrap();
    for src in [
        "src/gold.rs",
        "src/pred.rs",
        "src/goldbuild/mod.rs",
        "src/bin/mc-eval.rs",
        "src/error.rs",
    ] {
        let text = std::fs::read_to_string(format!("{DIR}/{src}")).unwrap();
        for code in text.split('"').filter(|s| {
            s.starts_with("E_") && s.chars().all(|c| c.is_ascii_uppercase() || c == '_')
        }) {
            assert!(
                codes.iter().any(|c| c == code),
                "{code} from {src} not described"
            );
        }
    }
}

#[test]
fn run_emits_json_report_when_piped() {
    let out = mc(&[
        "run",
        "--gold",
        "tests/data/gold-small.ndjson",
        "--predictions",
        "tests/data/pred-small.ndjson",
        "--bootstrap",
        "50",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = json(&out.stdout);
    assert_eq!(v["schema"], "mc-eval/report@1");
    assert_eq!(v["profiles"].as_array().unwrap().len(), 2);
    let p = &v["profiles"][1];
    assert_eq!(p["profile"], "p");
    assert!(p["levels"][0]["top1"]["ci"].is_array());
    assert!(p["tiers"][0]["levels"].is_array());
}

#[test]
fn text_format_renders_tables() {
    let out = mc(&[
        "run",
        "--gold",
        "tests/data/gold-small.ndjson",
        "--predictions",
        "tests/data/pred-small.ndjson",
        "--format",
        "text",
        "--bootstrap",
        "0",
    ]);
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(
        s.contains("profile p: 3 predicted")
            && s.contains("tier identifier")
            && s.contains("subclass"),
        "{s}"
    );
}

#[test]
fn errors_are_coded_json_with_next_command() {
    let out = mc(&[
        "run",
        "--gold",
        "missing.ndjson",
        "--predictions",
        "x.ndjson",
    ]);
    assert_eq!(out.status.code(), Some(4));
    let v = json(&out.stderr);
    assert_eq!(v["error"]["code"], "E_IO");
    assert!(v["error"]["next"].as_str().unwrap().contains("--gold"));

    let out = mc(&["run", "--gold", "tests/data/gold-small.ndjson"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(json(&out.stderr)["error"]["code"], "E_USAGE");

    let out = mc(&[
        "run",
        "--gold",
        "tests/data/gold-small.ndjson",
        "--predictions",
        "tests/data/gold-small.ndjson",
    ]);
    assert_eq!(out.status.code(), Some(3));
    assert_eq!(json(&out.stderr)["error"]["code"], "E_PRED_PARSE");
}

#[test]
fn validate_reports_unknown_items() {
    let out = mc(&[
        "validate",
        "--gold",
        "tests/data/gold-small.ndjson",
        "--predictions",
        "tests/data/pred-small.ndjson",
    ]);
    assert!(out.status.success());
    let v = json(&out.stdout);
    assert_eq!(v["gold"]["items"], 4);
    assert_eq!(v["predictions"]["unknown_items"][0]["item"], "nope");
}
