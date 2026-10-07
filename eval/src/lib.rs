//! `mc-eval`: the materials classifier's evaluation harness (R15-R17).
//!
//! Inputs are a gold set (one item per NDJSON line with the LCC classes a
//! correct placement may land in) and predictions (one line per item and
//! profile). The report gives top-1 and top-k accuracy per hierarchy level,
//! hierarchical accuracy, calibration (ECE, Brier), risk-coverage with
//! measured confidence thresholds, flagged rate and latency, per profile and
//! per deciding tier, each with a seeded percentile bootstrap interval.

pub mod bootstrap;
pub mod describe;
pub mod error;
pub mod gold;
pub mod goldbuild;
pub mod lcc;
pub mod outcome;
pub mod pred;
pub mod render;
pub mod report;
pub mod summary;
