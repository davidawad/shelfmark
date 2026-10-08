//! Identifier-tier data for the materials classifier: a deterministic builder
//! that turns the Open Library editions dump into an ISBN/LCCN/OCLC -> LCC
//! and DDC lookup table, plus the arXiv-category and URL-domain rule files.

pub mod aggregate;
pub mod build;
pub mod edition;
pub mod normalize;
pub mod output;
pub mod rules;
pub mod shelfmark;
