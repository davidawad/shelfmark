//! Percentile bootstrap over gold items with a seeded, dependency-free RNG,
//! so the same inputs and seed always give the same intervals.

use crate::outcome::Outcome;
use crate::summary::{percentile, summarize, Config, Summary};

/// SplitMix64: tiny, well-distributed, and stable across platforms.
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        SplitMix64(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n` (Lemire's multiply-shift; bias is negligible here).
    pub fn below(&mut self, n: usize) -> usize {
        ((self.next_u64() as u128 * n as u128) >> 64) as usize
    }
}

/// Summarize `outs` and attach `confidence`-level percentile intervals from
/// `resamples` bootstrap draws (no intervals when `resamples` is 0).
pub fn summarize_with_ci(
    outs: &[&Outcome],
    cfg: &Config,
    resamples: usize,
    seed: u64,
    confidence: f64,
) -> Summary {
    let mut full = summarize(outs, cfg);
    if resamples == 0 || outs.is_empty() {
        return full;
    }
    let width = full.stats_mut().len();
    let mut samples: Vec<Vec<f64>> = vec![Vec::with_capacity(resamples); width];
    let mut rng = SplitMix64::new(seed);
    let mut draw: Vec<&Outcome> = Vec::with_capacity(outs.len());
    for _ in 0..resamples {
        draw.clear();
        draw.extend((0..outs.len()).map(|_| outs[rng.below(outs.len())]));
        let mut s = summarize(&draw, cfg);
        // FAST stats are last and may be absent in a draw; zip keeps alignment.
        for (acc, st) in samples.iter_mut().zip(s.stats_mut()) {
            if st.value.is_finite() {
                acc.push(st.value);
            }
        }
    }
    let alpha = (1.0 - confidence) / 2.0;
    for (st, mut xs) in full.stats_mut().into_iter().zip(samples) {
        if xs.is_empty() || !st.value.is_finite() {
            continue;
        }
        xs.sort_by(f64::total_cmp);
        st.ci = Some([percentile(&xs, alpha), percentile(&xs, 1.0 - alpha)]);
    }
    full
}
