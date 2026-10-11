//! URL building, the disk cache, pacing and bounded retries, all over
//! the fixture transport.

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::{client, fixture, query, url};
use shelfmark_core::error::ErrorCode;
use shelfmark_tier_openlibrary::{DiskCache, FixtureTransport, Response, TransportError};

fn status(code: u16) -> Result<Response, TransportError> {
    Ok(Response {
        status: code,
        body: String::new(),
    })
}

#[test]
fn url_encodes_title_and_first_author() {
    let u = url(&query(
        "Ender's Game: 25th anniversary",
        &["Orson Scott Card", "x"],
    ));
    assert_eq!(
        u,
        "https://openlibrary.org/search.json?title=Ender%27s+Game%3A+25th+anniversary\
         &author=card\
         &fields=key%2Ctitle%2Csubtitle%2Cauthor_name%2Clcc%2Cddc%2Clc_classifications\
         %2Ceditions%2Ceditions.key%2Ceditions.title&limit=10"
    );
    assert!(!url(&query("Dune", &[])).contains("author="));
    // The family name, either way the name is written.
    assert!(url(&query("Leviathan", &["Hobbes, Thomas"])).contains("&author=hobbes&"));
    assert!(url(&query("The Road to Serfdom", &["F. A. Hayek"])).contains("&author=hayek&"));
    // Initials alone name no family; the next author is used.
    assert!(url(&query("X", &["J.", "Ada Lovelace"])).contains("&author=lovelace&"));
}

#[test]
fn search_parses_a_recorded_page() {
    let q = query("The Hobbit", &["J. R. R. Tolkien"]);
    let t = Arc::new(FixtureTransport::new().with(url(&q), fixture("hobbit.json")));
    let (c, _) = client(t.clone(), None);
    let docs = c.search(&q).unwrap();
    assert_eq!(docs.len(), 3);
    assert_eq!(docs[0].key, "/works/OL27482W");
    assert!(docs[0].lcc.len() > 10);
    assert!(!docs[0].ddc.is_empty());
}

#[test]
fn cache_serves_repeats_without_a_request() {
    let dir = tempfile::tempdir().unwrap();
    let q = query("The Hobbit", &["J. R. R. Tolkien"]);
    let t = Arc::new(FixtureTransport::new().with(url(&q), fixture("hobbit.json")));
    let (c, waits) = client(t.clone(), Some(DiskCache::new(dir.path().join("ol"))));
    let first = c.search(&q).unwrap();
    let second = c.search(&q).unwrap();
    assert_eq!(first, second);
    assert_eq!(t.requests().len(), 1);
    assert!(
        waits.lock().unwrap().is_empty(),
        "only one request went out"
    );
    let cached = DiskCache::new(dir.path().join("ol")).get(&url(&q)).unwrap();
    assert_eq!(cached, fixture("hobbit.json"));

    // A fresh client over the same directory and a dead transport still
    // answers from the cache.
    let dead = Arc::new(FixtureTransport::new());
    let (c2, _) = client(dead.clone(), Some(DiskCache::new(dir.path().join("ol"))));
    assert_eq!(c2.search(&q).unwrap(), first);
    assert!(dead.requests().is_empty());
}

#[test]
fn failures_and_bad_bodies_are_not_cached() {
    let dir = tempfile::tempdir().unwrap();
    let cache = DiskCache::new(dir.path());
    let q = query("Dune", &[]);
    let t = Arc::new(FixtureTransport::new().with(url(&q), "<html>busy</html>"));
    let (c, _) = client(t, Some(cache.clone()));
    let err = c.search(&q).unwrap_err();
    assert_eq!(err.code, ErrorCode::TierUnavailable);
    assert!(cache.get(&url(&q)).is_none());

    let t = Arc::new(FixtureTransport::new());
    let (c, _) = client(t, Some(cache.clone()));
    assert_eq!(c.search(&q).unwrap_err().code, ErrorCode::TierUnavailable);
    assert!(cache.get(&url(&q)).is_none());
}

#[test]
fn a_corrupt_cache_file_is_a_miss() {
    let dir = tempfile::tempdir().unwrap();
    let cache = DiskCache::new(dir.path());
    let q = query("The Hobbit", &["J. R. R. Tolkien"]);
    cache.put(&url(&q), "{not json");
    let t = Arc::new(FixtureTransport::new().with(url(&q), fixture("hobbit.json")));
    let (c, _) = client(t.clone(), Some(cache.clone()));
    assert_eq!(c.search(&q).unwrap().len(), 3);
    assert_eq!(t.requests().len(), 1);
    assert_eq!(cache.get(&url(&q)).unwrap(), fixture("hobbit.json"));
}

#[test]
fn requests_are_paced_one_per_interval() {
    let q1 = query("The Hobbit", &["J. R. R. Tolkien"]);
    let q2 = query(
        "Structure and Interpretation of Computer Programs",
        &["Harold Abelson"],
    );
    let t = Arc::new(
        FixtureTransport::new()
            .with(url(&q1), fixture("hobbit.json"))
            .with(url(&q2), fixture("sicp.json")),
    );
    let (c, waits) = client(t.clone(), None);
    c.search(&q1).unwrap();
    c.search(&q2).unwrap();
    c.search(&q1).unwrap();
    assert_eq!(t.requests().len(), 3);
    let w = waits.lock().unwrap().clone();
    assert_eq!(w.len(), 2, "every request after the first waited");
    // The fake sleeper does not advance time, so each wait is close to
    // the full interval.
    assert!(w
        .iter()
        .all(|d| *d > Duration::from_millis(900) && *d <= Duration::from_secs(1)));
}

#[test]
fn retries_transient_failures_with_backoff() {
    let q = query("The Hobbit", &["J. R. R. Tolkien"]);
    let u = url(&q);
    let t = Arc::new(
        FixtureTransport::new()
            .with(u.clone(), fixture("hobbit.json"))
            .queue(u.clone(), status(503))
            .queue(u.clone(), Err(TransportError::Timeout)),
    );
    let (c, waits) = client(t.clone(), None);
    assert_eq!(c.search(&q).unwrap().len(), 3);
    assert_eq!(t.requests().len(), 3);
    let w = waits.lock().unwrap().clone();
    // Backoff 1 s then 2 s (each at least the pacing interval).
    assert_eq!(w.len(), 2);
    assert!(w[0] > Duration::from_millis(900) && w[0] <= Duration::from_secs(1));
    assert!(w[1] > Duration::from_millis(1900) && w[1] <= Duration::from_secs(2));
}

#[test]
fn retries_are_bounded() {
    let q = query("Dune", &["Frank Herbert"]);
    let u = url(&q);
    let t = Arc::new(
        FixtureTransport::new()
            .queue(u.clone(), status(429))
            .queue(u.clone(), Err(TransportError::Network("reset".into())))
            .queue(u.clone(), status(502))
            .queue(u.clone(), status(200)),
    );
    let (c, _) = client(t.clone(), None);
    let err = c.search(&q).unwrap_err();
    assert_eq!(err.code, ErrorCode::TierUnavailable);
    assert!(err.message.contains("3 attempts"), "{}", err.message);
    assert!(err.message.contains("HTTP 502"), "{}", err.message);
    assert_eq!(t.requests().len(), 3, "one try plus two retries");
}

#[test]
fn client_errors_are_not_retried() {
    let q = query("Dune", &[]);
    let t = Arc::new(FixtureTransport::new());
    let (c, _) = client(t.clone(), None);
    let err = c.search(&q).unwrap_err();
    assert_eq!(err.code, ErrorCode::TierUnavailable);
    assert!(err.message.contains("HTTP 404"));
    assert_eq!(t.requests().len(), 1);
    assert!(err.next_command.is_some());
}
