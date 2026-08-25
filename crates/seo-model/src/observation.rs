//! Fetch observations. Failures stay in the graph.

use crate::Evidence;
use serde::{Deserialize, Serialize};

/// Outcome of one HTTP attempt. Missing evidence is never a silent pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchOutcome {
    /// Final HTTP response recorded.
    Response,
    /// Connect or read timeout.
    Timeout,
    /// DNS lookup failed.
    Dns,
    /// TLS handshake failed.
    Tls,
    /// Body or header exceeded the budget.
    BodyLimit,
    /// Response could not be parsed.
    ParseFailure,
    /// Blocked by robots.txt.
    RobotsBlocked,
    /// Blocked by the network policy (SSRF).
    Blocked,
    /// Other transport failure.
    Transport,
}

/// One recorded fetch attempt, success or failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchObservation {
    /// Requested URL.
    pub url: String,
    /// Outcome.
    pub outcome: FetchOutcome,
    /// HTTP status when a response was received.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// Transport or policy message.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
    /// Evidence for the observation.
    pub evidence: Evidence,
}

impl FetchObservation {
    /// Builds an observation bound to a crawl snapshot later via [`crate::Inventory::bind_run`].
    #[must_use]
    pub fn new(url: impl Into<String>, outcome: FetchOutcome, message: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            outcome,
            status: None,
            message: message.into(),
            evidence: Evidence::http(),
        }
    }
}
