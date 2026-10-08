//! The `shortlist` tier for shelfmark (plan R8 `shortlist-lite`; R7, R9).
//!
//! An embedding model places an item by cosine similarity against the
//! scheme's own labels, coarse to fine:
//!
//! 1. Every level-0 and level-1 class gets a label text derived from the
//!    scheme alone ([`labels`]), embedded once per (label set, model) and
//!    memoized in memory and on disk ([`LabelIndex`]).
//! 2. The item (title + text, clipped) is embedded and ranked against the
//!    level-0 classes; the best [`Params::k_classes`] are kept.
//! 3. The level-1 classes under those are ranked, and the best
//!    [`Params::n_candidates`] are emitted with a softmax score over every
//!    ranked fine-stage cosine. A kept class without subclasses stands for
//!    itself; a scheme without level-1 classes ranks all level-0 classes.
//!
//! Findings from the CPU semantic spike this honors:
//!
//! - Labels are embedded as **queries** (BGE's query prefix) and items as
//!   **passages** (no prefix). The other direction collapsed onto hub
//!   classes.
//! - No mean-centering of label vectors; it hurt BGE-small once flipped.
//! - Model threads are sized from `std::thread::available_parallelism()`,
//!   which honors the cgroup CPU quota, not the visible core count.
//!
//! The real model ([`FastembedBge`], BGE-small-en-v1.5 int8 via fastembed
//! and ONNX Runtime) is behind the `fastembed` feature. Without it the
//! tier still builds and runs against any [`Embedder`], which is how the
//! tests exercise it.
//!
//! Deterministic: identical input gives identical candidates, ranked by
//! score, then class code in byte order.

use std::cmp::Ordering;

use shelfmark_core::tier::{Candidate, ClassifyInput};

mod cache;
mod embedder;
#[cfg(feature = "fastembed")]
mod fastembed_bge;
mod index;
pub mod labels;
mod tier;

pub use embedder::{normalize, Embedder, BGE_QUERY_PREFIX};
#[cfg(feature = "fastembed")]
pub use fastembed_bge::{model_dir, model_threads, FastembedBge, LazyBge, BGE_MODEL_REPO};
pub use index::{Label, LabelIndex, CACHE_FORMAT_VERSION};
pub use tier::{Params, ShortlistTier, ALGORITHM, TEMPERATURE};

/// Model ID of BGE-small-en-v1.5 int8 (fastembed's `BGESmallENV15Q`).
pub const BGE_SMALL_MODEL_ID: &str = "bge-small-en-v1.5-q";

/// Rank best first: score, then class code in byte order (plan PD9).
pub(crate) fn rank(a: &Candidate, b: &Candidate) -> Ordering {
    b.score
        .total_cmp(&a.score)
        .then_with(|| a.class.code.as_bytes().cmp(b.class.code.as_bytes()))
}

/// The text embedded for an item: title and text (whichever are present
/// and non-blank) joined by a newline, clipped to `max_chars` characters on
/// a char boundary. Empty when there is nothing to embed.
pub fn item_text(input: &ClassifyInput, max_chars: usize) -> String {
    let parts: Vec<&str> = input
        .title
        .as_deref()
        .into_iter()
        .chain(std::iter::once(input.text.as_str()))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    clip(&parts.join("\n"), max_chars).to_string()
}

/// The first `max_chars` characters of `s`.
pub(crate) fn clip(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

/// Microseconds since `start`, saturating.
pub(crate) fn micros_since(start: std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_respects_char_boundaries() {
        assert_eq!(clip("héllo", 2), "hé");
        assert_eq!(clip("hi", 10), "hi");
        assert_eq!(clip("", 3), "");
    }

    #[test]
    fn item_text_joins_present_fields() {
        let mut input = ClassifyInput {
            title: Some(" Dragon book ".into()),
            text: "Compilers.".into(),
            ..ClassifyInput::default()
        };
        assert_eq!(item_text(&input, 1500), "Dragon book\nCompilers.");
        input.text = "   ".into();
        assert_eq!(item_text(&input, 1500), "Dragon book");
        input.title = None;
        assert_eq!(item_text(&input, 1500), "");
        input.text = "ééééé".into();
        assert_eq!(item_text(&input, 3), "ééé");
    }
}
