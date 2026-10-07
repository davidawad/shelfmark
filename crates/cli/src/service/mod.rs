//! [`Shelfmark`]: the one dispatcher over every verb (PD8). The CLI, gRPC
//! and the kb resource all call [`Service::call`] on it.

mod classify;
mod mutate;
mod read;
mod scheme;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::{json, Value};
use shelfmark_core::envelope::Mode;
use shelfmark_core::placement::{current, Placement};
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::tier::Profile;
use shelfmark_core::{
    Envelope, Error, ErrorCode, ProfileId, Request, SchemeId, Service, Verb, WorkId,
};

use crate::args::{bad, Args};
use crate::describe;
use crate::store::Store;
use crate::wiring::Wiring;

pub struct Shelfmark {
    wiring: Wiring,
    store: Mutex<Box<dyn Store>>,
}

impl Service for Shelfmark {
    fn call(&self, request: Request) -> Envelope {
        let verb = request.verb;
        Envelope::from_result(self.dispatch(&request).map_err(|mut e| {
            // R21: every error names a next command.
            if e.next_command.is_none() {
                e.next_command = Some(format!("shelfmark describe {}", verb.as_str()));
            }
            e
        }))
    }
}

impl Shelfmark {
    pub fn new(wiring: Wiring, store: Box<dyn Store>) -> Self {
        Shelfmark {
            wiring,
            store: Mutex::new(store),
        }
    }

    fn dispatch(&self, req: &Request) -> Result<Value, Error> {
        let spec = describe::spec(req.verb);
        let args = Args::new(req.verb, &req.args, &spec.arg_names())?;
        if req.mode == Mode::Commit && !req.verb.mutates() {
            return Err(bad(
                req.verb,
                format!(
                    "`{}` does not mutate, so it has no commit mode; drop --commit",
                    req.verb.as_str()
                ),
            ));
        }
        if req.mode == Mode::Commit {
            // A stale preview is a conflict before anything else is checked.
            check_revision(req, &self.store()?.revision()?)?;
        }
        match req.verb {
            Verb::Describe => self.describe(&args),
            Verb::Status => self.status(),
            Verb::Resolve => self.resolve(&args),
            Verb::Classify => self.classify(&args),
            Verb::Explain => self.explain(&args),
            Verb::ListFlagged => self.list_flagged(&args),
            Verb::Stale => self.stale(&args),
            Verb::Confirm => self.confirm(req, &args),
            Verb::Correct => self.correct(req, &args),
            Verb::Link => self.link(req, &args),
            Verb::SchemeCompile => self.scheme_compile(req, &args),
            Verb::SchemeBrowse => self.scheme_browse(&args),
            Verb::SchemeSearch => self.scheme_search(&args),
            Verb::SchemeValidate => self.scheme_validate(&args),
            Verb::Eval => self.eval(&args),
            Verb::Name => self.name(&args),
        }
    }

    fn store(&self) -> Result<MutexGuard<'_, Box<dyn Store>>, Error> {
        self.store
            .lock()
            .map_err(|_| Error::internal("the store lock was poisoned by an earlier panic"))
    }

    /// Every compiled scheme: the store's (which win on the same ID, being
    /// newer) then the built-in ones.
    fn indexes(&self, store: &dyn Store) -> Result<Vec<Arc<dyn CompiledIndex>>, Error> {
        let mut out: Vec<Arc<dyn CompiledIndex>> = Vec::new();
        for doc in store.schemes()? {
            out.push((self.wiring.compile)(&doc)?);
        }
        for index in &self.wiring.schemes {
            if !out.iter().any(|i| i.scheme() == index.scheme()) {
                out.push(index.clone());
            }
        }
        Ok(out)
    }

    /// The named scheme's index, or the default (first compiled) one.
    fn index(
        &self,
        store: &dyn Store,
        scheme: Option<&SchemeId>,
    ) -> Result<Arc<dyn CompiledIndex>, Error> {
        let all = self.indexes(store)?;
        let Some(id) = scheme else {
            return all.into_iter().next().ok_or_else(|| {
                Error::new(ErrorCode::SchemeNotCompiled, "no scheme is compiled")
                    .next("shelfmark scheme compile <scheme.json>")
            });
        };
        if let Some(index) = all.into_iter().find(|i| i.scheme() == id) {
            return Ok(index);
        }
        match self.wiring.known.iter().find(|(k, _)| k == id) {
            Some((_, source)) => Err(Error::scheme_not_compiled(id, source.as_deref())),
            None => Err(
                Error::new(ErrorCode::NotFound, format!("no scheme {id} is known"))
                    .next("shelfmark status"),
            ),
        }
    }

    fn scheme_arg(&self, store: &dyn Store, args: &Args) -> Result<Arc<dyn CompiledIndex>, Error> {
        self.index(store, args.id::<SchemeId>("scheme")?.as_ref())
    }

    fn profile(&self, id: Option<&ProfileId>) -> Result<Profile, Error> {
        let Some(id) = id else {
            return Ok(self.recommended().clone());
        };
        self.wiring
            .profiles
            .iter()
            .find(|p| &p.id == id)
            .cloned()
            .ok_or_else(|| {
                Error::new(ErrorCode::ProfileNotFound, format!("no profile {id}"))
                    .next("shelfmark describe")
            })
    }

    /// A profile runs here when every one of its tiers is built in.
    fn runnable(&self, p: &Profile) -> bool {
        p.tiers.iter().all(|t| self.wiring.tiers.contains_key(t))
    }

    /// The most capable runnable profile (R9), else the first declared.
    fn recommended(&self) -> &Profile {
        self.wiring
            .profiles
            .iter()
            .filter(|p| self.runnable(p))
            .max_by_key(|p| p.tiers.len())
            .unwrap_or(&self.wiring.profiles[0])
    }

    /// Hash of the profile definition plus its tiers' versions (R14).
    fn profile_hash(&self, p: &Profile) -> String {
        let versions: Vec<Value> = p
            .tiers
            .iter()
            .map(|t| match self.wiring.tiers.get(t) {
                Some(tier) => json!([t, tier.info().version]),
                None => json!([t, null]),
            })
            .collect();
        shelfmark_core::calibration::config_hash(&json!({ "profile": p, "tiers": versions }))
    }

    /// The current placement per (work, scheme), in first-seen order.
    fn currents(claims: &[Placement]) -> Vec<Placement> {
        let mut groups: BTreeMap<(String, String), (usize, Vec<Placement>)> = BTreeMap::new();
        for (i, p) in claims.iter().enumerate() {
            groups
                .entry((p.work.to_string(), p.scheme.to_string()))
                .or_insert_with(|| (i, Vec::new()))
                .1
                .push(p.clone());
        }
        let mut out: Vec<(usize, Placement)> = groups
            .into_values()
            .filter_map(|(i, g)| current(&g).cloned().map(|p| (i, p)))
            .collect();
        out.sort_by_key(|(i, _)| *i);
        out.into_iter().map(|(_, p)| p).collect()
    }

    /// A Work's current placements, optionally in one scheme.
    fn work_placements(
        &self,
        store: &dyn Store,
        work: &WorkId,
        scheme: Option<&SchemeId>,
    ) -> Result<Vec<Placement>, Error> {
        let claims: Vec<Placement> = store
            .claims()?
            .into_iter()
            .filter(|p| &p.work == work && scheme.is_none_or(|s| &p.scheme == s))
            .collect();
        Ok(Self::currents(&claims))
    }

    /// Preview by default; commit only at the expected revision.
    fn mutate(
        &self,
        req: &Request,
        store: &mut dyn Store,
        change: Value,
        apply: impl FnOnce(&mut dyn Store) -> Result<(), Error>,
    ) -> Result<Value, Error> {
        let revision = store.revision()?;
        let preview = describe::command(req.verb, &req.args);
        match req.mode {
            Mode::Preview => Ok(json!({
                "mode": "preview",
                "revision": revision,
                "change": change,
                "commit_command": format!("{preview} --commit --expected-revision {revision}"),
            })),
            Mode::Commit => {
                check_revision(req, &revision)?;
                apply(store)?;
                Ok(json!({
                    "mode": "commit",
                    "previous_revision": revision,
                    "revision": store.revision()?,
                    "change": change,
                }))
            }
        }
    }
}

/// `id` names a placement in `claims`, or fails with how to find one.
fn find_placement<'a>(
    claims: &'a [Placement],
    id: &shelfmark_core::PlacementId,
) -> Result<&'a Placement, Error> {
    claims.iter().find(|p| &p.id == id).ok_or_else(|| {
        Error::new(
            ErrorCode::NotFound,
            format!("no placement {id} in this store"),
        )
        .next("shelfmark list-flagged")
    })
}

/// A commit must name the revision its preview saw, and it must still be
/// current.
fn check_revision(req: &Request, current: &str) -> Result<(), Error> {
    let preview = describe::command(req.verb, &req.args);
    let Some(expected) = req.expected_revision.as_deref() else {
        return Err(Error::new(
            ErrorCode::RevisionRequired,
            "commit needs --expected-revision from a preview",
        )
        .next(preview));
    };
    if expected != current {
        return Err(Error::new(
            ErrorCode::RevisionConflict,
            "the store changed since that preview; preview again and review the change",
        )
        .details(json!({ "expected": expected, "current": current }))
        .next(preview));
    }
    Ok(())
}
