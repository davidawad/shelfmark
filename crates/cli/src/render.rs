//! Output: the envelope as one JSON line for programs, an indented
//! outline for a person at a terminal (R19).

use std::fmt::Write as _;

use serde_json::Value;
use shelfmark_core::Envelope;

/// One line of compact JSON.
pub fn json(envelope: &Envelope) -> String {
    serde_json::to_string(envelope).unwrap_or_else(|e| {
        format!(r#"{{"ok":false,"error":{{"code":"internal","message":"envelope did not serialize: {e}","next_command":"shelfmark describe"}}}}"#)
    })
}

/// A human rendering: data as an outline, or the error with its fix.
pub fn human(envelope: &Envelope) -> String {
    let mut out = String::new();
    match envelope {
        Envelope::Ok { data, .. } => outline(&mut out, data, 0),
        Envelope::Err { error, .. } => {
            let _ = writeln!(out, "error[{}]: {}", error.code, error.message);
            if let Some(next) = &error.next_command {
                let _ = writeln!(out, "  next: {next}");
            }
            if let Some(details) = &error.details {
                let _ = writeln!(out, "  details:");
                outline(&mut out, details, 2);
            }
        }
    }
    out
}

fn scalar(v: &Value) -> Option<String> {
    match v {
        Value::Null => Some("-".into()),
        Value::String(s) => Some(s.clone()),
        Value::Bool(_) | Value::Number(_) => Some(v.to_string()),
        Value::Array(a) if a.is_empty() => Some("[]".into()),
        Value::Array(a) if a.iter().all(|x| !x.is_object() && !x.is_array()) => {
            Some(a.iter().filter_map(scalar).collect::<Vec<_>>().join(", "))
        }
        Value::Object(m) if m.is_empty() => Some("{}".into()),
        _ => None,
    }
}

fn outline(out: &mut String, v: &Value, depth: usize) {
    let pad = "  ".repeat(depth);
    match v {
        Value::Object(map) => {
            for (k, x) in map {
                match scalar(x) {
                    Some(s) => {
                        let _ = writeln!(out, "{pad}{k}: {s}");
                    }
                    None => {
                        let _ = writeln!(out, "{pad}{k}:");
                        outline(out, x, depth + 1);
                    }
                }
            }
        }
        Value::Array(items) => {
            for x in items {
                match scalar(x) {
                    Some(s) => {
                        let _ = writeln!(out, "{pad}- {s}");
                    }
                    None => {
                        let _ = writeln!(out, "{pad}-");
                        outline(out, x, depth + 1);
                    }
                }
            }
        }
        other => {
            let _ = writeln!(out, "{pad}{}", scalar(other).unwrap_or_default());
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use shelfmark_core::{Envelope, Error, ErrorCode};

    #[test]
    fn human_rendering_outlines_data_and_shows_the_fix() {
        let ok = Envelope::ok(json!({"a": 1, "b": {"c": ["x", "y"]}, "d": [{"e": null}]}));
        assert_eq!(
            super::human(&ok),
            "a: 1\nb:\n  c: x, y\nd:\n  -\n    e: -\n"
        );
        let err = Envelope::err(Error::new(ErrorCode::NotFound, "gone").next("shelfmark status"));
        assert_eq!(
            super::human(&err),
            "error[not_found]: gone\n  next: shelfmark status\n"
        );
        assert!(super::json(&err).starts_with(r#"{"ok":false"#));
    }
}
