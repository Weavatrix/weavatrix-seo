//! Audit orchestration for site, repo, hybrid, and compare modes.

use crate::report::assemble;
use crate::request::{
    AuditRequest, EngineError, budget, crawl_site, crawl_site_with_seeds, empty_repo_inventory,
    read_revision, request_config_digest,
};
use crate::seeds::directed_seeds;
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
        let host = weavatrix_seo_model::AbsoluteUrl::parse(site)
            .ok()
            .map(|url| url.host().to_owned());
        let extra = host
            .as_deref()
            .map(|host| directed_seeds(request, host))
            .unwrap_or_default();
        crawl_site_with_seeds(site, &budget, &extra)?
    } else {
        empty_repo_inventory(request)
    };
    inventory.mode = request.mode;
    inventory.repo.clone_from(&request.repo);
    inventory.config_digest = request_config_digest(request);
    if let Some(repo) = request.repo.as_deref() {
        // The worktree revision is the source side of the comparison, not the
        // provenance of a live response. Stamping it onto crawled pages would
        // claim production was built from this commit.
        inventory.repo_revision = read_revision(repo);
    }
    if let Some(surface) = &surface {
        inventory.predicted_routes = surface.patterns();
        if let Some(repo) = request.repo.as_deref() {
            inventory.producers = surface.producer_facts(repo);
            let loaded = weavatrix_seo_source::load_policy(repo);
            inventory.policy = loaded.policy;
            inventory.policy_error = loaded.error;
        }
    }
    let mut competitor_inventories = Vec::new();
    if request.mode == AnalysisMode::Compare {
        let public = budget.clone().public_only();
        for origin in &request.competitors {
            competitor_inventories.push((origin.clone(), crawl_site(origin, &public)?));
        }
    }
    let report = assemble(
        request,
        inventory,
        surface.as_ref(),
        &competitor_inventories,
    );
    if let Some(dir) = request.history.as_deref() {
        let _ = weavatrix_seo_history::save(dir, &report);
    }
    Ok(report)
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
