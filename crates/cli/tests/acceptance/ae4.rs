//! AE4 (R3): a markdown conversion of an already-placed PDF links to the
//! existing Work instead of minting a new one; a near-duplicate (changed
//! words) is proposed for linking, and accepting the proposal links it.

use crate::common::Bench;

/// About 250 words of running text, deterministic.
fn essay() -> Vec<String> {
    let words = [
        "classification",
        "places",
        "every",
        "work",
        "under",
        "one",
        "class",
        "so",
        "that",
        "related",
        "material",
        "sits",
        "together",
        "on",
        "the",
        "shelf",
        "and",
        "readers",
        "browsing",
        "nearby",
        "find",
        "what",
        "they",
        "need",
        "computer",
        "software",
    ];
    (0..10)
        .map(|n| {
            let mut line: Vec<&str> = words.iter().cycle().skip(n * 3).take(25).copied().collect();
            line.push(["alpha", "beta", "gamma", "delta", "epsilon"][n % 5]);
            line.join(" ")
        })
        .collect()
}

#[test]
#[cfg_attr(not(feature = "pdf"), ignore = "needs --features pdf")]
fn markdown_conversion_of_a_placed_pdf_links_to_its_work() {
    let b = Bench::new("ae4");
    let lines = essay();
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let pdf = b.file("essay.pdf", crate::common::pdf(&refs));
    let placed = b.ok(&["classify", "--file", &pdf]);
    assert_eq!(placed["ingest"]["new_work"], true);
    let work = placed["placement"]["work"].clone();

    let md = format!("# {}\n\n{}\n", lines[0], lines[1..].join("\n\n"));
    let md = b.file("essay.md", md);
    let converted = b.ok(&["classify", "--file", &md]);
    assert_eq!(converted["ingest"]["format"], "markdown");
    assert_eq!(converted["ingest"]["new_item"], true);
    assert_eq!(converted["ingest"]["new_work"], false, "{converted}");
    assert_eq!(converted["placement"]["work"], work);
    // Same Work, same inputs: the placement is reused, not re-run.
    assert_eq!(converted["memo_hit"], true);
    assert_eq!(converted["placement"]["id"], placed["placement"]["id"]);

    let resolved = b.ok(&["resolve", work.as_str().unwrap()]);
    assert_eq!(
        resolved["resolved"]["items"],
        serde_json::json!([placed["item"], converted["item"]])
    );
}

#[test]
fn near_duplicate_copy_is_proposed_and_linked_to_the_existing_work() {
    let b = Bench::new("ae4-near");
    let lines = essay();
    let original = b.file("essay.txt", lines.join("\n"));
    let placed = b.ok(&["classify", "--file", &original]);
    let work = placed["placement"]["work"].as_str().unwrap().to_string();

    // An OCR-style copy: one word misread.
    let noisy = lines.join("\n").replacen("readers", "rcaders", 1);
    let copy = b.file("essay-ocr.md", noisy);
    let out = b.ok(&["classify", "--file", &copy]);
    assert_eq!(out["ingest"]["new_work"], true);
    let proposals = out["ingest"]["proposals"].as_array().unwrap();
    assert_eq!(proposals.len(), 1, "{out}");
    assert_eq!(proposals[0]["work"], work.as_str());
    let item = out["item"].as_str().unwrap();
    assert_eq!(
        out["next_command"],
        format!("shelfmark link --item {item} --work {work}")
    );

    b.commit(&["link", "--item", item, "--work", &work]);
    let r = b.ok(&["resolve", item]);
    assert_eq!(r["resolved"]["work"], work.as_str());
}
