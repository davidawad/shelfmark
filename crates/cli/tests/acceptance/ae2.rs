//! AE2 (R10, R11): a note whose vocabulary matches H and K with no clear
//! margin gets the top-ranked class, status flagged, and both candidates
//! in its trace.

use crate::common::{candidate_codes, Bench};

#[test]
fn no_clear_margin_flags_the_top_class_with_its_candidates() {
    let b = Bench::new("ae2");
    let out = b.ok(&[
        "classify",
        "--text",
        "Notes on law and the social sciences",
        "--profile",
        "profile:lexical@1",
    ]);
    let p = &out["placement"];
    assert_eq!(p["status"], "flagged");
    let mut codes = candidate_codes(p);
    let top = codes[0].clone();
    assert_eq!(p["class"], format!("class:lcc@2024:{top}"));
    codes.sort();
    assert_eq!(codes, ["H", "K"]);
    assert!(p["trace"]["deciding_tier"].is_null());
    let pid = p["id"].as_str().unwrap();
    assert_eq!(out["next_command"], format!("shelfmark explain {pid}"));

    let flagged = b.ok(&["list-flagged"]);
    assert_eq!(flagged["total"], 1);
    assert_eq!(flagged["flagged"][0]["placement"], p["id"]);
}
