//! A directory of JSONL files standing in for the placement store (U6)
//! and the item-to-work links (U7). Same layout idea as PD6, none of its
//! guarantees (no lock, no memo index).

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;
use shelfmark_core::calibration::{Calibration, EvalSummary, GoldExample};
use shelfmark_core::id::sha256_hex;
use shelfmark_core::placement::{Placement, PlacementLog};
use shelfmark_core::scheme::SchemeDoc;
use shelfmark_core::{Error, ErrorCode, ItemId, WorkId};

use crate::store::Store;

pub struct FsStore {
    dir: PathBuf,
}

#[derive(serde::Deserialize, Serialize)]
struct Link {
    item: ItemId,
    work: WorkId,
}

impl FsStore {
    /// Opening never touches the disk; the directory appears on first write.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        FsStore { dir: dir.into() }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    fn error(&self, path: &Path, what: impl std::fmt::Display) -> Error {
        Error::new(
            ErrorCode::StoreError,
            format!("store file {}: {what}", path.display()),
        )
        .next(format!(
            "shelfmark status --store {}",
            crate::describe::shell_quote(&self.location())
        ))
    }

    fn read(&self, name: &str) -> Result<Option<String>, Error> {
        let path = self.path(name);
        match fs::read_to_string(&path) {
            Ok(s) => Ok(Some(s)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(self.error(&path, e)),
        }
    }

    fn read_jsonl<T: DeserializeOwned>(&self, name: &str) -> Result<Vec<T>, Error> {
        let Some(text) = self.read(name)? else {
            return Ok(Vec::new());
        };
        text.lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
            .map(|(i, l)| {
                serde_json::from_str(l)
                    .map_err(|e| self.error(&self.path(name), format!("line {}: {e}", i + 1)))
            })
            .collect()
    }

    fn read_json<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, Error> {
        self.read(name)?
            .map(|s| serde_json::from_str(&s).map_err(|e| self.error(&self.path(name), e)))
            .transpose()
    }

    fn append_jsonl(&self, name: &str, value: &impl Serialize) -> Result<(), Error> {
        let path = self.path(name);
        let line = serde_json::to_string(value).map_err(|e| self.error(&path, e))?;
        fs::create_dir_all(&self.dir).map_err(|e| self.error(&self.dir, e))?;
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| self.error(&path, e))?;
        writeln!(f, "{line}").map_err(|e| self.error(&path, e))
    }

    fn scheme_files(&self) -> Result<Vec<PathBuf>, Error> {
        let dir = self.path("schemes");
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(self.error(&dir, e)),
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        files.sort();
        Ok(files)
    }
}

impl PlacementLog for FsStore {
    fn append(&mut self, placement: &Placement) -> Result<bool, Error> {
        if self.claims()?.iter().any(|p| p.id == placement.id) {
            return Ok(false);
        }
        self.append_jsonl("placements.jsonl", placement)?;
        Ok(true)
    }

    fn claims(&self) -> Result<Vec<Placement>, Error> {
        self.read_jsonl("placements.jsonl")
    }

    /// Covers every file a mutation writes, not just the claim log, so
    /// `link` and `scheme compile` commits move the revision too.
    fn revision(&self) -> Result<String, Error> {
        let mut all = String::new();
        for name in ["placements.jsonl", "gold.jsonl", "links.jsonl"] {
            all.push_str(&self.read(name)?.unwrap_or_default());
            all.push('\0');
        }
        for path in self.scheme_files()? {
            let text = fs::read_to_string(&path).map_err(|e| self.error(&path, e))?;
            all.push_str(&format!("{}\0{text}\0", path.display()));
        }
        Ok(format!("s256:{}", sha256_hex(all.as_bytes())))
    }
}

impl Store for FsStore {
    fn location(&self) -> String {
        self.dir.display().to_string()
    }

    fn append_gold(&mut self, example: &GoldExample) -> Result<(), Error> {
        self.append_jsonl("gold.jsonl", example)
    }

    fn gold(&self) -> Result<Vec<GoldExample>, Error> {
        self.read_jsonl("gold.jsonl")
    }

    fn append_link(&mut self, item: &ItemId, work: &WorkId) -> Result<bool, Error> {
        if self.links()?.iter().any(|(i, w)| i == item && w == work) {
            return Ok(false);
        }
        let link = Link {
            item: item.clone(),
            work: work.clone(),
        };
        self.append_jsonl("links.jsonl", &link)?;
        Ok(true)
    }

    fn links(&self) -> Result<Vec<(ItemId, WorkId)>, Error> {
        Ok(self
            .read_jsonl::<Link>("links.jsonl")?
            .into_iter()
            .map(|l| (l.item, l.work))
            .collect())
    }

    fn save_scheme(&mut self, doc: &SchemeDoc) -> Result<(), Error> {
        let dir = self.path("schemes");
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

    fn calibration(&self) -> Result<Option<Calibration>, Error> {
        self.read_json("calibration.json")
    }

    fn eval_summary(&self) -> Result<Option<EvalSummary>, Error> {
        self.read_json("eval.json")
    }
}
