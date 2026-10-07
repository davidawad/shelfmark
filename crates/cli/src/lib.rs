//! The `shelfmark` command line and [`Shelfmark`], the one
//! [`shelfmark_core::Service`] every surface calls (plan U9, PD8).
//!
//! - [`service`]: the dispatcher over the 16 verbs.
//! - [`describe`]: the verb catalog `describe` returns and the CLI form of
//!   every request, used for `next_command` and `commit_command`.
//! - [`cli`]: clap subcommands mapped to `Request {verb, args}`.
//! - [`batch`]: NDJSON batches keyed by input hash, with resume.
//! - [`render`]: the human rendering for a terminal.
//! - [`store`]: the store seam the service needs beyond `PlacementLog`.
//! - [`wiring`]: which index, tiers, profiles and store back the service.
//!   It holds in-crate fakes today; U13 replaces it with the real crates.

pub mod args;
pub mod batch;
pub mod cli;
pub mod describe;
pub mod render;
pub mod service;
pub mod store;
pub mod time;
pub mod wiring;

pub use service::Shelfmark;
