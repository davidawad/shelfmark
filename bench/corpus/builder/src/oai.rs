//! OAI-PMH record parsing for the arXivRaw, arXiv, and arXivOld formats.
//!
//! The arXivRaw and arXiv formats served by `oaipmh.arxiv.org` repeat the
//! `msc-class` string inside `acm-class`, so their ACM element is not trusted
//! when it mirrors the MSC element. arXivOld carries the submitted ACM classes;
//! when an arXivOld record is present for an ID, its ACM classes win.
//!
//! The `msc-class` text mixes codes with markers such as `(Primary)` and
//! `Secondary:`; only tokens shaped like MSC 2020 codes are kept.

use crate::{source_for, Paper};
use quick_xml::{events::Event, Reader};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// `arXivRaw` or `arXiv`: the primary record for a paper.
    Primary,
    /// `arXivOld`: used only for its ACM classes.
    Old,
}

fn format_of(name: &str) -> Option<Format> {
    match name {
        "arXivRaw" | "arXiv" => Some(Format::Primary),
        "arXivOld" => Some(Format::Old),
        _ => None,
    }
}

fn local_name(name: &[u8]) -> String {
    String::from_utf8_lossy(name)
        .rsplit(':')
        .next()
        .unwrap_or("")
        .to_string()
}

fn split_codes(s: &str) -> Vec<String> {
    s.split(|c: char| c == ',' || c.is_whitespace() || c == ';')
        .filter(|s| {
            !s.is_empty()
                && !s.eq_ignore_ascii_case("none")
                && !s.eq_ignore_ascii_case("n/a")
                && !s.eq_ignore_ascii_case("na")
        })
        .map(str::to_string)
        .collect()
}

/// The MSC 2020 code at the start of `s`, in canonical case: `05C10`,
/// `05Cxx`, `05-XX` or `05-01`.
fn msc_code_at(s: &[u8]) -> Option<String> {
    let digits = |b: &[u8]| b.iter().all(u8::is_ascii_digit);
    let xx = |b: &[u8]| b.eq_ignore_ascii_case(b"xx");
    let w = s.get(..5)?;
    if !digits(&w[..2]) || s.get(5).is_some_and(u8::is_ascii_alphanumeric) {
        return None;
    }
    let top = String::from_utf8_lossy(&w[..2]);
    if w[2] == b'-' && xx(&w[3..]) {
        Some(format!("{top}-XX"))
    } else if w[2] == b'-' && digits(&w[3..]) {
        Some(String::from_utf8_lossy(w).into_owned())
    } else if w[2].is_ascii_alphabetic() && xx(&w[3..]) {
        Some(format!("{top}{}xx", w[2].to_ascii_uppercase() as char))
    } else if w[2].is_ascii_alphabetic() && digits(&w[3..]) {
        Some(String::from_utf8_lossy(w).to_ascii_uppercase())
    } else {
        None
    }
}

/// MSC 2020 codes in an `msc-class` string, each once, in source order.
/// Markers (`(Primary)`, `Secondary:`), years, ACM codes and words are
/// dropped; a code glued to punctuation (`15A23.`, `74K20(primary)`,
/// `37Axx:37A30`) is kept, and lowercase letters are canonicalized.
pub fn msc_codes(s: &str) -> Vec<String> {
    let bytes = s.as_bytes();
    let mut out: Vec<String> = Vec::new();
    for i in 0..bytes.len() {
        if i > 0 && bytes[i - 1].is_ascii_alphanumeric() {
            continue;
        }
        if let Some(code) = msc_code_at(&bytes[i..]) {
            if !out.contains(&code) {
                out.push(code);
            }
        }
    }
    out
}

/// ACM classes such as `F.3.1 [Specifying ...]; I.2.11 [...]: Multiagent
/// systems`: drop bracketed descriptors and text after a colon, then keep the
/// leading code of each comma- or semicolon-separated part.
fn split_acm(s: &str) -> Vec<String> {
    let mut bare = String::with_capacity(s.len());
    let mut depth = 0usize;
    for c in s.chars() {
        match c {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => bare.push(c),
            _ => {}
        }
    }
    bare.split(';')
        .flat_map(|part| part.split(':').next().unwrap_or("").split(','))
        .filter_map(|part| part.split_whitespace().next())
        .flat_map(split_codes)
        .collect()
}

fn jel_code_tokens(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    bytes
        .windows(3)
        .enumerate()
        .filter(|(i, w)| {
            w[0].is_ascii_uppercase()
                && w[1].is_ascii_digit()
                && w[2].is_ascii_digit()
                && (*i == 0 || !bytes[*i - 1].is_ascii_alphanumeric())
                && (*i + 3 == bytes.len() || !bytes[*i + 3].is_ascii_alphanumeric())
        })
        .map(|(_, w)| String::from_utf8_lossy(w).into_owned())
        .collect()
}

fn jel_from_comments(text: &str) -> Vec<String> {
    let Some(index) = text.to_ascii_lowercase().find("jel") else {
        return Vec::new();
    };
    jel_code_tokens(&text[index + 3..])
}

fn empty_paper() -> Paper {
    Paper {
        id: String::new(),
        title: String::new(),
        abstract_text: String::new(),
        authors: vec![],
        categories: vec![],
        msc: vec![],
        acm_ccs: vec![],
        jel: vec![],
        license: "CC0 1.0".into(),
        source: source_for("", &[], &[], &[], &[]),
    }
}

/// Raw per-record state that is finalized when the metadata element closes.
#[derive(Default)]
struct Pending {
    format: Option<Format>,
    comments: String,
    msc_raw: String,
    acm_raw: String,
}

fn finish(mut p: Paper, pending: &Pending) -> Paper {
    p.msc = msc_codes(&pending.msc_raw);
    p.acm_ccs = split_acm(&pending.acm_raw);
    // arXivRaw/arXiv mirror msc-class into acm-class; such a value is not ACM.
    if pending.format == Some(Format::Primary)
        && !pending.acm_raw.is_empty()
        && split_codes(&pending.acm_raw) == split_codes(&pending.msc_raw)
    {
        p.acm_ccs.clear();
    }
    p.jel.extend(jel_from_comments(&pending.comments));
    p.jel
        .extend(p.categories.iter().flat_map(|c| jel_code_tokens(c)));
    p.jel.sort();
    p.jel.dedup();
    p.source = source_for(&p.id, &p.categories, &p.msc, &p.acm_ccs, &p.jel);
    p
}

fn append_line(field: &mut String, sep: &str, val: &str) {
    if !field.is_empty() {
        field.push_str(sep);
    }
    field.push_str(val);
}

/// Parse every arXiv metadata record in document order, tagged by format.
pub fn parse_records(xml: &str) -> Result<Vec<(Format, Paper)>, String> {
    let mut r = Reader::from_str(xml);
    r.config_mut().trim_text(true);
    let (mut out, mut cur, mut pending) = (Vec::new(), None::<Paper>, Pending::default());
    let (mut field, mut author, mut forenames) = (String::new(), String::new(), String::new());
    loop {
        match r.read_event().map_err(|e| e.to_string())? {
            Event::Start(e) => {
                let n = local_name(e.name().as_ref());
                if let Some(format) = format_of(&n) {
                    pending = Pending {
                        format: Some(format),
                        ..Pending::default()
                    };
                    cur = Some(empty_paper());
                }
                if n == "author" {
                    author.clear();
                    forenames.clear();
                }
                field = n;
            }
            Event::Text(t) => {
                let val = t.unescape().map_err(|e| e.to_string())?.into_owned();
                let Some(p) = cur.as_mut() else { continue };
                match field.as_str() {
                    "id" => p.id.push_str(&val),
                    "title" => append_line(&mut p.title, " ", &val),
                    "abstract" => append_line(&mut p.abstract_text, " ", &val),
                    "keyname" => author = val,
                    "forenames" => forenames = val,
                    "authors" => {
                        p.authors = val
                            .split(',')
                            .flat_map(|s| s.split(" and "))
                            .map(str::trim)
                            .map(|s| s.trim_start_matches("and "))
                            .filter(|s| !s.is_empty())
                            .map(str::to_string)
                            .collect()
                    }
                    "categories" => p.categories.extend(split_codes(&val)),
                    "msc-class" => append_line(&mut pending.msc_raw, ", ", &val),
                    "acm-class" => append_line(&mut pending.acm_raw, "; ", &val),
                    "jel" => p.jel.extend(split_codes(&val)),
                    "comments" => pending.comments.push_str(&val),
                    _ => {}
                }
            }
            Event::End(e) => {
                let n = local_name(e.name().as_ref());
                if n == "author" {
                    let name = [forenames.as_str(), author.as_str()]
                        .into_iter()
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    if let (false, Some(p)) = (name.is_empty(), cur.as_mut()) {
                        p.authors.push(name)
                    }
                }
                if let Some(format) = format_of(&n) {
                    if let Some(p) = cur.take().filter(|p| !p.id.is_empty()) {
                        out.push((format, finish(p, &pending)));
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(out)
}

/// ACM classes by arXiv ID from arXivOld records.
pub fn old_acm(records: &[(Format, Paper)]) -> BTreeMap<String, Vec<String>> {
    records
        .iter()
        .filter(|(format, _)| *format == Format::Old)
        .map(|(_, p)| (p.id.clone(), p.acm_ccs.clone()))
        .collect()
}

/// Replace a paper's ACM classes and keep its label sources in sync.
pub fn set_acm(p: &mut Paper, acm: Vec<String>) {
    p.acm_ccs = acm;
    p.source = source_for(&p.id, &p.categories, &p.msc, &p.acm_ccs, &p.jel);
}

/// Replace a paper's MSC classes and keep its label sources in sync.
pub fn set_msc(p: &mut Paper, msc: Vec<String>) {
    p.msc = msc;
    p.source = source_for(&p.id, &p.categories, &p.msc, &p.acm_ccs, &p.jel);
}

/// Primary records sorted and deduplicated by ID, with arXivOld ACM merged in.
pub fn parse(xml: &str) -> Result<Vec<Paper>, String> {
    let records = parse_records(xml)?;
    let acm = old_acm(&records);
    let mut out: Vec<Paper> = records
        .into_iter()
        .filter(|(format, _)| *format == Format::Primary)
        .map(|(_, p)| p)
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out.dedup_by(|a, b| a.id == b.id);
    for p in &mut out {
        if let Some(codes) = acm.get(&p.id) {
            set_acm(p, codes.clone());
        }
    }
    Ok(out)
}
