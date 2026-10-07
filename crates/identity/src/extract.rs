//! Plain text from an input file (R2): `.txt`, `.md`, `.html`, and `.pdf`
//! behind the `pdf` feature. The text only has to keep the words: Work
//! IDs hash it after normalization v1, which drops punctuation, case and
//! spacing, so a format conversion that keeps the words keeps the ID.

use std::path::Path;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use shelfmark_core::{Error, ErrorCode};

use crate::html::html_to_text;

/// An input format this crate can read text from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Text,
    Markdown,
    Html,
    Pdf,
}

impl Format {
    pub const ALL: [Format; 4] = [Format::Text, Format::Markdown, Format::Html, Format::Pdf];

    /// The format named by a file extension (case-insensitive).
    pub fn from_extension(ext: &str) -> Option<Format> {
        match ext.to_ascii_lowercase().as_str() {
            "txt" | "text" => Some(Format::Text),
            "md" | "markdown" | "mdown" | "mkd" => Some(Format::Markdown),
            "html" | "htm" | "xhtml" => Some(Format::Html),
            "pdf" => Some(Format::Pdf),
            _ => None,
        }
    }

    /// The format of a path, by its extension.
    pub fn from_path(path: &Path) -> Result<Format, Error> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Format::from_extension)
            .ok_or_else(|| {
                unsupported(
                    format!(
                        "{} has no supported extension (.txt, .md, .html, .pdf)",
                        path.display()
                    ),
                    None,
                )
            })
    }

    /// Whether this build can extract text from the format.
    pub fn supported(self) -> bool {
        self != Format::Pdf || cfg!(feature = "pdf")
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Format::Text => "text",
            Format::Markdown => "markdown",
            Format::Html => "html",
            Format::Pdf => "pdf",
        }
    }
}

fn unsupported(message: String, format: Option<Format>) -> Error {
    let mut details = serde_json::json!({ "reason": "unsupported_format" });
    if let Some(f) = format {
        details["format"] = f.as_str().into();
    }
    Error::new(ErrorCode::InputUnreadable, message).details(details)
}

#[cfg(feature = "pdf")]
fn unreadable(format: Format, reason: impl std::fmt::Display) -> Error {
    Error::new(
        ErrorCode::InputUnreadable,
        format!("could not extract {} text: {reason}", format.as_str()),
    )
    .details(serde_json::json!({ "reason": "extract_failed", "format": format.as_str() }))
}

/// Extract plain text from `bytes` in `format`.
pub fn extract(bytes: &[u8], format: Format) -> Result<String, Error> {
    match format {
        Format::Text => Ok(decode(bytes)),
        Format::Markdown => Ok(markdown_to_text(&decode(bytes))),
        Format::Html => Ok(html_to_text(&decode(bytes))),
        Format::Pdf => pdf_text(bytes),
    }
}

/// Text files are read as UTF-8; invalid sequences become U+FFFD, which
/// normalization drops like any other non-alphanumeric character.
fn decode(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(feature = "pdf")]
fn pdf_text(bytes: &[u8]) -> Result<String, Error> {
    // pdf-extract panics on some malformed files; a bad input must be an
    // error, never a crash.
    match std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem(bytes)) {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(e)) => Err(unreadable(Format::Pdf, e)),
        Err(_) => Err(unreadable(
            Format::Pdf,
            "the PDF parser failed on this file",
        )),
    }
}

#[cfg(not(feature = "pdf"))]
fn pdf_text(_bytes: &[u8]) -> Result<String, Error> {
    Err(not_in_build(Format::Pdf))
}

/// The error for a format this build cannot read (only PDF without the
/// `pdf` feature).
fn not_in_build(format: Format) -> Error {
    unsupported(
        format!(
            "{} text extraction is not in this build (cargo feature `{}`)",
            format.as_str(),
            format.as_str()
        ),
        Some(format),
    )
    .next(format!(
        "cargo install shelfmark --features {}",
        format.as_str()
    ))
}

/// The words of a CommonMark document: text and code, never link targets,
/// image URLs or YAML front matter. Blocks end with a line break so words
/// in adjacent blocks never fuse.
pub fn markdown_to_text(md: &str) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
    let mut out = String::with_capacity(md.len());
    let mut in_metadata = false;
    for event in Parser::new_ext(md, opts) {
        match event {
            Event::Start(Tag::MetadataBlock(_)) => in_metadata = true,
            Event::End(TagEnd::MetadataBlock(_)) => in_metadata = false,
            _ if in_metadata => {}
            Event::Text(t) | Event::Code(t) => out.push_str(&t),
            Event::Html(h) | Event::InlineHtml(h) => out.push_str(&html_to_text(&h)),
            Event::SoftBreak | Event::HardBreak | Event::Rule => out.push('\n'),
            Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::Item
                | TagEnd::CodeBlock
                | TagEnd::TableCell
                | TagEnd::FootnoteDefinition,
            ) => out.push('\n'),
            _ => {}
        }
    }
    out
}

/// Read a file and extract its text, choosing the format by extension.
/// Returns the bytes too, since the Item ID hashes them.
pub fn read_file(path: &Path) -> Result<(Vec<u8>, Format, String), Error> {
    let format = Format::from_path(path)?;
    if !format.supported() {
        return Err(not_in_build(format));
    }
    let bytes = std::fs::read(path).map_err(|e| {
        Error::new(
            ErrorCode::InputUnreadable,
            format!("could not read {}: {e}", path.display()),
        )
        .details(serde_json::json!({ "reason": "read_failed", "path": path.display().to_string() }))
    })?;
    let text = extract(&bytes, format)?;
    Ok((bytes, format, text))
}
