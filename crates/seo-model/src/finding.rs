//! Audit findings with stable fingerprints.

use crate::{ContentHash, Evidence, Locator};
use serde::{Deserialize, Serialize};

/// Gate-facing severity. Folklore length limits are never `error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Missing optional axis or display hint.
    Info,
    /// Visible problem that does not by itself block indexation.
    Warn,
    /// Blocks intended discovery, indexation, or publishes a contradiction.
    Error,
}

/// Catalogue family. Codes stay stable across releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FindingFamily {
    /// Crawl and discovery.
    Crawl,
    /// Indexability.
    Idx,
    /// Canonical graph.
    Canon,
    /// Sitemap integrity.
    Sitemap,
    /// Locale / hreflang.
    I18n,
    /// Raw versus rendered drift.
    Render,
    /// Titles and descriptions.
    Meta,
    /// Structured data.
    Schema,
    /// Internal links.
    Link,
    /// Duplication.
    Dup,
    /// Cannibalization.
    Cann,
    /// Content coverage.
    Content,
    /// Entity / topical graph.
    Entity,
    /// Market / jurisdiction.
    Market,
    /// Public claim integrity.
    Claim,
    /// Programmatic SEO.
    Prog,
    /// Performance.
    Perf,
    /// Local SEO.
    Local,
    /// AI-search readiness.
    Ai,
    /// Imported observations.
    Obs,
    /// Competitive gaps.
    Comp,
}

impl FindingFamily {
    /// Catalogue prefix, for example `WVX-SEO-CRAWL`.
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Crawl => "WVX-SEO-CRAWL",
            Self::Idx => "WVX-SEO-IDX",
            Self::Canon => "WVX-SEO-CANON",
            Self::Sitemap => "WVX-SEO-SITEMAP",
            Self::I18n => "WVX-SEO-I18N",
            Self::Render => "WVX-SEO-RENDER",
            Self::Meta => "WVX-SEO-META",
            Self::Schema => "WVX-SEO-SCHEMA",
            Self::Link => "WVX-SEO-LINK",
            Self::Dup => "WVX-SEO-DUP",
            Self::Cann => "WVX-SEO-CANN",
            Self::Content => "WVX-SEO-CONTENT",
            Self::Entity => "WVX-SEO-ENTITY",
            Self::Market => "WVX-SEO-MARKET",
            Self::Claim => "WVX-SEO-CLAIM",
            Self::Prog => "WVX-SEO-PROG",
            Self::Perf => "WVX-SEO-PERF",
            Self::Local => "WVX-SEO-LOCAL",
            Self::Ai => "WVX-SEO-AI",
            Self::Obs => "WVX-SEO-OBS",
            Self::Comp => "WVX-SEO-COMP",
        }
    }
}

/// One audit finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Catalogue identity such as `WVX-SEO-CRAWL-001`.
    pub code: String,
    /// Stable fingerprint `CODE:hash`.
    pub fingerprint: String,
    /// Family.
    pub family: FindingFamily,
    /// Gate severity.
    pub severity: Severity,
    /// Short explanation. Not a score.
    pub summary: String,
    /// Why the finding exists.
    pub why: String,
    /// Suggested next action.
    pub action: String,
    /// How to verify a fix.
    pub verification: String,
    /// Primary locator.
    pub locator: Locator,
    /// Additional affected URLs.
    pub affected_urls: Vec<String>,
    /// Evidence used to emit the finding.
    pub evidence: Evidence,
}

impl Finding {
    /// Builds a finding and fingerprints it from code + subject key.
    #[must_use]
    pub fn new(
        family: FindingFamily,
        number: u16,
        severity: Severity,
        subject: &str,
        summary: impl Into<String>,
        locator: Locator,
        evidence: Evidence,
    ) -> Self {
        let code = format!("{}-{number:03}", family.prefix());
        let fingerprint = format!("{code}:{}", ContentHash::of_str(subject).short());
        Self {
            code,
            fingerprint,
            family,
            severity,
            summary: summary.into(),
            why: String::new(),
            action: String::new(),
            verification: String::new(),
            locator,
            affected_urls: Vec::new(),
            evidence,
        }
    }

    /// Sets explanation fields.
    #[must_use]
    pub fn explained(
        mut self,
        why: impl Into<String>,
        action: impl Into<String>,
        verification: impl Into<String>,
    ) -> Self {
        self.why = why.into();
        self.action = action.into();
        self.verification = verification.into();
        self
    }

    /// Adds affected URLs, skipping duplicates of the primary locator URL.
    #[must_use]
    pub fn with_affected(mut self, urls: impl IntoIterator<Item = String>) -> Self {
        self.affected_urls.extend(urls);
        self.affected_urls.sort();
        self.affected_urls.dedup();
        self
    }
}
