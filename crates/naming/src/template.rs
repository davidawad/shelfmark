//! The naming template language.
//!
//! A template is literal text with `{variable}` placeholders; `/` separates
//! folders. `<...>` is an optional group, dropped when any variable inside
//! it has no value (`<` and `>` can never be literal in a path anyway).
//! `{{` and `}}` are literal braces. `{class_path}` expands to one folder
//! per class from the top-level class down, so it must fill its segments
//! alone. Every mistake is caught at parse time, with its byte offset.

use std::fmt;

use serde::Serialize;

/// The variables a template can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Var {
    /// One folder per class, top-level first.
    ClassPath,
    /// The placed class code, e.g. `QA76`.
    Class,
    /// The placed class label.
    ClassLabel,
    /// The Cutter number, without its leading dot.
    Cutter,
    Title,
    Author,
    /// Content type, e.g. `book`, `paper`, `course`.
    Type,
    Year,
    /// File extension without its dot.
    Ext,
    /// The permanent work ID.
    Work,
}

impl Var {
    pub const ALL: [Var; 10] = [
        Var::ClassPath,
        Var::Class,
        Var::ClassLabel,
        Var::Cutter,
        Var::Title,
        Var::Author,
        Var::Type,
        Var::Year,
        Var::Ext,
        Var::Work,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Var::ClassPath => "class_path",
            Var::Class => "class",
            Var::ClassLabel => "class_label",
            Var::Cutter => "cutter",
            Var::Title => "title",
            Var::Author => "author",
            Var::Type => "type",
            Var::Year => "year",
            Var::Ext => "ext",
            Var::Work => "work",
        }
    }

    fn parse(name: &str) -> Option<Var> {
        Var::ALL.into_iter().find(|v| v.as_str() == name)
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A piece of one path segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    Lit(String),
    Var(Var),
    /// Optional: dropped when any variable inside has no value.
    Group(Vec<Part>),
}

/// A parsed template: segments, each a list of parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    source: String,
    segments: Vec<Vec<Part>>,
}

/// Why a template failed to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateError {
    /// Byte offset into the template.
    pub position: usize,
    pub message: String,
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "template error at byte {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for TemplateError {}

/// `{class_path}` must fill a whole folder segment and never the file name.
fn check_class_path(segments: &[Vec<Part>], starts: &[usize]) -> Result<(), TemplateError> {
    let err = |position: usize, message: &str| TemplateError {
        position,
        message: message.into(),
    };
    for (n, (seg, &offset)) in segments.iter().zip(starts).enumerate() {
        let has_path = seg.contains(&Part::Var(Var::ClassPath));
        if has_path && seg.len() > 1 {
            return Err(err(
                offset,
                "`{class_path}` must be a whole folder: put `/` on both sides",
            ));
        }
        if has_path && n + 1 == segments.len() {
            return Err(err(offset, "`{class_path}` cannot be the file name"));
        }
    }
    Ok(())
}

impl Template {
    pub fn parse(source: &str) -> Result<Template, TemplateError> {
        let err = |position: usize, message: String| TemplateError { position, message };
        if source.trim().is_empty() {
            return Err(err(0, "template is empty".into()));
        }
        let mut segments = vec![Vec::new()];
        let mut starts = vec![0];
        let mut group: Option<(usize, Vec<Part>)> = None;
        let mut lit = String::new();
        let mut iter = source.char_indices().peekable();
        while let Some((i, c)) = iter.next() {
            match c {
                '{' if iter.peek().map(|p| p.1) == Some('{') => {
                    iter.next();
                    lit.push('{');
                }
                '}' if iter.peek().map(|p| p.1) == Some('}') => {
                    iter.next();
                    lit.push('}');
                }
                '{' => {
                    let rest = &source[i + 1..];
                    let close = rest
                        .find(['}', '{', '/', '<', '>'])
                        .filter(|&j| rest[j..].starts_with('}'))
                        .ok_or_else(|| {
                            err(i, "unclosed `{`; use `{{` for a literal brace".into())
                        })?;
                    let name = rest[..close].trim();
                    let var = Var::parse(name).ok_or_else(|| {
                        let known: Vec<_> = Var::ALL.iter().map(|v| v.as_str()).collect();
                        err(
                            i,
                            format!("unknown variable `{{{name}}}`; known: {}", known.join(", ")),
                        )
                    })?;
                    if var == Var::ClassPath && group.is_some() {
                        return Err(err(i, "`{class_path}` cannot be optional".into()));
                    }
                    let end = i + 1 + close;
                    while iter.next_if(|&(j, _)| j <= end).is_some() {}
                    flush(&mut lit, &mut group, &mut segments);
                    push(Part::Var(var), &mut group, &mut segments);
                }
                '}' => return Err(err(i, "stray `}`; use `}}` for a literal brace".into())),
                '<' => {
                    if group.is_some() {
                        return Err(err(i, "optional groups `<...>` cannot nest".into()));
                    }
                    flush(&mut lit, &mut group, &mut segments);
                    group = Some((i, Vec::new()));
                }
                '>' => {
                    flush(&mut lit, &mut group, &mut segments);
                    let (_, parts) = group
                        .take()
                        .ok_or_else(|| err(i, "`>` without an opening `<`".into()))?;
                    if !parts.iter().any(|p| matches!(p, Part::Var(_))) {
                        return Err(err(i, "optional group names no variable".into()));
                    }
                    push(Part::Group(parts), &mut group, &mut segments);
                }
                '/' => {
                    if group.is_some() {
                        return Err(err(i, "an optional group cannot span `/`".into()));
                    }
                    flush(&mut lit, &mut group, &mut segments);
                    if segments.last().is_some_and(Vec::is_empty) {
                        return Err(err(i, "empty folder name (leading or doubled `/`)".into()));
                    }
                    segments.push(Vec::new());
                    starts.push(i + 1);
                }
                '\\' => return Err(err(i, "use `/` to separate folders, not `\\`".into())),
                _ => lit.push(c),
            }
        }
        if let Some((start, _)) = group {
            return Err(err(start, "unclosed `<`".into()));
        }
        flush(&mut lit, &mut group, &mut segments);
        if segments.last().is_some_and(Vec::is_empty) {
            return Err(err(
                source.len(),
                "template ends with `/`; it needs a file name".into(),
            ));
        }
        check_class_path(&segments, &starts)?;
        Ok(Template {
            source: source.to_string(),
            segments,
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn segments(&self) -> &[Vec<Part>] {
        &self.segments
    }

    /// Every variable named, in template order.
    pub fn vars(&self) -> Vec<Var> {
        fn walk(parts: &[Part], out: &mut Vec<Var>) {
            for p in parts {
                match p {
                    Part::Var(v) => out.push(*v),
                    Part::Group(g) => walk(g, out),
                    Part::Lit(_) => {}
                }
            }
        }
        let mut out = Vec::new();
        for seg in &self.segments {
            walk(seg, &mut out);
        }
        out
    }
}

fn flush(lit: &mut String, group: &mut Option<(usize, Vec<Part>)>, segments: &mut [Vec<Part>]) {
    if !lit.is_empty() {
        push(Part::Lit(std::mem::take(lit)), group, segments);
    }
}

/// Add a part to the open group, or else to the current segment.
fn push(part: Part, group: &mut Option<(usize, Vec<Part>)>, segments: &mut [Vec<Part>]) {
    match group {
        Some((_, parts)) => parts.push(part),
        None => segments.last_mut().expect("one segment").push(part),
    }
}

impl std::str::FromStr for Template {
    type Err = TemplateError;
    fn from_str(s: &str) -> Result<Self, TemplateError> {
        Template::parse(s)
    }
}
