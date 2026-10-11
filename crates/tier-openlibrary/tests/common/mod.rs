//! Shared fixtures: the bundled LCC outline, recorded search pages and a
//! client over the fixture transport that never sleeps.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use shelfmark_scheme::SchemeIndex;
use shelfmark_tier_openlibrary::{
    Client, ClientConfig, DiskCache, FixtureTransport, Pacer, Query, Response, Transport,
    TransportError,
};

/// The LCC outline (`data/schemes/lcc/lcc.json`), compiled once.
pub fn lcc() -> &'static SchemeIndex {
    static INDEX: OnceLock<SchemeIndex> = OnceLock::new();
    INDEX.get_or_init(|| {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/schemes/lcc/lcc.json");
        shelfmark_scheme::compile_file(&path).expect("outline compiles")
    })
}

/// A recorded or synthetic search page from `tests/fixtures/`.
pub fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(path).expect("fixture exists")
}

pub fn query(title: &str, authors: &[&str]) -> Query {
    Query {
        title: title.into(),
        authors: authors.iter().map(|a| (*a).to_string()).collect(),
    }
}

/// The URL the default client sends for `q`.
pub fn url(q: &Query) -> String {
    Client::new(Box::new(FixtureTransport::new()), None).url(q)
}

/// The waits a pacer asked for.
pub type Waits = Arc<Mutex<Vec<Duration>>>;

/// A transport shared with the test, so it can read the request log
/// after the client took ownership.
pub struct Shared(pub Arc<FixtureTransport>);

impl Transport for Shared {
    fn get(&self, url: &str, timeout: Duration) -> Result<Response, TransportError> {
        self.0.get(url, timeout)
    }
}

/// A client over `transport` pacing at 1 s, recording waits instead of
/// sleeping, with a 1 s retry backoff.
pub fn client(transport: Arc<FixtureTransport>, cache: Option<DiskCache>) -> (Client, Waits) {
    let waits: Waits = Arc::default();
    let w = waits.clone();
    let pacer = Pacer::with_sleeper(
        Duration::from_secs(1),
        Box::new(move |d| w.lock().unwrap().push(d)),
    );
    let config = ClientConfig {
        backoff: Duration::from_secs(1),
        ..ClientConfig::default()
    };
    (
        Client::with(Box::new(Shared(transport)), cache, pacer, config),
        waits,
    )
}
