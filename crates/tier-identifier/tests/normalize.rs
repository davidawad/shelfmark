//! Identifier validation and canonical forms (R7).

use shelfmark_core::tier::Identifier;
use shelfmark_tier_identifier::normalize::{
    self, arxiv, doi, domain, isbn, isbn10_to_13, isbn10_valid, isbn13_valid, lccn, KeyError,
};

#[test]
fn isbn_checksums_accept_valid_and_reject_invalid() {
    for ok in ["0262033844", "0201633612", "080442957X", "043942089X"] {
        assert!(isbn10_valid(ok), "{ok}");
    }
    for bad in [
        "0262033845",
        "0201633613",
        "X262033844",
        "02620338",
        "026203384A",
    ] {
        assert!(!isbn10_valid(bad), "{bad}");
    }
    for ok in ["9780262033848", "9780306406157", "9791090636071"] {
        assert!(isbn13_valid(ok), "{ok}");
    }
    // Bad check digit; good checksum but not a 978/979 prefix.
    for bad in ["9780262033849", "9770262033840", "978026203384"] {
        assert!(!isbn13_valid(bad), "{bad}");
    }
}

#[test]
fn isbn10_normalizes_to_13() {
    assert_eq!(isbn10_to_13("0262033844"), "9780262033848");
    assert_eq!(isbn10_to_13("0201633612"), "9780201633610");
    assert_eq!(isbn10_to_13("080442957X"), "9780804429573");
    assert_eq!(isbn("ISBN 0-262-03384-4").as_deref(), Some("9780262033848"));
    assert_eq!(
        isbn("ISBN-13: 978-0-262-03384-8").as_deref(),
        Some("9780262033848")
    );
    assert_eq!(
        isbn("isbn-10: 0 262 03384 4").as_deref(),
        Some("9780262033848")
    );
    assert_eq!(isbn("0-262-03384-5"), None);
}

#[test]
fn lccn_normalizes_per_loc_rules() {
    // Vectors from https://www.loc.gov/marc/lccn-namespace.html
    for (raw, want) in [
        ("n78-890351", "n78890351"),
        ("n78-89035", "n78089035"),
        ("n 78890351 ", "n78890351"),
        (" 85000002 ", "85000002"),
        ("85-2 ", "85000002"),
        ("2001-000002", "2001000002"),
        ("75-425165//r75", "75425165"),
        (" 79139101 /AC/r932", "79139101"),
        ("LCCN: sn 85-000002", "sn85000002"),
        ("N78-890351", "n78890351"),
    ] {
        assert_eq!(lccn(raw).as_deref(), Some(want), "{raw:?}");
    }
    for bad in ["78-1234567", "abcd12345678", "85-", "123", "85-12a"] {
        assert_eq!(lccn(bad), None, "{bad:?}");
    }
}

#[test]
fn doi_is_lowercased_and_stripped() {
    for (raw, want) in [
        ("10.1145/359576.359579", "10.1145/359576.359579"),
        (
            "https://doi.org/10.1145/359576.359579",
            "10.1145/359576.359579",
        ),
        ("doi: 10.1000/ABC.def.", "10.1000/abc.def"),
        (
            "10.1002/(SICI)1097-4571(199806)49:8",
            "10.1002/(sici)1097-4571(199806)49:8",
        ),
        ("10.1000/xyz)", "10.1000/xyz"),
    ] {
        assert_eq!(doi(raw).as_deref(), Some(want), "{raw:?}");
    }
    for bad in ["10.12/abc", "11.1145/1", "10.1145/"] {
        assert_eq!(doi(bad), None, "{bad:?}");
    }
}

#[test]
fn arxiv_new_and_old_style() {
    for (raw, want) in [
        ("1706.03762", "1706.03762"),
        ("arXiv:1706.03762v5", "1706.03762"),
        ("https://arxiv.org/abs/2101.00001v2", "2101.00001"),
        ("https://arxiv.org/pdf/0704.0001.pdf", "0704.0001"),
        ("1412.9999", "1412.9999"),
        ("hep-th/9711200", "hep-th/9711200"),
        ("arXiv:math.GT/0309136v1", "math/0309136"),
        ("cond-mat/0703001", "cond-mat/0703001"),
    ] {
        assert_eq!(arxiv(raw).as_deref(), Some(want), "{raw:?}");
    }
    for bad in [
        "0703.0001",      // new style began 2007-04
        "1501.0001",      // 2015 on has five digits
        "1412.00001",     // before 2015 has four
        "1713.03762",     // month 13
        "hep-th/0704001", // old style ended 2007-03
        "math/9013001",   // month 13
        "hep-th/971120",
    ] {
        assert_eq!(arxiv(bad), None, "{bad:?}");
    }
}

#[test]
fn domain_strips_scheme_www_port_and_rejects_ips() {
    for (raw, want) in [
        ("https://www.law.cornell.edu/uscode/text", "law.cornell.edu"),
        ("http://user@Example.COM:8080/a?b#c", "example.com"),
        ("example.org.", "example.org"),
    ] {
        assert_eq!(domain(raw).as_deref(), Some(want), "{raw:?}");
    }
    for bad in ["localhost", "http://127.0.0.1/x", "https://-bad.com", ""] {
        assert_eq!(domain(bad), None, "{bad:?}");
    }
    assert_eq!(
        normalize::domain_suffixes("cs.example.ac.uk"),
        ["cs.example.ac.uk", "example.ac.uk", "ac.uk"]
    );
}

#[test]
fn table_keys_parse_to_canonical_identifiers() {
    assert_eq!(
        normalize::parse_key("isbn:0-262-03384-4"),
        Ok(Identifier::Isbn("9780262033848".into()))
    );
    assert_eq!(
        normalize::parse_key("domain:www.Example.com"),
        Ok(Identifier::UrlDomain("example.com".into()))
    );
    assert!(matches!(
        normalize::parse_key("issn:1234-5678"),
        Err(KeyError::UnknownType(_))
    ));
    assert!(matches!(
        normalize::parse_key("isbn:9780262033849"),
        Err(KeyError::InvalidValue(_))
    ));
    assert!(normalize::parse_key("9780262033848").is_err());
}
