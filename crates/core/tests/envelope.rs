//! The shared envelope and error shapes (R19, R21; AE7).

use serde_json::json;
use shelfmark_core::envelope::{completed_inputs, BatchLine, Mode};
use shelfmark_core::{Envelope, Error, ErrorCode, Request, SchemeId, Verb};

#[test]
fn ok_and_error_envelopes_match_the_core_resource_shape() {
    let ok = serde_json::to_value(Envelope::ok(json!({"n": 1}))).unwrap();
    assert_eq!(ok, json!({"ok": true, "data": {"n": 1}}));
    let err =
        Envelope::err(Error::new(ErrorCode::NotFound, "no such work").next("shelfmark status"));
    let v = serde_json::to_value(&err).unwrap();
    assert_eq!(
        v,
        json!({"ok": false, "error": {"code": "not_found", "message": "no such work",
                                      "next_command": "shelfmark status"}})
    );
    assert_eq!(serde_json::from_value::<Envelope>(v).unwrap(), err);
    assert_eq!(err.exit_code(), 3);
    assert!(serde_json::from_value::<Envelope>(json!({"ok": false, "data": 1})).is_err());
}

#[test]
fn ae7_uncompiled_scheme_names_the_compile_command() {
    let scheme = SchemeId::new("lcc", "2025");
    let e = Error::scheme_not_compiled(&scheme, Some("schemes/lcc.json"));
    assert_eq!(e.code.as_str(), "scheme_not_compiled");
    assert_eq!(
        e.next_command.as_deref(),
        Some("shelfmark scheme compile schemes/lcc.json")
    );
    assert!(e.message.contains("scheme:lcc@2025"));
    let unknown = Error::scheme_not_compiled(&scheme, None);
    assert!(unknown
        .next_command
        .unwrap()
        .starts_with("shelfmark scheme compile"));
}

#[test]
fn every_error_code_has_a_stable_string_and_nonzero_exit() {
    for code in ErrorCode::ALL {
        let s = serde_json::to_value(code).unwrap();
        assert_eq!(s, json!(code.as_str()));
        assert!(
            code.exit_code() > 1,
            "{code}: exit 1 is reserved for flagged runs"
        );
    }
}

#[test]
fn requests_default_to_preview_and_verbs_use_kebab_names() {
    let r: Request = serde_json::from_value(json!({"verb": "list-flagged"})).unwrap();
    assert_eq!(r.verb, Verb::ListFlagged);
    assert_eq!(r.mode, Mode::Preview);
    assert_eq!(r.args, json!({}));
    for v in Verb::ALL {
        assert_eq!(serde_json::to_value(v).unwrap(), json!(v.as_str()));
    }
    assert!(Verb::Confirm.mutates() && !Verb::Classify.mutates());
}

#[test]
fn batch_lines_flatten_the_envelope_and_resume_skips_only_successes() {
    let line = BatchLine {
        input: "item:s256:aa".into(),
        envelope: Envelope::ok(json!({"class": "x"})),
    };
    let text = serde_json::to_string(&line).unwrap();
    assert_eq!(
        text,
        r#"{"input":"item:s256:aa","ok":true,"data":{"class":"x"}}"#
    );
    assert_eq!(serde_json::from_str::<BatchLine>(&text).unwrap(), line);
    let failed = BatchLine {
        input: "item:s256:bb".into(),
        envelope: Envelope::err(Error::new(ErrorCode::InputUnreadable, "bad pdf")),
    };
    let ndjson = format!(
        "{text}\n{}\nnot json\n",
        serde_json::to_string(&failed).unwrap()
    );
    let done = completed_inputs(&ndjson);
    assert_eq!(done.into_iter().collect::<Vec<_>>(), ["item:s256:aa"]);
}
