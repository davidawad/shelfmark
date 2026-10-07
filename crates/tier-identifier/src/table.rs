//! Identifier tables (plan PD5).
//!
//! A table is a JSONL file, one `{"id":"isbn:9780262033848","class":"QA76.6"}`
//! per line; other fields on a row are ignored, blank lines are skipped.
//! Tables are loaded at run time from a file or a directory of `*.jsonl`
//! files and never compiled into the binary (size and license). Row IDs
//! go through the same normalization as extracted identifiers, so
//! `isbn:0-262-03384-4` matches an ISBN-13 found in text.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use shelfmark_core::error::{Error, ErrorCode};
use shelfmark_core::id::sha256_hex;
use shelfmark_core::tier::Identifier;

use crate::normalize::{self, KeyError};

/// One table row that names a class for an identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Class code as the table wrote it; the scheme decides what it means.
    pub class: String,
    /// File name of the table the row came from.
    pub table: String,
    /// 1-based line number in that table.
    pub line: usize,
}

impl Row {
    /// `table.jsonl:12`, the row reference evidence carries.
    pub fn reference(&self) -> String {
        format!("{}:{}", self.table, self.line)
    }
}

/// A table that failed to load. Line errors carry the 1-based line.
#[derive(Debug, thiserror::Error)]
pub enum TableError {
    #[error("cannot read identifier table {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{table}:{line}: {reason}")]
    Line {
        table: String,
        line: usize,
        reason: String,
    },
}

impl From<TableError> for Error {
    fn from(e: TableError) -> Self {
        let details = match &e {
            TableError::Io { path, .. } => {
                serde_json::json!({ "path": path.display().to_string() })
            }
            TableError::Line { table, line, .. } => {
                serde_json::json!({ "table": table, "line": line })
            }
        };
        Error::new(ErrorCode::StoreError, e.to_string()).details(details)
    }
}

#[derive(Deserialize)]
struct RawRow {
    id: String,
    class: String,
}

/// Every loaded row, keyed by canonical identifier.
#[derive(Debug, Clone, Default)]
pub struct Tables {
    rows: BTreeMap<Identifier, Vec<Row>>,
    /// (table name, sha256 of its bytes), in load order.
    sources: Vec<(String, String)>,
}

impl Tables {
    pub fn new() -> Self {
        Tables::default()
    }

    /// Load a single table file, or every `*.jsonl` file in a directory
    /// (sorted by name, not recursive).
    pub fn load(path: &Path) -> Result<Self, TableError> {
        let mut t = Tables::new();
        if path.is_dir() {
            t.load_dir(path)?;
        } else {
            t.load_file(path)?;
        }
        Ok(t)
    }

    /// Add every `*.jsonl` file in `dir`, sorted by file name. Returns how
    /// many files were loaded; an empty directory is not an error.
    pub fn load_dir(&mut self, dir: &Path) -> Result<usize, TableError> {
        let io = |source| TableError::Io {
            path: dir.to_path_buf(),
            source,
        };
        let mut files = Vec::new();
        for entry in std::fs::read_dir(dir).map_err(io)? {
            let path = entry.map_err(io)?.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "jsonl") {
                files.push(path);
            }
        }
        files.sort();
        for f in &files {
            self.load_file(f)?;
        }
        Ok(files.len())
    }

    /// Add one table file; it is named by its file name in evidence.
    pub fn load_file(&mut self, path: &Path) -> Result<(), TableError> {
        let text = std::fs::read_to_string(path).map_err(|source| TableError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        self.load_str(&name, &text)
    }

    /// Add a table from its text. All-or-nothing: on a bad line nothing
    /// from this table is kept.
    pub fn load_str(&mut self, name: &str, text: &str) -> Result<(), TableError> {
        let mut parsed = Vec::new();
        for (i, raw) in text.lines().enumerate() {
            let line = i + 1;
            if raw.trim().is_empty() {
                continue;
            }
            let err = |reason: String| TableError::Line {
                table: name.to_string(),
                line,
                reason,
            };
            let row: RawRow = serde_json::from_str(raw)
                .map_err(|e| err(format!("not a {{\"id\",\"class\"}} JSON object: {e}")))?;
            let id = normalize::parse_key(row.id.trim()).map_err(|e| match e {
                KeyError::UnknownType(t) => err(format!(
                    "id type {t:?} is not one of isbn, lccn, doi, arxiv, domain"
                )),
                KeyError::InvalidValue(k) => err(format!("id {k:?} is not a valid identifier")),
            })?;
            let class = row.class.trim();
            if class.is_empty() {
                return Err(err("class is empty".into()));
            }
            parsed.push((
                id,
                Row {
                    class: class.to_string(),
                    table: name.to_string(),
                    line,
                },
            ));
        }
        for (id, row) in parsed {
            let rows = self.rows.entry(id).or_default();
            if !rows.iter().any(|r| r.class == row.class) {
                rows.push(row);
            }
        }
        self.sources
            .push((name.to_string(), sha256_hex(text.as_bytes())));
        Ok(())
    }

    /// Rows for a canonical identifier. A URL domain falls back to its
    /// parent domains, most specific first; the identifier returned is the
    /// one that matched.
    pub fn lookup(&self, id: &Identifier) -> Option<(Identifier, &[Row])> {
        if let Identifier::UrlDomain(host) = id {
            return normalize::domain_suffixes(host).into_iter().find_map(|d| {
                let key = Identifier::UrlDomain(d);
                let rows = self.rows.get(&key)?;
                Some((key, rows.as_slice()))
            });
        }
        self.rows.get(id).map(|rows| (id.clone(), rows.as_slice()))
    }

    /// Number of distinct identifiers with at least one row.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Hash over every loaded table's name and bytes, in load order. Goes
    /// into the caller's config hash so a changed table makes cached
    /// placements stale.
    pub fn content_hash(&self) -> String {
        let joined: String = self
            .sources
            .iter()
            .map(|(name, hash)| format!("{name}\t{hash}\n"))
            .collect();
        format!("s256:{}", sha256_hex(joined.as_bytes()))
    }
}
