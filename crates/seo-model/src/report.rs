//! Audit and opportunity reports.

use crate::{Finding, Inventory, MAX_RISK, MIN_CONFIDENCE, SearchIntelligence};
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
    /// Raw search impressions when a provider supplied them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_impressions: Option<u32>,
    /// Raw clicks when a provider supplied them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_clicks: Option<u32>,
    /// Recoverable clicks estimated from expected CTR. Unmeasured when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recoverable_clicks: Option<u32>,
    /// External ranking difficulty when a provider supplied it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty_to_rank: Option<u16>,
    /// Difficulty of building the page truthfully from owned facts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty_to_build: Option<u16>,
    /// Expected CTR percent inferred from average position. Never exact truth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_ctr: Option<u16>,
    /// Conversion rate percent when a provider supplied it. Unmeasured when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversion_rate: Option<u16>,
    /// Display-only expected value: recoverable clicks × value × confidence / effort.
    ///
    /// Never consulted by [`Self::rank_key`]. Missing factors use a neutral 50
    /// so the number stays a display helper, not a hidden ranking score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_value: Option<u32>,
}

impl OpportunityAxes {
    /// Ordering key, highest first.
    ///
    /// This is lexicographic over the declared axes, never one opaque score:
    /// trust gates first, then measured demand, then value, and effort only
    /// breaks ties. An untrusted item sinks below everything else instead of
    /// being dropped, because it is still a real gap.
    #[must_use]
    pub fn rank_key(&self) -> (u8, u8, u16, u16, u16, u16, u16, u16, u16) {
        (
            self.trust_rank(),
            u8::from(self.demand.is_some()),
            self.demand.unwrap_or(0),
            self.visibility_gap.unwrap_or(0),
            self.business_value.unwrap_or(0),
            self.conversion_potential.unwrap_or(0),
            self.graph_leverage.unwrap_or(0),
            self.topical_fit.unwrap_or(0),
            self.cheapness(),
        )
    }

    /// Trust bucket used for ranking.
    ///
    /// Higher is better: measured-trusted, partially measured, unknown, then
    /// known-untrusted. Missing confidence/risk is **unknown**, never 100/0.
    #[must_use]
    pub fn trust_rank(&self) -> u8 {
        match (self.confidence, self.risk) {
            (Some(confidence), Some(risk)) if confidence >= MIN_CONFIDENCE && risk <= MAX_RISK => 3,
            (Some(confidence), None) if confidence >= MIN_CONFIDENCE => 2,
            (None, Some(risk)) if risk <= MAX_RISK => 2,
            (None, None) => 1,
            _ => 0,
        }
    }

    /// Whether this is confident enough and safe enough to act on first.
    ///
    /// An unscored axis is not a low score: a recommendation that never
    /// declared confidence is not thereby untrustworthy. A declared low
    /// confidence or high risk is.
    #[must_use]
    pub fn is_trusted(&self) -> bool {
        self.trust_rank() > 0
    }

    /// Cheaper work sorts first. Only a tie-breaker.
    #[must_use]
    pub fn cheapness(&self) -> u16 {
        100_u16.saturating_sub(self.implementation_cost.unwrap_or(0).min(100))
    }

    /// Display-only expected value. [`Self::rank_key`] never calls this.
    ///
    /// Returns `None` when recoverable clicks were not measured. Unmeasured
    /// value, confidence, and effort use 50 as a neutral display factor.
    #[must_use]
    #[allow(clippy::integer_division)]
    pub fn compute_expected_value(&self) -> Option<u32> {
        let clicks = self.recoverable_clicks?;
        let value = u32::from(self.business_value.unwrap_or(50).min(100));
        let confidence = u32::from(self.confidence.unwrap_or(50).min(100));
        let effort = u32::from(self.implementation_cost.unwrap_or(50).clamp(1, 100));
        Some(clicks.saturating_mul(value).saturating_mul(confidence) / effort)
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
    /// Programmatic safety verdict when this opportunity came from a page matrix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub programmatic_verdict: Option<String>,
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
            programmatic_verdict: None,
            axes: OpportunityAxes::default(),
        }
    }

    /// Attaches the page-matrix verdict this opportunity was compiled from.
    #[must_use]
    pub fn with_programmatic_verdict(mut self, verdict: impl Into<String>) -> Self {
        self.programmatic_verdict = Some(verdict.into());
        self
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
    /// Additive content, retrieval, outcome, and matrix intelligence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intelligence: Option<SearchIntelligence>,
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

#[cfg(test)]
mod tests {
    use super::OpportunityAxes;

    #[test]
    fn an_unscored_axis_is_not_a_low_score() {
        assert!(OpportunityAxes::default().is_trusted());
        assert_eq!(OpportunityAxes::default().trust_rank(), 1);
    }

    #[test]
    fn unknown_ranks_below_measured_trust_and_above_low_confidence() {
        let measured = OpportunityAxes {
            confidence: Some(80),
            risk: Some(10),
            ..OpportunityAxes::default()
        };
        let unknown = OpportunityAxes::default();
        let guessy = OpportunityAxes {
            demand: Some(100),
            confidence: Some(10),
            ..OpportunityAxes::default()
        };
        assert!(measured.rank_key() > unknown.rank_key());
        assert!(unknown.rank_key() > guessy.rank_key());
    }

    #[test]
    fn low_confidence_sinks_below_everything_trusted() {
        let trusted = OpportunityAxes {
            graph_leverage: Some(1),
            ..OpportunityAxes::default()
        };
        let guessy = OpportunityAxes {
            demand: Some(100),
            confidence: Some(10),
            ..OpportunityAxes::default()
        };
        assert!(!guessy.is_trusted());
        assert!(
            trusted.rank_key() > guessy.rank_key(),
            "a confident small win outranks an unreliable large one"
        );
    }

    #[test]
    fn high_risk_sinks_too() {
        let safe = OpportunityAxes::default();
        let risky = OpportunityAxes {
            demand: Some(100),
            risk: Some(90),
            ..OpportunityAxes::default()
        };
        assert!(!risky.is_trusted());
        assert!(safe.rank_key() > risky.rank_key());
    }

    #[test]
    fn effort_only_breaks_a_tie() {
        let cheap = OpportunityAxes {
            demand: Some(50),
            implementation_cost: Some(10),
            ..OpportunityAxes::default()
        };
        let costly = OpportunityAxes {
            demand: Some(50),
            implementation_cost: Some(90),
            ..OpportunityAxes::default()
        };
        let bigger_but_costly = OpportunityAxes {
            demand: Some(60),
            implementation_cost: Some(90),
            ..OpportunityAxes::default()
        };
        assert!(cheap.rank_key() > costly.rank_key());
        assert!(
            bigger_but_costly.rank_key() > cheap.rank_key(),
            "effort never outweighs measured demand"
        );
    }

    #[test]
    fn declared_value_axes_are_actually_used() {
        let valuable = OpportunityAxes {
            business_value: Some(90),
            ..OpportunityAxes::default()
        };
        assert!(valuable.rank_key() > OpportunityAxes::default().rank_key());
    }

    #[test]
    fn expected_value_is_display_only_and_never_ranks() {
        let low = OpportunityAxes {
            demand: Some(50),
            recoverable_clicks: Some(10),
            expected_value: Some(1),
            ..OpportunityAxes::default()
        };
        let high = OpportunityAxes {
            demand: Some(50),
            recoverable_clicks: Some(9_999),
            expected_value: Some(999_999),
            ..OpportunityAxes::default()
        };
        assert_eq!(
            low.rank_key(),
            high.rank_key(),
            "expected_value must not collapse prioritization"
        );
        let computed = OpportunityAxes {
            recoverable_clicks: Some(100),
            business_value: Some(80),
            confidence: Some(50),
            implementation_cost: Some(20),
            ..OpportunityAxes::default()
        };
        assert_eq!(computed.compute_expected_value(), Some(20_000));
    }
}
