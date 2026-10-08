//! Text out of the FreeLCC outline PDFs, one string per outline line.
//!
//! The PDFs are tagged: every outline line is one marked-content
//! paragraph (`BDC /P <<MCID n>> ... EMC`), page furniture is marked
//! `/Artifact`, and spaces are real glyphs. So a line is the paragraph's
//! glyphs in order, with no spacing guessed from positions. A paragraph
//! that wraps moves the text cursor down; the wrap becomes a newline.
//! Glyph codes decode through each font's encoding (WinAnsi, or the
//! ToUnicode map of the Identity-H fonts that carry diacritics). A page
//! with no tags (one older outline PDF) comes back as a single paragraph
//! whose lines break wherever the cursor moves down.

use std::collections::BTreeMap;

use lopdf::content::Operation;
use lopdf::Encoding;
use lopdf::{Document, Object};

/// The text of every tagged paragraph, one list per page, in content order.
pub fn pages(bytes: &[u8]) -> Result<Vec<Vec<String>>, String> {
    let doc = Document::load_mem(bytes).map_err(|e| format!("not a PDF: {e}"))?;
    let mut out = Vec::new();
    for (_, page) in doc.get_pages() {
        let content = doc
            .get_and_decode_page_content(page)
            .map_err(|e| format!("page content: {e}"))?;
        let fonts = doc
            .get_page_fonts(page)
            .map_err(|e| format!("page fonts: {e}"))?;
        let encodings: BTreeMap<Vec<u8>, Encoding> = fonts
            .into_iter()
            .filter_map(|(name, font)| font.get_font_encoding(&doc).ok().map(|e| (name, e)))
            .collect();
        let mut paras = Vec::new();
        page_paragraphs(&content.operations, &encodings, &mut paras);
        out.push(paras);
    }
    Ok(out)
}

fn page_paragraphs(
    ops: &[Operation],
    encodings: &BTreeMap<Vec<u8>, Encoding>,
    out: &mut Vec<String>,
) {
    let mut font: Option<&Encoding> = None;
    // Text shown outside any paragraph, for untagged pages.
    let mut loose = String::new();
    // Marked-content stack: true where the content is an artifact.
    let mut stack: Vec<bool> = Vec::new();
    // The open paragraph: its depth in `stack` and its text so far.
    let mut para: Option<(usize, String)> = None;
    for op in ops {
        let in_artifact = stack.iter().any(|&a| a);
        match op.operator.as_str() {
            "Tf" => {
                font = op
                    .operands
                    .first()
                    .and_then(|o| o.as_name().ok())
                    .and_then(|n| encodings.get(n));
            }
            "BDC" | "BMC" => {
                let tag = op.operands.first().and_then(|o| o.as_name().ok());
                let artifact = tag == Some(b"Artifact".as_slice());
                if para.is_none() && !artifact && !in_artifact {
                    para = Some((stack.len(), String::new()));
                }
                stack.push(artifact);
            }
            "EMC" => {
                stack.pop();
                if para.as_ref().is_some_and(|(d, _)| *d == stack.len()) {
                    let (_, text) = para.take().unwrap();
                    out.push(text);
                }
            }
            "Tj" | "'" | "\"" | "TJ" if !in_artifact => {
                let text = match para.as_mut() {
                    Some((_, text)) => text,
                    None => &mut loose,
                };
                for o in &op.operands {
                    show(o, font, text);
                }
            }
            "Td" | "TD" | "Tm" | "T*" if !in_artifact => {
                let dy = match op.operator.as_str() {
                    "Td" | "TD" => op.operands.get(1).and_then(num),
                    "T*" => Some(-1.0),
                    _ => None,
                };
                let text = match para.as_mut() {
                    Some((_, text)) => text,
                    None => &mut loose,
                };
                if dy.is_some_and(|d| d.abs() > 0.5) && !text.is_empty() {
                    text.push('\n');
                }
            }
            _ => {}
        }
    }
    if out.is_empty() && !loose.trim().is_empty() {
        out.push(loose);
    }
}

fn num(o: &Object) -> Option<f64> {
    match o {
        Object::Integer(i) => Some(*i as f64),
        Object::Real(r) => Some(*r as f64),
        _ => None,
    }
}

/// Append a shown string (or a TJ array's strings) as text.
fn show(o: &Object, font: Option<&Encoding>, text: &mut String) {
    match o {
        Object::String(bytes, _) => match font.and_then(|e| e.bytes_to_string(bytes).ok()) {
            Some(s) => text.push_str(&s),
            None => text.extend(bytes.iter().map(|&b| b as char)),
        },
        Object::Array(items) => items.iter().for_each(|i| show(i, font, text)),
        _ => {}
    }
}
