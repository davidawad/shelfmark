//! Plain text from HTML: drop markup, `<head>`, scripts, styles and
//! comments; decode entities. Block-level tags become a line break so
//! adjacent paragraphs never fuse into one word; inline tags vanish so
//! `fo<b>o</b>` stays one word.

/// Elements whose content is not body text.
const SKIP: &[&str] = &["head", "script", "style", "template", "noscript", "svg"];

/// Elements that sit inside a word without separating it.
const INLINE: &[&str] = &[
    "a", "abbr", "b", "bdi", "bdo", "cite", "code", "data", "dfn", "em", "font", "i", "kbd",
    "mark", "q", "s", "samp", "small", "span", "strike", "strong", "sub", "sup", "time", "tt", "u",
    "var", "wbr",
];

/// Extract the readable text of an HTML document or fragment.
pub fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut rest = html;
    while let Some(lt) = rest.find('<') {
        push_decoded(&mut out, &rest[..lt]);
        rest = &rest[lt..];
        if let Some(after) = rest.strip_prefix("<!--") {
            rest = after.find("-->").map_or("", |i| &after[i + 3..]);
            continue;
        }
        if let Some(after) = rest.strip_prefix("<![CDATA[") {
            let end = after.find("]]>").unwrap_or(after.len());
            out.push_str(&after[..end]);
            rest = after.get(end + 3..).unwrap_or("");
            continue;
        }
        let starts_tag = rest[1..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!' || c == '?');
        if !starts_tag {
            // `a < b` is text, not a tag.
            out.push('<');
            rest = &rest[1..];
            continue;
        }
        let Some(gt) = tag_end(rest) else {
            // A stray `<` with no closing `>` is text.
            push_decoded(&mut out, rest);
            return out;
        };
        let tag = &rest[1..gt];
        rest = &rest[gt + 1..];
        let name = tag_name(tag);
        if name.is_empty() {
            continue;
        }
        if !tag.starts_with('/') && SKIP.contains(&name.as_str()) && !tag.ends_with('/') {
            rest = skip_element(rest, &name);
            out.push('\n');
        } else if !INLINE.contains(&name.as_str()) {
            out.push('\n');
        }
    }
    push_decoded(&mut out, rest);
    out
}

/// Index of the `>` closing the tag at the start of `s`, honouring quoted
/// attribute values.
fn tag_end(s: &str) -> Option<usize> {
    let mut quote = None;
    for (i, b) in s.bytes().enumerate().skip(1) {
        match (quote, b) {
            (None, b'"' | b'\'') => quote = Some(b),
            (Some(q), _) if q == b => quote = None,
            (None, b'>') => return Some(i),
            _ => {}
        }
    }
    None
}

/// Lowercased element name of a tag body (`/p`, `br/`, `!doctype html`).
fn tag_name(tag: &str) -> String {
    tag.trim_start_matches('/')
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Skip past the matching `</name>` (case-insensitive), or to the end.
fn skip_element<'a>(rest: &'a str, name: &str) -> &'a str {
    let close = format!("</{name}");
    let lower = rest.to_ascii_lowercase();
    match lower.find(&close) {
        Some(i) => match rest[i..].find('>') {
            Some(j) => &rest[i + j + 1..],
            None => "",
        },
        None => "",
    }
}

fn push_decoded(out: &mut String, text: &str) {
    let mut rest = text;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let entity = rest[1..]
            .find(';')
            .filter(|&i| i <= 32)
            .and_then(|i| decode_entity(&rest[1..=i]).map(|c| (c, i + 2)));
        match entity {
            Some((c, len)) => {
                out.push(c);
                rest = &rest[len..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
}

/// Decode an entity body (between `&` and `;`). Unknown named entities
/// stay literal.
fn decode_entity(body: &str) -> Option<char> {
    if let Some(num) = body.strip_prefix('#') {
        let code = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => num.parse().ok()?,
        };
        return char::from_u32(code);
    }
    Some(match body {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        "shy" => '\u{ad}',
        "ndash" => '\u{2013}',
        "mdash" => '\u{2014}',
        "lsquo" => '\u{2018}',
        "rsquo" => '\u{2019}',
        "ldquo" => '\u{201c}',
        "rdquo" => '\u{201d}',
        "hellip" => '\u{2026}',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::html_to_text;

    #[test]
    fn strips_markup_and_skipped_elements() {
        let html = "<!DOCTYPE html><html><head><title>T</title></head><body>\
                    <script>var x = '<p>';</script><h1>Hello</h1><p>wor<b>ld</b> &amp; \
                    caf&#233; &#x41;<!-- gone --></p></body></html>";
        let text = html_to_text(html);
        assert!(!text.contains('T') && !text.contains("var"), "{text:?}");
        assert!(text.contains("Hello\n"), "{text:?}");
        assert!(text.contains("world & café A"), "{text:?}");
    }

    #[test]
    fn stray_angle_and_unknown_entities_stay_text() {
        assert_eq!(html_to_text("a < b &bogus; c"), "a < b &bogus; c");
        assert_eq!(html_to_text("1 < 2 > 0"), "1 < 2 > 0");
        assert_eq!(html_to_text("<a href=\"x>y\">link</a>"), "link");
    }
}
