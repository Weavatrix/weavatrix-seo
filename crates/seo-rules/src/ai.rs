//! AI-crawler surface: `/llms.txt` and robots groups for known agents.

use weavatrix_seo_model::{
    Evidence, Finding, FindingFamily, Inventory, Locator, RuleAuthority, Severity,
};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let Some(surface) = &inventory.ai_surface else {
        return;
    };
    if surface.llms_txt_status == Some(404) {
        let host = inventory
            .site
            .clone()
            .unwrap_or_else(|| inventory.hosts.first().cloned().unwrap_or_default());
        findings.push(
            Finding::new(
                FindingFamily::Ai,
                4,
                Severity::Info,
                &host,
                format!("{host} has no /llms.txt"),
                Locator::Url(format!("{}/llms.txt", host.trim_end_matches('/'))),
                Evidence::http(),
            )
            .with_authority(RuleAuthority::ExperimentalHeuristic)
            .explained(
                "llms.txt is an emerging AI-crawler hint, not a search-engine requirement.",
                "Publish /llms.txt only when the origin wants to describe itself to AI fetchers.",
                "GET /llms.txt returns 200, or the origin intentionally has none.",
            ),
        );
    }
    for agent in &surface.robots_disallow_all {
        let host = inventory
            .site
            .clone()
            .unwrap_or_else(|| inventory.hosts.first().cloned().unwrap_or_default());
        findings.push(
            Finding::new(
                FindingFamily::Ai,
                5,
                Severity::Warn,
                &host,
                format!("{host} robots.txt disallows `{agent}` from the whole origin"),
                Locator::Url(format!("{}/robots.txt", host.trim_end_matches('/'))),
                Evidence::http(),
            )
            .explained(
                "Citation and search-index bots honour robots.txt. Disallow: / hides the site from them.",
                "Allow the agent on the public indexable surface, or keep the block if that is the policy.",
                "The agent group is not Disallow: / unless the origin intends to opt out.",
            ),
        );
    }
}
