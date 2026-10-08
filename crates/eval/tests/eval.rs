//! Metrics slicing and the PD3 fit on hand-checked numbers.

use shelfmark_core::SchemeId;
use shelfmark_eval::{fit, fit_tier, summarize, thresholds, Outcome, Sample};

fn path(codes: &[&str]) -> Vec<String> {
    codes.iter().map(|c| c.to_string()).collect()
}

fn outcome(
    tier: Option<&str>,
    ranked: &[&[&str]],
    gold: &[&str],
    conf: f64,
    micros: u64,
) -> Outcome {
    Outcome {
        profile: "profile:lexical@1".into(),
        tier: tier.map(str::to_string),
        ranked: ranked.iter().map(|r| path(r)).collect(),
        acceptable: vec![path(gold)],
        confidence: conf,
        flagged: tier.is_none(),
        micros,
    }
}

#[test]
fn slices_per_profile_tier_and_level() {
    let outcomes = vec![
        // exact hit, decided by vocab
        outcome(
            Some("vocab"),
            &[&["Q", "QA", "QA76"]],
            &["Q", "QA", "QA76"],
            0.9,
            10,
        ),
        // right subclass, wrong caption; right class second
        outcome(
            None,
            &[&["Q", "QA", "QA9"], &["Q", "QA", "QA76"]],
            &["Q", "QA", "QA76"],
            0.4,
            20,
        ),
        // wrong main class, gold only two deep
        outcome(None, &[&["K", "KF"]], &["P", "PE"], 0.2, 30),
        // nothing produced
        outcome(None, &[], &["B", "BF"], 0.0, 40),
    ];
    let s = summarize(&outcomes, 5);
    let all = &s.per_profile["profile:lexical@1"];
    assert_eq!((all.n, all.k), (4, 5));
    assert_eq!(all.top1, 0.25);
    assert_eq!(all.topk, 0.5);
    assert_eq!(all.flagged_rate, 0.75);
    assert_eq!((all.p50_micros, all.p95_micros), (20, 40));
    // ECE: bins 0.9 (1 hit), 0.4 (miss), 0.2 (miss), 0.0 (miss).
    let want = (0.1 + 0.4 + 0.2 + 0.0) / 4.0;
    assert!((all.ece - want).abs() < 1e-12, "{}", all.ece);

    assert_eq!(s.per_tier["profile:lexical@1/vocab"].n, 1);
    assert_eq!(s.per_tier["profile:lexical@1/vocab"].top1, 1.0);
    assert_eq!(s.per_tier["profile:lexical@1/none"].n, 3);

    let l1 = &s.per_level["profile:lexical@1/1"];
    assert_eq!((l1.n, l1.top1), (4, 0.5));
    let l2 = &s.per_level["profile:lexical@1/2"];
    assert_eq!((l2.n, l2.top1), (4, 0.5));
    let l3 = &s.per_level["profile:lexical@1/3"];
    assert_eq!((l3.n, l3.top1, l3.topk), (2, 0.5, 1.0));
    assert!(!s.per_level.contains_key("profile:lexical@1/4"));
}

#[test]
fn any_acceptable_class_counts() {
    let mut o = outcome(Some("vocab"), &[&["Q", "QB"]], &["Q", "QA"], 0.8, 1);
    o.acceptable.push(path(&["Q", "QB"]));
    let s = summarize(&[o], 5);
    assert_eq!(s.per_profile["profile:lexical@1"].top1, 1.0);
}

#[test]
fn pav_pools_violators_and_fills_empty_bins() {
    // bin 2: 3/4 right, bin 5: 0/2 right (a violation), bin 8: 9/10 right
    let mut samples = Vec::new();
    samples.extend([(0.25, true), (0.25, true), (0.25, true), (0.25, false)]);
    samples.extend([(0.55, false), (0.55, false)]);
    samples.extend(std::iter::repeat_n((0.85, true), 9));
    samples.push((0.85, false));
    let c = fit_tier(&samples).unwrap();
    assert_eq!(c.support, 16);
    assert_eq!(c.bins.len(), 10);
    // smoothed: bin2 4/6, bin5 1/4 -> pooled (4/6*4 + 1/4*2)/6
    let pooled = (4.0 / 6.0 * 4.0 + 0.25 * 2.0) / 6.0;
    let top = 10.0 / 12.0;
    for (i, want) in [
        pooled, pooled, pooled, pooled, pooled, pooled, pooled, pooled, top, top,
    ]
    .iter()
    .enumerate()
    {
        assert!((c.bins[i] - want).abs() < 1e-12, "bin {i}: {}", c.bins[i]);
    }
    assert!(c.bins.windows(2).all(|w| w[0] <= w[1]), "monotone");
    assert_eq!(thresholds(&c, 0.8), Some(0.8));
    assert_eq!(thresholds(&c, 0.95), None);
    assert!(fit_tier(&[]).is_none());
}

#[test]
fn fit_keys_tiers_and_keeps_gold_identity() {
    let samples = vec![
        Sample {
            tier: "vocab".into(),
            raw: 0.9,
            correct: true,
        },
        Sample {
            tier: "rules".into(),
            raw: 0.3,
            correct: false,
        },
    ];
    let c = fit(
        SchemeId::new("lcc", "2024"),
        "s256:abc".into(),
        2,
        "2026-10-08T00:00:00Z".into(),
        &samples,
    );
    assert_eq!(c.gold_hash, "s256:abc");
    assert_eq!(c.tiers.len(), 2);
    assert!((c.apply("vocab", 0.95) - 2.0 / 3.0).abs() < 1e-5);
    assert!((c.apply("rules", 0.35) - 1.0 / 3.0).abs() < 1e-5);
    assert_eq!(
        c.apply("identifier", 0.7),
        0.7,
        "unfitted tier keeps raw scores"
    );
}
