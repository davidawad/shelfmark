//! Cutter vectors from the examples in LC CSM G 63, sec. 2.

use shelfmark_naming::cutter;

fn c(key: &str) -> String {
    cutter(key, 2).expect("cutter").to_string()
}

/// G 63 examples printed without an asterisk: the table alone gives them.
#[test]
fn table_examples_match_exactly() {
    let vectors = [
        // Initial vowels.
        ("IBM", "I26"),
        ("Idaho", "I33"),
        ("Inman", "I56"),
        ("Ipswich", "I67"),
        // Initial S.
        ("Sadron", "S23"),
        ("Schreiber", "S37"),
        ("Stinson", "S75"),
        ("Suryani", "S87"),
        // Initial Qu, and Qa-Qt.
        ("Queiroz", "Q45"),
        ("Quinn", "Q56"),
        ("Quorum", "Q67"),
        ("Qutub", "Q88"),
        ("Qadduri", "Q23"),
        ("Quade", "Q33"),
        // Other initial consonants.
        ("Campbell", "C36"),
        ("Ceccaldi", "C43"),
        ("Cobblestone", "C63"),
        ("Cryer", "C79"),
        ("Cuellar", "C84"),
        ("Cymbal", "C96"),
        // Asterisked in G 63, but the table gives the printed value.
        ("Symposium", "S96"),
    ];
    for (key, want) in vectors {
        assert_eq!(c(key), want, "{key}");
    }
}

/// G 63 examples with asterisks were adjusted by hand to fit a shelflist.
/// The table gives their first digit; the second comes from the table too,
/// so it may differ from the printed value.
#[test]
fn shelflist_adjusted_examples_share_the_first_digit() {
    let vectors = [
        ("Ilardo", "I4", "I43"),
        ("Import", "I48", "I47"),
        ("Ito", "I87", "I86"),
        ("Ivy", "I94", "I99"),
        ("Scanlon", "S29", "S23"),
        ("Shillingburg", "S53", "S55"),
        ("Singer", "S57", "S56"),
        ("Qvortrup", "Q97", "Q96"),
        ("Chertok", "C48", "C44"),
        ("Clark", "C58", "C53"),
    ];
    for (key, printed, ours) in vectors {
        let got = c(key);
        assert_eq!(got, ours, "{key}");
        assert_eq!(got[..2], printed[..2], "{key}: first digit");
    }
}

#[test]
fn lc_form_has_leading_dot_display_does_not() {
    let cut = cutter("Campbell", 2).unwrap();
    assert_eq!(cut.lc(), ".C36");
    assert_eq!(cut.to_string(), "C36");
    assert_eq!(serde_json::to_value(&cut).unwrap(), "C36");
}

#[test]
fn digits_are_configurable_and_extend_by_expansion() {
    assert_eq!(cutter("Campbell", 1).unwrap().to_string(), "C3");
    assert_eq!(cutter("Campbell", 3).unwrap().to_string(), "C367");
    assert_eq!(cutter("Campbell", 4).unwrap().to_string(), "C3673");
    // Short keys stop early.
    assert_eq!(cutter("Ca", 4).unwrap().to_string(), "C3");
}

#[test]
fn never_ends_in_zero_or_one() {
    for key in [
        "Aaron",
        "Zz",
        "Qu",
        "Xyz",
        "Eve",
        "Oz",
        "Sw",
        "1001 nights",
        "0",
    ] {
        let cut = cutter(key, 3).unwrap().to_string();
        assert!(!cut.ends_with(['0', '1']), "{key} -> {cut}");
    }
}

#[test]
fn numerals_use_the_a12_a19_span() {
    assert_eq!(c("100 best hamburger recipes"), "A12");
    assert_eq!(c("1981 commercial composite"), "A12");
    assert_eq!(c("9 lives"), "A19");
    let digits = cutter("1981", 3).unwrap();
    assert_eq!(digits.to_string(), "A129");
}

#[test]
fn case_diacritics_and_punctuation_fold() {
    assert_eq!(c("campbell"), c("CAMPBELL"));
    assert_eq!(c("Émile"), c("Emile"));
    assert_eq!(c("O'Brien"), c("OBrien"));
    assert_eq!(c("Straße"), c("Strasse"));
    assert_eq!(cutter("—", 2), None);
}

#[test]
fn ordering_is_decimal() {
    let mut cuts: Vec<_> = [
        "Cymbal",
        "Campbell",
        "Cobblestone",
        "Ceccaldi",
        "Cuellar",
        "Cryer",
    ]
    .iter()
    .map(|k| cutter(k, 2).unwrap())
    .collect();
    cuts.sort();
    let order: Vec<_> = cuts.iter().map(ToString::to_string).collect();
    assert_eq!(order, ["C36", "C43", "C63", "C79", "C84", "C96"]);
    assert!(cutter("Ca", 2).unwrap() < cutter("Campbell", 2).unwrap());
}
