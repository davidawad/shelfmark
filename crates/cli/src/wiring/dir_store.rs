//! A store directory (PD6): U6's claim log, gold set and memo index plus
//! U7's item links and MinHash index, side by side and under the one
//! `<store>/.lock`. Compiled schemes, the calibration and the latest eval
//! are plain files next to them.
//!
//! Opening touches nothing; reads of a store that does not exist yet are
//! empty, and the directory appears on the first write.

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use shelfmark_core::calibration::{Calibration, EvalSummary, GoldExample};
use shelfmark_core::id::sha256_hex;
use shelfmark_core::placement::{InputHashes, Placement, PlacementLog};
use shelfmark_core::scheme::SchemeDoc;
use shelfmark_core::{Error, ErrorCode, ItemId, WorkId};
use shelfmark_identity::{Identity, Ingested, Link, Minted, LINKS_FILE};

use crate::store::Store;

pub struct DirStore {
    dir: PathBuf,
}

impl DirStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        DirStore { dir: dir.into() }
    }

    fn exists(&self) -> bool {
        self.dir.is_dir()
    }

    fn log(&self) -> Result<shelfmark_store::Store, Error> {
        shelfmark_store::Store::open(&self.dir)
    }

    fn identity(&self) -> Result<Identity, Error> {
        Identity::open(&self.dir)
    }

    fn error(&self, path: &Path, what: impl std::fmt::Display) -> Error {
        Error::new(
            ErrorCode::StoreError,
            format!("store file {}: {what}", path.display()),
        )
        .next(format!("check the store at {}", self.dir.display()))
    }

    fn read_json<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, Error> {
        let path = self.dir.join(name);
        match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)
                .map(Some)
                .map_err(|e| self.error(&path, e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(self.error(&path, e)),
        }
    }

    fn scheme_files(&self) -> Result<Vec<PathBuf>, Error> {
        let dir = self.dir.join("schemes");
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut files: Vec<PathBuf> = fs::read_dir(&dir)
            .map_err(|e| self.error(&dir, e))?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        files.sort();
        Ok(files)
    }
}

impl PlacementLog for DirStore {
    fn append(&mut self, placement: &Placement) -> Result<bool, Error> {
        self.log()?.append(placement)
    }

    fn claims(&self) -> Result<Vec<Placement>, Error> {
        if !self.exists() {
            return Ok(Vec::new());
        }
        self.log()?.claims()
    }

    fn memo(&self, inputs: &InputHashes) -> Result<Option<Placement>, Error> {
        if !self.exists() {
            return Ok(None);
        }
        self.log()?.memo(inputs)
    }

    /// Covers every file a mutation writes, not just the claim log, so
    /// `link` and `scheme compile` commits move the revision too.
    fn revision(&self) -> Result<String, Error> {
        if !self.exists() {
            return Ok("empty".into());
        }
        let mut all = format!(
            "{}\0{}\0{}\0",
            self.log()?.revision()?,
            self.gold()?.len(),
            self.identity()?.revision()
        );
        for path in self.scheme_files()? {
            let text = fs::read_to_string(&path).map_err(|e| self.error(&path, e))?;
            all.push_str(&format!("{}\0{text}\0", path.display()));
        }
        Ok(format!("s256:{}", sha256_hex(all.as_bytes())))
    }
}

impl Store for DirStore {
    fn location(&self) -> String {
        self.dir.display().to_string()
    }

    fn append_gold(&mut self, example: &GoldExample) -> Result<(), Error> {
        self.log()?.append_gold(example).map(drop)
    }

    fn gold(&self) -> Result<Vec<GoldExample>, Error> {
        if !self.exists() {
            return Ok(Vec::new());
        }
        self.log()?.gold()
    }

    fn append_link(&mut self, item: &ItemId, work: &WorkId) -> Result<bool, Error> {
        self.identity()?.link(item, work).map_err(|e| {
            if e.code == ErrorCode::NotFound && e.next_command.is_none() {
                e.next("shelfmark classify --file <the file>")
            } else {
                e
            }
        })
    }

    /// Each item's current Work: its latest line in the link file.
    fn links(&self) -> Result<Vec<(ItemId, WorkId)>, Error> {
        let path = self.dir.join(LINKS_FILE);
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(self.error(&path, e)),
        };
        let mut out: Vec<(ItemId, WorkId)> = Vec::new();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let link: Link = serde_json::from_str(line).map_err(|e| self.error(&path, e))?;
            out.retain(|(i, _)| i != &link.item);
            out.push((link.item, link.work));
        }
        Ok(out)
    }

    fn ingest(&mut self, minted: &Minted) -> Result<Ingested, Error> {
        self.identity()?.ingest(minted)
    }

    fn save_scheme(&mut self, doc: &SchemeDoc) -> Result<(), Error> {
        let dir = self.dir.join("schemes");
        fs::create_dir_all(&dir).map_err(|e| self.error(&dir, e))?;
        let path = dir.join(format!("{}@{}.json", doc.name, doc.version));
        let text = serde_json::to_string_pretty(doc).map_err(|e| self.error(&path, e))?;
        fs::write(&path, text).map_err(|e| self.error(&path, e))
    }

    fn schemes(&self) -> Result<Vec<SchemeDoc>, Error> {
        self.scheme_files()?
            .into_iter()
            .map(|path| {
                let text = fs::read_to_string(&path).map_err(|e| self.error(&path, e))?;
                serde_json::from_str(&text).map_err(|e| self.error(&path, e))
            })
            .collect()
    }

    /// Name, length and modification time of every scheme file.
    fn schemes_key(&self) -> Result<Option<String>, Error> {
        let mut key = String::new();
        for path in self.scheme_files()? {
            let meta = fs::metadata(&path).map_err(|e| self.error(&path, e))?;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_nanos());
            key.push_str(&format!("{}\0{}\0{modified}\0", path.display(), meta.len()));
        }
        Ok(Some(key))
    }

    fn calibration(&self) -> Result<Option<Calibration>, Error> {
        self.read_json("calibration.json")
    }

    fn eval_summary(&self) -> Result<Option<EvalSummary>, Error> {
        self.read_json("eval.json")
    }
}
