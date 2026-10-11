//! The previewable mutations: confirm, correct, link, scheme compile
//! (R12, R15, PD8). Each builds its change, then goes through
//! [`Shelfmark::mutate`], which previews or commits at a revision.

use serde_json::{json, Value};
use shelfmark_core::calibration::GoldExample;
use shelfmark_core::placement::{
    effective_status, superseded_ids, AuthorKind, Placement, Provenance, Status,
};
use shelfmark_core::scheme::{CodeCheck, SchemeDoc};
use shelfmark_core::tier::ClassifyInput;
use shelfmark_core::{ClassId, Error, ErrorCode, ItemId, PlacementId, Request, WorkId};

use super::{find_placement, Shelfmark};
use crate::args::Args;
use crate::describe::shell_quote;
use crate::store::Store;

impl Shelfmark {
    pub(super) fn confirm(&self, req: &Request, args: &Args) -> Result<Value, Error> {
        let id: PlacementId = args.require_id("placement")?;
        let mut store = self.store()?;
        let old = live_claim(store.as_ref(), &id)?;
        let claim = human_claim(&old, old.class.clone(), args.str("author")?);
        self.commit_claim(req, store.as_mut(), old, claim)
    }

    pub(super) fn correct(&self, req: &Request, args: &Args) -> Result<Value, Error> {
        let id: PlacementId = args.require_id("placement")?;
        let class = args.require("class")?;
        let mut store = self.store()?;
        let old = live_claim(store.as_ref(), &id)?;
        let code = match class.parse::<ClassId>() {
            Ok(c) if c.scheme != old.scheme => {
                return Err(Error::new(
                    ErrorCode::InvalidCode,
                    format!("{c} is not in {}, the placement's scheme", old.scheme),
                )
                .next(format!(
                    "shelfmark scheme search <terms> --scheme {}",
                    old.scheme
                )));
            }
            Ok(c) => c.code,
            Err(_) => class.to_string(),
        };
        let index = self.index(store.as_ref(), Some(&old.scheme))?;
        match index.check_code(&code) {
            CodeCheck::Exact { .. } | CodeCheck::Within { .. } => {}
            check => {
                return Err(Error::new(
                    ErrorCode::InvalidCode,
                    format!("`{code}` is not a class in {}", old.scheme),
                )
                .details(json!({ "check": check }))
                .next(format!(
                    "shelfmark scheme search <terms> --scheme {}",
                    old.scheme
                )));
            }
        }
        let class = ClassId::new(old.scheme.clone(), code);
        let claim = human_claim(&old, class, args.str("author")?);
        self.commit_claim(req, store.as_mut(), old, claim)
    }

    /// Append a human claim and its gold example (R15).
    fn commit_claim(
        &self,
        req: &Request,
        store: &mut dyn Store,
        old: Placement,
        claim: Placement,
    ) -> Result<Value, Error> {
        // The input the Work was last classified from, so the eval can
        // re-run every profile on it; a claim from before inputs were kept
        // labels only the Work.
        let input = store.input(&claim.work)?.unwrap_or_else(|| ClassifyInput {
            work: Some(claim.work.clone()),
            ..Default::default()
        });
        let gold = GoldExample {
            work: claim.work.clone(),
            item: None,
            scheme: claim.scheme.clone(),
            class: claim.class.clone(),
            input,
            from_placement: Some(old.id.clone()),
            provenance: claim.provenance.clone(),
        };
        let change = json!({ "append": claim, "supersedes": old.id, "previous_class": old.class, "gold": gold });
        self.mutate(req, store, change, |s| {
            s.append(&claim)?;
            s.append_gold(&gold)
        })
    }

    pub(super) fn link(&self, req: &Request, args: &Args) -> Result<Value, Error> {
        let item: ItemId = args.require_id("item")?;
        // `work` is the pre-0.2 name of `content`, read for one release.
        let work: WorkId =
            match (args.id::<WorkId>("content")?, args.id::<WorkId>("work")?) {
                (Some(s), Some(w)) if s != w => return Err(crate::args::bad(
                    args.verb(),
                    "`content` and its old name `work` name different content; pass only `content`",
                )),
                (Some(s), _) | (None, Some(s)) => s,
                (None, None) => return Err(crate::args::bad(args.verb(), "`content` is required")),
            };
        let mut store = self.store()?;
        let existing: Vec<WorkId> = store
            .links()?
            .into_iter()
            .filter(|(i, _)| i == &item)
            .map(|(_, w)| w)
            .collect();
        if existing.is_empty() {
            // Only an ingested file can be linked (U7).
            return Err(
                Error::new(ErrorCode::NotFound, format!("{item} has not been ingested"))
                    .next("shelfmark classify --file <the file>"),
            );
        }
        // `work` duplicates `content` for one release (read-compat);
        // `specimen` is the public name of `item`.
        let change = json!({ "item": item, "specimen": item, "content": work, "work": work, "already_linked_to": existing });
        self.mutate(req, store.as_mut(), change, |s| {
            s.append_link(&item, &work).map(drop)
        })
    }

    pub(super) fn scheme_compile(&self, req: &Request, args: &Args) -> Result<Value, Error> {
        let path = args.require("path")?;
        let doc = read_scheme(path)?;
        let index = (self.wiring.compile)(&doc).map_err(|e| {
            e.next(format!(
                "shelfmark scheme validate --path {}",
                shell_quote(path)
            ))
        })?;
        let mut store = self.store()?;
        let replaces = self
            .indexes(store.as_ref())?
            .into_iter()
            .find(|i| i.scheme() == index.scheme())
            .map(|i| i.content_hash().to_string());
        let change = json!({
            "scheme": index.scheme(), "classes": index.len(), "content_hash": index.content_hash(),
            "source": path, "replaces_content_hash": replaces,
        });
        self.mutate(req, store.as_mut(), change, |s| s.save_scheme(&doc))
    }
}

/// A claim that can still be confirmed or corrected: present, not
/// superseded.
fn live_claim(store: &dyn Store, id: &PlacementId) -> Result<Placement, Error> {
    let claims = store.claims()?;
    let p = find_placement(&claims, id)?;
    if effective_status(p, &superseded_ids(&claims)) == Status::Superseded {
        return Err(
            Error::new(ErrorCode::InvalidRequest, format!("{id} is superseded"))
                .next(format!("shelfmark explain {}", p.work)),
        );
    }
    Ok(p.clone())
}

/// A confirmed claim by a person, replacing `old`.
fn human_claim(old: &Placement, class: ClassId, author: Option<&str>) -> Placement {
    let author = author
        .map(str::to_string)
        .or_else(|| std::env::var("USER").ok())
        .unwrap_or_else(|| "unknown".into());
    Placement {
        class,
        confidence: 1.0,
        status: Status::Confirmed,
        supersedes: Some(old.id.clone()),
        provenance: Provenance {
            author_kind: AuthorKind::Human,
            author,
            model: None,
            harness: Some(format!("shelfmark-cli {}", env!("CARGO_PKG_VERSION"))),
            run_id: None,
            prompt_hash: None,
            at: crate::time::now_rfc3339(),
        },
        ..old.clone()
    }
    .seal()
}

/// Read and parse a scheme file; shape errors are `scheme_invalid`.
pub(super) fn read_scheme(path: &str) -> Result<SchemeDoc, Error> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        Error::new(
            ErrorCode::InputUnreadable,
            format!("cannot read {path}: {e}"),
        )
        .next("shelfmark describe scheme-compile")
    })?;
    serde_json::from_str(&text).map_err(|e| {
        Error::new(
            ErrorCode::SchemeInvalid,
            format!("{path} is not a valid scheme file: {e}"),
        )
        .next(format!(
            "shelfmark scheme validate --path {}",
            shell_quote(path)
        ))
    })
}
