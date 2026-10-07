//! The link file: append-only lines in `links.jsonl`, reloaded into the
//! same state, written under `<store>/.lock`.

mod common;

use shelfmark_core::ErrorCode;
use shelfmark_identity::{Format, Identity, Link, Minted, Via, LINKS_FILE, SIGNATURES_FILE};

fn mint(text: &str) -> Minted {
    Minted::from_bytes(text.as_bytes(), Format::Text).unwrap()
}

#[test]
fn open_on_a_missing_directory_writes_nothing() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("store");
    let id = Identity::open(&dir).unwrap();
    assert_eq!(id.revision(), "s256:empty");
    let _ = id.preview(&mint("hello world"));
    assert!(!dir.exists());
}

#[test]
fn append_and_reload() {
    let store = tempfile::tempdir().unwrap();
    let (a, b) = (mint(&common::corpus(300, 1)), mint(&common::corpus(300, 2)));
    let mut id = Identity::open(store.path()).unwrap();
    let r0 = id.revision().to_string();
    id.ingest(&a).unwrap();
    let r1 = id.revision().to_string();
    id.ingest(&b).unwrap();
    let r2 = id.revision().to_string();
    assert!(r0 != r1 && r1 != r2, "revision changes on every append");

    // Re-ingesting a known item appends nothing.
    let again = id.ingest(&a).unwrap();
    assert!(!again.new_item && !again.new_work);
    assert_eq!(id.revision(), r2);

    assert!(id.link(&b.item, &a.work).unwrap());
    let r3 = id.revision().to_string();

    let text = std::fs::read_to_string(store.path().join(LINKS_FILE)).unwrap();
    let lines: Vec<Link> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(
        (lines[0].via, lines[1].via, lines[2].via),
        (Via::Minted, Via::Minted, Via::Linked)
    );
    assert_eq!(lines[2].item, b.item);
    assert_eq!(lines[2].work, a.work);
    let sigs = std::fs::read_to_string(store.path().join(SIGNATURES_FILE)).unwrap();
    assert_eq!(sigs.lines().count(), 2, "one signature per Work");
    assert!(store.path().join(".lock").exists());

    let reloaded = Identity::open(store.path()).unwrap();
    assert_eq!(reloaded.revision(), r3, "reload rebuilds the same revision");
    assert_eq!(reloaded.work_of(&a.item), Some(&a.work));
    assert_eq!(reloaded.work_of(&b.item), Some(&a.work));
    assert_eq!(reloaded.items_of(&a.work), vec![&a.item, &b.item]);
    assert!(reloaded.items_of(&b.work).is_empty());
    assert_eq!(reloaded.index().len(), 2);
}

#[test]
fn two_handles_see_each_others_appends() {
    let store = tempfile::tempdir().unwrap();
    let (a, b) = (mint(&common::corpus(300, 1)), mint(&common::corpus(300, 2)));
    let mut first = Identity::open(store.path()).unwrap();
    let mut second = Identity::open(store.path()).unwrap();
    first.ingest(&a).unwrap();
    second.ingest(&b).unwrap();
    assert!(
        second.link(&b.item, &a.work).unwrap(),
        "second caught up on a"
    );
    let text = std::fs::read_to_string(store.path().join(LINKS_FILE)).unwrap();
    assert_eq!(text.lines().count(), 3);
    assert_eq!(
        Identity::open(store.path()).unwrap().revision(),
        second.revision()
    );
}

#[test]
fn link_needs_a_known_item_and_work() {
    let store = tempfile::tempdir().unwrap();
    let (a, b) = (mint("first text here"), mint("second text here"));
    let mut id = Identity::open(store.path()).unwrap();
    id.ingest(&a).unwrap();
    let err = id.link(&b.item, &a.work).unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);
    let err = id.link(&a.item, &b.work).unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);
    assert!(!id.link(&a.item, &a.work).unwrap(), "already there");
}

#[test]
fn torn_last_line_is_skipped_on_load_and_blocks_append() {
    let store = tempfile::tempdir().unwrap();
    let (a, b) = (mint("first text here"), mint("second text here"));
    let mut id = Identity::open(store.path()).unwrap();
    id.ingest(&a).unwrap();
    let path = store.path().join(LINKS_FILE);
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str("{\"item\":\"item:s2");
    std::fs::write(&path, text).unwrap();

    let mut id = Identity::open(store.path()).unwrap();
    assert_eq!(id.work_of(&a.item), Some(&a.work));
    let err = id.ingest(&b).unwrap_err();
    assert_eq!(err.code, ErrorCode::StoreError);
    assert!(err.message.contains("incomplete line"), "{}", err.message);
}

#[test]
fn corrupt_line_is_a_store_error_naming_the_line() {
    let store = tempfile::tempdir().unwrap();
    std::fs::write(store.path().join(LINKS_FILE), "{}\n").unwrap();
    let err = Identity::open(store.path()).unwrap_err();
    assert_eq!(err.code, ErrorCode::StoreError);
    assert!(err.message.contains("line 1"), "{}", err.message);
}

#[test]
fn foreign_signature_scheme_is_refused() {
    let store = tempfile::tempdir().unwrap();
    let mut id = Identity::open(store.path()).unwrap();
    id.ingest(&mint("some words to sign")).unwrap();
    let path = store.path().join(SIGNATURES_FILE);
    let text = std::fs::read_to_string(&path)
        .unwrap()
        .replace("mh1-w5-p128", "mh9");
    std::fs::write(&path, text).unwrap();
    let err = Identity::open(store.path()).unwrap_err();
    assert_eq!(err.code, ErrorCode::StoreError);
    assert!(err.message.contains("mh9"), "{}", err.message);
}
