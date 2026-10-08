//! Reranker backends: Qwen3-Reranker-0.6B on this crate's candle Qwen3 (any precision, with
//! the shared Instruct+Query prefix computed once per item), and fastembed ONNX cross-encoders.

use crate::embedders::{hf_snapshot, TASK};
use crate::qwen3::{Precision, Qwen3};
use anyhow::{anyhow, Result};
use tokenizers::Tokenizer;

pub const QWEN3_RERANK: &str = "Qwen/Qwen3-Reranker-0.6B";

pub trait Reranker {
    /// One relevance score per document (higher is better).
    fn score(&mut self, query: &str, docs: &[String]) -> Result<Vec<f32>>;
}

pub fn build(spec: &str, max_len: usize) -> Result<Box<dyn Reranker>> {
    let (kind, arg) = spec.split_once(':').unwrap_or((spec, ""));
    Ok(match kind {
        "qwen3" => Box::new(Qwen3Rerank::new(Precision::parse(arg)?, max_len)?),
        "fe" => Box::new(FeRerank::new(arg, max_len)?),
        _ => anyhow::bail!("unknown reranker kind {kind:?}"),
    })
}

pub struct Qwen3Rerank {
    model: Qwen3,
    tok: Tokenizer,
    yes_no: [u32; 2],
    max_query_tokens: usize,
}

const PREFIX: &str = "<|im_start|>system\nJudge whether the Document meets the requirements based on the Query and the Instruct provided. Note that the answer can only be \"yes\" or \"no\".<|im_end|>\n<|im_start|>user\n";
const SUFFIX: &str = "<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n";

impl Qwen3Rerank {
    pub fn new_pub(p: Precision) -> Result<Self> {
        Self::new(p, 256)
    }

    fn new(p: Precision, max_len: usize) -> Result<Self> {
        let dir = hf_snapshot(
            QWEN3_RERANK,
            &["config.json", "tokenizer.json", "model.safetensors"],
        )?;
        let tok = Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| anyhow!("{e}"))?;
        let id = |t: &str| {
            tok.token_to_id(t)
                .ok_or_else(|| anyhow!("token {t:?} missing"))
        };
        let yes_no = [id("yes")?, id("no")?];
        Ok(Self {
            model: Qwen3::load(&dir, p)?,
            tok,
            yes_no,
            max_query_tokens: max_len,
        })
    }

    fn enc(&self, s: &str) -> Result<Vec<u32>> {
        Ok(self
            .tok
            .encode(s, false)
            .map_err(|e| anyhow!("{e}"))?
            .get_ids()
            .to_vec())
    }

    /// Unshared reference path (full prompt per document) used to check the prefix cache.
    pub fn score_uncached(&mut self, query: &str, docs: &[String]) -> Result<Vec<f32>> {
        let mut out = Vec::new();
        for d in docs {
            let mut ids = self.enc(PREFIX)?;
            ids.extend(self.query_ids(query)?);
            let prefix = self.model.prefix(&ids[..1])?;
            let mut cont = ids[1..].to_vec();
            cont.extend(self.enc(&format!("\n<Document>: {d}{SUFFIX}"))?);
            let l = self
                .model
                .continuation_logits(&prefix, &[cont], &self.yes_no)?;
            out.push(yes_prob(&l[0]));
        }
        Ok(out)
    }

    fn query_ids(&self, query: &str) -> Result<Vec<u32>> {
        let mut q = self.enc(&format!("<Instruct>: {TASK}\n<Query>: "))?;
        let mut body = self.enc(query)?;
        body.truncate(self.max_query_tokens);
        q.extend(body);
        Ok(q)
    }
}

fn yes_prob(l: &[f32]) -> f32 {
    // softmax over [yes, no]
    let (y, n) = (l[0], l[1]);
    let m = y.max(n);
    let (ey, en) = ((y - m).exp(), (n - m).exp());
    ey / (ey + en)
}

impl Reranker for Qwen3Rerank {
    fn score(&mut self, query: &str, docs: &[String]) -> Result<Vec<f32>> {
        let mut ids = self.enc(PREFIX)?;
        ids.extend(self.query_ids(query)?);
        let prefix = self.model.prefix(&ids)?;
        let conts: Vec<Vec<u32>> = docs
            .iter()
            .map(|d| self.enc(&format!("\n<Document>: {d}{SUFFIX}")))
            .collect::<Result<_>>()?;
        let mut out = Vec::with_capacity(docs.len());
        for chunk in conts.chunks(8) {
            for l in self
                .model
                .continuation_logits(&prefix, chunk, &self.yes_no)?
            {
                out.push(yes_prob(&l));
            }
        }
        Ok(out)
    }
}

pub struct FeRerank(fastembed::TextRerank);

impl FeRerank {
    fn new(name: &str, max_len: usize) -> Result<Self> {
        use fastembed::RerankerModel as M;
        let m = match name {
            "jina-reranker-v1-turbo-en" => M::JINARerankerV1TurboEn,
            "bge-reranker-base" => M::BGERerankerBase,
            "jina-reranker-v2-base-multilingual" => M::JINARerankerV2BaseMultilingual,
            _ => anyhow::bail!("unknown fastembed reranker {name:?}"),
        };
        let opts = fastembed::RerankInitOptions::new(m)
            .with_max_length(max_len)
            .with_show_download_progress(false);
        Ok(Self(fastembed::TextRerank::try_new(opts)?))
    }
}

impl Reranker for FeRerank {
    fn score(&mut self, query: &str, docs: &[String]) -> Result<Vec<f32>> {
        let refs: Vec<&str> = docs.iter().map(String::as_str).collect();
        let res = self.0.rerank(query, refs, false, Some(16))?;
        let mut out = vec![f32::MIN; docs.len()];
        for r in res {
            out[r.index] = r.score;
        }
        Ok(out)
    }
}
