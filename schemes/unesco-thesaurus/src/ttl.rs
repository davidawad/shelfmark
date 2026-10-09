//! A reader for the Turtle subset UNESCO's export uses: `PREFIX` lines,
//! then statements `subject pred obj (, obj)* (; pred obj ...)* .` whose
//! terms are `<iri>`, prefixed names and `"..."` literals with an
//! optional `@lang` or `^^datatype`. Blank nodes, collections and
//! triple-quoted strings are errors, never silent skips.

/// An object term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    /// An IRI (without brackets) or prefixed name, as written.
    Name(String),
    /// A literal: text and language tag (empty when none).
    Lit { text: String, lang: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: Term,
}

#[derive(Debug, PartialEq, Eq)]
enum Tok {
    Name(String),
    Lit(String, String),
    Semi,
    Comma,
    Dot,
}

fn is_end(c: char) -> bool {
    c.is_whitespace() || c == ';' || c == ','
}

fn lex(src: &str) -> Result<Vec<Tok>, String> {
    let cs: Vec<char> = src.chars().collect();
    let (mut i, mut out) = (0, Vec::new());
    while i < cs.len() {
        let c = cs[i];
        match c {
            _ if c.is_whitespace() => i += 1,
            ';' => {
                out.push(Tok::Semi);
                i += 1;
            }
            ',' => {
                out.push(Tok::Comma);
                i += 1;
            }
            '#' => {
                while i < cs.len() && cs[i] != '\n' {
                    i += 1;
                }
            }
            '<' => {
                let end = cs[i..]
                    .iter()
                    .position(|&c| c == '>')
                    .ok_or("unterminated <iri>")?;
                out.push(Tok::Name(cs[i + 1..i + end].iter().collect()));
                i += end + 1;
            }
            '"' => {
                if cs.get(i + 1) == Some(&'"') && cs.get(i + 2) == Some(&'"') {
                    return Err("triple-quoted strings are not supported".into());
                }
                let (text, next) = string(&cs, i + 1)?;
                i = next;
                let mut lang = String::new();
                if cs.get(i) == Some(&'@') {
                    i += 1;
                    while i < cs.len() && (cs[i].is_ascii_alphanumeric() || cs[i] == '-') {
                        lang.push(cs[i]);
                        i += 1;
                    }
                } else if cs.get(i) == Some(&'^') && cs.get(i + 1) == Some(&'^') {
                    i += 2;
                    while i < cs.len() && !is_end(cs[i]) {
                        i += 1;
                    }
                }
                out.push(Tok::Lit(text, lang));
            }
            '[' | '(' => return Err(format!("`{c}` (blank node or collection) is not supported")),
            _ => {
                let start = i;
                while i < cs.len() && !is_end(cs[i]) {
                    i += 1;
                }
                let mut word: String = cs[start..i].iter().collect();
                if word == "." {
                    out.push(Tok::Dot);
                    continue;
                }
                let dot = word.ends_with('.');
                if dot {
                    word.pop();
                }
                out.push(Tok::Name(word));
                if dot {
                    out.push(Tok::Dot);
                }
            }
        }
    }
    Ok(out)
}

/// The string body from `i` (after the opening quote): the text and the
/// index after the closing quote.
fn string(cs: &[char], mut i: usize) -> Result<(String, usize), String> {
    let mut s = String::new();
    while i < cs.len() {
        match cs[i] {
            '"' => return Ok((s, i + 1)),
            '\\' => {
                i += 1;
                let e = *cs.get(i).ok_or("dangling backslash")?;
                match e {
                    '"' | '\\' | '\'' => s.push(e),
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    'r' => s.push('\r'),
                    'b' => s.push('\u{8}'),
                    'f' => s.push('\u{c}'),
                    'u' | 'U' => {
                        let n = if e == 'u' { 4 } else { 8 };
                        let hex: String = cs
                            .get(i + 1..i + 1 + n)
                            .ok_or("short \\u escape")?
                            .iter()
                            .collect();
                        let ch = u32::from_str_radix(&hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                            .ok_or_else(|| format!("bad \\{e}{hex} escape"))?;
                        s.push(ch);
                        i += n;
                    }
                    _ => return Err(format!("unknown escape \\{e}")),
                }
                i += 1;
            }
            c => {
                s.push(c);
                i += 1;
            }
        }
    }
    Err("unterminated string".into())
}

/// Every triple in `src`, in file order.
pub fn parse(src: &str) -> Result<Vec<Triple>, String> {
    let body = src
        .lines()
        .filter(|l| {
            let l = l.trim_start();
            !l.starts_with("PREFIX") && !l.starts_with("@prefix")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let toks = lex(&body)?;
    let (mut triples, mut i) = (Vec::new(), 0);
    while i < toks.len() {
        let Tok::Name(subject) = &toks[i] else {
            return Err(format!("expected a subject, found {:?}", toks[i]));
        };
        i += 1;
        loop {
            let Some(Tok::Name(predicate)) = toks.get(i) else {
                return Err(format!("`{subject}`: expected a predicate at token {i}"));
            };
            i += 1;
            loop {
                let object = match toks.get(i) {
                    Some(Tok::Name(n)) => Term::Name(n.clone()),
                    Some(Tok::Lit(t, l)) => Term::Lit {
                        text: t.clone(),
                        lang: l.clone(),
                    },
                    other => return Err(format!("`{subject}` {predicate}: bad object {other:?}")),
                };
                i += 1;
                triples.push(Triple {
                    subject: subject.clone(),
                    predicate: predicate.clone(),
                    object,
                });
                if toks.get(i) == Some(&Tok::Comma) {
                    i += 1;
                } else {
                    break;
                }
            }
            match toks.get(i) {
                Some(Tok::Semi) => {
                    i += 1;
                    if toks.get(i) == Some(&Tok::Dot) {
                        i += 1;
                        break;
                    }
                }
                Some(Tok::Dot) => {
                    i += 1;
                    break;
                }
                other => return Err(format!("`{subject}`: expected `;` or `.`, found {other:?}")),
            }
        }
    }
    Ok(triples)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(t: &str, l: &str) -> Term {
        Term::Lit {
            text: t.into(),
            lang: l.into(),
        }
    }

    #[test]
    fn reads_statements_lists_and_escapes() {
        let src = "PREFIX : <http://x/>\n\
            :a  rdf:type skos:Concept;\n\
              skos:prefLabel \"A; b, c.\"@en , \"say \\\"hi\\\" \\u00e9\"@fr;\n\
              dct:modified \"2019-01-01T00:00:00+00:00\"^^xsd:dateTime;\n\
              skos:broader :b , :c .\n\
            <http://x/s> skos:prefLabel \"x\"@en .";
        let t = parse(src).unwrap();
        assert_eq!(t.len(), 7);
        assert_eq!(t[1].object, lit("A; b, c.", "en"));
        assert_eq!(t[2].object, lit("say \"hi\" \u{e9}", "fr"));
        assert_eq!(t[3].object, lit("2019-01-01T00:00:00+00:00", ""));
        assert_eq!(t[5].object, Term::Name(":c".into()));
        assert_eq!(t[6].subject, "http://x/s");
    }

    #[test]
    fn dotted_names_keep_their_dots() {
        let t = parse(":mt1.05 skos:notation \"1.05\" .\n:d skos:member :mt1.05 .").unwrap();
        assert_eq!(t[0].subject, ":mt1.05");
        assert_eq!(t[1].object, Term::Name(":mt1.05".into()));
    }

    #[test]
    fn refuses_unsupported_syntax() {
        assert!(parse(":a :p [ :q :r ] .").is_err());
        assert!(parse(":a :p \"x .").is_err());
        assert!(parse(":a :p :b").is_err());
    }
}
