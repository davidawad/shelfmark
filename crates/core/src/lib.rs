//! Shared contracts for shelfmark, the materials classifier.
//!
//! Every other crate builds against these types, so they change only with
//! a note on the parent task (plan U0):
//!
//! - [`id`]: L0-L3 typed, self-resolving IDs.
//! - [`scheme`]: L1 scheme data model and the [`scheme::CompiledIndex`] trait.
//! - [`tier`]: L2 [`tier::Tier`] trait, inputs, candidates and profiles.
//! - [`cascade`]: the deterministic stop rule over a profile's tiers.
//! - [`calibration`]: gold examples, fitted calibration, eval summary,
//!   config hash.
//! - [`placement`]: L3 claims, traces, input hashes, the log trait.
//! - [`error`] and [`envelope`]: stable error codes and the one
//!   request/response shape the CLI, gRPC and the kb resource share.

pub mod calibration;
pub mod cascade;
pub mod envelope;
pub mod error;
pub mod id;
pub mod placement;
pub mod scheme;
#[cfg(any(test, feature = "testkit"))]
pub mod testkit;
pub mod tier;

pub use envelope::{Envelope, Request, Service, Verb};
pub use error::{Error, ErrorCode};
pub use id::{AnyId, ClassId, ItemId, Kind, PlacementId, ProfileId, SchemeId, WorkId};
