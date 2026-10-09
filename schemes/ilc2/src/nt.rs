//! A small N-Triples reader: one statement per line, IRIs in `<>`, plain
//! or language-tagged or datatyped literals. No blank nodes: the ILC2
//! file has none, and one is an error.

/// The object of a triple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Iri(String),
    /// A literal's text (escapes resolved); language tag and datatype
    /// are dropped, the ILC2 file being English throughout.
    Literal(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: Object,
}

/// Parse a whole file. Errors name the 1-based line.
pub fn parse(text: &str) -> Result<Vec<Triple>, String> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        out.push(parse_line(line).map_err(|e| format!("line {}: {e}", i + 1))?);
    }
    Ok(out)
}

fn parse_line(line: &str) -> Result<Triple, String> {
    let (subject, rest) = iri(line)?;
    let (predicate, rest) = iri(rest.trim_start())?;
    let rest = rest.trim_start();
    let (object, rest) = if rest.starts_with('<') {
        let (o, r) = iri(rest)?;
        (Object::Iri(o), r)
    } else if rest.starts_with('"') {
        literal(rest)?
    } else {
        return Err(format!("unsupported object: {}", preview(rest)));
    };
    if rest.trim() != "." {
        return Err(format!("expected `.` after the object: {}", preview(rest)));
    }
    Ok(Triple {
        subject,
        predicate,
        object,
    })
}

fn preview(s: &str) -> String {
    s.chars().take(40).collect()
}

/// `<iri>` at the start of `s`; returns the IRI and the rest.
fn iri(s: &str) -> Result<(String, &str), String> {
    let body = s
        .strip_prefix('<')
        .ok_or_else(|| format!("expected `<`: {}", preview(s)))?;
    let end = body.find('>').ok_or("unterminated IRI")?;
    Ok((body[..end].to_string(), &body[end + 1..]))
}

/// `"text"` with an optional `@lang` or `^^<datatype>` at the start of `s`.
fn literal(s: &str) -> Result<(Object, &str), String> {
    let mut chars = s[1..].char_indices();
    let mut text = String::new();
    let end = loop {
        let (i, c) = chars.next().ok_or("unterminated literal")?;
        match c {
            '"' => break i + 2,
            '\\' => text.push(escape(&mut chars)?),
            c => text.push(c),
        }
    };
    let rest = &s[end..];
    let rest = if let Some(tag) = rest.strip_prefix('@') {
        let n = tag
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(tag.len());
        &tag[n..]
    } else if let Some(dt) = rest.strip_prefix("^^") {
        iri(dt)?.1
    } else {
        rest
    };
    Ok((Object::Literal(text), rest))
}

fn escape(chars: &mut std::str::CharIndices<'_>) -> Result<char, String> {
    let (_, c) = chars.next().ok_or("dangling backslash")?;
    let hex = |chars: &mut std::str::CharIndices<'_>, n: usize| -> Result<char, String> {
        let digits: String = chars.by_ref().take(n).map(|(_, c)| c).collect();
        u32::from_str_radix(&digits, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or_else(|| format!("bad \\u escape `{digits}`"))
    };
    Ok(match c {
        't' => '\t',
        'n' => '\n',
        'r' => '\r',
        'b' => '\u{8}',
        'f' => '\u{c}',
        '"' => '"',
        '\'' => '\'',
        '\\' => '\\',
        'u' => hex(chars, 4)?,
        'U' => hex(chars, 8)?,
        other => return Err(format!("unknown escape \\{other}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_iri_and_literal_objects() {
        let t = parse(
            "<a> <b> <c> .\n# comment\n\n<a> <b> \"x \\\"q\\\" \\u03B3\"@en .\n\
             <a> <b> \"k\"^^<http://www.w3.org/2001/XMLSchema#string> .\n",
        )
        .unwrap();
        assert_eq!(t.len(), 3);
        assert_eq!(t[0].object, Object::Iri("c".into()));
        assert_eq!(t[1].object, Object::Literal("x \"q\" \u{3b3}".into()));
        assert_eq!(t[2].object, Object::Literal("k".into()));
    }

    #[test]
    fn rejects_bad_lines() {
        assert!(parse("<a> <b> _:x .").unwrap_err().contains("line 1"));
        assert!(parse("<a> <b> \"open .").is_err());
        assert!(parse("<a> <b> <c>").is_err());
        assert!(parse("<a> <b> \"\\q\" .").is_err());
    }
}
