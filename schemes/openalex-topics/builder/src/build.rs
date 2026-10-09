//! OpenAlex entities to scheme classes.
//!
//! Hierarchy: domain (`D3`) > field (`F31`) > subfield (`S3106`) > topic
//! (`T10346`). A code is the OpenAlex id's last path segment with a
//! one-letter entity prefix, so every code is stable across refreshes.
//! A topic's keywords become weighted vocabulary (weight = OpenAlex's
//! keyword score); a topic's description, and the descriptions of the
//! upper levels, become class notes. Alternative names of domains, fields
//! and subfields become vocabulary too.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use shelfmark_core::scheme::{ClassDef, SchemeDoc, Source, VocabLink};

pub const KEYWORD_SOURCE: &str = "openalex-keyword";
pub const ALT_SOURCE: &str = "openalex-alt";
pub const CODE_GRAMMAR: &str = "[DFST][0-9]+";

/// One level of the hierarchy as OpenAlex returns it.
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    /// Numeric or `T`-prefixed id tail, e.g. `3106`, `T10346`.
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub alternatives: Vec<String>,
    /// Tail of the parent's id.
    pub parent: Option<String>,
    pub keywords: Vec<Keyword>,
    /// The field and domain the record itself names (subfields and
    /// topics), checked against the hierarchy.
    pub field: Option<String>,
    pub domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Keyword {
    pub slug: String,
    pub name: String,
    pub score: f64,
}

pub struct Provenance {
    pub version: String,
    pub description: String,
    pub license: String,
}

/// Counts for the README and the build log.
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    pub classes: usize,
    pub per_level: Vec<usize>,
    pub depth: usize,
    pub max_branching: usize,
    /// Mean children per class that has any.
    pub mean_branching: f64,
    pub vocab: usize,
}

fn tail(id: &str) -> &str {
    id.rsplit('/').next().unwrap_or(id)
}

fn text(v: &Value) -> Option<String> {
    v.as_str()
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| !s.is_empty())
}

fn parent_of(v: &Value, key: &str) -> Option<String> {
    v[key]["id"].as_str().map(|s| tail(s).to_string())
}

/// Parse one API page's `results` into entities. `parent` names the
/// result key holding the parent (`domain`, `field`, `subfield`).
pub fn parse_page(body: &[u8], parent: Option<&str>) -> Result<Vec<Entity>, String> {
    let v: Value = serde_json::from_slice(body).map_err(|e| e.to_string())?;
    let results = v["results"].as_array().ok_or("page has no results array")?;
    results
        .iter()
        .map(|r| {
            let id = r["id"]
                .as_str()
                .map(|s| tail(s).to_string())
                .ok_or("result without id")?;
            let name = text(&r["display_name"]).ok_or_else(|| format!("{id}: no display_name"))?;
            let parent_id = match parent {
                Some(key) => Some(parent_of(r, key).ok_or_else(|| format!("{id}: no {key}"))?),
                None => None,
            };
            let alternatives = r["display_name_alternatives"]
                .as_array()
                .map(|a| a.iter().filter_map(text).collect())
                .unwrap_or_default();
            let keywords = r["keywords"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|k| {
                            Some(Keyword {
                                slug: tail(k["id"].as_str()?).to_string(),
                                name: text(&k["display_name"])?,
                                score: k["score"].as_f64()?,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Ok(Entity {
                id,
                name,
                description: text(&r["description"]),
                alternatives,
                parent: parent_id,
                keywords,
                field: parent_of(r, "field"),
                domain: parent_of(r, "domain"),
            })
        })
        .collect()
}

/// Numeric order of ids ("T9" before "T10"), then text.
fn id_key(id: &str) -> (u64, &str) {
    let n = id
        .trim_start_matches(|c: char| c.is_ascii_alphabetic())
        .parse()
        .unwrap_or(u64::MAX);
    (n, id)
}

/// Link vocabulary, one per distinct lowercase term, keeping the
/// strongest weight. Terms equal to the class label add nothing.
fn vocab(label: &str, links: impl Iterator<Item = VocabLink>) -> Vec<VocabLink> {
    let mut best: BTreeMap<String, VocabLink> = BTreeMap::new();
    for l in links {
        let key = l.term.to_lowercase();
        if key == label.to_lowercase() {
            continue;
        }
        match best.get(&key) {
            Some(old) if old.weight >= l.weight => {}
            _ => {
                best.insert(key, l);
            }
        }
    }
    let mut out: Vec<VocabLink> = best.into_values().collect();
    out.sort_by(|a, b| {
        b.weight
            .total_cmp(&a.weight)
            .then_with(|| a.term.cmp(&b.term))
    });
    out
}

fn alt_links(e: &Entity) -> impl Iterator<Item = VocabLink> + '_ {
    e.alternatives.iter().map(|t| VocabLink {
        term: t.clone(),
        source: ALT_SOURCE.into(),
        source_id: None,
        weight: 1.0,
    })
}

fn keyword_links(e: &Entity) -> impl Iterator<Item = VocabLink> + '_ {
    e.keywords
        .iter()
        .filter(|k| k.score > 0.0)
        .map(|k| VocabLink {
            term: k.name.clone(),
            source: KEYWORD_SOURCE.into(),
            source_id: Some(k.slug.clone()),
            weight: k.score.min(1.0),
        })
}

fn class(prefix: char, e: &Entity, parent: Option<&str>, links: Vec<VocabLink>) -> ClassDef {
    ClassDef {
        code: format!("{prefix}{}", e.id.trim_start_matches('T')),
        label: e.name.clone(),
        parent: parent.map(str::to_string),
        range: None,
        vocab: vocab(&e.name, links.into_iter()),
        note: e.description.clone(),
    }
}

/// Depth-first walk emitting `level` entities under each parent in id order.
struct Levels<'a> {
    fields: BTreeMap<&'a str, Vec<&'a Entity>>,
    subfields: BTreeMap<&'a str, Vec<&'a Entity>>,
    topics: BTreeMap<&'a str, Vec<&'a Entity>>,
}

fn group<'a>(
    items: &'a [Entity],
    what: &str,
) -> Result<BTreeMap<&'a str, Vec<&'a Entity>>, String> {
    let mut seen = BTreeSet::new();
    let mut out: BTreeMap<&str, Vec<&Entity>> = BTreeMap::new();
    for e in items {
        if !seen.insert(e.id.as_str()) {
            return Err(format!("duplicate {what} {}", e.id));
        }
        let p = e
            .parent
            .as_deref()
            .ok_or_else(|| format!("{what} {} has no parent", e.id))?;
        out.entry(p).or_default().push(e);
    }
    for v in out.values_mut() {
        v.sort_by(|a, b| id_key(&a.id).cmp(&id_key(&b.id)));
    }
    Ok(out)
}

/// Build the scheme. Checks that every parent exists, that ids are unique,
/// and that each topic's own field and domain agree with its subfield's.
pub fn scheme(
    domains: &[Entity],
    fields: &[Entity],
    subfields: &[Entity],
    topics: &[Entity],
    prov: &Provenance,
) -> Result<(SchemeDoc, Stats), String> {
    let mut sorted: Vec<&Entity> = domains.iter().collect();
    sorted.sort_by(|a, b| id_key(&a.id).cmp(&id_key(&b.id)));
    let levels = Levels {
        fields: group(fields, "field")?,
        subfields: group(subfields, "subfield")?,
        topics: group(topics, "topic")?,
    };
    check_parents(&levels, domains, fields, subfields)?;
    check_lineage(fields, subfields, topics)?;

    let mut classes = Vec::new();
    for d in sorted {
        let dc = class('D', d, None, alt_links(d).collect());
        let dcode = dc.code.clone();
        classes.push(dc);
        for f in levels.fields.get(d.id.as_str()).into_iter().flatten() {
            let fc = class('F', f, Some(&dcode), alt_links(f).collect());
            let fcode = fc.code.clone();
            classes.push(fc);
            for s in levels.subfields.get(f.id.as_str()).into_iter().flatten() {
                let sc = class('S', s, Some(&fcode), alt_links(s).collect());
                let scode = sc.code.clone();
                classes.push(sc);
                for t in levels.topics.get(s.id.as_str()).into_iter().flatten() {
                    classes.push(class('T', t, Some(&scode), keyword_links(t).collect()));
                }
            }
        }
    }
    let expected = domains.len() + fields.len() + subfields.len() + topics.len();
    if classes.len() != expected {
        return Err(format!(
            "{} of {expected} entities are unreachable from a domain",
            expected - classes.len()
        ));
    }
    let doc = SchemeDoc {
        schema: Some(shelfmark_core::scheme::SCHEME_SCHEMA_ID.to_string()),
        name: "openalex-topics".into(),
        version: prov.version.clone(),
        title: "OpenAlex Topics".into(),
        source: Some(Source {
            description: prov.description.clone(),
            url: Some("https://docs.openalex.org/api-entities/topics".into()),
            license: Some(prov.license.clone()),
        }),
        code_grammar: CODE_GRAMMAR.into(),
        classes,
    };
    let stats = stats(&doc);
    Ok((doc, stats))
}

fn check_parents(
    l: &Levels,
    domains: &[Entity],
    fields: &[Entity],
    subfields: &[Entity],
) -> Result<(), String> {
    let ids = |v: &[Entity]| v.iter().map(|e| e.id.clone()).collect::<BTreeSet<_>>();
    let (d, f, s) = (ids(domains), ids(fields), ids(subfields));
    for (parents, kids, what) in [
        (&d, &l.fields, "field"),
        (&f, &l.subfields, "subfield"),
        (&s, &l.topics, "topic"),
    ] {
        if let Some(p) = kids.keys().find(|p| !parents.contains(**p)) {
            return Err(format!("a {what} names parent {p}, which does not exist"));
        }
    }
    Ok(())
}

/// Subfields and topics also carry their own field and domain; they must
/// match the ones reached through the hierarchy, or the source disagrees
/// with itself.
fn check_lineage(fields: &[Entity], subfields: &[Entity], topics: &[Entity]) -> Result<(), String> {
    let domain_of_field: BTreeMap<&str, &str> = fields
        .iter()
        .filter_map(|f| Some((f.id.as_str(), f.parent.as_deref()?)))
        .collect();
    let field_of_subfield: BTreeMap<&str, &str> = subfields
        .iter()
        .filter_map(|s| Some((s.id.as_str(), s.parent.as_deref()?)))
        .collect();
    for (e, what) in subfields
        .iter()
        .map(|e| (e, "subfield"))
        .chain(topics.iter().map(|e| (e, "topic")))
    {
        let sub = if what == "topic" {
            e.parent.as_deref().unwrap_or_default()
        } else {
            e.id.as_str()
        };
        let field = field_of_subfield.get(sub).copied();
        let domain = field.and_then(|f| domain_of_field.get(f).copied());
        if e.field.as_deref() != field || e.domain.as_deref() != domain {
            return Err(format!(
                "{what} {} names field {:?} and domain {:?}, but its place in the hierarchy gives {field:?} and {domain:?}",
                e.id, e.field, e.domain
            ));
        }
    }
    Ok(())
}

/// Class counts, depth and branching of a built scheme.
pub fn stats(doc: &SchemeDoc) -> Stats {
    let level: BTreeMap<&str, usize> = {
        let by: BTreeMap<&str, &ClassDef> =
            doc.classes.iter().map(|c| (c.code.as_str(), c)).collect();
        doc.classes
            .iter()
            .map(|c| {
                let mut n = 0;
                let mut cur = c;
                while let Some(p) = cur.parent.as_deref().and_then(|p| by.get(p)) {
                    n += 1;
                    cur = p;
                }
                (c.code.as_str(), n)
            })
            .collect()
    };
    let mut per_level = Vec::new();
    for n in level.values() {
        if per_level.len() <= *n {
            per_level.resize(n + 1, 0);
        }
        per_level[*n] += 1;
    }
    let mut kids: BTreeMap<&str, usize> = BTreeMap::new();
    for c in &doc.classes {
        if let Some(p) = c.parent.as_deref() {
            *kids.entry(p).or_default() += 1;
        }
    }
    let internal = kids.len().max(1);
    Stats {
        classes: doc.classes.len(),
        depth: per_level.len(),
        per_level,
        max_branching: kids.values().copied().max().unwrap_or(0),
        mean_branching: kids.values().sum::<usize>() as f64 / internal as f64,
        vocab: doc.classes.iter().map(|c| c.vocab.len()).sum(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(results: &str) -> Vec<u8> {
        format!(r#"{{"results": {results}}}"#).into_bytes()
    }

    const LINEAGE: &str = r#""subfield":{"id":"https://openalex.org/subfields/3106"},"field":{"id":"https://openalex.org/fields/31"},"domain":{"id":"https://openalex.org/domains/3"}"#;

    fn fixture() -> [Vec<Entity>; 4] {
        let d = parse_page(
            &page(r#"[{"id":"https://openalex.org/domains/3","display_name":"Physical Sciences","description":"non-living systems","display_name_alternatives":[]}]"#),
            None,
        )
        .unwrap();
        let f = parse_page(
            &page(r#"[{"id":"https://openalex.org/fields/31","display_name":"Physics and Astronomy","display_name_alternatives":["physics"],"domain":{"id":"https://openalex.org/domains/3"}}]"#),
            Some("domain"),
        )
        .unwrap();
        let s = parse_page(
            &page(r#"[{"id":"https://openalex.org/subfields/3106","display_name":"Nuclear Physics","display_name_alternatives":["HEP"],"field":{"id":"https://openalex.org/fields/31"},"domain":{"id":"https://openalex.org/domains/3"}}]"#),
            Some("field"),
        )
        .unwrap();
        let t = parse_page(
            &page(&format!(
                r#"[
              {{"id":"https://openalex.org/T10346","display_name":"Fusion research","description":"Plasma  and\nfusion.",
               "keywords":[{{"id":"https://openalex.org/keywords/tokamak","display_name":"Tokamak","score":0.9}},
                           {{"id":"https://openalex.org/keywords/tokamak-2","display_name":"tokamak","score":0.95}},
                           {{"id":"https://openalex.org/keywords/same","display_name":"Fusion Research","score":0.9}},
                           {{"id":"https://openalex.org/keywords/zero","display_name":"zero","score":0}}],
               {LINEAGE}}},
              {{"id":"https://openalex.org/T9","display_name":"Early topic",{LINEAGE}}}]"#
            )),
            Some("subfield"),
        )
        .unwrap();
        [d, f, s, t]
    }

    fn prov() -> Provenance {
        Provenance {
            version: "2026-10-09".into(),
            description: "d".into(),
            license: "CC0".into(),
        }
    }

    #[test]
    fn builds_the_four_levels_in_numeric_id_order() {
        let [d, f, s, t] = fixture();
        let (doc, st) = scheme(&d, &f, &s, &t, &prov()).unwrap();
        let codes: Vec<&str> = doc.classes.iter().map(|c| c.code.as_str()).collect();
        assert_eq!(codes, ["D3", "F31", "S3106", "T9", "T10346"]);
        assert_eq!(doc.classes[4].parent.as_deref(), Some("S3106"));
        assert_eq!(st.per_level, [1, 1, 1, 2]);
        assert_eq!(st.depth, 4);
        assert_eq!(st.max_branching, 2);
        assert_eq!(st.mean_branching, 4.0 / 3.0);
    }

    #[test]
    fn keywords_dedupe_by_case_keep_best_weight_and_drop_label_and_zero() {
        let [d, f, s, t] = fixture();
        let (doc, _) = scheme(&d, &f, &s, &t, &prov()).unwrap();
        let topic = doc.classes.iter().find(|c| c.code == "T10346").unwrap();
        assert_eq!(topic.vocab.len(), 1);
        assert_eq!(topic.vocab[0].term, "tokamak");
        assert_eq!(topic.vocab[0].weight, 0.95);
        assert_eq!(topic.vocab[0].source, KEYWORD_SOURCE);
        assert_eq!(topic.note.as_deref(), Some("Plasma and fusion."));
        let sub = doc.classes.iter().find(|c| c.code == "S3106").unwrap();
        assert_eq!(sub.vocab[0].term, "HEP");
        assert_eq!(sub.vocab[0].source, ALT_SOURCE);
    }

    #[test]
    fn rejects_a_missing_parent_a_duplicate_and_a_lineage_mismatch() {
        let [d, f, s, t] = fixture();
        assert!(scheme(&[], &f, &s, &t, &prov()).is_err());
        let dup = [t.clone(), t[..1].to_vec()].concat();
        assert!(scheme(&d, &f, &s, &dup, &prov())
            .unwrap_err()
            .contains("duplicate"));
        let mut bad = t;
        bad[0].field = Some("99".into());
        assert!(scheme(&d, &f, &s, &bad, &prov())
            .unwrap_err()
            .contains("hierarchy"));
    }
}
