//! Path segments must be legal on Windows, macOS and Linux.

use shelfmark_naming::sanitize::segment;

fn s(raw: &str) -> String {
    segment(raw, 120, true)
}

#[test]
fn illegal_characters_are_replaced() {
    assert_eq!(s(r#"a<b>c:d"e/f\g|h?i*j"#), "a_b_c_d_e_f_g_h_i_j");
    assert_eq!(s("AC/DC: live"), "AC_DC_ live");
}

#[test]
fn control_characters_and_whitespace_runs_collapse() {
    assert_eq!(s("a\tb\n\nc\u{0}d   e"), "a b c d e");
    assert_eq!(s("  padded  "), "padded");
}

#[test]
fn trailing_dots_and_spaces_are_trimmed() {
    assert_eq!(s("Vol. 2. . "), "Vol. 2");
    assert_eq!(s("etc..."), "etc");
}

#[test]
fn leading_dots_are_not_hidden_files() {
    assert_eq!(s(".bashrc"), "_bashrc");
    assert_eq!(s(".."), "_");
    assert_eq!(s("."), "_");
}

#[test]
fn windows_reserved_names_get_a_suffix() {
    for (raw, want) in [
        ("CON", "CON_"),
        ("con", "con_"),
        ("Nul.txt", "Nul_.txt"),
        ("COM1", "COM1_"),
        ("lpt9.tar.gz", "lpt9_.tar.gz"),
        ("COM¹", "COM¹_"),
        ("CONIN$", "CONIN$_"),
        ("AUX .pdf", "AUX_ .pdf"),
    ] {
        assert_eq!(s(raw), want, "{raw}");
    }
    for fine in ["CONSOLE", "Conan", "COM10", "auxiliary.pdf", "nul_"] {
        assert_eq!(s(fine), fine);
    }
}

#[test]
fn empty_becomes_placeholder() {
    assert_eq!(s(""), "_");
    assert_eq!(s(" . "), "_");
}

#[test]
fn length_is_capped_in_bytes() {
    let long = "x".repeat(300);
    assert_eq!(segment(&long, 255, false).len(), 255);
    let named = format!("{} [book].pdf", "word ".repeat(60));
    let out = segment(&named, 100, true);
    assert!(out.len() <= 100);
    assert!(out.ends_with(".pdf"), "{out}");
    assert!(!out.trim_end_matches(".pdf").ends_with(' '));
}

#[test]
fn output_is_nfc() {
    let decomposed = "Cafe\u{301}";
    assert_eq!(s(decomposed), "Caf\u{e9}");
}

#[test]
fn sanitizing_is_idempotent() {
    for raw in ["CON", ".x.", "a:b", "  ..  ", "Nul.txt", &"é".repeat(90)] {
        let once = segment(raw, 64, true);
        assert_eq!(segment(&once, 64, true), once, "{raw}");
    }
}
