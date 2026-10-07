//! What the service needs from a store beyond the claim log (plan PD6).
//!
//! The placement store (U6) and identity crate (U7) own the real
//! formats; this trait is the seam the CLI codes against, so U13 can put
//! them behind it without touching a verb.

use shelfmark_core::calibration::{Calibration, EvalSummary, GoldExample};
use shelfmark_core::placement::PlacementLog;
use shelfmark_core::scheme::SchemeDoc;
use shelfmark_core::{Error, ItemId, WorkId};

pub trait Store: PlacementLog + Send {
    /// Where the store lives, for messages and next commands.
    fn location(&self) -> String;
    /// Append a labelled example to the gold set (R15).
    fn append_gold(&mut self, example: &GoldExample) -> Result<(), Error>;
    fn gold(&self) -> Result<Vec<GoldExample>, Error>;
    /// Record that a file is a copy of a Work. `false` when already linked.
    fn append_link(&mut self, item: &ItemId, work: &WorkId) -> Result<bool, Error>;
    fn links(&self) -> Result<Vec<(ItemId, WorkId)>, Error>;
    /// Keep a compiled scheme so later calls can load it.
    fn save_scheme(&mut self, doc: &SchemeDoc) -> Result<(), Error>;
    fn schemes(&self) -> Result<Vec<SchemeDoc>, Error>;
    /// The fitted calibration the cascade applies, if any (PD3).
    fn calibration(&self) -> Result<Option<Calibration>, Error>;
    /// The latest eval's results, if any (R23).
    fn eval_summary(&self) -> Result<Option<EvalSummary>, Error>;
}
