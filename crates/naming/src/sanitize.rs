//! Make one path segment safe on Windows, macOS and Linux.
//!
//! The result is NFC, has no control characters and none of
//! `< > : " / \ | ? *`, does not start with a dot (hidden on Unix), does not
//! end with a dot or space (Windows strips those), is not a Windows device
//! name (`CON`, `com1.txt`, ...), and is at most `max_bytes` UTF-8 bytes.

use unicode_normalization::UnicodeNormalization;

/// Characters no segment may contain on Windows; `/` and `:` also cover
/// Linux and macOS.
const ILLEGAL: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Windows device names, matched case-insensitively against the part of a
/// name before its first dot.
const RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
    "COM8", "COM9", "COM¹", "COM²", "COM³", "LPT0", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6",
    "LPT7", "LPT8", "LPT9", "LPT¹", "LPT²", "LPT³", "CONIN$", "CONOUT$",
];

/// Longest extension kept whole when a segment is cut to length.
const MAX_EXT_BYTES: usize = 16;

/// Sanitize one segment. `keep_ext` keeps a trailing `.ext` intact when
/// the segment has to be shortened (for the file name).
pub fn segment(raw: &str, max_bytes: usize, keep_ext: bool) -> String {
    let mut s = String::with_capacity(raw.len());
    let mut space = false;
    for c in raw.nfc() {
        let c = if ILLEGAL.contains(&c) {
            '_'
        } else if c.is_control() || c.is_whitespace() {
            ' '
        } else {
            c
        };
        if c == ' ' {
            if !space && !s.is_empty() {
                s.push(' ');
            }
            space = true;
        } else {
            s.push(c);
            space = false;
        }
    }
    let mut s = trim_end(&s).to_string();
    if s.starts_with('.') {
        let dots = s.len() - s.trim_start_matches('.').len();
        s.replace_range(..dots, &"_".repeat(dots));
    }
    let s = cap(&s, max_bytes.max(8), keep_ext);
    let s = unreserve(s);
    if s.is_empty() {
        "_".to_string()
    } else {
        s
    }
}

/// Strip trailing dots and spaces, which Windows silently drops.
fn trim_end(s: &str) -> &str {
    s.trim_end_matches(['.', ' '])
}

/// Shorten to `max` bytes on a char boundary, keeping a short extension.
fn cap(s: &str, max: usize, keep_ext: bool) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let ext = match s.rfind('.') {
        Some(i) if keep_ext && i > 0 && s.len() - i <= MAX_EXT_BYTES && !s[i..].contains(' ') => {
            &s[i..]
        }
        _ => "",
    };
    let stem = &s[..s.len() - ext.len()];
    let mut end = max - ext.len();
    while !stem.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{ext}", trim_end(&stem[..end]))
}

/// Append `_` to a stem that is a Windows device name: `CON.txt` -> `CON_.txt`.
fn unreserve(mut s: String) -> String {
    let stem_end = s.find('.').unwrap_or(s.len());
    let stem = s[..stem_end].trim_end();
    if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(stem)) {
        s.insert(stem.len(), '_');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cut_keeps_extension_and_char_boundaries() {
        let long = format!("{}.pdf", "é".repeat(100));
        let out = segment(&long, 20, true);
        assert!(out.len() <= 20, "{out}");
        assert!(out.ends_with(".pdf"));
        assert_eq!(out, format!("{}.pdf", "é".repeat(8)));
    }

    #[test]
    fn cut_without_extension_for_folders() {
        let out = segment("Mathematics. Computer science", 12, false);
        assert_eq!(out, "Mathematics");
    }
}
