//! The clap surface: every subcommand becomes a `Request {verb, args}`
//! for [`Shelfmark`](crate::Shelfmark). Arguments use the same names as
//! the request args (see `describe`), so `describe::command` can render
//! any request back into the command that sends it.

use std::ffi::OsString;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{Args as ClapArgs, Parser, Subcommand};
use serde_json::{json, Map, Value};
use shelfmark_core::envelope::Mode;
use shelfmark_core::{Envelope, Error, ErrorCode, Request, Service, Verb};

use crate::{batch, render, wiring};

#[derive(Debug, Parser)]
#[command(
    name = "shelfmark",
    version,
    about = "Classify materials into any scheme; `shelfmark describe` lists everything"
)]
pub struct Cli {
    /// Print the JSON envelope even on a terminal.
    #[arg(long, global = true)]
    pub json: bool,
    /// Store directory.
    #[arg(
        long,
        global = true,
        env = "SHELFMARK_STORE",
        default_value = ".shelfmark"
    )]
    pub store: PathBuf,
    /// Commit a mutation instead of previewing it.
    #[arg(long, global = true)]
    pub commit: bool,
    /// The revision a preview reported; required with --commit.
    #[arg(long, global = true, value_name = "REVISION")]
    pub expected_revision: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, ClapArgs)]
pub struct Scoped {
    #[arg(long)]
    pub scheme: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(Debug, ClapArgs)]
pub struct Classify {
    #[arg(long)]
    pub text: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    /// Author name (repeatable).
    #[arg(long = "author")]
    pub authors: Vec<String>,
    #[arg(long)]
    pub file: Option<String>,
    /// isbn:, lccn:, doi:, arxiv: or domain: (repeatable).
    #[arg(long = "identifier")]
    pub identifiers: Vec<String>,
    #[arg(long)]
    pub url: Option<String>,
    /// Subject heading (repeatable).
    #[arg(long = "heading")]
    pub headings: Vec<String>,
    #[command(flatten)]
    pub scoped: Scoped,
    #[arg(long)]
    pub target: Option<f64>,
}

#[derive(Debug, ClapArgs)]
pub struct EvalArgs {
    /// NDJSON gold file to add to the store's gold set.
    #[arg(long)]
    pub gold: Option<String>,
    /// Ranks counted by top-k accuracy.
    #[arg(long)]
    pub k: Option<u64>,
    #[command(flatten)]
    pub scoped: Scoped,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Every noun, verb, profile and scheme; or one topic.
    Describe { topic: Option<String> },
    /// Schemes, profiles, machine, backlog, staleness, calibration, eval.
    Status,
    /// Resolve any typed ID.
    Resolve { id: String },
    /// Place content (a file or text) into a scheme.
    Classify(Classify),
    /// The full trace of a placement or of content's placements.
    Explain {
        id: String,
        #[arg(long)]
        scheme: Option<String>,
    },
    /// Flagged placements, least confident first.
    ListFlagged {
        #[arg(long)]
        scheme: Option<String>,
        #[arg(long)]
        limit: Option<u64>,
    },
    /// Confirm a placement (preview; --commit to apply).
    Confirm {
        placement: String,
        #[arg(long)]
        author: Option<String>,
    },
    /// Correct a placement's class (preview; --commit to apply).
    Correct {
        placement: String,
        #[arg(long)]
        class: String,
        #[arg(long)]
        author: Option<String>,
    },
    /// Placements made stale by a scheme or profile change.
    Stale(Scoped),
    /// Compile, browse, search or validate schemes.
    #[command(subcommand)]
    Scheme(SchemeCommand),
    /// Accuracy and calibration from the gold set (preview; --commit
    /// writes eval.json and calibration.json).
    Eval(EvalArgs),
    /// Link a specimen (a file) to existing content (preview; --commit to
    /// apply).
    Link {
        /// The specimen ID (`item:...`; `specimen:...` is accepted too).
        #[arg(long, visible_alias = "specimen")]
        item: String,
        /// The content ID (`work:...`; `content:...` is accepted too).
        /// `--work` is the deprecated spelling, kept for one release.
        #[arg(long, alias = "work")]
        content: String,
    },
    /// Preview the path for content's placement.
    Name {
        id: String,
        #[arg(long)]
        scheme: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        ext: Option<String>,
    },
    /// Send one raw request: '{"verb": "...", "args": {...}}'.
    Call { request: String },
    /// Classify files (or run NDJSON requests) as an NDJSON batch.
    Batch(Batch),
}

#[derive(Debug, Subcommand)]
pub enum SchemeCommand {
    Compile {
        path: String,
    },
    Browse {
        code: Option<String>,
        #[arg(long)]
        scheme: Option<String>,
    },
    Search {
        text: String,
        #[arg(long)]
        scheme: Option<String>,
        #[arg(long)]
        limit: Option<u64>,
    },
    Validate {
        code: Option<String>,
        #[arg(long)]
        scheme: Option<String>,
        #[arg(long)]
        path: Option<String>,
    },
}

#[derive(Debug, ClapArgs)]
pub struct Batch {
    /// Files to classify, one line each, keyed by item: ID.
    pub files: Vec<String>,
    /// NDJSON file of requests (`-` for stdin), keyed by request hash.
    #[arg(long)]
    pub requests: Option<String>,
    /// Write lines here (appending on --resume) and print a summary.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Skip inputs that already have an ok line in --out.
    #[arg(long)]
    pub resume: bool,
    #[command(flatten)]
    pub scoped: Scoped,
    #[arg(long)]
    pub target: Option<f64>,
}

/// An args object from `(name, value)` pairs, dropping absent values.
fn obj(pairs: Vec<(&str, Option<Value>)>) -> Value {
    Value::Object(
        pairs
            .into_iter()
            .filter_map(|(k, v)| v.map(|v| (k.to_string(), v)))
            .collect(),
    )
}

fn s(v: Option<String>) -> Option<Value> {
    v.map(Value::String)
}

fn list(v: Vec<String>) -> Option<Value> {
    (!v.is_empty()).then(|| json!(v))
}

fn scoped(sc: Scoped) -> Vec<(&'static str, Option<Value>)> {
    vec![("scheme", s(sc.scheme)), ("profile", s(sc.profile))]
}

fn eval_args(e: EvalArgs) -> Value {
    let mut pairs = vec![("gold", s(e.gold)), ("k", e.k.map(|k| json!(k)))];
    pairs.extend(scoped(e.scoped));
    obj(pairs)
}

/// The request a (non-batch, non-call) subcommand sends.
pub fn to_request(command: Command) -> Result<(Verb, Value), Error> {
    use Command as C;
    use SchemeCommand as S;
    Ok(match command {
        C::Describe { topic } => (Verb::Describe, obj(vec![("topic", s(topic))])),
        C::Status => (Verb::Status, json!({})),
        C::Resolve { id } => (Verb::Resolve, json!({ "id": id })),
        C::Classify(c) => {
            let mut pairs = vec![
                ("text", s(c.text)),
                ("title", s(c.title)),
                ("authors", list(c.authors)),
                ("file", s(c.file)),
                ("identifiers", list(c.identifiers)),
                ("url", s(c.url)),
                ("headings", list(c.headings)),
                ("target", c.target.map(|t| json!(t))),
            ];
            pairs.extend(scoped(c.scoped));
            (Verb::Classify, obj(pairs))
        }
        C::Explain { id, scheme } => (
            Verb::Explain,
            obj(vec![("id", Some(json!(id))), ("scheme", s(scheme))]),
        ),
        C::ListFlagged { scheme, limit } => (
            Verb::ListFlagged,
            obj(vec![
                ("scheme", s(scheme)),
                ("limit", limit.map(|l| json!(l))),
            ]),
        ),
        C::Confirm { placement, author } => (
            Verb::Confirm,
            obj(vec![
                ("placement", Some(json!(placement))),
                ("author", s(author)),
            ]),
        ),
        C::Correct {
            placement,
            class,
            author,
        } => (
            Verb::Correct,
            obj(vec![
                ("placement", Some(json!(placement))),
                ("class", Some(json!(class))),
                ("author", s(author)),
            ]),
        ),
        C::Stale(sc) => (Verb::Stale, obj(scoped(sc))),
        C::Eval(e) => (Verb::Eval, eval_args(e)),
        C::Link { item, content } => (Verb::Link, json!({ "item": item, "content": content })),
        C::Name {
            id,
            scheme,
            title,
            ext,
        } => (
            Verb::Name,
            obj(vec![
                ("id", Some(json!(id))),
                ("scheme", s(scheme)),
                ("title", s(title)),
                ("ext", s(ext)),
            ]),
        ),
        C::Scheme(S::Compile { path }) => (Verb::SchemeCompile, json!({ "path": path })),
        C::Scheme(S::Browse { code, scheme }) => (
            Verb::SchemeBrowse,
            obj(vec![("code", s(code)), ("scheme", s(scheme))]),
        ),
        C::Scheme(S::Search {
            text,
            scheme,
            limit,
        }) => (
            Verb::SchemeSearch,
            obj(vec![
                ("text", Some(json!(text))),
                ("scheme", s(scheme)),
                ("limit", limit.map(|l| json!(l))),
            ]),
        ),
        C::Scheme(S::Validate { code, scheme, path }) => (
            Verb::SchemeValidate,
            obj(vec![
                ("code", s(code)),
                ("scheme", s(scheme)),
                ("path", s(path)),
            ]),
        ),
        C::Call { .. } | C::Batch(_) => {
            return Err(Error::internal(
                "call and batch are not single subcommand requests",
            ))
        }
    })
}

fn usage_error(message: impl Into<String>) -> Envelope {
    Envelope::err(Error::new(ErrorCode::InvalidRequest, message).next("shelfmark describe"))
}

/// Parse `argv`, run, print, and return the exit code.
pub fn main(argv: impl IntoIterator<Item = OsString>) -> i32 {
    let argv: Vec<OsString> = argv.into_iter().collect();
    let tty = io::stdout().is_terminal();
    let cli = match Cli::try_parse_from(&argv) {
        Ok(cli) => cli,
        Err(e) if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) => {
            let _ = e.print();
            return 0;
        }
        Err(e) => {
            if tty && !argv.iter().any(|a| a == "--json") {
                let _ = e.print();
                return ErrorCode::InvalidRequest.exit_code();
            }
            let msg = e.render().to_string();
            let first = msg
                .lines()
                .next()
                .unwrap_or("bad arguments")
                .trim_start_matches("error: ");
            return print(&usage_error(first), true);
        }
    };
    let as_json = cli.json || !tty;
    let service = wiring::open(&cli.store);
    let mode = if cli.commit {
        Mode::Commit
    } else {
        Mode::Preview
    };
    let request = match cli.command {
        Command::Batch(b) => return run_batch(&service, b, as_json),
        Command::Call { request } => serde_json::from_str::<Request>(&request).map_err(|e| {
            Error::new(ErrorCode::InvalidRequest, format!("not a request: {e}"))
                .next("shelfmark describe")
        }),
        other => to_request(other).map(|(verb, args)| Request {
            verb,
            args,
            mode,
            expected_revision: cli.expected_revision.clone(),
        }),
    };
    let envelope = match request {
        Ok(r) => service.call(r),
        Err(e) => Envelope::err(e),
    };
    print(&envelope, as_json)
}

fn print(envelope: &Envelope, as_json: bool) -> i32 {
    let text = if as_json {
        format!("{}\n", render::json(envelope))
    } else {
        render::human(envelope)
    };
    let _ = io::stdout().write_all(text.as_bytes());
    envelope.exit_code()
}

fn run_batch(service: &dyn Service, b: Batch, as_json: bool) -> i32 {
    match batch_inner(service, b) {
        Ok((report, summarize)) => {
            if summarize {
                print(&Envelope::ok(&report), as_json);
            }
            report.exit_code
        }
        Err(e) => print(&Envelope::err(e), as_json),
    }
}

/// Lines go to `--out` (then a summary envelope is printed) or stdout.
fn batch_inner(service: &dyn Service, b: Batch) -> Result<(batch::Report, bool), Error> {
    let bad = |m: &str| Error::new(ErrorCode::InvalidRequest, m).next("shelfmark batch --help");
    if b.resume && b.out.is_none() {
        return Err(bad(
            "--resume needs --out: it skips inputs already done in that file",
        ));
    }
    if b.files.is_empty() == b.requests.is_none() {
        return Err(bad(
            "give files to classify or --requests, not both and not neither",
        ));
    }
    let mut inputs = Vec::new();
    if let Some(src) = &b.requests {
        let text = if src == "-" {
            io::stdin()
                .lock()
                .lines()
                .collect::<Result<Vec<_>, _>>()
                .map(|l| l.join("\n"))
        } else {
            std::fs::read_to_string(src)
        }
        .map_err(|e| {
            Error::new(
                ErrorCode::InputUnreadable,
                format!("cannot read {src}: {e}"),
            )
            .next("shelfmark batch --help")
        })?;
        inputs.extend(
            text.lines()
                .filter(|l| !l.trim().is_empty())
                .map(batch::request_input),
        );
    }
    let mut base = Map::new();
    if let Value::Object(m) = obj([
        scoped(b.scoped),
        vec![("target", b.target.map(|t| json!(t)))],
    ]
    .concat())
    {
        base = m;
    }
    inputs.extend(b.files.iter().map(|f| batch::file_input(f, &base)));
    let Some(out_path) = b.out else {
        let stdout = io::stdout();
        return Ok((
            batch::run(service, inputs, Default::default(), &mut stdout.lock())?,
            false,
        ));
    };
    let done = if b.resume {
        batch::resume_set(&out_path)?
    } else {
        Default::default()
    };
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(b.resume)
        .truncate(!b.resume)
        .open(&out_path)
        .map_err(|e| {
            Error::new(
                ErrorCode::StoreError,
                format!("cannot open {}: {e}", out_path.display()),
            )
            .next("shelfmark batch --help")
        })?;
    Ok((batch::run(service, inputs, done, &mut file)?, true))
}
