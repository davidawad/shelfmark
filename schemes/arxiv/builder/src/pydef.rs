//! A reader for the Python literals in arxiv-base's
//! `arxiv/taxonomy/definitions.py`.
//!
//! The file is data written as Python: module-level assignments of dict
//! literals whose values are strings, booleans, `date(y, m, d)` calls and
//! `Group(...)`/`Archive(...)`/`Category(...)` constructor calls with
//! keyword arguments. [`assignment`] finds one named assignment and parses
//! its value; anything outside that subset is an error, so a change in the
//! file's shape fails the build instead of being misread.

/// A parsed Python literal.
#[derive(Debug, Clone, PartialEq)]
pub enum Py {
    Str(String),
    Int(i64),
    Bool(bool),
    None,
    /// `name(arg, key=value, ...)`; positional arguments have no key.
    Call {
        name: String,
        args: Vec<(Option<String>, Py)>,
    },
    Dict(Vec<(Py, Py)>),
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Str(String),
    Int(i64),
    Ident(String),
    Punct(char),
}

fn tokenize(src: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() || c == '\\' {
            i += 1;
        } else if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '"' || c == '\'' {
            let (s, next) = string(&chars, i)?;
            out.push(Tok::Str(s));
            i = next;
        } else if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            let n = text.parse().map_err(|e| format!("number {text}: {e}"))?;
            out.push(Tok::Int(n));
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            out.push(Tok::Ident(chars[start..i].iter().collect()));
        } else {
            out.push(Tok::Punct(c));
            i += 1;
        }
    }
    Ok(out)
}

/// A string literal starting at `chars[i]`: its value and the index after it.
fn string(chars: &[char], i: usize) -> Result<(String, usize), String> {
    let q = chars[i];
    let triple = chars.get(i + 1) == Some(&q) && chars.get(i + 2) == Some(&q);
    let mut j = if triple { i + 3 } else { i + 1 };
    let mut s = String::new();
    loop {
        let Some(&c) = chars.get(j) else {
            return Err("unterminated string literal".into());
        };
        if c == q && (!triple || (chars.get(j + 1) == Some(&q) && chars.get(j + 2) == Some(&q))) {
            return Ok((s, if triple { j + 3 } else { j + 1 }));
        }
        if c == '\n' && !triple {
            return Err("newline in a string literal".into());
        }
        if c == '\\' {
            let e = *chars.get(j + 1).ok_or("unterminated escape")?;
            s.push(match e {
                'n' => '\n',
                't' => '\t',
                '\\' | '\'' | '"' => e,
                _ => return Err(format!("unsupported escape \\{e}")),
            });
            j += 2;
        } else {
            s.push(c);
            j += 1;
        }
    }
}

struct Parser<'a> {
    toks: &'a [Tok],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn next(&mut self) -> Result<Tok, String> {
        let t = self.peek().cloned().ok_or("unexpected end of file")?;
        self.pos += 1;
        Ok(t)
    }

    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(&Tok::Punct(c)) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        match self.next()? {
            Tok::Punct(p) if p == c => Ok(()),
            t => Err(format!("expected `{c}`, found {t:?}")),
        }
    }

    fn value(&mut self) -> Result<Py, String> {
        match self.next()? {
            Tok::Str(s) => Ok(Py::Str(s)),
            Tok::Int(n) => Ok(Py::Int(n)),
            Tok::Punct('{') => self.dict(),
            Tok::Ident(name) => match name.as_str() {
                "True" => Ok(Py::Bool(true)),
                "False" => Ok(Py::Bool(false)),
                "None" => Ok(Py::None),
                _ if self.eat('(') => self.call(name),
                _ => Err(format!("bare name `{name}` is not a literal")),
            },
            t => Err(format!("unexpected {t:?}")),
        }
    }

    fn dict(&mut self) -> Result<Py, String> {
        let mut items = Vec::new();
        while !self.eat('}') {
            let k = self.value()?;
            self.expect(':')?;
            let v = self.value()?;
            items.push((k, v));
            if !self.eat(',') {
                self.expect('}')?;
                break;
            }
        }
        Ok(Py::Dict(items))
    }

    fn call(&mut self, name: String) -> Result<Py, String> {
        let mut args = Vec::new();
        while !self.eat(')') {
            let keyword = match (self.peek(), self.toks.get(self.pos + 1)) {
                (Some(Tok::Ident(k)), Some(Tok::Punct('='))) => Some(k.clone()),
                _ => None,
            };
            if keyword.is_some() {
                self.pos += 2;
            }
            args.push((keyword, self.value()?));
            if !self.eat(',') {
                self.expect(')')?;
                break;
            }
        }
        Ok(Py::Call { name, args })
    }
}

/// The value of the module-level assignment to `target`
/// (`TARGET = ...` or `TARGET: Type = ...`).
pub fn assignment(src: &str, target: &str) -> Result<Py, String> {
    let toks = tokenize(src)?;
    let at = (0..toks.len())
        .find(|&i| {
            toks[i] == Tok::Ident(target.to_string())
                && (i == 0 || toks[i - 1] != Tok::Punct('.'))
                && matches!(toks.get(i + 1), Some(Tok::Punct(':' | '=')))
        })
        .ok_or_else(|| format!("no assignment to {target}"))?;
    let eq = (at..toks.len())
        .find(|&i| toks[i] == Tok::Punct('='))
        .ok_or_else(|| format!("{target}: no `=`"))?;
    let mut p = Parser {
        toks: &toks,
        pos: eq + 1,
    };
    p.value().map_err(|e| format!("{target}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"
"""Doc string with "quotes" and a # hash."""
from typing import Dict
# GROUPS = {"commented": 1}
GROUPS: Dict[str, Group] = {
    "grp_x":Group(id="grp_x", full_name='X \'s', is_active=True, start_year=1991,),
}
OTHER = {'a': 'b', 'c': date(1994, 11, 1)}
"#;

    #[test]
    fn parses_typed_and_plain_assignments() {
        let Py::Dict(groups) = assignment(SRC, "GROUPS").unwrap() else {
            panic!("not a dict")
        };
        assert_eq!(groups.len(), 1);
        let Py::Call { name, args } = &groups[0].1 else {
            panic!("not a call")
        };
        assert_eq!(name, "Group");
        assert_eq!(args[1], (Some("full_name".into()), Py::Str("X 's".into())));
        assert_eq!(args[2].1, Py::Bool(true));
        let Py::Dict(other) = assignment(SRC, "OTHER").unwrap() else {
            panic!("not a dict")
        };
        assert_eq!(
            other[1].1,
            Py::Call {
                name: "date".into(),
                args: vec![
                    (None, Py::Int(1994)),
                    (None, Py::Int(11)),
                    (None, Py::Int(1))
                ]
            }
        );
    }

    #[test]
    fn rejects_what_is_not_a_literal() {
        assert!(assignment("X = {k: v for k, v in Y}", "X").is_err());
        assert!(assignment("X = 1", "Y").is_err());
        assert!(assignment("X = 'open", "X").is_err());
    }
}
