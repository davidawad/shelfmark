//! The IPC scheme master file (`EN_ipc_scheme_<edition>.xml`). Entries
//! nest by hierarchy: section, class, subclass, main group, then
//! subgroups by dot level, so each class's parent is the entry it sits in.

use roxmltree::{Document, Node};

use crate::symbol::{compact, version};

/// One class read from the master file, in document order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub code: String,
    pub label: String,
    pub parent: Option<String>,
    /// The entry's references to other places (`entryReference`).
    pub references: Vec<String>,
    /// Part of an indexing scheme (`entryType="I"`), not the main scheme.
    pub indexing: bool,
}

/// A guidance heading over the main groups `start..=end` of a subclass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub text: String,
    pub start: String,
    pub end: String,
}

/// Everything the build takes from the master file.
#[derive(Debug, Default)]
pub struct Parsed {
    pub version: String,
    pub entries: Vec<Entry>,
    pub headings: Vec<Heading>,
    /// Non-class entries skipped: notes, catchword-index blocks, the
    /// section's class-title list.
    pub skipped: usize,
    /// Images in titles, written as `[image]` in labels.
    pub images: usize,
}

/// Parse the master file.
pub fn parse(xml: &str) -> Result<Parsed, String> {
    let doc = Document::parse(xml).map_err(|e| format!("not XML: {e}"))?;
    let root = doc.root_element();
    if root.tag_name().name() != "IPCScheme" {
        return Err(format!(
            "root is <{}>, not <IPCScheme>",
            root.tag_name().name()
        ));
    }
    let edition = root.attribute("edition").unwrap_or_default();
    let mut parsed = Parsed {
        version: version(edition).ok_or_else(|| format!("bad edition `{edition}`"))?,
        ..Parsed::default()
    };
    walk(root, None, &mut parsed)?;
    Ok(parsed)
}

fn elements<'a, 'i>(node: Node<'a, 'i>, name: &'a str) -> impl Iterator<Item = Node<'a, 'i>> {
    node.children()
        .filter(move |c| c.is_element() && c.tag_name().name() == name)
}

fn walk(node: Node, parent: Option<&str>, out: &mut Parsed) -> Result<(), String> {
    for e in elements(node, "ipcEntry") {
        let kind = e.attribute("kind").unwrap_or_default();
        let symbol = e.attribute("symbol").unwrap_or_default();
        let class = matches!(kind, "s" | "c" | "u" | "m")
            || (kind.len() == 1 && kind.as_bytes()[0].is_ascii_digit());
        if kind == "g" {
            let text = title(e, out).0;
            let end = e.attribute("endSymbol").unwrap_or(symbol);
            match (compact(symbol), compact(end)) {
                (Some(start), Some(end)) if !text.is_empty() => {
                    out.headings.push(Heading { text, start, end })
                }
                _ => return Err(format!("guidance heading `{symbol}` is unreadable")),
            }
            continue;
        }
        if !class {
            out.skipped += 1;
            continue;
        }
        let code = compact(symbol).ok_or_else(|| format!("bad symbol `{symbol}` ({kind})"))?;
        let (label, references) = title(e, out);
        if label.is_empty() {
            return Err(format!("`{code}` has no title"));
        }
        out.entries.push(Entry {
            code: code.clone(),
            label,
            parent: parent.map(str::to_string),
            references,
            indexing: e.attribute("entryType") == Some("I"),
        });
        walk(e, Some(&code), out)?;
    }
    Ok(())
}

/// The title of an entry (its title parts joined by `; `) and its
/// references, whitespace collapsed.
fn title(entry: Node, out: &mut Parsed) -> (String, Vec<String>) {
    let (mut parts, mut refs) = (Vec::new(), Vec::new());
    let body = elements(entry, "textBody").next();
    let title = body.and_then(|b| elements(b, "title").next());
    for part in title.into_iter().flat_map(|t| elements(t, "titlePart")) {
        let mut text = String::new();
        for child in part.children().filter(Node::is_element) {
            match child.tag_name().name() {
                "entryReference" => {
                    let mut r = String::new();
                    collect(child, &mut r, out);
                    push_clean(&mut refs, &r);
                }
                _ => collect(child, &mut text, out),
            }
        }
        push_clean(&mut parts, &text);
    }
    (parts.join("; "), refs)
}

fn push_clean(into: &mut Vec<String>, text: &str) {
    let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let clean = clean.trim_matches(|c: char| c == ',' || c == ';' || c.is_whitespace());
    if !clean.is_empty() {
        into.push(clean.to_string());
    }
}

/// Text under `node`, with symbol references written in compact form.
fn collect(node: Node, into: &mut String, out: &mut Parsed) {
    for c in node.children() {
        if c.is_text() {
            into.push_str(c.text().unwrap_or_default());
            continue;
        }
        let sym = |a: &str| {
            c.attribute(a)
                .map(|s| compact(s).unwrap_or_else(|| s.to_string()))
        };
        match c.tag_name().name() {
            "sref" => into.push_str(&sym("ref").unwrap_or_default()),
            "mref" => {
                into.push_str(&sym("ref").unwrap_or_default());
                into.push('-');
                into.push_str(&sym("endRef").unwrap_or_default());
            }
            "img" => {
                out.images += 1;
                into.push_str(" [image]");
            }
            _ => collect(c, into, out),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<?xml version="1.0"?>
<IPCScheme xmlns="http://www.wipo.int/classifications/ipc/masterfiles" edition="20260101" lang="EN">
<ipcEntry kind="s" symbol="H" entryType="K"><textBody><title><titlePart><text>ELECTRICITY</text></titlePart></title></textBody>
<ipcEntry kind="t" symbol="H01" endSymbol="H01" entryType="K"><textBody><title><titlePart><text>BASIC</text></titlePart></title></textBody></ipcEntry>
<ipcEntry kind="c" symbol="H01" entryType="K"><textBody><title><titlePart><text>ELECTRIC ELEMENTS</text></titlePart></title></textBody>
<ipcEntry kind="u" symbol="H01B" entryType="K"><textBody><title><titlePart><text>CABLES</text></titlePart><titlePart><text>CONDUCTORS </text><entryReference>waveguides <sref ref="H01P"/></entryReference></titlePart></title></textBody>
<ipcEntry kind="n" symbol="H01B" entryType="K"><textBody><note><noteParagraph><text>A note</text></noteParagraph></note></textBody></ipcEntry>
<ipcEntry kind="g" symbol="H01B0001000000" endSymbol="H01B0003000000" entryType="K"><textBody><title><titlePart><text>Materials</text></titlePart></title></textBody></ipcEntry>
<ipcEntry kind="m" symbol="H01B0001000000" entryType="K"><textBody><title><titlePart><text>Conductors containing <img src="f.gif"/> groups, see <mref ref="H01B0003000000" endRef="H01B0005000000"/></text></titlePart></title></textBody>
<ipcEntry kind="1" symbol="H01B0001020000" entryType="K"><textBody><title><titlePart><text>mainly
  consisting of metals</text></titlePart></title></textBody></ipcEntry>
</ipcEntry>
<ipcEntry kind="m" symbol="H01B0003000000" entryType="I"><textBody><title><titlePart><text>Insulators</text></titlePart></title></textBody></ipcEntry>
</ipcEntry></ipcEntry></ipcEntry></IPCScheme>"#;

    #[test]
    fn reads_hierarchy_titles_and_references() {
        let p = parse(XML).unwrap();
        assert_eq!(p.version, "2026.01");
        let codes: Vec<(&str, Option<&str>)> = p
            .entries
            .iter()
            .map(|e| (e.code.as_str(), e.parent.as_deref()))
            .collect();
        assert_eq!(
            codes,
            [
                ("H", None),
                ("H01", Some("H")),
                ("H01B", Some("H01")),
                ("H01B1/00", Some("H01B")),
                ("H01B1/02", Some("H01B1/00")),
                ("H01B3/00", Some("H01B")),
            ]
        );
        assert_eq!(p.entries[2].label, "CABLES; CONDUCTORS");
        assert_eq!(p.entries[2].references, ["waveguides H01P"]);
        assert_eq!(
            p.entries[3].label,
            "Conductors containing [image] groups, see H01B3/00-H01B5/00"
        );
        assert_eq!(p.entries[4].label, "mainly consisting of metals");
        assert!(p.entries[5].indexing && !p.entries[4].indexing);
        assert_eq!(
            p.headings,
            [Heading {
                text: "Materials".into(),
                start: "H01B1/00".into(),
                end: "H01B3/00".into()
            }]
        );
        assert_eq!((p.skipped, p.images), (2, 1));
    }

    #[test]
    fn rejects_other_documents() {
        assert!(parse("<IPCCatchwordIndex/>").is_err());
        assert!(parse("<IPCScheme edition=\"2026\"/>").is_err());
    }
}
