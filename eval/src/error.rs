//! Errors with a stable code, a plain explanation and the next command that
//! would fix them (R21).

use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EvalError {
    pub code: &'static str,
    pub message: String,
    pub next: String,
}

impl EvalError {
    pub fn new(code: &'static str, message: impl Into<String>, next: impl Into<String>) -> Self {
        EvalError {
            code,
            message: message.into(),
            next: next.into(),
        }
    }

    /// Process exit code: 2 for usage errors, 3 for bad input data, 4 for I/O.
    pub fn exit_code(&self) -> i32 {
        match self.code {
            "E_USAGE" => 2,
            "E_IO" | "E_WRITE" => 4,
            _ => 3,
        }
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}\n  next: {}", self.code, self.message, self.next)
    }
}

impl std::error::Error for EvalError {}

pub type Result<T> = std::result::Result<T, EvalError>;

pub fn read_file(path: &str, next: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| EvalError::new("E_IO", format!("cannot read `{path}`: {e}"), next))
}

/// Non-blank, non-`#` lines of an NDJSON document with 1-based line numbers.
pub fn ndjson_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim()))
        .filter(|(_, l)| !l.is_empty() && !l.starts_with('#'))
}
