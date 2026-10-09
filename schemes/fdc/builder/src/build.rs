//! `fdc.txt` to a shelfmark scheme.
//!
//! The file is one entry per line, `NUMBER TERMS` or `RANGE TERMS`, with
//! `#` comment lines. Its indentation is decoration and is not trusted:
//! the hierarchy comes from the decimal numbers themselves (a unit sits
//! under its tens class, a tens class under its hundreds class, a decimal
//! under the number one digit shorter) and from ranges, which are classes
//! that group the numbers inside them (`017-019 "Library catalogs"`).
//! `+` separates two subjects sharing a number, `|` two ways of saying one
//! subject; a quoted term is an authorized LCSH heading.

use std::collections::BTreeSet;

use shelfmark_core::scheme::{ClassDef, CodeRange, SchemeDoc, Source, VocabLink};

use crate::sources::{Locked, REPO};

/// Vocabulary source for a term the file quotes as an authorized LCSH heading.
pub const LCSH_TERM: &str = "fdc-lcsh";
/// Vocabulary source for any other term.
pub const PLAIN_TERM: &str = "fdc";

pub const CODE_GRAMMAR: &str = r"[0-9]{3}(\.[0-9]+)?(-[0-9]{3}(\.[0-9]+)?)?";

/// Upstream mistakes the build corrects, as (code as written, start of the
/// terms, corrected code). A correction that no longer applies fails the
/// build, so a fixed upstream is noticed.
const ERRATA: [(&str, &str, &str); 3] = [
    ("975", "\"Gulf States\"", "976"),
    ("855", "\"Speeches, addresses, etc., Spanish\"", "865"),
    ("857", "\"Spanish wit and humor\"", "867"),
];

/// A decimal number: integer part and fraction digits without trailing
/// zeros. The derived order is shelf order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Num {
    int: u32,
    frac: String,
}

impl Num {
    fn parse(s: &str) -> Result<Num, String> {
        let bad = || format!("`{s}` is not an FDC number");
        let (int, frac) = s.split_once('.').unwrap_or((s, ""));
        let digits = |t: &str| t.chars().all(|c| c.is_ascii_digit());
        if int.len() != 3 || !digits(int) || !digits(frac) || (s.contains('.') && frac.is_empty()) {
            return Err(bad());
        }
        Ok(Num {
            int: int.parse().map_err(|_| bad())?,
            frac: frac.trim_end_matches('0').to_string(),
        })
    }

    fn text(&self) -> String {
        if self.frac.is_empty() {
            format!("{:03}", self.int)
        } else {
            format!("{:03}.{}", self.int, self.frac)
        }
    }

    /// `self` is `other` or a deeper decimal of it (`962.4` under `962`).
    fn under(&self, other: &Num) -> bool {
        self.int == other.int && self.frac.starts_with(&other.frac)
    }

    /// The numbers above this one, nearest first: shorter decimals, then
    /// the tens class, then the hundreds class.
    fn ancestors(&self) -> Vec<Num> {
        let mut out = Vec::new();
        let mut cur = self.clone();
        loop {
            cur = if !cur.frac.is_empty() {
                let mut f = cur.frac.clone();
                f.pop();
                Num {
                    int: cur.int,
                    frac: f.trim_end_matches('0').to_string(),
                }
            } else if !cur.int.is_multiple_of(10) {
                Num {
                    int: cur.int / 10 * 10,
                    frac: String::new(),
                }
            } else if !cur.int.is_multiple_of(100) {
                Num {
                    int: cur.int / 100 * 100,
                    frac: String::new(),
                }
            } else {
                return out;
            };
            out.push(cur.clone());
        }
    }
}

/// One numbered line of `fdc.txt`.
#[derive(Debug, Clone)]
pub struct Entry {
    line: usize,
    /// The code as written, corrected for errata; `a-b` for a range.
    code: String,
    start: Num,
    /// The range's end as written, widened where the file's end is a tens
    /// class (see [`widen`]).
    end: Option<Num>,
    terms: String,
    note: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub classes: usize,
    pub roots: usize,
    pub ranges: usize,
    pub max_depth: usize,
    pub mean_depth: f64,
    pub depth_counts: Vec<usize>,
    pub internal: usize,
    pub max_branch: usize,
    pub max_branch_code: String,
    pub mean_branch: f64,
    pub vocab_terms: usize,
    pub lcsh_terms: usize,
    pub errata: Vec<String>,
    pub widened: Vec<String>,
    pub merged: Vec<String>,
}

/// Decode the HTML entities the file uses (it says so in its header).
fn decode(s: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let end = tail
            .find(';')
            .ok_or_else(|| format!("unterminated entity in `{s}`"))?;
        out.push_str(match &tail[..=end] {
            "&eacute;" => "é",
            "&ccedil;" => "ç",
            other => return Err(format!("unknown entity {other} in `{s}`")),
        });
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// A range whose end is a tens class (`940-990`) stands for that whole
/// class, since the entries 991-999 are listed inside it; widen the end to
/// the last number of the class.
fn widen(end: Num) -> Num {
    if end.frac.is_empty() && end.int.is_multiple_of(10) && !end.int.is_multiple_of(100) {
        Num {
            int: end.int + 9,
            frac: String::new(),
        }
    } else {
        end
    }
}

/// Parse `fdc.txt` into entries, applying the errata.
pub fn parse(text: &str, stats: &mut Stats) -> Result<Vec<Entry>, String> {
    parse_with(text, stats, &ERRATA)
}

fn parse_with(
    text: &str,
    stats: &mut Stats,
    errata: &[(&'static str, &'static str, &'static str)],
) -> Result<Vec<Entry>, String> {
    let mut errata_left: Vec<_> = errata.to_vec();
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = |m: String| format!("fdc.txt:{}: {m}", i + 1);
        let (head, terms) = line
            .split_once(char::is_whitespace)
            .ok_or_else(|| at(format!("no terms after `{line}`")))?;
        let terms = terms.trim().to_string();
        let mut note = None;
        let head = match errata_left
            .iter()
            .position(|(c, t, _)| *c == head && terms.starts_with(t))
        {
            Some(k) => {
                let (c, _, fixed) = errata_left.remove(k);
                stats
                    .errata
                    .push(format!("line {}: `{c}` {terms} is `{fixed}`", i + 1));
                note = Some(format!(
                    "fdc.txt numbers this {c}, a repeat of the number above; corrected to {fixed}."
                ));
                fixed
            }
            None => head,
        };
        let (start, end, code) = match head.split_once('-') {
            None => (Num::parse(head).map_err(at)?, None, head.to_string()),
            Some((a, b)) => {
                let end = Num::parse(b).map_err(at)?;
                let wide = widen(end.clone());
                if wide != end {
                    stats
                        .widened
                        .push(format!("{head} (ends at {})", wide.text()));
                }
                (Num::parse(a).map_err(at)?, Some(wide), head.to_string())
            }
        };
        let terms = decode(&terms).map_err(at)?;
        if let Some(prev) = out.iter_mut().find(|p: &&mut Entry| p.code == code) {
            // The same number listed twice is two subjects sharing it.
            prev.terms = format!("{} + {terms}", prev.terms);
            stats
                .merged
                .push(format!("line {}: `{code}` repeats an earlier line", i + 1));
            continue;
        }
        out.push(Entry {
            line: i + 1,
            code,
            start,
            end,
            terms,
            note,
        });
    }
    if let Some((c, t, _)) = errata_left.first() {
        return Err(format!(
            "erratum for `{c} {t}` no longer applies; review it"
        ));
    }
    Ok(out)
}

/// Split `s` at `sep` outside quotes and parentheses.
fn split_top(s: &str, sep: char) -> Vec<&str> {
    let (mut parts, mut depth, mut quoted, mut from) = (Vec::new(), 0i32, false, 0);
    for (i, c) in s.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            c if c == sep && !quoted && depth == 0 => {
                parts.push(&s[from..i]);
                from = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[from..]);
    parts.into_iter().map(str::trim).collect()
}

/// `(a + b)` to `a + b` when the parentheses wrap the whole text.
fn unwrap_parens(s: &str) -> &str {
    let Some(inner) = s.strip_prefix('(').and_then(|r| r.strip_suffix(')')) else {
        return s;
    };
    let balanced = inner
        .chars()
        .try_fold(0i32, |d, c| match c {
            '(' => Some(d + 1),
            ')' if d == 0 => None,
            ')' => Some(d - 1),
            _ => Some(d),
        })
        .is_some_and(|d| d == 0);
    if balanced {
        inner.trim()
    } else {
        s
    }
}

/// One subject term and whether the file marks it as an LCSH heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub text: String,
    pub lcsh: bool,
}

fn term(s: &str) -> Term {
    let lcsh =
        s.len() > 1 && s.starts_with('"') && s.ends_with('"') && !s[1..s.len() - 1].contains('"');
    let text = s.replace('"', "");
    Term {
        text: text.split_whitespace().collect::<Vec<_>>().join(" "),
        lcsh,
    }
}

/// The terms of an entry, by alternative (`|`) then subject (`+`).
pub fn terms(s: &str) -> Vec<Vec<Term>> {
    split_top(s, '|')
        .into_iter()
        .map(|alt| {
            split_top(unwrap_parens(alt), '+')
                .into_iter()
                .filter(|t| !t.is_empty())
                .map(term)
                .collect()
        })
        .collect()
}

/// The class label: the first alternative's subjects joined with `; `.
fn label(alts: &[Vec<Term>]) -> String {
    alts.first()
        .map(|a| {
            a.iter()
                .map(|t| t.text.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default()
}

/// Whether `d` is an ancestor of `e` by number or by range.
fn contains(d: &Entry, e: &Entry) -> bool {
    let e_end = e.end.as_ref().unwrap_or(&e.start);
    match &d.end {
        None => {
            if d.start == e.start {
                // A plain number heads a range that starts at it, if it is listed first.
                e.end.is_some() && d.line < e.line
            } else {
                e.start.ancestors().contains(&d.start)
            }
        }
        Some(d_end) => {
            if e.start < d.start || e_end > d_end && !e_end.under(d_end) {
                return false;
            }
            if e.start == d.start {
                // Listed first and wider, or listed first and plain.
                return d.line < e.line && (e.end.is_none() || e_end < d_end);
            }
            (&e.start, e_end) != (&d.start, d_end)
        }
    }
}

/// Index of the parent entry of `entries[i]`: the innermost entry that
/// contains it. `None` for a hundreds class.
fn parent_of(entries: &[Entry], i: usize) -> Result<Option<usize>, String> {
    let e = &entries[i];
    let cands: Vec<usize> = (0..entries.len())
        .filter(|&j| j != i && contains(&entries[j], e))
        .collect();
    let inner: Vec<usize> = cands
        .iter()
        .copied()
        .filter(|&c| {
            cands
                .iter()
                .all(|&d| d == c || contains(&entries[d], &entries[c]))
        })
        .collect();
    match inner.as_slice() {
        [one] => Ok(Some(*one)),
        [] if cands.is_empty()
            && e.end.is_none()
            && e.start.frac.is_empty()
            && e.start.int.is_multiple_of(100) =>
        {
            Ok(None)
        }
        _ => Err(format!(
            "fdc.txt:{}: `{}` has no single parent (candidates: {:?})",
            e.line,
            e.code,
            cands.iter().map(|&c| &entries[c].code).collect::<Vec<_>>()
        )),
    }
}

/// The span a plain hundreds or tens class covers (`010` is 010-019), so
/// the ranges inside it nest under it.
fn span(e: &Entry) -> Option<CodeRange> {
    if let Some(end) = &e.end {
        return Some(CodeRange {
            start: e.start.text(),
            end: end.text(),
        });
    }
    if !e.start.frac.is_empty() || !e.start.int.is_multiple_of(10) {
        return None;
    }
    let width = if e.start.int.is_multiple_of(100) {
        99
    } else {
        9
    };
    Some(CodeRange {
        start: e.start.text(),
        end: format!("{:03}", e.start.int + width),
    })
}

/// Build the scheme from the parsed entries.
pub fn build(
    entries: &[Entry],
    lock: &Locked,
    mut stats: Stats,
) -> Result<(SchemeDoc, Stats), String> {
    let mut classes = Vec::with_capacity(entries.len());
    let mut parents = Vec::with_capacity(entries.len());
    for (i, e) in entries.iter().enumerate() {
        let parent = parent_of(entries, i)?;
        parents.push(parent);
        let alts = terms(&e.terms);
        let label = label(&alts);
        if label.is_empty() {
            return Err(format!("fdc.txt:{}: `{}` has no label", e.line, e.code));
        }
        let mut seen = BTreeSet::from([label.clone()]);
        let mut vocab = Vec::new();
        for t in alts.iter().flatten() {
            if seen.insert(t.text.clone()) {
                stats.lcsh_terms += usize::from(t.lcsh);
                vocab.push(VocabLink {
                    term: t.text.clone(),
                    source: if t.lcsh { LCSH_TERM } else { PLAIN_TERM }.to_string(),
                    source_id: None,
                    weight: 1.0,
                });
            }
        }
        stats.vocab_terms += vocab.len();
        classes.push(ClassDef {
            code: e.code.clone(),
            label,
            parent: parent.map(|p| entries[p].code.clone()),
            range: span(e),
            vocab,
            note: e.note.clone(),
        });
    }
    let mut seen = BTreeSet::new();
    if let Some(c) = classes.iter().find(|c| !seen.insert(c.code.as_str())) {
        return Err(format!("duplicate code `{}`", c.code));
    }
    tree_stats(&parents, &classes, &mut stats);
    let doc = SchemeDoc {
        schema: None,
        name: "fdc".to_string(),
        version: "0.06".to_string(),
        title: "Free Decimal Correspondence".to_string(),
        source: Some(Source {
            description: format!(
                "Free Decimal Correspondence v0.06 (2010-08-19), edited by John Mark Ockerbloom; fdc.txt at commit {}, sha256 {}.",
                lock.commit, lock.sha256
            ),
            url: Some(REPO.to_string()),
            license: Some(
                "CC0 1.0: dedicated to the public domain by John Mark Ockerbloom".to_string(),
            ),
        }),
        code_grammar: CODE_GRAMMAR.to_string(),
        classes,
    };
    Ok((doc, stats))
}

/// Depth and branching of the built tree.
fn tree_stats(parents: &[Option<usize>], classes: &[ClassDef], s: &mut Stats) {
    let n = parents.len();
    let mut children = vec![0usize; n];
    for p in parents.iter().flatten() {
        children[*p] += 1;
    }
    let depth = |mut i: usize| {
        let mut d = 1;
        while let Some(p) = parents[i] {
            i = p;
            d += 1;
        }
        d
    };
    let depths: Vec<usize> = (0..n).map(depth).collect();
    s.classes = n;
    s.roots = parents.iter().filter(|p| p.is_none()).count();
    s.ranges = classes.iter().filter(|c| c.code.contains('-')).count();
    s.max_depth = depths.iter().copied().max().unwrap_or(0);
    s.mean_depth = depths.iter().sum::<usize>() as f64 / n.max(1) as f64;
    s.depth_counts = (1..=s.max_depth)
        .map(|d| depths.iter().filter(|&&x| x == d).count())
        .collect();
    let internal: Vec<usize> = (0..n).filter(|&i| children[i] > 0).collect();
    s.internal = internal.len();
    if let Some(&top) = internal
        .iter()
        .max_by_key(|&&i| (children[i], std::cmp::Reverse(i)))
    {
        s.max_branch = children[top];
        s.max_branch_code = classes[top].code.clone();
    }
    s.mean_branch =
        internal.iter().map(|&i| children[i]).sum::<usize>() as f64 / internal.len().max(1) as f64;
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
# comment
000 General works
  001 "Intellectual life"
 010 Bibliography
  011 General bibliographies + Universal catalogs
  017-019 "Library catalogs"
  017 "Classified catalogs"
  018 "Author catalogs"
 020 "Library science"
  020-025 Science and administration of libraries
  021 Scope
  026 "Special libraries"
100 Philosophy
 110 Metaphysics
  113-119 "Cosmology"
  114 Space
900 History
 940-990 "History, Modern" | Modern history
 940 "Europe"
 990 "Oceania" + "Polar regions"
  991-992 "Indonesia" + "Philippines"
   991.1 "Borneo"
   991.23 "Celebes (Indonesia)"
   991.2 "Sumatra"
"#;

    fn doc() -> (SchemeDoc, Stats) {
        let mut stats = Stats::default();
        let entries = parse_with(SAMPLE, &mut stats, &[]).unwrap();
        let lock = Locked {
            file: "fdc.txt".into(),
            url: "u".into(),
            commit: "c".into(),
            fetched: "2026-01-01".into(),
            bytes: 1,
            sha256: "s".into(),
        };
        build(&entries, &lock, stats).unwrap()
    }

    fn parent<'a>(d: &'a SchemeDoc, code: &str) -> Option<&'a str> {
        d.classes
            .iter()
            .find(|c| c.code == code)
            .unwrap()
            .parent
            .as_deref()
    }

    #[test]
    fn numbers_nest_by_digit() {
        let (d, _) = doc();
        assert_eq!(parent(&d, "000"), None);
        assert_eq!(parent(&d, "010"), Some("000"));
        assert_eq!(parent(&d, "011"), Some("010"));
        assert_eq!(parent(&d, "001"), Some("000"));
        assert_eq!(parent(&d, "991.1"), Some("991-992"));
        assert_eq!(parent(&d, "991.2"), Some("991-992"));
        assert_eq!(parent(&d, "991.23"), Some("991.2"));
    }

    #[test]
    fn ranges_group_the_numbers_inside_them() {
        let (d, _) = doc();
        assert_eq!(parent(&d, "017-019"), Some("010"));
        assert_eq!(parent(&d, "017"), Some("017-019"));
        assert_eq!(parent(&d, "113-119"), Some("110"));
        assert_eq!(parent(&d, "114"), Some("113-119"));
        // A plain number listed before a range that starts at it is its parent.
        assert_eq!(parent(&d, "020-025"), Some("020"));
        assert_eq!(parent(&d, "021"), Some("020-025"));
        assert_eq!(parent(&d, "026"), Some("020"));
    }

    #[test]
    fn a_range_ending_at_a_tens_class_covers_the_class() {
        let (d, s) = doc();
        let r = d.classes.iter().find(|c| c.code == "940-990").unwrap();
        assert_eq!(r.range.as_ref().unwrap().end, "999");
        assert_eq!(s.widened.len(), 1);
        assert_eq!(parent(&d, "990"), Some("940-990"));
    }

    #[test]
    fn tens_and_hundreds_carry_their_span() {
        let (d, _) = doc();
        let span = |code: &str| {
            let r = d
                .classes
                .iter()
                .find(|c| c.code == code)
                .unwrap()
                .range
                .clone();
            r.map(|r| (r.start, r.end))
        };
        assert_eq!(span("010"), Some(("010".into(), "019".into())));
        assert_eq!(span("100"), Some(("100".into(), "199".into())));
        assert_eq!(span("011"), None);
    }

    #[test]
    fn terms_split_into_label_and_vocabulary() {
        let (d, _) = doc();
        let c = d.classes.iter().find(|c| c.code == "940-990").unwrap();
        assert_eq!(c.label, "History, Modern");
        assert_eq!(c.vocab.len(), 1);
        assert_eq!(c.vocab[0].term, "Modern history");
        assert_eq!(c.vocab[0].source, PLAIN_TERM);
        let c = d.classes.iter().find(|c| c.code == "990").unwrap();
        assert_eq!(c.label, "Oceania; Polar regions");
        assert_eq!(c.vocab.len(), 2);
        assert!(c.vocab.iter().all(|v| v.source == LCSH_TERM));
    }

    #[test]
    fn parentheses_and_partial_quotes() {
        let t = terms(r#"("Emigration" + "Migration") from demographic perspectives | Migrations"#);
        assert_eq!(t.len(), 2);
        assert_eq!(
            t[0][0].text,
            "(Emigration + Migration) from demographic perspectives"
        );
        assert!(!t[0][0].lcsh);
        let t = terms(r#"("Palestine" + "Jews") | Ancient Judea"#);
        assert_eq!(t[0].len(), 2);
        let t = terms(r#""Poor" (from social service perspectives)"#);
        assert_eq!(t[0][0].text, "Poor (from social service perspectives)");
        assert!(!t[0][0].lcsh);
    }

    #[test]
    fn entities_decode() {
        assert_eq!(
            decode("Qu&eacute;bec Fran&ccedil;ais").unwrap(),
            "Québec Français"
        );
        assert!(decode("&nope;").is_err());
    }

    #[test]
    fn errata_apply_once_and_must_still_apply() {
        let mut stats = Stats::default();
        let text = "970 America\n975 \"South Atlantic States\"\n975 \"Gulf States\" + \"Southwest, Old\"\n";
        let errata = [ERRATA[0]];
        let entries = parse_with(text, &mut stats, &errata).unwrap();
        assert_eq!(entries[2].code, "976");
        assert_eq!(stats.errata.len(), 1);
        let stale = parse_with(
            "975 \"South Atlantic States\"\n",
            &mut Stats::default(),
            &errata,
        );
        assert!(stale.is_err());
    }

    #[test]
    fn tree_stats_count_depth_and_branching() {
        let (_, s) = doc();
        assert_eq!(s.roots, 3);
        assert!(s.max_depth >= 4);
        assert!(s.max_branch >= 3);
    }

    #[test]
    fn numbers_order_as_a_shelf() {
        let n = |s: &str| Num::parse(s).unwrap();
        assert!(n("962.4") < n("962.9"));
        assert!(n("971.2") < n("971.23"));
        assert!(n("971.23") < n("971.3"));
        assert!(n("999") > n("990.5"));
        assert!(n("991.23").under(&n("991.2")));
        assert!(!n("991.3").under(&n("991.2")));
        assert!(Num::parse("97").is_err());
        assert!(Num::parse("971.").is_err());
    }
}
