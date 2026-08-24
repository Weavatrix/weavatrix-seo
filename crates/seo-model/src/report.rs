//! Audit and opportunity reports.

use crate::{Finding, Inventory};
use serde::{Deserialize, Serialize};

/// Named axis. Never collapsed into one opaque SEO score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxisScore {
    /// Axis name.
    pub axis: String,
    /// Finding counts at error severity.
    pub errors: usize,
    /// Finding counts at warn severity.
    pub warnings: usize,
    /// Finding counts at info severity.
    pub infos: usize,
    /// True when this axis was not measured.
    pub unmeasured: bool,
}

/// Gap or construction opportunity. Distinct from a current defect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opportunity {
    /// Stable identity.
    pub id: String,
    /// Kind of gap.
    pub kind: String,
    /// What is missing.
    pub summary: String,
    /// Why it matters.
    pub why: String,
    /// Suggested construction.
    pub action: String,
    /// Subject URL or family.
    pub subject: String,
    /// Whether demand data was measured.
    pub demand: String,
}

impl Opportunity {
    /// Builds an opportunity whose demand is explicitly unmeasured.
    #[must_use]
    pub fn unmeasured_demand(
        kind: impl Into<String>,
        subject: impl Into<String>,
        summary: impl Into<String>,
        why: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        let kind = kind.into();
        let subject = subject.into();
        let id = format!(
            "WVX-SEO-OPP-{}:{}",
            kind.to_ascii_uppercase(),
            crate::ContentHash::of_str(&subject).short()
        );
        Self {
            id,
            kind,
            summary: summary.into(),
            why: why.into(),
            action: action.into(),
            subject,
            demand: "UNMEASURED".into(),
        }
    }
}

/// Full audit output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditReport {
    /// Inventory the audit was computed from.
    pub inventory: Inventory,
    /// Findings, highest severity first.
    pub findings: Vec<Finding>,
    /// Per-axis counts.
    pub axes: Vec<AxisScore>,
    /// Opportunities discovered in the same pass.
    pub opportunities: Vec<Opportunity>,
}

impl AuditReport {
    /// Finding by fingerprint or code.
    #[must_use]
    pub fn finding(&self, id: &str) -> Option<&Finding> {
        self.findings
            .iter()
            .find(|finding| finding.fingerprint == id || finding.code == id)
    }
}
