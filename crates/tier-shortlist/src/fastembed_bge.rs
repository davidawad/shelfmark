//! BGE-small-en-v1.5 int8 through fastembed and ONNX Runtime (feature
//! `fastembed`).
//!
//! - Model: fastembed's `BGESmallENV15Q` (`Qdrant/bge-small-en-v1.5-onnx-Q`,
//!   384 dimensions, MIT), max length 512 tokens.
//! - Files: downloaded on first load into [`model_dir`]:
//!   `FASTEMBED_CACHE_DIR` when set, else the user's cache directory
//!   (`$XDG_CACHE_HOME`, `~/.cache`, `~/Library/Caches` or `%LOCALAPPDATA%`)
//!   under `shelfmark/models`, so every working directory shares one copy.
//!   The ONNX Runtime library itself is fetched at build time.
//! - Threads: ONNX Runtime's intra-op pool is set explicitly to
//!   [`model_threads`], i.e. `std::thread::available_parallelism()`, which
//!   reads the cgroup CPU quota on Linux. On a box showing 16 cores under a
//!   4-CPU quota that is 4, not 16; oversubscribing the quota roughly
//!   doubled per-item latency in the spike.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
use shelfmark_core::error::{Error, ErrorCode};

use crate::embedder::{Embedder, BGE_QUERY_PREFIX};
use crate::BGE_SMALL_MODEL_ID;

/// Hugging Face repository fastembed fetches the model from.
pub const BGE_MODEL_REPO: &str = "Qdrant/bge-small-en-v1.5-onnx-Q";

/// BGE's maximum input length in tokens.
const MAX_LENGTH: usize = 512;

/// Labels per ONNX batch. Labels are batched in length order so padding
/// stays small. ONNX Runtime's arena keeps its high-water mark, so the
/// batch bounds peak memory: on LCC (374 labels, up to 512 tokens) a batch
/// of 32 peaked at 855 MB RSS against 361 MB for 8, and 8 was also faster
/// (5.4 s against 6.6 s at 4 threads).
const LABEL_BATCH: usize = 8;

/// Where model files are downloaded and looked up: `FASTEMBED_CACHE_DIR`
/// when set, else `shelfmark/models` in the user's cache directory, else
/// fastembed's own default (`.fastembed_cache` in the working directory).
pub fn model_dir() -> PathBuf {
    let env = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if let Some(dir) = env("FASTEMBED_CACHE_DIR") {
        return dir;
    }
    let user_cache = if cfg!(windows) {
        env("LOCALAPPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Caches"))
    } else {
        env("XDG_CACHE_HOME").or_else(|| env("HOME").map(|h| h.join(".cache")))
    };
    user_cache.map_or_else(
        || PathBuf::from(fastembed::get_cache_dir()),
        |c| c.join("shelfmark").join("models"),
    )
}

/// Intra-op threads for the model: the cgroup-aware CPU budget, at least 1.
pub fn model_threads() -> usize {
    std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
}

/// The loaded model. `TextEmbedding::embed` takes `&mut self`, so calls
/// are serialized through a mutex; one ONNX session already uses every
/// thread it is given.
pub struct FastembedBge {
    model: Mutex<TextEmbedding>,
    threads: usize,
}

impl FastembedBge {
    /// Load (downloading if needed) with [`model_threads`] threads.
    pub fn load() -> Result<Self, Error> {
        Self::load_with_threads(model_threads())
    }

    /// Load with an explicit intra-op thread count.
    pub fn load_with_threads(threads: usize) -> Result<Self, Error> {
        let threads = threads.max(1);
        let opts = TextInitOptions::new(EmbeddingModel::BGESmallENV15Q)
            .with_cache_dir(model_dir())
            .with_max_length(MAX_LENGTH)
            .with_intra_threads(threads)
            .with_show_download_progress(false);
        let model = TextEmbedding::try_new(opts).map_err(|e| load_error(&e.to_string()))?;
        Ok(FastembedBge {
            model: Mutex::new(model),
            threads,
        })
    }

    /// The intra-op thread count the session was built with.
    pub fn threads(&self) -> usize {
        self.threads
    }

    fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, Error> {
        let mut model = self.model.lock().unwrap_or_else(|e| e.into_inner());
        model
            .embed(texts, Some(LABEL_BATCH))
            .map_err(|e| Error::internal(format!("{BGE_SMALL_MODEL_ID} failed to embed: {e}")))
    }
}

/// The `tier_unavailable` error for a model that would not load.
fn load_error(cause: &str) -> Error {
    Error::new(
        ErrorCode::TierUnavailable,
        format!(
            "the shortlist tier could not load model {BGE_SMALL_MODEL_ID} ({BGE_MODEL_REPO}): {cause}"
        ),
    )
    .next("shelfmark classify --profile lexical <file>")
    .details(serde_json::json!({
        "tier": "shortlist",
        "model": BGE_SMALL_MODEL_ID,
        "model_repo": BGE_MODEL_REPO,
        "cache_dir": model_dir(),
        "cache_dir_env": "FASTEMBED_CACHE_DIR",
        "cause": cause,
    }))
}

impl Embedder for FastembedBge {
    fn model_id(&self) -> &str {
        BGE_SMALL_MODEL_ID
    }

    fn label_prefix(&self) -> &str {
        BGE_QUERY_PREFIX
    }

    fn embed_labels(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Error> {
        let mut order: Vec<usize> = (0..texts.len()).collect();
        order.sort_by_key(|&i| (texts[i].len(), i));
        let sorted: Vec<String> = order
            .iter()
            .map(|&i| format!("{BGE_QUERY_PREFIX}{}", texts[i]))
            .collect();
        let rows = self.embed(sorted)?;
        let mut out = vec![Vec::new(); texts.len()];
        for (row, &i) in rows.into_iter().zip(&order) {
            out[i] = row;
        }
        Ok(out)
    }

    fn embed_items(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Error> {
        self.embed(texts.to_vec())
    }
}

/// [`FastembedBge`] loaded on first use and kept, so a build with the
/// feature pays no model load for work that never ranks with it. A failed
/// load is kept too: every later call returns the same error without
/// retrying.
#[derive(Default)]
pub struct LazyBge {
    model: OnceLock<Result<FastembedBge, Error>>,
}

impl LazyBge {
    pub fn new() -> Self {
        LazyBge::default()
    }

    /// The model, loading it now if this is the first call.
    pub fn get(&self) -> Result<&FastembedBge, Error> {
        self.model
            .get_or_init(FastembedBge::load)
            .as_ref()
            .map_err(Clone::clone)
    }

    /// Whether a load has been attempted.
    pub fn is_loaded(&self) -> bool {
        self.model.get().is_some()
    }
}

impl Embedder for LazyBge {
    fn model_id(&self) -> &str {
        BGE_SMALL_MODEL_ID
    }

    fn label_prefix(&self) -> &str {
        BGE_QUERY_PREFIX
    }

    fn embed_labels(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Error> {
        self.get()?.embed_labels(texts)
    }

    fn embed_items(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Error> {
        self.get()?.embed_items(texts)
    }
}
