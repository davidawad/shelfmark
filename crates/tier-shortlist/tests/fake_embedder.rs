//! The tier end to end with a deterministic fake embedder (no model).

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use shelfmark_core::error::{Error, ErrorCode};
use shelfmark_core::scheme::{ClassDef, CompiledIndex, SchemeDoc};
use shelfmark_core::testkit::MemIndex;
use shelfmark_core::tier::{Candidate, ClassifyInput, Evidence, Tier};
use shelfmark_tier_shortlist::{Embedder, LabelIndex, Params, ShortlistTier};

const DIM: usize = 512;

/// Hashed bag of words: every lowercase alphanumeric word adds 1 to one of
/// `DIM` buckets (FNV-1a), then the vector is normalized. Counts its calls.
#[derive(Default)]
struct BagOfWords {
    label_calls: AtomicUsize,
    item_calls: AtomicUsize,
}

impl BagOfWords {
    fn vector(text: &str) -> Vec<f32> {
        let mut v = vec![0f32; DIM];
        for w in text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
        {
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for b in w.to_lowercase().bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
            v[(h % DIM as u64) as usize] += 1.0;
        }
        shelfmark_tier_shortlist::normalize(&mut v);
        v
    }
}

impl Embedder for BagOfWords {
    fn model_id(&self) -> &str {
        "fake-bow"
    }
    fn label_prefix(&self) -> &str {
        "q: "
    }
    fn embed_labels(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Error> {
        self.label_calls.fetch_add(1, Ordering::SeqCst);
        Ok(texts.iter().map(|t| Self::vector(t)).collect())
    }
    fn embed_items(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Error> {
        self.item_calls.fetch_add(1, Ordering::SeqCst);
        Ok(texts.iter().map(|t| Self::vector(t)).collect())
    }
}

/// An embedder whose model never loads.
struct Broken;

impl Embedder for Broken {
    fn model_id(&self) -> &str {
        "broken"
    }
    fn embed_labels(&self, _: &[String]) -> Result<Vec<Vec<f32>>, Error> {
        Err(Error::new(ErrorCode::TierUnavailable, "no model").next("fix it"))
    }
    fn embed_items(&self, t: &[String]) -> Result<Vec<Vec<f32>>, Error> {
        self.embed_labels(t)
    }
}

fn mini_lcc() -> MemIndex {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/schemes/mini-lcc.json"
    );
    let doc: SchemeDoc = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    MemIndex::new(&doc)
}

fn class(code: &str, label: &str, parent: Option<&str>) -> ClassDef {
    ClassDef {
        code: code.into(),
        label: label.into(),
        parent: parent.map(Into::into),
        range: None,
        vocab: Vec::new(),
        note: None,
    }
}

fn doc(classes: Vec<ClassDef>) -> SchemeDoc {
    SchemeDoc {
        schema: None,
        name: "toy".into(),
        version: "1".into(),
        title: "Toy scheme".into(),
        source: None,
        code_grammar: "[A-Z]{1,3}[0-9.]*".into(),
        classes,
    }
}

/// Five top-level classes with two subclasses each.
fn five_by_two(history_label: &str) -> SchemeDoc {
    doc(vec![
        class("B", "Philosophy religion", None),
        class("BL", "Religions mythology", Some("B")),
        class("BR", "Christianity church", Some("B")),
        class("E", "History of the Americas", None),
        class("E1", "United States civil war", Some("E")),
        class("E2", "Colonial america", Some("E")),
        class("M", "Music", None),
        class("ML", "Literature on music", Some("M")),
        class("MT", "Musical instruction", Some("M")),
        class("Q", "Science", None),
        class("QA", "Mathematics computing", Some("Q")),
        class("QC", "Physics", Some("Q")),
        class("T", "Technology", None),
        class("TK", "Electrical engineering", Some("T")),
        class("TL", history_label, Some("T")),
    ])
}

fn text(t: &str) -> ClassifyInput {
    ClassifyInput {
        text: t.into(),
        ..ClassifyInput::default()
    }
}

fn codes(c: &[Candidate]) -> Vec<&str> {
    c.iter().map(|c| c.class.code.as_str()).collect()
}

fn tmp_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "shelfmark-shortlist-test-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn tier(e: Arc<dyn Embedder>, cache: Option<PathBuf>) -> ShortlistTier {
    ShortlistTier::new(e, cache, Params::default())
}

fn assert_well_formed(c: &[Candidate]) {
    assert!(!c.is_empty());
    for w in c.windows(2) {
        assert!(
            w[0].score > w[1].score
                || (w[0].score == w[1].score && w[0].class.code <= w[1].class.code),
            "not ranked: {c:?}"
        );
    }
    for x in c {
        assert!(
            x.score.is_finite() && (0.0..=1.0).contains(&x.score),
            "{x:?}"
        );
        assert_eq!(
            x.evidence,
            vec![Evidence::Similarity {
                model: "fake-bow".into()
            }]
        );
    }
    assert!(c.iter().map(|x| x.score).sum::<f64>() <= 1.0 + 1e-9);
}

#[test]
fn mini_lcc_coarse_to_fine_picks_the_matching_subclass() {
    let index = mini_lcc();
    let t = tier(Arc::new(BagOfWords::default()), None);
    let out = t
        .run(
            &text("Mathematics and computer software for science"),
            &index,
            &[],
        )
        .unwrap();
    assert_eq!(out.candidates[0].class, index.class("QA"));
    assert!(out.candidates[0].score > 0.5, "{:?}", out.candidates);
    // H and K have no subclasses, so they stand for themselves.
    assert_eq!(codes(&out.candidates).len(), 3);
    assert_well_formed(&out.candidates);
    assert_eq!(out.cost.model_calls, 1);
}

#[test]
fn coarse_stage_keeps_three_classes_and_ranks_their_subclasses() {
    let index = MemIndex::new(&five_by_two("Motor vehicles aeronautics"));
    let t = tier(Arc::new(BagOfWords::default()), None);
    let out = t
        .run(&text("The civil war in the United States"), &index, &[])
        .unwrap();
    assert_eq!(out.candidates[0].class.code, "E1");
    // Subclasses of exactly three top-level classes: six candidates.
    assert_eq!(out.candidates.len(), 6);
    assert_well_formed(&out.candidates);
    let parents: std::collections::BTreeSet<String> = out
        .candidates
        .iter()
        .map(|c| index.get(&c.class.code).unwrap().parent.unwrap().code)
        .collect();
    assert_eq!(parents.len(), 3);
    assert!(parents.contains("E"));
}

#[test]
fn emits_at_most_n_and_level_zero_when_there_are_no_subclasses() {
    let flat = doc((0..15)
        .map(|i| class(&format!("A{i}"), &format!("topic{i} shared"), None))
        .collect());
    let index = MemIndex::new(&flat);
    let t = tier(Arc::new(BagOfWords::default()), None);
    let out = t.run(&text("topic7 shared"), &index, &[]).unwrap();
    assert_eq!(out.candidates.len(), 10);
    assert_eq!(out.candidates[0].class.code, "A7");
    assert_well_formed(&out.candidates);
    // Ties (every other class scores the same) break by code bytes.
    assert_eq!(codes(&out.candidates[1..3]), vec!["A0", "A1"]);
    assert_eq!(out.candidates[1].score, out.candidates[2].score);
}

#[test]
fn deterministic_across_runs_and_instances() {
    let index = MemIndex::new(&five_by_two("Motor vehicles aeronautics"));
    let input = text("Electrical engineering and physics of motors");
    let a = tier(Arc::new(BagOfWords::default()), None);
    let b = tier(Arc::new(BagOfWords::default()), None);
    let first = a.run(&input, &index, &[]).unwrap().candidates;
    assert_eq!(first, a.run(&input, &index, &[]).unwrap().candidates);
    assert_eq!(first, b.run(&input, &index, &[]).unwrap().candidates);
}

#[test]
fn labels_are_memoized_in_memory_and_on_disk() {
    let dir = tmp_dir("roundtrip");
    let index = MemIndex::new(&five_by_two("Motor vehicles aeronautics"));
    let input = text("Religions and mythology");

    let e1 = Arc::new(BagOfWords::default());
    let t1 = tier(e1.clone(), Some(dir.clone()));
    let first = t1.run(&input, &index, &[]).unwrap();
    t1.run(&input, &index, &[]).unwrap();
    assert_eq!(
        e1.label_calls.load(Ordering::SeqCst),
        1,
        "memoized in memory"
    );
    assert!(!t1.label_index(&index).unwrap().from_disk());
    let files: Vec<_> = std::fs::read_dir(dir.join("shortlist/fake-bow"))
        .unwrap()
        .map(|f| f.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(files.len(), 1, "{files:?}");
    assert!(
        files[0].ends_with(".f32") && files[0].len() == 68,
        "{files:?}"
    );

    let e2 = Arc::new(BagOfWords::default());
    let t2 = tier(e2.clone(), Some(dir.clone()));
    let second = t2.run(&input, &index, &[]).unwrap();
    assert_eq!(e2.label_calls.load(Ordering::SeqCst), 0, "loaded from disk");
    assert!(t2.label_index(&index).unwrap().from_disk());
    assert_eq!(first.candidates, second.candidates);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn disk_cache_invalidates_when_a_label_changes() {
    let dir = tmp_dir("invalidate");
    let input = text("Aeronautics");
    let e1 = Arc::new(BagOfWords::default());
    tier(e1.clone(), Some(dir.clone()))
        .run(
            &input,
            &MemIndex::new(&five_by_two("Motor vehicles aeronautics")),
            &[],
        )
        .unwrap();

    let changed = MemIndex::new(&five_by_two("Motor vehicles aeronautics astronautics"));
    let e2 = Arc::new(BagOfWords::default());
    let t2 = tier(e2.clone(), Some(dir.clone()));
    t2.run(&input, &changed, &[]).unwrap();
    assert_eq!(e2.label_calls.load(Ordering::SeqCst), 1, "rebuilt");
    assert!(!t2.label_index(&changed).unwrap().from_disk());
    let n = std::fs::read_dir(dir.join("shortlist/fake-bow"))
        .unwrap()
        .count();
    assert_eq!(n, 2);

    // A corrupt file is a miss, rebuilt and rewritten.
    for f in std::fs::read_dir(dir.join("shortlist/fake-bow")).unwrap() {
        std::fs::write(f.unwrap().path(), b"garbage").unwrap();
    }
    let e3 = Arc::new(BagOfWords::default());
    tier(e3.clone(), Some(dir.clone()))
        .run(&input, &changed, &[])
        .unwrap();
    assert_eq!(e3.label_calls.load(Ordering::SeqCst), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn in_memory_memo_follows_the_index() {
    let e = Arc::new(BagOfWords::default());
    let t = tier(e.clone(), None);
    let a = MemIndex::new(&five_by_two("Motor vehicles aeronautics"));
    let b = mini_lcc();
    t.run(&text("music"), &a, &[]).unwrap();
    t.run(&text("music"), &b, &[]).unwrap();
    t.run(&text("music"), &b, &[]).unwrap();
    assert_eq!(e.label_calls.load(Ordering::SeqCst), 2);
}

#[test]
fn empty_input_gives_no_candidates_and_no_model_calls() {
    let e = Arc::new(BagOfWords::default());
    let t = tier(e.clone(), None);
    let input = ClassifyInput {
        title: Some("  ".into()),
        text: "\n".into(),
        ..ClassifyInput::default()
    };
    let out = t.run(&input, &mini_lcc(), &[]).unwrap();
    assert!(out.candidates.is_empty());
    assert_eq!(out.cost.model_calls, 0);
    assert_eq!(e.label_calls.load(Ordering::SeqCst), 0);
    assert_eq!(e.item_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn title_alone_is_embedded() {
    let t = tier(Arc::new(BagOfWords::default()), None);
    let input = ClassifyInput {
        title: Some("Mathematics computer software".into()),
        ..ClassifyInput::default()
    };
    let out = t.run(&input, &mini_lcc(), &[]).unwrap();
    assert_eq!(out.candidates[0].class.code, "QA");
}

#[test]
fn info_declares_cost_and_versions_model_algorithm_and_params() {
    let t = tier(Arc::new(BagOfWords::default()), None);
    let info = t.info();
    assert_eq!(info.name, "shortlist");
    assert_eq!(info.version, "fake-bow/c2f-1");
    assert!(info.needs_model);
    assert_eq!(info.expected_memory_mb, 600);
    assert_eq!(info.expected_micros_per_item, 50_000);
    let tuned = ShortlistTier::new(
        Arc::new(BagOfWords::default()),
        None,
        Params {
            k_classes: 5,
            ..Params::default()
        },
    );
    assert_ne!(tuned.info().version, info.version);
}

#[test]
fn embedder_failure_is_returned_as_is() {
    let t = tier(Arc::new(Broken), None);
    let err = t.run(&text("anything"), &mini_lcc(), &[]).unwrap_err();
    assert_eq!(err.code, ErrorCode::TierUnavailable);
    assert_eq!(err.next_command.as_deref(), Some("fix it"));
}

#[test]
fn label_index_orders_top_level_first_with_parent_links() {
    let index = MemIndex::new(&five_by_two("Motor vehicles aeronautics"));
    let labels = LabelIndex::labels_of(&index);
    assert_eq!(labels.len(), 15);
    assert!(labels[..5]
        .iter()
        .all(|l| l.level == 0 && l.parent.is_none()));
    let tl = labels.iter().find(|l| l.code == "TL").unwrap();
    assert_eq!(labels[tl.parent.unwrap()].code, "T");
    assert_eq!(tl.text, "Technology — Motor vehicles aeronautics");
    assert_eq!(
        labels[0].text,
        "Philosophy religion. Includes: Religions mythology; Christianity church"
    );
    let built = LabelIndex::build(&index, &BagOfWords::default(), None).unwrap();
    assert_eq!(built.dim(), DIM);
    let norm: f32 = built.vector(3).iter().map(|x| x * x).sum();
    assert!((norm - 1.0).abs() < 1e-5);
    assert!(built.is_for(&index, "fake-bow"));
    assert!(!built.is_for(&index, "other"));
}
