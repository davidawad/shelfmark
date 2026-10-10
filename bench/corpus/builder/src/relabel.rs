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
