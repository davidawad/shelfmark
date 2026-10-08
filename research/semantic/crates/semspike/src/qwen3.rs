//! Minimal Qwen3 decoder (hidden states only) whose linear layers can be dense f32 or
//! quantized in-process to a GGML block format (Q8_0, Q4K, ...). Serves both
//! Qwen3-Embedding (last-token pooling) and Qwen3-Reranker (yes/no logit at the last token;
//! the 0.6B checkpoints tie `lm_head` to `embed_tokens`, so only two embedding rows are needed).
//!
//! Quantizing from the BF16 safetensors at load time (instead of shipping a GGUF) keeps a
//! single upstream artifact per model and lets one code path cover every precision.

use anyhow::{Context, Result};
use candle_core::quantized::{GgmlDType, QMatMul, QTensor};
use candle_core::{DType, Device, Module, Tensor, D};
use candle_nn::VarBuilder;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_attention_heads: usize,
    pub num_hidden_layers: usize,
    pub num_key_value_heads: usize,
    pub head_dim: Option<usize>,
    pub rms_norm_eps: f64,
    pub rope_theta: f64,
    pub vocab_size: usize,
    pub tie_word_embeddings: bool,
}

impl Config {
    fn head_dim(&self) -> usize {
        self.head_dim
            .unwrap_or(self.hidden_size / self.num_attention_heads)
    }
}

/// Weight precision for the transformer's linear layers (embeddings and norms stay f32).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    F32,
    Quant(GgmlDType),
}

impl Precision {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "f32" => Self::F32,
            "q8_0" => Self::Quant(GgmlDType::Q8_0),
            "q6k" => Self::Quant(GgmlDType::Q6K),
            "q4k" => Self::Quant(GgmlDType::Q4K),
            "q4_0" => Self::Quant(GgmlDType::Q4_0),
            other => anyhow::bail!("unknown precision {other:?} (f32|q8_0|q6k|q4k|q4_0)"),
        })
    }
}

struct QLin {
    q: QMatMul,
    w: std::sync::Arc<QTensor>,
}

enum Lin {
    Dense(Tensor),
    Quant(QLin),
}

/// Row count at which a quantized linear switches from the GGML kernel to dequantize+gemm
/// (override with SEMSPIKE_DEQUANT_ROWS; 0 = always dequantize, huge = never).
fn dequant_rows() -> usize {
    static ROWS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *ROWS.get_or_init(|| {
        std::env::var("SEMSPIKE_DEQUANT_ROWS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(16)
    })
}

impl Lin {
    fn load(vb: &VarBuilder, name: &str, out: usize, inp: usize, p: Precision) -> Result<Self> {
        let w = vb.get((out, inp), &format!("{name}.weight"))?;
        Ok(match p {
            Precision::F32 => Lin::Dense(w),
            Precision::Quant(dt) => {
                let w = std::sync::Arc::new(QTensor::quantize(&w, dt)?);
                Lin::Quant(QLin {
                    q: QMatMul::from_arc(w.clone())?,
                    w,
                })
            }
        })
    }

    fn forward(&self, xs: &Tensor) -> candle_core::Result<Tensor> {
        match self {
            // Flatten [B,T,in] to 2-D so the weight is not broadcast (copied) per batch row.
            Lin::Dense(w) => {
                let (b, t, i) = xs.dims3()?;
                xs.reshape((b * t, i))?.matmul(&w.t()?)?.reshape((b, t, ()))
            }
            Lin::Quant(QLin { q, w }) => {
                let (b, t, i) = xs.dims3()?;
                if b * t >= dequant_rows() {
                    // candle's quantized CPU kernel syncs its thread pool once per input row,
                    // so long prefills are faster as dequantize-this-layer + f32 gemm. Only one
                    // layer's f32 weights exist at a time; the resident copy stays quantized.
                    let wf = w.dequantize(&Device::Cpu)?;
                    xs.reshape((b * t, i))?
                        .matmul(&wf.t()?)?
                        .reshape((b, t, ()))
                } else {
                    q.forward(xs)
                }
            }
        }
    }
}

fn rms(xs: &Tensor, w: &Tensor, eps: f64) -> candle_core::Result<Tensor> {
    candle_nn::ops::rms_norm(&xs.contiguous()?, w, eps as f32)
}

struct Layer {
    q: Lin,
    k: Lin,
    v: Lin,
    o: Lin,
    q_norm: Tensor,
    k_norm: Tensor,
    gate: Lin,
    up: Lin,
    down: Lin,
    ln1: Tensor,
    ln2: Tensor,
}

/// Per-layer key/value tensors `[1, n_kv, T, D]` for a shared prompt prefix.
pub struct PrefixCache {
    kv: Vec<(Tensor, Tensor)>,
    pub len: usize,
}

pub struct Qwen3 {
    pub cfg: Config,
    embed: Tensor,
    layers: Vec<Layer>,
    norm: Tensor,
    cos: Tensor,
    sin: Tensor,
    pub device: Device,
}

const MAX_POS: usize = 4096;
const NEG: f32 = -1e9;

impl Qwen3 {
    pub fn load(dir: &Path, precision: Precision) -> Result<Self> {
        let device = Device::Cpu;
        let cfg: Config = serde_json::from_slice(&std::fs::read(dir.join("config.json"))?)
            .context("qwen3 config.json")?;
        anyhow::ensure!(cfg.tie_word_embeddings, "expected tied embeddings");
        let files = safetensor_files(dir)?;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&files, DType::F32, &device)? };
        let has_prefix = vb.contains_tensor("model.embed_tokens.weight");
        let vb = if has_prefix { vb.pp("model") } else { vb };
        let (h, i, hd) = (cfg.hidden_size, cfg.intermediate_size, cfg.head_dim());
        let (nq, nkv) = (cfg.num_attention_heads * hd, cfg.num_key_value_heads * hd);
        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        for l in 0..cfg.num_hidden_layers {
            let vb = vb.pp(format!("layers.{l}"));
            let a = vb.pp("self_attn");
            let m = vb.pp("mlp");
            layers.push(Layer {
                q: Lin::load(&a, "q_proj", nq, h, precision)?,
                k: Lin::load(&a, "k_proj", nkv, h, precision)?,
                v: Lin::load(&a, "v_proj", nkv, h, precision)?,
                o: Lin::load(&a, "o_proj", h, nq, precision)?,
                q_norm: a.get(hd, "q_norm.weight")?,
                k_norm: a.get(hd, "k_norm.weight")?,
                gate: Lin::load(&m, "gate_proj", i, h, precision)?,
                up: Lin::load(&m, "up_proj", i, h, precision)?,
                down: Lin::load(&m, "down_proj", h, i, precision)?,
                ln1: vb.get(h, "input_layernorm.weight")?,
                ln2: vb.get(h, "post_attention_layernorm.weight")?,
            });
        }
        // Keep the 151k-row embedding table in its native BF16 (lossless, half of f32);
        // gathered rows are widened to f32.
        let vb16 = unsafe { VarBuilder::from_mmaped_safetensors(&files, DType::BF16, &device)? };
        let vb16 = if has_prefix { vb16.pp("model") } else { vb16 };
        let embed = vb16.get((cfg.vocab_size, h), "embed_tokens.weight")?;
        let norm = vb.get(h, "norm.weight")?;
        let (cos, sin) = rope_tables(cfg.rope_theta, hd, &device)?;
        Ok(Self {
            cfg,
            embed,
            layers,
            norm,
            cos,
            sin,
            device,
        })
    }

    /// Run the decoder over `ids [B,T]` whose positions start at `offset`, attending to an
    /// optional cached prefix. `mask` is additive `[B,1,T,P+T]`. Returns final-norm hidden
    /// states `[B,T,H]`, and (when `keep_kv`) the new per-layer K/V.
    fn run(
        &self,
        ids: &Tensor,
        mask: &Tensor,
        offset: usize,
        prefix: Option<&PrefixCache>,
        keep_kv: bool,
    ) -> Result<(Tensor, Vec<(Tensor, Tensor)>)> {
        let (b, t) = ids.dims2()?;
        let hd = self.cfg.head_dim();
        let (nh, nkv) = (self.cfg.num_attention_heads, self.cfg.num_key_value_heads);
        let eps = self.cfg.rms_norm_eps;
        let cos = self.cos.narrow(0, offset, t)?;
        let sin = self.sin.narrow(0, offset, t)?;
        let scale = (hd as f64).powf(-0.5);
        let mut xs = self
            .embed
            .index_select(&ids.flatten_all()?, 0)?
            .to_dtype(DType::F32)?
            .reshape((b, t, ()))?;
        let mut new_kv = Vec::new();
        for (li, l) in self.layers.iter().enumerate() {
            let res = xs.clone();
            let h = rms(&xs, &l.ln1, eps)?;
            let q = l.q.forward(&h)?.reshape((b, t, nh, hd))?;
            let q = rms(&q, &l.q_norm, eps)?.transpose(1, 2)?.contiguous()?;
            let k = l.k.forward(&h)?.reshape((b, t, nkv, hd))?;
            let k = rms(&k, &l.k_norm, eps)?.transpose(1, 2)?.contiguous()?;
            let v =
                l.v.forward(&h)?
                    .reshape((b, t, nkv, hd))?
                    .transpose(1, 2)?
                    .contiguous()?;
            let q = candle_nn::rotary_emb::rope(&q, &cos, &sin)?;
            let k = candle_nn::rotary_emb::rope(&k, &cos, &sin)?;
            if keep_kv {
                new_kv.push((k.clone(), v.clone()));
            }
            let (k, v) = match prefix {
                Some(p) => {
                    let (pk, pv) = &p.kv[li];
                    let pk = pk.broadcast_as((b, nkv, p.len, hd))?;
                    let pv = pv.broadcast_as((b, nkv, p.len, hd))?;
                    (Tensor::cat(&[&pk, &k], 2)?, Tensor::cat(&[&pv, &v], 2)?)
                }
                None => (k, v),
            };
            let k = repeat_kv(&k, nh / nkv)?;
            let v = repeat_kv(&v, nh / nkv)?;
            let att = (q.matmul(&k.t()?.contiguous()?)? * scale)?.broadcast_add(mask)?;
            let att = candle_nn::ops::softmax_last_dim(&att)?;
            let out = att.matmul(&v)?.transpose(1, 2)?.reshape((b, t, nh * hd))?;
            xs = (res + l.o.forward(&out)?)?;
            let res = xs.clone();
            let h = rms(&xs, &l.ln2, eps)?;
            let mlp = (candle_nn::ops::silu(&l.gate.forward(&h)?)? * l.up.forward(&h)?)?;
            xs = (res + l.down.forward(&mlp)?)?;
        }
        Ok((rms(&xs, &self.norm, eps)?, new_kv))
    }

    /// Last-token hidden state for each sequence (left-padded batch), L2-normalised.
    pub fn embed_last_token(&self, seqs: &[Vec<u32>]) -> Result<Vec<Vec<f32>>> {
        let t = seqs.iter().map(Vec::len).max().unwrap_or(0);
        let (ids, valid) = pad(seqs, t, true);
        let ids = Tensor::from_vec(ids, (seqs.len(), t), &self.device)?;
        let mask = causal_mask(&valid, seqs.len(), 0, t, &self.device)?;
        let (hs, _) = self.run(&ids, &mask, 0, None, false)?;
        let last = hs.narrow(1, t - 1, 1)?.squeeze(1)?;
        let norm = last
            .sqr()?
            .sum_keepdim(D::Minus1)?
            .sqrt()?
            .clamp(1e-12, f64::MAX)?;
        Ok(last.broadcast_div(&norm)?.to_vec2()?)
    }

    pub fn prefix(&self, ids: &[u32]) -> Result<PrefixCache> {
        let t = ids.len();
        let mask = causal_mask(&[vec![true; t]], 1, 0, t, &self.device)?;
        let ids = Tensor::from_vec(ids.to_vec(), (1, t), &self.device)?;
        let (_, kv) = self.run(&ids, &mask, 0, None, true)?;
        Ok(PrefixCache { kv, len: t })
    }

    /// Logits for `token_ids` at the last real token of each continuation of `prefix`
    /// (right-padded batch). Uses the tied embedding matrix as the LM head.
    pub fn continuation_logits(
        &self,
        prefix: &PrefixCache,
        conts: &[Vec<u32>],
        token_ids: &[u32],
    ) -> Result<Vec<Vec<f32>>> {
        let t = conts.iter().map(Vec::len).max().unwrap_or(0);
        let (ids, valid) = pad(conts, t, false);
        let ids = Tensor::from_vec(ids, (conts.len(), t), &self.device)?;
        let mask = causal_mask(&valid, conts.len(), prefix.len, t, &self.device)?;
        let (hs, _) = self.run(&ids, &mask, prefix.len, Some(prefix), false)?;
        let idx = Tensor::from_vec(token_ids.to_vec(), token_ids.len(), &self.device)?;
        let head = self.embed.index_select(&idx, 0)?.to_dtype(DType::F32)?; // [K,H]
        let mut out = Vec::with_capacity(conts.len());
        for (i, c) in conts.iter().enumerate() {
            let h = hs.get(i)?.get(c.len() - 1)?.unsqueeze(0)?; // [1,H]
            out.push(h.matmul(&head.t()?)?.squeeze(0)?.to_vec1()?);
        }
        Ok(out)
    }
}

fn safetensor_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut v: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "safetensors"))
        .collect();
    v.sort();
    anyhow::ensure!(!v.is_empty(), "no safetensors in {}", dir.display());
    Ok(v)
}

fn rope_tables(theta: f64, hd: usize, dev: &Device) -> Result<(Tensor, Tensor)> {
    let inv: Vec<f32> = (0..hd / 2)
        .map(|i| 1.0 / theta.powf(2.0 * i as f64 / hd as f64) as f32)
        .collect();
    let inv = Tensor::from_vec(inv, (1, hd / 2), dev)?;
    let pos = Tensor::arange(0u32, MAX_POS as u32, dev)?.to_dtype(DType::F32)?;
    let f = pos.reshape((MAX_POS, 1))?.broadcast_mul(&inv)?;
    Ok((f.cos()?, f.sin()?))
}

fn repeat_kv(x: &Tensor, n: usize) -> candle_core::Result<Tensor> {
    if n == 1 {
        return Ok(x.clone());
    }
    let (b, kv, t, d) = x.dims4()?;
    x.unsqueeze(2)?
        .broadcast_as((b, kv, n, t, d))?
        .reshape((b, kv * n, t, d))
}

/// Pad to `t` with token 0; returns flat ids and per-position validity.
fn pad(seqs: &[Vec<u32>], t: usize, left: bool) -> (Vec<u32>, Vec<Vec<bool>>) {
    let mut ids = Vec::with_capacity(seqs.len() * t);
    let mut valid = Vec::with_capacity(seqs.len());
    for s in seqs {
        let n = t - s.len();
        let mut v = vec![false; t];
        if left {
            ids.extend(std::iter::repeat_n(0, n).chain(s.iter().copied()));
            v[n..].iter_mut().for_each(|x| *x = true);
        } else {
            ids.extend(s.iter().copied().chain(std::iter::repeat_n(0, n)));
            v[..s.len()].iter_mut().for_each(|x| *x = true);
        }
        valid.push(v);
    }
    (ids, valid)
}

/// Additive mask `[B,1,T,P+T]`: every query sees the whole (unpadded) prefix, then causal
/// attention over valid keys of its own sequence. Pad query rows keep their own key open so
/// softmax never sees an all-masked row.
fn causal_mask(valid: &[Vec<bool>], b: usize, p: usize, t: usize, dev: &Device) -> Result<Tensor> {
    let w = p + t;
    let mut m = vec![0f32; b * t * w];
    for (bi, v) in valid.iter().enumerate() {
        for i in 0..t {
            for j in 0..t {
                if j > i || (!v[j] && i != j) {
                    m[bi * t * w + i * w + p + j] = NEG;
                }
            }
        }
    }
    Ok(Tensor::from_vec(m, (b, 1, t, w), dev)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_left_and_right() {
        let (ids, v) = pad(&[vec![7, 8], vec![9]], 2, true);
        assert_eq!(ids, vec![7, 8, 0, 9]);
        assert_eq!(v, vec![vec![true, true], vec![false, true]]);
        let (ids, v) = pad(&[vec![9]], 2, false);
        assert_eq!((ids, v), (vec![9, 0], vec![vec![true, false]]));
    }

    #[test]
    fn mask_sees_prefix_is_causal_and_hides_pads() {
        // one sequence, prefix 2, continuation [real, pad]
        let m = causal_mask(&[vec![true, false]], 1, 2, 2, &Device::Cpu).unwrap();
        let m: Vec<Vec<f32>> = m.squeeze(0).unwrap().squeeze(0).unwrap().to_vec2().unwrap();
        assert_eq!(m[0], vec![0.0, 0.0, 0.0, NEG]); // real token: prefix + itself
                                                    // pad row: never all-masked (its own key stays open); its output is never read
        assert_eq!(m[1], vec![0.0, 0.0, 0.0, 0.0]);
        // left-padded batch: the real row must not see the leading pad key
        let m = causal_mask(&[vec![false, true]], 1, 0, 2, &Device::Cpu).unwrap();
        let m: Vec<Vec<f32>> = m.squeeze(0).unwrap().squeeze(0).unwrap().to_vec2().unwrap();
        assert_eq!(m[1], vec![NEG, 0.0]);
    }

    #[test]
    fn precision_parse() {
        assert_eq!(Precision::parse("f32").unwrap(), Precision::F32);
        assert!(Precision::parse("q8_0").is_ok());
        assert!(Precision::parse("int3").is_err());
    }
}
