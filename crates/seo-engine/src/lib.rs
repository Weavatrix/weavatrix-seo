//! Weavatrix SEO engine: inventory, audit, explain, opportunities, plan.

#![forbid(unsafe_code)]

mod html;
mod plan;
mod run;
mod text;

pub use html::render_html;
pub use plan::{PlanAction, SearchPlan, plan_from};
pub use run::{AuditRequest, EngineError, explain, run_audit, run_inventory};
pub use text::render_text;
pub use weavatrix_seo_model::{
    AnalysisMode, AuditReport, Finding, Inventory, Opportunity, Severity,
};
