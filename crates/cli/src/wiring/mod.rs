//! What backs the service: the real crates (plan U13).
//!
//! Schemes compile through `shelfmark-scheme` (U1), and the bundled scheme
//! is the LCC outline (U2, `data/schemes/lcc/lcc.json`) with every LCSH
//! heading that carries an LCC number as vocabulary
//! (`data/schemes/lcc/lcc-lcsh.json.gz`, see `LCSH.md` there). The tiers are U3's
//! identifier lookup and U4's vocabulary and rules tiers, the profiles
//! are U5's built-ins, and naming is U8's template with Cutter ordering.
//! The store directory ([`dir_store::DirStore`]) is U6's placement log
//! plus U7's identity links. The eval (U10) is `shelfmark-eval`, called
//! by the service directly.
//!
//! Per-store data the tiers read (plan PD5), both optional:
//! `<store>/tables/*.jsonl` identifier tables and `<store>/rules.json`
//! (which replaces the bundled `data/rules/starter.json`).
//!
//! `<store>/cache/` holds derived data only, safe to delete: compiled
//! schemes (`schemes/`, U1's compile cache keyed by content hash), built
//! vocab matchers (`vocab/`) and shortlist label vectors (`shortlist/`).
//! It is used once the store directory exists, so a process that only
//! reads an absent store writes nothing. Model files live outside the
//! store, in the user's cache (`shelfmark_tier_shortlist::model_dir`).

pub mod dir_store;
mod tiers;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use shelfmark_core::scheme::{CompiledIndex, SchemeDoc};
use shelfmark_core::tier::{Profile, Tier};
use shelfmark_core::{ClassId, Error, ErrorCode, SchemeId};
use shelfmark_naming::{Naming, NamingOptions, Work, DEFAULT_TEMPLATE};

use crate::service::Shelfmark;

pub use tiers::{parse_identifier, starter_rules, url_domain, RULES_FILE, TABLES_DIR};

pub type Compiler = Arc<dyn Fn(&SchemeDoc) -> Result<Arc<dyn CompiledIndex>, Error> + Send + Sync>;
/// Folder path and filename for a class, a title and an extension (R22).
pub type Namer = fn(&dyn CompiledIndex, &ClassId, &str, &str) -> Result<String, Error>;

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
}

/// The bundled scheme: the LCC outline with LCSH vocabulary, as the
/// gzipped canonical JSON (`serde_json::to_vec` of the document) of
/// `data/schemes/lcc/lcc-lcsh.json.gz`, so it hashes the same as the same
/// document compiled into a store.
const LCC_GZ: &[u8] = include_bytes!("../../../../data/schemes/lcc/lcc-lcsh.json.gz");

/// Inside a store, the directory of derived data.
pub const CACHE_DIR: &str = "cache";

/// The bundled scheme's canonical JSON.
pub fn bundled_lcc_bytes() -> Vec<u8> {
    let mut out = Vec::new();
    std::io::Read::read_to_end(&mut flate2::read::GzDecoder::new(LCC_GZ), &mut out)
        .expect("bundled scheme decompresses");
    out
}

/// The bundled LCC scheme, compiled once per process.
pub fn bundled_lcc() -> Arc<dyn CompiledIndex> {
    bundled_lcc_in(None)
}

/// The bundled scheme, through the compile cache in `cache` when given,
/// so only the first process validates and compiles it.
fn bundled_lcc_in(cache: Option<&Path>) -> Arc<dyn CompiledIndex> {
    static INDEX: OnceLock<Arc<dyn CompiledIndex>> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            let source = SchemeId::new("lcc", "2024").to_string();
            compile_bytes(&bundled_lcc_bytes(), &source, cache).expect("bundled scheme compiles")
        })
        .clone()
}

/// `<store>/cache`, once the store directory exists.
fn cache_dir(store: &Path) -> Option<PathBuf> {
    store.is_dir().then(|| store.join(CACHE_DIR))
}

/// A service over the real crates, with its store in `dir`.
pub fn open(dir: &Path) -> Shelfmark {
    Shelfmark::new(wiring(dir), Box::new(dir_store::DirStore::new(dir)))
}

/// The wiring for a store in `dir`: its tables and rules feed the tiers,
/// and its cache keeps compiled schemes and vocab matchers.
pub fn wiring(dir: &Path) -> Wiring {
    let cache = cache_dir(dir);
    let schemes = cache.as_ref().map(|c| c.join("schemes"));
    Wiring {
        schemes: vec![bundled_lcc_in(schemes.as_deref())],
        // Built or fetched, not bundled (R5): each scheme's tool under
        // `schemes/<name>/` writes the file a user compiles.
        known: vec![
            // FAST (R5), 79 MB: `fast-scheme build`.
            (
                SchemeId::new("fast", "2024"),
                Some("schemes/fast/out/fast.json".into()),
            ),
            // CPC: its terms keep it out of the binary.
            (
                SchemeId::new("cpc", "2026.08"),
                Some("schemes/cpc/cpc.json".into()),
            ),
            // MSC2020 is CC BY-NC-SA: `schemes/msc/tool` builds the file a user compiles.
            (SchemeId::new("msc", "2020"), None),
            // JEL may not be redistributed: `jel-scheme build`.
            (
                SchemeId::new("jel", "2026"),
                Some("schemes/jel/jel.json".into()),
            ),
            // arXiv ships as a data file to compile.
            (
                SchemeId::new("arxiv", "2024-04-01"),
                Some("schemes/arxiv/arxiv.json".into()),
            ),
            // NAICS 2022 loads from its data file.
            (
                SchemeId::new("naics", "2022"),
                Some("schemes/naics/naics.json".into()),
            ),
            // OpenAlex Topics (CC0), 13 MB: loads from its data file.
            (
                SchemeId::new("openalex-topics", "2026-10-09"),
                Some("schemes/openalex-topics/openalex-topics.json".into()),
            ),
            // FDC (CC0) loads from its data file.
            (
                SchemeId::new("fdc", "0.06"),
                Some("schemes/fdc/fdc.json".into()),
            ),
        ],
        compile: compiler(schemes),
        profiles: shelfmark_profile::builtins(),
        tiers: tiers::registry(dir, cache),
        name: name_path,
    }
}

/// Full JSON Schema validation and compile (U1), reusing the compile
/// cache in `cache` when given. The content hash is over the document's
/// canonical JSON, so a scheme hashes the same whether it came from a
/// file or from the store.
pub fn compiler(cache: Option<PathBuf>) -> Compiler {
    Arc::new(move |doc: &SchemeDoc| {
        let bytes = serde_json::to_vec(doc).map_err(|e| Error::internal(e.to_string()))?;
        compile_bytes(&bytes, &doc.id().to_string(), cache.as_deref())
    })
}

/// Compile canonical scheme bytes; a cache that cannot be written is
/// skipped, never an error.
fn compile_bytes(
    bytes: &[u8],
    source: &str,
    cache: Option<&Path>,
) -> Result<Arc<dyn CompiledIndex>, Error> {
    if let Some(dir) = cache {
        match shelfmark_scheme::CompileCache::new(dir).compile_object(bytes, source) {
            Ok(compiled) => return Ok(Arc::new(compiled.index)),
            Err(e) if e.code != ErrorCode::StoreError => return Err(e),
            Err(_) => {}
        }
    }
    Ok(Arc::new(shelfmark_scheme::compile_bytes(bytes, source)?))
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
