//! Hit rate of the `openlibrary` profile against `lexical` on a pinned
//! book list (`crates/tier-openlibrary/eval/books-50.json`).
//!
//! ```sh
//! cargo run -p shelfmark-cli --features openlibrary --example openlibrary_measure -- \
//!     crates/tier-openlibrary/eval/books-50.json /tmp/ol-store
//! ```
//!
//! Each book is classified twice from its title and authors alone, into
//! the bundled LCC outline. A hit is a placement in the class the scheme
//! gives the expected LC number; a subclass hit only needs the same
//! letters (PR, QA). Searches go to openlibrary.org, one a second, and
//! are cached in the store, so a re-run sends none. Prints one JSON line
//! per book, then the summary.

use std::path::PathBuf;

use serde_json::{json, Value};
use shelfmark_core::scheme::CodeCheck;
use shelfmark_core::{Envelope, Request, Service, Verb};

const PROFILES: [&str; 2] = ["profile:lexical@2", "profile:openlibrary@2"];

fn letters(code: &str) -> &str {
    let end = code
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(code.len());
    &code[..end]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(list), Some(store)) = (args.next(), args.next()) else {
        eprintln!("usage: openlibrary_measure BOOKS.json STORE_DIR");
        std::process::exit(2);
    };
    let store = PathBuf::from(store);
    std::fs::create_dir_all(&store).expect("store directory");
    let books: Value =
        serde_json::from_str(&std::fs::read_to_string(&list).expect("book list")).expect("json");
    let books = books["books"].as_array().expect("books");
    let service = shelfmark_cli::wiring::open(&store);
    let index = shelfmark_cli::wiring::bundled_lcc();

    let mut hits = [0u32; 2];
    let mut subclass_hits = [0u32; 2];
    let mut ol_matched = 0u32;
    let mut ol_decided = 0u32;
    for b in books {
        let expected_lc = b["expected_lc"].as_str().expect("expected_lc");
        let expected = match index.check_code(expected_lc) {
            CodeCheck::Exact { class } | CodeCheck::Within { class } => class.code,
            other => panic!("{expected_lc} is not in the outline: {other:?}"),
        };
        let mut row =
            json!({ "title": b["title"], "expected_lc": expected_lc, "expected": expected });
        for (i, profile) in PROFILES.iter().enumerate() {
            let req = Request::new(
                Verb::Classify,
                json!({
                    "title": b["title"], "authors": b["authors"],
                    "profile": profile, "scheme": "scheme:lcc@2024",
                }),
            );
            let data = match service.call(req) {
                Envelope::Ok { data, .. } => data,
                Envelope::Err { error, .. } => panic!("{}: {error:?}", b["title"]),
            };
            let p = &data["placement"];
            let class = p["class"].as_str().unwrap_or_default();
            let code = class.rsplit(':').next().unwrap_or_default().to_string();
            let hit = code == expected;
            let sub = letters(&code) == letters(expected_lc);
            hits[i] += u32::from(hit);
            subclass_hits[i] += u32::from(sub);
            if i == 1 {
                let matched = p["trace"]["candidates"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|c| {
                        c["per_tier"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .any(|t| t["tier"] == "openlibrary")
                    });
                ol_matched += u32::from(matched);
                ol_decided += u32::from(p["trace"]["deciding_tier"] == "openlibrary");
                row["openlibrary_matched"] = json!(matched);
            }
            row[profile.trim_start_matches("profile:")] = json!({
                "class": code, "hit": hit, "subclass_hit": sub,
                "status": p["status"], "confidence": p["confidence"],
            });
        }
        println!("{row}");
    }
    let n = books.len() as f64;
    let rate = |h: u32| (f64::from(h) / n * 1000.0).round() / 1000.0;
    let summary = json!({
        "books": books.len(),
        "lexical": { "hits": hits[0], "hit_rate": rate(hits[0]),
                     "subclass_hits": subclass_hits[0], "subclass_hit_rate": rate(subclass_hits[0]) },
        "openlibrary": { "hits": hits[1], "hit_rate": rate(hits[1]),
                         "subclass_hits": subclass_hits[1], "subclass_hit_rate": rate(subclass_hits[1]),
                         "matched": ol_matched, "decided": ol_decided },
    });
    println!("{}", serde_json::to_string_pretty(&summary).expect("json"));
}
