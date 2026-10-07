//! Build a gold set from a pinned manifest and raw source snapshots.
//!
//! `gold/fetch.sh` downloads each manifest record once into `gold/raw/`;
//! this module turns those snapshots into gold items offline, labelling each
//! item only from its own source metadata:
//!
//! - `arxiv`: the paper's primary category, mapped through the checked-in
//!   crosswalk (`arxiv-lcc-crosswalk.json`).
//! - `openlibrary`: the class numbers in the edition's `lc_classifications`.
//! - `wikipedia`: the article's Wikidata item's LCC values (property P1149).
//!
//! Re-running over the same snapshots gives byte-identical output.

mod sources;

use crate::error::{read_file, EvalError, Result};
use crate::gold::{Acceptable, GoldItem, GoldSet, Provenance};
use crate::lcc::{class_of_call_number, Class};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

pub use sources::{arxiv_entry, ArxivEntry};

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub name: String,
    #[serde(default)]
    pub crosswalk: Option<String>,
    pub sources: Vec<SourceRef>,
}

#[derive(Debug, Deserialize)]
pub struct SourceRef {
    pub kind: String,
    pub record: String,
}

#[derive(Debug, Deserialize)]
struct Crosswalk {
    version: String,
    categories: BTreeMap<String, Vec<String>>,
}

pub struct Built {
    pub items: Vec<GoldItem>,
    /// Human-readable build notes: counts, skipped values, dropped records.
    pub notes: Vec<String>,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A raw snapshot: its text, path relative to the gold dir, and digest.
struct Raw {
    text: String,
    rel: String,
    sha: String,
    retrieved_at: String,
}

struct Ctx {
    dir: PathBuf,
    retrieved: HashMap<String, String>,
}

impl Ctx {
    fn raw(&self, rel: &str) -> Result<Raw> {
        let path = self.dir.join(rel);
        let text = read_file(
            &path.to_string_lossy(),
            "run `gold/fetch.sh` to download missing snapshots",
        )?;
        Ok(Raw {
            sha: sha256_hex(text.as_bytes()),
            retrieved_at: self.retrieved.get(rel).cloned().unwrap_or_default(),
            rel: rel.to_string(),
            text,
        })
    }
}

fn labelled(codes: impl IntoIterator<Item = (Class, String)>) -> Vec<Acceptable> {
    let mut out: Vec<Acceptable> = Vec::new();
    for (c, from) in codes {
        let code = c.canonical();
        if !out.iter().any(|a| a.code() == code) {
            out.push(Acceptable::Labelled {
                code,
                path: c.path(),
                from: Some(from),
            });
        }
    }
    out
}

fn provenance(
    kind: &str,
    record: &str,
    url: String,
    license: &str,
    rule: String,
    raw: &Raw,
) -> Provenance {
    Provenance {
        kind: kind.into(),
        record: record.into(),
        url,
        retrieved_at: raw.retrieved_at.clone(),
        license: license.into(),
        label_rule: rule,
        raw: raw.rel.clone(),
        raw_sha256: raw.sha.clone(),
    }
}

fn build_arxiv(ctx: &Ctx, cw: &Crosswalk, id: &str) -> Result<GoldItem> {
    let raw = ctx.raw(&format!("raw/arxiv/{id}.xml"))?;
    let e = arxiv_entry(&raw.text).ok_or_else(|| {
        EvalError::new(
            "E_RAW_FORMAT",
            format!("arXiv snapshot for {id} has no entry"),
            "re-run `gold/fetch.sh --refresh`",
        )
    })?;
    let codes = cw.categories.get(&e.primary_category).ok_or_else(|| {
        EvalError::new(
            "E_CROSSWALK_GAP",
            format!("arXiv {id}: primary category `{}` is not in the crosswalk", e.primary_category),
            "add the category to gold/arxiv-lcc-crosswalk.json or drop the record from the manifest",
        )
    })?;
    let mut parsed = Vec::new();
    for c in codes {
        let class = Class::parse(c).map_err(|err| {
            EvalError::new(
                "E_CROSSWALK_CODE",
                err.0,
                "fix gold/arxiv-lcc-crosswalk.json",
            )
        })?;
        parsed.push((class, format!("crosswalk:{}", e.primary_category)));
    }
    let mut identifiers = BTreeMap::from([("arxiv".to_string(), id.to_string())]);
    if let Some(doi) = &e.doi {
        identifiers.insert("doi".into(), doi.clone());
    }
    let rule = format!(
        "arXiv primary category `{}` mapped by arxiv-lcc-crosswalk@{}",
        e.primary_category, cw.version
    );
    Ok(GoldItem {
        id: format!("gold:arxiv:{id}"),
        item: None,
        scheme: "lcc".into(),
        title: e.title,
        text: e.summary,
        identifiers,
        acceptable: labelled(parsed),
        fast: Vec::new(),
        source: Some(provenance(
            "arxiv",
            id,
            format!("https://arxiv.org/abs/{id}"),
            "arXiv metadata CC0 1.0",
            rule,
            &raw,
        )),
    })
}

fn build_openlibrary(ctx: &Ctx, key: &str, notes: &mut Vec<String>) -> Result<GoldItem> {
    let ed_raw = ctx.raw(&format!("raw/openlibrary/{key}.edition.json"))?;
    let wk_raw = ctx.raw(&format!("raw/openlibrary/{key}.work.json"))?;
    let ol = sources::openlibrary(&ed_raw.text, &wk_raw.text).map_err(|e| {
        EvalError::new(
            "E_RAW_FORMAT",
            format!("Open Library {key}: {e}"),
            "re-run `gold/fetch.sh --refresh`",
        )
    })?;
    let mut parsed = Vec::new();
    for call in &ol.lc_classifications {
        match class_of_call_number(call) {
            Some(c) => parsed.push((c, format!("lc_classifications:{}", call.trim()))),
            None if !call.trim().is_empty() => {
                notes.push(format!("openlibrary {key}: unparsed call number `{call}`"))
            }
            None => {}
        }
    }
    let mut identifiers = ol.identifiers;
    identifiers.insert("openlibrary".into(), key.into());
    Ok(GoldItem {
        id: format!("gold:openlibrary:{key}"),
        item: None,
        scheme: "lcc".into(),
        title: ol.title,
        text: ol.description,
        identifiers,
        acceptable: labelled(parsed),
        fast: Vec::new(),
        source: Some(provenance(
            "openlibrary",
            key,
            format!("https://openlibrary.org/books/{key}"),
            "Open Library data, CC0 per openlibrary.org/developers/licensing (unverified for dumps)",
            "class number of each edition `lc_classifications` call number (attached topical cutter kept, book number dropped)".into(),
            &ed_raw,
        )),
    })
}

fn build_wikipedia(ctx: &Ctx, qid: &str, notes: &mut Vec<String>) -> Result<GoldItem> {
    let sum_raw = ctx.raw(&format!("raw/wikipedia/{qid}.summary.json"))?;
    let claims_raw = ctx.raw(&format!("raw/wikipedia/{qid}.claims.json"))?;
    let wp = sources::wikipedia(&sum_raw.text, &claims_raw.text).map_err(|e| {
        EvalError::new(
            "E_RAW_FORMAT",
            format!("Wikipedia {qid}: {e}"),
            "re-run `gold/fetch.sh --refresh`",
        )
    })?;
    let mut parsed = Vec::new();
    for v in &wp.lcc {
        match Class::parse(v) {
            Ok(c) => parsed.push((c, format!("wikidata:P1149:{v}"))),
            Err(e) => notes.push(format!("wikipedia {qid}: skipped P1149 value: {e}")),
        }
    }
    let identifiers = BTreeMap::from([
        ("wikidata".to_string(), qid.to_string()),
        ("enwiki".to_string(), wp.page.clone()),
        ("enwiki_revision".to_string(), wp.revision.clone()),
    ]);
    Ok(GoldItem {
        id: format!("gold:wikipedia:{qid}"),
        item: None,
        scheme: "lcc".into(),
        title: wp.title,
        text: wp.extract,
        identifiers,
        acceptable: labelled(parsed),
        fast: Vec::new(),
        source: Some(provenance(
            "wikipedia",
            qid,
            format!("https://en.wikipedia.org/wiki/{}", wp.page),
            "extract CC BY-SA 4.0 (Wikipedia contributors); Wikidata claims CC0 1.0",
            "every non-deprecated Wikidata P1149 (Library of Congress Classification) value of the article's item".into(),
            &sum_raw,
        )),
    })
}

/// `raw/retrieved.tsv`: `<relative path>\t<UTC timestamp>` per snapshot.
fn retrieved_index(dir: &Path) -> HashMap<String, String> {
    let text = std::fs::read_to_string(dir.join("raw/retrieved.tsv")).unwrap_or_default();
    text.lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(p, t)| (p.to_string(), t.trim().to_string()))
        .collect()
}

pub fn build(manifest_path: &str) -> Result<Built> {
    let next = "pass the manifest with --manifest (see gold/manifest.json)";
    let m: Manifest = serde_json::from_str(&read_file(manifest_path, next)?)
        .map_err(|e| EvalError::new("E_MANIFEST", format!("{manifest_path}: {e}"), next))?;
    let dir = Path::new(manifest_path)
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();
    let cw_path = dir.join(m.crosswalk.as_deref().unwrap_or("arxiv-lcc-crosswalk.json"));
    let ctx = Ctx {
        retrieved: retrieved_index(&dir),
        dir,
    };
    let mut cw: Option<Crosswalk> = None;
    let mut items = Vec::new();
    let mut notes = Vec::new();
    for s in &m.sources {
        let item = match s.kind.as_str() {
            "arxiv" => {
                if cw.is_none() {
                    let p = cw_path.to_string_lossy().to_string();
                    let text = read_file(&p, "restore gold/arxiv-lcc-crosswalk.json")?;
                    cw = Some(serde_json::from_str(&text).map_err(|e| {
                        EvalError::new("E_CROSSWALK", format!("{p}: {e}"), "fix the crosswalk JSON")
                    })?);
                }
                build_arxiv(&ctx, cw.as_ref().expect("loaded above"), &s.record)?
            }
            "openlibrary" => build_openlibrary(&ctx, &s.record, &mut notes)?,
            "wikipedia" => build_wikipedia(&ctx, &s.record, &mut notes)?,
            other => {
                return Err(EvalError::new(
                    "E_MANIFEST",
                    format!("unknown source kind `{other}`"),
                    "use arxiv, openlibrary or wikipedia",
                ))
            }
        };
        if item.acceptable.is_empty() {
            notes.push(format!("{}: no usable LCC label, dropped", item.id));
            continue;
        }
        items.push(item);
    }
    // Validate exactly as a consumer would.
    GoldSet::from_items(items.clone())?;
    notes.insert(0, format!("built `{}`: {} items", m.name, items.len()));
    Ok(Built { items, notes })
}
