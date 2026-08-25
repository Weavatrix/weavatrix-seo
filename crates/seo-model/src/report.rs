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

/// Priority axes for one opportunity. Missing values are unmeasured.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OpportunityAxes {
    /// Search demand (impressions-derived). `None` is unmeasured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub demand: Option<u16>,
    /// Current visibility gap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility_gap: Option<u16>,
    /// Business value when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_value: Option<u16>,
    /// Conversion potential when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversion_potential: Option<u16>,
    /// Topical fit to the owned graph.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topical_fit: Option<u16>,
    /// Internal-graph leverage (orphan, depth, cornerstone).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_leverage: Option<u16>,
    /// Confidence of the recommendation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u16>,
    /// Implementation cost. Higher is harder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation_cost: Option<u16>,
    /// Risk of the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<u16>,
}

impl OpportunityAxes {
    /// Sort key: measured demand first, then gap and leverage.
    #[must_use]
    pub fn rank_key(&self) -> (u8, u16, u16, u16) {
        (
            u8::from(self.demand.is_some()),
            self.demand.unwrap_or(0),
            self.visibility_gap.unwrap_or(0),
            self.graph_leverage.unwrap_or(0),
        )
    }
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
    /// Separate priority axes. Never collapsed into one SEO score.
    #[serde(default)]
    pub axes: OpportunityAxes,
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
            axes: OpportunityAxes::default(),
        }
    }

    /// Attaches priority axes.
    #[must_use]
    pub fn with_axes(mut self, axes: OpportunityAxes) -> Self {
        if let Some(demand) = axes.demand {
            self.demand = format!("impressions:{demand}");
        }
        self.axes = axes;
        self
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
