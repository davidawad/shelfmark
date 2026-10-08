//! The first worksheet of an `.xlsx` workbook as rows of cell text.

use std::io::Cursor;

use calamine::{Data, Reader, Xlsx};

/// Every row of the first worksheet, each cell as text: numbers without a
/// fractional part print as integers (`11`, not `11.0`), empty cells as "".
pub fn rows(bytes: &[u8]) -> Result<Vec<Vec<String>>, String> {
    let mut book: Xlsx<_> = Xlsx::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let range = book
        .worksheet_range_at(0)
        .ok_or("the workbook has no worksheet")?
        .map_err(|e| e.to_string())?;
    Ok(range
        .rows()
        .map(|row| row.iter().map(text).collect())
        .collect())
}

fn text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", *f as i64),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_print_as_codes() {
        assert_eq!(text(&Data::Float(111110.0)), "111110");
        assert_eq!(text(&Data::Int(11)), "11");
        assert_eq!(text(&Data::String("31-33".into())), "31-33");
        assert_eq!(text(&Data::Empty), "");
    }
}
