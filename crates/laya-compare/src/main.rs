//! Experimental comparison (not a tier): answer the LCC coarse-to-fine task with
//! `laya-decision` choice questions capped at 20 options. Classes or subclasses beyond 20 are
//! cut to 20 with laya's own encoder shortlist (`predict_shortlist`), as upstream recommends.
//!
//!   cargo run --release -p laya-compare --features laya -- --data data --out results/laya.json

#[cfg(not(feature = "laya"))]
fn main() {
    eprintln!("laya-compare is feature-gated: rebuild with --features laya");
    std::process::exit(2);
}

#[cfg(feature = "laya")]
fn main() -> anyhow::Result<()> {
    imp::main()
}

#[cfg(feature = "laya")]
mod imp {
    use anyhow::{Context, Result};
    use laya::agent::{Agent, LoadOptions};
    use laya::{embed_fn_from_agent, predict_shortlist, Questions};
    use serde::Deserialize;
    use serde_json::{json, Map, Value};
    use std::time::Instant;

    const MAX_OPTIONS: usize = 20;

    #[derive(Deserialize)]
    struct Node {
        code: String,
        label: String,
        #[serde(default)]
        scope: Option<String>,
        #[serde(default)]
        subclasses: Vec<Node>,
    }

    #[derive(Deserialize)]
    struct Outline {
        classes: Vec<Node>,
    }

    #[derive(Deserialize)]
    struct Sample {
        id: String,
        text: String,
        gold_class: String,
        gold_subclass: String,
    }

    fn arg(name: &str, default: &str) -> String {
        let a: Vec<String> = std::env::args().collect();
        a.iter()
            .position(|x| x == name)
            .and_then(|i| a.get(i + 1).cloned())
            .unwrap_or_else(|| default.to_string())
    }

    fn rss() -> Value {
        let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
        let get = |k: &str| {
            s.lines()
                .find(|l| l.starts_with(k))
                .and_then(|l| l.split_whitespace().nth(1)?.parse::<f64>().ok())
                .map(|kb| kb / 1024.0)
        };
        json!({"rss_mb": get("VmRSS:"), "hwm_mb": get("VmHWM:")})
    }

    /// Choice criteria keyed by human-readable caption (what the model reads), plus the
    /// caption -> code map to translate answers back.
    fn criteria(nodes: &[Node]) -> (Value, Vec<(String, String)>) {
        let mut m = Map::new();
        let mut back = Vec::new();
        for n in nodes {
            let mut key = n.label.clone();
            if m.contains_key(&key) {
                key = format!("{} ({})", n.label, n.code);
            }
            let desc = n.scope.clone().unwrap_or_else(|| n.label.clone());
            m.insert(key.clone(), json!(desc));
            back.push((key, n.code.clone()));
        }
        (Value::Object(m), back)
    }

    fn ask(agent: &Agent, state: &Value, nodes: &[Node]) -> Result<(String, f64, usize)> {
        let (crit, back) = criteria(nodes);
        let mut q = Questions::new();
        q.insert(
            "lcc".into(),
            json!({"type": "choice",
                   "instructions": "Which library classification category best fits the subject of `document`?",
                   "criteria": crit}),
        );
        let ans = if nodes.len() > MAX_OPTIONS {
            let embed = embed_fn_from_agent(agent, 512, 32);
            predict_shortlist(agent, state, &q, embed, MAX_OPTIONS)?
        } else {
            agent.system_one(state, &q)?.to_json()
        };
        let a = &ans["answers"]["lcc"];
        let key = a["choice"].as_str().context("no choice")?.to_string();
        let code = back
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, c)| c.clone())
            .context("map")?;
        let conf = a["answer_confidence"].as_f64().unwrap_or(f64::NAN);
        Ok((code, conf, nodes.len().min(MAX_OPTIONS)))
    }

    pub fn main() -> Result<()> {
        let dd = std::path::PathBuf::from(arg("--data", "data"));
        let outline: Outline =
            serde_json::from_slice(&std::fs::read(dd.join("lcc-outline.json"))?)?;
        let samples: Vec<Sample> = std::fs::read_to_string(dd.join("samples.jsonl"))?
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(serde_json::from_str)
            .collect::<std::result::Result<_, _>>()?;
        let model = arg("--model", "convaiinnovations/laya");
        let max_chars: usize = arg("--max-chars", "1500").parse()?;
        let mem0 = rss();
        let t = Instant::now();
        let agent = Agent::load(&model, LoadOptions::default())?;
        let load_ms = t.elapsed().as_secs_f64() * 1e3;
        let mem_loaded = rss();

        let mut items = Vec::new();
        let (mut ok_c, mut ok_s, mut lat) = (0usize, 0usize, Vec::new());
        let limit: usize = arg("--limit", "0").parse()?;
        let samples: Vec<Sample> = if limit > 0 {
            samples.into_iter().take(limit).collect()
        } else {
            samples
        };
        for s in &samples {
            let text: String = s.text.chars().take(max_chars).collect();
            let state = json!({ "document": text });
            let t = Instant::now();
            let (cls, cconf, copts) = ask(&agent, &state, &outline.classes)?;
            let c = outline
                .classes
                .iter()
                .find(|c| c.code == cls)
                .context("class")?;
            let (sub, sconf, sopts) = ask(&agent, &state, &c.subclasses)?;
            let ms = t.elapsed().as_secs_f64() * 1e3;
            ok_c += usize::from(cls == s.gold_class);
            ok_s += usize::from(sub == s.gold_subclass);
            lat.push(ms);
            eprintln!(
                "{} {}/{} -> {cls}/{sub} ({ms:.0} ms)",
                s.id, s.gold_class, s.gold_subclass
            );
            items.push(
                json!({"id": s.id, "gold_class": s.gold_class, "gold_subclass": s.gold_subclass,
                "class": cls, "class_conf": cconf, "class_options": copts,
                "subclass": sub, "subclass_conf": sconf, "subclass_options": sopts, "ms": ms}),
            );
        }
        lat.sort_by(f64::total_cmp);
        let p = |q: f64| lat[((lat.len() - 1) as f64 * q).round() as usize];
        let n = samples.len() as f64;
        let out = json!({
            "profile": "laya-experimental", "embedder": format!("laya:{model}"), "reranker": null,
            "max_options": MAX_OPTIONS,
            "load": {"embedder_ms": load_ms},
            "mem": {"start": mem0, "after_load": mem_loaded, "end": rss()},
            "summary": {"n": samples.len(), "coarse_top1": ok_c as f64 / n,
                        "c2f_sub_top1": ok_s as f64 / n,
                        "item_ms_p50": p(0.5), "item_ms_p95": p(0.95)},
            "items": items,
        });
        let text = serde_json::to_string_pretty(&out)?;
        std::fs::write(arg("--out", "results/laya.json"), text)?;
        eprintln!("{}", out["summary"]);
        Ok(())
    }
}
