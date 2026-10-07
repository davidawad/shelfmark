//! Machine capability and profile recommendation (R9).
//!
//! [`detect`] reads total RAM, logical CPUs and the SIMD features CPU
//! model runtimes rely on. [`assess`] checks each profile against a
//! [`Capability`] and the tiers the caller can actually run, and
//! recommends the richest profile that fits. It is a pure function, so
//! callers and tests can pass any capability.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use shelfmark_core::tier::Profile;
use shelfmark_core::ProfileId;

use crate::builtin::tier_spec;

/// A profile may plan on at most this share of total RAM, leaving the
/// rest to the OS and the caller.
pub const MEMORY_BUDGET_PERCENT: u64 = 50;
/// Model tiers need at least this many logical CPUs.
pub const MODEL_MIN_CPUS: u32 = 2;

/// What this machine offers the cascade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub total_memory_mb: u64,
    pub logical_cpus: u32,
    /// x86-64 AVX2, used by quantized CPU model runtimes.
    pub avx2: bool,
    /// AArch64 NEON, the ARM counterpart.
    pub neon: bool,
    /// `std::env::consts::OS`, e.g. `linux`, `macos`, `windows`.
    pub os: String,
    /// `std::env::consts::ARCH`, e.g. `x86_64`, `aarch64`.
    pub arch: String,
}

impl Capability {
    /// Memory a profile may plan on: [`MEMORY_BUDGET_PERCENT`] of RAM.
    pub fn memory_budget_mb(&self) -> u64 {
        self.total_memory_mb * MEMORY_BUDGET_PERCENT / 100
    }

    /// Whether the CPU has the SIMD a quantized model runtime needs.
    pub fn has_model_simd(&self) -> bool {
        self.avx2 || self.neon
    }
}

/// Read this machine's capability.
pub fn detect() -> Capability {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    Capability {
        total_memory_mb: sys.total_memory() / (1024 * 1024),
        logical_cpus: std::thread::available_parallelism()
            .map(|n| u32::try_from(n.get()).unwrap_or(u32::MAX))
            .unwrap_or(1),
        avx2: has_avx2(),
        neon: has_neon(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
    }
}

#[cfg(target_arch = "x86_64")]
fn has_avx2() -> bool {
    std::arch::is_x86_feature_detected!("avx2")
}

#[cfg(not(target_arch = "x86_64"))]
fn has_avx2() -> bool {
    false
}

#[cfg(target_arch = "aarch64")]
fn has_neon() -> bool {
    std::arch::is_aarch64_feature_detected!("neon")
}

#[cfg(not(target_arch = "aarch64"))]
fn has_neon() -> bool {
    false
}

/// Whether one profile fits, and why not.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assessment {
    pub profile: ProfileId,
    pub fits: bool,
    /// Empty when the profile fits.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
    pub expected_memory_mb: u32,
    pub expected_micros_per_item: u64,
}

/// Every profile assessed, plus the one to use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recommendation {
    pub capability: Capability,
    /// The richest fitting profile; `None` when nothing fits.
    pub recommended: Option<ProfileId>,
    pub profiles: Vec<Assessment>,
}

fn assess_one(profile: &Profile, cap: &Capability, runnable: &BTreeSet<String>) -> Assessment {
    let mut reasons = Vec::new();
    let budget = cap.memory_budget_mb();
    if u64::from(profile.expected_memory_mb) > budget {
        reasons.push(format!(
            "needs {} MB; budget is {budget} MB ({MEMORY_BUDGET_PERCENT}% of {} MB)",
            profile.expected_memory_mb, cap.total_memory_mb
        ));
    }
    let missing: Vec<&str> = profile
        .tiers
        .iter()
        .filter(|t| !runnable.contains(*t))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        reasons.push(format!("tiers not runnable here: {}", missing.join(", ")));
    }
    let needs_model = profile
        .tiers
        .iter()
        .any(|t| tier_spec(t).is_some_and(|s| s.needs_model));
    if needs_model {
        if !cap.has_model_simd() {
            reasons.push(format!(
                "model tiers need AVX2 or NEON; {} has neither",
                cap.arch
            ));
        }
        if cap.logical_cpus < MODEL_MIN_CPUS {
            reasons.push(format!(
                "model tiers need {MODEL_MIN_CPUS} logical CPUs; found {}",
                cap.logical_cpus
            ));
        }
    }
    Assessment {
        profile: profile.id.clone(),
        fits: reasons.is_empty(),
        reasons,
        expected_memory_mb: profile.expected_memory_mb,
        expected_micros_per_item: profile.expected_micros_per_item,
    }
}

/// Assess `profiles` (cheapest first, as [`crate::builtins`] returns
/// them) against `cap`. `runnable` names the tiers the caller can run:
/// registered and not reporting `tier_unavailable`. The recommendation is
/// the last profile in the list that fits.
pub fn assess(
    cap: &Capability,
    profiles: &[Profile],
    runnable: &BTreeSet<String>,
) -> Recommendation {
    let assessed: Vec<Assessment> = profiles
        .iter()
        .map(|p| assess_one(p, cap, runnable))
        .collect();
    let recommended = assessed
        .iter()
        .rev()
        .find(|a| a.fits)
        .map(|a| a.profile.clone());
    Recommendation {
        capability: cap.clone(),
        recommended,
        profiles: assessed,
    }
}
