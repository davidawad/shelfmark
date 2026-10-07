//! MinHash signatures over word 5-shingles (plan PD2, R3).
//!
//! Text is normalized with normalization v1 (the same as Work IDs), split
//! into words, and every run of [`SHINGLE`] consecutive words is hashed.
//! Each of [`PERMUTATIONS`] universal hashes `(a*x + b) mod (2^61 - 1)`
//! keeps its minimum over the shingle set. The `a, b` pairs come from a
//! SplitMix64 stream with a fixed seed, and the shingle hash is FNV-1a, so
//! a signature is the same on every run, machine and platform. Changing
//! any of that is a new [`SCHEME`].

use std::fmt;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use shelfmark_core::id::normalize_text_v1;

/// Words per shingle.
pub const SHINGLE: usize = 5;
/// Hash functions per signature.
pub const PERMUTATIONS: usize = 128;
/// Names this exact derivation; stored next to every persisted signature.
pub const SCHEME: &str = "mh1-w5-p128";

const MERSENNE_61: u64 = (1 << 61) - 1;
const SEED: u64 = 0x5348_454c_464d_524b; // "SHELFMRK"

/// One step of SplitMix64: advances `state`, returns the next output.
pub(crate) fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// 64-bit FNV-1a.
pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The fixed `(a, b)` coefficients, `a` in `[1, p)` and `b` in `[0, p)`.
fn coefficients() -> &'static [(u64, u64); PERMUTATIONS] {
    static COEFFS: std::sync::OnceLock<[(u64, u64); PERMUTATIONS]> = std::sync::OnceLock::new();
    COEFFS.get_or_init(|| {
        let mut state = SEED;
        let mut out = [(0, 0); PERMUTATIONS];
        for pair in &mut out {
            let a = splitmix64(&mut state) % (MERSENNE_61 - 1) + 1;
            let b = splitmix64(&mut state) % MERSENNE_61;
            *pair = (a, b);
        }
        out
    })
}

fn permute(x: u64, (a, b): (u64, u64)) -> u64 {
    ((u128::from(a) * u128::from(x) + u128::from(b)) % u128::from(MERSENNE_61)) as u64
}

/// Hashes of the word 5-shingles of `text` after normalization v1, sorted
/// and deduplicated. A text with 1-4 words is one shingle; an empty text
/// has none.
pub fn shingles(text: &str) -> Vec<u64> {
    let normalized = normalize_text_v1(text);
    let words: Vec<&str> = normalized.split(' ').filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<u64> = words
        .windows(SHINGLE.min(words.len()))
        .map(|w| fnv1a(w.join(" ").as_bytes()) % MERSENNE_61)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// A MinHash signature: one minimum per permutation.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Signature(pub [u64; PERMUTATIONS]);

impl Signature {
    /// The signature of `text`. An empty text gives the all-`u64::MAX`
    /// signature, which is similar to nothing (see [`Signature::is_empty`]).
    pub fn of_text(text: &str) -> Signature {
        Signature::of_shingles(&shingles(text))
    }

    pub fn of_shingles(shingles: &[u64]) -> Signature {
        let mut mins = [u64::MAX; PERMUTATIONS];
        for (min, &coeff) in mins.iter_mut().zip(coefficients()) {
            for &x in shingles {
                *min = (*min).min(permute(x, coeff));
            }
        }
        Signature(mins)
    }

    /// True for the signature of a text with no words.
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&m| m == u64::MAX)
    }

    /// Estimated Jaccard similarity of the two shingle sets: the share of
    /// permutations whose minima agree. Empty signatures match nothing.
    pub fn similarity(&self, other: &Signature) -> f64 {
        if self.is_empty() || other.is_empty() {
            return 0.0;
        }
        let same = self.0.iter().zip(&other.0).filter(|(a, b)| a == b).count();
        same as f64 / PERMUTATIONS as f64
    }

    /// Lowercase hex, 16 digits per permutation.
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(PERMUTATIONS * 16);
        for m in self.0 {
            s.push_str(&format!("{m:016x}"));
        }
        s
    }
}

/// Exact Jaccard similarity of two sorted, deduplicated shingle sets.
pub fn jaccard(a: &[u64], b: &[u64]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let (mut i, mut j, mut both) = (0, 0, 0usize);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                both += 1;
                i += 1;
                j += 1;
            }
        }
    }
    both as f64 / (a.len() + b.len() - both) as f64
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({:016x}..)", self.0[0])
    }
}

/// Why a string is not a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadSignature;

impl fmt::Display for BadSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a signature is {} lowercase hex digits",
            PERMUTATIONS * 16
        )
    }
}

impl FromStr for Signature {
    type Err = BadSignature;
    fn from_str(s: &str) -> Result<Self, BadSignature> {
        if s.len() != PERMUTATIONS * 16
            || !s
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(BadSignature);
        }
        let mut mins = [0u64; PERMUTATIONS];
        for (i, m) in mins.iter_mut().enumerate() {
            *m = u64::from_str_radix(&s[i * 16..(i + 1) * 16], 16).map_err(|_| BadSignature)?;
        }
        Ok(Signature(mins))
    }
}

impl Serialize for Signature {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(de::Error::custom)
    }
}
