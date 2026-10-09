//! Positioned text runs out of the FreeLCC Text PDFs.
//!
//! The Text PDFs are untagged. Every run of text is its own `BT ... ET`
//! block with an absolute `Tm` matrix, so a run is (x, y, font size,
//! text) with no spacing to guess. Glyph codes decode through each font's
//! encoding, as in `lcc-outline`.

use std::collections::BTreeMap;

use lopdf::content::Operation;
use lopdf::{Document, Encoding, Object};

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    /// Font resource name on the page (`F1`, `F2`): bold and italic differ.
    pub font: String,
    pub text: String,
}

fn num(o: &Object) -> Option<f32> {
    match o {
        Object::Integer(i) => Some(*i as f32),
        Object::Real(r) => Some(*r),
        _ => None,
    }
}

/// The runs of every page, in content order.
pub fn pages(bytes: &[u8]) -> Result<Vec<Vec<Run>>, String> {
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
        out.push(runs(&content.operations, &encodings));
    }
    Ok(out)
}

fn runs(ops: &[Operation], encodings: &BTreeMap<Vec<u8>, Encoding>) -> Vec<Run> {
    let mut out = Vec::new();
    let mut font: Option<&Encoding> = None;
    let mut name = String::new();
    let mut size = 0.0;
    let (mut x, mut y) = (0.0, 0.0);
    let mut cur: Option<Run> = None;
    let flush = |cur: &mut Option<Run>, out: &mut Vec<Run>| {
        if let Some(r) = cur.take() {
            if !r.text.trim().is_empty() {
                out.push(r);
            }
        }
    };
    for op in ops {
        match op.operator.as_str() {
            "BT" => {
                flush(&mut cur, &mut out);
                x = 0.0;
                y = 0.0;
            }
            "ET" => flush(&mut cur, &mut out),
            "Tf" => {
                if let Some(n) = op.operands.first().and_then(|o| o.as_name().ok()) {
                    font = encodings.get(n);
                    name = String::from_utf8_lossy(n).into_owned();
                }
                size = op.operands.get(1).and_then(num).unwrap_or(0.0);
            }
            "Tm" => {
                flush(&mut cur, &mut out);
                x = op.operands.get(4).and_then(num).unwrap_or(0.0);
                y = op.operands.get(5).and_then(num).unwrap_or(0.0);
            }
            "Td" | "TD" => {
                flush(&mut cur, &mut out);
                x += op.operands.first().and_then(num).unwrap_or(0.0);
                y += op.operands.get(1).and_then(num).unwrap_or(0.0);
            }
            "Tj" | "'" | "\"" | "TJ" => {
                let run = cur.get_or_insert_with(|| Run {
                    x,
                    y,
                    size,
                    font: name.clone(),
                    text: String::new(),
                });
                for o in &op.operands {
                    show(o, font, &mut run.text);
                }
            }
            _ => {}
        }
    }
    flush(&mut cur, &mut out);
    out
}

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
