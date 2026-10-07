//! The gold set (R15): `gold.jsonl`, append-only labelled examples.

use shelfmark_core::calibration::{gold_set_hash, GoldExample};
use shelfmark_core::error::Error;
use shelfmark_core::SchemeId;

use crate::{jsonl, Store, GOLD};

impl Store {
    /// Append a labelled example. Appending one equal to an example already
    /// in the set is a no-op that returns `false`.
    pub fn append_gold(&self, example: &GoldExample) -> Result<bool, Error> {
        let _lock = self.lock_exclusive()?;
        let path = self.path(GOLD);
        let existing: Vec<GoldExample> = jsonl::read(&path)?;
        if existing.contains(example) {
            return Ok(false);
        }
        jsonl::append(&path, std::slice::from_ref(example))?;
        Ok(true)
    }

    /// Every gold example in append order.
    pub fn gold(&self) -> Result<Vec<GoldExample>, Error> {
        let _lock = self.lock_shared()?;
        jsonl::read(&self.path(GOLD))
    }

    /// Gold examples for one scheme version, in append order.
    pub fn gold_for(&self, scheme: &SchemeId) -> Result<Vec<GoldExample>, Error> {
        Ok(self
            .gold()?
            .into_iter()
            .filter(|e| &e.scheme == scheme)
            .collect())
    }

    /// `calibration::gold_set_hash` of one scheme version's gold set.
    pub fn gold_hash(&self, scheme: &SchemeId) -> Result<String, Error> {
        Ok(gold_set_hash(&self.gold_for(scheme)?))
    }
}
