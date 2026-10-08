//! The verb catalog (R20) and the CLI form of any request.
//!
//! `describe` returns this catalog so an agent can operate the tool from
//! its output alone; [`command`] renders a request back into the
//! subcommand that sends it, which is how every `next_command` and
//! `commit_command` stays runnable as given.

use serde::Serialize;
use serde_json::Value;
use shelfmark_core::Verb;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ArgSpec {
    pub name: &'static str,
    /// `string`, `number`, `integer`, `id`, `path` or `list`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub required: bool,
    pub help: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct VerbSpec {
    pub verb: Verb,
    /// Subcommand words, e.g. `scheme compile`.
    pub cli: &'static str,
    /// The argument the CLI takes positionally, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positional: Option<&'static str>,
    pub summary: &'static str,
    pub args: &'static [ArgSpec],
    pub example: &'static str,
}

impl VerbSpec {
    pub fn arg_names(&self) -> Vec<&'static str> {
        self.args.iter().map(|a| a.name).collect()
    }
}

const fn arg(name: &'static str, kind: &'static str, help: &'static str) -> ArgSpec {
    ArgSpec {
        name,
        kind,
        required: false,
        help,
    }
}

const fn req(name: &'static str, kind: &'static str, help: &'static str) -> ArgSpec {
    ArgSpec {
        name,
        kind,
        required: true,
        help,
    }
}

const SCHEME: ArgSpec = arg(
    "scheme",
    "id",
    "scheme ID; default: the first compiled scheme",
);
const PROFILE: ArgSpec = arg(
    "profile",
    "id",
    "profile ID; default: the recommended profile",
);
const AUTHOR: ArgSpec = arg(
    "author",
    "string",
    "who is making the claim; default: $USER",
);

const fn verb(
    verb: Verb,
    cli: &'static str,
    positional: Option<&'static str>,
    summary: &'static str,
    args: &'static [ArgSpec],
    example: &'static str,
) -> VerbSpec {
    VerbSpec {
        verb,
        cli,
        positional,
        summary,
        args,
        example,
    }
}

pub const VERBS: [VerbSpec; 16] = [
    verb(Verb::Describe, "describe", Some("topic"),
        "List every noun, verb, profile and scheme; with a topic (a verb, `ids` or `errors`), just that",
        &[arg("topic", "string", "a verb name, `ids` or `errors`")],
        "shelfmark describe classify"),
    verb(Verb::Status, "status", None,
        "Loaded schemes, profiles, machine capability, flagged backlog, stale placements, calibration and the latest eval",
        &[], "shelfmark status"),
    verb(Verb::Resolve, "resolve", Some("id"),
        "Resolve any typed ID (work, item, scheme, class, profile, placement) to what it names",
        &[req("id", "id", "any typed ID")],
        "shelfmark resolve class:lcc@2024:QA76"),
    verb(Verb::Classify, "classify", None,
        "Place a Work into a scheme; appends a memoized machine claim with its trace",
        &[arg("text", "string", "the Work's text"),
          arg("title", "string", "title"),
          arg("file", "path", "a .txt, .md, .html or (pdf feature) .pdf file; minted and ingested first"),
          arg("identifiers", "list", "isbn:, lccn:, doi:, arxiv: or domain: identifiers"),
          arg("url", "string", "source URL"),
          arg("headings", "list", "subject headings from a catalog record"),
          SCHEME, PROFILE,
          arg("target", "number", "target confidence in [0, 1]; default: the profile's")],
        "shelfmark classify --title 'Compilers' --text 'compilers (computer programs)'"),
    verb(Verb::Explain, "explain", Some("id"),
        "The full trace of a placement, or of a Work's current placements",
        &[req("id", "id", "placement: or work: ID"), SCHEME],
        "shelfmark explain work:nt1s256:<hex>"),
    verb(Verb::ListFlagged, "list-flagged", None,
        "Current placements flagged as low confidence, least confident first",
        &[SCHEME, arg("limit", "integer", "at most this many; default 50")],
        "shelfmark list-flagged --limit 10"),
    verb(Verb::Confirm, "confirm", Some("placement"),
        "Confirm a machine placement; adds a gold example (previewable mutation)",
        &[req("placement", "id", "the placement: to confirm"), AUTHOR],
        "shelfmark confirm placement:s256:<hex>"),
    verb(Verb::Correct, "correct", Some("placement"),
        "Replace a placement's class with a confirmed one; adds a gold example (previewable mutation)",
        &[req("placement", "id", "the placement: to correct"),
          req("class", "string", "a class: ID or a code in the placement's scheme"), AUTHOR],
        "shelfmark correct placement:s256:<hex> --class K"),
    verb(Verb::Stale, "stale", None,
        "Current placements a scheme or profile change made stale; a newer scheme version stales only the classes it changed",
        &[SCHEME, PROFILE], "shelfmark stale"),
    verb(Verb::SchemeCompile, "scheme compile", Some("path"),
        "Validate and compile a scheme data file into the store (previewable mutation)",
        &[req("path", "path", "scheme JSON file")],
        "shelfmark scheme compile schemes/my-scheme.json"),
    verb(Verb::SchemeBrowse, "scheme browse", Some("code"),
        "A class with its ancestors and children; without a code, the top level",
        &[arg("code", "string", "class code"), SCHEME],
        "shelfmark scheme browse QA"),
    verb(Verb::SchemeSearch, "scheme search", Some("text"),
        "Search class labels and vocabulary",
        &[req("text", "string", "search text"), SCHEME,
          arg("limit", "integer", "at most this many; default 10")],
        "shelfmark scheme search software"),
    verb(Verb::SchemeValidate, "scheme validate", Some("code"),
        "Check a class code against a scheme, or a scheme file against the schema",
        &[arg("code", "string", "class code to check"), SCHEME,
          arg("path", "path", "scheme JSON file to validate instead")],
        "shelfmark scheme validate QA76.76.C65"),
    verb(Verb::Eval, "eval", None,
        "Top-1/top-k accuracy, calibration error, flagged rate and latency from the gold set",
        &[SCHEME, PROFILE], "shelfmark eval"),
    verb(Verb::Link, "link", None,
        "Link a file (item:) to an existing Work (previewable mutation)",
        &[req("item", "id", "item: ID of the file"), req("work", "id", "work: ID")],
        "shelfmark link --item item:s256:<hex> --work work:nt1s256:<hex>"),
    verb(Verb::Name, "name", Some("id"),
        "Preview the folder path and filename for a Work's current placement",
        &[req("id", "id", "placement: or work: ID"), SCHEME,
          arg("title", "string", "title for the filename"),
          arg("ext", "string", "file extension; default pdf")],
        "shelfmark name work:nt1s256:<hex> --title 'Dragon Book'"),
];

pub fn spec(verb: Verb) -> &'static VerbSpec {
    VERBS
        .iter()
        .find(|s| s.verb == verb)
        .expect("every verb has a spec")
}

/// The CLI command that sends `verb` with `args`.
pub fn command(verb: Verb, args: &Value) -> String {
    let spec = spec(verb);
    let mut out = format!("shelfmark {}", spec.cli);
    let Some(map) = args.as_object() else {
        return out;
    };
    if let Some(v) = spec.positional.and_then(|p| map.get(p)) {
        push_value(&mut out, v);
    }
    for (k, v) in map {
        if Some(k.as_str()) == spec.positional || v.is_null() {
            continue;
        }
        let flag = flag_name(k);
        match v {
            Value::Array(items) => {
                for item in items {
                    out.push_str(&format!(" --{flag}"));
                    push_value(&mut out, item);
                }
            }
            v => {
                out.push_str(&format!(" --{flag}"));
                push_value(&mut out, v);
            }
        }
    }
    out
}

/// List arguments take one flag per value, named in the singular.
fn flag_name(arg: &str) -> String {
    match arg {
        "identifiers" => "identifier".into(),
        "headings" => "heading".into(),
        other => other.replace('_', "-"),
    }
}

fn push_value(out: &mut String, v: &Value) {
    out.push(' ');
    match v {
        Value::String(s) => out.push_str(&shell_quote(s)),
        other => out.push_str(&shell_quote(&other.to_string())),
    }
}

/// Quote for a POSIX shell when needed.
pub fn shell_quote(s: &str) -> String {
    let safe = !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"@%+=:,./_-".contains(&b));
    if safe {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use shelfmark_core::Verb;

    use super::{command, spec, VERBS};

    #[test]
    fn commands_put_the_positional_first_and_quote_values() {
        assert_eq!(
            command(
                Verb::Correct,
                &json!({"placement": "placement:s256:ab", "class": "K", "author": "D O'Neil"})
            ),
            r"shelfmark correct placement:s256:ab --author 'D O'\''Neil' --class K"
        );
        assert_eq!(
            command(
                Verb::Classify,
                &json!({"identifiers": ["isbn:1", "doi:2"], "target": 0.5})
            ),
            "shelfmark classify --identifier isbn:1 --identifier doi:2 --target 0.5"
        );
        assert_eq!(
            command(Verb::SchemeCompile, &json!({"path": "a b.json"})),
            "shelfmark scheme compile 'a b.json'"
        );
    }

    #[test]
    fn every_verb_has_one_spec_whose_positional_is_an_arg() {
        for v in Verb::ALL {
            let s = spec(v);
            assert_eq!(VERBS.iter().filter(|x| x.verb == v).count(), 1);
            if let Some(p) = s.positional {
                assert!(s.arg_names().contains(&p), "{v:?}");
            }
        }
    }
}
