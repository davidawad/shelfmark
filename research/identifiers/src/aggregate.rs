//! Folding edition facts into the lookup table and its statistics.
//!
//! Folding is order-independent: every count is a sum and every sample is
//! the lexicographically smallest set, so parallel parsing cannot change the
//! output.

use crate::edition::{EditionFacts, Key, FIELDS};
use crate::normalize::ClassNumber;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub const INVALID_SAMPLE: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Scheme {
    Lcc,
    Ddc,
}

impl Scheme {
    pub fn name(self) -> &'static str {
        match self {
            Scheme::Lcc => "lcc",
            Scheme::Ddc => "ddc",
        }
    }
}

/// One distinct class number seen in the dump.
#[derive(Debug)]
pub struct Value {
    pub scheme: Scheme,
    pub class: ClassNumber,
}

#[derive(Debug, Default, Serialize)]
pub struct FieldStats {
    pub editions_with_field: u64,
    pub values_valid: u64,
    pub values_invalid: u64,
    /// The lexicographically smallest rejected raw values.
    pub invalid_sample: BTreeSet<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct EditionStats {
    pub with_any_key: u64,
    pub with_lcc: u64,
    pub with_ddc: u64,
    pub with_key_and_lcc: u64,
    pub with_key_and_ddc: u64,
    pub with_key_and_any_class: u64,
    /// Editions with a usable LCC, by first class letter.
    pub lcc_by_main_class: BTreeMap<String, u64>,
    /// Editions with a usable DDC, by DDC main class digit.
    pub ddc_by_main_class: BTreeMap<String, u64>,
    /// Editions with any usable class, by `source_records` prefix
    /// (an edition with several sources counts once per distinct prefix).
    pub class_by_source: BTreeMap<String, u64>,
}

#[derive(Debug, Default, Serialize)]
pub struct Stats {
    pub dump_lines: u64,
    pub edition_records: u64,
    pub parse_errors: u64,
    pub fields: BTreeMap<String, FieldStats>,
    pub editions: EditionStats,
    /// Filled in after the table is folded; see `crate::output`.
    pub keys: BTreeMap<String, crate::output::KeyStats>,
    pub table: crate::output::TableStats,
}

/// Everything the dump says about one identifier.
#[derive(Debug, Default)]
pub struct Entry {
    /// Editions carrying this identifier and at least one class.
    pub editions: u32,
    /// (value id, number of editions asserting it)
    pub values: Vec<(u32, u32)>,
}

#[derive(Default)]
pub struct Table {
    pub values: Vec<Value>,
    value_ids: HashMap<(Scheme, String), u32>,
    pub entries: HashMap<Key, Entry>,
    /// Every valid identifier in the dump, classified or not (coverage denominator).
    pub seen_isbn: HashSet<u64>,
    pub seen_oclc: HashSet<u64>,
    pub seen_lccn: HashSet<Box<str>>,
    pub stats: Stats,
}

impl Table {
    pub fn new() -> Self {
        let mut t = Table::default();
        for f in FIELDS {
            t.stats.fields.insert(f.to_string(), FieldStats::default());
        }
        t
    }

    fn value_id(&mut self, scheme: Scheme, class: &ClassNumber) -> u32 {
        if let Some(id) = self.value_ids.get(&(scheme, class.full.clone())) {
            return *id;
        }
        let id = self.values.len() as u32;
        self.values.push(Value {
            scheme,
            class: class.clone(),
        });
        self.value_ids.insert((scheme, class.full.clone()), id);
        id
    }

    pub fn fold(&mut self, f: EditionFacts) {
        self.stats.edition_records += 1;
        for (i, name) in FIELDS.iter().enumerate() {
            let fs = self.stats.fields.get_mut(*name).expect("field registered");
            fs.editions_with_field += f.present[i] as u64;
            fs.values_valid += f.counts[i].valid;
            fs.values_invalid += f.counts[i].invalid;
        }
        for (i, raw) in f.rejected {
            let sample = &mut self
                .stats
                .fields
                .get_mut(FIELDS[i])
                .expect("field registered")
                .invalid_sample;
            if sample.len() < INVALID_SAMPLE || sample.last().is_some_and(|last| raw < *last) {
                sample.insert(raw);
                if sample.len() > INVALID_SAMPLE {
                    sample.pop_last();
                }
            }
        }

        let e = &mut self.stats.editions;
        let has_key = !f.keys.is_empty();
        let (has_lcc, has_ddc) = (!f.lcc.is_empty(), !f.ddc.is_empty());
        e.with_any_key += has_key as u64;
        e.with_lcc += has_lcc as u64;
        e.with_ddc += has_ddc as u64;
        e.with_key_and_lcc += (has_key && has_lcc) as u64;
        e.with_key_and_ddc += (has_key && has_ddc) as u64;
        e.with_key_and_any_class += (has_key && (has_lcc || has_ddc)) as u64;
        let lcc_tops: BTreeSet<String> = f.lcc.iter().map(|c| c.top[..1].to_string()).collect();
        for t in lcc_tops {
            *e.lcc_by_main_class.entry(t).or_default() += 1;
        }
        let ddc_tops: BTreeSet<&str> = f.ddc.iter().map(|c| c.top.as_str()).collect();
        for t in ddc_tops {
            *e.ddc_by_main_class.entry(t.to_string()).or_default() += 1;
        }
        for s in &f.sources {
            *e.class_by_source.entry(s.clone()).or_default() += 1;
        }

        for key in &f.keys {
            match key {
                Key::Isbn(n) => self.seen_isbn.insert(*n),
                Key::Oclc(n) => self.seen_oclc.insert(*n),
                Key::Lccn(s) => self.seen_lccn.insert(s.clone()),
            };
        }
        if !has_lcc && !has_ddc || !has_key {
            return;
        }
        let ids: Vec<u32> = f
            .lcc
            .iter()
            .map(|c| (Scheme::Lcc, c))
            .chain(f.ddc.iter().map(|c| (Scheme::Ddc, c)))
            .map(|(s, c)| self.value_id(s, c))
            .collect();
        for key in f.keys {
            let entry = self.entries.entry(key).or_default();
            entry.editions += 1;
            for id in &ids {
                match entry.values.iter_mut().find(|(v, _)| v == id) {
                    Some((_, support)) => *support += 1,
                    None => entry.values.push((*id, 1)),
                }
            }
        }
    }
}
