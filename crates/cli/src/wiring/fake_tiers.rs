//! Stand-in tiers until U3 (identifier) and U4 (lexical) land. Enough
//! behaviour to exercise the cascade end to end, no claim to accuracy.

use std::collections::BTreeMap;

use shelfmark_core::id::normalize_text_v1;
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::{
    Candidate, ClassifyInput, Evidence, Identifier, Tier, TierInfo, TierOutput,
};
use shelfmark_core::{ClassId, Error};

fn info(name: &str) -> TierInfo {
    TierInfo {
        name: name.into(),
        version: "fake-1".into(),
        needs_model: false,
        expected_memory_mb: 1,
        expected_micros_per_item: 100,
    }
}

/// Exact identifier lookup in an in-memory table keyed like PD5
/// (`isbn:9780262033848` -> class code).
pub struct IdentifierTier {
    pub table: BTreeMap<String, String>,
}

impl Tier for IdentifierTier {
    fn info(&self) -> TierInfo {
        info("identifier")
    }

    fn run(
        &self,
        input: &ClassifyInput,
        index: &dyn CompiledIndex,
        _: &[Candidate],
    ) -> Result<TierOutput, Error> {
        let candidates = input
            .identifiers
            .iter()
            .filter_map(|id| {
                let code = self.table.get(&id.key())?;
                Some(Candidate {
                    class: ClassId::new(index.scheme().clone(), code),
                    score: 1.0,
                    evidence: vec![Evidence::Identifier {
                        identifier: id.clone(),
                        table: "fake".into(),
                    }],
                })
            })
            .collect();
        Ok(TierOutput {
            candidates,
            ..Default::default()
        })
    }
}

/// Counts whole-phrase matches of each class's vocabulary (weight as
/// given) and label (weight 0.5) in the title, text and headings; the
/// score is the matched weight, capped at 1.
pub struct VocabTier;

impl Tier for VocabTier {
    fn info(&self) -> TierInfo {
        info("vocab")
    }

    fn run(
        &self,
        input: &ClassifyInput,
        index: &dyn CompiledIndex,
        _: &[Candidate],
    ) -> Result<TierOutput, Error> {
        let mut parts: Vec<&str> = vec![input.title.as_deref().unwrap_or(""), &input.text];
        parts.extend(input.headings.iter().map(String::as_str));
        let hay = format!(" {} ", normalize_text_v1(&parts.join(" ")));
        let mut candidates = Vec::new();
        for (code, links) in index.vocab() {
            let label = index.get(code).map(|r| r.label).unwrap_or_default();
            let terms = links
                .iter()
                .map(|l| (l.term.as_str(), l.source.as_str(), l.weight))
                .chain(std::iter::once((label.as_str(), "label", 0.5)));
            let mut score = 0.0;
            let mut evidence = Vec::new();
            for (term, source, weight) in terms {
                let needle = normalize_text_v1(term);
                if needle.is_empty() {
                    continue;
                }
                let count = hay.matches(&format!(" {needle} ")).count() as u32;
                if count > 0 {
                    score += weight;
                    evidence.push(Evidence::Vocab {
                        term: term.into(),
                        source: source.into(),
                        count,
                    });
                }
            }
            if score > 0.0 {
                candidates.push(Candidate {
                    class: ClassId::new(index.scheme().clone(), code),
                    score: f64::min(score, 1.0),
                    evidence,
                });
            }
        }
        candidates.sort_by(|a, b| b.score.total_cmp(&a.score));
        Ok(TierOutput {
            candidates,
            ..Default::default()
        })
    }
}

/// Configured rules: none configured yet, so never a candidate.
pub struct RulesTier;

impl Tier for RulesTier {
    fn info(&self) -> TierInfo {
        info("rules")
    }

    fn run(
        &self,
        _: &ClassifyInput,
        _: &dyn CompiledIndex,
        _: &[Candidate],
    ) -> Result<TierOutput, Error> {
        Ok(TierOutput::default())
    }
}

/// `isbn:978...`, `lccn:`, `doi:`, `arxiv:`, `domain:` to an identifier,
/// lightly normalized (U3 does the real checksums).
pub fn parse_identifier(s: &str) -> Option<Identifier> {
    let (kind, value) = s.split_once(':')?;
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some(match kind.to_ascii_lowercase().as_str() {
        "isbn" => Identifier::Isbn(value.chars().filter(char::is_ascii_alphanumeric).collect()),
        "lccn" => Identifier::Lccn(value.into()),
        "doi" => Identifier::Doi(value.trim_start_matches("https://doi.org/").to_lowercase()),
        "arxiv" => Identifier::Arxiv(value.into()),
        "domain" => Identifier::UrlDomain(value.trim_start_matches("www.").to_lowercase()),
        _ => return None,
    })
}

/// The registrable-ish domain of a URL: host, lowercased, without `www.`.
pub fn url_domain(url: &str) -> Option<Identifier> {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let host = rest.split(['/', '?', '#']).next()?.split(':').next()?;
    (!host.is_empty())
        .then(|| Identifier::UrlDomain(host.trim_start_matches("www.").to_lowercase()))
}
