//! AE1 (R7, R13): a PDF containing an ISBN whose LC call number is in the
//! identifier table is decided by the identifier tier at full confidence,
//! and the trace names the ISBN.

use crate::common::{pdf, Bench};

#[test]
#[cfg_attr(not(feature = "pdf"), ignore = "needs --features pdf")]
fn pdf_with_an_isbn_is_decided_by_the_identifier_tier() {
    let b = Bench::new("ae1");
    let file = b.file(
        "clrs.pdf",
        pdf(&[
            "Introduction to Algorithms, third edition",
            "Cormen, Leiserson, Rivest and Stein",
            "ISBN 978-0-262-03384-8",
        ]),
    );
    let out = b.ok(&["classify", "--file", &file]);
    let p = &out["placement"];
    assert_eq!(out["ingest"]["format"], "pdf");
    assert_eq!(p["trace"]["deciding_tier"], "identifier");
    assert_eq!(p["confidence"], 1.0);
    assert_eq!(p["status"], "machine");
    // The table row says QA76.6, which files under the scheme's QA76.
    assert_eq!(p["class"], "class:mini-lcc@2024:QA76");
    let evidence = &p["trace"]["candidates"][0]["evidence"][0];
    assert_eq!(evidence["identifier"]["type"], "isbn");
    assert_eq!(evidence["identifier"]["value"], "9780262033848");

    // The full trace is what `explain` returns for the placement.
    let pid = p["id"].as_str().unwrap();
    let ex = b.ok(&["explain", pid]);
    assert_eq!(ex["explanations"][0]["placement"]["trace"], p["trace"]);
}
