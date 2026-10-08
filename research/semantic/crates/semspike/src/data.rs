//! Scheme outline (two levels) and labelled sample loading, plus the label texts each
//! tier embeds or reranks against.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct Node {
    pub code: String,
    pub label: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub subclasses: Vec<Node>,
}

#[derive(Deserialize, Debug)]
pub struct Outline {
    pub scheme: String,
    pub version: String,
    pub classes: Vec<Node>,
}

/// A flattened subclass with a back-reference to its top-level class.
#[derive(Debug, Clone)]
pub struct Leaf {
    pub code: String,
    pub class_idx: usize,
    pub text: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Sample {
    pub id: String,
    pub text: String,
    pub gold_class: String,
    pub gold_subclass: String,
}

impl Outline {
    pub fn load(path: &Path) -> Result<Self> {
        let o: Outline = serde_json::from_slice(&std::fs::read(path)?)
            .with_context(|| format!("parse {}", path.display()))?;
        anyhow::ensure!(!o.classes.is_empty(), "outline has no classes");
        Ok(o)
    }

    /// Text describing a top-level class: caption, scope and its subclass captions.
    pub fn class_text(&self, i: usize) -> String {
        let c = &self.classes[i];
        let subs: Vec<&str> = c.subclasses.iter().map(|s| s.label.as_str()).collect();
        let mut t = c.label.clone();
        if let Some(s) = &c.scope {
            t.push_str(". ");
            t.push_str(s);
        }
        if !subs.is_empty() {
            t.push_str(". Includes: ");
            t.push_str(&subs.join("; "));
        }
        t
    }

    pub fn leaves(&self) -> Vec<Leaf> {
        let mut out = Vec::new();
        for (ci, c) in self.classes.iter().enumerate() {
            for s in &c.subclasses {
                let mut text = format!("{} — {}", c.label, s.label);
                if let Some(sc) = &s.scope {
                    text.push_str(": ");
                    text.push_str(sc);
                }
                out.push(Leaf {
                    code: s.code.clone(),
                    class_idx: ci,
                    text,
                });
            }
        }
        out
    }
}

pub fn load_samples(path: &Path) -> Result<Vec<Sample>> {
    let raw = std::fs::read_to_string(path)?;
    raw.lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, l)| serde_json::from_str(l).with_context(|| format!("samples line {}", i + 1)))
        .collect()
}

/// Truncate to at most `max_chars` characters on a char boundary (inputs are long summaries).
pub fn clip(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn data(f: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data")
            .join(f)
    }

    #[test]
    fn outline_is_two_level_lcc_with_unique_codes() {
        let o = Outline::load(&data("lcc-outline.json")).unwrap();
        assert_eq!(o.classes.len(), 21);
        let mut seen = HashSet::new();
        for c in &o.classes {
            assert!(!c.subclasses.is_empty(), "{} has no subclasses", c.code);
            for s in &c.subclasses {
                assert!(
                    s.code.starts_with(&c.code),
                    "{} not under {}",
                    s.code,
                    c.code
                );
                assert!(seen.insert(s.code.clone()), "duplicate {}", s.code);
            }
        }
        assert_eq!(o.leaves().len(), seen.len());
    }

    #[test]
    fn samples_cover_every_class_with_valid_gold_labels() {
        let o = Outline::load(&data("lcc-outline.json")).unwrap();
        let s = load_samples(&data("samples.jsonl")).unwrap();
        let mut classes = HashSet::new();
        let mut ids = HashSet::new();
        for x in &s {
            assert!(ids.insert(x.id.clone()), "duplicate id {}", x.id);
            let c = o.classes.iter().find(|c| c.code == x.gold_class).unwrap();
            assert!(
                c.subclasses.iter().any(|sc| sc.code == x.gold_subclass),
                "{}",
                x.id
            );
            classes.insert(x.gold_class.clone());
        }
        assert_eq!(classes.len(), 21);
    }

    #[test]
    fn clip_respects_char_boundaries() {
        assert_eq!(clip("héllo", 2), "hé");
        assert_eq!(clip("hi", 10), "hi");
    }
}
