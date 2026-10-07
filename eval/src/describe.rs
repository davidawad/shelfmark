//! `mc-eval describe`: everything an agent needs to drive the harness
//! without reading other documentation (R20).

use serde_json::{json, Value};

pub fn describe() -> Value {
    json!({
        "tool": "mc-eval",
        "version": env!("CARGO_PKG_VERSION"),
        "purpose": "Score materials-classifier predictions against a labelled gold set: top-1/top-k per LCC hierarchy level, hierarchical accuracy, calibration (ECE, Brier), risk-coverage with measured confidence thresholds, flagged rate and latency, per profile and per deciding tier, with seeded bootstrap intervals.",
        "nouns": {
            "gold item": "one labelled item: {id, title, text, identifiers, acceptable: [LCC code or range], fast?: [heading], source: provenance}; schema schemas/gold-item.schema.json",
            "prediction": "one classifier result: {item, profile, tier, candidates: [{class, score}], confidence?, flagged, latency_ms?, fast?}; schema schemas/prediction.schema.json",
            "report": "mc-eval/report@1 JSON; one summary per profile and per tier within it",
            "manifest": "pinned source records the starter gold set is built from (gold/manifest.json)"
        },
        "levels": ["1 class (Q)", "2 subclass (QA)", "3 caption (QA76, or a range such as QB630-638.8)", "4 number (QA76.73)", "5 cutter (QA76.73.C15)"],
        "verbs": {
            "describe": "this document",
            "run": "mc-eval run --gold FILE --predictions FILE... [--k 3,5] [--level 2] [--bootstrap 1000] [--seed N] [--confidence 0.95] [--bins 10] [--precision 0.8,0.9,0.95] [--missing as-wrong|skip] [--format auto|json|text] [--out FILE]",
            "validate": "mc-eval validate --gold FILE [--predictions FILE...]",
            "gold build": "mc-eval gold build --manifest gold/manifest.json [--out FILE]   (offline, from gold/raw snapshots; download them first with gold/fetch.sh)",
            "gold stats": "mc-eval gold stats --gold FILE"
        },
        "semantics": {
            "correct at level L": "some acceptable class reaches L and the top candidate agrees with it at every level 1..L; items whose gold does not reach L are outside that level's denominator",
            "top-k": "any of the first k candidates is correct at L",
            "hierarchical": "accuracy = mean matched depth / gold depth (hierarchical recall); precision = matched depth / predicted depth; f1 their harmonic mean",
            "calibration": "top-1 confidence against correctness at the level; ECE over equal-width bins, top-label Brier score",
            "risk-coverage": "items ranked by confidence at --level; aurc = mean risk over all coverages; thresholds = lowest confidence whose accepted set meets each --precision target",
            "missing": "gold items with no prediction for a profile are scored wrong at confidence 0 (as-wrong) or left out (skip)",
            "experimental": "profiles named experimental:<name> (e.g. experimental:laya-decision) are reported as comparisons, not product tiers",
            "bootstrap": "percentile intervals over items resampled with replacement; same inputs and --seed give the same intervals"
        },
        "output": "JSON when stdout is not a terminal (or --format json), text otherwise",
        "errors": {
            "shape": {"error": {"code": "E_...", "message": "plain explanation", "next": "command that fixes it"}},
            "exit_codes": {"0": "ok", "2": "usage", "3": "invalid input data", "4": "I/O"},
            "codes": ["E_USAGE", "E_IO", "E_WRITE", "E_GOLD_PARSE", "E_GOLD_EMPTY", "E_GOLD_CODE", "E_GOLD_PATH", "E_GOLD_DUP", "E_PRED_PARSE", "E_PRED_DUP", "E_PRED_CODE", "E_PRED_SCORE", "E_PRED_CONFIDENCE", "E_PRED_LATENCY", "E_MANIFEST", "E_CROSSWALK", "E_CROSSWALK_CODE", "E_CROSSWALK_GAP", "E_RAW_FORMAT"]
        },
        "example": {
            "call": "mc-eval run --gold gold/starter-v1.ndjson --predictions fixtures/predictions-baseline.ndjson --format json",
            "prediction_line": {"item": "gold:arxiv:2401.03802", "profile": "lexical@0.1", "tier": "vocabulary", "candidates": [{"class": "QB981", "score": 0.82}, {"class": "QC174", "score": 0.11}], "flagged": false, "latency_ms": 3.1}
        }
    })
}
