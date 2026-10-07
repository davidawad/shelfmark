//! MinHash determinism: the signature is pinned to golden values, so a
//! change of seed, hash or shingling (which would silently orphan every
//! stored signature) fails here and needs a new scheme name.

mod common;

use shelfmark_core::WorkId;
use shelfmark_identity::index::{BANDS, ROWS};
use shelfmark_identity::minhash::{jaccard, shingles, PERMUTATIONS, SCHEME, SHINGLE};
use shelfmark_identity::{NearDupIndex, Signature};

const FOX: &str = "The quick brown fox jumps over the lazy dog, again and again.";

#[test]
fn parameters_match_the_plan() {
    assert_eq!((SHINGLE, PERMUTATIONS, SCHEME), (5, 128, "mh1-w5-p128"));
    assert_eq!(BANDS * ROWS, PERMUTATIONS);
}

#[test]
fn signature_is_pinned_across_runs() {
    let s = Signature::of_text(FOX);
    assert_eq!(s.0[0], 0x0009_a913_eccd_b918);
    assert_eq!(s.0[63], 0x03bc_20c5_686b_6d0e);
    assert_eq!(s.0[127], 0x0282_25c0_9759_7527);
    assert_eq!(
        shelfmark_core::id::sha256_hex(s.to_hex().as_bytes()),
        "0f4648b898ca8164daae0a4ec2065fcf93aa08fdced211915f74d135b7ca3b7c"
    );
    assert_eq!(s, Signature::of_text(FOX), "same process, same answer");
}

#[test]
fn signature_ignores_what_normalization_ignores() {
    let loud = "THE   quick—brown FOX\njumps over the lazy dog!! Again; and again";
    assert_eq!(Signature::of_text(FOX), Signature::of_text(loud));
}

#[test]
fn hex_round_trips_and_rejects_garbage() {
    let s = Signature::of_text(FOX);
    let hex = s.to_hex();
    assert_eq!(hex.len(), PERMUTATIONS * 16);
    assert_eq!(hex.parse::<Signature>().unwrap(), s);
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(serde_json::from_str::<Signature>(&json).unwrap(), s);
    assert!("abc".parse::<Signature>().is_err());
    assert!(hex.to_uppercase().parse::<Signature>().is_err());
}

#[test]
fn short_and_empty_texts() {
    assert_eq!(shingles("two words").len(), 1);
    assert!(shingles(" ... ").is_empty());
    let empty = Signature::of_text("");
    assert!(empty.is_empty());
    assert_eq!(empty.similarity(&empty), 0.0, "empty is similar to nothing");
    let mut index = NearDupIndex::new();
    index.insert(WorkId::from_text(""), empty.clone());
    assert!(index.similar(&empty, 0.0, &[]).is_empty());
}

#[test]
fn estimate_tracks_exact_jaccard() {
    let a = common::corpus(800, 3);
    for every in [200, 40, 10] {
        let b = common::ocr_noise(&a, every);
        let exact = jaccard(&shingles(&a), &shingles(&b));
        let est = Signature::of_text(&a).similarity(&Signature::of_text(&b));
        assert!(
            (exact - est).abs() < 0.15,
            "every {every}: exact {exact} est {est}"
        );
    }
}

#[test]
fn index_orders_by_similarity_then_work_id() {
    let base = common::corpus(1000, 11);
    let near = common::ocr_noise(&base, 120);
    let mut index = NearDupIndex::new();
    let (wb, wn) = (WorkId::from_text(&base), WorkId::from_text(&near));
    assert!(index.insert(wb.clone(), Signature::of_text(&base)));
    assert!(index.insert(wn.clone(), Signature::of_text(&near)));
    assert!(
        !index.insert(wb.clone(), Signature::of_text(&base)),
        "no duplicates"
    );
    let hits = index.similar(&Signature::of_text(&base), 0.5, &[]);
    assert_eq!(hits[0], (wb.clone(), 1.0));
    assert_eq!(hits[1].0, wn);
    let hits = index.similar(&Signature::of_text(&base), 0.5, &[&wb]);
    assert_eq!(hits.len(), 1);
}
