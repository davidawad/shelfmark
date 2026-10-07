//! What backs the service: the real crates (plan U13).
//!
//! Schemes compile through `shelfmark-scheme` (U1). The tiers are U3's
//! identifier lookup and U4's vocabulary and rules tiers, the profiles
//! are U5's built-ins, and naming is U8's template with Cutter ordering.
//! The store directory ([`dir_store::DirStore`]) is U6's placement log
//! plus U7's identity links. There is no eval: the harness (U10) lives
//! outside this workspace for now.
//!
//! Per-store data the tiers read (plan PD5), both optional:
//! `<store>/tables/*.jsonl` identifier tables and `<store>/rules.json`.

pub mod dir_store;
mod tiers;

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use shelfmark_core::calibration::{EvalSummary, GoldExample};
use shelfmark_core::scheme::{CompiledIndex, SchemeDoc};
use shelfmark_core::tier::{Profile, Tier};
use shelfmark_core::{ClassId, Error, SchemeId};
use shelfmark_naming::{Naming, NamingOptions, Work, DEFAULT_TEMPLATE};

use crate::service::Shelfmark;

pub use tiers::{parse_identifier, url_domain, RULES_FILE, TABLES_DIR};

pub type Compiler = fn(&SchemeDoc) -> Result<Arc<dyn CompiledIndex>, Error>;
/// Folder path and filename for a class, a title and an extension (R22).
pub type Namer = fn(&dyn CompiledIndex, &ClassId, &str, &str) -> Result<String, Error>;
pub type Evaluator = fn(&[GoldExample], &[Profile]) -> Result<EvalSummary, Error>;

pub struct Wiring {
    /// Schemes compiled into the binary.
    pub schemes: Vec<Arc<dyn CompiledIndex>>,
    /// Schemes the tool knows of but has not compiled, with the file
    /// that defines each when known (AE7).
    pub known: Vec<(SchemeId, Option<String>)>,
    pub compile: Compiler,
    pub profiles: Vec<Profile>,
    /// Tier implementations by name; a profile tier missing here is
    /// unavailable on this build.
    pub tiers: BTreeMap<String, Arc<dyn Tier>>,
    pub name: Namer,
    pub eval: Option<Evaluator>,
}

/// The bundled scheme until the LCC outline (U2) lands.
const MINI_LCC: &str = include_str!("../../../../fixtures/schemes/mini-lcc.json");

/// A service over the real crates, with its store in `dir`.
pub fn open(dir: &Path) -> Shelfmark {
    Shelfmark::new(wiring(dir), Box::new(dir_store::DirStore::new(dir)))
}

/// The wiring for a store in `dir`: its tables and rules feed the tiers.
pub fn wiring(dir: &Path) -> Wiring {
    let doc: SchemeDoc = serde_json::from_str(MINI_LCC).expect("bundled scheme parses");
    Wiring {
        schemes: vec![compile(&doc).expect("bundled scheme compiles")],
        known: vec![(SchemeId::new("lcc", "2024"), None)],
        compile,
        profiles: shelfmark_profile::builtins(),
        tiers: tiers::registry(dir),
        name: name_path,
        eval: None,
    }
}

/// Full JSON Schema validation and compile (U1). The content hash is over
/// the document's canonical JSON, so a scheme hashes the same whether it
/// came from a file or from the store.
pub fn compile(doc: &SchemeDoc) -> Result<Arc<dyn CompiledIndex>, Error> {
    let bytes = serde_json::to_vec(doc).map_err(|e| Error::internal(e.to_string()))?;
    let index = shelfmark_scheme::compile_bytes(&bytes, &doc.id().to_string())?;
    Ok(Arc::new(index))
}

/// U8's default template: `QA Mathematics/QA76 Computer software/A36 Compilers.pdf`.
pub fn name_path(
    index: &dyn CompiledIndex,
    class: &ClassId,
    title: &str,
    ext: &str,
) -> Result<String, Error> {
    let naming = Naming::new(DEFAULT_TEMPLATE, NamingOptions::default())?;
    let mut work = Work::new(class.clone(), title);
    work.ext = Some(ext.to_string());
    Ok(naming.preview(index, &work)?.to_string())
}
