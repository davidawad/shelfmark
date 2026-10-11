//! The real transport: ureq over rustls (feature `http`).

use std::io::Read;
use std::time::Duration;

use crate::transport::{Response, Transport, TransportError};
use crate::USER_AGENT;

/// Largest response body read; a search page is a few kilobytes.
const MAX_BODY: u64 = 4 * 1024 * 1024;

/// HTTPS GETs with [`USER_AGENT`]. Non-2xx statuses come back as
/// responses, so the client decides what to retry.
pub struct UreqTransport {
    agent: ureq::Agent,
}

impl UreqTransport {
    pub fn new() -> Self {
        let config = ureq::Agent::config_builder()
            .user_agent(USER_AGENT)
            .http_status_as_error(false)
            .build();
        UreqTransport {
            agent: config.into(),
        }
    }
}

impl Default for UreqTransport {
    fn default() -> Self {
        UreqTransport::new()
    }
}

fn classify(e: &ureq::Error) -> TransportError {
    match e {
        ureq::Error::Timeout(_) => TransportError::Timeout,
        other => TransportError::Network(other.to_string()),
    }
}

impl Transport for UreqTransport {
    fn get(&self, url: &str, timeout: Duration) -> Result<Response, TransportError> {
        let mut resp = self
            .agent
            .get(url)
            .header("Accept", "application/json")
            .config()
            .timeout_global(Some(timeout))
            .build()
            .call()
            .map_err(|e| classify(&e))?;
        let status = resp.status().as_u16();
        let mut body = String::new();
        resp.body_mut()
            .as_reader()
            .take(MAX_BODY)
            .read_to_string(&mut body)
            .map_err(|e| TransportError::Network(e.to_string()))?;
        Ok(Response { status, body })
    }
}
