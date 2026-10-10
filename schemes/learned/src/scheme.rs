//! From class tree to scheme file: codes in shelf order, c-TF-IDF terms and
//! central titles per class, the hand-written labels file, and the
//! `SchemeDoc` itself.

use crate::classtree::ClassTree;
use crate::graph::Graph;
use crate::terms::{auto_label, doc_terms, Bag, Lexicon};
use shelfmark_bench::CorpusRow;
use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, VocabLink};
use std::collections::BTreeMap;

pub const CODE_GRAMMAR: &str = "[0-9]{2}(\\.[0-9]{2})*";
const TERMS: usize = 10;
const TITLES: usize = 5;

/// One class with what a labeller needs to see.
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub node: usize,
    pub code: String,
    pub parent: Option<String>,
    pub depth: usize,
    pub n: usize,
    pub leaf: bool,
    pub terms: Vec<(String, f64)>,
    pub titles: Vec<String>,
}

impl ClassInfo {
    /// The top three terms; a label written for other terms is stale.
    pub fn fingerprint(&self) -> String {
        self.terms
            .iter()
            .take(3)
            .map(|t| t.0.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Codes by node: a class's code is its parent's code plus its two-digit
/// position among its siblings, so codes sort in shelf order.
pub fn codes(tree: &ClassTree) -> Vec<String> {
    let mut out = vec![String::new(); tree.nodes.len()];
    for v in tree.preorder() {
        for (k, &c) in tree.nodes[v].children.iter().enumerate() {
            out[c] = if v == 0 {
                format!("{:02}", k + 1)
            } else {
                format!("{}.{:02}", out[v], k + 1)
            };
        }
    }
    out
}

/// Classes in preorder with their c-TF-IDF terms and most central titles
/// (largest edge weight to the class's other works).
pub fn describe(tree: &ClassTree, g: &Graph, rows: &[CorpusRow]) -> Vec<ClassInfo> {
    let code = codes(tree);
    let leaf_of = tree.item_leaf();
    let mut lex = Lexicon::new();
    let docs: Vec<_> = rows
        .iter()
        .map(|r| lex.intern(doc_terms(&r.title, &r.description)))
        .collect();
    let order = tree.preorder();
    let mut bags = vec![Bag::default(); tree.nodes.len()];
    for (item, terms) in docs.iter().enumerate() {
        bags[leaf_of[item]].add_doc(terms);
    }
    for &v in order.iter().rev() {
        for &c in &tree.nodes[v].children.clone() {
            let child = std::mem::take(&mut bags[c]);
            bags[v].merge(&child);
            bags[c] = child;
        }
    }
    let leaves = tree.leaves();
    let mean = leaves.iter().map(|&l| bags[l].tokens).sum::<f64>() / leaves.len().max(1) as f64;
    let mut central = vec![0.0; tree.n()];
    for &(a, b, w) in &g.edges {
        if leaf_of[a] == leaf_of[b] {
            central[a] += w;
            central[b] += w;
        }
    }
    let depth = tree.depths();
    order
        .into_iter()
        .filter(|&v| v != 0)
        .map(|v| {
            let node = &tree.nodes[v];
            let mut items: Vec<usize> = tree.order[node.lo..node.hi].to_vec();
            items.sort_by(|&a, &b| central[b].total_cmp(&central[a]).then(a.cmp(&b)));
            ClassInfo {
                node: v,
                code: code[v].clone(),
                parent: (node.parent != 0).then(|| code[node.parent].clone()),
                depth: depth[v],
                n: node.len(),
                leaf: node.is_leaf(),
                terms: lex.top(&bags[v], mean, TERMS),
                titles: items
                    .iter()
                    .take(TITLES)
                    .map(|&i| rows[i].title.clone())
                    .collect(),
            }
        })
        .collect()
}

/// `code<TAB>label<TAB>fingerprint` lines; `#` starts a comment.
pub fn read_labels(text: &str) -> Result<BTreeMap<String, (String, String)>, String> {
    let mut out = BTreeMap::new();
    for (k, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 3 || f[1].trim().is_empty() {
            return Err(format!(
                "labels line {}: want code, label, fingerprint",
                k + 1
            ));
        }
        out.insert(f[0].to_owned(), (f[1].trim().to_owned(), f[2].to_owned()));
    }
    Ok(out)
}

/// Markdown sheet for writing labels: one block per class.
pub fn label_sheet(infos: &[ClassInfo]) -> String {
    let mut s = String::from(
        "# Label sheet\n\nOne entry per class: code, size, c-TF-IDF terms, most central titles.\n",
    );
    for c in infos {
        let terms: Vec<String> = c.terms.iter().map(|t| t.0.clone()).collect();
        s += &format!(
            "\n## {} ({} works, depth {}{})\n\nfingerprint: {}\nterms: {}\n",
            c.code,
            c.n,
            c.depth,
            if c.leaf { ", leaf" } else { "" },
            c.fingerprint(),
            terms.join(", ")
        );
        for t in &c.titles {
            s += &format!("- {}\n", t.replace('\n', " "));
        }
    }
    s
}

/// The scheme's `source.description`: inputs, objective and labelling.
pub fn source_description(
    n: usize,
    corpus_sha: &str,
    graph_sha: &str,
    subset_of: Option<usize>,
    cfg: &crate::FitConfig,
) -> String {
    let p = &cfg.params;
    let subset = subset_of
        .map(|all| format!(", a subset of its {all} works"))
        .unwrap_or_default();
    format!(
        "Fitted by schemes/learned (learned-scheme build) to {n} works of the benchmark corpus (corpus.jsonl sha256 {corpus_sha}{subset}) \
         on the k=20 kNN cosine graph (sha256 {graph_sha}). Objective: normalized Dasgupta cost + {} x expected navigation cost \
         (per-branch cost {} bits + log2 fan-out); 2-{} child classes, at most {} levels, leaf classes of at least {} works; seed {}. \
         Labels: written by hand from each class's c-TF-IDF terms and most central titles; the terms are the class vocabulary (source ctfidf).",
        p.lambda, p.branch_cost, p.max_children, p.max_depth, p.min_leaf, cfg.seed
    )
}

pub struct Meta {
    pub version: String,
    pub description: String,
}

/// Builds the scheme. With `labels`, every class needs a label whose
/// fingerprint matches its current terms, unless `auto` allows falling back
/// to the top terms.
pub fn scheme_doc(
    infos: &[ClassInfo],
    labels: Option<&BTreeMap<String, (String, String)>>,
    auto: bool,
    meta: &Meta,
) -> Result<SchemeDoc, String> {
    let mut stale = Vec::new();
    let classes = infos
        .iter()
        .map(|c| {
            let fp = c.fingerprint();
            let label = match labels.and_then(|l| l.get(&c.code)) {
                Some((label, f)) if *f == fp => label.clone(),
                other => {
                    if labels.is_some() && !auto {
                        stale.push(format!(
                            "{} ({})",
                            c.code,
                            if other.is_some() { "stale" } else { "missing" }
                        ));
                    }
                    auto_label(&c.terms)
                }
            };
            let top = c.terms.first().map_or(1.0, |t| t.1);
            let terms: Vec<&str> = c.terms.iter().map(|t| t.0.as_str()).collect();
            ClassDef {
                code: c.code.clone(),
                label,
                parent: c.parent.clone(),
                range: None,
                vocab: c
                    .terms
                    .iter()
                    .filter(|t| t.1 > 0.0)
                    .map(|(t, s)| VocabLink {
                        term: t.clone(),
                        source: "ctfidf".into(),
                        source_id: None,
                        weight: (s / top).clamp(f64::MIN_POSITIVE, 1.0),
                    })
                    .collect(),
                note: Some(format!(
                    "{} works in the fitting corpus. c-TF-IDF terms: {}.",
                    c.n,
                    terms.join(", ")
                )),
            }
        })
        .collect();
    if !stale.is_empty() {
        let shown = stale
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "{} classes lack a current label: {shown}",
            stale.len()
        ));
    }
    Ok(SchemeDoc {
        schema: None,
        name: "learned".into(),
        version: meta.version.clone(),
        title: "Learned general classification (similarity- and navigation-optimized prototype)"
            .into(),
        source: Some(Source {
            description: meta.description.clone(),
            url: None,
            license: Some("CC0 1.0 (https://creativecommons.org/publicdomain/zero/1.0/)".into()),
        }),
        code_grammar: CODE_GRAMMAR.into(),
        classes,
    })
}

pub fn scheme_json(doc: &SchemeDoc) -> Result<String, String> {
    serde_json::to_string_pretty(doc)
        .map(|s| s + "\n")
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classtree::tests::two_leaves;

    #[test]
    fn codes_follow_shelf_order() {
        let t = two_leaves();
        assert_eq!(codes(&t), vec!["", "01", "02"]);
    }

    #[test]
    fn labels_file_round_trip_and_staleness() {
        let labels = read_labels("# c\n01\tStars\tstar\n").unwrap();
        let info = ClassInfo {
            node: 1,
            code: "01".into(),
            parent: None,
            depth: 1,
            n: 3,
            leaf: true,
            terms: vec![("star".into(), 2.0), ("galaxy".into(), 1.0)],
            titles: vec![],
        };
        let meta = Meta {
            version: "1".into(),
            description: "test".into(),
        };
        assert!(scheme_doc(std::slice::from_ref(&info), Some(&labels), false, &meta).is_err());
        let ok = read_labels("01\tStars\tstar, galaxy\n").unwrap();
        let doc = scheme_doc(&[info], Some(&ok), false, &meta).unwrap();
        assert_eq!(doc.classes[0].label, "Stars");
        assert_eq!(doc.classes[0].vocab[1].weight, 0.5);
    }
}
