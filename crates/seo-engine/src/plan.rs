//! Architecture compiler: opportunities become machine-checkable actions.

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{AuditReport, Opportunity, OpportunityAxes};
use weavatrix_seo_programmatic::SafetyVerdict;

/// Plan verb. Weavatrix SEO stays read-only; mutations belong elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlanKind {
    /// Create a missing URL or family.
    Create,
    /// Improve an existing URL.
    Improve,
    /// Merge overlapping URLs.
    Consolidate,
    /// Add an internal link.
    Link,
    /// Keep for users, drop from the index.
    Noindex,
    /// Remove a URL from the intended search surface.
    Delete,
}

impl std::fmt::Display for PlanKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Create => "CREATE",
            Self::Improve => "IMPROVE",
            Self::Consolidate => "CONSOLIDATE",
            Self::Link => "LINK",
            Self::Noindex => "NOINDEX",
            Self::Delete => "DELETE",
        })
    }
}

/// One construction action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanAction {
    /// Verb.
    pub kind: PlanKind,
    /// Subject URL or family.
    pub subject: String,
    /// Why this action exists.
    pub why: String,
    /// Finding fingerprints or opportunity ids.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
    /// Other subjects that should happen first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    /// Source implementation location when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_location: Option<String>,
    /// Domain facts required before shipping.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_facts: Vec<String>,
    /// Internal-link placements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link_placements: Vec<String>,
    /// Schema types that must exist.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schema_requirements: Vec<String>,
    /// Programmatic safety verdict when this is a matrix family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub programmatic_verdict: Option<String>,
    /// Acceptance condition.
    pub acceptance: String,
    /// How to verify.
    pub verification: String,
    /// Priority axes copied from the opportunity.
    #[serde(default)]
    pub axes: OpportunityAxes,
}

/// Machine-checkable search architecture plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchPlan {
    /// Ordered actions.
    pub actions: Vec<PlanAction>,
}

/// Compiles a plan from the current report. Does not draft copy or mutate source.
#[must_use]
pub fn plan_from(report: &AuditReport) -> SearchPlan {
    let actions = report
        .opportunities
        .iter()
        .map(|item| from_opportunity(item, report))
        .collect();
    SearchPlan { actions }
}

fn from_opportunity(item: &Opportunity, report: &AuditReport) -> PlanAction {
    let kind = match item.kind.as_str() {
        "link_gap" | "link_rec" => PlanKind::Link,
        "cannibal" | "duplicate" => PlanKind::Consolidate,
        "create_family" => PlanKind::Create,
        "noindex" => PlanKind::Noindex,
        "delete" => PlanKind::Delete,
        _ => PlanKind::Improve,
    };
    let evidence: Vec<String> = report
        .findings
        .iter()
        .filter(|finding| {
            finding.locator.subject_url() == item.subject
                || finding.affected_urls.iter().any(|url| url == &item.subject)
        })
        .map(|finding| finding.fingerprint.clone())
        .take(8)
        .collect();
    let source_location = report.inventory.facts.iter().find_map(|fact| {
        if fact.source.contains(&item.subject) || fact.target.contains(&item.subject) {
            fact.locator
                .as_ref()
                .map(|locator| locator.subject_url().to_owned())
        } else {
            None
        }
    });
    let programmatic_verdict = programmatic_for(&item.subject);
    let (required_facts, schema_requirements, link_placements) = extras(kind, item);
    PlanAction {
        kind,
        subject: item.subject.clone(),
        why: item.why.clone(),
        evidence,
        dependencies: Vec::new(),
        source_location,
        required_facts,
        link_placements,
        schema_requirements,
        programmatic_verdict,
        acceptance: item.action.clone(),
        verification: match kind {
            PlanKind::Link => "The source HTML contains a crawlable link to the target.".into(),
            PlanKind::Consolidate => {
                "One indexable URL remains for this intent, or each URL has distinct facts.".into()
            }
            PlanKind::Create => "A live URL matches the predicted family.".into(),
            PlanKind::Noindex => "The URL returns noindex or is omitted from the sitemap.".into(),
            PlanKind::Delete => "The URL is gone from sitemap and internal links.".into(),
            PlanKind::Improve => "The listed acceptance condition is true on a new crawl.".into(),
        },
        axes: item.axes.clone(),
    }
}

fn extras(kind: PlanKind, item: &Opportunity) -> (Vec<String>, Vec<String>, Vec<String>) {
    match kind {
        PlanKind::Link => (Vec::new(), Vec::new(), vec![item.action.clone()]),
        PlanKind::Improve if item.kind == "content_gap" => (Vec::new(), Vec::new(), Vec::new()),
        PlanKind::Create => (
            vec!["unique local facts per combination".into()],
            vec!["Service".into()],
            Vec::new(),
        ),
        _ => (Vec::new(), Vec::new(), Vec::new()),
    }
}

fn programmatic_for(subject: &str) -> Option<String> {
    if subject.contains(":city") || subject.contains("category/") {
        Some(format!("{:?}", SafetyVerdict::Review))
    } else {
        None
    }
}
