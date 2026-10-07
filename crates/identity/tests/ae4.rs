//! AE4 shape: a format conversion that keeps the words gets the same Work
//! ID; an OCR-noised copy gets its own Work plus a link proposal; accepting
//! the proposal links the item to the existing Work.

mod common;

use shelfmark_identity::{Format, Identity, Minted, Via, PROPOSAL_THRESHOLD};

use common::{corpus, ocr_noise};

const TXT: &str = "The Art of Computer Programming\n\n\
                   Volume one covers fundamental algorithms, from basic\n\
                   concepts to information structures.\n";

const MD: &str = "---\ntitle: The Art of Computer Programming\n---\n\n\
                  # The Art of *Computer* Programming\n\n\
                  Volume **one** covers [fundamental algorithms](https://example.org/taocp),\n\
                  from basic concepts to `information structures`.\n";

const HTML: &str = "<!doctype html><html><head><title>TAOCP</title>\
                    <style>p { color: red }</style></head><body>\
                    <h1>The Art of Computer Programming</h1>\
                    <p>Volume <em>one</em> covers fundamental&nbsp;algorithms, from basic \
                    concepts to <a href=\"/is\">information structures</a>.</p></body></html>";

#[test]
fn same_words_in_txt_md_and_html_mint_one_work() {
    let txt = Minted::from_bytes(TXT.as_bytes(), Format::Text).unwrap();
    let md = Minted::from_bytes(MD.as_bytes(), Format::Markdown).unwrap();
    let html = Minted::from_bytes(HTML.as_bytes(), Format::Html).unwrap();
    assert_eq!(txt.work, md.work, "md text: {:?}", md.text);
    assert_eq!(txt.work, html.work, "html text: {:?}", html.text);
    assert_ne!(txt.item, md.item, "different bytes, different items");
    assert_ne!(md.item, html.item);
    assert!(txt.work.to_string().starts_with("work:nt1s256:"));
    assert!(txt.item.to_string().starts_with("item:s256:"));
}

#[test]
fn from_path_picks_the_format_by_extension() {
    let dir = tempfile::tempdir().unwrap();
    let txt = dir.path().join("taocp.txt");
    let md = dir.path().join("taocp.MD");
    std::fs::write(&txt, TXT).unwrap();
    std::fs::write(&md, MD).unwrap();
    let a = Minted::from_path(&txt).unwrap();
    let b = Minted::from_path(&md).unwrap();
    assert_eq!((a.format, b.format), (Format::Text, Format::Markdown));
    assert_eq!(a.work, b.work);
}

#[test]
fn converted_copy_links_to_the_existing_work_through_the_store() {
    let store = tempfile::tempdir().unwrap();
    let mut id = Identity::open(store.path()).unwrap();
    let first = id
        .ingest(&Minted::from_bytes(TXT.as_bytes(), Format::Text).unwrap())
        .unwrap();
    assert!(first.new_item && first.new_work);
    let second = id
        .ingest(&Minted::from_bytes(MD.as_bytes(), Format::Markdown).unwrap())
        .unwrap();
    assert!(second.new_item && !second.new_work, "{second:?}");
    assert_eq!(second.work, first.work);
    assert!(second.proposals.is_empty(), "same Work is never proposed");
    assert_eq!(id.items_of(&first.work), vec![&first.item, &second.item]);
}

#[test]
fn ocr_noised_copy_is_a_proposal_not_the_same_id() {
    let clean = corpus(1200, 7);
    let noisy = ocr_noise(&clean, 90);
    assert_ne!(clean, noisy);

    let store = tempfile::tempdir().unwrap();
    let mut id = Identity::open(store.path()).unwrap();
    let original = id
        .ingest(&Minted::from_bytes(clean.as_bytes(), Format::Text).unwrap())
        .unwrap();
    // An unrelated text is indexed too and must not be proposed.
    id.ingest(&Minted::from_bytes(corpus(1200, 99).as_bytes(), Format::Text).unwrap())
        .unwrap();

    let copy = Minted::from_bytes(noisy.as_bytes(), Format::Text).unwrap();
    assert_ne!(copy.work, original.work, "noise changes the Work ID");
    let preview = id.preview(&copy);
    let ingested = id.ingest(&copy).unwrap();
    assert_eq!(preview, ingested, "preview predicts ingest");

    assert_eq!(ingested.work, copy.work, "nothing is linked automatically");
    assert_eq!(ingested.proposals.len(), 1, "{:?}", ingested.proposals);
    let p = &ingested.proposals[0];
    assert_eq!(p.work, original.work);
    assert_eq!(p.from_work, copy.work);
    assert_eq!(p.item, copy.item);
    assert!(p.similarity >= PROPOSAL_THRESHOLD, "{}", p.similarity);
    assert!(p.similarity < 1.0);
    assert_eq!(p.method, "mh1-w5-p128");

    // Accepting the proposal is the CLI's link verb.
    assert!(id.link(&copy.item, &p.work).unwrap());
    assert!(!id.link(&copy.item, &p.work).unwrap(), "idempotent");
    assert_eq!(id.work_of(&copy.item), Some(&original.work));

    // Reload: the link survives, and a re-ingest reports the linked Work
    // with no proposal back to the Work it already belongs to.
    let id = Identity::open(store.path()).unwrap();
    assert_eq!(id.work_of(&copy.item), Some(&original.work));
    let again = id.preview(&copy);
    assert!(!again.new_item);
    assert_eq!(again.work, original.work);
    assert_eq!(again.minted_work, copy.work);
    assert!(again.proposals.is_empty(), "{:?}", again.proposals);
    let last = std::fs::read_to_string(store.path().join("links.jsonl")).unwrap();
    let last: shelfmark_identity::Link =
        serde_json::from_str(last.lines().last().unwrap()).unwrap();
    assert_eq!(last.via, Via::Linked);
    assert!(last.similarity.unwrap() >= PROPOSAL_THRESHOLD);
}

#[test]
fn heavily_changed_text_gets_no_proposal() {
    let clean = corpus(1200, 7);
    let store = tempfile::tempdir().unwrap();
    let mut id = Identity::open(store.path()).unwrap();
    id.ingest(&Minted::from_bytes(clean.as_bytes(), Format::Text).unwrap())
        .unwrap();
    let mangled = ocr_noise(&clean, 6);
    let out = id
        .ingest(&Minted::from_bytes(mangled.as_bytes(), Format::Text).unwrap())
        .unwrap();
    assert!(out.proposals.is_empty(), "{:?}", out.proposals);
}
