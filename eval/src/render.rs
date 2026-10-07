//! Plain-text rendering of a report for a person at a terminal. Agents get
//! the JSON report (the default whenever stdout is not a TTY).

use crate::report::Report;
use crate::summary::{Stat, Summary};
use std::fmt::Write;

fn pct(s: &Stat) -> String {
    if !s.value.is_finite() {
        return "-".into();
    }
    match s.ci {
        Some([lo, hi]) => format!(
            "{:.1} [{:.1},{:.1}]",
            s.value * 100.0,
            lo * 100.0,
            hi * 100.0
        ),
        None => format!("{:.1}", s.value * 100.0),
    }
}

fn num(s: &Stat) -> String {
    if !s.value.is_finite() {
        return "-".into();
    }
    match s.ci {
        Some([lo, hi]) => format!("{:.3} [{:.3},{:.3}]", s.value, lo, hi),
        None => format!("{:.3}", s.value),
    }
}

fn summary(out: &mut String, s: &Summary, indent: &str) {
    let _ = writeln!(out, "{indent}n={} missing={}", s.n, s.missing);
    let ks: Vec<&String> = s
        .levels
        .first()
        .map(|l| l.topk.keys().collect())
        .unwrap_or_default();
    let mut head = format!(
        "{indent}{:<3} {:<9} {:>4}  {:<20}",
        "lvl", "name", "n", "top-1 %"
    );
    for k in &ks {
        let _ = write!(head, " {:<20}", format!("top-{k} %"));
    }
    let _ = writeln!(out, "{head} {:<22} brier", "ece");
    for l in &s.levels {
        let mut row = format!(
            "{indent}{:<3} {:<9} {:>4}  {:<20}",
            l.level,
            l.name,
            l.n,
            pct(&l.top1)
        );
        for k in &ks {
            let _ = write!(row, " {:<20}", pct(&l.topk[*k]));
        }
        let _ = writeln!(out, "{row} {:<22} {}", num(&l.ece), num(&l.brier));
    }
    let h = &s.hierarchical;
    let _ = writeln!(
        out,
        "{indent}hierarchical: accuracy {}  precision {}  f1 {}",
        pct(&h["accuracy"]),
        pct(&h["precision"]),
        pct(&h["f1"])
    );
    let _ = writeln!(
        out,
        "{indent}level {}: aurc {}  flagged {}%  acc flagged {}%  unflagged {}%",
        s.calibration_level,
        num(&s.aurc),
        pct(&s.flagged_rate),
        pct(&s.accuracy_flagged),
        pct(&s.accuracy_unflagged)
    );
    for t in &s.thresholds {
        let thr = t
            .threshold
            .map_or("unreachable".to_string(), |v| format!(">= {v:.3}"));
        let _ = writeln!(
            out,
            "{indent}  precision >= {:.2}: confidence {thr} (coverage {:.1}%)",
            t.target_precision,
            t.coverage * 100.0
        );
    }
    let l = &s.latency_ms;
    if l["mean"].value.is_finite() {
        let _ = writeln!(
            out,
            "{indent}latency ms: mean {}  p50 {}  p95 {}",
            num(&l["mean"]),
            num(&l["p50"]),
            num(&l["p95"])
        );
    }
    if !s.fast.is_empty() {
        let _ = writeln!(
            out,
            "{indent}FAST: precision {}  recall {}  f1 {}",
            pct(&s.fast["precision"]),
            pct(&s.fast["recall"]),
            pct(&s.fast["f1"])
        );
    }
}

pub fn text(r: &Report) -> String {
    let mut out = String::new();
    let o = &r.options;
    let _ = writeln!(
        out,
        "gold {} ({} items, {} main classes)  bootstrap {}x seed {} at {:.0}%",
        r.gold.path,
        r.gold.items,
        r.gold.main_classes,
        o.bootstrap,
        o.seed,
        o.confidence * 100.0
    );
    for w in &r.warnings {
        let _ = writeln!(out, "warning: {w}");
    }
    for p in &r.profiles {
        let tag = if p.experimental {
            " (experimental comparison)"
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "\nprofile {}{tag}: {} predicted",
            p.profile, p.predicted
        );
        summary(&mut out, &p.summary, "  ");
        for t in &p.tiers {
            let _ = writeln!(
                out,
                "\n  tier {} ({:.1}% of predictions)",
                t.tier,
                t.share * 100.0
            );
            summary(&mut out, &t.summary, "    ");
        }
    }
    out
}
