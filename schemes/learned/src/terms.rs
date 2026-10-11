//! Class-based TF-IDF (c-TF-IDF, as in BERTopic): each class's text is the
//! concatenated titles and abstracts of its works, and a term scores
//! `tf(t, c) / tokens(c) * ln(1 + A / f(t))`, with `A` the mean token count
//! of a leaf class and `f(t)` the term's corpus count. Unigrams and adjacent
//! bigrams (no dropped word between) are counted; titles count twice.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::stopwords::STOP;

fn is_stop(w: &str) -> bool {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| STOP.iter().copied().collect())
        .contains(w)
}

/// Words of `text` worth counting, each with whether it directly follows
/// the previous kept word (so the pair may form a bigram).
pub fn words(text: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    let mut joined = false;
    for raw in text.split(|c: char| !c.is_alphanumeric()) {
        if raw.is_empty() {
            continue;
        }
        let w = raw.to_lowercase();
        let keep = w.chars().count() >= 3
            && w.chars().any(char::is_alphabetic)
            && !w.chars().all(|c| c.is_ascii_digit())
            && !is_stop(&w);
        if keep {
            out.push((w, joined));
            joined = true;
        } else {
            joined = false;
        }
    }
    out
}

/// Term counts of one text (title counted twice).
pub fn doc_terms(title: &str, body: &str) -> HashMap<String, f64> {
    let mut out = HashMap::new();
    for (text, weight) in [(title, 2.0), (body, 1.0)] {
        let ws = words(text);
        for (k, (w, joined)) in ws.iter().enumerate() {
            *out.entry(w.clone()).or_insert(0.0) += weight;
            if *joined && k > 0 && ws[k - 1].0 != *w {
                *out.entry(format!("{} {}", ws[k - 1].0, w)).or_insert(0.0) += weight;
            }
        }
    }
    out
}

/// Interned term counts per class.
#[derive(Default, Clone)]
pub struct Bag {
    pub count: HashMap<u32, f64>,
    pub docs: HashMap<u32, u32>,
    pub tokens: f64,
    pub n: usize,
}

impl Bag {
    pub fn add_doc(&mut self, terms: &[(u32, f64)]) {
        self.n += 1;
        for &(t, c) in terms {
            *self.count.entry(t).or_insert(0.0) += c;
            *self.docs.entry(t).or_insert(0) += 1;
            self.tokens += c;
        }
    }
    pub fn merge(&mut self, other: &Bag) {
        self.n += other.n;
        self.tokens += other.tokens;
        for (&t, &c) in &other.count {
            *self.count.entry(t).or_insert(0.0) += c;
        }
        for (&t, &d) in &other.docs {
            *self.docs.entry(t).or_insert(0) += d;
        }
    }
}

/// Corpus term statistics shared by all classes.
pub struct Lexicon {
    pub names: Vec<String>,
    ids: HashMap<String, u32>,
    pub corpus: HashMap<u32, f64>,
}

impl Lexicon {
    pub fn new() -> Self {
        Lexicon {
            names: Vec::new(),
            ids: HashMap::new(),
            corpus: HashMap::new(),
        }
    }
    pub fn intern(&mut self, terms: HashMap<String, f64>) -> Vec<(u32, f64)> {
        let mut out: Vec<(u32, f64)> = terms
            .into_iter()
            .map(|(t, c)| {
                let next = self.names.len() as u32;
                let id = *self.ids.entry(t.clone()).or_insert_with(|| {
                    self.names.push(t);
                    next
                });
                *self.corpus.entry(id).or_insert(0.0) += c;
                (id, c)
            })
            .collect();
        out.sort_by_key(|x| x.0);
        out
    }
    /// Top `k` c-TF-IDF terms of `bag` with scores, given `A`. A term must
    /// occur in at least `max(2, 5%)` of the class's works.
    pub fn top(&self, bag: &Bag, mean_tokens: f64, k: usize) -> Vec<(String, f64)> {
        let min_docs = 2.max((bag.n as f64 * 0.05).ceil() as u32).min(bag.n as u32);
        let mut scored: Vec<(f64, &str)> = bag
            .count
            .iter()
            .filter(|(t, _)| bag.docs[t] >= min_docs)
            .map(|(t, &c)| {
                let idf = (1.0 + mean_tokens / self.corpus[t]).ln();
                (
                    c / bag.tokens.max(1.0) * idf,
                    self.names[*t as usize].as_str(),
                )
            })
            .collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(b.1)));
        scored
            .into_iter()
            .take(k)
            .map(|(s, t)| (t.to_owned(), s))
            .collect()
    }
}

impl Default for Lexicon {
    fn default() -> Self {
        Self::new()
    }
}

/// Fallback label from top terms: "Graph neural networks / node classification".
pub fn auto_label(terms: &[(String, f64)]) -> String {
    let picked: Vec<&str> = terms.iter().take(3).map(|t| t.0.as_str()).collect();
    let mut label = picked.join(" / ");
    if let Some(first) = label.get(..1) {
        label = first.to_uppercase() + &label[1..];
    }
    if label.is_empty() {
        "Unlabelled".into()
    } else {
        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopword_list_has_no_duplicates() {
        let set: HashSet<_> = STOP.iter().collect();
        assert_eq!(set.len(), STOP.len());
    }

    #[test]
    fn bigrams_only_across_adjacent_kept_words() {
        let t = doc_terms("Graph neural networks", "graph of networks");
        assert_eq!(t["graph neural"], 2.0);
        assert_eq!(t["neural networks"], 2.0);
        assert_eq!(t["graph"], 3.0);
        assert!(!t.contains_key("graph networks"));
    }

    #[test]
    fn distinctive_terms_rank_first() {
        let mut lex = Lexicon::new();
        let mut a = Bag::default();
        let mut b = Bag::default();
        for _ in 0..3 {
            a.add_doc(&lex.intern(doc_terms("quantum spin chains", "quantum spin")));
            b.add_doc(&lex.intern(doc_terms("protein folding", "spin")));
        }
        let mean = (a.tokens + b.tokens) / 2.0;
        let top = lex.top(&a, mean, 2);
        assert_eq!(top[0].0, "quantum");
        assert_eq!(auto_label(&top), "Quantum / quantum spin");
    }
}
