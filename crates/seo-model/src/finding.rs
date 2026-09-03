//! Audit findings with stable fingerprints.

use crate::{ContentHash, Evidence, Locator, RuleAuthority};
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
    /// Accessibility of the live document.
    A11y,
    /// Response security headers.
    Security,
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
            Self::A11y => "WVX-SEO-A11Y",
            Self::Security => "WVX-SEO-SEC",
            Self::Local => "WVX-SEO-LOCAL",
            Self::Ai => "WVX-SEO-AI",
            Self::Obs => "WVX-SEO-OBS",
            Self::Comp => "WVX-SEO-COMP",
        }
    }

    /// Every catalogue family, in a stable order for the semantics digest.
    pub const ALL: [Self; 23] = [
        Self::Crawl,
        Self::Idx,
        Self::Canon,
        Self::Sitemap,
        Self::I18n,
        Self::Render,
        Self::Meta,
        Self::Schema,
        Self::Link,
        Self::Dup,
        Self::Cann,
        Self::Content,
        Self::Entity,
        Self::Market,
        Self::Claim,
        Self::Prog,
        Self::Perf,
        Self::A11y,
        Self::Security,
        Self::Local,
        Self::Ai,
        Self::Obs,
        Self::Comp,
    ];
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
    /// Why the rule is legitimate. Distinct from evidence kind.
    #[serde(default)]
    pub authority: RuleAuthority,
    /// Present only when the emitter overrode the registry default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity_override: Option<Severity>,
}

impl Finding {
    /// Builds a finding from the registry. Severity and authority come from the catalogue.
    #[must_use]
    pub fn from_rule(
        family: FindingFamily,
        number: u16,
        subject: &str,
        summary: impl Into<String>,
        locator: Locator,
        evidence: Evidence,
    ) -> Self {
        let severity = crate::registry::lookup(family, number).map_or_else(
            || family_fallback_severity(family),
            |rule| rule.default_severity,
        );
        Self::build(
            family, number, severity, None, subject, summary, locator, evidence,
        )
    }

    /// Builds a finding and fingerprints it from code + subject key.
    ///
    /// When `severity` disagrees with the registry, it is recorded as
    /// [`Self::severity_override`].
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
        let registered = crate::registry::lookup(family, number).map(|rule| rule.default_severity);
        let override_sev = registered.filter(|default| *default != severity);
        Self::build(
            family,
            number,
            severity,
            override_sev,
            subject,
            summary,
            locator,
            evidence,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        family: FindingFamily,
        number: u16,
        severity: Severity,
        severity_override: Option<Severity>,
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
            authority: crate::registry::authority(family, number),
            severity_override,
        }
    }

    /// Explicit severity that is not the registry default. Serialized on the finding.
    #[must_use]
    pub fn with_severity_override(mut self, severity: Severity) -> Self {
        self.severity_override = Some(severity);
        self.severity = severity;
        self
    }

    /// Overrides catalogue authority for a project-specific contract.
    #[must_use]
    pub fn with_authority(mut self, authority: RuleAuthority) -> Self {
        self.authority = authority;
        self
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

const fn family_fallback_severity(family: FindingFamily) -> Severity {
    match family {
        FindingFamily::Claim | FindingFamily::Idx | FindingFamily::Crawl => Severity::Error,
        FindingFamily::Ai | FindingFamily::Obs | FindingFamily::Comp => Severity::Info,
        _ => Severity::Warn,
    }
}
