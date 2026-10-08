//! The IPC catchword index master file
//! (`EN_ipc_catchwordindex_<edition>.xml`): alphabetical catchwords and
//! the places in the scheme each points at. Nested entries carry their
//! full indication ("building aspects of ABATTOIRS"), so each is one term.

use roxmltree::{Document, Node};

use crate::symbol::compact;

/// What one catchword entry points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Code(String),
    /// Every sibling from `start` to `end` (`mref`).
    Span(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catchword {
    /// The entry's `id` attribute, e.g. `CW259852291`.
    pub id: String,
    pub term: String,
    pub targets: Vec<Target>,
}

/// Every catchword entry that points somewhere, in document order.
pub fn parse(xml: &str) -> Result<Vec<Catchword>, String> {
    let doc = Document::parse(xml).map_err(|e| format!("not XML: {e}"))?;
    let root = doc.root_element();
    if root.tag_name().name() != "IPCCatchwordIndex" {
        return Err(format!(
            "root is <{}>, not <IPCCatchwordIndex>",
            root.tag_name().name()
        ));
    }
    let mut out = Vec::new();
    for e in root.descendants().filter(|n| named(n, "CWEntry")) {
        let term = e
            .children()
            .find(|n| named(n, "CWIndication"))
            .map(|n| {
                let text: String = n
                    .descendants()
                    .filter(Node::is_text)
                    .filter_map(|t| t.text())
                    .collect();
                text.split_whitespace().collect::<Vec<_>>().join(" ")
            })
            .unwrap_or_default();
        let mut targets = Vec::new();
        for refs in e.children().filter(|n| named(n, "CWReferences")) {
            for r in refs.children().filter(Node::is_element) {
                let sym = |a: &str| r.attribute(a).and_then(compact);
                match (r.tag_name().name(), sym("ref"), sym("endRef")) {
                    ("sref", Some(code), _) => targets.push(Target::Code(code)),
                    ("mref", Some(start), Some(end)) => targets.push(Target::Span(start, end)),
                    (tag, ..) => return Err(format!("unreadable <{tag}> under `{term}`")),
                }
            }
        }
        if term.is_empty() || targets.is_empty() {
            continue;
        }
        let id = e.attribute("id").unwrap_or_default().to_string();
        out.push(Catchword { id, term, targets });
    }
    Ok(out)
}

fn named(n: &Node, name: &str) -> bool {
    n.is_element() && n.tag_name().name() == name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nested_entries_and_spans() {
        let xml = r#"<IPCCatchwordIndex xmlns="http://www.wipo.int/classifications/ipc/masterfiles">
<CWEntry id="CW1"><CWIndication>ABACUSES</CWIndication><CWReferences><sref ref="G06C0001000000"/></CWReferences></CWEntry>
<CWEntry id="CW2"><CWIndication>ABATTOIRS</CWIndication>
  <CWEntry id="CW3"><CWIndication>equipment for  ABATTOIRS</CWIndication><CWReferences><sref ref="A22B"/><mref ref="B41L0045000000" endRef="B41L0049000000"/></CWReferences></CWEntry></CWEntry>
<CWEntry id="CW4"><CWIndication>VITAMIN B<sub>1</sub></CWIndication><CWReferences><sref ref="C07D0415000000"/></CWReferences></CWEntry>
</IPCCatchwordIndex>"#;
        let cw = parse(xml).unwrap();
        let terms: Vec<&str> = cw.iter().map(|c| c.term.as_str()).collect();
        assert_eq!(terms, ["ABACUSES", "equipment for ABATTOIRS", "VITAMIN B1"]);
        assert_eq!(cw[0].targets, [Target::Code("G06C1/00".into())]);
        assert_eq!(
            cw[1].targets,
            [
                Target::Code("A22B".into()),
                Target::Span("B41L45/00".into(), "B41L49/00".into())
            ]
        );
        assert_eq!(cw[1].id, "CW3");
    }
}
