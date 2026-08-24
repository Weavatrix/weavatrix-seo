//! Weavatrix SEO engine: inventory, audit, explain, opportunities, plan.

#![forbid(unsafe_code)]

mod axes;
mod html;
mod plan;
mod report;
mod request;
mod run;
mod source;
mod text;

pub use html::render_html;
pub use plan::{PlanAction, SearchPlan, plan_from};
pub use request::{AuditRequest, EngineError};
pub use run::{explain, run_audit, run_inventory};
pub use text::render_text;
pub use weavatrix_seo_gate::{GateVerdict, evaluate as evaluate_gate, load_fingerprints};
pub use weavatrix_seo_model::{
    AnalysisMode, AuditReport, Finding, Inventory, Opportunity, Severity,
};
