//! Audit orchestration for site, repo, hybrid, and compare modes.

use crate::report::assemble;
use crate::request::{
    budget, crawl_site, empty_repo_inventory, read_revision, request_config_digest, AuditRequest,
    EngineError,
};
use weavatrix_seo_model::{AnalysisMode, AuditReport, Finding, Inventory};
use weavatrix_seo_nextjs::predict;

/// Runs inventory + audit + opportunities for a request.
///
/// # Errors
///
/// Returns [`EngineError`] when both site and repo are missing, or a crawl fails.
pub fn run_audit(request: &AuditRequest) -> Result<AuditReport, EngineError> {
    if request.site.is_none() && request.repo.is_none() {
        return Err(EngineError::Usage(
            "provide --site URL and/or --repo PATH".into(),
        ));
    }
    let budget = budget(request);
    let surface = request.repo.as_deref().map(predict);
    let mut inventory = if let Some(site) = request.site.as_deref() {
        crawl_site(site, &budget)?
    } else {
        empty_repo_inventory(request)
    };
    inventory.mode = request.mode;
    inventory.repo.clone_from(&request.repo);
    inventory.config_digest = request_config_digest(request);
    if let Some(repo) = request.repo.as_deref() {
        inventory.repo_revision = read_revision(repo);
        if let Some(revision) = &inventory.repo_revision {
            for page in &mut inventory.pages {
                page.evidence.revision = Some(revision.clone());
            }
        }
    }
    if let Some(surface) = &surface {
        inventory.predicted_routes = surface.patterns();
    }
    let mut competitor_inventories = Vec::new();
    if request.mode == AnalysisMode::Compare {
        let public = budget.clone().public_only();
        for origin in &request.competitors {
            competitor_inventories.push((origin.clone(), crawl_site(origin, &public)?));
        }
    }
    Ok(assemble(
        request,
        inventory,
        surface.as_ref(),
        &competitor_inventories,
    ))
}

/// Inventory-only convenience.
///
/// # Errors
///
/// Propagates [`run_audit`].
pub fn run_inventory(request: &AuditRequest) -> Result<Inventory, EngineError> {
    Ok(run_audit(request)?.inventory)
}

/// Explains one finding.
#[must_use]
pub fn explain<'a>(report: &'a AuditReport, id: &str) -> Option<&'a Finding> {
    report.finding(id)
}
