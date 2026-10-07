//! Rendering a work to a preview path against a compiled index.

use shelfmark_core::id::{SchemeId, WorkId};
use shelfmark_core::testkit::MemIndex;
use shelfmark_core::ErrorCode;
use shelfmark_naming::{
    ClassPathStyle, CutterSource, Naming, NamingOptions, Work, DEFAULT_TEMPLATE,
};

fn work(index: &MemIndex) -> Work {
    let mut w = Work::new(index.class("QA76"), "The Art of Computer Programming");
    w.author = Some("Knuth, Donald E.".into());
    w.content_type = Some("book".into());
    w.year = Some(1968);
    w.ext = Some(".pdf".into());
    w
}

fn naming(template: &str) -> Naming {
    Naming::new(template, NamingOptions::default()).unwrap()
}

#[test]
fn bead_template_renders_class_path_from_ancestors() {
    let index = MemIndex::sample();
    let path = naming("{class_path}/{cutter} {title} [{type}].{ext}")
        .preview(&index, &work(&index))
        .unwrap();
    assert_eq!(
        path.to_string(),
        "QA Mathematics/QA76 Computer software/K58 The Art of Computer Programming [book].pdf"
    );
    assert_eq!(path.segments.len(), 3);
    assert_eq!(path.cutter.as_ref().unwrap().lc(), ".K58");
    let json = serde_json::to_value(&path).unwrap();
    assert_eq!(json["cutter"], "K58");
}

#[test]
fn same_input_same_path() {
    let index = MemIndex::sample();
    let n = naming(DEFAULT_TEMPLATE);
    let w = work(&index);
    let first = n.preview(&index, &w).unwrap();
    for _ in 0..5 {
        let again = naming(DEFAULT_TEMPLATE)
            .preview(&MemIndex::sample(), &w.clone())
            .unwrap();
        assert_eq!(again, first);
    }
}

#[test]
fn optional_group_drops_without_a_value() {
    let index = MemIndex::sample();
    let mut w = work(&index);
    w.content_type = None;
    let path = naming(DEFAULT_TEMPLATE).preview(&index, &w).unwrap();
    assert_eq!(
        path.segments.last().unwrap(),
        "K58 The Art of Computer Programming.pdf"
    );
}

#[test]
fn missing_required_variable_is_an_error_with_a_hint() {
    let index = MemIndex::sample();
    let mut w = work(&index);
    w.content_type = Some("  ".into());
    let e = naming("{title} [{type}].{ext}")
        .preview(&index, &w)
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidRequest);
    assert_eq!(e.details.unwrap()["variable"], "type");
    assert!(e.next_command.unwrap().contains("<[{type}]>"));
}

#[test]
fn cutter_falls_back_to_title_without_its_article() {
    let index = MemIndex::sample();
    let mut w = work(&index);
    w.author = None;
    let n = naming("{cutter}");
    // "Art of ..." -> A + r(7) + t(8).
    assert_eq!(n.preview(&index, &w).unwrap().to_string(), "A78");
    w.author = Some("Donald Knuth".into());
    assert_eq!(n.preview(&index, &w).unwrap().to_string(), "K58");
    let by_title = Naming::new(
        "{cutter}",
        NamingOptions {
            cutter_from: CutterSource::Title,
            ..NamingOptions::default()
        },
    )
    .unwrap();
    assert_eq!(by_title.preview(&index, &w).unwrap().to_string(), "A78");
}

#[test]
fn values_cannot_inject_folders_or_reserved_names() {
    let index = MemIndex::sample();
    let mut w = Work::new(index.class("K"), "../../etc/passwd: CON?");
    w.ext = Some("txt".into());
    let path = naming("{class_path}/{title}.{ext}")
        .preview(&index, &w)
        .unwrap();
    assert_eq!(path.segments, ["K Law", "___.._etc_passwd_ CON_.txt"]);
    let mut w = Work::new(index.class("K"), "con");
    w.ext = Some("txt".into());
    let path = naming("{title}.{ext}").preview(&index, &w).unwrap();
    assert_eq!(path.to_string(), "con_.txt");
}

#[test]
fn class_path_style_and_other_variables() {
    let index = MemIndex::sample();
    let mut w = work(&index);
    w.work = Some(WorkId::from_text("taocp"));
    let n = Naming::new(
        "{class_path}/{class} {class_label}/{year} {author} - {work}",
        NamingOptions {
            class_path: ClassPathStyle::Code,
            ..NamingOptions::default()
        },
    )
    .unwrap();
    let path = n.preview(&index, &w).unwrap();
    assert_eq!(path.segments[..3], ["QA", "QA76", "QA76 Computer software"]);
    let last = &path.segments[3];
    assert!(last.starts_with("1968 Knuth, Donald E. - work_"), "{last}");
    assert!(!last.contains(':'));
}

#[test]
fn long_titles_are_capped_keeping_the_extension() {
    let index = MemIndex::sample();
    let mut w = work(&index);
    w.title = "Word ".repeat(100);
    let n = Naming::new(
        DEFAULT_TEMPLATE,
        NamingOptions {
            max_segment_bytes: 64,
            ..NamingOptions::default()
        },
    )
    .unwrap();
    let path = n.preview(&index, &w).unwrap();
    let name = path.segments.last().unwrap();
    assert!(name.len() <= 64, "{name}");
    assert!(name.ends_with(".pdf"), "{name}");
}

#[test]
fn unknown_class_and_wrong_scheme_are_errors() {
    let index = MemIndex::sample();
    let w = Work::new(index.class("ZZ9"), "x");
    let e = naming(DEFAULT_TEMPLATE).preview(&index, &w).unwrap_err();
    assert_eq!(e.code, ErrorCode::NotFound);
    assert!(e.next_command.is_some());

    let other = shelfmark_core::id::ClassId::new(SchemeId::new("lcc", "2024"), "QA76");
    let e = naming(DEFAULT_TEMPLATE)
        .preview(&index, &Work::new(other, "x"))
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidRequest);
}

#[test]
fn preview_path_joins_for_the_platform() {
    let index = MemIndex::sample();
    let path = naming(DEFAULT_TEMPLATE)
        .preview(&index, &work(&index))
        .unwrap();
    let buf = path.to_path_buf();
    assert_eq!(buf.components().count(), 3);
    assert!(buf.is_relative());
}
