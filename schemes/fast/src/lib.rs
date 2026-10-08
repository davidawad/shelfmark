//! Builds shelfmark's FAST scheme (`scheme:fast@2024`) from OCLC's FAST
//! topical N-Triples; see `build` for the rules and README.md for use.

pub mod build;
pub mod graph;
pub mod nt;
pub mod report;
pub mod sample;
pub mod sources;

use std::io::{BufReader, Cursor};
use std::path::Path;

use graph::Graph;
use sources::Lock;

/// The three generated files.
pub struct Outputs {
    pub scheme: String,
    pub sample: String,
    pub notes: String,
}

impl Outputs {
    pub const SCHEME: &'static str = "fast.json";
    pub const SAMPLE: &'static str = "fast-sample.json";
    pub const NOTES: &'static str = "SOURCES.md";

    /// Scheme, sample, notes by position.
    pub fn get(&self, i: usize) -> &str {
        [&self.scheme, &self.sample, &self.notes][i]
    }
}

/// Read the locked sources from `cache` and build every output.
pub fn generate(cache: &Path, lock: &Lock) -> Result<Outputs, String> {
    let mut g = Graph::default();
    for l in &lock.sources {
        let mut zip = zip::ZipArchive::new(Cursor::new(sources::read(cache, l)?))
            .map_err(|e| format!("{}: {e}", l.file))?;
        let member = sources::member(&mut zip, l)?;
        g.read_into(BufReader::new(member))
            .map_err(|e| format!("{}: {e}", l.file))?;
    }
    g.finish();
    Ok(from_graph(&g, lock))
}

/// Build every output from an already-read graph.
pub fn from_graph(g: &Graph, lock: &Lock) -> Outputs {
    let (doc, stats) = build::build(g, &report::provenance(lock));
    let scheme = report::scheme_json(&doc);
    let sample_doc = sample::sample(&doc, sample::SEEDS);
    let sample = report::scheme_json(&sample_doc);
    let notes = report::sources_md(lock, &stats, &doc, &scheme, &sample_doc, &sample);
    Outputs {
        scheme,
        sample,
        notes,
    }
}
