//! L2 profiles for shelfmark (R8-R10, plan U5 and PD4).
//!
//! - [`builtin`]: the declared profiles `lexical`, `shortlist` and
//!   `rerank`, and what each built-in tier costs.
//! - [`capability`]: what this machine has, and which profiles fit it.
//! - [`semantic`]: the `shortlist` and `rerank` tier stubs.
//! - [`builder`]: a profile plus a registry of tiers becomes a runnable
//!   [`builder::Cascade`] for `shelfmark_core::cascade::run`.
//!
//! Tier implementations are injected by the caller, so this crate does
//! not depend on the lexical tier crates.

pub mod builder;
pub mod builtin;
pub mod capability;
pub mod semantic;

pub use builder::{build, profile_hash, runnable, Cascade, Registry};
pub use builtin::{builtins, find};
pub use capability::{assess, detect, Assessment, Capability, Recommendation};
