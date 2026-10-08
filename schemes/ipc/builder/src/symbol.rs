//! IPC symbols. The master files write a group as 14 characters,
//! subclass + 4-digit main group + 6-digit subgroup (`A01B0001020000`);
//! the scheme uses the compact printed form without the space
//! (`A01B1/02`), since a shelfmark code may not contain whitespace.

/// Regex every IPC code in the scheme matches: section, class, subclass,
/// or `main/sub` group.
pub const CODE_GRAMMAR: &str = "[A-H]([0-9]{2}([A-Z]([1-9][0-9]{0,3}/[0-9]{2,6})?)?)?";

/// The compact form of a master-file symbol: `A`, `A01`, `A01B` stay as
/// they are; `A01B0001020000` becomes `A01B1/02` and `A01B0001022000`
/// becomes `A01B1/022`. `None` for anything else.
pub fn compact(symbol: &str) -> Option<String> {
    let b = symbol.as_bytes();
    let section = |c: u8| (b'A'..=b'H').contains(&c);
    let digits = |s: &[u8]| s.iter().all(u8::is_ascii_digit);
    match b.len() {
        1 if section(b[0]) => Some(symbol.to_string()),
        3 if section(b[0]) && digits(&b[1..3]) => Some(symbol.to_string()),
        4 if section(b[0]) && digits(&b[1..3]) && b[3].is_ascii_uppercase() => {
            Some(symbol.to_string())
        }
        14 if section(b[0])
            && digits(&b[1..3])
            && b[3].is_ascii_uppercase()
            && digits(&b[4..14]) =>
        {
            let main = symbol[4..8].trim_start_matches('0');
            if main.is_empty() {
                return None;
            }
            let mut sub = symbol[8..14].trim_end_matches('0').to_string();
            while sub.len() < 2 {
                sub.push('0');
            }
            Some(format!("{}{main}/{sub}", &symbol[..4]))
        }
        _ => None,
    }
}

/// The edition attribute (`20260101`) as the scheme version (`2026.01`).
pub fn version(edition: &str) -> Option<String> {
    let ok = edition.len() == 8 && edition.bytes().all(|c| c.is_ascii_digit());
    ok.then(|| format!("{}.{}", &edition[..4], &edition[4..6]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compacts_master_file_symbols() {
        for (from, to) in [
            ("A", "A"),
            ("A01", "A01"),
            ("A01B", "A01B"),
            ("A01B0001000000", "A01B1/00"),
            ("A01B0001020000", "A01B1/02"),
            ("A01B0001022000", "A01B1/022"),
            ("H01L0021027100", "H01L21/0271"),
            ("A61K2800100000", "A61K2800/10"),
            ("C12R0001010000", "C12R1/01"),
            ("H04W0084180000", "H04W84/18"),
        ] {
            assert_eq!(compact(from).as_deref(), Some(to), "{from}");
        }
        for bad in [
            "",
            "I",
            "A1",
            "A01b",
            "A01B0000000000",
            "A01B00010200",
            "a01B",
        ] {
            assert_eq!(compact(bad), None, "{bad}");
        }
    }

    #[test]
    fn compact_symbols_match_the_grammar() {
        for code in [
            "A",
            "H01",
            "H01B",
            "A01B1/00",
            "H01L21/02104",
            "A61K2800/10",
        ] {
            assert!(matches(code), "{code}");
        }
        for code in ["I", "A01B 1/00", "A01B01/00", "A01B1/0", "A01B1"] {
            assert!(!matches(code), "{code}");
        }
    }

    /// Whether `code` passes the grammar as the scheme compiler anchors it.
    fn matches(code: &str) -> bool {
        let doc = serde_json::json!({"name": "t", "version": "1", "title": "t",
            "code_grammar": CODE_GRAMMAR, "classes": [{"code": code, "label": "x"}]});
        shelfmark_scheme::validate_value(&doc).is_ok()
    }

    #[test]
    fn versions_from_editions() {
        assert_eq!(version("20260101").as_deref(), Some("2026.01"));
        assert_eq!(version("2026010"), None);
        assert_eq!(version("2026-01-"), None);
    }
}
