//! Scheme browsing and checking (R6) and naming (R22).

use serde_json::{json, Value};
use shelfmark_core::id::AnyId;
use shelfmark_core::scheme::CodeCheck;
use shelfmark_core::{Error, ErrorCode, SchemeId, Verb};

use super::mutate::read_scheme;
use super::{find_placement, Shelfmark};
use crate::args::{bad, Args};
use crate::describe::shell_quote;

impl Shelfmark {
    pub(super) fn scheme_browse(&self, args: &Args) -> Result<Value, Error> {
        let store = self.store()?;
        let index = self.scheme_arg(store.as_ref(), args)?;
        let Some(code) = args.str("code")? else {
            return Ok(json!({ "scheme": index.scheme(), "children": index.children(None) }));
        };
        let record = index.get(code).ok_or_else(|| {
            Error::new(
                ErrorCode::NotFound,
                format!("`{code}` is not a class in {}", index.scheme()),
            )
            .details(json!({ "check": index.check_code(code) }))
            .next(format!(
                "shelfmark scheme search {} --scheme {}",
                shell_quote(code),
                index.scheme()
            ))
        })?;
        Ok(json!({
            "scheme": index.scheme(), "class": record,
            "ancestors": index.ancestors(code), "children": index.children(Some(code)),
        }))
    }

    pub(super) fn scheme_search(&self, args: &Args) -> Result<Value, Error> {
        let text = args.require("text")?;
        let limit = args.usize("limit")?.unwrap_or(10);
        let store = self.store()?;
        let index = self.scheme_arg(store.as_ref(), args)?;
        Ok(json!({ "scheme": index.scheme(), "hits": index.search(text, limit) }))
    }

    pub(super) fn scheme_validate(&self, args: &Args) -> Result<Value, Error> {
        match (args.str("code")?, args.str("path")?) {
            (Some(code), None) => {
                let store = self.store()?;
                let index = self.scheme_arg(store.as_ref(), args)?;
                Ok(
                    json!({ "scheme": index.scheme(), "code": code, "check": index.check_code(code) }),
                )
            }
            (None, Some(path)) => {
                let doc = read_scheme(path)?;
                let index = (self.wiring.compile)(&doc).map_err(|e| {
                    e.next(format!(
                        "shelfmark scheme validate --path {}",
                        shell_quote(path)
                    ))
                })?;
                Ok(json!({
                    "path": path, "valid": true, "scheme": index.scheme(), "classes": index.len(),
                    "next_command": format!("shelfmark scheme compile {}", shell_quote(path)),
                }))
            }
            _ => Err(bad(
                Verb::SchemeValidate,
                "give exactly one of `code` or `path`",
            )),
        }
    }

    pub(super) fn name(&self, args: &Args) -> Result<Value, Error> {
        let id: AnyId = args.require_id("id")?;
        let store = self.store()?;
        let claims = store.claims()?;
        let placement = match &id {
            AnyId::Placement(p) => find_placement(&claims, p)?.clone(),
            AnyId::Work(w) => {
                let scheme = args.id::<SchemeId>("scheme")?;
                let found = self.work_placements(store.as_ref(), w, scheme.as_ref())?;
                match found.as_slice() {
                    [one] => one.clone(),
                    [] => {
                        return Err(Error::new(
                            ErrorCode::NotFound,
                            format!("{w} has no placement to name"),
                        )
                        .next("shelfmark classify --text <text>"));
                    }
                    _ => {
                        return Err(bad(
                            Verb::Name,
                            format!("{w} is placed in several schemes; pass `scheme`"),
                        ))
                    }
                }
            }
            other => {
                return Err(bad(
                    Verb::Name,
                    format!("name takes a placement: or work: ID, not {}:", other.kind()),
                ))
            }
        };
        let index = self.index(store.as_ref(), Some(&placement.scheme))?;
        // A corrected code deeper than the scheme's classes files under
        // the class that contains it.
        let class = match index.check_code(&placement.class.code) {
            CodeCheck::Within { class } => class,
            _ => placement.class.clone(),
        };
        let title = args.str("title")?.unwrap_or(&placement.class.code);
        let ext = args.str("ext")?.unwrap_or("pdf");
        Ok(json!({
            "path": (self.wiring.name)(index.as_ref(), &class, title, ext)?,
            "placement": placement.id, "class": placement.class,
        }))
    }
}
