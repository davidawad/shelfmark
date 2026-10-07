//! The eval report: one summary per profile, and per deciding tier within it.

use crate::bootstrap::summarize_with_ci;
use crate::gold::{stats, GoldSet};
use crate::outcome::{missing, score, Outcome};
use crate::pred::Loaded;
use crate::summary::{Config, Summary};
use serde::Serialize;
use std::collections::BTreeMap;

pub const REPORT_SCHEMA: &str = "mc-eval/report@1";

/// Profiles named `experimental:<name>` (e.g. `experimental:laya-decision`)
/// are reported but marked as comparisons, not product tiers.
pub const EXPERIMENTAL_PREFIX: &str = "experimental:";

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MissingPolicy {
    /// A gold item without a prediction counts as wrong at zero confidence.
    AsWrong,
    /// Gold items without a prediction are left out of that profile.
    Skip,
}

#[derive(Debug, Clone, Serialize)]
pub struct Options {
    #[serde(flatten)]
    pub metrics: Config,
    pub bootstrap: usize,
    pub seed: u64,
    pub confidence: f64,
    pub missing: MissingPolicy,
}

#[derive(Debug, Serialize)]
pub struct GoldInfo {
    pub path: String,
    pub sha256: String,
    pub items: usize,
    pub main_classes: usize,
}

#[derive(Debug, Serialize)]
pub struct TierReport {
    pub tier: String,
    /// Share of this profile's predictions decided by the tier.
    pub share: f64,
    #[serde(flatten)]
    pub summary: Summary,
}

#[derive(Debug, Serialize)]
pub struct ProfileReport {
    pub profile: String,
    pub experimental: bool,
    pub predicted: usize,
    #[serde(flatten)]
    pub summary: Summary,
    pub tiers: Vec<TierReport>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub gold: GoldInfo,
    pub options: Options,
    pub warnings: Vec<String>,
    pub profiles: Vec<ProfileReport>,
}

pub fn run(
    gold: &GoldSet,
    gold_path: &str,
    gold_sha256: String,
    loaded: &Loaded,
    opt: &Options,
) -> Report {
    let mut warnings = Vec::new();
    if !loaded.unknown.is_empty() {
        let shown: Vec<_> = loaded
            .unknown
            .iter()
            .take(5)
            .map(|(_, k)| k.as_str())
            .collect();
        warnings.push(format!(
            "{} prediction(s) name items not in the gold set and were ignored (e.g. {})",
            loaded.unknown.len(),
            shown.join(", ")
        ));
    }
    let mut by_profile: BTreeMap<&str, Vec<Outcome>> = BTreeMap::new();
    let mut tier_of: BTreeMap<(&str, usize), &str> = BTreeMap::new();
    for p in &loaded.predictions {
        by_profile
            .entry(&p.profile)
            .or_default()
            .push(score(gold, p));
        tier_of.insert((&p.profile, p.gold), &p.tier);
    }
    let summarize = |outs: &[&Outcome]| {
        summarize_with_ci(outs, &opt.metrics, opt.bootstrap, opt.seed, opt.confidence)
    };
    let mut profiles = Vec::new();
    for (profile, mut outs) in by_profile {
        let predicted = outs.len();
        if predicted < gold.len() {
            let gap = gold.len() - predicted;
            match opt.missing {
                MissingPolicy::AsWrong => {
                    warnings.push(format!(
                        "profile `{profile}`: {gap} gold item(s) unpredicted, scored as wrong"
                    ));
                    let have: std::collections::HashSet<_> = outs.iter().map(|o| o.gold).collect();
                    outs.extend(
                        (0..gold.len())
                            .filter(|i| !have.contains(i))
                            .map(|i| missing(gold, i)),
                    );
                }
                MissingPolicy::Skip => warnings.push(format!(
                    "profile `{profile}`: {gap} gold item(s) unpredicted, skipped"
                )),
            }
        }
        outs.sort_by_key(|o| o.gold);
        let all: Vec<&Outcome> = outs.iter().collect();
        let mut tiers: BTreeMap<&str, Vec<&Outcome>> = BTreeMap::new();
        for o in outs.iter().filter(|o| !o.missing) {
            tiers
                .entry(tier_of[&(profile, o.gold)])
                .or_default()
                .push(o);
        }
        let tiers = tiers
            .into_iter()
            .map(|(tier, os)| TierReport {
                tier: tier.to_string(),
                share: os.len() as f64 / predicted as f64,
                summary: summarize(&os),
            })
            .collect();
        profiles.push(ProfileReport {
            profile: profile.to_string(),
            experimental: profile.starts_with(EXPERIMENTAL_PREFIX),
            predicted,
            summary: summarize(&all),
            tiers,
        });
    }
    if profiles.is_empty() {
        warnings.push("no predictions matched the gold set".into());
    }
    Report {
        schema: REPORT_SCHEMA,
        gold: GoldInfo {
            path: gold_path.to_string(),
            sha256: gold_sha256,
            items: gold.len(),
            main_classes: stats(gold).main_classes.len(),
        },
        options: opt.clone(),
        warnings,
        profiles,
    }
}
