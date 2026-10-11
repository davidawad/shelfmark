//! The `openlibrary` tier: look a book up on Open Library by title and
//! author, and place it by the LC call numbers catalogers gave its
//! editions.
//!
//! - [`transport`]: the [`Transport`] trait every request goes through,
//!   and [`FixtureTransport`], a fake that serves recorded bodies.
//! - [`client`]: [`Client`] builds the search URL and adds the politeness
//!   rules: one request per [`Pacer`] interval (1 s by default), an
//!   on-disk [`DiskCache`], and bounded retries on timeouts, 429 and 5xx.
//! - [`matching`]: normalized title and author similarity, and the best
//!   record above a confidence threshold.
//! - [`callnumber`]: LC call numbers (raw `lc_classifications` or Open
//!   Library's padded `lcc` sort form) to the deepest covering class of
//!   the active scheme.
//! - [`OpenLibraryTier`]: the [`Tier`](shelfmark_core::tier::Tier).
//!
//! With feature `http`, [`UreqTransport`] talks to openlibrary.org over
//! rustls with a User-Agent naming the repository. Nothing here touches
//! the network unless a caller hands it that transport.

pub mod cache;
pub mod callnumber;
pub mod client;
#[cfg(feature = "http")]
pub mod http;
pub mod matching;
pub mod pacer;
mod tier;
pub mod transport;

pub use cache::DiskCache;
pub use client::{Client, ClientConfig, Doc, Edition, Editions, Query, SEARCH_URL};
#[cfg(feature = "http")]
pub use http::UreqTransport;
pub use matching::{best_match, Match, DEFAULT_THRESHOLD};
pub use pacer::Pacer;
pub use tier::OpenLibraryTier;
pub use transport::{FixtureTransport, Response, Transport, TransportError};

/// The tier's name in profiles.
pub const NAME: &str = "openlibrary";
/// Version of the matching and mapping logic (part of the profile hash).
pub const VERSION: &str = "1";
/// Catalog name in evidence.
pub const CATALOG: &str = "openlibrary";
/// The User-Agent every request sends: who is asking and where to reach
/// them, as Open Library asks of API clients.
pub const USER_AGENT: &str = concat!(
    "shelfmark/",
    env!("CARGO_PKG_VERSION"),
    " (+https://gitlab.com/davidawad/shelfmark)"
);
