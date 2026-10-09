//! `eurovoc_in_skos_core_concepts.rdf`: RDF/XML in which every resource is
//! an `rdf:Description`. Three kinds matter:
//!
//! - a `skos:Concept` in the `domains` scheme: one of the 21 domains;
//! - a `skos:ConceptScheme` with a notation: one of the 127 microthesauri
//!   (the EuroVoc and domains schemes carry no notation and are skipped);
//! - any other `skos:Concept`: a descriptor, with `skos:broader` links, the
//!   microthesauri it belongs to (`skos:inScheme`), its English preferred
//!   label and English alternative labels.
//!
//! Only English labels are read. Everything the builder would otherwise have
//! to guess at is an error: a missing English label, a notation that is not a
//! domain or microthesaurus code, a link to a resource the file does not
//! describe, or a descriptor outside every microthesaurus.

use std::collections::BTreeMap;

const SKOS: &str = "http://www.w3.org/2004/02/skos/core#";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
/// The scheme holding all descriptors, and the scheme holding the domains.
const MAIN_SCHEME: &str = "100141";
const DOMAINS_SCHEME: &str = "domains";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Domain {
    /// Two-digit notation, e.g. `04`.
    pub code: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Microthesaurus {
    /// Four-digit notation, e.g. `0406`; its first two digits are the domain.
    pub code: String,
    pub label: String,
    pub domain: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub id: String,
    pub label: String,
    /// English alternative labels, without duplicates.
    pub alt: Vec<String>,
    /// Ids of every `skos:broader` descriptor, in [`id_key`] order.
    pub broader: Vec<String>,
    /// Codes of every microthesaurus the descriptor belongs to, ascending.
    pub mts: Vec<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thesaurus {
    pub domains: Vec<Domain>,
    pub mts: Vec<Microthesaurus>,
    pub terms: Vec<Term>,
}

pub fn thesaurus(xml: &[u8]) -> Result<Thesaurus, String> {
    let text = std::str::from_utf8(xml).map_err(|e| format!("source is not UTF-8: {e}"))?;
    let doc = roxmltree::Document::parse(text).map_err(|e| format!("source XML: {e}"))?;
    let mut domains = Vec::new();
    let mut mt_codes: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut raw = Vec::new();
    for d in doc.root_element().children().filter(|n| n.is_element()) {
        let id = last_segment(
            d.attribute((RDF, "about"))
                .ok_or("a resource has no rdf:about")?,
        );
        let kinds: Vec<&str> = d
            .children()
            .filter(|n| n.has_tag_name((RDF, "type")))
            .filter_map(|n| n.attribute((RDF, "resource")))
            .collect();
        let schemes = resources(d, "inScheme");
        if kinds.contains(&concat_skos("ConceptScheme").as_str()) {
            if let Some(code) = text_of(d, "notation") {
                mt_codes.insert(id.to_string(), (code, label_of(d, &id)?));
            }
        } else if schemes.iter().any(|s| s == DOMAINS_SCHEME) {
            let code =
                text_of(d, "notation").ok_or_else(|| format!("domain {id} has no notation"))?;
            domains.push(Domain {
                label: strip_code(&label_of(d, &id)?, &code),
                code,
            });
        } else {
            raw.push((id.to_string(), d, schemes));
        }
    }
    let mts = microthesauri(&mt_codes, &domains)?;
    let by_scheme: BTreeMap<&str, &str> = mt_codes
        .iter()
        .map(|(k, v)| (k.as_str(), v.0.as_str()))
        .collect();
    let mut terms = raw
        .into_iter()
        .map(|(id, d, schemes)| term(&id, d, &schemes, &by_scheme))
        .collect::<Result<Vec<_>, _>>()?;
    domains.sort_by(|a, b| a.code.cmp(&b.code));
    terms.sort_by_key(|t| id_key(&t.id));
    check(&domains, &terms)?;
    Ok(Thesaurus {
        domains,
        mts,
        terms,
    })
}

fn microthesauri(
    codes: &BTreeMap<String, (String, String)>,
    domains: &[Domain],
) -> Result<Vec<Microthesaurus>, String> {
    let mut out = codes
        .values()
        .map(|(code, raw)| {
            let ok = code.len() == 4 && code.bytes().all(|b| b.is_ascii_digit());
            let domain = code[..2.min(code.len())].to_string();
            if !ok || !domains.iter().any(|d| d.code == domain) {
                return Err(format!(
                    "microthesaurus {code} is not a four-digit code in a known domain"
                ));
            }
            Ok(Microthesaurus {
                label: strip_code(raw, code),
                code: code.clone(),
                domain,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    out.sort_by(|a, b| a.code.cmp(&b.code));
    Ok(out)
}

fn term(
    id: &str,
    d: roxmltree::Node,
    schemes: &[String],
    by_scheme: &BTreeMap<&str, &str>,
) -> Result<Term, String> {
    let mut mts = Vec::new();
    for s in schemes.iter().filter(|s| *s != MAIN_SCHEME) {
        let code = by_scheme
            .get(s.as_str())
            .ok_or_else(|| format!("{id} is in unknown scheme {s}"))?;
        mts.push((*code).to_string());
    }
    mts.sort();
    mts.dedup();
    if mts.is_empty() {
        return Err(format!("descriptor {id} is in no microthesaurus"));
    }
    let mut broader = resources(d, "broader");
    broader.sort_by_key(|b| id_key(b));
    broader.dedup();
    let label = label_of(d, id)?;
    let mut alt: Vec<String> = Vec::new();
    for a in english(d, "altLabel") {
        if a != label && !alt.contains(&a) {
            alt.push(a);
        }
    }
    Ok(Term {
        id: id.to_string(),
        label,
        alt,
        broader,
        mts,
        scope: english(d, "scopeNote").into_iter().next(),
    })
}

/// Every cross-reference resolves and no id repeats.
fn check(domains: &[Domain], terms: &[Term]) -> Result<(), String> {
    if domains.is_empty() || terms.is_empty() {
        return Err("source has no domains or no descriptors".into());
    }
    let ids: std::collections::BTreeSet<&str> = terms.iter().map(|t| t.id.as_str()).collect();
    if ids.len() != terms.len() {
        return Err("a descriptor id appears twice".into());
    }
    for t in terms {
        if let Some(b) = t.broader.iter().find(|b| !ids.contains(b.as_str())) {
            return Err(format!(
                "{} has broader {b}, which the source does not describe",
                t.id
            ));
        }
        if t.broader.contains(&t.id) {
            return Err(format!("{} is its own broader term", t.id));
        }
    }
    Ok(())
}

/// Sort key of a descriptor id: numeric ids first by value, then the
/// hash-style ids (`c_58d943b0`) by text.
pub fn id_key(id: &str) -> (bool, u64, String) {
    match id.parse::<u64>() {
        Ok(n) => (false, n, String::new()),
        Err(_) => (true, 0, id.to_string()),
    }
}

fn concat_skos(name: &str) -> String {
    format!("{SKOS}{name}")
}

fn last_segment(uri: &str) -> String {
    uri.rsplit('/').next().unwrap_or(uri).to_string()
}

fn resources(d: roxmltree::Node, name: &str) -> Vec<String> {
    d.children()
        .filter(|n| n.has_tag_name((SKOS, name)))
        .filter_map(|n| n.attribute((RDF, "resource")))
        .map(last_segment)
        .collect()
}

fn text_of(d: roxmltree::Node, name: &str) -> Option<String> {
    let t = d
        .children()
        .find(|n| n.has_tag_name((SKOS, name)))?
        .text()?;
    Some(t.trim().to_string()).filter(|s| !s.is_empty())
}

/// Texts of every English `skos:<name>`, whitespace collapsed.
fn english(d: roxmltree::Node, name: &str) -> Vec<String> {
    d.children()
        .filter(|n| n.has_tag_name((SKOS, name)))
        .filter(|n| n.attribute((roxmltree::NS_XML_URI, "lang")) == Some("en"))
        .filter_map(|n| n.text())
        .map(|t| t.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|t| !t.is_empty())
        .collect()
}

fn label_of(d: roxmltree::Node, id: &str) -> Result<String, String> {
    let mut labels = english(d, "prefLabel");
    match labels.len() {
        1 => Ok(labels.remove(0)),
        n => Err(format!("{id} has {n} English preferred labels, expected 1")),
    }
}

/// `"6011 animal product"` with code `6011` is `"animal product"`.
fn strip_code(label: &str, code: &str) -> String {
    label
        .strip_prefix(code)
        .map_or(label, str::trim)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_leading_codes() {
        assert_eq!(strip_code("6011 fixture item", "6011"), "fixture item");
        assert_eq!(strip_code("04 FIXTURE DOMAIN ALPHA", "04"), "FIXTURE DOMAIN ALPHA");
        assert_eq!(strip_code("no code", "04"), "no code");
    }

    #[test]
    fn rejects_unresolved_broader_terms() {
        let xml = br#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
            xmlns:skos="http://www.w3.org/2004/02/skos/core#">
          <rdf:Description rdf:about="http://x/domains"><rdf:type rdf:resource="http://www.w3.org/2004/02/skos/core#ConceptScheme"/></rdf:Description>
          <rdf:Description rdf:about="http://x/1"><rdf:type rdf:resource="http://www.w3.org/2004/02/skos/core#Concept"/>
            <skos:inScheme rdf:resource="http://x/domains"/><skos:notation>04</skos:notation>
            <skos:prefLabel xml:lang="en">04 FIXTURE DOMAIN ALPHA</skos:prefLabel></rdf:Description>
          <rdf:Description rdf:about="http://x/2"><rdf:type rdf:resource="http://www.w3.org/2004/02/skos/core#ConceptScheme"/>
            <skos:notation>0406</skos:notation><skos:prefLabel xml:lang="en">0406 fixture group one</skos:prefLabel></rdf:Description>
          <rdf:Description rdf:about="http://x/3"><rdf:type rdf:resource="http://www.w3.org/2004/02/skos/core#Concept"/>
            <skos:inScheme rdf:resource="http://x/2"/><skos:broader rdf:resource="http://x/9"/>
            <skos:prefLabel xml:lang="en">orphan</skos:prefLabel></rdf:Description>
        </rdf:RDF>"#;
        let e = thesaurus(xml).unwrap_err();
        assert!(e.contains("broader 9"), "{e}");
    }
}
