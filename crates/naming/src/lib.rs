//! Naming for shelfmark placements (plan U8, R22): a template turns a
//! placed work into a folder path and file name, with Cutter ordering
//! inside each class.
//!
//! The result is a preview path only. This crate never touches the file
//! system; filing and moving belong to the kb resource (preview then
//! commit). The same template, options, index and work always give the
//! same path.
//!
//! ```
//! use shelfmark_core::testkit::MemIndex;
//! use shelfmark_naming::{Naming, NamingOptions, Work};
//!
//! let index = MemIndex::sample();
//! let naming = Naming::new(
//!     "{class_path}/{cutter} {title}< [{type}]>.{ext}",
//!     NamingOptions::default(),
//! )
//! .unwrap();
//! let mut work = Work::new(index.class("QA76"), "Compilers");
//! work.author = Some("Aho, Alfred V.".into());
//! work.ext = Some("pdf".into());
//! let path = naming.preview(&index, &work).unwrap();
//! assert_eq!(path.to_string(), "QA Mathematics/QA76 Computer software/A36 Compilers.pdf");
//! ```

pub mod cutter;
pub mod sanitize;
pub mod template;

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use shelfmark_core::id::{ClassId, WorkId};
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::{Error, ErrorCode};

pub use cutter::{cutter, Cutter};
pub use template::{Part, Template, TemplateError, Var};

/// The template kb uses when its policy names none.
pub const DEFAULT_TEMPLATE: &str = "{class_path}/{cutter} {title}< [{type}]>.{ext}";

/// How `{class_path}` names each class folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassPathStyle {
    /// `QA76`
    Code,
    /// `QA76 Computer software`
    #[default]
    CodeLabel,
}

/// What the Cutter is taken from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutterSource {
    /// The author's surname, or the title when there is no author (LC
    /// practice: the main entry).
    #[default]
    AuthorThenTitle,
    /// Always the title, leading article skipped.
    Title,
}

/// Naming options; kb carries these as data next to the template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NamingOptions {
    pub class_path: ClassPathStyle,
    pub cutter_from: CutterSource,
    /// Digits after the Cutter letter; LC uses 2.
    pub cutter_digits: usize,
    /// Longest segment in UTF-8 bytes. 255 is the common file-system
    /// limit; the default leaves room under Windows' 260-character path.
    pub max_segment_bytes: usize,
}

impl Default for NamingOptions {
    fn default() -> Self {
        NamingOptions {
            class_path: ClassPathStyle::CodeLabel,
            cutter_from: CutterSource::AuthorThenTitle,
            cutter_digits: 2,
            max_segment_bytes: 120,
        }
    }
}

/// The facts about a work that a template can use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Work {
    /// The class it is placed in (primary scheme).
    pub class: ClassId,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Content type, e.g. `book`; it goes in the name, never the folder.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    /// File extension, with or without its dot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ext: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work: Option<WorkId>,
}

impl Work {
    pub fn new(class: ClassId, title: impl Into<String>) -> Self {
        Work {
            class,
            title: title.into(),
            author: None,
            content_type: None,
            year: None,
            ext: None,
            work: None,
        }
    }
}

/// A proposed relative path. Nothing has been created or moved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreviewPath {
    /// Folders then file name, each already sanitized.
    pub segments: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cutter: Option<Cutter>,
}

impl PreviewPath {
    pub fn to_path_buf(&self) -> PathBuf {
        self.segments.iter().collect()
    }
}

/// Always `/`-separated, whatever the platform.
impl fmt::Display for PreviewPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.segments.join("/"))
    }
}

/// A parsed template plus its options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Naming {
    template: Template,
    options: NamingOptions,
}

impl Naming {
    /// Parse `template`; unknown variables and bad syntax fail here, not
    /// at render time.
    pub fn new(template: &str, options: NamingOptions) -> Result<Naming, Error> {
        let template = Template::parse(template).map_err(|e| template_error(template, &e))?;
        Ok(Naming { template, options })
    }

    pub fn template(&self) -> &Template {
        &self.template
    }

    pub fn options(&self) -> &NamingOptions {
        &self.options
    }

    /// The path `work` would get under this template.
    pub fn preview(&self, index: &dyn CompiledIndex, work: &Work) -> Result<PreviewPath, Error> {
        if &work.class.scheme != index.scheme() {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                format!(
                    "{} is not in scheme {}, the scheme this index holds",
                    work.class,
                    index.scheme()
                ),
            ));
        }
        let mut ancestors = index.ancestors(&work.class.code);
        if ancestors.is_empty() {
            return Err(Error::new(
                ErrorCode::NotFound,
                format!("{} is not a class in {}", work.class, index.scheme()),
            )
            .next(format!("shelfmark scheme search {} <text>", index.scheme())));
        }
        ancestors.reverse();
        let opts = &self.options;
        let cutter = cutter::cutter(&self.cutter_key(work), opts.cutter_digits);
        let own = &ancestors[ancestors.len() - 1];
        let values = Values {
            class: own.id.code.clone(),
            class_label: own.label.clone(),
            cutter: cutter.as_ref().map(ToString::to_string),
            title: some(&work.title),
            author: work.author.as_deref().and_then(some),
            content_type: work.content_type.as_deref().and_then(some),
            year: work.year.map(|y| y.to_string()),
            ext: work
                .ext
                .as_deref()
                .map(|e| e.trim_start_matches('.'))
                .and_then(some),
            work: work.work.as_ref().map(ToString::to_string),
        };
        let last = self.template.segments().len() - 1;
        let mut segments = Vec::new();
        for (n, seg) in self.template.segments().iter().enumerate() {
            if seg == &[Part::Var(Var::ClassPath)] {
                for rec in &ancestors {
                    let name = match opts.class_path {
                        ClassPathStyle::Code => rec.id.code.clone(),
                        ClassPathStyle::CodeLabel => format!("{} {}", rec.id.code, rec.label),
                    };
                    segments.push(sanitize::segment(&name, opts.max_segment_bytes, false));
                }
                continue;
            }
            let raw = render(seg, &values)?;
            segments.push(sanitize::segment(&raw, opts.max_segment_bytes, n == last));
        }
        Ok(PreviewPath { segments, cutter })
    }

    fn cutter_key(&self, work: &Work) -> String {
        let author = work
            .author
            .as_deref()
            .map(str::trim)
            .filter(|a| !a.is_empty());
        match (self.options.cutter_from, author) {
            (CutterSource::AuthorThenTitle, Some(a)) => surname(a).to_string(),
            _ => skip_article(&work.title).to_string(),
        }
    }
}

struct Values {
    class: String,
    class_label: String,
    cutter: Option<String>,
    title: Option<String>,
    author: Option<String>,
    content_type: Option<String>,
    year: Option<String>,
    ext: Option<String>,
    work: Option<String>,
}

impl Values {
    fn get(&self, var: Var) -> Option<&str> {
        match var {
            Var::ClassPath => None,
            Var::Class => Some(&self.class),
            Var::ClassLabel => Some(&self.class_label),
            Var::Cutter => self.cutter.as_deref(),
            Var::Title => self.title.as_deref(),
            Var::Author => self.author.as_deref(),
            Var::Type => self.content_type.as_deref(),
            Var::Year => self.year.as_deref(),
            Var::Ext => self.ext.as_deref(),
            Var::Work => self.work.as_deref(),
        }
    }
}

/// Render one segment. A group whose variables lack a value is dropped;
/// a bare variable without one is an error.
fn render(parts: &[Part], values: &Values) -> Result<String, Error> {
    let mut out = String::new();
    for part in parts {
        match part {
            Part::Lit(s) => out.push_str(s),
            Part::Var(v) => match values.get(*v) {
                Some(s) => out.push_str(s),
                None => {
                    return Err(Error::new(
                        ErrorCode::InvalidRequest,
                        format!("the naming template needs {{{v}}} but this work has none"),
                    )
                    .details(serde_json::json!({ "variable": v.as_str() }))
                    .next(format!(
                        "wrap it in an optional group, e.g. `<[{{{v}}}]>`, or supply {v}"
                    )))
                }
            },
            Part::Group(inner) => {
                if let Ok(s) = render(inner, values) {
                    out.push_str(&s);
                }
            }
        }
    }
    Ok(out)
}

fn some(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

/// `Knuth, Donald E.` -> `Knuth`; `Donald Knuth` -> `Knuth`.
fn surname(author: &str) -> &str {
    match author.split_once(',') {
        Some((last, _)) => last.trim(),
        None => author.split_whitespace().last().unwrap_or(author),
    }
}

/// Skip a leading English article, as LC filing does (G 100).
fn skip_article(title: &str) -> &str {
    let t = title.trim_start();
    for article in ["the ", "an ", "a "] {
        if t.len() > article.len()
            && t.is_char_boundary(article.len())
            && t[..article.len()].eq_ignore_ascii_case(article)
        {
            return t[article.len()..].trim_start();
        }
    }
    t
}

fn template_error(source: &str, e: &TemplateError) -> Error {
    let known: Vec<_> = Var::ALL.iter().map(|v| v.as_str()).collect();
    Error::new(ErrorCode::InvalidRequest, e.to_string())
        .details(serde_json::json!({
            "template": source,
            "position": e.position,
            "variables": known,
        }))
        .next("shelfmark describe name")
}
