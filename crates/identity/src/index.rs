//! The near-duplicate index (R3): LSH banding over MinHash signatures, and
//! link proposals for Works whose estimated Jaccard similarity reaches
//! [`PROPOSAL_THRESHOLD`].
//!
//! [`BANDS`] bands of [`ROWS`] rows put a pair in the same bucket with
//! probability `1 - (1 - s^8)^16`: about 0.9996 at similarity 0.8 and
//! 0.11 at 0.5, so true near-duplicates are found and most unrelated
//! Works are never compared.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use shelfmark_core::{ItemId, WorkId};

use crate::minhash::{fnv1a, Signature, PERMUTATIONS, SCHEME};

/// LSH bands.
pub const BANDS: usize = 16;
/// Signature rows per band.
pub const ROWS: usize = PERMUTATIONS / BANDS;
/// A proposal needs at least this estimated Jaccard similarity.
pub const PROPOSAL_THRESHOLD: f64 = 0.8;

/// A suggestion that an item's text is a near-duplicate of an existing
/// Work. It is data only: accepting it is the CLI's `link` verb, which
/// appends a link (see [`crate::Identity::link`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkProposal {
    /// The item whose text was compared.
    pub item: ItemId,
    /// The Work the item's own text minted.
    pub from_work: WorkId,
    /// The existing Work it resembles.
    pub work: WorkId,
    /// Estimated Jaccard similarity of the word 5-shingle sets.
    pub similarity: f64,
    /// The signature derivation the estimate came from.
    pub method: String,
}

/// Signatures of known Works, bucketed by band.
#[derive(Debug, Default, Clone)]
pub struct NearDupIndex {
    works: Vec<(WorkId, Signature)>,
    position: HashMap<WorkId, usize>,
    buckets: HashMap<(usize, u64), Vec<usize>>,
}

fn band_key(sig: &Signature, band: usize) -> u64 {
    let mut bytes = [0u8; ROWS * 8];
    for (i, m) in sig.0[band * ROWS..(band + 1) * ROWS].iter().enumerate() {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&m.to_le_bytes());
    }
    fnv1a(&bytes)
}

impl NearDupIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.works.len()
    }

    pub fn is_empty(&self) -> bool {
        self.works.is_empty()
    }

    pub fn contains(&self, work: &WorkId) -> bool {
        self.position.contains_key(work)
    }

    pub fn signature(&self, work: &WorkId) -> Option<&Signature> {
        self.position.get(work).map(|&i| &self.works[i].1)
    }

    /// Add a Work. Returns `false` (and changes nothing) if it is already
    /// indexed. Empty signatures are kept but never bucketed.
    pub fn insert(&mut self, work: WorkId, sig: Signature) -> bool {
        if self.position.contains_key(&work) {
            return false;
        }
        let at = self.works.len();
        if !sig.is_empty() {
            for band in 0..BANDS {
                self.buckets
                    .entry((band, band_key(&sig, band)))
                    .or_default()
                    .push(at);
            }
        }
        self.position.insert(work.clone(), at);
        self.works.push((work, sig));
        true
    }

    /// Works similar to `sig` at or above `threshold`, most similar first
    /// (ties by Work ID), skipping every Work in `exclude`.
    pub fn similar(
        &self,
        sig: &Signature,
        threshold: f64,
        exclude: &[&WorkId],
    ) -> Vec<(WorkId, f64)> {
        if sig.is_empty() {
            return Vec::new();
        }
        let mut seen: Vec<usize> = (0..BANDS)
            .filter_map(|band| self.buckets.get(&(band, band_key(sig, band))))
            .flatten()
            .copied()
            .collect();
        seen.sort_unstable();
        seen.dedup();
        let mut out: Vec<(WorkId, f64)> = seen
            .into_iter()
            .map(|i| &self.works[i])
            .filter(|(w, _)| !exclude.contains(&w))
            .map(|(w, s)| (w.clone(), sig.similarity(s)))
            .filter(|&(_, s)| s >= threshold)
            .collect();
        out.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        out
    }

    /// Link proposals for an item whose text minted `from_work` with
    /// signature `sig`. `current` is the Work the item is already linked
    /// to, if any; neither it nor `from_work` is proposed.
    pub fn propose(
        &self,
        item: &ItemId,
        from_work: &WorkId,
        sig: &Signature,
        current: Option<&WorkId>,
    ) -> Vec<LinkProposal> {
        let mut exclude = vec![from_work];
        exclude.extend(current);
        self.similar(sig, PROPOSAL_THRESHOLD, &exclude)
            .into_iter()
            .map(|(work, similarity)| LinkProposal {
                item: item.clone(),
                from_work: from_work.clone(),
                work,
                similarity,
                method: SCHEME.to_string(),
            })
            .collect()
    }
}
