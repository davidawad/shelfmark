//! The Open Library search client: URL building, cache, pacing and
//! retries over a [`Transport`].
//!
//! Search API: <https://openlibrary.org/dev/docs/api/search>. One query
//! is `search.json?title=..&author=..&fields=..&limit=..`. `author` is
//! the first author's family name: Open Library returns only works
//! matching every word of both fields (the author under any name form it
//! knows), so "F. A. Hayek" would miss "Friedrich A. von Hayek" while
//! "Hayek" finds him. The matcher then checks the full names.

use std::time::Duration;

use serde::Deserialize;
use shelfmark_core::error::{Error, ErrorCode};

use crate::cache::DiskCache;
use crate::matching::family_name;
use crate::pacer::Pacer;
use crate::transport::{Transport, TransportError};

/// The search endpoint.
pub const SEARCH_URL: &str = "https://openlibrary.org/search.json";
/// The fields a query asks for. `lcc` is the work's LC call numbers in
/// Open Library's padded sort form; `lc_classifications` is the raw form
/// edition records carry, read when present. `editions.title` is the
/// title of the edition that matched the query, which differs from the
/// work's when the work is filed under its original-language title.
const FIELDS: &str =
    "key,title,subtitle,author_name,lcc,ddc,lc_classifications,editions,editions.key,editions.title";

/// What to look up.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    pub title: String,
    pub authors: Vec<String>,
}

/// One search result (a work), with the fields the tier reads.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Doc {
    pub key: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub author_name: Vec<String>,
    /// LC call numbers in sort form, e.g. `PR-6039.00000000.O32 H6 1988`.
    #[serde(default)]
    pub lcc: Vec<String>,
    /// Raw LC call numbers, e.g. `PR6039.O32 H6 1988`.
    #[serde(default)]
    pub lc_classifications: Vec<String>,
    /// Dewey numbers; read for a future DDC scheme, unused today.
    #[serde(default)]
    pub ddc: Vec<String>,
    /// The editions that matched the query (the best one first).
    #[serde(default)]
    pub editions: Option<Editions>,
}

impl Doc {
    /// Titles of the editions that matched the query.
    pub fn edition_titles(&self) -> impl Iterator<Item = &str> {
        self.editions
            .iter()
            .flat_map(|e| e.docs.iter())
            .map(|e| e.title.as_str())
            .filter(|t| !t.trim().is_empty())
    }
}

/// The `editions` block of a search result.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Editions {
    #[serde(default)]
    pub docs: Vec<Edition>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Edition {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub title: String,
}

#[derive(Deserialize)]
struct Page {
    #[serde(default)]
    docs: Vec<Doc>,
}

/// Request settings. The defaults follow Open Library's guidance for API
/// clients: identify yourself, and no more than one request a second.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientConfig {
    pub base_url: String,
    /// Results asked for per query.
    pub limit: u32,
    /// Per-request timeout.
    pub timeout: Duration,
    /// Retries after the first attempt, on timeouts, network errors, 429
    /// and 5xx.
    pub max_retries: u32,
    /// Wait before the first retry; doubled for each later one.
    pub backoff: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            base_url: SEARCH_URL.into(),
            limit: 10,
            timeout: Duration::from_secs(10),
            max_retries: 2,
            backoff: Duration::from_secs(2),
        }
    }
}

/// Searches through a transport, politely.
pub struct Client {
    transport: Box<dyn Transport>,
    cache: Option<DiskCache>,
    pacer: Pacer,
    config: ClientConfig,
}

impl Client {
    /// A client pacing at one request a second.
    pub fn new(transport: Box<dyn Transport>, cache: Option<DiskCache>) -> Self {
        Client::with(
            transport,
            cache,
            Pacer::new(Duration::from_secs(1)),
            ClientConfig::default(),
        )
    }

    pub fn with(
        transport: Box<dyn Transport>,
        cache: Option<DiskCache>,
        pacer: Pacer,
        config: ClientConfig,
    ) -> Self {
        Client {
            transport,
            cache,
            pacer,
            config,
        }
    }

    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// The search URL for `query`.
    pub fn url(&self, query: &Query) -> String {
        let mut url = format!(
            "{}?title={}",
            self.config.base_url,
            encode(query.title.trim())
        );
        if let Some(a) = query.authors.iter().find_map(|a| family_name(a)) {
            url.push_str("&author=");
            url.push_str(&encode(&a));
        }
        url.push_str(&format!(
            "&fields={}&limit={}",
            encode(FIELDS),
            self.config.limit
        ));
        url
    }

    /// The works matching `query`, from the cache when it has them.
    /// Errors are `tier_unavailable`, so a cascade carries on without
    /// this tier when Open Library cannot be reached.
    pub fn search(&self, query: &Query) -> Result<Vec<Doc>, Error> {
        let url = self.url(query);
        if let Some(body) = self.cache.as_ref().and_then(|c| c.get(&url)) {
            if let Ok(page) = serde_json::from_str::<Page>(&body) {
                return Ok(page.docs);
            }
        }
        let body = self.fetch(&url)?;
        let page: Page = serde_json::from_str(&body).map_err(|e| {
            unavailable(
                &url,
                format!("Open Library sent a body that is not a search page: {e}"),
            )
        })?;
        if let Some(c) = &self.cache {
            c.put(&url, &body);
        }
        Ok(page.docs)
    }

    /// GET with pacing and bounded retries; the body of a 200.
    fn fetch(&self, url: &str) -> Result<String, Error> {
        let mut last = String::new();
        for attempt in 0..=self.config.max_retries {
            if attempt == 0 {
                self.pacer.wait();
            } else {
                self.pacer
                    .wait_at_least(self.config.backoff * 2u32.saturating_pow(attempt - 1));
            }
            match self.transport.get(url, self.config.timeout) {
                Ok(r) if r.status == 200 => return Ok(r.body),
                Ok(r) if r.status == 429 || r.status >= 500 => {
                    last = format!("HTTP {}", r.status);
                }
                Ok(r) => {
                    return Err(unavailable(
                        url,
                        format!("Open Library answered HTTP {}", r.status),
                    ))
                }
                Err(e @ (TransportError::Timeout | TransportError::Network(_))) => {
                    last = e.to_string();
                }
            }
        }
        Err(unavailable(
            url,
            format!(
                "Open Library did not answer after {} attempts: {last}",
                self.config.max_retries + 1
            ),
        ))
    }
}

fn unavailable(url: &str, message: String) -> Error {
    Error::new(ErrorCode::TierUnavailable, message)
        .next("shelfmark classify --profile lexical <file>")
        .details(serde_json::json!({ "tier": crate::NAME, "url": url }))
}

/// Percent-encode a query value (RFC 3986 unreserved characters pass).
fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}
