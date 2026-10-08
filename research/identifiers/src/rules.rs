//! The arXiv category map and the URL-domain rules: loading, validation
//! and resolution.

use crate::normalize;
use std::collections::{BTreeMap, BTreeSet};

/// Data rows of a `#`-commented TSV with a header, as (line number, columns).
fn rows<'a>(text: &'a str, header: &str) -> Result<Vec<(usize, Vec<&'a str>)>, String> {
    let mut lines = text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.starts_with('#') && !l.trim().is_empty());
    match lines.next() {
        Some((_, h)) if h == header => {}
        other => {
            return Err(format!(
                "expected header {header:?}, got {:?}",
                other.map(|o| o.1)
            ))
        }
    }
    let width = header.split('\t').count();
    lines
        .map(|(i, l)| {
            let cols: Vec<&str> = l.split('\t').collect();
            if cols.len() != width {
                return Err(format!(
                    "line {}: {} columns, expected {width}",
                    i + 1,
                    cols.len()
                ));
            }
            Ok((i + 1, cols))
        })
        .collect()
}

fn check_lcc(line: usize, lcc: &str) -> Result<(), String> {
    match normalize::lcc(lcc) {
        Some(c) if c.full == lcc => Ok(()),
        _ => Err(format!(
            "line {line}: {lcc:?} is not a normalized LCC class"
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArxivEntry {
    pub lcc: String,
    pub exact: bool,
}

/// arXiv category (or archive) -> LCC, with aliases already resolved.
pub struct ArxivMap(pub BTreeMap<String, ArxivEntry>);

pub const ARXIV_HEADER: &str = "category\tlcc\tfit\talias_of\theading";

impl ArxivMap {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut direct = BTreeMap::new();
        let mut aliases = BTreeMap::new();
        for (line, c) in rows(text, ARXIV_HEADER)? {
            let (cat, lcc, fit, alias) = (c[0], c[1], c[2], c[3]);
            let exact = match fit {
                "exact" => true,
                "broad" => false,
                _ => {
                    return Err(format!(
                        "line {line}: fit must be exact or broad, got {fit:?}"
                    ))
                }
            };
            if direct.contains_key(cat) || aliases.contains_key(cat) {
                return Err(format!("line {line}: duplicate category {cat}"));
            }
            match (lcc.is_empty(), alias.is_empty()) {
                (false, true) => {
                    check_lcc(line, lcc)?;
                    direct.insert(
                        cat.to_string(),
                        ArxivEntry {
                            lcc: lcc.to_string(),
                            exact,
                        },
                    );
                }
                (true, false) => {
                    aliases.insert(cat.to_string(), (line, alias.to_string()));
                }
                _ => {
                    return Err(format!(
                        "line {line}: exactly one of lcc and alias_of must be set"
                    ))
                }
            }
        }
        for (cat, (line, target)) in aliases {
            let entry = direct
                .get(&target)
                .cloned()
                .ok_or_else(|| format!("line {line}: alias target {target} has no direct lcc"))?;
            direct.insert(cat, entry);
        }
        Ok(ArxivMap(direct))
    }

    /// Category names in a taxonomy snapshot (`category\tname`) missing from this map.
    pub fn missing_from(&self, taxonomy: &str) -> Result<BTreeSet<String>, String> {
        Ok(rows(taxonomy, "category\tname")?
            .into_iter()
            .map(|(_, c)| c[0].to_string())
            .filter(|c| !self.0.contains_key(c))
            .collect())
    }

    /// Resolve a category, falling back to its archive (`math.XX` -> `math`).
    pub fn resolve(&self, category: &str) -> Option<&ArxivEntry> {
        self.0
            .get(category)
            .or_else(|| self.0.get(category.split('.').next()?))
    }
}

/// The category carried by an old-style arXiv ID (`hep-th/9901001`,
/// `math.AG/0101001`, optionally `arXiv:`-prefixed or versioned). New-style
/// IDs (`2101.00001`) carry none: their category needs arXiv metadata.
pub fn arxiv_category_from_id(id: &str) -> Option<&str> {
    let id = id.strip_prefix("arXiv:").unwrap_or(id);
    let (cat, number) = id.split_once('/')?;
    let digits = number.split('v').next()?;
    (digits.len() == 7 && digits.chars().all(|c| c.is_ascii_digit()) && !cat.is_empty())
        .then_some(cat)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainRule {
    pub domain: String,
    pub path_prefix: String,
    /// LCC class, or `=arxiv` to delegate to the arXiv map.
    pub lcc: String,
    pub ddc: String,
    pub decides: bool,
}

pub const DOMAIN_HEADER: &str = "domain\tpath_prefix\tlcc\tddc\tstrength\tnote";

pub struct DomainRules(pub Vec<DomainRule>);

impl DomainRules {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut seen = BTreeSet::new();
        let mut rules = Vec::new();
        for (line, c) in rows(text, DOMAIN_HEADER)? {
            let (domain, path, lcc, ddc, strength) = (c[0], c[1], c[2], c[3], c[4]);
            if domain.is_empty()
                || domain != domain.to_ascii_lowercase()
                || domain.contains('/')
                || !domain.contains('.')
            {
                return Err(format!(
                    "line {line}: {domain:?} is not a lowercase host name"
                ));
            }
            if !path.is_empty() && !path.starts_with('/') {
                return Err(format!("line {line}: path_prefix must start with /"));
            }
            if !seen.insert((domain, path)) {
                return Err(format!("line {line}: duplicate rule {domain}{path}"));
            }
            if lcc != "=arxiv" {
                check_lcc(line, lcc)?;
            }
            if !ddc.is_empty() && normalize::ddc(ddc).map(|d| d.full) != Some(ddc.to_string()) {
                return Err(format!(
                    "line {line}: {ddc:?} is not a normalized DDC number"
                ));
            }
            let decides = match strength {
                "decide" => true,
                "hint" => false,
                _ => return Err(format!("line {line}: strength must be decide or hint")),
            };
            rules.push(DomainRule {
                domain: domain.into(),
                path_prefix: path.into(),
                lcc: lcc.into(),
                ddc: ddc.into(),
                decides,
            });
        }
        Ok(DomainRules(rules))
    }

    /// The most specific rule matching `url`: longest domain, then longest path prefix.
    pub fn resolve(&self, url: &str) -> Option<&DomainRule> {
        let rest = url.split_once("://").map_or(url, |(_, r)| r);
        let (authority, path) = rest
            .find('/')
            .map_or((rest, "/"), |i| (&rest[..i], &rest[i..]));
        let host = authority
            .rsplit('@')
            .next()?
            .split(':')
            .next()?
            .to_ascii_lowercase();
        let host = host.trim_end_matches('.');
        self.0
            .iter()
            .filter(|r| host == r.domain || host.ends_with(&format!(".{}", r.domain)))
            .filter(|r| path.starts_with(&r.path_prefix))
            .max_by_key(|r| (r.domain.len(), r.path_prefix.len()))
    }
}
