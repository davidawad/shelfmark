//! Subcommand dispatch and flag parsing for the `vocab-ingest` binary.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use std::collections::HashMap;

use vocab_ingest::{enrich, eval, fast, io, lcsh, matcher, rdfxml, table};

pub fn run(args: &[String]) -> Result<(), String> {
    let (cmd, rest) = args
        .split_first()
        .ok_or("usage: vocab-ingest <lcsh|fast|sample|match|eval|enrich-scheme> [flags]")?;
    let flags = Flags::parse(rest)?;
    match cmd.as_str() {
        "lcsh" => cmd_lcsh(&flags),
        "fast" => cmd_fast(&flags),
        "match" => cmd_match(&flags),
        "eval" => cmd_eval(&flags),
        "sample" => cmd_sample(&flags),
        "enrich-scheme" => cmd_enrich(&flags),
        other => Err(format!("unknown command {other}")),
    }
}

fn cmd_lcsh(flags: &Flags) -> Result<(), String> {
    let started = Instant::now();
    let input = flags.path("input")?;
    let ingest = lcsh::ingest(io::open(&input).map_err(err(&input))?).map_err(|e| e.to_string())?;
    write_table(&flags.path("out")?, &ingest.headings)?;
    let stats = flags.path("stats")?;
    io::write_json(&stats, &ingest.stats).map_err(err(&stats))?;
    eprintln!(
        "lcsh: {} headings, {} active with LCC, {:.1}s",
        ingest.stats.headings,
        ingest.stats.active_with_lcc,
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

fn cmd_fast(flags: &Flags) -> Result<(), String> {
    let mut builder = fast::Builder::default();
    for nt in flags.all("nt") {
        let path = Path::new(nt);
        builder
            .add_ntriples(io::open(path).map_err(err(path))?)
            .map_err(err(path))?;
    }
    for dir in flags.all("rdfxml") {
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| format!("{dir}: {e}"))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.to_string_lossy().ends_with(".rdf.xml"))
            .collect();
        files.sort();
        for path in &files {
            let triples = rdfxml::read_triples(io::open(path).map_err(err(path))?)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            triples.iter().for_each(|t| builder.add(t));
        }
    }
    let lcsh = match flags.get("lcsh") {
        Some(p) => {
            let rows = load_table(Path::new(p))?;
            let map: HashMap<String, Vec<String>> =
                rows.into_iter().map(|h| (h.id, h.lcc)).collect();
            Some(map)
        }
        None => None,
    };
    let (rows, stats) = builder.finish(lcsh.as_ref());
    write_table(&flags.path("out")?, &rows)?;
    let stats_path = flags.path("stats")?;
    io::write_json(&stats_path, &stats).map_err(err(&stats_path))?;
    eprintln!(
        "fast: {} headings, {} with LCC via LCSH",
        stats.headings, stats.with_lcc_via_lcsh
    );
    Ok(())
}

fn build_matcher(flags: &Flags) -> Result<(matcher::Matcher, Vec<table::Heading>), String> {
    let started = Instant::now();
    let mut rows = Vec::new();
    for t in flags.all("table") {
        rows.extend(load_table(Path::new(t))?);
    }
    if rows.is_empty() {
        return Err("missing --table".into());
    }
    let kept = rows.clone();
    let m = matcher::Matcher::build(rows)?;
    eprintln!(
        "matcher: {} headings, {} patterns, built in {:.1}s",
        m.heading_count(),
        m.pattern_count(),
        started.elapsed().as_secs_f64()
    );
    Ok((m, kept))
}

fn cmd_match(flags: &Flags) -> Result<(), String> {
    let (m, _) = build_matcher(flags)?;
    let text = flags.get("text").ok_or("missing --text")?;
    let result = m.find(text);
    serde_json::to_writer_pretty(std::io::stdout(), &result).map_err(|e| e.to_string())?;
    println!();
    Ok(())
}

#[derive(serde::Serialize)]
struct EvalReport {
    tables: Vec<String>,
    headings: usize,
    patterns: usize,
    fixtures: eval::FixtureReport,
    self_check: eval::SelfCheckReport,
}

fn cmd_eval(flags: &Flags) -> Result<(), String> {
    let (m, rows) = build_matcher(flags)?;
    let path = flags.path("fixtures")?;
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let fixtures: Vec<eval::Fixture> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let n: usize = flags
        .get("self-check")
        .unwrap_or("2000")
        .parse()
        .map_err(|e| format!("--self-check: {e}"))?;
    let report = EvalReport {
        tables: flags
            .all("table")
            .iter()
            .map(|t| {
                Path::new(t)
                    .file_name()
                    .map_or(t.to_string(), |f| f.to_string_lossy().into_owned())
            })
            .collect(),
        headings: m.heading_count(),
        patterns: m.pattern_count(),
        fixtures: eval::run_fixtures(&m, &fixtures),
        self_check: eval::self_check(&m, &rows, n),
    };
    let out = flags.path("out")?;
    io::write_json(&out, &report).map_err(err(&out))?;
    let f = &report.fixtures;
    let s = &report.self_check;
    eprintln!(
        "fixtures: {}/{} subclass top1, {}/{} top3, {}/{} main class, {} no vote; self-check: {}/{} recovered, {}/{} own subclass top1",
        f.subclass_top1, f.cases, f.subclass_top3, f.cases, f.main_class_top1, f.cases, f.no_vote,
        s.heading_recovered, s.sampled, s.own_subclass_top1, s.sampled
    );
    Ok(())
}

/// Deterministic subset: every Nth row plus explicitly named ids.
fn cmd_sample(flags: &Flags) -> Result<(), String> {
    let rows = load_table(&flags.path("table")?)?;
    let every: usize = flags
        .get("every")
        .unwrap_or("2000")
        .parse()
        .map_err(|e| format!("--every: {e}"))?;
    let ids: Vec<&str> = flags
        .get("ids")
        .map(|s| s.split(',').collect())
        .unwrap_or_default();
    let picked: Vec<_> = rows
        .into_iter()
        .enumerate()
        .filter(|(i, h)| i % every.max(1) == 0 || ids.contains(&h.id.as_str()))
        .map(|(_, h)| h)
        .collect();
    write_table(&flags.path("out")?, &picked)?;
    eprintln!("sample: {} rows", picked.len());
    Ok(())
}

/// `--scheme lcc.json --table lcsh.tsv.gz --out lcc-lcsh.json [--alt true]
/// [--inherit N] [--title T] [--description D] [--canonical true]
/// [--stats stats.json]`: the scheme with every placed heading in its
/// vocabulary. `--canonical true` writes the bytes shelfmark hashes
/// (`serde_json::to_vec` of the parsed `SchemeDoc`, no trailing newline),
/// which is what the bundled `lcc-lcsh.json.gz` holds.
fn cmd_enrich(flags: &Flags) -> Result<(), String> {
    let scheme_path = flags.path("scheme")?;
    let bytes = std::fs::read(&scheme_path).map_err(err(&scheme_path))?;
    let index = shelfmark_scheme::compile_bytes(&bytes, &scheme_path.display().to_string())
        .map_err(|e| e.message)?;
    let mut doc: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", scheme_path.display()))?;
    let headings = load_table(&flags.path("table")?)?;
    let opts = enrich::Options {
        alt: flags.get("alt") == Some("true"),
        inherit: flags
            .get("inherit")
            .unwrap_or("0")
            .parse()
            .map_err(|e| format!("--inherit: {e}"))?,
    };
    let stats = enrich::enrich(&mut doc, &headings, &index, &opts)?;
    if let Some(title) = flags.get("title") {
        doc["title"] = title.into();
    }
    if let Some(description) = flags.get("description") {
        doc["source"]["description"] = description.into();
    }
    let out = flags.path("out")?;
    let bytes = if flags.get("canonical") == Some("true") {
        let parsed: shelfmark_core::scheme::SchemeDoc =
            serde_json::from_value(doc).map_err(|e| format!("enriched scheme: {e}"))?;
        serde_json::to_vec(&parsed).map_err(|e| e.to_string())?
    } else {
        (serde_json::to_string(&doc).map_err(|e| e.to_string())? + "\n").into_bytes()
    };
    std::fs::write(&out, bytes).map_err(err(&out))?;
    if let Some(path) = flags.get("stats") {
        io::write_json(Path::new(path), &stats).map_err(err(Path::new(path)))?;
    }
    eprintln!("enrich-scheme: {stats:?}");
    Ok(())
}

pub fn load_table(path: &Path) -> Result<Vec<table::Heading>, String> {
    table::read_table(io::open(path).map_err(err(path))?)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn write_table(path: &Path, rows: &[table::Heading]) -> Result<(), String> {
    let mut out = io::create(path).map_err(err(path))?;
    table::write_table(rows, &mut out).map_err(err(path))?;
    out.flush().map_err(err(path))
}

fn err(path: &Path) -> impl Fn(std::io::Error) -> String + '_ {
    move |e| format!("{}: {e}", path.display())
}

/// Parsed `--flag value` pairs; repeated flags keep every value.
pub struct Flags(Vec<(String, String)>);

impl Flags {
    pub fn parse(args: &[String]) -> Result<Flags, String> {
        let mut out = Vec::new();
        let mut it = args.iter();
        while let Some(a) = it.next() {
            let name = a
                .strip_prefix("--")
                .ok_or_else(|| format!("unexpected argument {a}"))?;
            let value = it.next().ok_or_else(|| format!("--{name} needs a value"))?;
            out.push((name.to_string(), value.clone()));
        }
        Ok(Flags(out))
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn all(&self, name: &str) -> Vec<&str> {
        self.0
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    pub fn path(&self, name: &str) -> Result<PathBuf, String> {
        self.get(name)
            .map(PathBuf::from)
            .ok_or_else(|| format!("missing --{name}"))
    }
}
