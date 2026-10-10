//! `learned-scheme build|insert|quantize|subset|split-ids`; see README.md.

mod cli;

use std::collections::BTreeMap;

const USAGE: &str = "usage:
  learned-scheme build --corpus C.jsonl --graph G.json[.gz] --out scheme.json
      [--subset ids.txt] [--vectors V.f32] [--labels labels.tsv] [--allow-auto-labels]
      [--centroids out.json.gz] [--placements out.jsonl] [--report out.json] [--sheet out.md]
      [--lambda 0.005] [--branch-cost 1] [--max-depth 4] [--max-children 12] [--min-leaf 5]
      [--seed 20261010] [--version 2026-10-10] [--model NAME]
  learned-scheme insert --scheme S.json --centroids C.json.gz --corpus R.jsonl --vectors V.json.gz
      --out scheme.json --placements out.jsonl [--ids ids.txt] [--base-placements P.jsonl]
      [--report out.json] [--split-after 12] [--min-child 5] [--max-cos 0.5]
  learned-scheme quantize --corpus R.jsonl --vectors V.f32 --out V.json.gz [--ids ids.txt] [--model NAME]
  learned-scheme subset --corpus C.jsonl --graph G.json[.gz] --ids ids.txt
      --out-corpus out.jsonl --out-graph out.json [--placements P.jsonl --out-placements out.jsonl]
  learned-scheme split-ids --corpus C.jsonl --fraction 0.2 --seed S --train train.txt --test test.txt";

pub struct Args {
    values: BTreeMap<String, String>,
    flags: Vec<String>,
}

impl Args {
    fn parse(raw: &[String]) -> Result<Self, String> {
        let mut values = BTreeMap::new();
        let mut flags = Vec::new();
        let mut i = 0;
        while i < raw.len() {
            let key = raw[i]
                .strip_prefix("--")
                .ok_or(format!("unexpected argument {}", raw[i]))?;
            if key.starts_with("allow-") {
                flags.push(key.to_owned());
            } else {
                i += 1;
                let v = raw.get(i).ok_or(format!("missing value for --{key}"))?;
                values.insert(key.to_owned(), v.clone());
            }
            i += 1;
        }
        Ok(Args { values, flags })
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
    pub fn need(&self, key: &str) -> Result<&str, String> {
        self.get(key).ok_or(format!("missing --{key}"))
    }
    pub fn num<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T, String> {
        match self.get(key) {
            None => Ok(default),
            Some(v) => v.parse().map_err(|_| format!("bad --{key} value {v}")),
        }
    }
    pub fn flag(&self, key: &str) -> bool {
        self.flags.iter().any(|f| f == key)
    }
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let result = match raw.split_first() {
        Some((cmd, rest)) => Args::parse(rest).and_then(|a| match cmd.as_str() {
            "build" => cli::build(&a),
            "insert" => cli::insert(&a),
            "quantize" => cli::quantize(&a),
            "subset" => cli::subset(&a),
            "split-ids" => cli::split_ids(&a),
            _ => Err(USAGE.into()),
        }),
        None => Err(USAGE.into()),
    };
    if let Err(e) = result {
        eprintln!("learned-scheme: {e}");
        std::process::exit(2);
    }
}
