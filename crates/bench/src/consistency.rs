//! Inter-classifier consistency: agreement between two placements of the
//! same items in the same scheme (e.g. source labels against shelfmark).
use crate::catalogue::Scores;
use crate::partition::{compare, kappa};
use shelfmark_core::scheme::SchemeDoc;
use std::collections::BTreeMap;

fn put(s: &mut Scores, key: &str, value: f64) {
    s.insert(key.to_owned(), value);
}

/// Exact and per-depth agreement of `b` with `a` on items both place,
/// Cohen's kappa at depth 1 and on exact classes, and ARI.
pub fn consistency(scheme: &SchemeDoc, a: &[Option<String>], b: &[Option<String>]) -> Scores {
    let parent: BTreeMap<&str, Option<&str>> = scheme
        .classes
        .iter()
        .map(|c| (c.code.as_str(), c.parent.as_deref()))
        .collect();
    let chain = |code: &str| {
        let mut out = vec![code.to_owned()];
        let mut cur = code;
        while let Some(Some(p)) = parent.get(cur) {
            out.push((*p).to_owned());
            cur = p;
        }
        out.reverse();
        out
    };
    let mut ids = BTreeMap::<String, usize>::new();
    let mut id = |x: &str| {
        let next = ids.len();
        *ids.entry(x.to_owned()).or_insert(next)
    };
    let mut cols: [[Vec<Option<usize>>; 2]; 3] = Default::default();
    for (x, y) in a.iter().zip(b) {
        let (Some(x), Some(y)) = (x, y) else { continue };
        let (cx, cy) = (chain(x), chain(y));
        for (d, pair) in cols.iter_mut().enumerate() {
            // Depth 0 = exact class; 1 and 2 = ancestors at that depth.
            let pick = |c: &Vec<String>| {
                if d == 0 {
                    c.last().cloned()
                } else {
                    c.get(d - 1).cloned()
                }
            };
            pair[0].push(pick(&cx).map(|v| id(&v)));
            pair[1].push(pick(&cy).map(|v| id(&v)));
        }
    }
    let mut s = Scores::new();
    let (k0, p0, n) = kappa(&cols[0][0], &cols[0][1]);
    let (k1, p1, _) = kappa(&cols[1][0], &cols[1][1]);
    let (_, p2, _) = kappa(&cols[2][0], &cols[2][1]);
    put(&mut s, "consistency.items", n as f64);
    put(&mut s, "consistency.exact", p0);
    put(&mut s, "consistency.kappa_exact", k0);
    put(&mut s, "consistency.depth1", p1);
    put(&mut s, "consistency.kappa_depth1", k1);
    put(&mut s, "consistency.depth2", p2);
    put(
        &mut s,
        "consistency.ari",
        compare(&cols[0][0], &cols[0][1]).ari,
    );
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use shelfmark_core::scheme::ClassDef;

    fn scheme() -> SchemeDoc {
        let class = |code: &str, parent: Option<&str>| ClassDef {
            code: code.into(),
            label: code.into(),
            parent: parent.map(Into::into),
            range: None,
            vocab: vec![],
            note: None,
        };
        SchemeDoc {
            schema: None,
            name: "toy".into(),
            version: "1".into(),
            title: "toy".into(),
            source: None,
            code_grammar: ".*".into(),
            classes: vec![
                class("A", None),
                class("A1", Some("A")),
                class("A2", Some("A")),
                class("B", None),
            ],
        }
    }

    #[test]
    fn consistency_hand_checked() {
        let code = |c: &str| Some(c.to_owned());
        let a = [code("A1"), code("A1"), code("B"), None];
        let b = [code("A1"), code("A2"), code("B"), code("B")];
        let s = consistency(&scheme(), &a, &b);
        assert_eq!(s["consistency.items"], 3.0);
        assert!((s["consistency.exact"] - 2.0 / 3.0).abs() < 1e-12);
        assert_eq!(s["consistency.depth1"], 1.0);
        assert_eq!(s["consistency.kappa_depth1"], 1.0);
        // Depth 2 compares only items whose classes both reach depth 2: A1 vs A1, A1 vs A2.
        assert_eq!(s["consistency.depth2"], 0.5);
    }
}
