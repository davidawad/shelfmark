//! One udcsummary.info tree page: the `d.add(...)` calls its dtree
//! script makes, one per class, read into [`Node`]s.
//!
//! A call is `d.add(id, parent, 'code', 'html', 'code', 'caption', '',
//! '', '', open, leaf)`. Ids are local to the page; the root's parent is
//! `-1`. A root carries its caption only inside `html`.

/// One class as the page lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: i64,
    pub parent: i64,
    pub code: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
enum Arg {
    Int(i64),
    Str(String),
    Bool,
}

/// Every node on a page, in page order.
pub fn parse(page: &str) -> Result<Vec<Node>, String> {
    let mut out = Vec::new();
    for (n, line) in page.lines().enumerate() {
        let Some(call) = line.trim().strip_prefix("d.add(") else {
            continue;
        };
        let at = |e: String| format!("line {}: {e}", n + 1);
        let args = args(call).map_err(at)?;
        out.push(node(&args).map_err(at)?);
    }
    if out.is_empty() {
        return Err("no d.add(...) calls; the page layout has changed".to_string());
    }
    Ok(out)
}

fn node(args: &[Arg]) -> Result<Node, String> {
    let (Some(Arg::Int(id)), Some(Arg::Int(parent)), Some(Arg::Str(code))) =
        (args.first(), args.get(1), args.get(2))
    else {
        return Err("expected d.add(id, parent, 'code', ...)".to_string());
    };
    let (Some(Arg::Str(html)), Some(Arg::Str(caption))) = (args.get(3), args.get(5)) else {
        return Err(format!("`{code}`: expected 'html' and 'caption'"));
    };
    let label = if caption.trim().is_empty() {
        let text = strip_html(html);
        text.strip_prefix(code.as_str())
            .unwrap_or(&text)
            .to_string()
    } else {
        decode(caption)
    };
    Ok(Node {
        id: *id,
        parent: *parent,
        code: code.clone(),
        label: squash(&label),
    })
}

/// The arguments of one call, read up to its closing `)`.
fn args(call: &str) -> Result<Vec<Arg>, String> {
    let mut out = Vec::new();
    let mut chars = call.chars().peekable();
    loop {
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        match chars.peek() {
            Some('\'') => {
                chars.next();
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('\\') => s.push(chars.next().ok_or("unterminated escape")?),
                        Some('\'') => break,
                        Some(c) => s.push(c),
                        None => return Err("unterminated string".to_string()),
                    }
                }
                out.push(Arg::Str(s));
            }
            Some(_) => {
                let mut word = String::new();
                while let Some(&c) = chars.peek().filter(|&&c| c != ',' && c != ')') {
                    word.push(c);
                    chars.next();
                }
                let word = word.trim();
                out.push(match word {
                    "true" | "false" => Arg::Bool,
                    _ => Arg::Int(word.parse().map_err(|_| format!("bad argument `{word}`"))?),
                });
            }
            None => return Err("unterminated call".to_string()),
        }
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        match chars.next() {
            Some(',') => {}
            Some(')') => return Ok(out),
            _ => return Err("expected `,` or `)`".to_string()),
        }
    }
}

/// The text of an HTML fragment: tags dropped, `&nbsp;` as a space.
fn strip_html(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    squash(&decode(&out))
}

/// The few HTML entities the pages use, decoded; `&amp;` last.
fn decode(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// Runs of whitespace as one space, trimmed.
fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_calls_escapes_and_root_captions() {
        let page = "<script>\nd = new dTree('d');\n\
            d.add(0,-1,'0','<span class=\"nodetag\">0</span>&nbsp;&nbsp;GENERALITIES','0','','','','',false,true);\n\
            d.add(1,0,'00','<div class=\"nodetag\">00</div></span>&nbsp;&nbsp;x','00','Writers\\' notes. A &amp; B','','','',false,true);\n\
            </script>";
        let nodes = parse(page).unwrap();
        assert_eq!(
            nodes,
            vec![
                Node {
                    id: 0,
                    parent: -1,
                    code: "0".into(),
                    label: "GENERALITIES".into()
                },
                Node {
                    id: 1,
                    parent: 0,
                    code: "00".into(),
                    label: "Writers' notes. A & B".into()
                },
            ]
        );
    }

    #[test]
    fn header_label_comes_from_html() {
        let nodes = parse("d.add(0,-1,'---','<span class=\"nodetag\"></span>COMMON AUXILIARY NUMBERS','---','','','','',false,true);").unwrap();
        assert_eq!(nodes[0].label, "COMMON AUXILIARY NUMBERS");
    }

    #[test]
    fn rejects_a_page_without_calls_and_a_broken_call() {
        assert!(parse("<html></html>").is_err());
        assert!(parse("d.add(0,-1,'0','x").is_err());
        assert!(parse("d.add(zero,-1,'0','x','0','y','','','',false,true);").is_err());
    }
}
