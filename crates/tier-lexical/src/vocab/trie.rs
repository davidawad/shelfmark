//! A trie over whole tokens of normalized (v1) text.
//!
//! Normalized text is lowercase alphanumeric runs separated by single
//! spaces, and a vocab hit must start and end on a token boundary, so a
//! phrase is a token sequence and matching is a walk from each token. The
//! trie is plain data (no automaton to rebuild), which keeps building it
//! cheap for schemes with a hundred thousand phrases.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Phrase occurrences as `(start byte, end byte, phrase index)`.
pub(crate) type Hit = (usize, usize, usize);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TokenTrie {
    /// Token text to token number.
    tokens: HashMap<String, u32>,
    /// `(node, token)` to child node; node 0 is the root.
    edges: HashMap<(u32, u32), u32>,
    /// The phrase ending at each node, if any.
    ends: Vec<Option<u32>>,
}

impl TokenTrie {
    /// A trie holding each phrase in `phrases` under its index. Phrases are
    /// non-empty normalized text; a repeated phrase keeps its first index.
    pub fn build<'a>(phrases: impl IntoIterator<Item = &'a str>) -> Self {
        let mut trie = TokenTrie {
            ends: vec![None],
            ..TokenTrie::default()
        };
        for (pid, phrase) in phrases.into_iter().enumerate() {
            let mut node = 0u32;
            for token in phrase.split(' ') {
                let next_token = u32::try_from(trie.tokens.len()).expect("under 2^32 tokens");
                let tok = *trie.tokens.entry(token.to_string()).or_insert(next_token);
                let next_node = u32::try_from(trie.ends.len()).expect("under 2^32 nodes");
                node = *trie.edges.entry((node, tok)).or_insert_with(|| next_node);
                if node == next_node {
                    trie.ends.push(None);
                }
            }
            let end = &mut trie.ends[node as usize];
            if end.is_none() {
                *end = Some(u32::try_from(pid).expect("under 2^32 phrases"));
            }
        }
        trie
    }

    /// Every phrase occurrence in one normalized field, overlapping ones
    /// included, ordered by start then end.
    pub fn hits(&self, field: &str) -> Vec<Hit> {
        // (start, end, token number or None when the token is unknown)
        let mut spans: Vec<(usize, usize, Option<u32>)> = Vec::new();
        let mut start = 0;
        for token in field.split(' ') {
            let end = start + token.len();
            spans.push((start, end, self.tokens.get(token).copied()));
            start = end + 1;
        }
        let mut hits = Vec::new();
        for i in 0..spans.len() {
            let mut node = 0u32;
            for &(_, end, tok) in &spans[i..] {
                let Some(next) = tok.and_then(|t| self.edges.get(&(node, t))) else {
                    break;
                };
                node = *next;
                if let Some(pid) = self.ends[node as usize] {
                    hits.push((spans[i].0, end, pid as usize));
                }
            }
        }
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_whole_token_phrases_including_overlaps() {
        let trie = TokenTrie::build(["organic chemistry", "chemistry", "chem"]);
        let hits = trie.hits("organic chemistry of chemists");
        assert_eq!(hits, [(0, 17, 0), (8, 17, 1)]);
    }

    #[test]
    fn a_repeated_phrase_keeps_its_first_index() {
        let trie = TokenTrie::build(["a b", "a b"]);
        assert_eq!(trie.hits("a b"), [(0, 3, 0)]);
    }

    #[test]
    fn unknown_tokens_end_a_walk() {
        let trie = TokenTrie::build(["a b c"]);
        assert!(trie.hits("a b x c").is_empty());
    }
}
