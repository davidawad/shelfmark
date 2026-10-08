//! Builds the MSC2020 scheme file from the official CSV at msc2020.org.
//!
//! MSC2020 is published under CC BY-NC-SA 4.0, which a MIT/Apache binary
//! cannot carry, so shelfmark does not bundle it. This tool fetches the
//! CSV, checks it against `sources.lock.json` and writes a scheme file the
//! user compiles with `shelfmark scheme compile`.

pub mod csv;
pub mod scheme;
pub mod sources;
