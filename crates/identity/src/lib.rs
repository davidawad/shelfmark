//! Identity for shelfmark (plan U7; R1-R3, AE4, PD2).
//!
//! - [`extract`]: plain text from `.txt`, `.md`, `.html` and (with the
//!   `pdf` feature) `.pdf`.
//! - [`Minted`]: an input's Item ID (file bytes), Work ID (normalized
//!   text, `WorkId::from_text`) and MinHash signature.
//! - [`Identity`]: the store directory's append-only `links.jsonl`
//!   (item to Work) and `minhash.jsonl`, with near-duplicate
//!   [`LinkProposal`]s returned as data.
//! - [`minhash`] and [`index`]: word 5-shingle MinHash (128 fixed
//!   permutations) and LSH banding.

pub mod extract;
mod html;
pub mod identity;
pub mod index;
mod jsonl;
pub mod lock;
pub mod minhash;

pub use extract::Format;
pub use html::html_to_text;
pub use identity::{
    Identity, Ingested, Link, Minted, SignatureRecord, Via, LINKS_FILE, SIGNATURES_FILE,
};
pub use index::{LinkProposal, NearDupIndex, PROPOSAL_THRESHOLD};
pub use minhash::Signature;
