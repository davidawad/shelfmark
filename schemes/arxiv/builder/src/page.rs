//! The categories listed on <https://arxiv.org/category_taxonomy>, read
//! from its HTML: a group per `<h2 id="accordion-head-grp_...">`, an
//! archive per `<h3>Name<br><span>(id)</span></h3>` and a category per
//! `<h4>code <span>(Name)</span></h4>`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub group: String,
    /// The `<h3>` archive heading the category sits under, if any (a group
    /// that is a single archive has none).
    pub archive: Option<String>,
    pub code: String,
    pub name: String,
}

const GROUP: &str = "id=\"accordion-head-";
const ARCHIVE: &str = "<h3>";
const CATEGORY: &str = "<h4>";

/// Every category on the page, in page order.
pub fn parse(html: &str) -> Result<Vec<Listed>, String> {
    let body = html
        .find("id=\"category_taxonomy_list\"")
        .map(|i| &html[i..])
        .ok_or("no category_taxonomy_list on the page")?;
    let mut out = Vec::new();
    let (mut group, mut archive) = (None::<String>, None::<String>);
    let mut rest = body;
    loop {
        let next = [GROUP, ARCHIVE, CATEGORY]
            .into_iter()
            .filter_map(|m| rest.find(m).map(|i| (i, m)))
            .min();
        let Some((i, marker)) = next else { break };
        rest = &rest[i + marker.len()..];
        match marker {
            GROUP => {
                let end = rest.find('"').ok_or("unterminated group id")?;
                group = Some(rest[..end].to_string());
                archive = None;
            }
            ARCHIVE => {
                let end = rest.find("</h3>").ok_or("unterminated <h3>")?;
                archive = Some(paren(&rest[..end]).ok_or("<h3> without (archive id)")?);
            }
            _ => {
                let end = rest.find("</h4>").ok_or("unterminated <h4>")?;
                let h4 = &rest[..end];
                let (code, tail) = h4.split_once(" <span>").ok_or("<h4> without <span>")?;
                out.push(Listed {
                    group: group.clone().ok_or("category before any group")?,
                    archive: archive.clone(),
                    code: decode(code.trim()),
                    name: paren(tail).ok_or("<h4> without (name)")?,
                });
            }
        }
    }
    if out.is_empty() {
        return Err("no categories on the page".into());
    }
    Ok(out)
}

/// The text inside the last `(...)` of an HTML fragment, decoded.
fn paren(fragment: &str) -> Option<String> {
    let open = fragment.find('(')?;
    let close = fragment.rfind(')')?;
    (open < close).then(|| decode(&fragment[open + 1..close]))
}

/// Decode the HTML entities Jinja's autoescape emits.
fn decode(s: &str) -> String {
    s.replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    const HTML: &str = r#"<h4>Category Name<br><strong>(Category ID)</strong></h4>
<div id="category_taxonomy_list"><h2 class="accordion-head" id="accordion-head-grp_cs">
<h4>cs.AI <span>(Artificial Intelligence)</span></h4><p>Covers (things)</p>
<h2 class="accordion-head" id="accordion-head-grp_physics">
<h3>Astrophysics<br><span>(astro-ph)</span></h3>
<h4>astro-ph.CO <span>(Cosmology &amp; Nongalactic Astrophysics)</span></h4>
<h3>High Energy Physics - Theory<br><span>(hep-th)</span></h3>
<h4>hep-th <span>(High Energy Physics - Theory)</span></h4>"#;

    #[test]
    fn reads_groups_archives_and_categories() {
        let got = parse(HTML).unwrap();
        assert_eq!(got.len(), 3);
        assert_eq!(
            got[0],
            Listed {
                group: "grp_cs".into(),
                archive: None,
                code: "cs.AI".into(),
                name: "Artificial Intelligence".into()
            }
        );
        assert_eq!(got[1].archive.as_deref(), Some("astro-ph"));
        assert_eq!(got[1].name, "Cosmology & Nongalactic Astrophysics");
        assert_eq!(got[2].archive.as_deref(), Some("hep-th"));
    }

    #[test]
    fn an_empty_page_is_an_error() {
        assert!(parse("<html></html>").is_err());
        assert!(parse("<div id=\"category_taxonomy_list\"></div>").is_err());
    }
}
