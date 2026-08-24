//! Target architecture sketch from opportunities.

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::Opportunity;

/// One construction action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanAction {
    /// Create, improve, consolidate, or link.
    pub kind: String,
    /// Subject URL or family.
    pub subject: String,
    /// Acceptance condition.
    pub acceptance: String,
}

/// Machine-checkable plan stub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchPlan {
    /// Ordered actions.
    pub actions: Vec<PlanAction>,
}

/// Builds a plan from opportunities. Does not draft copy.
#[must_use]
pub fn plan_from(opportunities: &[Opportunity]) -> SearchPlan {
    let actions = opportunities
        .iter()
        .map(|item| PlanAction {
            kind: item.kind.clone(),
            subject: item.subject.clone(),
            acceptance: item.action.clone(),
        })
        .collect();
    SearchPlan { actions }
}
