//! Command implementations.

use crate::Args;
use learned_scheme::graph::{load_similarity, read_maybe_gz, Graph};
use learned_scheme::insert::{CentroidFile, InsertParams, Inserter, LeafCentroid};
use learned_scheme::scheme::{
    describe, label_sheet, read_labels, scheme_doc, scheme_json, source_description, Meta,
};
use learned_scheme::vectors::{self, normalize, Quantized, VectorFile, VectorRow};
use learned_scheme::{fit, ClassTree, FitConfig, Params};
use serde_json::json;
use shelfmark_bench::{sha256_hex, CorpusRow};
use shelfmark_core::scheme::SchemeDoc;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

type Res<T> = Result<T, String>;

fn read_text(path: &str) -> Res<String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

/// Corpus rows with their raw lines.
fn read_corpus(path: &str) -> Res<(Vec<CorpusRow>, Vec<String>)> {
    let text = read_text(path)?;
    let lines: Vec<String> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_owned)
        .collect();
    let rows = lines
        .iter()
        .enumerate()
        .map(|(k, l)| serde_json::from_str(l).map_err(|e| format!("{path} line {}: {e}", k + 1)))
        .collect::<Res<_>>()?;
    Ok((rows, lines))
}

fn read_ids(path: Option<&str>) -> Res<Option<BTreeSet<String>>> {
    path.map(|p| {
        Ok(read_text(p)?
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect())
    })
    .transpose()
}

/// Indices of `rows` kept by `ids` (all rows without a filter).
fn keep(rows: &[CorpusRow], ids: &Option<BTreeSet<String>>) -> Res<Vec<usize>> {
    let kept: Vec<usize> = (0..rows.len())
        .filter(|&i| ids.as_ref().is_none_or(|s| s.contains(&rows[i].id)))
        .collect();
    if let Some(s) = ids {
        if kept.len() != s.len() {
            return Err(format!(
                "{} of {} listed ids are in the corpus",
                kept.len(),
                s.len()
            ));
        }
    }
    Ok(kept)
}

fn write(path: &str, text: &str) -> Res<()> {
    if let Some(dir) = Path::new(path)
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
    {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("{path}: {e}"))
}

fn placement_line(id: &str, code: &str) -> String {
    json!({ "id": id, "classes": [{ "code": code, "mass": 1.0 }] }).to_string() + "\n"
}

fn config(a: &Args) -> Res<FitConfig> {
    let d = FitConfig::default();
    let p = Params::default();
    Ok(FitConfig {
        params: Params {
            lambda: a.num("lambda", p.lambda)?,
            branch_cost: a.num("branch-cost", p.branch_cost)?,
            max_depth: a.num("max-depth", p.max_depth)?,
            max_children: a.num("max-children", p.max_children)?,
            min_leaf: a.num("min-leaf", p.min_leaf)?,
            ..p
        },
        seed: a.num("seed", d.seed)?,
        ..d
    })
}

fn shape(tree: &ClassTree) -> serde_json::Value {
    let depth = tree.depths();
    let classes: Vec<usize> = tree.preorder().into_iter().filter(|&v| v != 0).collect();
    let mut by_depth = BTreeMap::new();
    for &v in &classes {
        *by_depth.entry(depth[v]).or_insert(0usize) += 1;
    }
    let mut sizes: Vec<usize> = tree.leaves().iter().map(|&l| tree.nodes[l].len()).collect();
    sizes.sort_unstable();
    let kids: Vec<usize> = tree
        .preorder()
        .into_iter()
        .filter(|&v| !tree.nodes[v].is_leaf())
        .map(|v| tree.nodes[v].children.len())
        .collect();
    let item_depth: f64 = tree
        .leaves()
        .iter()
        .map(|&l| (depth[l] * tree.nodes[l].len()) as f64)
        .sum::<f64>()
        / tree.n() as f64;
    json!({
        "classes": classes.len(),
        "leaf_classes": sizes.len(),
        "internal_classes": classes.len() - sizes.len(),
        "classes_by_depth": by_depth,
        "mean_item_class_depth": item_depth,
        "leaf_size": { "min": sizes.first(), "median": sizes.get(sizes.len() / 2), "max": sizes.last() },
        "children": { "min": kids.iter().min(), "max": kids.iter().max(), "mean": kids.iter().sum::<usize>() as f64 / kids.len().max(1) as f64, "root": tree.nodes[0].children.len() },
    })
}

pub fn build(a: &Args) -> Res<()> {
    let cfg = config(a)?;
    let (rows, _) = read_corpus(a.need("corpus")?)?;
    let graph_bytes = read_maybe_gz(Path::new(a.need("graph")?))?;
    let sim = load_similarity(Path::new(a.need("graph")?))?;
    if sim.n != rows.len() {
        return Err(format!("graph has {} items, corpus {}", sim.n, rows.len()));
    }
    let ids = read_ids(a.get("subset"))?;
    let kept = keep(&rows, &ids)?;
    let full = Graph::from_similarity(&sim)?;
    let g = if ids.is_some() {
        full.induced(&kept)
    } else {
        full
    };
    let rows: Vec<CorpusRow> = kept.iter().map(|&i| rows[i].clone()).collect();
    let fitted = fit(&g, &cfg)?;
    let tree = &fitted.tree;
    let infos = describe(tree, &g, &rows);
    let labels = a
        .get("labels")
        .map(read_text)
        .transpose()?
        .map(|t| read_labels(&t))
        .transpose()?;
    let corpus_sha = sha256_hex(read_text(a.need("corpus")?)?.as_bytes());
    let graph_sha = sha256_hex(&graph_bytes);
    let version = a.get("version").unwrap_or("2026-10-10").to_owned();
    let subset_of = ids.as_ref().map(|_| sim.n);
    let description = source_description(rows.len(), &corpus_sha, &graph_sha, subset_of, &cfg);
    let doc = scheme_doc(
        &infos,
        labels.as_ref(),
        a.flag("allow-auto-labels"),
        &Meta {
            version,
            description,
        },
    )?;
    let text = scheme_json(&doc)?;
    write(a.need("out")?, &text)?;
    let code_of: BTreeMap<usize, &str> = infos.iter().map(|c| (c.node, c.code.as_str())).collect();
    let leaf_of = tree.item_leaf();
    if let Some(path) = a.get("placements") {
        let out: String = rows
            .iter()
            .enumerate()
            .map(|(i, r)| placement_line(&r.id, code_of[&leaf_of[i]]))
            .collect();
        write(path, &out)?;
    }
    if let Some(path) = a.get("sheet") {
        write(path, &label_sheet(&infos))?;
    }
    if let (Some(path), Some(vpath)) = (a.get("centroids"), a.get("vectors")) {
        let all = vectors::read_f32(Path::new(vpath), sim.n)?;
        let mut leaves = Vec::new();
        for leaf in tree.leaves() {
            let node = &tree.nodes[leaf];
            let mut sum = vec![0.0f64; all[0].len()];
            for &item in &tree.order[node.lo..node.hi] {
                sum.iter_mut()
                    .zip(&all[kept[item]])
                    .for_each(|(s, &x)| *s += f64::from(x));
            }
            normalize(&mut sum);
            leaves.push(LeafCentroid {
                code: code_of[&leaf].to_owned(),
                n: node.len(),
                v: Quantized::new(&sum),
            });
        }
        let file = CentroidFile {
            scheme: format!("learned@{}", doc.version),
            model: a.get("model").unwrap_or(vectors::MODEL).into(),
            dims: all[0].len(),
            leaves,
        };
        vectors::write_json(Path::new(path), &file)?;
    }
    let report = json!({
        "scheme": format!("learned@{}", doc.version),
        "config": cfg,
        "inputs": { "corpus_sha256": corpus_sha, "graph_sha256": graph_sha, "n": g.n, "edges": g.edges.len(), "edge_weight": g.total,
                    "subset": ids.as_ref().map(|s| s.len()) },
        "stages": fitted.stages,
        "objective": tree.evaluate(&g, &cfg.params),
        "shape": shape(tree),
        "scheme_sha256": sha256_hex(text.as_bytes()),
    });
    let report = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n";
    match a.get("report") {
        Some(path) => write(path, &report),
        None => {
            print!("{report}");
            Ok(())
        }
    }
}

pub fn insert(a: &Args) -> Res<()> {
    let before: SchemeDoc =
        serde_json::from_str(&read_text(a.need("scheme")?)?).map_err(|e| e.to_string())?;
    let centroids: CentroidFile = vectors::read_json(Path::new(a.need("centroids")?))?;
    let vf: VectorFile = vectors::read_json(Path::new(a.need("vectors")?))?;
    let by_id: BTreeMap<&str, &VectorRow> = vf.rows.iter().map(|r| (r.id.as_str(), r)).collect();
    let (rows, _) = read_corpus(a.need("corpus")?)?;
    let kept = keep(&rows, &read_ids(a.get("ids"))?)?;
    let d = InsertParams::default();
    let params = InsertParams {
        split_after: a.num("split-after", d.split_after)?,
        min_child: a.num("min-child", d.min_child)?,
        max_cos: a.num("max-cos", d.max_cos)?,
        ..d
    };
    let mut ins = Inserter::new(before.clone(), &centroids, params)?;
    for &i in &kept {
        let r = &rows[i];
        let v = by_id
            .get(r.id.as_str())
            .ok_or(format!("no vector for {}", r.id))?;
        ins.insert(&r.id, &r.title, &v.v.unit());
    }
    ins.doc.version = format!("{}.ins{}", before.version, kept.len());
    learned_scheme::insert::check_stable(&before, &ins.doc)?;
    write(a.need("out")?, &scheme_json(&ins.doc)?)?;
    let mut out = String::new();
    if let Some(path) = a.get("base-placements") {
        for line in read_text(path)?.lines().filter(|l| !l.trim().is_empty()) {
            let v: serde_json::Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
            let id = v["id"].as_str().ok_or("placement without id")?;
            let code = v["classes"][0]["code"]
                .as_str()
                .ok_or("placement without class")?;
            out += &placement_line(id, &ins.refined(code));
        }
    }
    for &i in &kept {
        out += &placement_line(&rows[i].id, ins.class_of(&rows[i].id).unwrap());
    }
    write(a.need("placements")?, &out)?;
    let report = json!({
        "scheme": format!("learned@{}", ins.doc.version),
        "base": format!("learned@{}", before.version),
        "inserted": kept.len(),
        "params": params,
        "classes_before": before.classes.len(),
        "classes_after": ins.doc.classes.len(),
        "splits": ins.splits,
        "stable": true,
    });
    let text = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())? + "\n";
    match a.get("report") {
        Some(path) => write(path, &text),
        None => {
            print!("{text}");
            Ok(())
        }
    }
}

pub fn quantize(a: &Args) -> Res<()> {
    let (rows, _) = read_corpus(a.need("corpus")?)?;
    let all = vectors::read_f32(Path::new(a.need("vectors")?), rows.len())?;
    let kept = keep(&rows, &read_ids(a.get("ids"))?)?;
    let file = VectorFile {
        model: a.get("model").unwrap_or(vectors::MODEL).into(),
        dims: all[0].len(),
        rows: kept
            .iter()
            .map(|&i| VectorRow {
                id: rows[i].id.clone(),
                v: Quantized::new(&all[i].iter().map(|&x| f64::from(x)).collect::<Vec<_>>()),
            })
            .collect(),
    };
    vectors::write_json(Path::new(a.need("out")?), &file)
}

pub fn subset(a: &Args) -> Res<()> {
    let (rows, lines) = read_corpus(a.need("corpus")?)?;
    let sim = load_similarity(Path::new(a.need("graph")?))?;
    let kept = keep(&rows, &read_ids(Some(a.need("ids")?))?)?;
    let g = Graph::from_similarity(&sim)?.induced(&kept);
    write(
        a.need("out-corpus")?,
        &kept
            .iter()
            .map(|&i| lines[i].clone() + "\n")
            .collect::<String>(),
    )?;
    let text =
        serde_json::to_string(&g.to_similarity(sim.k, sim.tau)).map_err(|e| e.to_string())?;
    write(a.need("out-graph")?, &text)?;
    if let (Some(inp), Some(out)) = (a.get("placements"), a.get("out-placements")) {
        let ids: BTreeSet<&str> = kept.iter().map(|&i| rows[i].id.as_str()).collect();
        let text: String = read_text(inp)?
            .lines()
            .filter(|l| {
                serde_json::from_str::<serde_json::Value>(l)
                    .ok()
                    .and_then(|v| v["id"].as_str().map(|s| ids.contains(s)))
                    .unwrap_or(false)
            })
            .map(|l| l.to_owned() + "\n")
            .collect();
        write(out, &text)?;
    }
    Ok(())
}

pub fn split_ids(a: &Args) -> Res<()> {
    let (rows, _) = read_corpus(a.need("corpus")?)?;
    let fraction: f64 = a.num("fraction", 0.2)?;
    let seed = a.need("seed")?;
    let mut ranked: Vec<(String, usize)> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| (sha256_hex(format!("{seed}:{}", r.id).as_bytes()), i))
        .collect();
    ranked.sort();
    let take = (rows.len() as f64 * fraction).round() as usize;
    let test: BTreeSet<usize> = ranked.iter().take(take).map(|x| x.1).collect();
    let list = |want: bool| -> String {
        (0..rows.len())
            .filter(|i| test.contains(i) == want)
            .map(|i| rows[i].id.clone() + "\n")
            .collect()
    };
    write(a.need("train")?, &list(false))?;
    write(a.need("test")?, &list(true))
}
