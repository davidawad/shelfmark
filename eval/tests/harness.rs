//! End-to-end metric checks on a four-item fixture whose expected values are
//! worked out by hand in the comments.

use mc_eval::gold::GoldSet;
use mc_eval::pred;
use mc_eval::report::{run, MissingPolicy, Options, ProfileReport, Report};
use mc_eval::summary::Config;

const GOLD: &str = include_str!("data/gold-small.ndjson");
const PRED: &str = include_str!("data/pred-small.ndjson");

fn opts(missing: MissingPolicy, bootstrap: usize) -> Options {
    Options {
        metrics: Config {
            ks: vec![3],
            primary_level: 2,
            bins: 10,
            target_precisions: vec![0.9, 0.99],
        },
        bootstrap,
        seed: 7,
        confidence: 0.95,
        missing,
    }
}

fn report(missing: MissingPolicy, bootstrap: usize) -> Report {
    let gold = GoldSet::parse(GOLD).unwrap();
    let loaded = pred::parse(PRED, &gold).unwrap();
    run(
        &gold,
        "gold-small",
        "x".into(),
        &loaded,
        &opts(missing, bootstrap),
    )
}

fn profile<'a>(r: &'a Report, name: &str) -> &'a ProfileReport {
    r.profiles.iter().find(|p| p.profile == name).unwrap()
}

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn per_level_accuracy_counts_missing_as_wrong() {
    let r = report(MissingPolicy::AsWrong, 0);
    let p = profile(&r, "p");
    assert_eq!((p.predicted, p.summary.n, p.summary.missing), (3, 4, 1));
    let top1: Vec<f64> = p.summary.levels.iter().map(|l| l.top1.value).collect();
    let n: Vec<usize> = p.summary.levels.iter().map(|l| l.n).collect();
    // g1 right to depth 5; g2 right only at class Q; g3 right to depth 3; g4 missing.
    assert_eq!(n, vec![4, 4, 4, 2, 2]);
    for (got, want) in top1.iter().zip([0.75, 0.5, 0.5, 0.5, 0.5]) {
        close(*got, want);
    }
    // g2's second candidate QB981 sits inside QB980-QB991.
    close(p.summary.levels[2].topk["3"].value, 0.75);
}

#[test]
fn hierarchical_calibration_and_risk_coverage() {
    let r = report(MissingPolicy::AsWrong, 0);
    let s = &profile(&r, "p").summary;
    // Recall per item: 1, 1/3, 1, 0. Precision: 1, 1/3 (QC174 depth 3), 1, 0.
    close(
        s.hierarchical["accuracy"].value,
        (1.0 + 1.0 / 3.0 + 1.0) / 4.0,
    );
    close(
        s.hierarchical["precision"].value,
        (1.0 + 1.0 / 3.0 + 1.0) / 4.0,
    );
    // Level 2 pairs: (.9 right) (.6 wrong) (.8 right) (0 wrong).
    let l2 = &s.levels[1];
    close(l2.brier.value, (0.01 + 0.36 + 0.04 + 0.0) / 4.0);
    close(l2.ece.value, (0.1 + 0.6 + 0.2 + 0.0) / 4.0);
    // Ranked .9 .8 .6 0 -> risks 0, 0, 1/3, 1/2.
    close(s.aurc.value, (1.0 / 3.0 + 0.5) / 4.0);
    let t90 = &s.thresholds[0];
    assert_eq!(t90.threshold, Some(0.8));
    close(t90.coverage, 0.5);
    close(s.thresholds[1].coverage, 0.5);
    assert_eq!(s.reliability.len(), 4);
    close(s.flagged_rate.value, 0.25);
    close(s.accuracy_flagged.value, 0.0);
    close(s.accuracy_unflagged.value, 2.0 / 3.0);
}

#[test]
fn latency_fast_tiers_and_warnings() {
    let r = report(MissingPolicy::AsWrong, 0);
    let p = profile(&r, "p");
    let l = &p.summary.latency_ms;
    close(l["mean"].value, 3.0);
    close(l["p50"].value, 3.0);
    close(l["p95"].value, 5.0);
    close(p.summary.fast["precision"].value, 0.5);
    close(p.summary.fast["recall"].value, 1.0);
    let tiers: Vec<(&str, f64)> = p.tiers.iter().map(|t| (t.tier.as_str(), t.share)).collect();
    assert_eq!(tiers.len(), 2);
    assert_eq!(tiers[0].0, "identifier");
    close(tiers[0].1, 1.0 / 3.0);
    close(p.tiers[1].summary.levels[2].top1.value, 0.5);
    assert!(r
        .warnings
        .iter()
        .any(|w| w.contains("1 prediction(s) name items not in the gold set")));
    let exp = profile(&r, "experimental:laya-decision");
    assert!(exp.experimental && !p.experimental);
}

#[test]
fn skip_policy_leaves_out_unpredicted_items() {
    let r = report(MissingPolicy::Skip, 0);
    let s = &profile(&r, "p").summary;
    assert_eq!((s.n, s.missing), (3, 0));
    close(s.levels[0].top1.value, 1.0);
    close(s.levels[3].top1.value, 1.0);
}

#[test]
fn bootstrap_is_seeded_and_brackets_the_estimate() {
    let a = serde_json::to_string(&report(MissingPolicy::AsWrong, 300)).unwrap();
    let b = serde_json::to_string(&report(MissingPolicy::AsWrong, 300)).unwrap();
    assert_eq!(a, b);
    let r = report(MissingPolicy::AsWrong, 300);
    for l in &profile(&r, "p").summary.levels {
        let [lo, hi] = l.top1.ci.expect("interval present");
        assert!(
            lo <= l.top1.value && l.top1.value <= hi,
            "{lo} {} {hi}",
            l.top1.value
        );
        assert!((0.0..=1.0).contains(&lo) && (0.0..=1.0).contains(&hi));
    }
}

#[test]
fn rejects_bad_predictions_with_codes() {
    let gold = GoldSet::parse(GOLD).unwrap();
    let cases = [
        ("{\"item\":\"g1\"}\n{\"item\":\"g1\"}", "E_PRED_DUP"),
        (
            "{\"item\":\"g1\",\"candidates\":[{\"class\":\"I99\"}]}",
            "E_PRED_CODE",
        ),
        ("{\"item\":\"g1\",\"confidence\":1.5}", "E_PRED_CONFIDENCE"),
        ("{\"item\":\"g1\",\"latency_ms\":-1}", "E_PRED_LATENCY"),
        ("not json", "E_PRED_PARSE"),
    ];
    for (text, code) in cases {
        let e = pred::parse(text, &gold).unwrap_err();
        assert_eq!(e.code, code, "{text}");
        assert!(!e.next.is_empty());
    }
}

#[test]
fn rejects_bad_gold_with_codes() {
    let cases = [
        ("", "E_GOLD_EMPTY"),
        ("{\"id\":\"a\",\"title\":\"t\",\"acceptable\":[]}", "E_GOLD_EMPTY"),
        ("{\"id\":\"a\",\"title\":\"t\",\"acceptable\":[\"Q\"]}\n{\"id\":\"a\",\"title\":\"t\",\"acceptable\":[\"Q\"]}", "E_GOLD_DUP"),
        ("{\"id\":\"a\",\"title\":\"t\",\"acceptable\":[\"W1\"]}", "E_GOLD_CODE"),
        ("{\"id\":\"a\",\"title\":\"t\",\"acceptable\":[{\"code\":\"QA76\",\"path\":[\"Q\"]}]}", "E_GOLD_PATH"),
        ("{\"id\":\"a\"}", "E_GOLD_PARSE"),
    ];
    for (text, code) in cases {
        assert_eq!(GoldSet::parse(text).unwrap_err().code, code, "{text}");
    }
}
