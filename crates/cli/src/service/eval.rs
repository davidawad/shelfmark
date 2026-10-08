//! `eval` (R16, R17, PD3): every runnable profile's cascade over the gold
//! set, scored by `shelfmark-eval`, and the per-tier calibration fitted
//! from it. A previewable mutation: the preview reports the metrics and
//! the proposed calibration; `--commit` writes `calibration.json` and
//! `eval.json`, after appending a `--gold` file's new examples to the
//! store's gold set so `status` judges freshness against them.
//!
//! Each profile runs twice. The fit pass is uncalibrated, so PD3 bins each
//! tier's raw top score; the measure pass applies the fitted calibration,
//! so the metrics describe what `classify` does after the commit. Both
//! passes use the same examples, so the metrics are in-sample.

use std::collections::BTreeMap;
use std::time::Instant;

use serde_json::{json, Value};
use shelfmark_core::calibration::{gold_set_hash, Calibration, EvalSummary, GoldExample};
use shelfmark_core::cascade::{self, Decision};
use shelfmark_core::placement::{Status, TierRunState, Trace};
use shelfmark_core::scheme::{CodeCheck, CompiledIndex};
use shelfmark_core::tier::{ClassifyInput, Profile, Tier};
use shelfmark_core::{Error, ErrorCode, ProfileId, Request};
use shelfmark_eval::{Outcome, Sample, DEFAULT_K};

use super::{gold_file, Shelfmark};
use crate::args::Args;
use crate::store::Store;

/// A gold Work ready to classify: its input and every acceptable class
/// as a path from the scheme root.
struct Labelled {
    input: ClassifyInput,
    acceptable: Vec<Vec<String>>,
}

impl Shelfmark {
    pub(super) fn eval(&self, req: &Request, args: &Args) -> Result<Value, Error> {
        let k = args.usize("k")?.unwrap_or(DEFAULT_K).max(1);
        let (profiles, skipped_profiles) = self.eval_profiles(args)?;
        let mut store = self.store()?;
        let index = self.scheme_arg(store.as_ref(), args)?;
        let stored = store.gold()?;
        let imported = match args.str("gold")? {
            Some(path) => Some(gold_file::read(path, index.as_ref(), &stored)?),
            None => None,
        };
        let new: Vec<GoldExample> = imported
            .as_ref()
            .map(|i| i.examples.clone())
            .unwrap_or_default();
        let gold: Vec<GoldExample> = stored.iter().chain(&new).cloned().collect();
        let (labelled, unusable) = labelled(store.as_ref(), index.as_ref(), &gold)?;
        if labelled.is_empty() {
            return Err(Error::new(
                ErrorCode::NotFound,
                format!("no usable gold examples in {} to evaluate", index.scheme()),
            )
            .details(json!({ "gold": gold.len(), "unusable": unusable }))
            .next("shelfmark eval --gold <gold.ndjson>"));
        }
        let tiers: BTreeMap<String, &dyn Tier> = self
            .wiring
            .tiers
            .iter()
            .map(|(n, t)| (n.clone(), t.as_ref()))
            .collect();
        let run = |p: &Profile, l: &Labelled, cal: Option<&Calibration>| {
            let start = Instant::now();
            let decision = match cascade::run(p, &tiers, index.as_ref(), &l.input, None, cal) {
                Ok(d) => Some(d),
                Err(e) if e.code == ErrorCode::NotFound => None,
                Err(e) => return Err(e),
            };
            Ok((decision, start.elapsed().as_micros() as u64))
        };

        // The fit pass lets no tier decide, so every tier runs on every
        // example and each gets samples, not only on what earlier tiers
        // missed (margins never exceed 1).
        let mut samples = Vec::new();
        for p in &profiles {
            let fit_profile = Profile {
                min_margin: 2.0,
                ..p.clone()
            };
            for l in &labelled {
                if let (Some(d), _) = run(&fit_profile, l, None)? {
                    samples.extend(tier_samples(&d.trace, l, index.as_ref()));
                }
            }
        }
        let now = crate::time::now_rfc3339();
        let gold_hash = gold_set_hash(&gold);
        let calibration = shelfmark_eval::fit(
            index.scheme().clone(),
            gold_hash.clone(),
            gold.len(),
            now.clone(),
            &samples,
        );
        let mut outcomes = Vec::new();
        for p in &profiles {
            for l in &labelled {
                let (d, micros) = run(p, l, Some(&calibration))?;
                outcomes.push(outcome(&p.id, d, l, micros, index.as_ref()));
            }
        }
        let slices = shelfmark_eval::summarize(&outcomes, k);
        let summary = EvalSummary {
            scheme: index.scheme().clone(),
            gold_hash,
            gold_count: gold.len(),
            at: now,
            per_profile: slices.per_profile,
            per_tier: slices.per_tier,
            per_level: slices.per_level,
        };
        let change = json!({
            "eval": summary,
            "calibration": calibration,
            "thresholds": thresholds(&profiles, &calibration),
            "gold": {
                "stored": stored.len(),
                "imported": new.len(),
                "duplicates": imported.as_ref().map_or(0, |i| i.duplicates),
                "skipped": imported.as_ref().map_or(Vec::new(), |i| i.skipped.clone()),
                "evaluated": labelled.len(),
                "unusable": unusable,
            },
            "skipped_profiles": skipped_profiles,
        });
        self.mutate(req, store.as_mut(), change, |s| {
            for g in &new {
                s.append_gold(g)?;
            }
            s.save_eval(&summary, &calibration)
        })
    }

    /// The profile asked for, else every runnable one, plus the ones
    /// skipped and the tiers each is missing.
    fn eval_profiles(&self, args: &Args) -> Result<(Vec<Profile>, Vec<Value>), Error> {
        let missing = |p: &Profile| -> Vec<String> {
            p.tiers
                .iter()
                .filter(|t| !self.wiring.tiers.contains_key(*t))
                .cloned()
                .collect()
        };
        if let Some(id) = args.id::<ProfileId>("profile")? {
            let p = self.profile(Some(&id))?;
            let gone = missing(&p);
            if !gone.is_empty() {
                return Err(Error::new(
                    ErrorCode::TierUnavailable,
                    format!("{id} needs tiers this build lacks: {}", gone.join(", ")),
                )
                .next("shelfmark status"));
            }
            return Ok((vec![p], Vec::new()));
        }
        let (run, skip): (Vec<&Profile>, Vec<&Profile>) =
            self.wiring.profiles.iter().partition(|p| self.runnable(p));
        let skipped = skip
            .iter()
            .map(|p| json!({ "profile": p.id, "missing_tiers": missing(p) }))
            .collect();
        Ok((run.into_iter().cloned().collect(), skipped))
    }
}

/// A class code as its path from the scheme root.
fn path(index: &dyn CompiledIndex, code: &str) -> Vec<String> {
    let mut out: Vec<String> = index
        .ancestors(code)
        .into_iter()
        .map(|r| r.id.code)
        .collect();
    if out.is_empty() {
        out.push(code.to_string());
    }
    out.reverse();
    out
}

/// The gold examples in `index`'s scheme grouped by Work. A Work's
/// acceptable classes are those its latest example shares a source with:
/// every class a gold file accepted, or the one a person last confirmed
/// or corrected. Also counts what could not be used, and why.
fn labelled(
    store: &dyn Store,
    index: &dyn CompiledIndex,
    gold: &[GoldExample],
) -> Result<(Vec<Labelled>, Value), Error> {
    let mut by_work: BTreeMap<String, Vec<&GoldExample>> = BTreeMap::new();
    for g in gold.iter().filter(|g| &g.scheme == index.scheme()) {
        by_work.entry(g.work.to_string()).or_default().push(g);
    }
    let (mut no_input, mut not_in_scheme) = (0, 0);
    let mut out = Vec::new();
    for group in by_work.values() {
        let latest = group.last().expect("groups are non-empty");
        let mut acceptable: Vec<Vec<String>> = Vec::new();
        for g in group
            .iter()
            .filter(|g| g.from_placement == latest.from_placement)
        {
            if let CodeCheck::Exact { class } | CodeCheck::Within { class } =
                index.check_code(&g.class.code)
            {
                let p = path(index, &class.code);
                if !acceptable.contains(&p) {
                    acceptable.push(p);
                }
            }
        }
        if acceptable.is_empty() {
            not_in_scheme += 1;
            continue;
        }
        let input = if is_empty(&latest.input) {
            store.input(&latest.work)?
        } else {
            Some(latest.input.clone())
        };
        match input.filter(|i| !is_empty(i)) {
            Some(input) => out.push(Labelled { input, acceptable }),
            None => no_input += 1,
        }
    }
    Ok((
        out,
        json!({ "no_input": no_input, "class_not_in_scheme": not_in_scheme }),
    ))
}

fn is_empty(input: &ClassifyInput) -> bool {
    input.text.trim().is_empty()
        && input.title.is_none()
        && input.identifiers.is_empty()
        && input.url.is_none()
        && input.headings.is_empty()
}

/// For each tier that ran, its top raw score and whether that class was
/// acceptable (PD3's fit input).
fn tier_samples(trace: &Trace, l: &Labelled, index: &dyn CompiledIndex) -> Vec<Sample> {
    let mut out = Vec::new();
    for run in trace.tiers.iter().filter(|r| r.state == TierRunState::Ran) {
        let mut best: Option<(f64, &str)> = None;
        for c in &trace.candidates {
            if let Some(s) = c.per_tier.iter().find(|s| s.tier == run.tier) {
                if best.is_none_or(|(b, _)| s.score > b) {
                    best = Some((s.score, &c.class.code));
                }
            }
        }
        if let Some((raw, code)) = best {
            out.push(Sample {
                tier: run.tier.clone(),
                raw,
                correct: l.acceptable.contains(&path(index, code)),
            });
        }
    }
    out
}

/// A decision as the scorer sees it; no decision is a flagged miss at
/// confidence 0, as mc-eval scores it.
fn outcome(
    profile: &ProfileId,
    decision: Option<Decision>,
    l: &Labelled,
    micros: u64,
    index: &dyn CompiledIndex,
) -> Outcome {
    let (tier, ranked, confidence, flagged) = match decision {
        Some(d) => {
            // The decided class first: a later tier can decide a class the
            // pool ranks below an undecided earlier tier's top.
            let mut ranked = vec![path(index, &d.class.code)];
            ranked.extend(
                d.trace
                    .candidates
                    .iter()
                    .filter(|c| c.class != d.class)
                    .map(|c| path(index, &c.class.code)),
            );
            (
                d.trace.deciding_tier.clone(),
                ranked,
                d.confidence,
                d.status == Status::Flagged,
            )
        }
        None => (None, Vec::new(), 0.0, true),
    };
    Outcome {
        profile: profile.to_string(),
        tier,
        ranked,
        acceptable: l.acceptable.clone(),
        confidence,
        flagged,
        micros,
    }
}

/// R17: per profile and tier, the lowest raw score whose calibrated
/// confidence reaches the profile's default target.
fn thresholds(profiles: &[Profile], calibration: &Calibration) -> Value {
    let mut out = serde_json::Map::new();
    for p in profiles {
        let tiers: Vec<Value> = p
            .tiers
            .iter()
            .filter_map(|t| {
                let c = calibration.tiers.get(t)?;
                Some(json!({
                    "tier": t,
                    "target": p.default_target,
                    "raw_score": shelfmark_eval::thresholds(c, p.default_target),
                }))
            })
            .collect();
        out.insert(p.id.to_string(), Value::Array(tiers));
    }
    Value::Object(out)
}
