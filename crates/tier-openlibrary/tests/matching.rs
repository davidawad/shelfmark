//! Title and author similarity, scores and the threshold.

mod common;

use common::{fixture, query};
use shelfmark_tier_openlibrary::matching::{
    author_similarity, fold, ranked, score, title_similarity,
};
use shelfmark_tier_openlibrary::{best_match, Doc, DEFAULT_THRESHOLD};

fn docs(name: &str) -> Vec<Doc> {
    let v: serde_json::Value = serde_json::from_str(&fixture(name)).unwrap();
    serde_json::from_value(v["docs"].clone()).unwrap()
}

#[test]
fn fold_strips_marks_brackets_and_punctuation() {
    assert_eq!(fold("Les Misérables"), "les miserables");
    assert_eq!(
        fold("Structure and Interpretation of Computer Programs (SICP)"),
        "structure and interpretation of computer programs"
    );
    assert_eq!(fold("Ender's Game"), "enders game");
    assert_eq!(fold("Pride & Prejudice"), "pride and prejudice");
    assert_eq!(fold("J.R.R. Tolkien"), "j r r tolkien");
}

#[test]
fn title_similarity_ignores_articles_subtitles_and_asides() {
    assert_eq!(title_similarity("The Hobbit", "Hobbit", None), 1.0);
    assert_eq!(
        title_similarity("Dune", "Dune", Some("a novel")),
        1.0,
        "the bare title still matches"
    );
    assert_eq!(
        title_similarity("Capital: a critique of political economy", "Capital", None),
        1.0,
        "the query's main title matches"
    );
    assert_eq!(
        title_similarity("The Hobbit", "The Hobbit, or There and Back Again", None),
        1.0
    );
    let partial = title_similarity("The Hobbit", "The Hobbit & The Lord of the Rings", None);
    assert!(partial < 0.5, "{partial}");
    assert_eq!(title_similarity("Dune", "Emma", None), 0.0);
}

#[test]
fn author_similarity_reads_family_names_either_way() {
    let doc = vec!["J.R.R. Tolkien".to_string()];
    assert_eq!(author_similarity(&["J. R. R. Tolkien".into()], &doc), 1.0);
    assert_eq!(author_similarity(&["Tolkien, J. R. R.".into()], &doc), 1.0);
    assert_eq!(author_similarity(&["Ursula K. Le Guin".into()], &doc), 0.0);
    let two = vec![
        "Harold Abelson".to_string(),
        "Gerald Jay Sussman".to_string(),
    ];
    assert_eq!(
        author_similarity(&["Abelson".into(), "Knuth".into()], &two),
        0.5
    );
}

#[test]
fn score_weighs_title_and_author() {
    let d = docs("synthetic.json");
    let herbert = score(&query("Dune", &["Frank Herbert"]), &d[1]);
    assert_eq!((herbert.title, herbert.author), (1.0, Some(1.0)));
    assert!((herbert.score - 1.0).abs() < 1e-12);
    // Open Library filed it under the first author, but no listed name
    // shows it: half the author credit.
    let other = score(&query("Dune", &["Frank Herbert"]), &d[0]);
    assert_eq!(other.author, Some(0.5));
    assert!((other.score - 0.875).abs() < 1e-12);
    // No author given: the title alone, discounted.
    let bare = score(&query("Dune", &[]), &d[1]);
    assert_eq!(bare.author, None);
    assert!((bare.score - 0.85).abs() < 1e-12);
}

#[test]
fn threshold_drops_weak_matches() {
    let d = docs("synthetic.json");
    let q = query("Dune", &["Frank Herbert"]);
    let r = ranked(&q, &d, DEFAULT_THRESHOLD);
    let keys: Vec<&str> = r.iter().map(|m| m.doc.key.as_str()).collect();
    assert_eq!(keys, ["/works/OL2SYNW", "/works/OL1SYNW"], "best first");
    // A sequel's title is not near enough without a name match.
    let messiah = query("Dune Messiah", &["Jane Austen"]);
    assert!(ranked(&messiah, &d, DEFAULT_THRESHOLD).is_empty());
    // With one, it is still a different title.
    assert!(ranked(
        &query("Dune Messiah", &["Frank Herbert"]),
        &d,
        DEFAULT_THRESHOLD
    )
    .is_empty());
    assert!(ranked(&query("Emma", &[]), &d, DEFAULT_THRESHOLD).is_empty());
    // A looser threshold admits the partial title.
    assert_eq!(ranked(&messiah, &d, 0.5).len(), 2);
}

#[test]
fn an_edition_title_matches_an_original_language_work() {
    let d = docs("synthetic.json");
    let m = best_match(
        &query("Critique of Pure Reason", &["Immanuel Kant"]),
        &d,
        DEFAULT_THRESHOLD,
        |_| true,
    )
    .unwrap();
    assert_eq!(m.doc.key, "/works/OL6SYNW");
    assert_eq!(m.title, 1.0);
}

#[test]
fn a_name_form_we_cannot_read_still_matches_an_exact_title() {
    let d = docs("odyssey.json");
    let m = best_match(
        &query("The Odyssey", &["Homer"]),
        &d,
        DEFAULT_THRESHOLD,
        |_| true,
    )
    .unwrap();
    assert_eq!(m.author, Some(0.5));
    assert!((m.score - 0.875).abs() < 1e-12);
    // A near title does not get through on half the author credit.
    assert!(ranked(
        &query("The Odyssey of Homer", &["Homer"]),
        &d,
        DEFAULT_THRESHOLD
    )
    .is_empty());
}

#[test]
fn best_match_skips_records_the_caller_cannot_use() {
    let d = docs("synthetic.json");
    let q = query("Night Thoughts", &["Edward Young"]);
    let first = best_match(&q, &d, DEFAULT_THRESHOLD, |_| true).unwrap();
    assert_eq!(
        first.doc.key, "/works/OL4SYNW",
        "ties keep Open Library's order"
    );
    let usable = best_match(&q, &d, DEFAULT_THRESHOLD, |doc| {
        doc.lcc.iter().any(|c| c.starts_with("PR"))
    })
    .unwrap();
    assert_eq!(usable.doc.key, "/works/OL5SYNW");
}

#[test]
fn recorded_pages_match_the_right_work() {
    let hobbit = docs("hobbit.json");
    let m = best_match(
        &query("The Hobbit", &["J. R. R. Tolkien"]),
        &hobbit,
        DEFAULT_THRESHOLD,
        |_| true,
    )
    .unwrap();
    assert_eq!(m.doc.key, "/works/OL27482W");
    let sicp = docs("sicp.json");
    let m = best_match(
        &query(
            "Structure and Interpretation of Computer Programs",
            &["Harold Abelson"],
        ),
        &sicp,
        DEFAULT_THRESHOLD,
        |d| !d.lcc.is_empty(),
    )
    .unwrap();
    assert_eq!(m.doc.key, "/works/OL3267304W");
    let kant = docs("kant.json");
    let m = best_match(
        &query("Critique of Pure Reason", &["Immanuel Kant"]),
        &kant,
        DEFAULT_THRESHOLD,
        |_| true,
    )
    .unwrap();
    assert_eq!(
        m.doc.key, "/works/OL17104789W",
        "the series aside is ignored"
    );
}
