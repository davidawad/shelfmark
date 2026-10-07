//! What backs the service: compiled schemes, the scheme compiler, tiers,
//! profiles, naming, the eval and the store.
//!
//! Everything here is a stand-in until the real crates land: schemes
//! compile into the core testkit's `MemIndex` (U1), tiers are the fakes in
//! [`fake_tiers`] (U3, U4), profiles are declared here (U5), the store is
//! [`fake_store::FsStore`] (U6, U7), naming is a plain join (U8), and
//! there is no eval (U10). U13 replaces this module; nothing else in the
//! crate names a fake.

pub mod fake_store;
pub mod fake_tiers;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use shelfmark_core::calibration::{EvalSummary, GoldExample};
use shelfmark_core::scheme::{ClassRecord, CompiledIndex, SchemeDoc};
use shelfmark_core::testkit::MemIndex;
use shelfmark_core::tier::{Profile, Tier};
use shelfmark_core::{Error, ErrorCode, ProfileId, SchemeId};

use crate::service::Shelfmark;

pub type Compiler = fn(&SchemeDoc) -> Result<Arc<dyn CompiledIndex>, Error>;
/// Folder path and filename for a class (ancestors, top level first), a
/// title and an extension (R22).
pub type Namer = fn(&[ClassRecord], &str, &str) -> String;
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

/// The bundled test scheme until the LCC outline (U2) lands.
const MINI_LCC: &str = include_str!("../../../../fixtures/schemes/mini-lcc.json");

/// A service over the fakes, with its store in `dir`.
pub fn open(dir: &Path) -> Shelfmark {
    Shelfmark::new(fake(), Box::new(fake_store::FsStore::new(dir)))
}

pub fn fake() -> Wiring {
    let doc: SchemeDoc = serde_json::from_str(MINI_LCC).expect("bundled scheme parses");
    let mut table = BTreeMap::new();
    table.insert("isbn:9780262033848".to_string(), "QA76".to_string());
    let tiers: [(&str, Arc<dyn Tier>); 3] = [
        ("identifier", Arc::new(fake_tiers::IdentifierTier { table })),
        ("vocab", Arc::new(fake_tiers::VocabTier)),
        ("rules", Arc::new(fake_tiers::RulesTier)),
    ];
    Wiring {
        schemes: vec![compile(&doc).expect("bundled scheme compiles")],
        known: vec![(SchemeId::new("lcc", "2024"), None)],
        compile,
        profiles: profiles(),
        tiers: tiers.into_iter().map(|(n, t)| (n.to_string(), t)).collect(),
        name: name_path,
        eval: None,
    }
}

fn profile(name: &str, desc: &str, tiers: &[&str], mem: u32, micros: u64) -> Profile {
    Profile {
        id: ProfileId::new(name, "1"),
        description: desc.into(),
        tiers: tiers.iter().map(|t| (*t).into()).collect(),
        default_target: 0.8,
        min_margin: 0.1,
        expected_memory_mb: mem,
        expected_micros_per_item: micros,
    }
}

/// The three v1 profiles (R8, PD4); U5 owns the real declarations.
pub fn profiles() -> Vec<Profile> {
    let lexical = ["identifier", "vocab", "rules"];
    vec![
        profile(
            "lexical",
            "Identifiers, scheme vocabulary and rules; no model",
            &lexical,
            64,
            2_000,
        ),
        profile(
            "shortlist",
            "Lexical, then an embedding shortlist",
            &[&lexical[..], &["shortlist"]].concat(),
            1_200,
            40_000,
        ),
        profile(
            "rerank",
            "Lexical, embedding shortlist, then a cross-encoder rerank",
            &[&lexical[..], &["shortlist", "rerank"]].concat(),
            2_400,
            250_000,
        ),
    ]
}

/// Structural checks only: codes unique and non-empty, parents defined.
/// U1 adds JSON Schema validation and the code grammar.
pub fn compile(doc: &SchemeDoc) -> Result<Arc<dyn CompiledIndex>, Error> {
    let mut seen = BTreeSet::new();
    let mut problems = Vec::new();
    for c in &doc.classes {
        if c.code.trim().is_empty() {
            problems.push(format!("class `{}` has an empty code", c.label));
        } else if !seen.insert(c.code.as_str()) {
            problems.push(format!("code `{}` is defined twice", c.code));
        }
    }
    for c in &doc.classes {
        if let Some(p) = &c.parent {
            if !seen.contains(p.as_str()) {
                problems.push(format!("`{}` names undefined parent `{p}`", c.code));
            }
        }
    }
    if !problems.is_empty() {
        return Err(Error::new(
            ErrorCode::SchemeInvalid,
            format!("{} does not validate: {}", doc.id(), problems.join("; ")),
        )
        .details(serde_json::json!({ "problems": problems })));
    }
    Ok(Arc::new(MemIndex::new(doc)))
}

/// `QA/QA76/<title>.<ext>`, with path separators and reserved characters
/// in the title replaced.
pub fn name_path(ancestors: &[ClassRecord], title: &str, ext: &str) -> String {
    let clean: String = title
        .chars()
        .map(|c| {
            if "/\\:*?\"<>|".contains(c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let mut parts: Vec<String> = ancestors.iter().map(|r| r.id.code.clone()).collect();
    parts.push(format!("{}.{ext}", clean.trim()));
    parts.join("/")
}
