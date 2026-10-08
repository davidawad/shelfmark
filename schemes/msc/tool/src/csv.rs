//! The `MSC_2020.csv` reader.
//!
//! Despite the name the file is tab-separated: a `code text description`
//! header, then one row per class with every field in double quotes, CRLF
//! line ends, encoded as Latin-1 (`Gödel` is byte 0xF6).

/// One class row as published.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub code: String,
    /// The short caption.
    pub text: String,
    /// The caption plus cross-references (`[See also 01A70]`).
    pub description: String,
    /// 1-based line in the file, for error messages.
    pub line: usize,
}

/// UTF-8 when the bytes are valid UTF-8, Latin-1 otherwise.
pub fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| char::from(b)).collect(),
    }
}

/// Parse the whole file. Any malformed row is an error naming its line.
pub fn parse(bytes: &[u8]) -> Result<Vec<Row>, String> {
    let text = decode(bytes);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let mut lines = text.lines().enumerate();
    let header = lines.next().map(|(_, l)| fields(l));
    if header != Some(Ok(vec!["code".into(), "text".into(), "description".into()])) {
        return Err("line 1: expected the header `code<TAB>text<TAB>description`".into());
    }
    let mut rows = Vec::new();
    for (i, line) in lines {
        let n = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        let f = fields(line).map_err(|e| format!("line {n}: {e}"))?;
        let [code, text, description]: [String; 3] = f
            .try_into()
            .map_err(|f: Vec<String>| format!("line {n}: expected 3 fields, found {}", f.len()))?;
        if code.is_empty() || text.is_empty() {
            return Err(format!("line {n}: empty code or text"));
        }
        rows.push(Row {
            code,
            text,
            description,
            line: n,
        });
    }
    if rows.is_empty() {
        return Err("no class rows".into());
    }
    Ok(rows)
}

/// Split one line on tabs and unquote each field (`""` is a literal quote).
fn fields(line: &str) -> Result<Vec<String>, String> {
    line.split('\t')
        .map(|f| {
            let f = f.trim_end_matches('\r');
            match f.strip_prefix('"').and_then(|f| f.strip_suffix('"')) {
                Some(inner) => Ok(inner.replace("\"\"", "\"")),
                None if !f.contains('"') => Ok(f.to_string()),
                None => Err(format!("unbalanced quotes in `{f}`")),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin1_decodes_and_quotes_unwrap() {
        let bytes =
            b"code\ttext\tdescription\r\n\"03F40\"\t\"G\xf6del\"\t\"G\xf6del \"\"x\"\"\"\r\n";
        let rows = parse(bytes).unwrap();
        assert_eq!(rows[0].text, "Gödel");
        assert_eq!(rows[0].description, "Gödel \"x\"");
        assert_eq!(rows[0].line, 2);
    }

    #[test]
    fn a_short_row_names_its_line() {
        let bytes = b"code\ttext\tdescription\n\"00A05\"\t\"x\"\n";
        assert_eq!(
            parse(bytes).unwrap_err(),
            "line 2: expected 3 fields, found 2"
        );
    }

    #[test]
    fn a_wrong_header_is_refused() {
        assert!(parse(b"a\tb\tc\n").unwrap_err().starts_with("line 1"));
    }
}
