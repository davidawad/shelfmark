//! English stopwords, and the minimum-information rule for vocab phrases.
//!
//! A phrase made only of stopwords (a class labelled `To`, an index
//! catchword `TO`) carries no subject: matching it hits nearly every
//! input, so one such class would otherwise collect a large share of all
//! placements. Such phrases never enter the matcher. A phrase with at
//! least one other token matches in full, stopwords included, as before.

/// Common English function words, in byte order (normalized v1 form).
#[rustfmt::skip]
const STOPWORDS: [&str; 125] = [
    "a", "about", "above", "after", "again", "against", "all", "am", "an", "and", "any", "are",
    "as", "at", "be", "because", "been", "before", "being", "below", "between", "both", "but",
    "by", "can", "could", "did", "do", "does", "doing", "down", "during", "each", "few", "for",
    "from", "further", "had", "has", "have", "having", "he", "her", "here", "hers", "herself",
    "him", "himself", "his", "how", "i", "if", "in", "into", "is", "it", "its", "itself", "just",
    "me", "more", "most", "my", "myself", "no", "nor", "not", "now", "of", "off", "on", "once",
    "only", "or", "other", "our", "ours", "ourselves", "out", "over", "own", "same", "she",
    "should", "so", "some", "such", "than", "that", "the", "their", "theirs", "them",
    "themselves", "then", "there", "these", "they", "this", "those", "through", "to", "too",
    "under", "until", "up", "very", "was", "we", "were", "what", "when", "where", "which",
    "while", "who", "whom", "why", "will", "with", "would", "you", "your", "yours", "yourself",
];

/// Whether `token` (normalized v1) is an English stopword.
pub fn is_stopword(token: &str) -> bool {
    STOPWORDS.binary_search(&token).is_ok()
}

/// Whether a normalized (v1) phrase carries too little information to
/// match on: it is empty or every token is a stopword.
pub fn is_uninformative(phrase: &str) -> bool {
    phrase.split(' ').all(|t| t.is_empty() || is_stopword(t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_is_sorted_and_unique() {
        assert!(STOPWORDS.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn stopword_only_phrases_are_uninformative() {
        for p in ["", "to", "the", "of the", "to and from"] {
            assert!(is_uninformative(p), "{p:?}");
        }
        for p in ["law", "to law", "history of the arts", "c", "2"] {
            assert!(!is_uninformative(p), "{p:?}");
        }
    }
}
