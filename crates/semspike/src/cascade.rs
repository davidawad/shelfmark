//! Coarse-to-fine classification over a two-level outline: embed the item, rank the
//! top-level classes, shortlist subclasses inside the best `k_classes`, optionally rerank
//! the top `n_rerank` subclasses. Records per-item predictions, scores and timings.

use crate::data::{clip, Leaf, Outline, Sample};
use crate::embedders::Embedder;
use crate::rerankers::Reranker;
use anyhow::Result;
use serde::Serialize;
use std::time::Instant;

pub struct Index {
    pub class_emb: Vec<Vec<f32>>,
    pub leaves: Vec<Leaf>,
    pub leaf_emb: Vec<Vec<f32>>,
    /// Mean label vectors subtracted from both sides when centering (zeros otherwise).
    class_mean: Vec<f32>,
    leaf_mean: Vec<f32>,
    /// Labels were embedded as queries; items are embedded as passages.
    flip: bool,
}

/// Embed every label. With `center`, subtract the mean label vector per level (from labels
/// here and from queries at classify time): a training-free correction for "hub" labels
/// (e.g. A General Works) that sit close to everything.
pub fn build_index(o: &Outline, e: &mut dyn Embedder, center: bool, flip: bool) -> Result<Index> {
    let class_texts: Vec<String> = (0..o.classes.len()).map(|i| o.class_text(i)).collect();
    let leaves = o.leaves();
    let leaf_texts: Vec<String> = leaves.iter().map(|l| l.text.clone()).collect();
    let (mut class_emb, mut leaf_emb) = if flip {
        (
            e.embed_label_queries(&class_texts)?,
            e.embed_label_queries(&leaf_texts)?,
        )
    } else {
        (e.embed_docs(&class_texts)?, e.embed_docs(&leaf_texts)?)
    };
    let class_mean = mean_or_zero(&class_emb, center);
    let leaf_mean = mean_or_zero(&leaf_emb, center);
    class_emb.iter_mut().for_each(|v| sub(v, &class_mean));
    leaf_emb.iter_mut().for_each(|v| sub(v, &leaf_mean));
    Ok(Index {
        class_emb,
        leaves,
        leaf_emb,
        class_mean,
        leaf_mean,
        flip,
    })
}

fn mean_or_zero(rows: &[Vec<f32>], center: bool) -> Vec<f32> {
    let mut m = vec![0f32; rows.first().map_or(0, Vec::len)];
    if center {
        for r in rows {
            m.iter_mut()
                .zip(r)
                .for_each(|(a, b)| *a += b / rows.len() as f32);
        }
    }
    m
}

fn sub(v: &mut [f32], m: &[f32]) {
    v.iter_mut().zip(m).for_each(|(a, b)| *a -= b);
}

fn cos(a: &[f32], b: &[f32]) -> f32 {
    let (mut d, mut na, mut nb) = (0f32, 0f32, 0f32);
    for (x, y) in a.iter().zip(b) {
        d += x * y;
        na += x * x;
        nb += y * y;
    }
    d / (na.sqrt() * nb.sqrt()).max(1e-12)
}

fn ranked(scores: &[f32]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..scores.len()).collect();
    idx.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]));
    idx
}

#[derive(Serialize, Debug, Clone)]
pub struct ItemResult {
    pub id: String,
    pub gold_class: String,
    pub gold_subclass: String,
    pub coarse_top3: Vec<String>,
    pub coarse_margin: f32,
    pub c2f_shortlist: Vec<String>,
    pub c2f_margin: f32,
    pub flat_top1: String,
    pub rerank_top1: Option<String>,
    pub rerank_margin: Option<f32>,
    pub embed_ms: f64,
    pub shortlist_ms: f64,
    pub rerank_ms: Option<f64>,
}

pub struct Params {
    pub k_classes: usize,
    pub n_shortlist: usize,
    pub n_rerank: usize,
    pub max_chars: usize,
}

pub fn classify(
    o: &Outline,
    ix: &Index,
    e: &mut dyn Embedder,
    r: Option<&mut (dyn Reranker + '_)>,
    s: &Sample,
    p: &Params,
) -> Result<ItemResult> {
    let text = clip(&s.text, p.max_chars).to_string();
    let t0 = Instant::now();
    let one = std::slice::from_ref(&text);
    let q = if ix.flip {
        e.embed_docs(one)?
    } else {
        e.embed_queries(one)?
    }
    .remove(0);
    let embed_ms = t0.elapsed().as_secs_f64() * 1e3;

    let t1 = Instant::now();
    let (mut qc, mut ql) = (q.clone(), q);
    sub(&mut qc, &ix.class_mean);
    sub(&mut ql, &ix.leaf_mean);
    let cs: Vec<f32> = ix.class_emb.iter().map(|c| cos(&qc, c)).collect();
    let cr = ranked(&cs);
    let top_classes = &cr[..p.k_classes.min(cr.len())];
    let ls: Vec<f32> = ix.leaf_emb.iter().map(|l| cos(&ql, l)).collect();
    let lr = ranked(&ls);
    let c2f: Vec<usize> = lr
        .iter()
        .copied()
        .filter(|&i| top_classes.contains(&ix.leaves[i].class_idx))
        .collect();
    let shortlist: Vec<usize> = c2f.iter().copied().take(p.n_shortlist).collect();
    let shortlist_ms = t1.elapsed().as_secs_f64() * 1e3;

    let margin = |sc: &[f32], r: &[usize]| {
        if r.len() > 1 {
            sc[r[0]] - sc[r[1]]
        } else {
            1.0
        }
    };
    let (mut rerank_top1, mut rerank_margin, mut rerank_ms) = (None, None, None);
    if let Some(r) = r {
        let cand: Vec<usize> = shortlist.iter().copied().take(p.n_rerank).collect();
        let docs: Vec<String> = cand.iter().map(|&i| ix.leaves[i].text.clone()).collect();
        let t2 = Instant::now();
        let sc = r.score(&text, &docs)?;
        rerank_ms = Some(t2.elapsed().as_secs_f64() * 1e3);
        let rr = ranked(&sc);
        rerank_top1 = Some(ix.leaves[cand[rr[0]]].code.clone());
        rerank_margin = Some(margin(&sc, &rr));
    }

    Ok(ItemResult {
        id: s.id.clone(),
        gold_class: s.gold_class.clone(),
        gold_subclass: s.gold_subclass.clone(),
        coarse_top3: cr
            .iter()
            .take(3)
            .map(|&i| o.classes[i].code.clone())
            .collect(),
        coarse_margin: margin(&cs, &cr),
        c2f_shortlist: shortlist
            .iter()
            .map(|&i| ix.leaves[i].code.clone())
            .collect(),
        c2f_margin: margin(&ls, &c2f),
        flat_top1: ix.leaves[lr[0]].code.clone(),
        rerank_top1,
        rerank_margin,
        embed_ms,
        shortlist_ms,
        rerank_ms,
    })
}

#[derive(Serialize, Debug)]
pub struct Summary {
    pub n: usize,
    pub coarse_top1: f64,
    pub coarse_top3: f64,
    pub c2f_class_top1: f64,
    pub c2f_sub_top1: f64,
    pub c2f_sub_recall_at_n: f64,
    pub flat_sub_top1: f64,
    pub rerank_sub_top1: Option<f64>,
    pub rerank_class_top1: Option<f64>,
    pub embed_ms_p50: f64,
    pub embed_ms_p95: f64,
    pub rerank_ms_p50: Option<f64>,
    pub rerank_ms_p95: Option<f64>,
    pub item_ms_p50: f64,
}

fn class_of<'a>(o: &'a Outline, sub: &str) -> &'a str {
    o.classes
        .iter()
        .find(|c| c.subclasses.iter().any(|s| s.code == sub))
        .map(|c| c.code.as_str())
        .unwrap_or("?")
}

pub fn pct(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

pub fn summarize(o: &Outline, items: &[ItemResult]) -> Summary {
    let n = items.len() as f64;
    let rate = |f: &dyn Fn(&ItemResult) -> bool| items.iter().filter(|i| f(i)).count() as f64 / n;
    let has_rr = items.iter().all(|i| i.rerank_top1.is_some());
    let first = |i: &ItemResult| i.c2f_shortlist.first().cloned().unwrap_or_default();
    let mut emb: Vec<f64> = items.iter().map(|i| i.embed_ms).collect();
    let mut rr: Vec<f64> = items.iter().filter_map(|i| i.rerank_ms).collect();
    let mut tot: Vec<f64> = items
        .iter()
        .map(|i| i.embed_ms + i.shortlist_ms + i.rerank_ms.unwrap_or(0.0))
        .collect();
    Summary {
        n: items.len(),
        coarse_top1: rate(&|i| i.coarse_top3[0] == i.gold_class),
        coarse_top3: rate(&|i| i.coarse_top3.contains(&i.gold_class)),
        c2f_class_top1: rate(&|i| class_of(o, &first(i)) == i.gold_class),
        c2f_sub_top1: rate(&|i| first(i) == i.gold_subclass),
        c2f_sub_recall_at_n: rate(&|i| i.c2f_shortlist.contains(&i.gold_subclass)),
        flat_sub_top1: rate(&|i| i.flat_top1 == i.gold_subclass),
        rerank_sub_top1: has_rr
            .then(|| rate(&|i| i.rerank_top1.as_deref() == Some(i.gold_subclass.as_str()))),
        rerank_class_top1: has_rr.then(|| {
            rate(&|i| class_of(o, i.rerank_top1.as_deref().unwrap_or("")) == i.gold_class)
        }),
        embed_ms_p50: pct(&mut emb, 0.5),
        embed_ms_p95: pct(&mut emb, 0.95),
        rerank_ms_p50: has_rr.then(|| pct(&mut rr, 0.5)),
        rerank_ms_p95: has_rr.then(|| pct(&mut rr, 0.95)),
        item_ms_p50: pct(&mut tot, 0.5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranking_and_cosine() {
        assert_eq!(ranked(&[0.1, 0.9, 0.5]), vec![1, 2, 0]);
        assert!((cos(&[1.0, 0.0], &[2.0, 0.0]) - 1.0).abs() < 1e-6);
        assert!(cos(&[1.0, 0.0], &[0.0, 3.0]).abs() < 1e-6);
    }

    #[test]
    fn percentile_nearest_rank() {
        let mut v = vec![5.0, 1.0, 3.0];
        assert_eq!(pct(&mut v, 0.5), 3.0);
        assert_eq!(pct(&mut v, 1.0), 5.0);
        assert!(pct(&mut [], 0.5).is_nan());
    }
}
