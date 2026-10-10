//! Rewrite ACM classes of an existing corpus from re-harvested arXivOld records.

use crate::oai::{self, Format};
use crate::Paper;
use std::collections::BTreeMap;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub rows: usize,
    pub acm_rows_before: usize,
    pub acm_values_before: usize,
    pub acm_rows_after: usize,
    pub acm_values_after: usize,
    /// Rows whose MSC classes differ from the arXivOld record's MSC classes.
    pub msc_mismatches: Vec<String>,
}

/// Replace `acm_ccs` on every row; every row ID must have an arXivOld record.
pub fn relabel(rows: &mut [Paper], records: &[(Format, Paper)]) -> Result<Summary, String> {
    let old: BTreeMap<&str, &Paper> = records
        .iter()
        .filter(|(format, _)| *format == Format::Old)
        .map(|(_, p)| (p.id.as_str(), p))
        .collect();
    let mut summary = Summary {
        rows: rows.len(),
        ..Summary::default()
    };
    let mut missing = Vec::new();
    for row in rows.iter_mut() {
        summary.acm_rows_before += usize::from(!row.acm_ccs.is_empty());
        summary.acm_values_before += row.acm_ccs.len();
        let Some(record) = old.get(row.id.as_str()) else {
            missing.push(row.id.clone());
            continue;
        };
        if record.msc != row.msc {
            summary.msc_mismatches.push(row.id.clone());
        }
        oai::set_acm(row, record.acm_ccs.clone());
        summary.acm_rows_after += usize::from(!row.acm_ccs.is_empty());
        summary.acm_values_after += row.acm_ccs.len();
    }
    if !missing.is_empty() {
        return Err(format!(
            "{} corpus IDs lack an arXivOld record, first: {:?}",
            missing.len(),
            &missing[..missing.len().min(10)]
        ));
    }
    Ok(summary)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MscSummary {
    pub rows: usize,
    pub msc_rows_before: usize,
    pub msc_values_before: usize,
    pub msc_rows_after: usize,
    pub msc_values_after: usize,
    /// Dropped tokens and how often each occurred.
    pub dropped: BTreeMap<String, usize>,
}

/// Keep only MSC 2020 codes in `msc` on every row; no other field changes.
///
/// The stored values are the `msc-class` text split at commas, semicolons and
/// whitespace, none of which can occur inside a code, so filtering them gives
/// the same codes as filtering the source text.
pub fn relabel_msc(rows: &mut [Paper]) -> MscSummary {
    let mut summary = MscSummary {
        rows: rows.len(),
        ..MscSummary::default()
    };
    for row in rows.iter_mut() {
        summary.msc_rows_before += usize::from(!row.msc.is_empty());
        summary.msc_values_before += row.msc.len();
        let mut codes: Vec<String> = Vec::new();
        for token in &row.msc {
            let found = oai::msc_codes(token);
            if found.is_empty() {
                *summary.dropped.entry(token.clone()).or_default() += 1;
            }
            codes.extend(
                found
                    .into_iter()
                    .filter(|c| !codes.contains(c))
                    .collect::<Vec<_>>(),
            );
        }
        oai::set_msc(row, codes);
        summary.msc_rows_after += usize::from(!row.msc.is_empty());
        summary.msc_values_after += row.msc.len();
    }
    summary
}
