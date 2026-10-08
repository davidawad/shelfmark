//! [`LabelIndex`]: one unit vector per level-0 and level-1 class of a
//! compiled scheme, with parent links.

use std::path::Path;

use shelfmark_core::error::Error;
use shelfmark_core::id::SchemeId;
use shelfmark_core::scheme::CompiledIndex;

use crate::embedder::{normalize, Embedder};
use crate::{cache, labels};

/// Version of the on-disk label-cache format and of how label texts are
/// built; bump it to orphan every cached file.
pub const CACHE_FORMAT_VERSION: u32 = 1;

/// One embedded class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub code: String,
    /// 0 or 1.
    pub level: u32,
    /// Position of the level-0 parent in [`LabelIndex::labels`], for a
    /// level-1 class.
    pub parent: Option<usize>,
    /// The text that was embedded (see [`crate::labels`]).
    pub text: String,
}

/// Unit-normalized label vectors for the level-0 and level-1 classes of
/// one compiled index, for one model. Level-0 labels come first, in the
/// index's order, then level-1 labels grouped under their parents.
#[derive(Debug, Clone)]
pub struct LabelIndex {
    scheme: SchemeId,
    content_hash: String,
    model_id: String,
    labels: Vec<Label>,
    dim: usize,
    /// Row-major, `labels.len() x dim`.
    vectors: Vec<f32>,
    from_disk: bool,
}

impl LabelIndex {
    /// The labels of `index`, in [`LabelIndex`] order.
    pub fn labels_of(index: &dyn CompiledIndex) -> Vec<Label> {
        let tops = index.children(None);
        let names = |code: &str| -> Vec<String> {
            index
                .children(Some(code))
                .into_iter()
                .map(|c| c.label)
                .collect()
        };
        let mut out: Vec<Label> = tops
            .iter()
            .map(|t| {
                let kids = names(&t.id.code);
                let kid_refs: Vec<&str> = kids.iter().map(String::as_str).collect();
                Label {
                    code: t.id.code.clone(),
                    level: 0,
                    parent: None,
                    text: labels::top_text(&t.label, &kid_refs),
                }
            })
            .collect();
        for (pi, t) in tops.iter().enumerate() {
            for s in index.children(Some(&t.id.code)) {
                let kids = names(&s.id.code);
                let kid_refs: Vec<&str> = kids.iter().map(String::as_str).collect();
                out.push(Label {
                    text: labels::sub_text(&t.label, &s.label, &kid_refs),
                    code: s.id.code,
                    level: 1,
                    parent: Some(pi),
                });
            }
        }
        out
    }

    /// Embed every label of `index` with `embedder`, or load the vectors
    /// from `cache_dir` when an earlier build stored them for exactly the
    /// same label texts and model. A fresh build is written back to
    /// `cache_dir` (best effort).
    pub fn build(
        index: &dyn CompiledIndex,
        embedder: &dyn Embedder,
        cache_dir: Option<&Path>,
    ) -> Result<LabelIndex, Error> {
        let labels = Self::labels_of(index);
        let model_id = embedder.model_id().to_string();
        let file = cache_dir.map(|dir| {
            let key = cache::key(&model_id, embedder.label_prefix(), &labels);
            cache::path(dir, &model_id, &key)
        });
        let mut out = LabelIndex {
            scheme: index.scheme().clone(),
            content_hash: index.content_hash().to_string(),
            model_id,
            labels,
            dim: 0,
            vectors: Vec::new(),
            from_disk: false,
        };
        if out.labels.is_empty() {
            return Ok(out);
        }
        if let Some((dim, vectors)) = file.as_deref().and_then(|p| cache::read(p, &out.labels)) {
            out.dim = dim;
            out.vectors = vectors;
            out.from_disk = true;
            return Ok(out);
        }
        let texts: Vec<String> = out.labels.iter().map(|l| l.text.clone()).collect();
        let rows = embedder.embed_labels(&texts)?;
        let dim = rows.first().map_or(0, Vec::len);
        if rows.len() != texts.len() || dim == 0 || rows.iter().any(|r| r.len() != dim) {
            return Err(Error::internal(format!(
                "embedder {} returned {} label vectors (dimension {dim}) for {} labels, or ragged rows",
                out.model_id,
                rows.len(),
                texts.len()
            )));
        }
        out.dim = dim;
        out.vectors = rows
            .into_iter()
            .flat_map(|mut r| {
                normalize(&mut r);
                r
            })
            .collect();
        if let Some(p) = &file {
            cache::write(p, &out.labels, dim, &out.vectors);
        }
        Ok(out)
    }

    /// Whether this index was built for `index`'s content and `model_id`.
    pub fn is_for(&self, index: &dyn CompiledIndex, model_id: &str) -> bool {
        self.content_hash == index.content_hash() && self.model_id == model_id
    }

    /// The scheme the labels belong to.
    pub fn scheme(&self) -> &SchemeId {
        &self.scheme
    }

    pub fn labels(&self) -> &[Label] {
        &self.labels
    }

    /// Vector dimension (0 for a scheme without classes).
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// The unit vector of label `i`.
    pub fn vector(&self, i: usize) -> &[f32] {
        &self.vectors[i * self.dim..(i + 1) * self.dim]
    }

    /// Whether the vectors were loaded from the disk cache rather than
    /// embedded.
    pub fn from_disk(&self) -> bool {
        self.from_disk
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }
}
