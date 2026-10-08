//! Label texts: what the tier embeds for each class, derived from the
//! scheme alone (labels and hierarchy, no editorial scope strings).
//!
//! - A level-0 class: its label, then `. Includes: ` and its children's
//!   labels joined by `; ` (when it has children).
//! - A level-1 class: `{parent label} — {label}`, then `. Includes: ` and up
//!   to [`MAX_INCLUDED`] of its children's labels joined by `; ` (when it
//!   has children).
//!
//! Children are taken in the index's order, so the texts, and with them
//! the label-cache key, are deterministic for one compiled scheme.

/// Most child labels a level-1 class text lists.
pub const MAX_INCLUDED: usize = 12;

/// Text for a level-0 class with label `label` and children `children`.
pub fn top_text(label: &str, children: &[&str]) -> String {
    with_includes(label.to_string(), children)
}

/// Text for a level-1 class `label` under `parent`, with children
/// `children` (only the first [`MAX_INCLUDED`] are listed).
pub fn sub_text(parent: &str, label: &str, children: &[&str]) -> String {
    let shown = &children[..children.len().min(MAX_INCLUDED)];
    with_includes(format!("{parent} — {label}"), shown)
}

fn with_includes(mut text: String, children: &[&str]) -> String {
    if !children.is_empty() {
        text.push_str(". Includes: ");
        text.push_str(&children.join("; "));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_lists_every_child() {
        assert_eq!(top_text("Law", &[]), "Law");
        assert_eq!(
            top_text("Science", &["Mathematics", "Astronomy", "Physics"]),
            "Science. Includes: Mathematics; Astronomy; Physics"
        );
        let many: Vec<String> = (0..20).map(|i| format!("c{i}")).collect();
        let refs: Vec<&str> = many.iter().map(String::as_str).collect();
        assert_eq!(top_text("X", &refs).matches(';').count(), 19);
    }

    #[test]
    fn level_one_names_parent_and_caps_children() {
        assert_eq!(
            sub_text("Science", "Mathematics", &[]),
            "Science — Mathematics"
        );
        assert_eq!(
            sub_text("Science", "Mathematics", &["Algebra", "Geometry"]),
            "Science — Mathematics. Includes: Algebra; Geometry"
        );
        let many: Vec<String> = (0..20).map(|i| format!("c{i}")).collect();
        let refs: Vec<&str> = many.iter().map(String::as_str).collect();
        let t = sub_text("P", "L", &refs);
        assert!(t.ends_with("c10; c11"), "{t}");
        assert_eq!(t.matches(';').count(), MAX_INCLUDED - 1);
    }
}
