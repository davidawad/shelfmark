//! Benchmark the `shortlist` tier with BGE-small-en-v1.5 int8 over the
//! bundled LCC scheme. Run on a remote box only:
//!
//! ```text
//! cargo run --release -p shelfmark-tier-shortlist --features fastembed \
//!     --example shortlist-bench -- [out.json]
//! ```
//!
//! Measures model load, cold label-index build, warm (disk cache) build,
//! per-item latency after one warm-up item, peak RSS and threads, and
//! writes the numbers as JSON (default `shortlist-bench.json` in the working directory).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{ClassifyInput, Tier};
use shelfmark_tier_shortlist::{model_threads, FastembedBge, LabelIndex, Params, ShortlistTier};

const ITEMS: [(&str, &str); 20] = [
    ("Compilers", "Lexical analysis, parsing, type checking and code generation for programming language compilers."),
    ("Battle Cry of Freedom", "The American Civil War from secession and Fort Sumter to Gettysburg, Lincoln and Reconstruction."),
    ("On the Origin of Species", "Natural selection, variation under domestication, the struggle for existence and the evolution of species."),
    ("The Wealth of Nations", "Division of labour, markets, prices, wages, profit and the accumulation of capital in national economies."),
    ("Gray's Anatomy", "Descriptive and surgical anatomy of the human body: bones, muscles, nerves, blood vessels and organs."),
    ("The Republic", "Plato's dialogue on justice, the ideal city, the philosopher king and the theory of forms."),
    ("A Treatise on Harmony", "Chords, counterpoint, voice leading and the theory of tonal harmony for composers and students of music."),
    ("Pride and Prejudice", "A novel of manners following Elizabeth Bennet and Mr Darcy in the English country gentry."),
    ("The Art of War", "Ancient Chinese military strategy: planning, tactics, terrain, spies and the conduct of armies in the field."),
    ("Principles of Soil Management", "Tillage, fertilizers, crop rotation, drainage and the fertility of farm soils."),
    ("Contracts in a Nutshell", "Offer and acceptance, consideration, breach, remedies and the common law of contracts."),
    ("Practical Navigation", "Celestial navigation, charts, the sextant, seamanship and handling of ships at sea."),
    ("The Elements of Euclid", "Plane geometry: points, lines, triangles, circles and proofs from axioms."),
    ("Democracy in America", "Tocqueville on political institutions, equality, associations and self-government in the United States."),
    ("Electrical Engineering Handbook", "Circuits, motors, generators, transmission lines and power distribution systems."),
    ("The Story of Painting", "European painting from the Renaissance masters to the Impressionists: techniques, schools and artists."),
    ("Library Cataloging", "Descriptive cataloging, subject headings, classification and the organization of library collections."),
    ("Teaching Children to Read", "Methods of reading instruction, curriculum and classroom practice in elementary schools."),
    ("A History of Ancient Rome", "The Roman Republic and Empire: the Caesars, the legions, the Senate and the fall of Rome."),
    ("Atlas of World Geography", "Continents, climates, landforms, maps and the physical and human geography of the world."),
];

fn status_kb(field: &str) -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    s.lines()
        .find(|l| l.starts_with(field))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

fn os_threads() -> usize {
    std::fs::read_dir("/proc/self/task").map_or(0, Iterator::count)
}

fn secs(t: Instant) -> f64 {
    t.elapsed().as_secs_f64()
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--phase") {
        let report = phase(&args[1] == "cold", Path::new(&args[2]));
        println!("{}", serde_json::to_string(&report).expect("json"));
        return;
    }
    let out_path = args
        .first()
        .map_or_else(|| PathBuf::from("shortlist-bench.json"), PathBuf::from);
    // Each phase runs in a fresh process so its memory figures are its own:
    // `cold` embeds the labels, `warm` is a later process reading them from
    // the disk cache (the steady state for a user).
    let label_cache =
        std::env::temp_dir().join(format!("shelfmark-shortlist-bench-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&label_cache);
    let me = std::env::current_exe().expect("own path");
    let run_phase = |name: &str| -> serde_json::Value {
        let out = std::process::Command::new(&me)
            .args(["--phase", name])
            .arg(&label_cache)
            .output()
            .expect("phase runs");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).expect("phase json")
    };
    let cold = run_phase("cold");
    let warm = run_phase("warm");
    let _ = std::fs::remove_dir_all(&label_cache);
    let report = serde_json::json!({
        "tier_version": cold["tier_version"],
        "model": "bge-small-en-v1.5-q (fastembed BGESmallENV15Q, Qdrant/bge-small-en-v1.5-onnx-Q)",
        "fastembed": "7.1.0 (features ort-download-binaries-rustls-tls, hf-hub-rustls-tls)",
        "box": {
            "available_parallelism": model_threads(),
            "cgroup_cpu_max": std::fs::read_to_string("/sys/fs/cgroup/cpu.max").ok().map(|s| s.trim().to_string()),
            "visible_cpus": std::fs::read_to_string("/proc/cpuinfo").map_or(0, |s| s.matches("processor\t:").count()),
        },
        "cold_process": cold,
        "warm_process": warm,
    });
    let text = serde_json::to_string_pretty(&report).expect("json");
    if let Some(dir) = out_path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    std::fs::write(&out_path, &text).expect("write report");
    println!("{text}");
}

/// One measured process. `cold`: embed the labels into an empty
/// `label_cache`, then time a second build from disk. Otherwise: the
/// labels must already be on disk.
fn phase(cold: bool, label_cache: &Path) -> serde_json::Value {
    let model_dir = shelfmark_tier_shortlist::model_dir();
    let model_cached = model_dir
        .join("models--Qdrant--bge-small-en-v1.5-onnx-Q")
        .exists();

    let lcc = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/schemes/lcc/lcc.json");
    let t = Instant::now();
    let index = shelfmark_scheme::compile_file(&lcc).expect("lcc compiles");
    let compile_s = secs(t);

    let threads = model_threads();
    let t = Instant::now();
    let model = FastembedBge::load().expect("model loads");
    let load_s = secs(t);
    let threads_after_load = os_threads();
    let rss_after_load = status_kb("VmRSS:");

    let mut cold_s = None;
    if cold {
        let t = Instant::now();
        let built = LabelIndex::build(&index, &model, Some(label_cache)).expect("labels embed");
        cold_s = Some(secs(t));
        assert!(!built.from_disk(), "cold phase needs an empty label cache");
    }
    let t = Instant::now();
    let warm = LabelIndex::build(&index, &model, Some(label_cache)).expect("labels load");
    let warm_s = secs(t);
    assert!(warm.from_disk(), "labels must come from the disk cache");
    let rss_after_labels = status_kb("VmRSS:");
    let labels = warm.labels().len();
    let level0 = warm.labels().iter().filter(|l| l.level == 0).count();

    let tier = ShortlistTier::new(
        Arc::new(model),
        Some(label_cache.to_path_buf()),
        Params::default(),
    );
    let run = |title: &str, text: &str| {
        let input = ClassifyInput {
            title: Some(title.into()),
            text: text.into(),
            ..ClassifyInput::default()
        };
        let t = Instant::now();
        let out = tier.run(&input, &index, &[]).expect("tier runs");
        (secs(t) * 1e3, out)
    };
    let (warmup_ms, _) = run("Warm-up", "A short warm-up text about libraries and books.");
    let mut ms = Vec::new();
    let mut placed = Vec::new();
    for (title, text) in ITEMS {
        let (m, out) = run(title, text);
        ms.push(m);
        let best = &out.candidates[0];
        let top = index
            .ancestors(&best.class.code)
            .last()
            .map(|r| r.id.code.clone());
        placed.push(serde_json::json!({
            "title": title,
            "class": best.class.code,
            "top_level": top,
            "score": best.score,
            "runner_up": out.candidates.get(1).map(|c| (c.class.code.clone(), c.score)),
            "ms": m,
        }));
    }
    let mut sorted = ms.clone();
    sorted.sort_by(f64::total_cmp);
    // The texts above are short (~100-150 chars). The worst case per item
    // is a full 1,500-character clip: time that separately.
    let long: String = ITEMS.iter().map(|(_, t)| *t).collect::<Vec<_>>().join(" ");
    let mut long_ms: Vec<f64> = (0..10).map(|_| run("Long", &long).0).collect();
    long_ms.sort_by(f64::total_cmp);
    let mb = |k: Option<u64>| k.map(|k| k as f64 / 1024.0);
    serde_json::json!({
        "phase": if cold { "cold" } else { "warm" },
        "tier_version": tier.info().version,
        "intra_op_threads": threads,
        "os_threads_after_load": threads_after_load,
        "scheme": { "path": "data/schemes/lcc/lcc.json", "classes": index.len(), "compile_s": compile_s },
        "labels": { "count": labels, "level0": level0, "level1": labels - level0, "dim": warm.dim() },
        "model_files_cached_before_run": model_cached,
        "model_load_s": load_s,
        "label_index_cold_s": cold_s,
        "label_index_from_disk_s": warm_s,
        "warmup_item_ms": warmup_ms,
        "per_item_ms": { "n": ms.len(), "p50": percentile(&sorted, 0.5), "p95": percentile(&sorted, 0.95),
                         "min": sorted[0], "max": sorted[sorted.len() - 1], "all": ms },
        "long_item_1500_chars_ms": { "n": long_ms.len(), "p50": percentile(&long_ms, 0.5), "p95": percentile(&long_ms, 0.95) },
        "rss_after_load_mb": mb(rss_after_load),
        "rss_after_labels_mb": mb(rss_after_labels),
        "rss_end_mb": mb(status_kb("VmRSS:")),
        "peak_rss_mb": mb(status_kb("VmHWM:")),
        "items": placed,
    })
}
