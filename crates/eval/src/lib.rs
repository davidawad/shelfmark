//! The eval harness inside the binary (plan U10; R16, R17, PD3).
//!
//! The caller runs each profile's cascade over the gold set and hands
//! over one [`Outcome`] per example and profile, with every class as its
//! path from the scheme root. This crate only scores and fits:
//!
//! - [`summarize`]: top-1/top-k accuracy, ECE, flagged rate and latency
//!   per profile, per deciding tier and per hierarchy level.
//! - [`fit`]: the PD3 calibration: per tier, gold-set accuracy binned by
//!   raw score and made monotone by pool-adjacent-violators.
//! - [`thresholds`]: the lowest raw score whose calibrated confidence
//!   reaches a target (R17).

mod fit;
mod metrics;

pub use fit::{fit, fit_tier, thresholds, Sample};
pub use metrics::{summarize, Outcome, Slices, DEFAULT_K};
