//! `stale`: current placements a scheme or profile change makes stale
//! (R14, AE6).
//!
//! A placement is checked against the newest compiled version of its
//! scheme. Against the same version it is stale when the scheme's content
//! changed. Against a newer version it is stale only when its own class
//! changed there (gone, relabelled, moved, re-ranged, or split into
//! different children), so a version that splits one class lists exactly
//! the Works placed in that class.

use std::sync::Arc;

use serde_json::{json, Value};
use shelfmark_core::placement::AuthorKind;
use shelfmark_core::scheme::CompiledIndex;
use shelfmark_core::{Error, ProfileId, SchemeId};

use super::Shelfmark;
use crate::args::Args;
use crate::store::Store;

impl Shelfmark {
    pub(super) fn stale(&self, args: &Args) -> Result<Value, Error> {
        let scheme = args.id::<SchemeId>("scheme")?;
        let profile = args.id::<ProfileId>("profile")?;
        let store = self.store()?;
        let stale = self.stale_list(store.as_ref(), scheme.as_ref(), profile.as_ref())?;
        Ok(json!({ "total": stale.len(), "stale": stale }))
    }

    /// `scheme` filters by scheme name, so `stale --scheme lcc@2025` lists
    /// the `lcc@2024` placements that version makes stale.
    pub(super) fn stale_list(
        &self,
        store: &dyn Store,
        scheme: Option<&SchemeId>,
        profile: Option<&ProfileId>,
    ) -> Result<Vec<Value>, Error> {
        let indexes = self.indexes(store)?;
        let mut out = Vec::new();
        for p in Self::currents(&store.claims()?) {
            if scheme.is_some_and(|s| s.0.name != p.scheme.0.name)
                || profile.is_some_and(|q| q != &p.profile)
            {
                continue;
            }
            let mut reasons = Vec::new();
            let same = indexes.iter().find(|i| i.scheme() == &p.scheme);
            match (same, newest(&indexes, &p.scheme)) {
                (Some(cur), Some(new)) if new.scheme() == cur.scheme() => {
                    if cur.content_hash() != p.inputs.scheme {
                        reasons.push("scheme changed".to_string());
                    }
                }
                (Some(old), Some(new)) => {
                    if class_changed(old.as_ref(), new.as_ref(), &p.class.code) {
                        reasons.push(format!("class changed in {}", new.scheme()));
                    }
                }
                _ => reasons.push("scheme is not compiled".to_string()),
            }
            if p.provenance.author_kind != AuthorKind::Human {
                match self.wiring.profiles.iter().find(|q| q.id == p.profile) {
                    None => reasons.push("profile no longer exists".to_string()),
                    Some(q) if self.profile_hash(q) != p.inputs.profile => {
                        reasons.push("profile changed".to_string())
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

/// The compiled version of `scheme`'s name with the greatest version.
fn newest<'a>(
    indexes: &'a [Arc<dyn CompiledIndex>],
    scheme: &SchemeId,
) -> Option<&'a Arc<dyn CompiledIndex>> {
    indexes
        .iter()
        .filter(|i| i.scheme().0.name == scheme.0.name)
        .max_by(|a, b| a.scheme().0.version.cmp(&b.scheme().0.version))
}

/// Whether `code` means something different in `new` than in `old`. A code
/// below the scheme's classes is judged by the class that contains it.
fn class_changed(old: &dyn CompiledIndex, new: &dyn CompiledIndex, code: &str) -> bool {
    let code = match old.check_code(code) {
        shelfmark_core::scheme::CodeCheck::Within { class } => class.code,
        _ => code.to_string(),
    };
    let (Some(a), Some(b)) = (old.get(&code), new.get(&code)) else {
        return true;
    };
    let parent =
        |r: &shelfmark_core::scheme::ClassRecord| r.parent.as_ref().map(|c| c.code.clone());
    let kids = |i: &dyn CompiledIndex| -> Vec<String> {
        i.children(Some(&code))
            .into_iter()
            .map(|c| c.id.code)
            .collect()
    };
    a.label != b.label || parent(&a) != parent(&b) || a.range != b.range || kids(old) != kids(new)
}
