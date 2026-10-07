//! Gold set: one NDJSON line per labelled item, each with the LCC classes a
//! correct placement may land in and the provenance of that label (R15).

use crate::error::{ndjson_lines, read_file, EvalError, Result};
use crate::lcc::Class;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// One acceptable class; `path` is derived from `code` and kept in the file
/// so a reader can see the hierarchy without running the tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Acceptable {
    Code(String),
    Labelled {
        code: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        path: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from: Option<String>,
    },
}

impl Acceptable {
    pub fn code(&self) -> &str {
        match self {
            Acceptable::Code(c) => c,
            Acceptable::Labelled { code, .. } => code,
        }
    }
}

/// Where a label came from and the deterministic rule that produced it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Provenance {
    /// `arxiv`, `openlibrary`, `wikipedia`, `correction`, ...
    pub kind: String,
    /// Source record identifier (arXiv ID, OL edition key, Wikidata QID).
    pub record: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub retrieved_at: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub license: String,
    /// The labelling rule, specific enough to re-derive the label.
    pub label_rule: String,
    /// Raw snapshot path relative to the gold directory, and its digest.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub raw: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub raw_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoldItem {
    /// Stable gold-set key that predictions reference (`gold:arxiv:2401.03802`).
    pub id: String,
    /// L0 `item:` ID when the item is a known file; predictions may use it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
    #[serde(default = "default_scheme")]
    pub scheme: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub identifiers: BTreeMap<String, String>,
    pub acceptable: Vec<Acceptable>,
    /// Optional FAST headings (IDs or labels) the item should also receive.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fast: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Provenance>,
}

fn default_scheme() -> String {
    "lcc".into()
}

#[derive(Debug, Clone)]
pub struct GoldSet {
    pub items: Vec<GoldItem>,
    /// Parsed `acceptable` classes, parallel to `items`.
    pub classes: Vec<Vec<Class>>,
    index: HashMap<String, usize>,
}

impl GoldSet {
    pub fn from_items(items: Vec<GoldItem>) -> Result<GoldSet> {
        let mut classes = Vec::with_capacity(items.len());
        let mut index = HashMap::new();
        for (n, it) in items.iter().enumerate() {
            let at = format!("gold item `{}`", it.id);
            let next = "mc-eval validate --gold <FILE>";
            if it.acceptable.is_empty() {
                return Err(EvalError::new(
                    "E_GOLD_EMPTY",
                    format!("{at} has no acceptable classes"),
                    next,
                ));
            }
            let mut cs = Vec::new();
            for a in &it.acceptable {
                let c = Class::parse(a.code())
                    .map_err(|e| EvalError::new("E_GOLD_CODE", format!("{at}: {e}"), next))?;
                if let Acceptable::Labelled { path, .. } = a {
                    if !path.is_empty() && *path != c.path() {
                        return Err(EvalError::new(
                            "E_GOLD_PATH",
                            format!(
                                "{at}: path {path:?} does not match code `{}` (expected {:?})",
                                a.code(),
                                c.path()
                            ),
                            "delete the `path` field or regenerate with `mc-eval gold build`",
                        ));
                    }
                }
                cs.push(c);
            }
            for key in std::iter::once(&it.id).chain(it.item.as_ref()) {
                if index.insert(key.clone(), n).is_some() {
                    return Err(EvalError::new(
                        "E_GOLD_DUP",
                        format!("duplicate gold key `{key}`"),
                        next,
                    ));
                }
            }
            classes.push(cs);
        }
        Ok(GoldSet {
            items,
            classes,
            index,
        })
    }

    pub fn parse(text: &str) -> Result<GoldSet> {
        let mut items = Vec::new();
        for (ln, line) in ndjson_lines(text) {
            let it: GoldItem = serde_json::from_str(line).map_err(|e| {
                EvalError::new(
                    "E_GOLD_PARSE",
                    format!("gold line {ln}: {e}"),
                    "fix the line to match schemas/gold-item.schema.json",
                )
            })?;
            items.push(it);
        }
        if items.is_empty() {
            return Err(EvalError::new(
                "E_GOLD_EMPTY",
                "gold set has no items",
                "mc-eval gold build",
            ));
        }
        GoldSet::from_items(items)
    }

    pub fn load(path: &str) -> Result<GoldSet> {
        GoldSet::parse(&read_file(
            path,
            "pass an existing gold NDJSON file with --gold",
        )?)
    }

    /// Resolve a prediction's `item` (gold `id` or L0 `item:` ID).
    pub fn lookup(&self, key: &str) -> Option<usize> {
        self.index.get(key).copied()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Deepest level any acceptable class of item `i` reaches.
    pub fn depth(&self, i: usize) -> usize {
        self.classes[i].iter().map(Class::depth).max().unwrap_or(0)
    }
}

/// Counts that describe a gold set's coverage.
#[derive(Debug, Serialize)]
pub struct GoldStats {
    pub items: usize,
    pub main_classes: BTreeMap<String, usize>,
    pub subclasses: usize,
    pub by_source: BTreeMap<String, usize>,
    pub depth: BTreeMap<usize, usize>,
    pub multi_label: usize,
    pub with_fast: usize,
}

pub fn stats(g: &GoldSet) -> GoldStats {
    let mut s = GoldStats {
        items: g.len(),
        main_classes: BTreeMap::new(),
        subclasses: 0,
        by_source: BTreeMap::new(),
        depth: BTreeMap::new(),
        multi_label: 0,
        with_fast: 0,
    };
    let mut subs = std::collections::BTreeSet::new();
    for (i, it) in g.items.iter().enumerate() {
        let mut mains = std::collections::BTreeSet::new();
        for c in &g.classes[i] {
            mains.insert(c.letters()[..1].to_string());
            subs.insert(c.letters().to_string());
        }
        for m in mains {
            *s.main_classes.entry(m).or_default() += 1;
        }
        let kind = it
            .source
            .as_ref()
            .map(|p| p.kind.clone())
            .unwrap_or_else(|| "unknown".into());
        *s.by_source.entry(kind).or_default() += 1;
        *s.depth.entry(g.depth(i)).or_default() += 1;
        s.multi_label += usize::from(g.classes[i].len() > 1);
        s.with_fast += usize::from(!it.fast.is_empty());
    }
    s.subclasses = subs.len();
    s
}
