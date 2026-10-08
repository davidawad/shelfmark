//! The CPC title list: one tab-separated file per section inside
//! `CPCTitleListYYYYMM.zip`, each line `symbol \t level \t title`. The
//! level is empty for sections, classes and subclasses and a dot count
//! for groups (0 for a main group, 1 for a one-dot subgroup, ...).

use std::io::{Cursor, Read};

/// One line of the title list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title {
    pub symbol: String,
    /// Dot level of a group; `None` above main-group level.
    pub level: Option<u8>,
    pub title: String,
}

/// Every title in the archive, section files in name order and lines in
/// file order.
pub fn read_zip(bytes: &[u8]) -> Result<Vec<Title>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut names: Vec<String> = archive
        .file_names()
        .filter(|n| n.ends_with(".txt"))
        .map(str::to_string)
        .collect();
    names.sort();
    if names.is_empty() {
        return Err("no .txt section files in the archive".into());
    }
    let mut out = Vec::new();
    for name in names {
        let mut text = String::new();
        archive
            .by_name(&name)
            .map_err(|e| format!("{name}: {e}"))?
            .read_to_string(&mut text)
            .map_err(|e| format!("{name}: {e}"))?;
        out.extend(parse(&text).map_err(|e| format!("{name}: {e}"))?);
    }
    Ok(out)
}

/// Parse one section file.
pub fn parse(text: &str) -> Result<Vec<Title>, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let mut fields = line.splitn(3, '\t');
        let (Some(symbol), Some(level), Some(title)) =
            (fields.next(), fields.next(), fields.next())
        else {
            return Err(format!("line {}: expected symbol, level and title", i + 1));
        };
        let level = match level.trim() {
            "" => None,
            l => Some(
                l.parse()
                    .map_err(|_| format!("line {}: level `{l}` is not a number", i + 1))?,
            ),
        };
        out.push(Title {
            symbol: symbol.trim().to_string(),
            level,
            title: title.split_whitespace().collect::<Vec<_>>().join(" "),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const TEXT: &str = "\u{feff}A\t\tHUMAN NECESSITIES\r\nA01B1/00\t0\tHand tools\r\n\r\nA01B1/02\t1\tSpades;  Shovels\r\n";

    #[test]
    fn parses_lines() {
        let t = parse(TEXT).unwrap();
        assert_eq!(t.len(), 3);
        assert_eq!(t[0].level, None);
        assert_eq!(t[1].level, Some(0));
        assert_eq!(t[2].title, "Spades; Shovels");
        assert!(parse("A01\n").is_err());
        assert!(parse("A01B1/00\tx\tHand tools\n").is_err());
    }

    #[test]
    fn reads_sections_in_name_order() {
        let mut buf = Vec::new();
        {
            let mut w = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opts = zip::write::SimpleFileOptions::default();
            for (name, text) in [
                (
                    "cpc-section-B_20260801.txt",
                    "B\t\tPERFORMING OPERATIONS; TRANSPORTING\n",
                ),
                ("cpc-section-A_20260801.txt", TEXT),
                ("README.pdf", "ignored"),
            ] {
                w.start_file(name, opts).unwrap();
                w.write_all(text.as_bytes()).unwrap();
            }
            w.finish().unwrap();
        }
        let t = read_zip(&buf).unwrap();
        let symbols: Vec<&str> = t.iter().map(|t| t.symbol.as_str()).collect();
        assert_eq!(symbols, ["A", "A01B1/00", "A01B1/02", "B"]);
    }
}
