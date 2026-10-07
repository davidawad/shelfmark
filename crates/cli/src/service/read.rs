//! Read-only verbs: describe, status, resolve, explain, list-flagged,
//! stale (R13, R14, R20, R23).

use serde_json::{json, Value};
use shelfmark_core::id::{AnyId, Kind};
use shelfmark_core::placement::{effective_status, superseded_ids, AuthorKind, Placement, Status};
use shelfmark_core::{Error, ErrorCode, ProfileId, SchemeId, Verb};

use super::{find_placement, Shelfmark};
use crate::args::{bad, Args};
use crate::describe::{self, VERBS};
use crate::store::Store;

const NOUNS: [(Kind, &str, &str, &str); 6] = [
    (
        Kind::Work,
        "L0",
        "work:nt1s256:<64 hex>",
        "a piece of content, whatever its file format",
    ),
    (
        Kind::Item,
        "L0",
        "item:s256:<64 hex>",
        "one file (hash of its bytes), linked to a Work",
    ),
    (
        Kind::Scheme,
        "L1",
        "scheme:<name>@<version>",
        "a classification scheme",
    ),
    (
        Kind::Class,
        "L1",
        "class:<name>@<version>:<code>",
        "one class in a scheme version",
    ),
    (
        Kind::Profile,
        "L2",
        "profile:<name>@<version>",
        "a named cascade of tiers",
    ),
    (
        Kind::Placement,
        "L3",
        "placement:s256:<64 hex>",
        "an append-only claim that a Work belongs in a class",
    ),
];

impl Shelfmark {
    pub(super) fn describe(&self, args: &Args) -> Result<Value, Error> {
        let nouns: Vec<Value> = NOUNS
            .iter()
            .map(|(k, layer, form, what)| json!({"kind": k, "layer": layer, "id_form": form, "names": what}))
            .collect();
        let errors: Vec<Value> = ErrorCode::ALL
            .iter()
            .map(|c| json!({"code": c, "exit_code": c.exit_code()}))
            .collect();
        let verb = |s: &describe::VerbSpec| {
            json!({
                "name": s.verb, "cli": format!("shelfmark {}", s.cli), "mutates": s.verb.mutates(),
                "summary": s.summary, "positional": s.positional, "args": s.args, "example": s.example,
            })
        };
        match args.str("topic")? {
            None => {}
            Some("ids") => return Ok(json!({ "nouns": nouns })),
            Some("errors") => return Ok(json!({ "error_codes": errors })),
            Some(t) => {
                return VERBS
                    .iter()
                    .find(|s| s.verb.as_str() == t || s.cli == t)
                    .map(verb)
                    .ok_or_else(|| {
                        bad(
                            Verb::Describe,
                            format!("no topic `{t}`; topics are verb names, `ids` and `errors`"),
                        )
                        .next("shelfmark describe")
                    });
            }
        }
        let store = self.store()?;
        let schemes = self.scheme_list(store.as_ref())?;
        Ok(json!({
            "tool": "shelfmark",
            "version": env!("CARGO_PKG_VERSION"),
            "nouns": nouns,
            "verbs": VERBS.iter().map(verb).collect::<Vec<_>>(),
            "profiles": self.profile_list(),
            "schemes": schemes,
            "conventions": {
                "output": "JSON envelope {ok, data|error} when stdout is not a terminal or with --json",
                "mutations": "confirm, correct, link and scheme compile preview by default; rerun with --commit --expected-revision <revision from the preview>",
                "errors": "every error has a stable code, a message and a next_command; the exit code follows the code (see `describe errors`)",
                "batch": "shelfmark batch FILE... --out results.ndjson [--resume]: one line per input keyed by its hash",
                "store": "--store DIR, else $SHELFMARK_STORE, else ./.shelfmark",
            },
            "error_codes": errors,
            "example": "shelfmark classify --title 'Engineering a Compiler' --text 'compilers (computer programs)' --json",
        }))
    }

    fn profile_list(&self) -> Vec<Value> {
        let rec = self.recommended().id.clone();
        self.wiring
            .profiles
            .iter()
            .map(|p| json!({
                "id": p.id, "description": p.description, "tiers": p.tiers,
                "expected_memory_mb": p.expected_memory_mb,
                "expected_micros_per_item": p.expected_micros_per_item,
                "runnable": self.runnable(p), "recommended": p.id == rec,
                "unavailable_tiers": p.tiers.iter().filter(|t| !self.wiring.tiers.contains_key(*t)).collect::<Vec<_>>(),
            }))
            .collect()
    }

    fn scheme_list(&self, store: &dyn Store) -> Result<Vec<Value>, Error> {
        let mut out: Vec<Value> = self
            .indexes(store)?
            .iter()
            .map(|i| json!({"id": i.scheme(), "compiled": true, "classes": i.len(), "content_hash": i.content_hash()}))
            .collect();
        for (id, source) in &self.wiring.known {
            if !out.iter().any(|s| s["id"] == json!(id)) {
                out.push(json!({"id": id, "compiled": false, "source": source}));
            }
        }
        Ok(out)
    }

    pub(super) fn status(&self) -> Result<Value, Error> {
        let store = self.store()?;
        let claims = store.claims()?;
        let currents = Self::currents(&claims);
        let flagged = currents
            .iter()
            .filter(|p| p.status == Status::Flagged)
            .count();
        let stale = self.stale_list(store.as_ref(), None, None)?;
        let gold = store.gold()?;
        let gold_hash = shelfmark_core::calibration::gold_set_hash(&gold);
        let calibration = store.calibration()?.map(|c| {
            json!({
                "scheme": c.scheme, "fitted_at": c.fitted_at, "gold_count": c.gold_count,
                "stale": c.is_stale(&gold_hash),
            })
        });
        Ok(json!({
            "store": store.location(),
            "revision": store.revision()?,
            "schemes": self.scheme_list(store.as_ref())?,
            "profiles": self.profile_list(),
            "recommended_profile": self.recommended().id,
            "machine": {
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "cpus": std::thread::available_parallelism().map_or(1, |n| n.get()),
            },
            "placements": { "claims": claims.len(), "current": currents.len() },
            "flagged_backlog": flagged,
            "stale_placements": stale.len(),
            "gold": { "count": gold.len(), "hash": gold_hash },
            "calibration": calibration,
            "eval": store.eval_summary()?,
        }))
    }

    pub(super) fn resolve(&self, args: &Args) -> Result<Value, Error> {
        let id: AnyId = args.require_id("id")?;
        let store = self.store()?;
        let data = match &id {
            AnyId::Work(w) => {
                let items: Vec<_> = store
                    .links()?
                    .into_iter()
                    .filter(|(_, lw)| lw == w)
                    .map(|(i, _)| i)
                    .collect();
                json!({"placements": self.work_placements(store.as_ref(), w, None)?, "items": items})
            }
            AnyId::Item(i) => {
                let work = store
                    .links()?
                    .into_iter()
                    .find(|(li, _)| li == i)
                    .map(|(_, w)| w);
                let work = work.ok_or_else(|| {
                    Error::new(ErrorCode::NotFound, format!("{i} is not linked to a Work"))
                        .next(format!("shelfmark link --item {i} --work <work-id>"))
                })?;
                json!({ "work": work })
            }
            AnyId::Scheme(s) => {
                let index = self.index(store.as_ref(), Some(s))?;
                json!({"classes": index.len(), "content_hash": index.content_hash(), "top_level": index.children(None)})
            }
            AnyId::Class(c) => {
                let index = self.index(store.as_ref(), Some(&c.scheme))?;
                let record = index.get(&c.code).ok_or_else(|| {
                    Error::new(
                        ErrorCode::NotFound,
                        format!("{c} is not a class in its scheme"),
                    )
                    .details(json!({ "check": index.check_code(&c.code) }))
                    .next(format!(
                        "shelfmark scheme validate {} --scheme {}",
                        c.code, c.scheme
                    ))
                })?;
                json!({"class": record, "ancestors": index.ancestors(&c.code), "children": index.children(Some(&c.code))})
            }
            AnyId::Profile(p) => {
                let profile = self.profile(Some(p))?;
                json!({"profile": profile, "runnable": self.runnable(&profile), "hash": self.profile_hash(&profile)})
            }
            AnyId::Placement(p) => {
                let claims = store.claims()?;
                let found = find_placement(&claims, p)?;
                json!({"placement": found, "status": effective_status(found, &superseded_ids(&claims))})
            }
        };
        Ok(json!({ "id": id, "kind": id.kind(), "resolved": data }))
    }

    pub(super) fn explain(&self, args: &Args) -> Result<Value, Error> {
        let id: AnyId = args.require_id("id")?;
        let store = self.store()?;
        let claims = store.claims()?;
        let superseded = superseded_ids(&claims);
        let placements = match &id {
            AnyId::Placement(p) => vec![find_placement(&claims, p)?.clone()],
            AnyId::Work(w) => {
                let scheme = args.id::<SchemeId>("scheme")?;
                let found = self.work_placements(store.as_ref(), w, scheme.as_ref())?;
                if found.is_empty() {
                    return Err(
                        Error::new(ErrorCode::NotFound, format!("{w} has no placements"))
                            .next("shelfmark classify --text <text>"),
                    );
                }
                found
            }
            other => {
                return Err(bad(
                    Verb::Explain,
                    format!(
                        "explain takes a placement: or work: ID, not {}:",
                        other.kind()
                    ),
                ))
            }
        };
        let out: Vec<Value> = placements
            .iter()
            .map(|p| json!({"summary": summary(p), "status": effective_status(p, &superseded), "placement": p}))
            .collect();
        Ok(json!({ "id": id, "explanations": out }))
    }

    pub(super) fn list_flagged(&self, args: &Args) -> Result<Value, Error> {
        let scheme = args.id::<SchemeId>("scheme")?;
        let limit = args.usize("limit")?.unwrap_or(50);
        let store = self.store()?;
        let mut flagged: Vec<Placement> = Self::currents(&store.claims()?)
            .into_iter()
            .filter(|p| {
                p.status == Status::Flagged && scheme.as_ref().is_none_or(|s| &p.scheme == s)
            })
            .collect();
        flagged.sort_by(|a, b| {
            a.confidence
                .total_cmp(&b.confidence)
                .then_with(|| a.id.cmp(&b.id))
        });
        let total = flagged.len();
        let items: Vec<Value> = flagged
            .iter()
            .take(limit)
            .map(|p| json!({
                "placement": p.id, "work": p.work, "class": p.class, "confidence": p.confidence,
                "flag_reason": p.trace.flag_reason, "next_command": format!("shelfmark explain {}", p.id),
            }))
            .collect();
        Ok(json!({ "total": total, "flagged": items }))
    }

    pub(super) fn stale(&self, args: &Args) -> Result<Value, Error> {
        let scheme = args.id::<SchemeId>("scheme")?;
        let profile = args.id::<ProfileId>("profile")?;
        let store = self.store()?;
        let stale = self.stale_list(store.as_ref(), scheme.as_ref(), profile.as_ref())?;
        Ok(json!({ "total": stale.len(), "stale": stale }))
    }

    /// Current placements whose recorded scheme or profile hash no longer
    /// matches what this build would use (R14, AE6).
    fn stale_list(
        &self,
        store: &dyn Store,
        scheme: Option<&SchemeId>,
        profile: Option<&ProfileId>,
    ) -> Result<Vec<Value>, Error> {
        let indexes = self.indexes(store)?;
        let mut out = Vec::new();
        for p in Self::currents(&store.claims()?) {
            if scheme.is_some_and(|s| s != &p.scheme) || profile.is_some_and(|q| q != &p.profile) {
                continue;
            }
            let mut reasons = Vec::new();
            match indexes.iter().find(|i| i.scheme() == &p.scheme) {
                None => reasons.push("scheme is not compiled"),
                Some(i) if i.content_hash() != p.inputs.scheme => reasons.push("scheme changed"),
                Some(_) => {}
            }
            if p.provenance.author_kind != AuthorKind::Human {
                match self.wiring.profiles.iter().find(|q| q.id == p.profile) {
                    None => reasons.push("profile no longer exists"),
                    Some(q) if self.profile_hash(q) != p.inputs.profile => {
                        reasons.push("profile changed")
                    }
                    Some(_) => {}
                }
            }
            if !reasons.is_empty() {
                out.push(json!({"placement": p.id, "work": p.work, "class": p.class, "status": p.status, "reasons": reasons}));
            }
        }
        Ok(out)
    }
}

/// One line saying what decided a placement.
fn summary(p: &Placement) -> String {
    let t = &p.trace;
    match (&p.status, &t.deciding_tier) {
        (Status::Confirmed, _) => format!("{} confirmed by {}", p.class, p.provenance.author),
        (_, Some(tier)) => format!(
            "{} at confidence {:.3}: tier `{tier}` reached target {:.3} with margin {:.3}",
            p.class, p.confidence, t.target, t.margin
        ),
        (_, None) => format!(
            "{} at confidence {:.3}, flagged: {} ({} candidates)",
            p.class,
            p.confidence,
            t.flag_reason.as_deref().unwrap_or("no tier decided"),
            t.candidates.len()
        ),
    }
}
