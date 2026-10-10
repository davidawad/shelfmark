use quick_xml::{events::Event, Reader};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct Paper {
    id: String,
    title: String,
    abstract_text: String,
    authors: Vec<String>,
    categories: Vec<String>,
    msc: Vec<String>,
    acm_ccs: Vec<String>,
    jel: Vec<String>,
    license: &'static str,
    source: Source,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct Source {
    kind: &'static str,
    url: String,
    metadata_license: &'static str,
    label_sources: BTreeMap<&'static str, Vec<String>>,
}

fn source_for(
    id: &str,
    categories: &[String],
    msc: &[String],
    acm: &[String],
    jel: &[String],
) -> Source {
    Source {
        kind: "arxiv",
        url: format!("https://arxiv.org/abs/{id}"),
        metadata_license: "CC0 1.0",
        label_sources: BTreeMap::from([
            ("arxiv", categories.to_vec()),
            ("msc", msc.to_vec()),
            ("acm_ccs", acm.to_vec()),
            ("jel", jel.to_vec()),
        ]),
    }
}

fn local_name(name: &[u8]) -> String {
    String::from_utf8_lossy(name)
        .rsplit(':')
        .next()
        .unwrap_or("")
        .to_string()
}

fn split_codes(s: &str) -> Vec<String> {
    s.split(|c: char| c == ',' || c.is_whitespace() || c == ';')
        .filter(|s| {
            !s.is_empty()
                && !s.eq_ignore_ascii_case("none")
                && !s.eq_ignore_ascii_case("n/a")
                && !s.eq_ignore_ascii_case("na")
        })
        .map(str::to_string)
        .collect()
}

fn jel_code_tokens(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    bytes
        .windows(3)
        .enumerate()
        .filter(|(i, w)| {
            w[0].is_ascii_uppercase()
                && w[1].is_ascii_digit()
                && w[2].is_ascii_digit()
                && (*i == 0 || !bytes[*i - 1].is_ascii_alphanumeric())
                && (*i + 3 == bytes.len() || !bytes[*i + 3].is_ascii_alphanumeric())
        })
        .map(|(_, w)| String::from_utf8_lossy(w).into_owned())
        .collect()
}

fn jel_from_comments(text: &str) -> Vec<String> {
    let Some(index) = text.to_ascii_lowercase().find("jel") else {
        return Vec::new();
    };
    jel_code_tokens(&text[index + 3..])
}

fn parse(xml: &str) -> Result<Vec<Paper>, String> {
    let mut r = Reader::from_str(xml);
    r.config_mut().trim_text(true);
    let (mut out, mut cur, mut field, mut author, mut forenames, mut comments) = (
        Vec::new(),
        None::<Paper>,
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    );
    loop {
        match r.read_event().map_err(|e| e.to_string())? {
            Event::Start(e) => {
                let n = local_name(e.name().as_ref());
                if n == "record" {
                    continue;
                }
                if n == "arXiv" || n == "arXivRaw" {
                    comments.clear();
                    cur = Some(Paper {
                        id: String::new(),
                        title: String::new(),
                        abstract_text: String::new(),
                        authors: vec![],
                        categories: vec![],
                        msc: vec![],
                        acm_ccs: vec![],
                        jel: vec![],
                        license: "CC0 1.0",
                        source: source_for("", &[], &[], &[], &[]),
                    });
                }
                if n == "author" {
                    author.clear();
                    forenames.clear();
                }
                field = n;
            }
            Event::Text(t) => {
                let val = t.unescape().map_err(|e| e.to_string())?.into_owned();
                if let Some(p) = cur.as_mut() {
                    match field.as_str() {
                        "id" => p.id.push_str(&val),
                        "title" => {
                            if !p.title.is_empty() {
                                p.title.push(' ');
                            }
                            p.title.push_str(&val);
                        }
                        "abstract" => {
                            if !p.abstract_text.is_empty() {
                                p.abstract_text.push(' ');
                            }
                            p.abstract_text.push_str(&val);
                        }
                        "keyname" => author = val,
                        "forenames" => forenames = val,
                        "authors" => {
                            p.authors = val
                                .split(',')
                                .flat_map(|s| s.split(" and "))
                                .map(str::trim)
                                .map(|s| s.trim_start_matches("and "))
                                .filter(|s| !s.is_empty())
                                .map(str::to_string)
                                .collect()
                        }
                        "categories" => p.categories.extend(split_codes(&val)),
                        "msc-class" => p.msc.extend(split_codes(&val)),
                        "acm-class" => p.acm_ccs.extend(split_codes(&val)),
                        "jel" => p.jel.extend(split_codes(&val)),
                        "comments" => comments.push_str(&val),
                        _ => {}
                    }
                }
            }
            Event::End(e) => {
                let n = local_name(e.name().as_ref());
                if n == "author" {
                    let name = [forenames.as_str(), author.as_str()]
                        .into_iter()
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    if !name.is_empty() {
                        if let Some(p) = cur.as_mut() {
                            p.authors.push(name)
                        }
                    }
                }
                if n == "arXiv" || n == "arXivRaw" {
                    if let Some(p) = cur.take() {
                        if !p.id.is_empty() {
                            let mut p = p;
                            p.jel.extend(jel_from_comments(&comments));
                            p.jel
                                .extend(p.categories.iter().flat_map(|c| jel_code_tokens(c)));
                            p.jel.sort();
                            p.jel.dedup();
                            p.source = source_for(&p.id, &p.categories, &p.msc, &p.acm_ccs, &p.jel);
                            out.push(p)
                        }
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out.dedup_by(|a, b| a.id == b.id);
    Ok(out)
}

fn rank(id: &str, seed: &str) -> [u8; 32] {
    Sha256::digest(format!("{seed}:{id}").as_bytes()).into()
}

fn top_archive(category: &str) -> &str {
    if category == "q-alg" {
        return "math";
    }
    if category == "chem-ph" {
        return "physics";
    }
    if [
        "astro-ph", "cond-mat", "gr-qc", "hep-ex", "hep-lat", "hep-ph", "hep-th", "math-ph",
        "nlin", "nucl-ex", "nucl-th", "physics", "quant-ph",
    ]
    .iter()
    .any(|prefix| category == *prefix || category.starts_with(&format!("{prefix}.")))
    {
        "physics"
    } else {
        category.split('.').next().unwrap_or(category)
    }
}

fn sample(papers: Vec<Paper>, n: usize, floor: usize, seed: &str) -> Vec<Paper> {
    let mut strata: BTreeMap<String, Vec<Paper>> = BTreeMap::new();
    for p in papers {
        let archive = p
            .categories
            .first()
            .map(|c| top_archive(c))
            .unwrap_or("unknown")
            .to_string();
        strata.entry(archive).or_default().push(p);
    }
    for v in strata.values_mut() {
        v.sort_by_key(|p| rank(&p.id, seed));
    }
    let total = strata.values().map(Vec::len).sum::<usize>();
    if n >= total {
        return strata.into_values().flatten().collect();
    }
    let mut quotas: BTreeMap<String, usize> = strata
        .iter()
        .map(|(k, v)| (k.clone(), floor.min(v.len())))
        .collect();
    let mut remaining = n.saturating_sub(quotas.values().sum());
    while remaining > 0 {
        let active = strata
            .iter()
            .filter(|(k, v)| quotas[*k] < v.len())
            .map(|(k, v)| (k.clone(), v.len() - quotas[k]))
            .collect::<Vec<_>>();
        if active.is_empty() {
            break;
        }
        let weight = active.iter().map(|(_, cap)| *cap).sum::<usize>();
        let mut added = 0;
        let mut fractions = Vec::new();
        for (k, cap) in &active {
            let numerator = remaining.saturating_mul(*cap);
            let share = (numerator / weight).min(*cap);
            *quotas.get_mut(k).unwrap() += share;
            added += share;
            fractions.push((numerator % weight, k.clone()));
        }
        remaining -= added;
        fractions.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        for (_, k) in fractions {
            if remaining == 0 {
                break;
            }
            if quotas[&k] < strata[&k].len() {
                *quotas.get_mut(&k).unwrap() += 1;
                remaining -= 1;
            }
        }
    }
    let mut chosen = Vec::with_capacity(n);
    for (k, quota) in quotas {
        chosen.extend(strata[&k].iter().take(quota).cloned());
    }
    chosen.sort_by(|a, b| a.id.cmp(&b.id));
    chosen
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: paper-corpus-builder <OAI-XML> <COUNT> <OUTPUT.jsonl>");
        std::process::exit(2);
    }
    let xml = fs::read_to_string(&args[1])?;
    let all = parse(&xml).map_err(io::Error::other)?;
    let count: usize = args[2].parse()?;
    let selected = sample(all, count, 100, "arxiv-corpus-v1-2026");
    if selected.len() != count {
        return Err(format!(
            "requested {count} records, but only {} are available",
            selected.len()
        )
        .into());
    }
    let mut out = fs::File::create(&args[3])?;
    for p in selected {
        serde_json::to_writer(&mut out, &p)?;
        writeln!(out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIXTURE: &str = r#"<OAI-PMH><record><metadata><arXivRaw><id>2601.12345</id><authors><author><keyname>Doe</keyname><forenames>Jane</forenames></author></authors><title>A &amp; B</title><categories>cs.CL math.ML</categories><msc-class>68T50, 62H30</msc-class><acm-class>I.2.7</acm-class><jel>D83</jel><comments>JEL codes: G12 and C63</comments><abstract>Text &amp; more.</abstract></arXivRaw></metadata></record><record><metadata><arXivRaw><id>2601.99999</id><title>Physics</title><categories>physics.optics</categories><acm-class>None</acm-class><abstract>Light.</abstract></arXivRaw></metadata></record></OAI-PMH>"#;

    #[test]
    fn fixture_parsing_and_stratified_sample_are_repeatable() {
        let parsed = parse(FIXTURE).unwrap();
        assert_eq!(parsed.len(), 2);
        let p = parsed.iter().find(|p| p.id == "2601.12345").unwrap();
        assert_eq!(p.title, "A & B");
        assert_eq!(p.authors, vec!["Jane Doe"]);
        assert_eq!(p.categories, vec!["cs.CL", "math.ML"]);
        assert_eq!(p.msc, vec!["68T50", "62H30"]);
        assert_eq!(p.acm_ccs, vec!["I.2.7"]);
        assert_eq!(p.jel, vec!["C63", "D83", "G12"]);
        assert!(parsed[1].acm_ccs.is_empty());
        assert_eq!(
            sample(parsed.clone(), 2, 1, "fixed"),
            sample(parsed, 2, 1, "fixed")
        );
    }

    #[test]
    fn sample_is_proportional_with_a_per_archive_floor() {
        let mut papers = parse(FIXTURE).unwrap();
        for i in 0..8 {
            papers.push(Paper {
                id: format!("cs-{i}"),
                title: String::new(),
                abstract_text: String::new(),
                authors: vec![],
                categories: vec!["cs.CL".into()],
                msc: vec![],
                acm_ccs: vec![],
                jel: vec![],
                license: "CC0 1.0",
                source: source_for("", &[], &[], &[], &[]),
            });
        }
        let selected = sample(papers, 5, 1, "fixed");
        assert_eq!(selected.len(), 5);
        assert!(selected
            .iter()
            .any(|p| p.categories[0].starts_with("physics")));
        assert!(
            selected
                .iter()
                .filter(|p| p.categories[0].starts_with("cs."))
                .count()
                >= 3
        );
        assert_eq!(top_archive("hep-th"), "physics");
        assert_eq!(top_archive("astro-ph.CO"), "physics");
        assert_eq!(top_archive("q-bio.GN"), "q-bio");
    }
}
