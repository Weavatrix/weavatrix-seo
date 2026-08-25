//! Weavatrix SEO engine: inventory, audit, explain, opportunities, plan.

#![forbid(unsafe_code)]

mod axes;
mod graph;
mod html;
mod observe;
mod plan;
mod report;
mod request;
mod run;
mod source;
mod text;

pub use html::render_html;
pub use plan::{PlanAction, PlanKind, SearchPlan, plan_from};
pub use request::{AuditRequest, EngineError};
pub use run::{explain, run_audit, run_inventory};
pub use text::render_text;
pub use weavatrix_seo_gate::{
    Baseline, GateVerdict, evaluate as evaluate_gate, from_report as baseline_from_report,
    load_baseline, load_fingerprints,
};
pub use weavatrix_seo_model::{
    AnalysisMode, AuditReport, Finding, Inventory, Opportunity, Severity,
};
