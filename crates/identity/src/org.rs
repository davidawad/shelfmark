//! The words of an Org document (R2): headline text, body text, list
//! items, table cells and block contents. Like Markdown front matter,
//! `#+KEYWORD:` lines (title, options, block delimiters), property and
//! logbook drawers, planning lines and comments are dropped, and links
//! keep their description rather than their target, so an Org note and
//! its Markdown conversion keep the same words.

/// Plain text from Org markup, one output line per kept input line.
pub fn org_to_text(org: &str) -> String {
    let mut out = String::with_capacity(org.len());
    let mut in_drawer = false;
    let mut in_comment_block = false;
    let lines: Vec<&str> = org.lines().collect();
    for (i, &line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if in_comment_block {
            in_comment_block = !lower.starts_with("#+end_comment");
            continue;
        }
        if in_drawer {
            in_drawer = !lower.eq_ignore_ascii_case(":end:");
            continue;
        }
        if lower.starts_with("#+begin_comment") {
            in_comment_block = true;
            continue;
        }
        // As in Org, a `:NAME:` line opens a drawer only when an `:END:`
        // closes it; otherwise it is text.
        if is_drawer_start(trimmed)
            && lines[i + 1..]
                .iter()
                .any(|l| l.trim().eq_ignore_ascii_case(":end:"))
        {
            in_drawer = true;
            continue;
        }
        if trimmed.starts_with("#+") || trimmed == "#" || trimmed.starts_with("# ") {
            continue;
        }
        if is_planning(trimmed) {
            continue;
        }
        let body = headline_text(line).unwrap_or(line);
        push_without_links(&mut out, body);
        out.push('\n');
    }
    out
}

/// `:PROPERTIES:`, `:LOGBOOK:` or any other `:NAME:` drawer opener.
fn is_drawer_start(trimmed: &str) -> bool {
    let Some(name) = trimmed.strip_prefix(':').and_then(|s| s.strip_suffix(':')) else {
        return false;
    };
    !name.is_empty()
        && !name.eq_ignore_ascii_case("end")
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

fn is_planning(trimmed: &str) -> bool {
    ["SCHEDULED:", "DEADLINE:", "CLOSED:"]
        .iter()
        .any(|k| trimmed.starts_with(k))
}

/// A headline's title: stars, TODO keyword, priority cookie and trailing
/// `:tags:` removed. `None` when the line is not a headline.
fn headline_text(line: &str) -> Option<&str> {
    let rest = line.trim_start_matches('*');
    if rest.len() == line.len() || !rest.starts_with(' ') {
        return None;
    }
    let mut rest = rest.trim();
    for kw in ["TODO ", "DONE "] {
        if let Some(r) = rest.strip_prefix(kw) {
            rest = r.trim_start();
        }
    }
    if rest.starts_with("[#") && rest.as_bytes().get(3) == Some(&b']') {
        rest = rest[4..].trim_start();
    }
    if let Some((title, tags)) = rest.rsplit_once(char::is_whitespace) {
        if tags.len() > 2 && tags.starts_with(':') && tags.ends_with(':') {
            rest = title.trim_end();
        }
    }
    Some(rest)
}

/// Copy `text`, replacing `[[target][description]]` with the description
/// and a bare `[[target]]` with its target, as Org displays it.
fn push_without_links(out: &mut String, text: &str) {
    let mut rest = text;
    while let Some(start) = rest.find("[[") {
        let Some(len) = rest[start..].find("]]") else {
            break;
        };
        out.push_str(&rest[..start]);
        let link = &rest[start + 2..start + len];
        out.push_str(link.split_once("][").map_or(link, |(_, desc)| desc));
        rest = &rest[start + len + 2..];
    }
    out.push_str(rest);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headline_parts_are_stripped() {
        assert_eq!(
            headline_text("** TODO [#A] Compilers :cs:book:"),
            Some("Compilers")
        );
        assert_eq!(headline_text("* Plain"), Some("Plain"));
        assert_eq!(headline_text("*bold* text"), None);
    }

    #[test]
    fn an_unclosed_drawer_is_text() {
        let text = org_to_text("* A\n:note:\nreal text\n* B\n");
        assert_eq!(text, "A\n:note:\nreal text\nB\n");
    }

    #[test]
    fn links_keep_descriptions_not_targets() {
        let mut out = String::new();
        push_without_links(
            &mut out,
            "see [[https://x.org/a][the book]] and [[file:b.org]] end",
        );
        assert_eq!(out, "see the book and file:b.org end");
    }
}
