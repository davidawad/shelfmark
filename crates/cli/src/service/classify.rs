//! `classify`: run a profile's cascade and append the claim, reusing a
//! memoized claim when the inputs are unchanged (R7, R10, R11, R14). A
//! file is minted and ingested first (R1-R3), so its Work is the one an
//! accepted link points at and near-duplicates come back as proposals.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use shelfmark_core::calibration::config_hash;
use shelfmark_core::cascade;
use shelfmark_core::placement::{current, AuthorKind, InputHashes, Placement, Provenance, Status};
use shelfmark_core::tier::{ClassifyInput, Tier};
use shelfmark_core::{Error, ProfileId, Verb, WorkId};

use super::Shelfmark;
use crate::args::{bad, Args};
use shelfmark_identity::{Ingested, Minted};

use crate::wiring::{parse_identifier, url_domain};

impl Shelfmark {
    pub(super) fn classify(&self, args: &Args) -> Result<Value, Error> {
        let (mut input, minted) = classify_input(args)?;
        let profile = self.profile(args.id::<ProfileId>("profile")?.as_ref())?;
        let target = args.f64("target")?;
        let mut store = self.store()?;
        let ingested = match &minted {
            Some(m) => Some(store.ingest(m)?),
            None => None,
        };
        if let Some(ing) = &ingested {
            input.work = Some(ing.work.clone());
        }
        let item = ingested.as_ref().map(|i| i.item.clone());
        let index = self.scheme_arg(store.as_ref(), args)?;
        let calibration = store.calibration()?;
        let work = input.work.clone().expect("classify_input sets the work");
        // What a later confirm or correct labels as gold (R15).
        store.save_input(&input)?;
        let inputs = InputHashes {
            work: work.clone(),
            scheme: index.content_hash().to_string(),
            profile: self.profile_hash(&profile),
            config: config_hash(&json!({
                "target": target.unwrap_or(profile.default_target),
                // The whole fit: a refit on the same gold (after a rules
                // or table change) must not serve claims made under the old one.
                "calibration": calibration.as_ref().map(|c| config_hash(&json!(c))),
            })),
        };
        if let Some(hit) = store.memo(&inputs)? {
            let mut out = json!({ "placement": hit, "memo_hit": true, "item": item });
            add_ingest(&mut out, ingested.as_ref());
            return Ok(out);
        }
        let tiers: BTreeMap<String, &dyn Tier> = self
            .wiring
            .tiers
            .iter()
            .map(|(n, t)| (n.clone(), t.as_ref()))
            .collect();
        let decision = cascade::run(
            &profile,
            &tiers,
            index.as_ref(),
            &input,
            target,
            calibration.as_ref(),
        )?;
        // A re-run with changed inputs replaces the live machine claim; a
        // confirmed claim already outranks it and is left alone.
        let previous = store.claims_for(&work, index.scheme())?;
        let supersedes = current(&previous)
            .filter(|p| p.status != Status::Confirmed)
            .map(|p| p.id.clone());
        let placement = Placement {
            // Placeholder: `seal` sets the content ID.
            id: shelfmark_core::PlacementId::from_canonical(b""),
            work,
            scheme: index.scheme().clone(),
            class: decision.class,
            profile: profile.id.clone(),
            confidence: decision.confidence,
            status: decision.status,
            supersedes,
            inputs,
            trace: decision.trace,
            provenance: Provenance {
                author_kind: AuthorKind::Tool,
                author: "shelfmark".into(),
                model: None,
                harness: Some(format!("shelfmark-cli {}", env!("CARGO_PKG_VERSION"))),
                run_id: None,
                prompt_hash: None,
                at: crate::time::now_rfc3339(),
            },
        }
        .seal();
        store.append(&placement)?;
        let mut out = json!({ "placement": placement, "memo_hit": false, "item": item });
        if placement.status == Status::Flagged {
            out["next_command"] = json!(format!("shelfmark explain {}", placement.id));
        }
        add_ingest(&mut out, ingested.as_ref());
        Ok(out)
    }
}

/// What ingesting the file did, and the link command for the closest
/// near-duplicate when there is one (R3). Linking comes before explaining
/// a flagged placement: once linked, the copy shares the existing Work's
/// placements.
fn add_ingest(out: &mut Value, ingested: Option<&Ingested>) {
    let Some(ing) = ingested else { return };
    out["ingest"] = json!({
        "new_item": ing.new_item,
        "new_specimen": ing.new_work,
        "minted_specimen": ing.minted_work,
        // Pre-0.2 names, kept for one release.
        "new_work": ing.new_work,
        "minted_work": ing.minted_work,
        "format": ing.format.as_str(),
        "proposals": ing.proposals,
    });
    if let Some(p) = ing.proposals.first() {
        out["next_command"] = json!(format!(
            "shelfmark link --item {} --specimen {}",
            ing.item, p.work
        ));
    }
}

/// The classify input from `text`/`file`, `title`, `authors`, `identifiers`, `url`
/// and `headings`, with its Work ID, plus the minted file if one was given.
pub(super) fn classify_input(args: &Args) -> Result<(ClassifyInput, Option<Minted>), Error> {
    let mut minted = None;
    let mut text = args.str("text")?.map(str::to_string);
    if let Some(path) = args.str("file")? {
        if text.is_some() {
            return Err(bad(Verb::Classify, "give `text` or `file`, not both"));
        }
        let m = Minted::from_path(std::path::Path::new(path)).map_err(|e| {
            if e.next_command.is_some() {
                return e;
            }
            e.next(format!(
                "shelfmark classify --text <text of {}>",
                crate::describe::shell_quote(path)
            ))
        })?;
        text = Some(m.text.clone());
        minted = Some(m);
    }
    let mut identifiers = Vec::new();
    for s in args.strings("identifiers")? {
        identifiers.push(parse_identifier(&s).ok_or_else(|| {
            bad(
                Verb::Classify,
                format!("`{s}` is not an identifier (isbn:, lccn:, doi:, arxiv:, domain:)"),
            )
        })?);
    }
    let url = args.str("url")?.map(str::to_string);
    identifiers.extend(url.as_deref().and_then(url_domain));
    identifiers.sort();
    identifiers.dedup();
    let input = ClassifyInput {
        work: None,
        title: args.str("title")?.map(str::to_string),
        authors: args.strings("authors")?,
        text: text.unwrap_or_default(),
        identifiers,
        url,
        headings: args.strings("headings")?,
    };
    let input = with_work(input).ok_or_else(|| {
        bad(
            Verb::Classify,
            "nothing to classify: give text, file, title, identifiers, url or headings",
        )
    })?;
    Ok((input, minted))
}

/// The input with its Work ID set: the Work is its text (PD2); without
/// text, what was supplied. `None` when nothing classifiable is left.
pub(super) fn with_work(input: ClassifyInput) -> Option<ClassifyInput> {
    let basis = if input.text.trim().is_empty() {
        let mut parts: Vec<String> = input.title.iter().cloned().collect();
        parts.extend(input.authors.iter().cloned());
        parts.extend(input.identifiers.iter().map(|i| i.key()));
        parts.extend(input.headings.iter().cloned());
        parts.join("\n")
    } else {
        input.text.clone()
    };
    if shelfmark_core::id::normalize_text_v1(&basis).is_empty() {
        return None;
    }
    Some(ClassifyInput {
        work: Some(WorkId::from_text(&basis)),
        ..input
    })
}
