//! The fitting pipeline: average linkage, rotation local search, shelf
//! orientation, exact collapse, order-preserving local search.

use crate::binary::{average_linkage, orient, rotate};
use crate::classtree::{ClassTree, Params};
use crate::collapse::collapse;
use crate::graph::Graph;
use crate::search::improve;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct FitConfig {
    pub params: Params,
    pub seed: u64,
    pub rotation_passes: usize,
    pub search_rounds: usize,
}

impl Default for FitConfig {
    fn default() -> Self {
        FitConfig {
            params: Params::default(),
            seed: 20261010,
            rotation_passes: 20,
            search_rounds: 10,
        }
    }
}

/// Objective after one stage. Binary stages have no navigation term (every
/// item would be its own class), so only their Dasgupta cost is reported.
#[derive(Debug, Clone, Serialize)]
pub struct Stage {
    pub name: &'static str,
    pub dasgupta: f64,
    pub nav: Option<f64>,
    pub total: Option<f64>,
    pub seconds: f64,
    pub detail: serde_json::Value,
}

pub struct Fit {
    pub tree: ClassTree,
    pub stages: Vec<Stage>,
}

pub fn fit(g: &Graph, cfg: &FitConfig) -> Result<Fit, String> {
    let nw = g.n as f64 * g.total;
    let mut stages = Vec::new();
    let mut clock = Instant::now();
    let mut lap = || {
        let s = clock.elapsed().as_secs_f64();
        clock = Instant::now();
        s
    };
    let mut b = average_linkage(g);
    stages.push(Stage {
        name: "average_linkage",
        dasgupta: b.dasgupta(g) / nw,
        nav: None,
        total: None,
        seconds: lap(),
        detail: serde_json::json!({}),
    });
    let rotations = rotate(&mut b, g, cfg.seed, cfg.rotation_passes);
    stages.push(Stage {
        name: "rotation_local_search",
        dasgupta: b.dasgupta(g) / nw,
        nav: None,
        total: None,
        seconds: lap(),
        detail: serde_json::json!({ "rotations": rotations }),
    });
    orient(&mut b, g);
    let collapsed = collapse(&b, g, &cfg.params)?;
    let mut tree = collapsed.tree;
    tree.validate(&cfg.params)?;
    let o = tree.evaluate(g, &cfg.params);
    stages.push(Stage {
        name: "collapse",
        dasgupta: o.dasgupta,
        nav: Some(o.nav),
        total: Some(o.total),
        seconds: lap(),
        detail: serde_json::json!({ "dp_total": collapsed.dp_total, "classes": tree.nodes.len() - 1 }),
    });
    let stats = improve(&mut tree, g, &cfg.params, cfg.seed, cfg.search_rounds);
    tree.validate(&cfg.params)?;
    let o = tree.evaluate(g, &cfg.params);
    stages.push(Stage {
        name: "order_preserving_local_search",
        dasgupta: o.dasgupta,
        nav: Some(o.nav),
        total: Some(o.total),
        seconds: lap(),
        detail: serde_json::json!({ "stats": stats, "classes": tree.nodes.len() - 1 }),
    });
    Ok(Fit { tree, stages })
}
