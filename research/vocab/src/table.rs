//! The compact heading table both ingesters emit.
//!
//! One TSV row per heading, sorted by `id`, list cells sorted and joined with
//! `|`. Backslash escapes `\\`, `\t`, `\n` and `\|` keep cells unambiguous, so
//! the same input always produces byte-identical output.

use std::io::{self, BufRead, Write};

pub const HEADER: &str =
    "id\tvocab\tkind\tpref_label\talt_labels\tbroader\tlcc\tlcc_source\tsame_as\tdeprecated";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Heading {
    /// `sh86007588` for LCSH, `fst01001522` for FAST.
    pub id: String,
    /// `lcsh` or `fast`.
    pub vocab: String,
    /// MADS type (Topic, ComplexSubject, ...) or FAST facet.
    pub kind: String,
    pub pref_label: String,
    pub alt_labels: Vec<String>,
    /// Broader heading ids in the same vocabulary.
    pub broader: Vec<String>,
    /// Canonical LCC numbers or ranges (see `lcc::LccRef::cell`).
    pub lcc: Vec<String>,
    /// `direct` (on the record), `lcsh` (joined through `same_as`) or empty.
    pub lcc_source: String,
    /// Cross-vocabulary links; for FAST, the LCSH ids it was derived from.
    pub same_as: Vec<String>,
    pub deprecated: bool,
}

impl Heading {
    /// Sort and dedupe every list so output never depends on input order.
    pub fn canonicalize(&mut self) {
        for list in [
            &mut self.alt_labels,
            &mut self.broader,
            &mut self.lcc,
            &mut self.same_as,
        ] {
            list.sort();
            list.dedup();
        }
        let pref = self.pref_label.clone();
        self.alt_labels.retain(|a| *a != pref);
    }

    pub fn write_row(&self, out: &mut impl Write) -> io::Result<()> {
        let cells = [
            esc(&self.id),
            esc(&self.vocab),
            esc(&self.kind),
            esc(&self.pref_label),
            join(&self.alt_labels),
            join(&self.broader),
            join(&self.lcc),
            esc(&self.lcc_source),
            join(&self.same_as),
            (if self.deprecated { "1" } else { "0" }).to_string(),
        ];
        writeln!(out, "{}", cells.join("\t"))
    }

    pub fn parse_row(line: &str) -> Result<Heading, String> {
        let cells: Vec<&str> = line.split('\t').collect();
        if cells.len() != 10 {
            return Err(format!("expected 10 cells, got {}", cells.len()));
        }
        Ok(Heading {
            id: unesc(cells[0]),
            vocab: unesc(cells[1]),
            kind: unesc(cells[2]),
            pref_label: unesc(cells[3]),
            alt_labels: split(cells[4]),
            broader: split(cells[5]),
            lcc: split(cells[6]),
            lcc_source: unesc(cells[7]),
            same_as: split(cells[8]),
            deprecated: cells[9] == "1",
        })
    }
}

pub fn write_table(headings: &[Heading], out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "{HEADER}")?;
    for h in headings {
        h.write_row(out)?;
    }
    Ok(())
}

pub fn read_table(input: impl BufRead) -> Result<Vec<Heading>, String> {
    let mut rows = Vec::new();
    for (n, line) in input.lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        if n == 0 {
            if line != HEADER {
                return Err(format!("unexpected header: {line}"));
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        rows.push(Heading::parse_row(&line).map_err(|e| format!("line {}: {e}", n + 1))?);
    }
    Ok(rows)
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '|' => out.push_str("\\|"),
            c => out.push(c),
        }
    }
    out
}

fn join(items: &[String]) -> String {
    items.iter().map(|s| esc(s)).collect::<Vec<_>>().join("|")
}

fn unesc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// Split a list cell on unescaped `|`.
fn split(cell: &str) -> Vec<String> {
    if cell.is_empty() {
        return Vec::new();
    }
    let mut items = Vec::new();
    let mut cur = String::new();
    let mut chars = cell.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                cur.push('\\');
                if let Some(n) = chars.next() {
                    cur.push(n);
                }
            }
            '|' => items.push(unesc(&std::mem::take(&mut cur))),
            c => cur.push(c),
        }
    }
    items.push(unesc(&cur));
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_awkward_cells() {
        let mut h = Heading {
            id: "sh1".into(),
            vocab: "lcsh".into(),
            kind: "Topic".into(),
            pref_label: "A|B\tC\\D".into(),
            alt_labels: vec!["z".into(), "a|b".into(), "z".into(), "A|B\tC\\D".into()],
            broader: vec![],
            lcc: vec!["QA76".into()],
            lcc_source: "direct".into(),
            same_as: vec![],
            deprecated: true,
        };
        h.canonicalize();
        assert_eq!(h.alt_labels, vec!["a|b".to_string(), "z".to_string()]);
        let mut buf = Vec::new();
        write_table(std::slice::from_ref(&h), &mut buf).unwrap();
        let back = read_table(&buf[..]).unwrap();
        assert_eq!(back, vec![h]);
    }
}
