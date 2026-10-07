//! Typed IDs (R1, R2, R4, R20): grammar, round-trips, rejections.

use shelfmark_core::id::{normalize_text_v1, IdError};
use shelfmark_core::{AnyId, ClassId, ItemId, Kind, PlacementId, ProfileId, SchemeId, WorkId};

const HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn every_kind_round_trips_through_any_id() {
    let ids = [
        format!("work:nt1s256:{HEX}"),
        format!("item:s256:{HEX}"),
        "scheme:lcc@2024".to_string(),
        "class:lcc@2024:QA76.76.C65".to_string(),
        "profile:lexical@1".to_string(),
        format!("placement:s256:{HEX}"),
    ];
    for (id, kind) in ids.iter().zip(Kind::ALL) {
        let parsed: AnyId = id.parse().unwrap();
        assert_eq!(parsed.kind(), kind, "{id}");
        assert_eq!(parsed.to_string(), *id);
        let json = serde_json::to_string(&parsed).unwrap();
        assert_eq!(serde_json::from_str::<AnyId>(&json).unwrap(), parsed);
    }
}

#[test]
fn work_id_is_stable_across_format_noise() {
    let pdf = "Compilers:  Principles, Techniques,\nand Tools";
    let md = "# compilers principles techniques and tools\n";
    assert_eq!(WorkId::from_text(pdf), WorkId::from_text(md));
    assert_ne!(WorkId::from_text(pdf), WorkId::from_text("Compilers"));
    assert_eq!(normalize_text_v1("  Ünïcode—DASH  x "), "ünïcode dash x");
    assert!(WorkId::from_text("x")
        .to_string()
        .starts_with("work:nt1s256:"));
}

#[test]
fn item_id_hashes_bytes() {
    let id = ItemId::from_bytes(b"abc");
    assert_eq!(
        id.to_string(),
        "item:s256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn class_ids_keep_colons_in_codes_and_reject_whitespace() {
    let c: ClassId = "class:x@1:A:B".parse().unwrap();
    assert_eq!(c.code, "A:B");
    assert_eq!(c.scheme, SchemeId::new("x", "1"));
    assert!("class:x@1:A B".parse::<ClassId>().is_err());
    assert!("class:x@1:".parse::<ClassId>().is_err());
    assert!("class:x:QA".parse::<ClassId>().is_err());
}

#[test]
fn rejections_name_the_problem() {
    assert!(matches!(
        "lcc@2024".parse::<AnyId>(),
        Err(IdError::NoKind(_))
    ));
    assert!(matches!(
        "shelf:x".parse::<AnyId>(),
        Err(IdError::UnknownKind(_))
    ));
    assert!(matches!(
        "scheme:lcc".parse::<SchemeId>(),
        Err(IdError::Malformed { .. })
    ));
    assert!(matches!(
        "profile:lexical@1".parse::<SchemeId>(),
        Err(IdError::Malformed { .. })
    ));
    assert!(
        format!("work:s256:{HEX}").parse::<WorkId>().is_err(),
        "wrong algorithm"
    );
    assert!(
        "work:nt1s256:abc".parse::<WorkId>().is_err(),
        "short digest"
    );
    let upper = HEX.replace('0', "A");
    assert!(format!("placement:s256:{upper}")
        .parse::<PlacementId>()
        .is_err());
    assert!("profile:a b@1".parse::<ProfileId>().is_err());
}

#[test]
fn nfkc_makes_decomposed_accents_and_ligatures_match() {
    let nfc = "R\u{e9}sum\u{e9} of the \u{fb01}eld";
    let nfd = "Re\u{301}sume\u{301} of the field";
    assert_eq!(normalize_text_v1(nfc), "résumé of the field");
    assert_eq!(WorkId::from_text(nfc), WorkId::from_text(nfd));
}
