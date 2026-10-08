//! `classificationTree.xml`: nested `<classification level="N">` elements,
//! each with a `<code>` and a `<description>`, into a flat list in
//! document order with parents taken from the nesting.

/// One code as the XML states it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub code: String,
    /// 1 (letter), 2 (letter and digit) or 3 (letter and two digits).
    pub level: u8,
    pub label: String,
    pub parent: Option<String>,
}

/// Every classification in document order. Fails on anything the builder
/// would otherwise have to guess at: a missing code or description, a
/// level that disagrees with the nesting or the code's length, a child
/// code that does not extend its parent's, a repeated code, or a character
/// entity the label decoder does not know.
pub fn entries(xml: &[u8]) -> Result<Vec<Entry>, String> {
    let text = std::str::from_utf8(xml).map_err(|e| format!("source is not UTF-8: {e}"))?;
    let doc = roxmltree::Document::parse(text).map_err(|e| format!("source XML: {e}"))?;
    let mut out = Vec::new();
    walk(doc.root_element(), None, 1, &mut out)?;
    let mut seen = std::collections::BTreeSet::new();
    for e in &out {
        if !seen.insert(e.code.as_str()) {
            return Err(format!("code {} appears twice", e.code));
        }
    }
    if out.is_empty() {
        return Err("source has no classifications".into());
    }
    Ok(out)
}

fn walk(
    node: roxmltree::Node,
    parent: Option<&str>,
    depth: u8,
    out: &mut Vec<Entry>,
) -> Result<(), String> {
    for c in node.children().filter(|n| n.has_tag_name("classification")) {
        let code = child_text(c, "code")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("a classification under {parent:?} has no code"))?
            .to_string();
        let raw =
            child_text(c, "description").ok_or_else(|| format!("{code} has no description"))?;
        let label = label(raw).map_err(|e| format!("{code}: {e}"))?;
        if label.is_empty() {
            return Err(format!("{code} has an empty description"));
        }
        let level: u8 = c
            .attribute("level")
            .and_then(|l| l.parse().ok())
            .ok_or_else(|| format!("{code} has no numeric level"))?;
        if level != depth || code.len() != usize::from(depth) {
            return Err(format!(
                "{code} is level {level} at nesting depth {depth}; JEL codes are 1-3 characters, one per level"
            ));
        }
        if let Some(p) = parent {
            if !code.starts_with(p) {
                return Err(format!("{code} is nested under {p} but does not extend it"));
            }
        }
        out.push(Entry {
            code: code.clone(),
            level,
            label,
            parent: parent.map(str::to_string),
        });
        walk(c, Some(&code), depth + 1, out)?;
    }
    Ok(())
}

fn child_text<'a>(node: roxmltree::Node<'a, 'a>, tag: &str) -> Option<&'a str> {
    node.children().find(|n| n.has_tag_name(tag))?.text()
}

/// A description as a label: HTML character entities decoded (the AEA
/// writes `&bull;`-separated topic lists inside CDATA), `•` separators
/// written as `; ` (the AEA's own flat `classifications.xml` does the
/// same) and whitespace collapsed.
pub fn label(raw: &str) -> Result<String, String> {
    let mut s = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(at) = rest.find('&') {
        s.push_str(&rest[..at]);
        let tail = &rest[at..];
        // CDATA keeps a literal `&` too ("R&D"): only `&name;` is an entity.
        let name = tail[1..].find(';').map(|e| &tail[1..=e]).filter(|n| {
            !n.is_empty()
                && n.len() <= 8
                && n.chars().all(|c| c == '#' || c.is_ascii_alphanumeric())
        });
        let Some(name) = name else {
            s.push('&');
            rest = &tail[1..];
            continue;
        };
        s.push(entity(name).ok_or_else(|| format!("unknown entity &{name};"))?);
        rest = &tail[name.len() + 2..];
    }
    s.push_str(rest);
    let words = s.split_whitespace().collect::<Vec<_>>().join(" ");
    Ok(words.replace(" •", ";").replace('•', ";"))
}

fn entity(name: &str) -> Option<char> {
    if let Some(num) = name.strip_prefix('#') {
        let n = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => num.parse().ok()?,
        };
        return char::from_u32(n);
    }
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => ' ',
        "bull" => '•',
        "ndash" => '–',
        "mdash" => '—',
        "rsquo" => '’',
        "lsquo" => '‘',
        "rdquo" => '”',
        "ldquo" => '“',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_decode_entities_and_bullets() {
        assert_eq!(
            label("Role of Economics &bull; Role of Economists").unwrap(),
            "Role of Economics; Role of Economists"
        );
        assert_eq!(label("  Pre&ndash;1800\n ").unwrap(), "Pre–1800");
        assert_eq!(label("R&amp;D &#38; &#x41;").unwrap(), "R&D & A");
        assert!(label("A &foo; B").unwrap_err().contains("&foo;"));
        assert_eq!(label("Innovation and R&D").unwrap(), "Innovation and R&D");
        assert_eq!(label("A & B; C").unwrap(), "A & B; C");
    }

    #[test]
    fn nesting_sets_parents_and_rejects_bad_levels() {
        let xml = br#"<data><classification level="1"><code>A</code><description>a</description>
            <classification level="2"><code>A1</code><description><![CDATA[x &bull; y]]></description>
            <classification level="3"><code>A10</code><description>g</description></classification>
            </classification></classification></data>"#;
        let e = entries(xml).unwrap();
        let got: Vec<_> = e
            .iter()
            .map(|e| (e.code.as_str(), e.parent.as_deref()))
            .collect();
        assert_eq!(got, [("A", None), ("A1", Some("A")), ("A10", Some("A1"))]);
        assert_eq!(e[1].label, "x; y");

        let wrong_prefix =
            br#"<data><classification level="1"><code>A</code><description>a</description>
            <classification level="2"><code>B1</code><description>b</description></classification>
            </classification></data>"#;
        assert!(entries(wrong_prefix)
            .unwrap_err()
            .contains("does not extend"));
        let wrong_level = br#"<data><classification level="2"><code>A</code><description>a</description></classification></data>"#;
        assert!(entries(wrong_level).unwrap_err().contains("level 2"));
        let twice = br#"<data><classification level="1"><code>A</code><description>a</description></classification>
            <classification level="1"><code>A</code><description>a</description></classification></data>"#;
        assert!(entries(twice).unwrap_err().contains("twice"));
    }
}
