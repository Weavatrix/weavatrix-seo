//! Weavatrix SEO engine: inventory, audit, explain, opportunities, plan.

#![forbid(unsafe_code)]

mod axes;
mod diff;
mod explain;
mod graph;
mod html;
mod observe;
mod plan;
mod query;
mod report;
mod request;
mod retrieve;
mod run;
mod seeds;
mod source;
mod text;

pub use diff::diff_paths;
pub use explain::{ExplainHop, Explanation, explain_chain};
pub use html::render_html;
pub use plan::{PlanAction, PlanKind, SearchPlan, plan_from};
pub use query::{Query, QueryResult, parse as parse_query, run as run_query, run_on_report};
pub use request::{AuditRequest, EngineError};
pub use retrieve::{chunks_for, retrieve, similar};
pub use run::{explain, run_audit, run_inventory};
pub use text::render_text;
pub use weavatrix_seo_gate::{
    Baseline, GateVerdict, evaluate as evaluate_gate, from_report as baseline_from_report,
    load_baseline, load_fingerprints,
};
pub use weavatrix_seo_history::{SearchDiff, load as load_snapshot, save as save_history};
pub use weavatrix_seo_model::{
    AnalysisMode, AuditReport, Finding, Inventory, Opportunity, Severity,
};
pub use weavatrix_seo_semantic::{LinkInputs, PageRow, VectorRow};

/// Deterministic page vectors and SEO link profiles for an audited report.
///
/// The model is first-party and lexical, so nothing external is needed to
/// produce these. Node identities are `page:<url>`.
#[must_use]
pub fn link_inputs(report: &AuditReport) -> LinkInputs {
    let (architecture, _) = weavatrix_seo_architecture::analyze(&report.inventory);
    weavatrix_seo_semantic::link_inputs(&report.inventory, &architecture)
}
