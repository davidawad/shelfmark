//! Resident and peak memory of this process (Linux `/proc/self/status`; `null` elsewhere).

use serde_json::{json, Value};

fn field_mb(status: &str, key: &str) -> Option<f64> {
    let line = status.lines().find(|l| l.starts_with(key))?;
    let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024.0)
}

pub fn snapshot() -> Value {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    json!({
        "rss_mb": field_mb(&s, "VmRSS:"),
        "hwm_mb": field_mb(&s, "VmHWM:"),
        "rss_anon_mb": field_mb(&s, "RssAnon:"),
        "rss_file_mb": field_mb(&s, "RssFile:"),
    })
}
