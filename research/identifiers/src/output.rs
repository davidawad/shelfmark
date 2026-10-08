//! Writing the lookup table, its conflicts and its statistics.
//!
//! Every file is sorted, so identical input always yields identical bytes.

use crate::aggregate::{Scheme, Table};
use crate::edition::Key;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub const LOOKUP_HEADER: &str = "key\tscheme\tprefix\tfull\tsupport";
pub const CONFLICT_HEADER: &str = "key\tscheme\tlevel\teditions\tcandidates";

#[derive(Debug, Default, Serialize)]
pub struct KeyStats {
    /// Distinct valid identifiers anywhere in the dump.
    pub distinct: u64,
    pub with_lcc: u64,
    pub with_ddc: u64,
    pub with_either: u64,
    pub with_both: u64,
    /// Keys with more than one full LC call number (editions, years, volumes).
    pub lcc_multiple_full: u64,
    /// Keys whose LCC candidates disagree on class prefix / subclass letters.
    pub lcc_conflict_prefix: u64,
    pub lcc_conflict_top: u64,
    /// Of `lcc_conflict_top`, keys carried by a single edition: Open Library
    /// merged several libraries' call numbers into one record, so the
    /// disagreement is between source libraries, not between editions.
    pub lcc_conflict_top_single_edition: u64,
    /// Keys whose DDC candidates disagree on section / main class.
    pub ddc_conflict_prefix: u64,
    pub ddc_conflict_top: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct TableStats {
    pub keys: u64,
    pub lookup_rows: u64,
    pub conflict_rows: u64,
    pub distinct_lcc_full: u64,
    pub distinct_lcc_prefix: u64,
    pub distinct_ddc_full: u64,
    pub lookup_bytes: u64,
    pub conflicts_bytes: u64,
}

/// Write `lookup.tsv`, `conflicts.tsv` and `stats.json` into `dir`.
pub fn write_all(table: &mut Table, dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let mut keys: Vec<(String, &Key)> = table.entries.keys().map(|k| (k.render(), k)).collect();
    keys.sort();

    let mut key_stats: BTreeMap<String, KeyStats> = BTreeMap::new();
    for (kind, n) in [
        ("isbn", table.seen_isbn.len()),
        ("lccn", table.seen_lccn.len()),
        ("oclc", table.seen_oclc.len()),
    ] {
        key_stats.entry(kind.to_string()).or_default().distinct = n as u64;
    }

    let mut lookup = BufWriter::with_capacity(1 << 20, File::create(dir.join("lookup.tsv"))?);
    let mut conflicts = BufWriter::with_capacity(1 << 20, File::create(dir.join("conflicts.tsv"))?);
    writeln!(lookup, "{LOOKUP_HEADER}")?;
    writeln!(conflicts, "{CONFLICT_HEADER}")?;
    let mut ts = TableStats {
        keys: keys.len() as u64,
        ..Default::default()
    };

    for (rendered, key) in &keys {
        let ks = key_stats.entry(key.kind().to_string()).or_default();
        let entry = &table.entries[*key];
        let mut rows: Vec<(Scheme, &str, &str, &str, u32)> = entry
            .values
            .iter()
            .map(|(id, support)| {
                let v = &table.values[*id as usize];
                (
                    v.scheme,
                    v.class.prefix.as_str(),
                    v.class.full.as_str(),
                    v.class.top.as_str(),
                    *support,
                )
            })
            .collect();
        rows.sort();
        let (mut has_lcc, mut has_ddc) = (false, false);
        for scheme in [Scheme::Lcc, Scheme::Ddc] {
            let mine: Vec<_> = rows.iter().filter(|r| r.0 == scheme).collect();
            if mine.is_empty() {
                continue;
            }
            match scheme {
                Scheme::Lcc => has_lcc = true,
                Scheme::Ddc => has_ddc = true,
            }
            for (_, prefix, full, _, support) in &mine {
                writeln!(
                    lookup,
                    "{rendered}\t{}\t{prefix}\t{full}\t{support}",
                    scheme.name()
                )?;
                ts.lookup_rows += 1;
            }
            let prefix_conflict = conflict(&mine, |r| r.1);
            let top_conflict = conflict(&mine, |r| r.3);
            for (level, candidates) in [("prefix", &prefix_conflict), ("top", &top_conflict)] {
                if let Some(c) = candidates {
                    writeln!(
                        conflicts,
                        "{rendered}\t{}\t{level}\t{}\t{c}",
                        scheme.name(),
                        entry.editions
                    )?;
                    ts.conflict_rows += 1;
                }
            }
            let (p, t) = (
                prefix_conflict.is_some() as u64,
                top_conflict.is_some() as u64,
            );
            match scheme {
                Scheme::Lcc => {
                    ks.lcc_multiple_full += (mine.len() > 1) as u64;
                    ks.lcc_conflict_prefix += p;
                    ks.lcc_conflict_top += t;
                    ks.lcc_conflict_top_single_edition += t * (entry.editions == 1) as u64;
                }
                Scheme::Ddc => {
                    ks.ddc_conflict_prefix += p;
                    ks.ddc_conflict_top += t;
                }
            }
        }
        ks.with_lcc += has_lcc as u64;
        ks.with_ddc += has_ddc as u64;
        ks.with_either += (has_lcc || has_ddc) as u64;
        ks.with_both += (has_lcc && has_ddc) as u64;
    }
    lookup.flush()?;
    conflicts.flush()?;
    drop((lookup, conflicts));

    let distinct = |scheme: Scheme, f: fn(&crate::normalize::ClassNumber) -> &str| {
        table
            .values
            .iter()
            .filter(|v| v.scheme == scheme)
            .map(|v| f(&v.class))
            .collect::<BTreeSet<_>>()
            .len() as u64
    };
    ts.distinct_lcc_full = distinct(Scheme::Lcc, |c| &c.full);
    ts.distinct_lcc_prefix = distinct(Scheme::Lcc, |c| &c.prefix);
    ts.distinct_ddc_full = distinct(Scheme::Ddc, |c| &c.full);
    ts.lookup_bytes = std::fs::metadata(dir.join("lookup.tsv"))?.len();
    ts.conflicts_bytes = std::fs::metadata(dir.join("conflicts.tsv"))?.len();
    table.stats.keys = key_stats;
    table.stats.table = ts;

    let mut stats = serde_json::to_string_pretty(&table.stats).map_err(io::Error::other)?;
    stats.push('\n');
    std::fs::write(dir.join("stats.json"), stats)
}

type Row<'a> = (Scheme, &'a str, &'a str, &'a str, u32);

/// If the rows disagree at the level picked by `level`, the candidates as
/// `value=support;...`, highest support first (ties in lexical order).
fn conflict<'a>(rows: &[&Row<'a>], level: fn(&Row<'a>) -> &'a str) -> Option<String> {
    let mut support: BTreeMap<&str, u32> = BTreeMap::new();
    for r in rows {
        *support.entry(level(r)).or_default() += r.4;
    }
    if support.len() < 2 {
        return None;
    }
    let mut ranked: Vec<(&str, u32)> = support.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    Some(
        ranked
            .iter()
            .map(|(v, s)| format!("{v}={s}"))
            .collect::<Vec<_>>()
            .join(";"),
    )
}
