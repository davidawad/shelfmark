//! Deterministic test text: long enough for 5-shingles to be meaningful,
//! varied enough that unrelated texts share almost no shingles.

#![allow(dead_code)]

const SYLLABLES: &[&str] = &[
    "ka", "lo", "mi", "ten", "ras", "vel", "dor", "pi", "su", "gan", "be", "thor", "qua", "ny",
    "ex", "ol", "mar", "ci", "fen", "ru", "zo", "hal", "wem", "tri",
];

fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state >> 33
}

/// `n` pseudo-words from `seed`, as sentences of twelve words.
pub fn corpus(n: usize, seed: u64) -> String {
    let mut state = seed;
    let mut out = String::new();
    for i in 0..n {
        let len = 2 + next(&mut state) % 3;
        let word: String = (0..len)
            .map(|_| SYLLABLES[(next(&mut state) % SYLLABLES.len() as u64) as usize])
            .collect();
        if i % 12 == 0 {
            if i > 0 {
                out.push_str(".\n");
            }
            let mut c = word.chars();
            out.extend(c.next().unwrap().to_uppercase());
            out.push_str(c.as_str());
        } else {
            out.push(' ');
            out.push_str(&word);
        }
    }
    out.push_str(".\n");
    out
}

/// Simulated OCR noise: in every `every`-th word, `m` reads as `rn`, `l`
/// as `1` and `o` as `0`.
pub fn ocr_noise(text: &str, every: usize) -> String {
    let mut n = 0;
    text.split(' ')
        .map(|w| {
            n += 1;
            if n % every == 0 {
                w.replace('m', "rn").replace('l', "1").replace('o', "0")
            } else {
                w.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
