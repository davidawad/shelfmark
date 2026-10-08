//! Render a markdown comparison table from `semspike run` result files.

use anyhow::Result;
use serde_json::Value;

fn f(v: &Value, ptr: &str, scale: f64, digits: usize) -> String {
    match v.pointer(ptr).and_then(Value::as_f64) {
        Some(x) => format!("{:.*}", digits, x * scale),
        None => "—".into(),
    }
}

pub fn print(paths: &[String]) -> Result<()> {
    println!(
        "| profile | embedder | reranker | class@1 (coarse) | class@3 (coarse) | class@1 (c2f) | sub@1 (c2f) | sub recall@10 | sub@1 (rerank) | class@1 (rerank) | load s | index s | RSS MB | peak MB | embed p50 ms | embed p95 ms | rerank p50 ms | item p50 ms |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for p in paths {
        let v: Value = serde_json::from_slice(&std::fs::read(p)?)?;
        let load = v
            .pointer("/load/embedder_ms")
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            + v.pointer("/load/reranker_ms")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
        println!(
            "| {} | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {:.1} | {} | {} | {} | {} | {} | {} | {} |",
            v["profile"].as_str().unwrap_or("-"),
            v["embedder"].as_str().unwrap_or("-"),
            v["reranker"].as_str().map(|r| format!("`{r}`")).unwrap_or("—".into()),
            f(&v, "/summary/coarse_top1", 100.0, 0),
            f(&v, "/summary/coarse_top3", 100.0, 0),
            f(&v, "/summary/c2f_class_top1", 100.0, 0),
            f(&v, "/summary/c2f_sub_top1", 100.0, 0),
            f(&v, "/summary/c2f_sub_recall_at_n", 100.0, 0),
            f(&v, "/summary/rerank_sub_top1", 100.0, 0),
            f(&v, "/summary/rerank_class_top1", 100.0, 0),
            load / 1e3,
            f(&v, "/load/index_ms", 1e-3, 1),
            f(&v, "/mem/after_load/rss_mb", 1.0, 0),
            f(&v, "/mem/end/hwm_mb", 1.0, 0),
            f(&v, "/summary/embed_ms_p50", 1.0, 0),
            f(&v, "/summary/embed_ms_p95", 1.0, 0),
            f(&v, "/summary/rerank_ms_p50", 1.0, 0),
            f(&v, "/summary/item_ms_p50", 1.0, 0),
        );
    }
    Ok(())
}
