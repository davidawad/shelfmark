//! LCSH and FAST vocabulary ingest plus a label matcher prototype for the
//! materials classifier's vocabulary tier (plan R7), and [`enrich`], which
//! folds a heading table into a shelfmark scheme's vocabulary.

pub mod enrich;
pub mod eval;
pub mod fast;
pub mod io;
pub mod lcc;
pub mod lcsh;
pub mod matcher;
pub mod ntriples;
pub mod rdfxml;
pub mod table;
