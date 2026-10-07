//! AE7 (R21): a request naming a scheme that is not compiled fails with a
//! stable code and gives the compile command.

use crate::common::Bench;

#[test]
fn uncompiled_scheme_error_has_a_stable_code_and_the_compile_command() {
    let b = Bench::new("ae7");
    let (env, code) = b.run(&[
        "classify",
        "--text",
        "anything",
        "--scheme",
        "scheme:lcc@2024",
    ]);
    assert_eq!(env["ok"], false);
    assert_eq!(env["error"]["code"], "scheme_not_compiled");
    assert_eq!(
        code,
        shelfmark_core::ErrorCode::SchemeNotCompiled.exit_code()
    );
    let next = env["error"]["next_command"].as_str().unwrap();
    assert!(next.starts_with("shelfmark scheme compile"), "{next}");

    // The same shape from every verb that takes a scheme.
    let (browse, _) = b.run(&["scheme", "browse", "--scheme", "scheme:lcc@2024"]);
    assert_eq!(browse["error"]["code"], "scheme_not_compiled");
    assert_eq!(
        browse["error"]["next_command"],
        env["error"]["next_command"]
    );
}
