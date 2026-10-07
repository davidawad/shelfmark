//! Template parse errors are caught at parse time, with a position.

use shelfmark_core::ErrorCode;
use shelfmark_naming::{Naming, NamingOptions, Part, Template, Var, DEFAULT_TEMPLATE};

fn err(src: &str) -> (usize, String) {
    let e = Template::parse(src).expect_err(src);
    (e.position, e.message)
}

#[test]
fn default_and_bead_templates_parse() {
    let t = Template::parse(DEFAULT_TEMPLATE).unwrap();
    assert_eq!(
        t.vars(),
        [Var::ClassPath, Var::Cutter, Var::Title, Var::Type, Var::Ext]
    );
    let t = Template::parse("{class_path}/{cutter} {title} [{type}].{ext}").unwrap();
    assert_eq!(t.segments().len(), 2);
    assert_eq!(
        t.segments()[1],
        [
            Part::Var(Var::Cutter),
            Part::Lit(" ".into()),
            Part::Var(Var::Title),
            Part::Lit(" [".into()),
            Part::Var(Var::Type),
            Part::Lit("].".into()),
            Part::Var(Var::Ext),
        ]
    );
}

#[test]
fn unknown_variable_names_its_position_and_the_known_ones() {
    let (pos, msg) = err("{class_path}/{cuter} {title}");
    assert_eq!(pos, 13);
    assert!(msg.contains("unknown variable `{cuter}`"), "{msg}");
    assert!(msg.contains("cutter"), "{msg}");
}

#[test]
fn syntax_errors() {
    for (src, pos, needle) in [
        ("", 0, "empty"),
        ("{title", 0, "unclosed `{`"),
        ("{title/x}", 0, "unclosed `{`"),
        ("{}", 0, "unknown variable"),
        ("title}", 5, "stray `}`"),
        ("{title}<[{type}]", 7, "unclosed `<`"),
        ("{title}>", 7, "without an opening"),
        ("<a<{type}>>", 2, "cannot nest"),
        ("<[{type}]/x>", 9, "cannot span"),
        ("<plain>", 6, "names no variable"),
        ("/{title}", 0, "empty folder"),
        ("a//{title}", 2, "empty folder"),
        ("{class_path}/", 13, "needs a file name"),
        ("a\\{title}", 1, "use `/`"),
        ("x {class_path}/{title}", 0, "whole folder"),
        ("a/{class_path}", 2, "cannot be the file name"),
        ("<{class_path}>/{title}", 1, "cannot be optional"),
    ] {
        let (got_pos, msg) = err(src);
        assert!(msg.contains(needle), "{src:?}: {msg}");
        assert_eq!(got_pos, pos, "{src:?}: {msg}");
    }
}

#[test]
fn escaped_braces_are_literal() {
    let t = Template::parse("{{{title}}}").unwrap();
    assert_eq!(
        t.segments()[0],
        [
            Part::Lit("{".into()),
            Part::Var(Var::Title),
            Part::Lit("}".into())
        ]
    );
}

#[test]
fn naming_new_maps_to_invalid_request_with_details() {
    let e = Naming::new("{titel}", NamingOptions::default()).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidRequest);
    assert!(e.next_command.is_some());
    let details = e.details.unwrap();
    assert_eq!(details["position"], 0);
    assert_eq!(details["template"], "{titel}");
    assert!(details["variables"]
        .as_array()
        .unwrap()
        .contains(&"title".into()));
}
