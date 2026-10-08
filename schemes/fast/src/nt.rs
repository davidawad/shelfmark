//! A line-at-a-time N-Triples reader, enough for OCLC's FAST dumps: IRIs,
//! blank nodes, plain literals with N-Triples escapes, and literals tagged
//! with a datatype or language (the tag is dropped).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Iri(String),
    Blank(String),
    Literal(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triple {
    /// An IRI, or a blank node as `_:label`.
    pub subject: String,
    pub predicate: String,
    pub object: Object,
}

/// Parse one line. Blank lines and comments are `Ok(None)`.
pub fn parse_line(line: &str) -> Result<Option<Triple>, String> {
    let rest = line.trim();
    if rest.is_empty() || rest.starts_with('#') {
        return Ok(None);
    }
    let (subject, rest) = match blank(rest) {
        Some((b, r)) => (format!("_:{b}"), r),
        None => iri(rest)?,
    };
    let (predicate, rest) = iri(rest.trim_start())?;
    let rest = rest.trim_start();
    let (object, rest) = if let Some((b, r)) = blank(rest) {
        (Object::Blank(b), r)
    } else if rest.starts_with('<') {
        let (o, r) = iri(rest)?;
        (Object::Iri(o), r)
    } else {
        let (o, r) = literal(rest)?;
        (Object::Literal(o), r)
    };
    if rest.trim() != "." {
        return Err(format!(
            "expected `.` after the object, found `{}`",
            rest.trim()
        ));
    }
    Ok(Some(Triple {
        subject,
        predicate,
        object,
    }))
}

fn iri(s: &str) -> Result<(String, &str), String> {
    let body = s
        .strip_prefix('<')
        .ok_or_else(|| format!("expected an IRI at `{}`", head(s)))?;
    let end = body
        .find('>')
        .ok_or_else(|| format!("unterminated IRI at `{}`", head(s)))?;
    Ok((body[..end].to_string(), &body[end + 1..]))
}

/// A `_:label` blank node: the label and the rest.
fn blank(s: &str) -> Option<(String, &str)> {
    let body = s.strip_prefix("_:")?;
    let end = body.find(char::is_whitespace).unwrap_or(body.len());
    Some((body[..end].to_string(), &body[end..]))
}

fn literal(s: &str) -> Result<(String, &str), String> {
    let mut chars = s
        .strip_prefix('"')
        .ok_or_else(|| format!("expected a literal at `{}`", head(s)))?
        .char_indices();
    let body = &s[1..];
    let mut out = String::new();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Ok((out, skip_tag(&body[i + 1..]))),
            '\\' => {
                let (_, e) = chars.next().ok_or("dangling escape")?;
                match e {
                    't' => out.push('\t'),
                    'b' => out.push('\u{8}'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    'f' => out.push('\u{c}'),
                    '"' | '\'' | '\\' => out.push(e),
                    'u' | 'U' => {
                        let n = if e == 'u' { 4 } else { 8 };
                        let hex: String = (0..n)
                            .filter_map(|_| chars.next().map(|(_, h)| h))
                            .collect();
                        let cp = u32::from_str_radix(&hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                            .ok_or_else(|| format!("bad escape \\{e}{hex}"))?;
                        out.push(cp);
                    }
                    _ => return Err(format!("unknown escape \\{e}")),
                }
            }
            _ => out.push(c),
        }
    }
    Err(format!("unterminated literal at `{}`", head(s)))
}

/// Skip a `^^<datatype>` or `@lang` suffix.
fn skip_tag(s: &str) -> &str {
    if let Some(r) = s.strip_prefix("^^<") {
        return r.find('>').map_or("", |e| &r[e + 1..]);
    }
    if let Some(r) = s.strip_prefix('@') {
        return r.find(|c: char| c.is_whitespace()).map_or("", |e| &r[e..]);
    }
    s
}

fn head(s: &str) -> &str {
    let end = s.char_indices().nth(40).map_or(s.len(), |(i, _)| i);
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_iris_literals_escapes_and_tags() {
        let t = parse_line(r#"<http://x/1> <http://p> "A \"b\" é\\" ."#)
            .unwrap()
            .unwrap();
        assert_eq!(t.subject, "http://x/1");
        assert_eq!(t.object, Object::Literal("A \"b\" é\\".into()));
        let t = parse_line(r#"<a> <b> "http://w"^^<http://t#anyURI> ."#)
            .unwrap()
            .unwrap();
        assert_eq!(t.object, Object::Literal("http://w".into()));
        let t = parse_line(r#"<a> <b> "chat"@fr ."#).unwrap().unwrap();
        assert_eq!(t.object, Object::Literal("chat".into()));
        let t = parse_line("<a> <b> <c> .").unwrap().unwrap();
        assert_eq!(t.object, Object::Iri("c".into()));
        let t = parse_line("<a> <b> _:B1 .").unwrap().unwrap();
        assert_eq!(t.object, Object::Blank("B1".into()));
        let t = parse_line(r#"_:B1 <b> "x" ."#).unwrap().unwrap();
        assert_eq!(t.subject, "_:B1");
        assert_eq!(parse_line("  ").unwrap(), None);
        assert!(parse_line("<a> <b> \"x").is_err());
        assert!(parse_line("<a> <b> <c>").is_err());
    }

    #[test]
    fn a_literal_may_start_with_at() {
        let t = parse_line(r#"<a> <b> "@ symbol" ."#).unwrap().unwrap();
        assert_eq!(t.object, Object::Literal("@ symbol".into()));
    }
}
