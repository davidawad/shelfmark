//! Text extraction per format, and the error codes for inputs that cannot
//! be read.

use shelfmark_core::{ErrorCode, WorkId};
use shelfmark_identity::extract::{extract, markdown_to_text};
use shelfmark_identity::{Format, Minted};

#[test]
fn extensions_map_to_formats() {
    for (ext, f) in [
        ("txt", Format::Text),
        ("MD", Format::Markdown),
        ("htm", Format::Html),
        ("pdf", Format::Pdf),
    ] {
        assert_eq!(Format::from_extension(ext), Some(f));
    }
    assert_eq!(Format::from_extension("docx"), None);
    assert_eq!(
        serde_json::to_string(&Format::Markdown).unwrap(),
        "\"markdown\""
    );
}

#[test]
fn unknown_extension_is_input_unreadable() {
    let err = Minted::from_path(std::path::Path::new("notes.docx")).unwrap_err();
    assert_eq!(err.code, ErrorCode::InputUnreadable);
    assert_eq!(err.details.unwrap()["reason"], "unsupported_format");
}

#[test]
fn missing_file_is_input_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    let err = Minted::from_path(&dir.path().join("gone.txt")).unwrap_err();
    assert_eq!(err.code, ErrorCode::InputUnreadable);
    assert_eq!(err.details.unwrap()["reason"], "read_failed");
}

#[test]
fn text_tolerates_bom_and_bad_utf8() {
    let text = extract(b"\xEF\xBB\xBFcaf\xC3\xA9 \xFF bar", Format::Text).unwrap();
    assert_eq!(WorkId::from_text(&text), WorkId::from_text("café bar"));
}

#[test]
fn markdown_keeps_words_not_markup() {
    let md = "Intro\n\n- one\n- two\n\n```rust\nlet x = 1;\n```\n\n\
              | a | b |\n|---|---|\n| c | d |\n\n![alt text](img.png) end<br>tail";
    let text = markdown_to_text(md);
    assert_eq!(
        WorkId::from_text(&text),
        WorkId::from_text("Intro one two let x 1 a b c d alt text end tail"),
        "{text:?}"
    );
    assert!(!text.contains("img.png"));
}

#[test]
fn blocks_never_fuse_words() {
    let html = extract(b"<p>end</p><p>start</p><div>x</div>y", Format::Html).unwrap();
    assert_eq!(WorkId::from_text(&html), WorkId::from_text("end start x y"));
    let md = extract(b"# Head\nbody\n\n> quote", Format::Markdown).unwrap();
    assert_eq!(WorkId::from_text(&md), WorkId::from_text("head body quote"));
}

#[cfg(not(feature = "pdf"))]
#[test]
fn pdf_without_feature_is_unsupported_with_a_next_command() {
    let err = extract(b"%PDF-1.4", Format::Pdf).unwrap_err();
    assert_eq!(err.code, ErrorCode::InputUnreadable);
    let details = err.details.clone().unwrap();
    assert_eq!(details["reason"], "unsupported_format");
    assert_eq!(details["format"], "pdf");
    assert_eq!(
        err.next_command.as_deref(),
        Some("cargo install shelfmark --features pdf")
    );
    // From a path the file is never read.
    let err = Minted::from_path(std::path::Path::new("/nonexistent/book.pdf")).unwrap_err();
    assert_eq!(err.details.unwrap()["reason"], "unsupported_format");
}

#[cfg(feature = "pdf")]
mod pdf {
    use super::*;

    /// A one-page PDF showing `line` in Helvetica, with a correct xref.
    fn pdf(line: &str) -> Vec<u8> {
        let stream = format!("BT /F1 12 Tf 72 720 Td ({line}) Tj ET");
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
             /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
                .to_string(),
            format!(
                "<< /Length {} >>\nstream\n{stream}\nendstream",
                stream.len()
            ),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
                .to_string(),
        ];
        let mut out = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (i, body) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend(format!("{} 0 obj\n{body}\nendobj\n", i + 1).bytes());
        }
        let xref = out.len();
        out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).bytes());
        for off in offsets {
            out.extend(format!("{off:010} 00000 n \n").bytes());
        }
        out.extend(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                objects.len() + 1
            )
            .bytes(),
        );
        out
    }

    #[test]
    fn pdf_text_matches_the_txt_work() {
        let words = "Fundamental algorithms and information structures";
        let minted = Minted::from_bytes(&pdf(words), Format::Pdf).unwrap();
        assert_eq!(minted.work, WorkId::from_text(words), "{:?}", minted.text);
    }

    #[test]
    fn broken_pdf_is_input_unreadable_not_a_panic() {
        let err = extract(b"%PDF-1.4\nnot really a pdf", Format::Pdf).unwrap_err();
        assert_eq!(err.code, ErrorCode::InputUnreadable);
        assert_eq!(err.details.unwrap()["reason"], "extract_failed");
    }
}
