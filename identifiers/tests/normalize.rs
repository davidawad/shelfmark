use identifier_tables::normalize::{ddc, isbn13, lcc, lccn, oclc};

#[test]
fn isbn10_and_isbn13_share_a_key() {
    assert_eq!(isbn13("0-521-41382-6"), Some(9780521413824));
    assert_eq!(isbn13("978-0-521-41382-4"), Some(9780521413824));
    assert_eq!(isbn13("080442957X"), Some(9780804429573));
    assert_eq!(isbn13("080442957x"), Some(9780804429573));
}

#[test]
fn isbn_rejects_bad_checksums_and_shapes() {
    assert_eq!(isbn13("0521413827"), None);
    assert_eq!(isbn13("9780521413825"), None);
    assert_eq!(isbn13("0003100043045"), None); // 13 digits, no 978/979 prefix
    assert_eq!(isbn13("$7.50"), None);
    assert_eq!(isbn13("B00BZQIO5Q"), None); // ASIN
    assert_eq!(isbn13("12345X7890"), None); // X only allowed as check digit
}

#[test]
fn lccn_follows_loc_normalization() {
    assert_eq!(lccn("96038953").as_deref(), Some("96038953"));
    assert_eq!(lccn("n78-890351").as_deref(), Some("n78890351"));
    assert_eq!(lccn("n78-89035").as_deref(), Some("n78089035"));
    assert_eq!(lccn("  2001-1114 ").as_deref(), Some("2001001114"));
    assert_eq!(lccn("85-2 /AC/r932").as_deref(), Some("85000002"));
    assert_eq!(lccn("SH 85026371").as_deref(), Some("sh85026371"));
    assert_eq!(lccn("978-1-840-22076-6"), None);
    assert_eq!(lccn("123"), None);
}

#[test]
fn oclc_strips_prefixes_and_zeros() {
    assert_eq!(oclc("(OCoLC)00012345"), Some(12345));
    assert_eq!(oclc("ocm35638157"), Some(35638157));
    assert_eq!(oclc("ocn503266847"), Some(503266847));
    assert_eq!(oclc("on1109389989"), Some(1109389989));
    assert_eq!(oclc("0"), None);
    assert_eq!(oclc("12a"), None);
}

#[test]
fn lcc_keeps_full_number_and_class_prefix() {
    let c = lcc("GE145 .A53 1997").unwrap();
    assert_eq!(
        (c.full.as_str(), c.prefix.as_str(), c.top.as_str()),
        ("GE145.A53 1997", "GE145", "GE")
    );
    assert_eq!(lcc("GE145.A53 1997").unwrap().full, "GE145.A53 1997");
    let c = lcc("QA76.76.C65 S5 1990").unwrap();
    assert_eq!((c.prefix.as_str(), c.top.as_str()), ("QA76.76", "QA"));
    assert_eq!(lcc("KF 4550").unwrap().prefix, "KF4550");
    assert_eq!(lcc("PN1993 .N38 Bd. 2").unwrap().prefix, "PN1993");
    assert_eq!(lcc("DS").unwrap().prefix, "DS");
    assert_eq!(lcc("DJK76.2").unwrap().top, "DJK");
    assert_eq!(lcc("KFN5225").unwrap().top, "KFN");
    assert_eq!(lcc("K230").unwrap().top, "K");
}

#[test]
fn lcc_rejects_local_and_non_lc_marks() {
    for raw in [
        "",
        "MLCS 92/01234",
        "#M1042",
        "MFM Orien China 230",
        "LAW <Gen Ireland 1999>",
        "W 100",
        "IN PROCESS",
        "qa76",
        "LAW",
        "LAW 1954.M3 no. 110",
        "MLC 12345",
        "CPB Box no. 123",
        "ESL 12",
        "KX 100",
    ] {
        assert_eq!(lcc(raw), None, "{raw:?}");
    }
}

#[test]
fn ddc_drops_prime_marks_and_edition_numbers() {
    let c = ddc("636.7/53").unwrap();
    assert_eq!(
        (c.full.as_str(), c.prefix.as_str(), c.top.as_str()),
        ("636.753", "636", "6")
    );
    assert_eq!(ddc("813/.54 20").unwrap().full, "813.54");
    assert_eq!(ddc("362/0924").unwrap().full, "362.0924");
    assert_eq!(ddc("330").unwrap().full, "330");
    assert_eq!(ddc("[641.5945]").unwrap().full, "641.5945");
}

#[test]
fn ddc_rejects_non_numeric_marks() {
    for raw in [
        "[E]", "B", "Fic", "-Fic-", "23/.92", "3301.31", "59", "(556.4)",
    ] {
        assert_eq!(ddc(raw), None, "{raw:?}");
    }
}
