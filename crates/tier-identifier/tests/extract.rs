//! Finding identifiers in text and metadata.

use shelfmark_core::tier::{ClassifyInput, Identifier};
use shelfmark_tier_identifier::extract;

fn from_text(text: &str) -> Vec<Identifier> {
    extract(&ClassifyInput {
        text: text.into(),
        ..Default::default()
    })
    .into_iter()
    .collect()
}

fn isbn(v: &str) -> Identifier {
    Identifier::Isbn(v.into())
}

#[test]
fn labelled_isbns_of_both_lengths() {
    let page = "Copyright 2009 MIT.\nISBN 978-0-262-03384-8 (hardcover : alk. paper)\n\
                ISBN-10: 0-201-63361-2 1994\nISBN (pbk.): 080442957X";
    assert_eq!(
        from_text(page),
        [
            isbn("9780201633610"),
            isbn("9780262033848"),
            isbn("9780804429573")
        ]
    );
}

#[test]
fn bare_isbn13_found_but_bare_isbn10_and_bad_checksums_ignored() {
    assert_eq!(from_text("see 9780262033848."), [isbn("9780262033848")]);
    assert!(from_text("call 0262033844 now").is_empty());
    assert!(from_text("ISBN 978-0-262-03384-9").is_empty());
    assert!(from_text("order 97802620338481234").is_empty());
}

#[test]
fn lccn_doi_and_arxiv_in_text() {
    let text = "Library of Congress Control Number: 2001-000002\n\
                https://lccn.loc.gov/n78890351\n\
                doi:10.1145/359576.359579.\n\
                arXiv:1706.03762v5 [cs.CL] 6 Dec 2017; also arXiv:hep-th/9711200\n\
                and 1706.03762 alone is not enough";
    assert_eq!(
        from_text(text),
        [
            Identifier::Lccn("2001000002".into()),
            Identifier::Lccn("n78890351".into()),
            Identifier::Doi("10.1145/359576.359579".into()),
            Identifier::Arxiv("1706.03762".into()),
            Identifier::Arxiv("hep-th/9711200".into()),
        ]
    );
}

#[test]
fn url_gives_domain_and_any_ids_in_it_but_text_links_give_no_domain() {
    let input = ClassifyInput {
        url: Some("https://arxiv.org/abs/1706.03762v5".into()),
        text: "see https://www.law.cornell.edu/".into(),
        ..Default::default()
    };
    let got: Vec<Identifier> = extract(&input).into_iter().collect();
    assert_eq!(
        got,
        [
            Identifier::Arxiv("1706.03762".into()),
            Identifier::UrlDomain("arxiv.org".into()),
        ]
    );
}

#[test]
fn supplied_identifiers_are_normalized_and_invalid_ones_dropped() {
    let input = ClassifyInput {
        identifiers: vec![
            isbn("0-262-03384-4"),
            isbn("0-262-03384-5"),
            Identifier::Doi("https://doi.org/10.1145/359576.359579".into()),
            Identifier::Lccn("n78-89035".into()),
        ],
        ..Default::default()
    };
    let got: Vec<Identifier> = extract(&input).into_iter().collect();
    assert_eq!(
        got,
        [
            isbn("9780262033848"),
            Identifier::Lccn("n78089035".into()),
            Identifier::Doi("10.1145/359576.359579".into()),
        ]
    );
}

#[test]
fn duplicates_collapse() {
    let text = "ISBN 0-262-03384-4 ... ISBN 978-0-262-03384-8 ... 9780262033848";
    assert_eq!(from_text(text), [isbn("9780262033848")]);
}
