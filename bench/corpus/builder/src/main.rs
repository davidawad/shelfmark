mod oai;
mod relabel;

use oai::parse;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Paper {
    id: String,
    title: String,
    abstract_text: String,
    authors: Vec<String>,
    categories: Vec<String>,
    msc: Vec<String>,
    acm_ccs: Vec<String>,
    jel: Vec<String>,
    license: String,
    source: Source,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Source {
    kind: String,
    url: String,
    metadata_license: String,
    label_sources: BTreeMap<String, Vec<String>>,
}

fn source_for(
    id: &str,
    categories: &[String],
    msc: &[String],
    acm: &[String],
    jel: &[String],
) -> Source {
    Source {
        kind: "arxiv".into(),
        url: format!("https://arxiv.org/abs/{id}"),
        metadata_license: "CC0 1.0".into(),
        label_sources: BTreeMap::from([
            ("arxiv".into(), categories.to_vec()),
            ("msc".into(), msc.to_vec()),
            ("acm_ccs".into(), acm.to_vec()),
            ("jel".into(), jel.to_vec()),
        ]),
    }
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

fn write_jsonl(path: &str, papers: &[Paper]) -> io::Result<()> {
    let mut out = io::BufWriter::new(fs::File::create(path)?);
    for p in papers {
        serde_json::to_writer(&mut out, p)?;
        writeln!(out)?;
    }
    out.flush()
}

/// Rewrite ACM classes of a pinned corpus from arXivOld records only.
fn relabel_acm(corpus: &str, xml: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut rows = fs::read_to_string(corpus)?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<Vec<Paper>, _>>()?;
    let records = oai::parse_records(&fs::read_to_string(xml)?).map_err(io::Error::other)?;
    let summary = relabel::relabel(&mut rows, &records).map_err(io::Error::other)?;
    eprintln!("{summary:?}");
    write_jsonl(output, &rows)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() == 5 && args[1] == "--relabel-acm" {
        return relabel_acm(&args[2], &args[3], &args[4]);
    }
    if args.len() != 4 {
        eprintln!("usage: paper-corpus-builder <OAI-XML> <COUNT> <OUTPUT.jsonl>");
        eprintln!(
            "       paper-corpus-builder --relabel-acm <CORPUS.jsonl> <OAI-XML> <OUTPUT.jsonl>"
        );
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
    write_jsonl(&args[3], &selected)?;
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

    const RAW_NS: &str = r#"xmlns="http://arxiv.org/OAI/arXivRaw/""#;

    fn raw_record(id: &str, msc: &str, acm: &str) -> String {
        format!(
            "<record><metadata><arXivRaw {RAW_NS}><id>{id}</id><title>T</title>\
             <categories>cs.LG math.OC</categories><msc-class>{msc}</msc-class>\
             <acm-class>{acm}</acm-class><abstract>A.</abstract></arXivRaw></metadata></record>"
        )
    }

    fn old_record(id: &str, msc: &str, acm: &str) -> String {
        format!(
            "<record><metadata><arXivOld xmlns=\"http://arxiv.org/OAI/arXivOld/\">\
             <id>{id}</id><title>T</title><categories>cs.LG math.OC</categories>\
             <msc-class>{msc}</msc-class><acm-class>{acm}</acm-class>\
             </arXivOld></metadata></record>"
        )
    }

    #[test]
    fn arxiv_raw_record_keeps_msc_and_acm_fields_separate() {
        let xml = format!(
            "<OAI-PMH>{}</OAI-PMH>",
            raw_record("2602.00001", "68T05, 90C26", "I.2.6; G.1.6")
        );
        let parsed = parse(&xml).unwrap();
        let p = &parsed[0];
        assert_eq!(p.msc, vec!["68T05", "90C26"]);
        assert_eq!(p.acm_ccs, vec!["I.2.6", "G.1.6"]);
        assert_eq!(p.source.label_sources["msc"], p.msc);
        assert_eq!(p.source.label_sources["acm_ccs"], p.acm_ccs);
    }

    #[test]
    fn mirrored_acm_class_is_dropped_or_taken_from_arxiv_old() {
        let mirrored = raw_record("2602.00002", "93C10, 34D45", "93C10, 34D45");
        let xml = format!("<OAI-PMH>{mirrored}</OAI-PMH>");
        let p = &parse(&xml).unwrap()[0];
        assert_eq!(p.msc, vec!["93C10", "34D45"]);
        assert!(p.acm_ccs.is_empty());
        assert!(p.source.label_sources["acm_ccs"].is_empty());

        let old = old_record(
            "2602.00002",
            "93C10, 34D45",
            "I.2.8 [Problem Solving]: Graph search, scheduling; I.6",
        );
        let merged = parse(&format!("<OAI-PMH>{old}{mirrored}</OAI-PMH>")).unwrap();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].msc, vec!["93C10", "34D45"]);
        assert_eq!(merged[0].acm_ccs, vec!["I.2.8", "I.6"]);
        assert_eq!(
            merged[0].source.label_sources["acm_ccs"],
            vec!["I.2.8", "I.6"]
        );
    }

    #[test]
    fn relabel_replaces_only_acm_classes_of_pinned_rows() {
        let mut rows = vec![
            Paper {
                acm_ccs: vec!["93C10".into()],
                ..parse(&raw_record("2602.00003", "93C10", "93C10")).unwrap()[0].clone()
            },
            parse(&raw_record("2602.00004", "", "")).unwrap()[0].clone(),
        ];
        let before = rows.clone();
        let xml = format!(
            "<OAI-PMH>{}{}</OAI-PMH>",
            old_record("2602.00003", "93C10", "I.2.8"),
            old_record("2602.00004", "05C85", "")
        );
        let records = oai::parse_records(&xml).unwrap();
        let summary = relabel::relabel(&mut rows, &records).unwrap();
        assert_eq!(rows[0].acm_ccs, vec!["I.2.8"]);
        assert_eq!(rows[0].source.label_sources["acm_ccs"], vec!["I.2.8"]);
        assert_eq!(rows[1], before[1]);
        assert_eq!(
            (rows[0].title.as_str(), &rows[0].msc),
            (before[0].title.as_str(), &before[0].msc)
        );
        assert_eq!(
            summary,
            relabel::Summary {
                rows: 2,
                acm_rows_before: 1,
                acm_values_before: 1,
                acm_rows_after: 1,
                acm_values_after: 1,
                msc_mismatches: vec!["2602.00004".into()],
            }
        );
        let partial = oai::parse_records(&old_record("2602.00003", "", "I.2.8")).unwrap();
        assert!(relabel::relabel(&mut rows, &partial).is_err());
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
                license: "CC0 1.0".into(),
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
