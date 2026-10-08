//! Embedding backends behind one trait: fastembed ONNX (small models, ORT int8 variants),
//! fastembed's candle Qwen3 (f32/bf16), and this crate's quantizable candle Qwen3.

use crate::qwen3::{Precision, Qwen3};
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use tokenizers::Tokenizer;

pub const QWEN3_EMB: &str = "Qwen/Qwen3-Embedding-0.6B";
pub const TASK: &str =
    "Given a description of a book or document, retrieve the library classification category that best fits its subject";

/// Instruction when the direction is flipped: labels are the queries, items the passages.
pub const LABEL_TASK: &str =
    "Given a library classification category, retrieve descriptions of books or documents that belong to it";

pub trait Embedder {
    fn embed_queries(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn embed_docs(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    /// Labels embedded as queries (flipped direction); same prefix as item queries unless
    /// the model takes a task instruction.
    fn embed_label_queries(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.embed_queries(texts)
    }
}

/// Download (or reuse) a Hugging Face snapshot and return its local directory.
pub fn hf_snapshot(repo: &str, files: &[&str]) -> Result<PathBuf> {
    let api = hf_hub::api::sync::ApiBuilder::from_env()
        .with_progress(false)
        .build()?;
    let r = api.model(repo.to_string());
    let mut dir = None;
    for f in files {
        let p = r.get(f)?;
        dir = p.parent().map(|d| d.to_path_buf());
    }
    dir.ok_or_else(|| anyhow!("no files requested from {repo}"))
}

/// Build an embedder from a profile spec such as `fe:bge-small-en-v1.5-q`,
/// `fe-qwen3:bf16`, or `qwen3:q8_0`.
pub fn build(spec: &str, max_len: usize) -> Result<Box<dyn Embedder>> {
    let (kind, arg) = spec.split_once(':').unwrap_or((spec, ""));
    Ok(match kind {
        "fe" => Box::new(FeOnnx::new(arg, max_len)?),
        "fe-qwen3" => Box::new(FeQwen3::new(arg, max_len)?),
        "qwen3" => Box::new(OwnQwen3::new(Precision::parse(arg)?, max_len)?),
        _ => anyhow::bail!("unknown embedder kind {kind:?}"),
    })
}

fn qwen_query(t: &str) -> String {
    format!("Instruct: {TASK}\nQuery:{t}")
}

fn qwen_label_query(t: &str) -> String {
    format!("Instruct: {LABEL_TASK}\nQuery:{t}")
}

// ---------------------------------------------------------------- fastembed ONNX (ORT)

pub struct FeOnnx {
    model: fastembed::TextEmbedding,
    qprefix: &'static str,
    dprefix: &'static str,
}

impl FeOnnx {
    fn new(name: &str, max_len: usize) -> Result<Self> {
        use fastembed::EmbeddingModel as M;
        let bge_q = "Represent this sentence for searching relevant passages: ";
        let (m, qp, dp) = match name {
            "bge-small-en-v1.5" => (M::BGESmallENV15, bge_q, ""),
            "bge-small-en-v1.5-q" => (M::BGESmallENV15Q, bge_q, ""),
            "bge-base-en-v1.5-q" => (M::BGEBaseENV15Q, bge_q, ""),
            "all-minilm-l6-v2-q" => (M::AllMiniLML6V2Q, "", ""),
            "multilingual-e5-small" => (M::MultilingualE5Small, "query: ", "passage: "),
            "snowflake-arctic-xs-q" => (M::SnowflakeArcticEmbedXSQ, bge_q, ""),
            "embeddinggemma-300m-q" => (
                M::EmbeddingGemma300MQ,
                "task: search result | query: ",
                "title: none | text: ",
            ),
            _ => anyhow::bail!("unknown fastembed model {name:?}"),
        };
        let opts = fastembed::TextInitOptions::new(m)
            .with_max_length(max_len)
            .with_show_download_progress(false);
        let model = fastembed::TextEmbedding::try_new(opts)?;
        Ok(Self {
            model,
            qprefix: qp,
            dprefix: dp,
        })
    }
}

impl Embedder for FeOnnx {
    fn embed_queries(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let t: Vec<String> = texts
            .iter()
            .map(|t| format!("{}{t}", self.qprefix))
            .collect();
        Ok(self.model.embed(t, None)?)
    }
    fn embed_docs(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let t: Vec<String> = texts
            .iter()
            .map(|t| format!("{}{t}", self.dprefix))
            .collect();
        Ok(self.model.embed(t, None)?)
    }
}

// ---------------------------------------------------- fastembed candle Qwen3 (f32 / bf16)

pub struct FeQwen3(fastembed::Qwen3TextEmbedding);

impl FeQwen3 {
    fn new(dtype: &str, max_len: usize) -> Result<Self> {
        let dt = match dtype {
            "f32" => candle_core::DType::F32,
            "bf16" => candle_core::DType::BF16,
            "f16" => candle_core::DType::F16,
            _ => anyhow::bail!("fe-qwen3 dtype must be f32|bf16|f16"),
        };
        let m = fastembed::Qwen3TextEmbedding::from_hf(
            QWEN3_EMB,
            &candle_core::Device::Cpu,
            dt,
            max_len,
        )?;
        Ok(Self(m))
    }
}

impl Embedder for FeQwen3 {
    fn embed_label_queries(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let t: Vec<String> = texts.iter().map(|t| qwen_label_query(t)).collect();
        self.embed_docs(&t)
    }
    fn embed_queries(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let t: Vec<String> = texts.iter().map(|t| qwen_query(t)).collect();
        Ok(self.0.embed(&t)?)
    }
    fn embed_docs(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut out = Vec::new();
        for chunk in texts.chunks(16) {
            out.extend(self.0.embed(chunk)?);
        }
        Ok(out)
    }
}

// ------------------------------------------------------ own candle Qwen3 (f32 / quantized)

pub struct OwnQwen3 {
    model: Qwen3,
    tok: Tokenizer,
    eos: u32,
    max_len: usize,
}

impl OwnQwen3 {
    fn new(p: Precision, max_len: usize) -> Result<Self> {
        let dir = hf_snapshot(
            QWEN3_EMB,
            &["config.json", "tokenizer.json", "model.safetensors"],
        )?;
        let tok = Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| anyhow!("{e}"))?;
        let eos = tok
            .token_to_id("<|endoftext|>")
            .ok_or_else(|| anyhow!("no eos token"))?;
        Ok(Self {
            model: Qwen3::load(&dir, p)?,
            tok,
            eos,
            max_len,
        })
    }

    /// Tokenize and make sure each sequence ends in `<|endoftext|>`, as the official
    /// Qwen3-Embedding recipe pools the hidden state of that final token.
    fn ids(&self, texts: &[String]) -> Result<Vec<Vec<u32>>> {
        texts
            .iter()
            .map(|t| {
                let enc = self
                    .tok
                    .encode(t.as_str(), true)
                    .map_err(|e| anyhow!("{e}"))?;
                let mut ids = enc.get_ids().to_vec();
                if ids.last() != Some(&self.eos) || ids.len() > self.max_len {
                    ids.truncate(self.max_len - 1);
                    ids.push(self.eos);
                }
                Ok(ids)
            })
            .collect()
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut out = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(16) {
            out.extend(self.model.embed_last_token(&self.ids(chunk)?)?);
        }
        Ok(out)
    }
}

impl Embedder for OwnQwen3 {
    fn embed_label_queries(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let t: Vec<String> = texts.iter().map(|t| qwen_label_query(t)).collect();
        self.embed(&t)
    }
    fn embed_queries(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let t: Vec<String> = texts.iter().map(|t| qwen_query(t)).collect();
        self.embed(&t)
    }
    fn embed_docs(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.embed(texts)
    }
}
