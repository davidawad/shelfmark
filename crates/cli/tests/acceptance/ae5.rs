//! AE5 (R12, R28): when a reviewer corrects a Work's class and its file moves,
//! a citation through the Work ID still resolves and the machine claim is
//! superseded, not deleted.
//!
//! This is the shelfmark half. The kb half (`kd` evidence carrying the
//! `work:` ID, the file actually moving in a kb tree) runs in the kb core
//! resource's own acceptance tests, which drive this binary.

use crate::common::Bench;

#[test]
fn correction_supersedes_the_machine_claim_and_the_work_id_still_resolves() {
    let b = Bench::new("ae5");
    let placed = b.ok(&["classify", "--text", "Computer software", "--target", "0.5"]);
    let machine = placed["placement"]["id"].as_str().unwrap().to_string();
    let work = placed["placement"]["work"].as_str().unwrap().to_string();
    assert_eq!(
        placed["placement"]["class"],
        "class:lcc@2024:QA76.75-76.765"
    );
    let before = b.ok(&["name", &work, "--title", "Engineering a Compiler"]);

    let corrected = b.commit(&["correct", &machine, "--class", "QA", "--author", "reviewer"]);
    assert!(corrected.get("revision").is_some(), "{corrected}");

    // The citation is the Work ID: it resolves to the confirmed class.
    let r = b.ok(&["resolve", &work]);
    let current = &r["resolved"]["placements"][0];
    assert_eq!(current["class"], "class:lcc@2024:QA");
    assert_eq!(current["status"], "confirmed");
    assert_eq!(current["supersedes"], machine.as_str());

    // The machine claim is still in the log, now superseded.
    let old = b.ok(&["explain", &machine]);
    assert_eq!(old["explanations"][0]["status"], "superseded");
    assert_eq!(
        old["explanations"][0]["placement"]["class"],
        "class:lcc@2024:QA76.75-76.765"
    );

    // Its file name moves with the class.
    let after = b.ok(&["name", &work, "--title", "Engineering a Compiler"]);
    assert_ne!(before["path"], after["path"]);
    assert!(after["path"]
        .as_str()
        .unwrap()
        .starts_with("Q Science/QA Mathematics/"));
}
