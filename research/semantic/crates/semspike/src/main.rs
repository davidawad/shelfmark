//! semspike — CPU semantic-tier spike over the LCC outline.
//!
//!   semspike run --embedder SPEC [--reranker SPEC] --out results/x.json
//!   semspike load --embedder SPEC | --reranker SPEC     # prefetch weights only
//!   semspike parity                                       # backend cross-checks
//!   semspike report results/*.json                        # markdown table
//!
//! Embedder SPEC: fe:<fastembed-name> | fe-qwen3:<f32|bf16> | qwen3:<f32|q8_0|q6k|q4k|q4_0>
//! Reranker SPEC: qwen3:<precision> | fe:<fastembed-reranker-name>

mod cascade;
mod data;
mod embedders;
mod mem;
mod qwen3;
mod report;
mod rerankers;

use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<()> {
    // fastembed defaults to ./.fastembed_cache; keep model weights out of the source tree.
    if std::env::var_os("FASTEMBED_CACHE_DIR").is_none() {
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            let dir = PathBuf::from(home).join(".cache").join("fastembed");
            std::env::set_var("FASTEMBED_CACHE_DIR", dir);
        }
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    let opts = parse_opts(&args[1.min(args.len())..]);
    match cmd {
        "run" => run(&opts),
        "load" => load(&opts),
        "parity" => parity(&opts),
        "fwd" => fwd(&opts),
        "report" => report::print(&args[1..]),
        _ => {
            eprintln!("usage: semspike run|load|parity|report  (see src/main.rs header)");
            std::process::exit(2)
        }
    }
}

fn parse_opts(a: &[String]) -> HashMap<String, String> {
    let mut m = HashMap::new();
    let mut i = 0;
    while i < a.len() {
        if let Some(k) = a[i].strip_prefix("--") {
            let v = a.get(i + 1).filter(|v| !v.starts_with("--")).cloned();
            i += if v.is_some() { 2 } else { 1 };
            m.insert(k.to_string(), v.unwrap_or_else(|| "true".into()));
        } else {
            i += 1;
        }
    }
    m
}

fn num(o: &HashMap<String, String>, k: &str, d: usize) -> usize {
    o.get(k).and_then(|v| v.parse().ok()).unwrap_or(d)
}

fn data_dir(o: &HashMap<String, String>) -> PathBuf {
    o.get("data")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn load(o: &HashMap<String, String>) -> Result<()> {
    let max_len = num(o, "max-len", 512);
    if let Some(e) = o.get("embedder") {
        let mut e = embedders::build(e, max_len)?;
        e.embed_docs(&["warm".into()])?;
    }
    if let Some(r) = o.get("reranker") {
        rerankers::build(r, max_len)?.score("warm", &["warm".into()])?;
    }
    println!("{}", json!({"ok": true, "mem": mem::snapshot()}));
    Ok(())
}

fn run(o: &HashMap<String, String>) -> Result<()> {
    let dd = data_dir(o);
    let outline = data::Outline::load(&dd.join("lcc-outline.json"))?;
    let mut samples = data::load_samples(&dd.join("samples.jsonl"))?;
    samples.truncate(num(o, "limit", samples.len()));
    let max_len = num(o, "max-len", 512);
    let p = cascade::Params {
        k_classes: num(o, "k-classes", 3),
        n_shortlist: num(o, "n", 10),
        n_rerank: num(o, "n", 10),
        max_chars: num(o, "max-chars", 1500),
    };
    let espec = o.get("embedder").context("--embedder required")?;
    let mem0 = mem::snapshot();

    let t = Instant::now();
    let mut e = embedders::build(espec, max_len)?;
    let embedder_load_ms = ms(t);
    let mem_embedder = mem::snapshot();

    let t = Instant::now();
    let ix = cascade::build_index(
        &outline,
        e.as_mut(),
        o.contains_key("center"),
        o.contains_key("flip"),
    )?;
    let index_ms = ms(t);
    eprintln!("index built in {index_ms:.0} ms");
    let n_labels = ix.class_emb.len() + ix.leaf_emb.len();

    let (mut r, reranker_load_ms) = match o.get("reranker") {
        Some(spec) => {
            let t = Instant::now();
            let r = rerankers::build(spec, num(o, "rerank-query-tokens", 256))?;
            (Some(r), Some(ms(t)))
        }
        None => (None, None),
    };
    let mem_loaded = mem::snapshot();

    // One untimed warm-up item so first-call allocation does not skew p50/p95.
    if let Some(s) = samples.first() {
        cascade::classify(&outline, &ix, e.as_mut(), r.as_deref_mut(), s, &p)?;
    }
    let mut items = Vec::with_capacity(samples.len());
    let t = Instant::now();
    for s in &samples {
        let it = cascade::classify(&outline, &ix, e.as_mut(), r.as_deref_mut(), s, &p)?;
        let rr = it
            .rerank_ms
            .map(|m| format!(" rerank {m:.0} ms"))
            .unwrap_or_default();
        eprintln!("{} embed {:.0} ms{rr}", it.id, it.embed_ms);
        items.push(it);
    }
    let wall_ms = ms(t);
    let summary = cascade::summarize(&outline, &items);
    let out = json!({
        "embedder": espec,
        "reranker": o.get("reranker"),
        "profile": o.get("profile"),
        "scheme": {"name": outline.scheme, "version": outline.version,
                    "classes": outline.classes.len(), "subclasses": ix.leaves.len()},
        "params": {"center": o.contains_key("center"), "flip": o.contains_key("flip"), "k_classes": p.k_classes, "n": p.n_shortlist, "max_chars": p.max_chars,
                   "max_len": max_len},
        "env": env_info(),
        "load": {"embedder_ms": embedder_load_ms, "reranker_ms": reranker_load_ms,
                 "index_ms": index_ms, "index_labels": n_labels,
                 "index_ms_per_label": index_ms / n_labels as f64},
        "mem": {"start": mem0, "after_embedder": mem_embedder, "after_load": mem_loaded,
                "end": mem::snapshot()},
        "wall_ms": wall_ms,
        "summary": summary,
        "items": items,
    });
    let text = serde_json::to_string_pretty(&out)?;
    match o.get("out") {
        Some(path) => std::fs::write(path, text)?,
        None => println!("{text}"),
    }
    eprintln!("{}", serde_json::to_string(&out["summary"])?);
    Ok(())
}

fn env_info() -> serde_json::Value {
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .map(str::to_string)
        })
        .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string());
    json!({
        "os": std::env::consts::OS, "arch": std::env::consts::ARCH, "cpu": cpu,
        "threads": std::thread::available_parallelism().map(|n| n.get()).ok(),
        "rayon_threads": std::env::var("RAYON_NUM_THREADS").ok(),
    })
}

/// Cross-check backends: own candle Qwen3 f32 vs fastembed's Qwen3 f32 (same weights, two
/// implementations), quantized vs f32 drift, and the reranker prefix cache vs full prompts.
fn parity(o: &HashMap<String, String>) -> Result<()> {
    let texts: Vec<String> = vec![
        "A treatise on the differential calculus with numerous examples.".into(),
        "English literature — English literature: British novels, poetry and drama".into(),
        "Pride and Prejudice. A novel of manners about the Bennet sisters.".into(),
    ];
    let cosine = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
    let reference = embedders::build("fe-qwen3:f32", 512)?.embed_docs(&texts)?;
    let mut rows = Vec::new();
    for spec in ["qwen3:f32", "qwen3:q8_0", "qwen3:q4k"] {
        let got = embedders::build(spec, 512)?.embed_docs(&texts)?;
        let c: Vec<f32> = got
            .iter()
            .zip(&reference)
            .map(|(a, b)| cosine(a, b))
            .collect();
        rows.push(json!({"embedder": spec, "cos_vs_fastembed_f32": c}));
    }
    let precision = o
        .get("rerank-precision")
        .map(String::as_str)
        .unwrap_or("f32");
    let mut rr = rerankers::Qwen3Rerank::new_pub(qwen3::Precision::parse(precision)?)?;
    let docs: Vec<String> = texts[1..].to_vec();
    let cached = rerankers::Reranker::score(&mut rr, &texts[2], &docs)?;
    let full = rr.score_uncached(&texts[2], &docs)?;
    let out = json!({"embedding": rows,
        "reranker": {"precision": precision, "cached": cached, "uncached": full}});
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

/// Forward-pass microbenchmark: one query of about `--words` words, `--reps` times.
fn fwd(o: &HashMap<String, String>) -> Result<()> {
    let mut e = embedders::build(o.get("embedder").context("--embedder")?, 512)?;
    let text = vec!["library classification of books ".repeat(num(o, "words", 60) / 4)];
    for _ in 0..num(o, "reps", 3) {
        let t = Instant::now();
        e.embed_queries(&text)?;
        eprintln!("query fwd {:.0} ms", ms(t));
    }
    let docs: Vec<String> = (0..16)
        .map(|i| format!("label {i} of some class"))
        .collect();
    let t = Instant::now();
    e.embed_docs(&docs)?;
    eprintln!("16 short docs {:.0} ms", ms(t));
    Ok(())
}
