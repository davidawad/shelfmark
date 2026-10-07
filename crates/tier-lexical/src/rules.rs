//! The `rules` tier: configured `{when: {domain|term|regex}, class,
//! weight}` rules from a JSON file.
//!
//! ```json
//! {"rules": [
//!   {"id": "arxiv-cs", "when": {"domain": "arxiv.org"}, "class": "QA76", "weight": 0.6},
//!   {"when": {"term": "statute"}, "class": "K", "weight": 0.9},
//!   {"when": {"regex": "(?i)\\bRFC ?\\d{3,4}\\b"}, "class": "TK5105", "weight": 0.8}
//! ]}
//! ```
//!
//! - `domain` matches the input URL's host and any `url_domain`
//!   identifier, at the domain itself or any subdomain.
//! - `term` matches as whole words in the normalized (v1) title, text or
//!   headings.
//! - `regex` (Rust `regex` syntax) runs on the raw title, text, URL and
//!   headings.
//!
//! Precedence is by weight: a class's score is the highest weight among
//! its rules that fired, and its evidence lists every fired rule in file
//! order. Classes then rank by score, ties by class code bytes.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Instant;

use regex::Regex;
use serde::{Deserialize, Serialize};
use shelfmark_core::error::{Error, ErrorCode};
use shelfmark_core::id::{normalize_text_v1, sha256_hex, ClassId};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};
use shelfmark_core::tier::{
    Candidate, ClassifyInput, Cost, Evidence, Identifier, Tier, TierInfo, TierOutput,
};

use crate::{micros_since, normalized_fields, rank};

/// A rules file.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulesConfig {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// Name recorded in evidence; defaults to `rule-<n>` (1-based).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub when: RuleWhen,
    /// Class code in the scheme being classified into.
    pub class: String,
    /// Score given to the class when the rule fires, in (0, 1].
    pub weight: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum RuleWhen {
    Domain(String),
    Term(String),
    Regex(String),
}

enum Test {
    Domain(String),
    Term(String),
    Regex(Regex),
}

struct Compiled {
    label: String,
    test: Test,
    class: String,
    weight: f64,
}

/// The `rules` tier.
pub struct RulesTier {
    version: String,
    rules: Vec<Compiled>,
}

impl std::fmt::Debug for RulesTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RulesTier")
            .field("version", &self.version)
            .field("rules", &self.rules.len())
            .finish()
    }
}

fn invalid(message: String) -> Error {
    Error::new(ErrorCode::InvalidRequest, message)
}

/// Lowercase, no `www.`, no trailing dot.
fn clean_domain(d: &str) -> String {
    let d = d.trim().trim_end_matches('.').to_lowercase();
    d.strip_prefix("www.").map(str::to_string).unwrap_or(d)
}

/// Host of a URL, cleaned like a rule domain.
fn url_host(url: &str) -> Option<String> {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = host.split(':').next()?;
    (!host.is_empty()).then(|| clean_domain(host))
}

impl RulesTier {
    pub const NAME: &'static str = "rules";

    /// Validate and compile `config`. Fails with `invalid_request` on a
    /// bad weight, an empty class, term or domain, a duplicate id, or a
    /// regex that does not compile.
    pub fn from_config(config: &RulesConfig) -> Result<Self, Error> {
        let mut ids = BTreeSet::new();
        let mut rules = Vec::with_capacity(config.rules.len());
        for (i, rule) in config.rules.iter().enumerate() {
            let id = rule.id.clone().unwrap_or_else(|| format!("rule-{}", i + 1));
            if !ids.insert(id.clone()) {
                return Err(invalid(format!("rule id {id:?} is used twice")));
            }
            if !(rule.weight.is_finite() && rule.weight > 0.0 && rule.weight <= 1.0) {
                return Err(invalid(format!(
                    "rule {id} has weight {}; weights must be in (0, 1]",
                    rule.weight
                )));
            }
            if rule.class.trim().is_empty() {
                return Err(invalid(format!("rule {id} names no class")));
            }
            let (test, shown) = match &rule.when {
                RuleWhen::Domain(d) => (Test::Domain(clean_domain(d)), format!("domain:{d}")),
                RuleWhen::Term(t) => (Test::Term(normalize_text_v1(t)), format!("term:{t}")),
                RuleWhen::Regex(r) => {
                    let re = Regex::new(r)
                        .map_err(|e| invalid(format!("rule {id} regex does not compile: {e}")))?;
                    (Test::Regex(re), format!("regex:{r}"))
                }
            };
            if matches!(&test, Test::Domain(s) | Test::Term(s) if s.is_empty()) {
                return Err(invalid(format!("rule {id} has an empty {shown:?}")));
            }
            rules.push(Compiled {
                label: format!("{id}: {shown}"),
                test,
                class: rule.class.trim().to_string(),
                weight: rule.weight,
            });
        }
        let canonical = serde_json::to_string(config)
            .map_err(|e| Error::internal(format!("rules config serializes: {e}")))?;
        Ok(RulesTier {
            version: format!("rules-1+{}", &sha256_hex(canonical.as_bytes())[..12]),
            rules,
        })
    }

    pub fn from_json(json: &str) -> Result<Self, Error> {
        let config: RulesConfig = serde_json::from_str(json)
            .map_err(|e| invalid(format!("rules config is not valid: {e}")))?;
        Self::from_config(&config)
    }

    pub fn from_path(path: &Path) -> Result<Self, Error> {
        let json = std::fs::read_to_string(path).map_err(|e| {
            Error::new(
                ErrorCode::InputUnreadable,
                format!("cannot read rules file {}: {e}", path.display()),
            )
            .details(serde_json::json!({ "path": path.display().to_string() }))
        })?;
        Self::from_json(&json).map_err(|e| {
            let message = format!("{}: {}", path.display(), e.message);
            Error { message, ..e }
        })
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Rule-fired candidates for `input`, without checking class codes.
    fn fire(&self, input: &ClassifyInput) -> BTreeMap<&str, (f64, Vec<Evidence>)> {
        let hosts: BTreeSet<String> = input
            .url
            .as_deref()
            .and_then(url_host)
            .into_iter()
            .chain(input.identifiers.iter().filter_map(|id| match id {
                Identifier::UrlDomain(d) => Some(clean_domain(d)),
                _ => None,
            }))
            .collect();
        let padded: Vec<String> = normalized_fields(input)
            .iter()
            .map(|f| format!(" {f} "))
            .collect();
        let raw: Vec<&str> = input
            .title
            .iter()
            .chain(std::iter::once(&input.text))
            .chain(input.url.iter())
            .chain(input.headings.iter())
            .map(String::as_str)
            .collect();

        let mut fired: BTreeMap<&str, (f64, Vec<Evidence>)> = BTreeMap::new();
        for rule in &self.rules {
            let hit = match &rule.test {
                Test::Domain(d) => hosts.iter().any(|h| {
                    h == d || h.strip_suffix(d.as_str()).is_some_and(|p| p.ends_with('.'))
                }),
                Test::Term(t) => {
                    let needle = format!(" {t} ");
                    padded.iter().any(|f| f.contains(&needle))
                }
                Test::Regex(re) => raw.iter().any(|s| re.is_match(s)),
            };
            if hit {
                let entry = fired.entry(&rule.class).or_insert((0.0, Vec::new()));
                entry.0 = entry.0.max(rule.weight);
                entry.1.push(Evidence::Rule {
                    rule: rule.label.clone(),
                });
            }
        }
        fired
    }
}

impl Tier for RulesTier {
    fn info(&self) -> TierInfo {
        TierInfo {
            name: Self::NAME.into(),
            version: self.version.clone(),
            needs_model: false,
            expected_memory_mb: 1,
            expected_micros_per_item: 50,
        }
    }

    fn run(
        &self,
        input: &ClassifyInput,
        index: &dyn CompiledIndex,
        _prior: &[Candidate],
    ) -> Result<TierOutput, Error> {
        let start = Instant::now();
        let mut candidates = Vec::new();
        for (code, (score, evidence)) in self.fire(input) {
            match index.check_code(code) {
                CodeCheck::Exact { .. } | CodeCheck::Within { .. } => {}
                check => {
                    return Err(Error::new(
                        ErrorCode::InvalidCode,
                        format!(
                            "a rule names class {code}, which is not in {}",
                            index.scheme()
                        ),
                    )
                    .next(format!("shelfmark scheme search {} {code}", index.scheme()))
                    .details(serde_json::json!({ "class": code, "check": check })))
                }
            }
            candidates.push(Candidate {
                class: ClassId::new(index.scheme().clone(), code),
                score,
                evidence,
            });
        }
        candidates.sort_by(rank);
        Ok(TierOutput {
            candidates,
            cost: Cost {
                micros: micros_since(start),
                model_calls: 0,
            },
        })
    }
}
