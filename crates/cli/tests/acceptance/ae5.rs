//! AE5 (R12, R28): when David corrects a Work's class and its file moves,
//! a citation through the Work ID still resolves and the machine claim is
//! superseded, not deleted.
//!
//! The shelfmark half runs now. The kb half (`kd` evidence carrying the
//! `work:` ID, the file actually moving) needs the kb kind (U12).

use crate::common::Bench;

#[test]
fn correction_supersedes_the_machine_claim_and_the_work_id_still_resolves() {
    let b = Bench::new("ae5");
    let placed = b.ok(&[
        "classify",
        "--text",
        "Compilers (Computer programs)",
        "--target",
        "0.5",
    ]);
    let machine = placed["placement"]["id"].as_str().unwrap().to_string();
    let work = placed["placement"]["work"].as_str().unwrap().to_string();
    assert_eq!(placed["placement"]["class"], "class:mini-lcc@2024:QA76");
    let before = b.ok(&["name", &work, "--title", "Engineering a Compiler"]);

    let corrected = b.commit(&["correct", &machine, "--class", "QA", "--author", "david"]);
    assert!(corrected.get("revision").is_some(), "{corrected}");

    // The citation is the Work ID: it resolves to the confirmed class.
    let r = b.ok(&["resolve", &work]);
    let current = &r["resolved"]["placements"][0];
    assert_eq!(current["class"], "class:mini-lcc@2024:QA");
    assert_eq!(current["status"], "confirmed");
    assert_eq!(current["supersedes"], machine.as_str());

    // The machine claim is still in the log, now superseded.
    let old = b.ok(&["explain", &machine]);
    assert_eq!(old["explanations"][0]["status"], "superseded");
    assert_eq!(
        old["explanations"][0]["placement"]["class"],
        "class:mini-lcc@2024:QA76"
    );

    // Its file name moves with the class.
    let after = b.ok(&["name", &work, "--title", "Engineering a Compiler"]);
    assert_ne!(before["path"], after["path"]);
    assert!(after["path"]
        .as_str()
        .unwrap()
        .starts_with("Q Science/QA Mathematics/"));
}

#[test]
#[ignore = "pending U12 (kb core resource, dot-materials-classifier-v1-gis4b.5): kd evidence with work: and kb move"]
fn kd_citation_resolves_after_the_kb_moves_the_file() {
    panic!(
        "pending: file a Work through `resource kb file`, cite it from kd evidence \
         carrying work:, correct the class, commit `resource kb move`, then check \
         `resource kd list --cites work:<id>` and the new kb path"
    );
}
