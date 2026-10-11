//! The network boundary. [`Client`](crate::Client) only ever calls
//! [`Transport::get`], so tests swap in [`FixtureTransport`] and never
//! open a socket.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

/// A completed HTTP exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

/// A request that did not complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// No response within the transport's timeout.
    Timeout,
    /// Connection, TLS or read failure.
    Network(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Timeout => f.write_str("timed out"),
            TransportError::Network(m) => write!(f, "network error: {m}"),
        }
    }
}

/// One HTTP GET. Implementations set their own timeouts and User-Agent.
pub trait Transport: Send + Sync {
    fn get(&self, url: &str, timeout: Duration) -> Result<Response, TransportError>;
}

/// Serves recorded bodies by exact URL and logs every request. A URL
/// with queued outcomes returns them in order before falling back to its
/// recorded body; an unknown URL is a 404.
#[derive(Debug, Default)]
pub struct FixtureTransport {
    bodies: BTreeMap<String, String>,
    queued: Mutex<BTreeMap<String, Vec<Result<Response, TransportError>>>>,
    log: Mutex<Vec<String>>,
}

impl FixtureTransport {
    pub fn new() -> Self {
        FixtureTransport::default()
    }

    /// Answer `url` with status 200 and `body`.
    pub fn with(mut self, url: impl Into<String>, body: impl Into<String>) -> Self {
        self.bodies.insert(url.into(), body.into());
        self
    }

    /// Answer the next request for `url` with `outcome` (then the next
    /// queued one, then the recorded body).
    pub fn queue(self, url: impl Into<String>, outcome: Result<Response, TransportError>) -> Self {
        self.queued
            .lock()
            .expect("fixture lock")
            .entry(url.into())
            .or_default()
            .push(outcome);
        self
    }

    /// Every URL requested so far, in order.
    pub fn requests(&self) -> Vec<String> {
        self.log.lock().expect("fixture lock").clone()
    }
}

impl Transport for FixtureTransport {
    fn get(&self, url: &str, _timeout: Duration) -> Result<Response, TransportError> {
        self.log.lock().expect("fixture lock").push(url.to_string());
        if let Some(q) = self.queued.lock().expect("fixture lock").get_mut(url) {
            if !q.is_empty() {
                return q.remove(0);
            }
        }
        Ok(match self.bodies.get(url) {
            Some(body) => Response {
                status: 200,
                body: body.clone(),
            },
            None => Response {
                status: 404,
                body: String::new(),
            },
        })
    }
}
