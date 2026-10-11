//! Inserting new works without renaming anything.
//!
//! A new work goes to the leaf class with the nearest centroid (cosine on
//! unit embeddings). A leaf then tries a two-means split, anchored on its
//! fitted centroid, in two cases:
//!
//! - distinct group: it has taken at least `split_after` new works and the
//!   split's two centroids have cosine at most `max_cos`;
//! - capacity: its new works reach its capacity, which is its fitted size,
//!   or the median fitted leaf size for a leaf made by insertion (so a leaf
//!   splits when it has doubled, and inserted material is kept at the leaf
//!   sizes the fit chose).
//!
//! Either way both sides need `min_child` works. The leaf gets two
//! children: `<code>.01` keeps the core (its earlier works and the new works
//! near them) and `<code>.02` takes the other group. Only leaf classes are
//! ever split, existing codes, labels, parents and order never change, and
//! an existing work's class can only become a descendant of its old class.

use crate::terms::{auto_label, words};
use crate::vectors::{dot, normalize, Quantized};
use serde::{Deserialize, Serialize};
use shelfmark_core::scheme::{ClassDef, SchemeDoc, VocabLink};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentroidFile {
    pub scheme: String,
    pub model: String,
    pub dims: usize,
    pub leaves: Vec<LeafCentroid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafCentroid {
    pub code: String,
    pub n: usize,
    #[serde(flatten)]
    pub v: Quantized,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct InsertParams {
    pub split_after: usize,
    pub min_child: usize,
    pub max_cos: f64,
    pub iterations: usize,
}

impl Default for InsertParams {
    fn default() -> Self {
        InsertParams {
            split_after: 12,
            min_child: 5,
            max_cos: 0.5,
            iterations: 10,
        }
    }
}

struct NewWork {
    id: String,
    title: String,
    v: Vec<f64>,
}

struct Leaf {
    core: Vec<f64>,
    n_old: usize,
    capacity: usize,
    new: Vec<NewWork>,
    unit: Vec<f64>,
}

impl Leaf {
    fn refresh(&mut self) {
        let mut sum: Vec<f64> = self.core.iter().map(|x| x * self.n_old as f64).collect();
        for w in &self.new {
            sum.iter_mut().zip(&w.v).for_each(|(s, x)| *s += x);
        }
        normalize(&mut sum);
        self.unit = sum;
    }
}

/// Checks the insertion guarantees against the scheme before insertion.
pub fn check_stable(before: &SchemeDoc, after: &SchemeDoc) -> Result<(), String> {
    let old: BTreeMap<&str, _> = before
        .classes
        .iter()
        .map(|c| (c.code.as_str(), c))
        .collect();
    let parents: BTreeSet<&str> = before
        .classes
        .iter()
        .filter_map(|c| c.parent.as_deref())
        .collect();
    let kept: Vec<&str> = after
        .classes
        .iter()
        .map(|c| c.code.as_str())
        .filter(|c| old.contains_key(c))
        .collect();
    let order: Vec<&str> = before.classes.iter().map(|c| c.code.as_str()).collect();
    if kept != order {
        return Err("existing classes changed order or disappeared".into());
    }
    for c in &after.classes {
        match old.get(c.code.as_str()) {
            Some(&o) if o != c => return Err(format!("class {} changed", c.code)),
            Some(_) => {}
            None => {
                let p = c
                    .parent
                    .as_deref()
                    .ok_or(format!("new class {} at top level", c.code))?;
                if parents.contains(p) {
                    return Err(format!("new class {} under internal class {p}", c.code));
                }
            }
        }
    }
    Ok(())
}

/// Centroids of the two halves of `vs` cut by the sign of the projection on
/// their first principal direction (principal direction divisive
/// partitioning, Boley 1998), found by power iteration from a fixed start.
fn principal_halves(vs: &[&[f64]]) -> (Vec<f64>, Vec<f64>) {
    let dims = vs[0].len();
    let mut mean = vec![0.0; dims];
    for v in vs {
        mean.iter_mut()
            .zip(*v)
            .for_each(|(m, x)| *m += x / vs.len() as f64);
    }
    let centred: Vec<Vec<f64>> = vs
        .iter()
        .map(|v| v.iter().zip(&mean).map(|(x, m)| x - m).collect())
        .collect();
    let mut u: Vec<f64> = (0..dims).map(|i| 1.0 + (i % 7) as f64).collect();
    normalize(&mut u);
    for _ in 0..100 {
        let mut next = vec![0.0; dims];
        for x in &centred {
            let s = dot(x, &u);
            next.iter_mut().zip(x).for_each(|(a, b)| *a += s * b);
        }
        normalize(&mut next);
        u = next;
    }
    let (mut a, mut b) = (vec![0.0; dims], vec![0.0; dims]);
    for (v, x) in vs.iter().zip(&centred) {
        let side = if dot(x, &u) > 0.0 { &mut a } else { &mut b };
        side.iter_mut().zip(*v).for_each(|(s, y)| *s += y);
    }
    normalize(&mut a);
    normalize(&mut b);
    (a, b)
}

#[derive(Debug, Clone, Serialize)]
pub struct Split {
    pub cause: &'static str,
    pub leaf: String,
    pub core: String,
    pub new: String,
    pub after_inserts: usize,
    pub core_new_works: usize,
    pub new_works: usize,
    pub centroid_cos: f64,
}

pub struct Inserter {
    pub doc: SchemeDoc,
    pub params: InsertParams,
    pub splits: Vec<Split>,
    leaves: BTreeMap<String, Leaf>,
    placed: BTreeMap<String, String>,
    inserted: usize,
    median_leaf: usize,
}

impl Inserter {
    pub fn new(
        doc: SchemeDoc,
        centroids: &CentroidFile,
        params: InsertParams,
    ) -> Result<Self, String> {
        let parents: BTreeSet<&str> = doc
            .classes
            .iter()
            .filter_map(|c| c.parent.as_deref())
            .collect();
        let mut sizes: Vec<usize> = centroids.leaves.iter().map(|c| c.n).collect();
        sizes.sort_unstable();
        let median_leaf = sizes.get(sizes.len() / 2).copied().unwrap_or(1).max(1);
        let mut leaves = BTreeMap::new();
        for c in &centroids.leaves {
            if parents.contains(c.code.as_str()) || !doc.classes.iter().any(|d| d.code == c.code) {
                return Err(format!("centroid for {} which is not a leaf class", c.code));
            }
            let core = c.v.unit();
            leaves.insert(
                c.code.clone(),
                Leaf {
                    unit: core.clone(),
                    core,
                    n_old: c.n,
                    capacity: c.n.max(1),
                    new: Vec::new(),
                },
            );
        }
        for d in &doc.classes {
            if !parents.contains(d.code.as_str()) && !leaves.contains_key(&d.code) {
                return Err(format!("leaf class {} has no centroid", d.code));
            }
        }
        Ok(Inserter {
            doc,
            params,
            splits: Vec::new(),
            leaves,
            placed: BTreeMap::new(),
            inserted: 0,
            median_leaf,
        })
    }

    /// Current class of an inserted work.
    pub fn class_of(&self, id: &str) -> Option<&str> {
        self.placed.get(id).map(String::as_str)
    }

    /// Current class of a work that was in leaf `code` before insertion:
    /// follows core children down through every split.
    pub fn refined(&self, code: &str) -> String {
        let mut code = code.to_owned();
        while let Some(s) = self.splits.iter().find(|s| s.leaf == code) {
            code = s.core.clone();
        }
        code
    }

    /// Files `v` (any length-matched embedding) and returns its class.
    pub fn insert(&mut self, id: &str, title: &str, v: &[f64]) -> String {
        let mut v = v.to_vec();
        normalize(&mut v);
        let mut best: Option<(&String, f64)> = None;
        for (code, leaf) in &self.leaves {
            let s = dot(&v, &leaf.unit);
            if best.is_none_or(|b| s > b.1) {
                best = Some((code, s));
            }
        }
        let code = best.expect("at least one leaf").0.clone();
        self.inserted += 1;
        let leaf = self.leaves.get_mut(&code).unwrap();
        leaf.new.push(NewWork {
            id: id.to_owned(),
            title: title.to_owned(),
            v,
        });
        leaf.refresh();
        self.placed.insert(id.to_owned(), code.clone());
        let leaf = &self.leaves[&code];
        let full = leaf.new.len() >= leaf.capacity;
        if full || leaf.new.len() >= self.params.split_after {
            self.try_split(&code, full);
        }
        self.placed[id].clone()
    }

    fn try_split(&mut self, code: &str, full: bool) {
        let p = self.params;
        let leaf = &self.leaves[code];
        let vs: Vec<&[f64]> = leaf.new.iter().map(|w| w.v.as_slice()).collect();
        let (mut c1, mut c2) = if full {
            principal_halves(&vs)
        } else {
            let c1 = if leaf.n_old > 0 {
                leaf.core.clone()
            } else {
                leaf.unit.clone()
            };
            let far = (0..vs.len())
                .min_by(|&a, &b| dot(vs[a], &c1).total_cmp(&dot(vs[b], &c1)))
                .unwrap();
            (c1, vs[far].to_vec())
        };
        // The side nearer the fitted core is the core side.
        if leaf.n_old > 0 && dot(&c2, &leaf.core) > dot(&c1, &leaf.core) {
            std::mem::swap(&mut c1, &mut c2);
        }
        let mut side = vec![false; leaf.new.len()];
        for _ in 0..p.iterations {
            for (k, w) in leaf.new.iter().enumerate() {
                side[k] = dot(&w.v, &c2) > dot(&w.v, &c1);
            }
            let mut s1: Vec<f64> = leaf.core.iter().map(|x| x * leaf.n_old as f64).collect();
            let mut s2 = vec![0.0; s1.len()];
            for (w, &second) in leaf.new.iter().zip(&side) {
                let s = if second { &mut s2 } else { &mut s1 };
                s.iter_mut().zip(&w.v).for_each(|(a, x)| *a += x);
            }
            normalize(&mut s1);
            normalize(&mut s2);
            c1 = s1;
            c2 = s2;
        }
        let n2 = side.iter().filter(|&&s| s).count();
        let n1 = leaf.n_old + leaf.new.len() - n2;
        let cos = dot(&c1, &c2);
        if n2 < p.min_child || n1 < p.min_child || (cos > p.max_cos && !full) {
            return;
        }
        let mut leaf = self.leaves.remove(code).unwrap();
        let (core_code, new_code) = (format!("{code}.01"), format!("{code}.02"));
        let (mut core_new, mut group) = (Vec::new(), Vec::new());
        for (w, second) in std::mem::take(&mut leaf.new).into_iter().zip(side) {
            if second {
                group.push(w);
            } else {
                core_new.push(w);
            }
        }
        self.splits.push(Split {
            cause: if cos <= p.max_cos {
                "distinct"
            } else {
                "capacity"
            },
            leaf: code.to_owned(),
            core: core_code.clone(),
            new: new_code.clone(),
            after_inserts: self.inserted,
            core_new_works: core_new.len(),
            new_works: group.len(),
            centroid_cos: cos,
        });
        self.add_children(code, &core_code, &new_code, &group);
        for (w, target) in core_new
            .iter()
            .map(|w| (w, &core_code))
            .chain(group.iter().map(|w| (w, &new_code)))
        {
            self.placed.insert(w.id.clone(), target.clone());
        }
        let mut core = Leaf {
            core: leaf.core,
            n_old: leaf.n_old,
            capacity: if leaf.n_old > 0 {
                leaf.n_old
            } else {
                self.median_leaf
            },
            new: core_new,
            unit: Vec::new(),
        };
        core.refresh();
        let mut fresh = Leaf {
            core: vec![0.0; c2.len()],
            n_old: 0,
            capacity: self.median_leaf,
            new: group,
            unit: Vec::new(),
        };
        fresh.refresh();
        self.leaves.insert(core_code, core);
        self.leaves.insert(new_code, fresh);
    }

    fn add_children(&mut self, code: &str, core_code: &str, new_code: &str, group: &[NewWork]) {
        let at = self
            .doc
            .classes
            .iter()
            .position(|c| c.code == code)
            .unwrap();
        let label = self.doc.classes[at].label.clone();
        let mut df: BTreeMap<String, f64> = BTreeMap::new();
        for w in group {
            let mut seen = BTreeSet::new();
            for (t, _) in words(&w.title) {
                if seen.insert(t.clone()) {
                    *df.entry(t).or_insert(0.0) += 1.0;
                }
            }
        }
        let mut terms: Vec<(String, f64)> = df.into_iter().collect();
        terms.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        terms.truncate(8);
        let top = terms.first().map_or(1.0, |t| t.1);
        let vocab = terms
            .iter()
            .map(|(t, c)| VocabLink {
                term: t.clone(),
                source: "insertion".into(),
                source_id: None,
                weight: c / top,
            })
            .collect();
        let core_label = if label.ends_with("(general)") {
            label.clone()
        } else {
            format!("{label} (general)")
        };
        let core = ClassDef {
            code: core_code.to_owned(),
            label: core_label,
            parent: Some(code.to_owned()),
            range: None,
            vocab: vec![],
            note: Some(format!(
                "Created by insertion: the works of {code} before the split and the new works near them."
            )),
        };
        let fresh = ClassDef {
            code: new_code.to_owned(),
            label: auto_label(&terms),
            parent: Some(code.to_owned()),
            range: None,
            vocab,
            note: Some(format!(
                "Created by insertion from {} new works that formed a distinct group in {code}; label from their most frequent title words.",
                group.len()
            )),
        };
        self.doc.classes.splice(at + 1..at + 1, [core, fresh]);
    }
}
