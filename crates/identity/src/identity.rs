//! Minting (R1, R2) and the item-to-Work link file (R3, AE4).
//!
//! A store directory holds two files written here:
//!
//! - `links.jsonl`: append-only [`Link`] records. An item's current Work
//!   is its latest link; its first link is always the Work its own text
//!   minted.
//! - `minhash.jsonl`: one [`SignatureRecord`] per Work, derived from the
//!   text and rebuildable by re-ingesting the items.
//!
//! Both are appended only under the store lock (plan PD6).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use shelfmark_core::{Error, ErrorCode, ItemId, WorkId};

use crate::extract::{extract, read_file, Format};
use crate::index::{LinkProposal, NearDupIndex};
use crate::jsonl::JsonlLog;
use crate::lock::with_store_lock;
use crate::minhash::{Signature, SCHEME};

/// The link file's name inside a store directory.
pub const LINKS_FILE: &str = "links.jsonl";
/// The signature file's name inside a store directory.
pub const SIGNATURES_FILE: &str = "minhash.jsonl";

/// How an item came to point at a Work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Via {
    /// The Work its own text minted.
    Minted,
    /// An accepted link (usually a near-duplicate proposal).
    Linked,
}

/// One line of `links.jsonl`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub item: ItemId,
    pub work: WorkId,
    pub via: Via,
    /// Estimated Jaccard similarity between the item's minted Work and
    /// `work`, for `linked` records where both signatures are known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub similarity: Option<f64>,
}

/// One line of `minhash.jsonl`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignatureRecord {
    pub work: WorkId,
    pub scheme: String,
    pub signature: Signature,
}

/// The IDs and text of one input, before anything is written.
#[derive(Debug, Clone, PartialEq)]
pub struct Minted {
    pub item: ItemId,
    pub work: WorkId,
    pub format: Format,
    pub signature: Signature,
    /// The extracted text, kept so callers can classify it without
    /// extracting twice.
    pub text: String,
}

impl Minted {
    /// Mint from a file's bytes in a known format.
    pub fn from_bytes(bytes: &[u8], format: Format) -> Result<Minted, Error> {
        let text = extract(bytes, format)?;
        Ok(Minted::from_parts(bytes, format, text))
    }

    /// Mint from a file, choosing the format by extension.
    pub fn from_path(path: &Path) -> Result<Minted, Error> {
        let (bytes, format, text) = read_file(path)?;
        Ok(Minted::from_parts(&bytes, format, text))
    }

    fn from_parts(bytes: &[u8], format: Format, text: String) -> Minted {
        Minted {
            item: ItemId::from_bytes(bytes),
            work: WorkId::from_text(&text),
            format,
            signature: Signature::of_text(&text),
            text,
        }
    }
}

/// What ingesting an item did (or, from [`Identity::preview`], would do).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ingested {
    pub item: ItemId,
    /// The item's current Work: an earlier accepted link if there is one,
    /// otherwise the Work its text minted.
    pub work: WorkId,
    /// The Work the item's text minted.
    pub minted_work: WorkId,
    pub format: Format,
    /// The item was not in the link file before.
    pub new_item: bool,
    /// No item had minted this Work before.
    pub new_work: bool,
    /// Near-duplicate Works the item could be linked to instead.
    pub proposals: Vec<LinkProposal>,
}

/// The identity side of a store directory: links plus the near-duplicate
/// index.
#[derive(Debug)]
pub struct Identity {
    dir: PathBuf,
    links: JsonlLog<Link>,
    signatures: JsonlLog<SignatureRecord>,
    current: HashMap<ItemId, WorkId>,
    minted: HashMap<ItemId, WorkId>,
    items: HashMap<WorkId, Vec<ItemId>>,
    index: NearDupIndex,
}

impl Identity {
    /// Load the link and signature files from `dir` (absent files are
    /// empty; nothing is created until the first write).
    pub fn open(dir: impl Into<PathBuf>) -> Result<Identity, Error> {
        let dir = dir.into();
        let (links, link_records) = JsonlLog::open(dir.join(LINKS_FILE))?;
        let (signatures, sig_records) = JsonlLog::open(dir.join(SIGNATURES_FILE))?;
        let mut id = Identity {
            dir,
            links,
            signatures,
            current: HashMap::new(),
            minted: HashMap::new(),
            items: HashMap::new(),
            index: NearDupIndex::new(),
        };
        id.absorb_links(link_records);
        id.absorb_signatures(sig_records)?;
        Ok(id)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Opaque revision of the link file for preview/commit; changes on
    /// every appended link.
    pub fn revision(&self) -> &str {
        self.links.revision()
    }

    /// The item's current Work.
    pub fn work_of(&self, item: &ItemId) -> Option<&WorkId> {
        self.current.get(item)
    }

    /// Items whose current Work is `work`, in first-linked order.
    pub fn items_of(&self, work: &WorkId) -> Vec<&ItemId> {
        self.items
            .get(work)
            .into_iter()
            .flatten()
            .filter(|i| self.current.get(*i) == Some(work))
            .collect()
    }

    /// Whether any item points, or has pointed, at `work`.
    pub fn knows_work(&self, work: &WorkId) -> bool {
        self.items.contains_key(work)
    }

    pub fn index(&self) -> &NearDupIndex {
        &self.index
    }

    fn absorb_links(&mut self, records: Vec<Link>) {
        for link in records {
            if link.via == Via::Minted {
                self.minted
                    .entry(link.item.clone())
                    .or_insert_with(|| link.work.clone());
            }
            let items = self.items.entry(link.work.clone()).or_default();
            if !items.contains(&link.item) {
                items.push(link.item.clone());
            }
            self.current.insert(link.item, link.work);
        }
    }

    fn absorb_signatures(&mut self, records: Vec<SignatureRecord>) -> Result<(), Error> {
        for rec in records {
            if rec.scheme != SCHEME {
                return Err(Error::new(
                    ErrorCode::StoreError,
                    format!(
                        "{} holds a {} signature; this build reads {SCHEME}",
                        self.signatures.path().display(),
                        rec.scheme
                    ),
                ));
            }
            self.index.insert(rec.work, rec.signature);
        }
        Ok(())
    }

    /// What [`Identity::ingest`] would do, without writing.
    pub fn preview(&self, minted: &Minted) -> Ingested {
        let current = self.current.get(&minted.item);
        Ingested {
            item: minted.item.clone(),
            work: current.unwrap_or(&minted.work).clone(),
            minted_work: minted.work.clone(),
            format: minted.format,
            new_item: current.is_none(),
            new_work: !self.knows_work(&minted.work),
            proposals: self
                .index
                .propose(&minted.item, &minted.work, &minted.signature, current),
        }
    }

    /// Record an item: link it to its minted Work if it is new, and index
    /// the Work's signature if no item minted it before. An item whose
    /// words match an existing Work exactly gets that Work's ID (AE4);
    /// a near-duplicate gets its own Work plus proposals.
    pub fn ingest(&mut self, minted: &Minted) -> Result<Ingested, Error> {
        let dir = self.dir.clone();
        with_store_lock(&dir, || {
            self.catch_up()?;
            let out = self.preview(minted);
            if out.new_item {
                let link = Link {
                    item: minted.item.clone(),
                    work: minted.work.clone(),
                    via: Via::Minted,
                    similarity: None,
                };
                self.links.append_locked(&link)?;
                self.absorb_links(vec![link]);
            }
            if !self.index.contains(&minted.work) {
                let rec = SignatureRecord {
                    work: minted.work.clone(),
                    scheme: SCHEME.to_string(),
                    signature: minted.signature.clone(),
                };
                self.signatures.append_locked(&rec)?;
                self.index.insert(rec.work, rec.signature);
            }
            Ok(out)
        })
    }

    /// Point `item` at `work`, e.g. to accept a [`LinkProposal`]. Returns
    /// `false` (writing nothing) if the item already points there. Both
    /// must already be known: the item from an ingest, the Work from any
    /// item.
    pub fn link(&mut self, item: &ItemId, work: &WorkId) -> Result<bool, Error> {
        let dir = self.dir.clone();
        with_store_lock(&dir, || {
            self.catch_up()?;
            let Some(current) = self.current.get(item) else {
                return Err(Error::new(
                    ErrorCode::NotFound,
                    format!("{item} has not been ingested"),
                )
                .details(serde_json::json!({ "item": item.to_string() })));
            };
            if current == work {
                return Ok(false);
            }
            if !self.knows_work(work) {
                return Err(
                    Error::new(ErrorCode::NotFound, format!("no item belongs to {work}"))
                        .details(serde_json::json!({ "work": work.to_string() })),
                );
            }
            let similarity = self
                .minted
                .get(item)
                .and_then(|w| self.index.signature(w))
                .zip(self.index.signature(work))
                .map(|(a, b)| a.similarity(b));
            let link = Link {
                item: item.clone(),
                work: work.clone(),
                via: Via::Linked,
                similarity,
            };
            self.links.append_locked(&link)?;
            self.absorb_links(vec![link]);
            Ok(true)
        })
    }

    /// Pick up lines appended by another writer since this handle loaded.
    fn catch_up(&mut self) -> Result<(), Error> {
        let links = self.links.catch_up()?;
        self.absorb_links(links);
        let sigs = self.signatures.catch_up()?;
        self.absorb_signatures(sigs)
    }
}
