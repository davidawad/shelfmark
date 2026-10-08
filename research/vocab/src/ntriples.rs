//! Minimal line-oriented N-Triples parser.
//!
//! Enough of the W3C grammar for the id.loc.gov and OCLC bulk dumps: IRIs,
//! blank nodes, and literals with escapes, language tags or datatypes.
//! Comment and blank lines yield `None`; malformed lines yield an error the
//! caller counts rather than aborting a multi-gigabyte ingest.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Iri(String),
    Blank(String),
    Literal {
        value: String,
        lang: Option<String>,
        datatype: Option<String>,
    },
}

impl Term {
    pub fn as_iri(&self) -> Option<&str> {
        match self {
            Term::Iri(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_blank(&self) -> Option<&str> {
        match self {
            Term::Blank(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_literal(&self) -> Option<&str> {
        match self {
            Term::Literal { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Language tag of a literal; `None` for untagged literals and non-literals.
    pub fn lang(&self) -> Option<&str> {
        match self {
            Term::Literal { lang, .. } => lang.as_deref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triple {
    pub subject: Term,
    pub predicate: String,
    pub object: Term,
}

/// Parse one N-Triples line. `Ok(None)` for comments and blank lines.
pub fn parse_line(line: &str) -> Result<Option<Triple>, String> {
    let mut cur = Cursor { s: line, pos: 0 };
    cur.skip_ws();
    if cur.at_end() || cur.peek() == Some('#') {
        return Ok(None);
    }
    let subject = cur.term()?;
    cur.skip_ws();
    let predicate = match cur.term()? {
        Term::Iri(p) => p,
        other => return Err(format!("predicate is not an IRI: {other:?}")),
    };
    cur.skip_ws();
    let object = cur.term()?;
    cur.skip_ws();
    if cur.peek() != Some('.') {
        return Err(format!("expected '.' at byte {}", cur.pos));
    }
    Ok(Some(Triple {
        subject,
        predicate,
        object,
    }))
}

struct Cursor<'a> {
    s: &'a str,
    pos: usize,
}

impl Cursor<'_> {
    fn at_end(&self) -> bool {
        self.pos >= self.s.len()
    }

    fn peek(&self) -> Option<char> {
        self.s[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\r' | '\n')) {
            self.pos += 1;
        }
    }

    fn term(&mut self) -> Result<Term, String> {
        match self.peek() {
            Some('<') => Ok(Term::Iri(self.iri()?)),
            Some('_') => {
                if !self.s[self.pos..].starts_with("_:") {
                    return Err(format!("bad blank node at byte {}", self.pos));
                }
                self.pos += 2;
                let start = self.pos;
                while let Some(c) = self.peek() {
                    if c.is_whitespace() {
                        break;
                    }
                    self.pos += c.len_utf8();
                }
                Ok(Term::Blank(self.s[start..self.pos].to_string()))
            }
            Some('"') => self.literal(),
            other => Err(format!("unexpected {other:?} at byte {}", self.pos)),
        }
    }

    fn iri(&mut self) -> Result<String, String> {
        self.bump(); // '<'
        let rest = &self.s[self.pos..];
        let end = rest
            .find('>')
            .ok_or_else(|| "unterminated IRI".to_string())?;
        let raw = &rest[..end];
        self.pos += end + 1;
        if raw.contains('\\') {
            unescape(raw)
        } else {
            Ok(raw.to_string())
        }
    }

    fn literal(&mut self) -> Result<Term, String> {
        self.bump(); // opening quote
        let mut value = String::new();
        loop {
            match self.bump() {
                None => return Err("unterminated literal".into()),
                Some('"') => break,
                Some('\\') => {
                    let esc = self.bump().ok_or("dangling escape")?;
                    value.push(self.escape(esc)?);
                }
                Some(c) => value.push(c),
            }
        }
        let mut lang = None;
        let mut datatype = None;
        if self.peek() == Some('@') {
            self.bump();
            let start = self.pos;
            while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '-') {
                self.pos += 1;
            }
            lang = Some(self.s[start..self.pos].to_ascii_lowercase());
        } else if self.s[self.pos..].starts_with("^^") {
            self.pos += 2;
            datatype = Some(self.iri()?);
        }
        Ok(Term::Literal {
            value,
            lang,
            datatype,
        })
    }

    fn escape(&mut self, esc: char) -> Result<char, String> {
        Ok(match esc {
            't' => '\t',
            'b' => '\u{8}',
            'n' => '\n',
            'r' => '\r',
            'f' => '\u{c}',
            '"' => '"',
            '\'' => '\'',
            '\\' => '\\',
            'u' => self.hex(4)?,
            'U' => self.hex(8)?,
            other => return Err(format!("unknown escape \\{other}")),
        })
    }

    fn hex(&mut self, n: usize) -> Result<char, String> {
        let digits = self
            .s
            .get(self.pos..self.pos + n)
            .ok_or("short unicode escape")?;
        self.pos += n;
        let code = u32::from_str_radix(digits, 16).map_err(|e| e.to_string())?;
        char::from_u32(code).ok_or_else(|| format!("invalid code point {code:x}"))
    }
}

fn unescape(raw: &str) -> Result<String, String> {
    let mut out = String::with_capacity(raw.len());
    let mut cur = Cursor { s: raw, pos: 0 };
    while let Some(c) = cur.bump() {
        if c == '\\' {
            let esc = cur.bump().ok_or("dangling escape")?;
            out.push(cur.escape(esc)?);
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_iri_literal_and_blank() {
        let t = parse_line(r#"<http://x/s> <http://x/p> "Café \"q\""@EN ."#)
            .unwrap()
            .unwrap();
        assert_eq!(t.subject, Term::Iri("http://x/s".into()));
        assert_eq!(t.object.as_literal(), Some("Café \"q\""));
        assert_eq!(t.object.lang(), Some("en"));

        let t = parse_line("_:b1 <http://x/p> _:b2 .").unwrap().unwrap();
        assert_eq!(t.subject.as_blank(), Some("b1"));
        assert_eq!(t.object.as_blank(), Some("b2"));

        let t = parse_line(r#"_:b <http://x/p> "1"^^<http://x/int> ."#)
            .unwrap()
            .unwrap();
        assert!(
            matches!(t.object, Term::Literal { ref datatype, .. } if datatype.as_deref() == Some("http://x/int"))
        );
    }

    #[test]
    fn skips_comments_and_rejects_garbage() {
        assert_eq!(parse_line("# BEGIN /x").unwrap(), None);
        assert_eq!(parse_line("   ").unwrap(), None);
        assert!(parse_line("<http://x/s> \"lit\" <o> .").is_err());
        assert!(parse_line("<http://x/s> <http://x/p> \"open .").is_err());
    }
}
